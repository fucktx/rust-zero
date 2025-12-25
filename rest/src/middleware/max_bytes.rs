//! MaxBytes middleware（axum）。
//!
//! 对应 go-zero：`WithMaxBytes(...)`（限制请求体大小）。

#[cfg(feature = "axum")]
pub mod axum_max_bytes {
    use axum::extract::DefaultBodyLimit;

    pub fn layer(max_bytes: i64) -> DefaultBodyLimit {
        // <=0 视为关闭限制（给一个很大的值）
        if max_bytes <= 0 {
            return DefaultBodyLimit::disable();
        }
        DefaultBodyLimit::max(max_bytes as usize)
    }
}
