//! `rust_zero`：对外统一入口（facade crate）。
//!
//! 使用方推荐只依赖 `rust-zero`：
//! - `rust_zero::core::...`
//! - `rust_zero::rest::...`
//! - `rust_zero::conf::must_load(...)`

pub mod core {
    pub use rz_core::*;
}

pub mod conf {
    pub use rz_core::conf::*;
}

pub mod rest {
    pub use rz_rest::*;
    // 宏需要显式 re-export，保证 `rust_zero::rest::router!` 可用
    pub use rz_rest::add_routes;
    pub use rz_rest::router;
}
