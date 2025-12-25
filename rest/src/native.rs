//! 原生框架入口（零运行时抽象成本）：
//! - `feature="axum"`：直接运行 `axum::Router`（即 `Router<()>`）
//! - `feature="actix"`：直接运行 actix 的 `HttpServer::new(factory)`

/// 运行原生框架（按 feature 选择）。
///
/// 注意：本函数在不同 feature 下签名不同，但调用点写法保持一致：
/// `rest::native::run(engine, app_or_factory).await?;`
#[cfg(feature = "axum")]
pub async fn run(engine: crate::engine::Engine, app: axum::Router) -> anyhow::Result<()> {
    crate::server::axum::run(engine, app).await
}

#[cfg(feature = "actix")]
pub async fn run<F, I, S, B>(engine: crate::engine::Engine, factory: F) -> std::io::Result<()>
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
    crate::server::actix::run(engine, factory).await
}
