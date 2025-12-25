pub mod config;
pub mod dsl;
pub mod engine;
pub mod middleware;
pub mod native;
pub mod server;
pub mod web;

// 方便上层（以及 rsctl 模板）直接 `use rest::RestConf;`
pub use config::RestConf;
pub use engine::Engine;
#[cfg(feature = "axum")]
pub use server::Server;

/// go-zero 风格 options：写法类似 `rest::WithPrefix("/api/v1")`。
///
/// 这些 options 的目标是“API 形态框架无关”，但真正落到底层框架（axum/actix）的实现会按 feature 分流。
pub struct WithPrefix(pub &'static str);

/// go-zero 风格 options：写法类似 `rest::WithMiddleware(layer)`。
pub struct WithMiddleware<M>(pub M);

/// go-zero 风格 options：写法类似 `rest::WithJwt(secret)`。
///
/// - 需要启用 `rest` 的 feature：`jwt`（同时要求 `axum`，因为目前只实现了 axum 版）。
pub struct WithJwt<S>(pub S);

// -------- 下面是“先把接口补齐”的 WithXxx（按 go-zero service.go 命名）--------
// 说明：其中一部分在当前版本会在宏展开阶段给出 “暂未实现/需开启 feature” 的明确提示。

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
