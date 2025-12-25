//! 框架中间件实现集合（按功能分文件）。
//!
//! Rust 2024 推荐的模块组织方式：使用 `src/middleware.rs` 作为父模块文件，
//! 子模块放在 `src/middleware/*.rs`，避免 `mod.rs`。

pub mod cors;
pub mod jwt;
pub mod max_bytes;
pub mod not_found;
pub mod timeout;
