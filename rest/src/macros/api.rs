//! API 层相关宏：按框架实现拆分。

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "actix")]
pub mod actix;
