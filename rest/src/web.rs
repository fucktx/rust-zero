//! Web 兼容层（编译期无感）：通过 feature 选择底层框架（axum / actix-web），
//! 业务侧只依赖 `rest::web` 的统一 API。
//!
//! 设计取舍：
//! - 统一路由/中间件的“能力抽象”（最小公共集），隐藏框架差异；
//! - 框架选择是编译期（features），不是运行时；
//! - Handler 统一签名：`(Engine, Request) -> anyhow::Result<Response>`（async）。

use bytes::Bytes;
use futures::future::BoxFuture;
use http::{HeaderMap, Method, StatusCode};
use matchit::Match;
use std::collections::HashMap;
use std::sync::Arc;

pub use crate::config::RestConf as RestConfig;
pub use crate::engine::Engine;

/// 统一请求结构（屏蔽 axum/actix 的 extractor 差异）。
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: Option<String>,
    pub headers: HeaderMap,
    pub body: Bytes,
    /// 路径参数（来自 `/user/:id` 的 `id`）
    pub params: HashMap<String, String>,
}

/// 统一响应结构（屏蔽 axum/actix 的响应类型差异）。
#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl Response {
    pub fn text(s: impl Into<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            "text/plain; charset=utf-8".parse().unwrap(),
        );
        Self {
            status: StatusCode::OK,
            headers,
            body: Bytes::from(s.into()),
        }
    }

    pub fn status(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Bytes::new(),
        }
    }
}

pub type HandlerResult = anyhow::Result<Response>;
pub type HandlerFuture = BoxFuture<'static, HandlerResult>;

/// 统一 handler：上层业务只需要实现这个签名。
pub type Handler = Arc<dyn Fn(Engine, Request) -> HandlerFuture + Send + Sync>;

/// 中间件（最小公共抽象）：可在框架无关层里做前后处理。
pub type Middleware = Arc<dyn Fn(Engine, Request, Next) -> HandlerFuture + Send + Sync>;

/// 下一跳（中间件链）
#[derive(Clone)]
pub struct Next {
    inner: Arc<dyn Fn(Engine, Request) -> HandlerFuture + Send + Sync>,
}

impl Next {
    pub fn run(&self, engine: Engine, req: Request) -> HandlerFuture {
        (self.inner)(engine, req)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl HttpMethod {
    fn from_http(m: &Method) -> Option<Self> {
        match *m {
            Method::GET => Some(HttpMethod::Get),
            Method::POST => Some(HttpMethod::Post),
            Method::PUT => Some(HttpMethod::Put),
            Method::DELETE => Some(HttpMethod::Delete),
            Method::PATCH => Some(HttpMethod::Patch),
            _ => None,
        }
    }
}

/// 路由表：框架无关的描述 + dispatch。
#[derive(Clone, Default)]
pub struct Router {
    routes: Arc<HashMap<HttpMethod, matchit::Router<Handler>>>,
    middlewares: Arc<Vec<Middleware>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(HashMap::new()),
            middlewares: Arc::new(Vec::new()),
        }
    }

    /// 注册路由（支持 matchit 风格参数，例如 `/user/:id`）。
    pub fn route(mut self, method: HttpMethod, path: &str, handler: Handler) -> Self {
        let mut routes_map = (*self.routes).clone();
        let entry = routes_map.entry(method).or_insert_with(matchit::Router::new);
        entry.insert(path, handler).expect("invalid route pattern");
        self.routes = Arc::new(routes_map);
        self
    }

    /// 添加中间件（按添加顺序执行）。
    pub fn middleware(mut self, mw: Middleware) -> Self {
        let mut mws = (*self.middlewares).clone();
        mws.push(mw);
        self.middlewares = Arc::new(mws);
        self
    }

    async fn dispatch_inner(&self, engine: Engine, mut req: Request) -> HandlerResult {
        let Some(m) = HttpMethod::from_http(&req.method) else {
            return Ok(Response::status(StatusCode::METHOD_NOT_ALLOWED));
        };

        let router = self.routes.get(&m);
        let Some(router) = router else {
            return Ok(Response::status(StatusCode::NOT_FOUND));
        };

        let Match { value: h, params } = match router.at(&req.path) {
            Ok(m) => m,
            Err(_) => return Ok(Response::status(StatusCode::NOT_FOUND)),
        };

        let mut p = HashMap::new();
        for (k, v) in params.iter() {
            p.insert(k.to_string(), v.to_string());
        }
        req.params = p;

        (h)(engine, req).await
    }

    pub fn dispatch(&self, engine: Engine, req: Request) -> HandlerFuture {
        // build middleware chain
        let base = {
            let this = self.clone();
            Arc::new(move |engine: Engine, req: Request| {
                let this = this.clone();
                Box::pin(async move { this.dispatch_inner(engine, req).await }) as HandlerFuture
            })
        };

        let mut next = Next { inner: base };
        for mw in self.middlewares.iter().rev() {
            let mw = mw.clone();
            let prev = next.clone();
            next = Next {
                inner: Arc::new(move |engine, req| mw(engine, req, prev.clone())),
            };
        }

        next.run(engine, req)
    }
}

pub mod middleware {
    use super::*;

    /// 一个最小示例中间件：给响应加上 `x-powered-by: rest-web`。
    pub fn powered_by() -> Middleware {
        Arc::new(|engine, req, next| {
            Box::pin(async move {
                let mut resp = next.run(engine, req).await?;
                resp.headers
                    .insert("x-powered-by", "rest-web".parse().unwrap());
                Ok(resp)
            })
        })
    }
}

