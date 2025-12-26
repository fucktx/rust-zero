//! `rz_rest`：rust-zero 的 Web 适配层/中间件/路由 DSL。
//!
//! 说明：对外更推荐通过门面 crate `rust-zero` 使用：
//! - `rust_zero::rest::...`
//! - `rust_zero::rest::router! { ... }`

pub mod config;
pub mod dsl;
pub mod engine;
pub mod middleware;
pub mod native;
pub mod server;

pub use config::RestConf;
pub use engine::Engine;
#[cfg(all(feature = "axum", not(feature = "actix")))]
pub use server::Server;

// go-zero 风格 options（保持现有 API 形态）
pub struct WithPrefix(pub &'static str);
pub struct WithMiddleware<M>(pub M);
pub struct WithJwt<S>(pub S);

pub struct WithCors(pub Vec<String>);
pub struct WithCorsHeaders(pub Vec<String>);
pub struct WithCustomCors;

pub struct WithMaxBytes(pub i64);
pub struct WithMaxConns(pub usize);
pub struct WithTimeout(pub std::time::Duration);
pub struct WithRecover;
pub struct WithGunzip;
pub struct WithMetrics;
pub struct WithPrometheus;
pub struct WithBreaker;
pub struct WithShedding;
pub struct WithSSE;

pub struct WithSignature(pub crate::config::SignatureConf);

pub struct WithNotFoundHandler<H>(pub H);
pub struct WithNotAllowedHandler;
pub struct WithFileServer;
pub struct WithTLSConfig;
pub struct WithUnauthorizedCallback;
pub struct WithUnsignedCallback;
