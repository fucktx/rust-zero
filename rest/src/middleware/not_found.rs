//! NotFound handler（axum）。
//!
//! 对应 go-zero：`WithNotFoundHandler(...)`（更接近 server-level 的配置）。

#[cfg(feature = "axum")]
pub mod axum_not_found {
    pub fn apply<R>(
        router: axum::Router<R>,
        handler: impl axum::handler::Handler<(), R> + 'static,
    ) -> axum::Router<R>
    where
        R: Clone + Send + Sync + 'static,
    {
        router.fallback(handler)
    }
}