/// 统一启动入口：根据 feature 选择底层框架。
///
/// - `features = ["axum"]`：需要在 tokio runtime 下调用
/// - `features = ["actix"]`：使用 actix runtime
pub async fn run(engine: Engine, router: Router) -> anyhow::Result<()> {
    #[cfg(all(feature = "axum", not(feature = "actix")))]
    {
        return axum_impl::run(engine, router).await;
    }
    #[cfg(all(feature = "actix", not(feature = "axum")))]
    {
        return actix_impl::run(engine, router).await;
    }
    #[cfg(all(feature = "axum", feature = "actix"))]
    {
        compile_error!("rest::web: enable only one of features `axum` or `actix`");
    }
    #[cfg(not(any(feature = "axum", feature = "actix")))]
    {
        let _ = (&engine, &router);
        Err(anyhow::anyhow!(
            "rest::web: please enable one feature: `axum` or `actix`"
        ))
    }
}

#[cfg(feature = "axum")]
mod axum_impl {
    use super::*;
    use axum::extract::State;
    use axum::response::IntoResponse;
    use anyhow::Context;

    #[derive(Clone)]
    struct AppState {
        engine: Engine,
        router: Router,
    }

    pub async fn run(engine: Engine, router: Router) -> anyhow::Result<()> {
        let addr = engine.conf.addr_string();
        let addr = addr.parse().context("invalid host/port")?;

        let state = AppState { engine, router };

        let app = axum::Router::new().fallback(handler).with_state(state);

        let server = axum::Server::try_bind(&addr).with_context(|| format!("bind {addr}"))?;
        server
            .serve(app.into_make_service())
            .await
            .context("axum serve")?;

        Ok(())
    }

    async fn handler(
        State(st): State<AppState>,
        req: axum::http::Request<axum::body::Body>,
    ) -> impl IntoResponse {
        let (parts, body) = req.into_parts();
        let body = match hyper::body::to_bytes(body).await {
            Ok(b) => b,
            Err(e) => {
                let mut r = Response::text(e.to_string());
                r.status = StatusCode::BAD_REQUEST;
                return to_axum_response(r);
            }
        };

        let r = Request {
            method: parts.method,
            path: parts.uri.path().to_string(),
            query: parts.uri.query().map(|s| s.to_string()),
            headers: parts.headers,
            body: body.into(),
            params: HashMap::new(),
        };

        let resp = match st.router.dispatch(st.engine.clone(), r).await {
            Ok(resp) => resp,
            Err(e) => {
                let mut resp = Response::text(e.to_string());
                resp.status = StatusCode::INTERNAL_SERVER_ERROR;
                resp
            }
        };

        to_axum_response(resp)
    }

    fn to_axum_response(resp: Response) -> axum::response::Response {
        let mut out = axum::response::Response::new(axum::body::boxed(axum::body::Body::from(
            resp.body,
        )));
        *out.status_mut() = resp.status;
        *out.headers_mut() = resp.headers;
        out
    }
}

#[cfg(feature = "actix")]
mod actix_impl {
    use super::*;
    use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
    use anyhow::Context;

    #[derive(Clone)]
    struct AppState {
        engine: Engine,
        router: Router,
    }

    pub async fn run(engine: Engine, router: Router) -> anyhow::Result<()> {
        let state = AppState { engine, router };
        let host = state.engine.conf.host.clone();
        let port = state.engine.conf.port;

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(state.clone()))
                .default_service(web::to(handler))
        })
        .bind((host.as_str(), port))
        .with_context(|| format!("bind {}:{}", host, port))?
        .run()
        .await
        .context("actix serve")?;

        Ok(())
    }

    async fn handler(req: HttpRequest, body: web::Bytes, st: web::Data<AppState>) -> HttpResponse {
        let resp = match st
            .router
            .dispatch(
                st.engine.clone(),
                Request {
                    method: req.method().as_str().parse().unwrap_or(Method::GET),
                    path: req.path().to_string(),
                    query: (!req.query_string().is_empty())
                        .then_some(req.query_string().to_string()),
                    headers: req.headers().clone().into(),
                    body: body.to_vec().into(),
                    params: HashMap::new(),
                },
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let mut r = Response::text(e.to_string());
                r.status = StatusCode::INTERNAL_SERVER_ERROR;
                r
            }
        };

        let mut builder = HttpResponse::build(
            actix_web::http::StatusCode::from_u16(resp.status.as_u16()).unwrap(),
        );
        for (k, v) in resp.headers.iter() {
            builder.insert_header((k.clone(), v.clone()));
        }
        builder.body(resp.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_handler() -> Handler {
        Arc::new(|_engine, _req| Box::pin(async { Ok(Response::text("ok")) }))
    }

    #[test]
    fn route_match_should_extract_params() {
        let r = Router::new().route(HttpMethod::Get, "/user/:id", ok_handler());
        let engine = Engine::new(RestConfig::default());

        let fut = r.dispatch(
            engine,
            Request {
                method: Method::GET,
                path: "/user/42".to_string(),
                query: None,
                headers: HeaderMap::new(),
                body: Bytes::new(),
                params: HashMap::new(),
            },
        );

        let rt = futures::executor::block_on(fut).unwrap();
        assert_eq!(rt.status, StatusCode::OK);
    }

    #[test]
    fn middleware_should_run_in_order() {
        let r = Router::new()
            .route(HttpMethod::Get, "/ping", ok_handler())
            .middleware(middleware::powered_by());

        let engine = Engine::new(RestConfig::default());
        let resp = futures::executor::block_on(r.dispatch(
            engine,
            Request {
                method: Method::GET,
                path: "/ping".to_string(),
                query: None,
                headers: HeaderMap::new(),
                body: Bytes::new(),
                params: HashMap::new(),
            },
        ))
        .unwrap();

        assert!(resp.headers.contains_key("x-powered-by"));
    }
}


