//! 编译期 DSL（宏展开）：
//! - **零运行时抽象成本**（无 dyn/BoxFuture/统一 Request 转换）
//! - 表面写法保持一致，但在不同 feature 下展开为原生框架调用
//!
//! 用法示例：
//! ```no_run
//! # use rest::{Engine, RestConf};
//! # // 注意：workspace `--all-features` 可能同时启用 axum+actix；此时 router! 会拒绝展开。
//! # #[cfg(all(feature="axum", not(feature="actix")))]
//! async fn demo() -> anyhow::Result<()> {
//! let engine = Engine::new(RestConf::default());
//! let app = rest::router! {
//!   group "/api" {
//!     GET "/ping" => ping;
//!   }
//!   GET "/ping" => ping;
//! };
//! rest::native::axum::run(engine, app).await
//! }
//! # #[cfg(all(feature="axum", not(feature="actix")))]
//! async fn ping() -> &'static str { "ok" }
//! ```

/// 统一 DSL：展开为原生框架代码。
///
/// 重要：这里的 `cfg(feature=...)` 必须绑定到 **rest crate 自身** 的 feature，
/// 不能在宏体内部再写 `#[cfg(...)]`（那会在调用方 crate 里判定，导致生成工程报错）。
#[cfg(all(feature = "axum", not(feature = "actix")))]
#[macro_export]
macro_rules! router {
    ($($tt:tt)*) => {{
        $crate::__router_axum!($($tt)*)
    }};
}

#[cfg(all(feature = "actix", not(feature = "axum")))]
#[macro_export]
macro_rules! router {
    ($($tt:tt)*) => {{
        $crate::__router_actix_factory!($($tt)*)
    }};
}

// 两个框架同时启用：crate 允许编译，但使用 router! 时给出明确错误（避免歧义）。
#[cfg(all(feature = "axum", feature = "actix"))]
#[macro_export]
macro_rules! router {
    ($($tt:tt)*) => {{
        compile_error!("rest::router!: enable only one of features `axum` or `actix`");
    }};
}

// 未启用任何框架：使用 router! 时给出明确错误。
#[cfg(not(any(feature = "axum", feature = "actix")))]
#[macro_export]
macro_rules! router {
    ($($tt:tt)*) => {{
        compile_error!(
            "rest::router!: please enable one feature on the `rest` crate: `axum` or `actix`"
        );
    }};
}

/// API 层路由/中间件宏：按框架实现拆分到子模块目录。
pub mod api;
