//! `zero`：对外统一入口（facade crate）。
//!
//! 推荐使用方式：
//! - `zero::core::...`
//! - `zero::rest::...`
//!
//! 注意：
//! - 对外统一通过 `zero::core` / `zero::rest` 访问。

/// `zero::core::*` -> 本项目 `core` crate。
pub mod core {
    pub use ::core::*;
}

/// 兼容 go-zero 风格：`zero::conf::must_load(...)`。
pub mod conf {
    pub use ::core::conf::*;
}

/// `zero::rest::*` -> `rest` crate。
pub mod rest {
    pub use ::rest::*;

    // 宏需要显式 re-export，保证 `zero::rest::router!` / `zero::rest::add_routes!` 可用
    pub use ::rest::router;

    /// `add_routes!` 目前仅在 axum feature 下提供。
    #[cfg(feature = "axum")]
    pub use ::rest::add_routes;
}
