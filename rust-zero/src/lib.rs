//! `rust_zero`：将原本独立的 `core` + `rest` 合并为一个 crate。
//!
//! 说明：
//! - 包名/目录名可以是 `rust-zero`
//! - 但在 Rust 代码里 crate 标识符必须是 `rust_zero`（`-` 会变成 `_`）
//!
//! 对外推荐使用：
//! - `rust_zero::core::...`
//! - `rust_zero::rest::...`
//! - `rust_zero::conf::must_load(...)`（等价于 `rust_zero::core::conf::must_load(...)`）

pub mod core;
pub mod rest;

/// 兼容 go-zero 的 `conf.MustLoad` 风格：`rust_zero::conf::must_load(...)`。
pub mod conf {
    pub use crate::core::conf::*;
}

// ---------------------------------------------------------------------
// 兼容层：为了让 DSL 宏内部仍能用 `$crate::WithXxx` / `$crate::middleware::...`，
// 我们在 crate 根 re-export `rest` 里的对应类型/模块。
// 这样既满足“单 crate”目标，又不需要把宏大改一遍。
// ---------------------------------------------------------------------

pub use crate::rest::{
    WithBreaker, WithCors, WithCorsHeaders, WithCustomCors, WithFileServer, WithGunzip, WithJwt,
    WithMaxBytes, WithMaxConns, WithMetrics, WithMiddleware, WithNotAllowedHandler,
    WithNotFoundHandler, WithPrefix, WithPrometheus, WithRecover, WithSSE, WithShedding,
    WithSignature, WithTLSConfig, WithTimeout, WithUnauthorizedCallback, WithUnsignedCallback,
};

pub mod middleware {
    pub use crate::rest::middleware::*;
}
