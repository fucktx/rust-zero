//! Web 框架启动适配层（axum / actix-web ...）。
//!
//! 设计目标：
//! - `rest` crate 本身只提供“配置 + 运行入口”；
//! - 具体 Web 框架通过 feature 开关按需引入（避免强依赖）。

/// 通用错误类型：不同框架实现可能返回不同的错误，这里统一用 anyhow。
#[cfg(any(feature = "axum", feature = "actix"))]
pub type Result<T> = anyhow::Result<T>;

/// axum 适配：`cargo add rest --features axum`
#[cfg(feature = "axum")]
pub mod axum {
    use crate::engine::Engine;
    use anyhow::Context;
    use std::net::SocketAddr;

    /// 启动 axum 服务（需要在 tokio runtime 中调用）。
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
    use actix_service::IntoServiceFactory;
    use actix_service::{Service, ServiceFactory};
    use actix_web::dev::AppConfig;
    use actix_web::{Error, HttpServer};
    use actix_http::{Request, Response};
    use actix_http::body::MessageBody;
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
        let app = ::axum::Router::new();
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


