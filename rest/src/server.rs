//! Web 框架启动适配层（axum / actix-web ...）。
//!
//! 设计目标：
//! - `rest` crate 本身只提供“配置 + 运行入口”；
//! - 具体 Web 框架通过 feature 开关按需引入（避免强依赖）。

/// 通用错误类型：不同框架实现可能返回不同的错误，这里统一用 anyhow。
#[cfg(any(feature = "axum", feature = "actix"))]
pub type Result<T> = anyhow::Result<T>;

// ------------------------
// go-zero 风格 Server（axum）
// ------------------------
//
// 说明：
// - 这是一个“薄封装”：底层依然是 axum::Router + rest::native::run；
// - 用于让上层代码更接近 go-zero 的 `MustNewServer/AddRoutes/Start` 习惯；
// - 推荐配合 `rest::add_routes!` 使用（编译期展开，零运行时抽象成本）。

#[cfg(feature = "axum")]
pub struct Server<S = ()> {
    engine: crate::engine::Engine,
    router: ::axum::Router<S>,
}

#[cfg(feature = "axum")]
impl Server<()> {
    pub fn new<S>(conf: crate::RestConf) -> anyhow::Result<Server<S>>
    where
        S: Clone + Send + Sync + 'static,
    {
        conf.validate().map_err(anyhow::Error::msg)?;
        Ok(Server {
            engine: crate::engine::Engine::new(conf),
            router: ::axum::Router::<S>::new(),
        })
    }

    pub fn must_new<S>(conf: crate::RestConf) -> Server<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        Self::new::<S>(conf).unwrap()
    }
}

#[cfg(feature = "axum")]
impl<S> Server<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn with_router(self, router: ::axum::Router<S>) -> Self {
        Self { router, ..self }
    }

    pub fn add_routes(mut self, routes: ::axum::Router<S>) -> Self {
        self.router = self.router.merge(routes);
        self
    }

    /// 对齐 go-zero：AddRoute（单个 route 复用 AddRoutes）。
    pub fn add_route(self, route: ::axum::Router<S>) -> Self {
        self.add_routes(route)
    }

    /// 对齐 go-zero：Use（给所有 routes 增加 middleware）。
    ///
    /// 注意：axum 区分 `layer`（包含 fallback）与 `route_layer`（仅 routes）。
    /// go-zero 的 Use 语义更接近“作用于业务 routes”，因此这里用 `route_layer`。
    pub fn use_layer<L>(mut self, layer: L) -> Self
    where
        L: tower_layer::Layer<::axum::routing::Route> + Clone + Send + 'static,
        L::Service:
            tower_service::Service<http::Request<::axum::body::Body>> + Clone + Send + 'static,
        <L::Service as tower_service::Service<http::Request<::axum::body::Body>>>::Response:
            ::axum::response::IntoResponse + 'static,
        <L::Service as tower_service::Service<http::Request<::axum::body::Body>>>::Error:
            Into<std::convert::Infallible> + 'static,
        <L::Service as tower_service::Service<http::Request<::axum::body::Body>>>::Future:
            Send + 'static,
    {
        self.router = self.router.route_layer(layer);
        self
    }

    /// 对齐 go-zero：WithNotFoundHandler（设置 404 handler）。
    pub fn with_not_found_handler<H, T>(mut self, handler: H) -> Self
    where
        H: ::axum::handler::Handler<T, S> + Clone + Send + 'static,
        T: 'static,
    {
        self.router = self.router.fallback(handler);
        self
    }

    /// 固化 state，把 `Router<S>` 转为可直接运行的 `Router<()>`。
    pub fn with_state(self, state: S) -> Server<()> {
        Server {
            engine: self.engine,
            router: self.router.with_state::<()>(state),
        }
    }

    pub fn into_router(self) -> ::axum::Router<S> {
        self.router
    }

    pub fn conf(&self) -> &crate::RestConf {
        &self.engine.conf
    }
}

#[cfg(feature = "axum")]
impl Server<()> {
    pub async fn start(self) -> anyhow::Result<()> {
        let engine = self.engine;
        let router = engine.apply_defaults(self.router);
        crate::native::run(engine, router).await
    }
}

/// axum 适配：`cargo add rest --features axum`
#[cfg(feature = "axum")]
pub mod axum {
    use crate::engine::Engine;
    use anyhow::Context;
    use std::net::SocketAddr;

    /// 启动 axum 服务（需要在 tokio runtime 中调用）。
    ///
    /// 说明：axum 的 `Router<S>`（带状态类型）需要先 `.with_state(state)` 把状态“固化”后，
    /// 才能变成可直接 `into_make_service()` 的 `Router<()>`。
    pub async fn run(engine: Engine, app: axum::Router) -> anyhow::Result<()> {
        let addr: SocketAddr = engine
            .conf
            .addr_string()
            .parse()
            .context("invalid host/port")?;

        // try_bind: 端口占用时不 panic，返回错误
        let server = axum::Server::try_bind(&addr).with_context(|| format!("bind {addr}"))?;

        server
            .serve(app.into_make_service())
            .await
            .context("axum server run")?;

        Ok(())
    }
}

/// actix-web 适配：`cargo add rest --features actix`
#[cfg(feature = "actix")]
pub mod actix {
    use crate::engine::Engine;
    use actix_http::body::MessageBody;
    use actix_http::{Request, Response};
    use actix_service::IntoServiceFactory;
    use actix_service::{Service, ServiceFactory};
    use actix_web::dev::AppConfig;
    use actix_web::{Error, HttpServer};
    use std::io;

    /// 启动 actix-web 服务（使用 actix runtime）。
    ///
    /// 用法示例：
    /// ```no_run
    /// # use rest::{config::RestConf, engine::Engine, server};
    /// # #[cfg(feature="actix")]
    /// # async fn demo() -> std::io::Result<()> {
    /// let engine = Engine::new(RestConf::default());
    /// server::actix::run(engine, || actix_web::App::new()).await
    /// # }
    /// ```
    pub async fn run<F, I, S, B>(engine: Engine, factory: F) -> io::Result<()>
    where
        F: Fn() -> I + Send + Clone + 'static,
        I: IntoServiceFactory<S, Request>,
        S: ServiceFactory<Request, Config = AppConfig> + 'static,
        S::Error: Into<Error> + 'static,
        S::InitError: std::fmt::Debug,
        S::Response: Into<Response<B>> + 'static,
        <S::Service as Service<Request>>::Future: 'static,
        S::Service: 'static,
        B: MessageBody + 'static,
    {
        // 这里用 (host, port) 绑定，避免手动拼接 addr string
        HttpServer::new(factory)
            .bind((engine.conf.host.as_str(), engine.conf.port))?
            .run()
            .await
    }
}

#[cfg(test)]
mod tests {
    // 这些测试主要用于保证“上层能用”：在开启 feature 时，接口/类型约束应能顺利编译。

    #[cfg(feature = "axum")]
    #[test]
    fn axum_run_signature_should_compile() {
        use crate::{config::RestConf, engine::Engine};
        let engine = Engine::new(RestConf::default());
        let app = ::axum::Router::<()>::new();
        let _fut = crate::server::axum::run(engine, app);
        // 不 await：仅做类型/签名编译验证
    }

    #[cfg(feature = "actix")]
    #[test]
    fn actix_run_signature_should_compile() {
        use crate::{config::RestConf, engine::Engine};
        let engine = Engine::new(RestConf::default());
        let _fut = crate::server::actix::run(engine, || actix_web::App::new());
        // 不 await：仅做类型/签名编译验证
    }
}
