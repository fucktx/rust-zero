//! 原生框架入口（零运行时抽象成本）：
//! - `feature="axum"`：直接运行 `axum::Router`（即 `Router<()>`）
//! - `feature="actix"`：直接运行 actix 的 `HttpServer::new(factory)`

/// 运行原生框架（按 feature 选择）。
///
/// 注意：本函数在不同 feature 下签名不同，但调用点写法保持一致：
/// `rest::native::run(engine, app_or_factory).await?;`
/// axum 入口（当开启 `axum` feature 时可用）。
#[cfg(feature = "axum")]
pub mod axum {
    pub async fn run(
        engine: crate::rest::engine::Engine,
        app: ::axum::Router,
    ) -> anyhow::Result<()> {
        crate::rest::server::axum::run(engine, app).await
    }
}

/// actix 入口（当开启 `actix` feature 时可用）。
#[cfg(feature = "actix")]
pub mod actix {
    pub async fn run<F, I, S, B>(
        engine: crate::rest::engine::Engine,
        factory: F,
    ) -> std::io::Result<()>
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
        crate::rest::server::actix::run(engine, factory).await
    }
}

/// 统一入口：仅当 **恰好启用一个** 框架 feature 时提供（避免 workspace `--all-features` 下重名）。
#[cfg(all(feature = "axum", not(feature = "actix")))]
pub async fn run(engine: super::engine::Engine, app: ::axum::Router) -> anyhow::Result<()> {
    self::axum::run(engine, app).await
}

#[cfg(all(feature = "actix", not(feature = "axum")))]
pub async fn run<F, I, S, B>(engine: super::engine::Engine, factory: F) -> std::io::Result<()>
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
    self::actix::run(engine, factory).await
}
