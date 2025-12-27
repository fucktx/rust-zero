//! Web 框架启动适配层（axum / actix-web ...）。

#[cfg(any(feature = "axum", feature = "actix"))]
pub type Result<T> = anyhow::Result<T>;

/// 启动服务（对齐 go-zero 的 `server.Start()` 语义）。
///
/// 为避免 workspace `--all-features` 下重名冲突，`start(...)` 仅在 **恰好启用一个框架 feature** 时提供。
#[cfg(all(feature = "axum", not(feature = "actix")))]
pub async fn start(engine: crate::engine::Engine, app: ::axum::Router) -> anyhow::Result<()> {
    crate::native::axum::run(engine, app).await
}

#[cfg(all(feature = "actix", not(feature = "axum")))]
pub async fn start<F, I, S, B>(engine: crate::engine::Engine, factory: F) -> std::io::Result<()>
where
    F: Fn() -> I + Send + Clone + 'static,
    I: actix_service::IntoServiceFactory<S, actix_http::Request>,
    S: actix_service::ServiceFactory<actix_http::Request, Config = actix_web::dev::AppConfig>
        + 'static,
    S::Error: Into<actix_web::Error> + 'static,
    S::InitError: std::fmt::Debug,
    S::Response: Into<actix_http::Response<B>> + 'static,
    <S::Service as actix_service::Service<actix_http::Request>>::Future: 'static,
    S::Service: 'static,
    B: actix_http::body::MessageBody + 'static,
{
    crate::native::actix::run(engine, factory).await
}

// go-zero 风格 Server（axum）
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

    pub fn add_route(self, route: ::axum::Router<S>) -> Self {
        self.add_routes(route)
    }

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

    pub fn with_not_found_handler<H, T>(mut self, handler: H) -> Self
    where
        H: ::axum::handler::Handler<T, S> + Clone + Send + 'static,
        T: 'static,
    {
        self.router = self.router.fallback(handler);
        self
    }

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
        crate::native::axum::run(engine, router).await
    }
}

/// axum native runner
#[cfg(feature = "axum")]
pub mod axum {
    use crate::engine::Engine;
    use anyhow::Context;
    use std::net::SocketAddr;

    pub async fn run(engine: Engine, app: axum::Router) -> anyhow::Result<()> {
        let addr: SocketAddr = engine
            .conf
            .addr_string()
            .parse()
            .context("invalid host/port")?;

        let server = axum::Server::try_bind(&addr).with_context(|| format!("bind {addr}"))?;
        server
            .serve(app.into_make_service())
            .await
            .context("axum server run")?;
        Ok(())
    }
}

/// actix native runner
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
        HttpServer::new(factory)
            .bind((engine.conf.host.as_str(), engine.conf.port))?
            .run()
            .await
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "axum")]
    #[test]
    fn axum_run_signature_should_compile() {
        let engine = crate::Engine::new(crate::RestConf::default());
        let app = ::axum::Router::<()>::new();
        let _fut = crate::server::axum::run(engine, app);
    }

    #[cfg(feature = "actix")]
    #[test]
    fn actix_run_signature_should_compile() {
        let engine = crate::Engine::new(crate::RestConf::default());
        let _fut = crate::server::actix::run(engine, actix_web::App::new);
    }
}
