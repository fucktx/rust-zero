//! rest：Web 适配层/中间件/路由 DSL（从原 `rest` crate 合并而来）。

pub mod config;
pub mod dsl;
pub mod engine;
pub mod middleware;
pub mod native;
pub mod server;
// `web` 模块（早期运行时抽象层）已不再推荐使用；保留空间但不再对外暴露。

/// 方便上层（以及 rsctl 模板）直接 `use rust_zero::rest::RestConf;`
pub use self::config::RestConf;
pub use self::engine::Engine;
#[cfg(feature = "axum")]
pub use self::server::Server;

/// 让调用方可以使用 `rust_zero::rest::router! { ... }`
pub use crate::router;

/// go-zero 风格 options：写法类似 `rest::WithPrefix("/api/v1")`。
///
/// 这些 options 的目标是“API 形态框架无关”，但真正落到底层框架（axum/actix）的实现会按 feature 分流。
pub struct WithPrefix(pub &'static str);

/// go-zero 风格 options：写法类似 `rest::WithMiddleware(layer)`。
pub struct WithMiddleware<M>(pub M);

/// go-zero 风格 options：写法类似 `rest::WithJwt(secret)`。
///
/// - 需要启用 `rest` 的 feature：`jwt`（并配合选择框架 feature：`axum` 或 `actix`）。
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

pub struct WithSignature(pub self::config::SignatureConf);

pub struct WithNotFoundHandler<H>(pub H);
pub struct WithNotAllowedHandler;
pub struct WithFileServer;
pub struct WithTLSConfig;
pub struct WithUnauthorizedCallback;
pub struct WithUnsignedCallback;
