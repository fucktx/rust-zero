//! 编译期 DSL（宏展开）：
//! - **零运行时抽象成本**（无 dyn/BoxFuture/统一 Request 转换）
//! - 表面写法保持一致，但在不同 feature 下展开为原生框架调用
//!
//! 用法示例：
//! ```no_run
//! # use rz_rest::{Engine, RestConf};
//! # // 注意：workspace `--all-features` 可能同时启用 axum+actix；此时 router! 会拒绝展开。
//! # #[cfg(all(feature="axum", not(feature="actix")))]
//! async fn demo() -> anyhow::Result<()> {
//! let engine = Engine::new(RestConf::default());
//! let app = rz_rest::router! {
//!   group "/api" {
//!     GET "/ping" => ping;
//!   }
//!   GET "/ping" => ping;
//! };
//! rz_rest::native::axum::run(engine, app).await
//! }
//! # #[cfg(all(feature="axum", not(feature="actix")))]
//! async fn ping() -> &'static str { "ok" }
//! ```

/// 统一 DSL：展开为原生框架代码。
///
/// 重要：这里的 `cfg(feature=...)` 必须绑定到 **rz-rest crate 自身** 的 feature，
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
            "rest::router!: please enable one feature on the `rz-rest` crate: `axum` or `actix`"
        );
    }};
}

// -------------------------
// go-zero style: add_routes! (axum only for now)
// -------------------------

#[cfg(feature = "axum")]
#[macro_export]
macro_rules! add_routes {
    ( [ $($routes:tt),* $(,)? ] ) => {{
        let mut __r = ::axum::Router::new();
        $(
            __r = $crate::__add_routes_axum_one!(__r, $routes);
        )*
        __r
    }};

    ( [ $($routes:tt),* $(,)? ], ) => {{
        $crate::add_routes!([ $($routes),* ])
    }};

    ( [ $($routes:tt),* $(,)? ], $($opts:tt)+ ) => {{
        let mut __r = ::axum::Router::new();
        $(
            __r = $crate::__add_routes_axum_one!(__r, $routes);
        )*
        __r = $crate::__add_routes_axum_opts!(__r, $($opts)+);
        __r
    }};
}

#[cfg(feature = "axum")]
#[macro_export]
#[doc(hidden)]
macro_rules! __add_routes_axum_one {
    ($router:expr, { method: $method:expr, path: $path:literal, handler: $handler:expr } ) => {{
        let _ = $method;
        $router.route($path, $handler)
    }};
}

#[cfg(feature = "axum")]
#[macro_export]
#[doc(hidden)]
macro_rules! __add_routes_axum_opts {
    ($router:expr, ) => { $router };
    ($router:expr) => { $router };

    ($router:expr, $crate::WithPrefix($prefix:expr) $(, $($rest:tt)*)? ) => {{
        let nested = ::axum::Router::new().nest($prefix, $router);
        $crate::__add_routes_axum_opts!(nested $(, $($rest)*)?)
    }};
    ($router:expr, $m:ident :: WithPrefix($prefix:expr) $(, $($rest:tt)*)? ) => {{
        let nested = ::axum::Router::new().nest($prefix, $router);
        $crate::__add_routes_axum_opts!(nested $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithPrefix($prefix:expr) $(, $($rest:tt)*)? ) => {{
        let nested = ::axum::Router::new().nest($prefix, $router);
        $crate::__add_routes_axum_opts!(nested $(, $($rest)*)?)
    }};

    ($router:expr, $crate::WithMiddleware($mw:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($mw);
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, $m:ident :: WithMiddleware($mw:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($mw);
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithMiddleware($mw:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($mw);
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};

    ($router:expr, $crate::WithJwt($secret:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer(::axum::middleware::from_fn_with_state(
            $crate::middleware::jwt::axum_jwt::state($secret),
            $crate::middleware::jwt::axum_jwt::auth,
        ));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};

    ($router:expr, $crate::WithTimeout($dur:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer(::axum::middleware::from_fn_with_state(
            $crate::middleware::timeout::axum_timeout::state($dur),
            $crate::middleware::timeout::axum_timeout::run,
        ));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};

    ($router:expr, $m:ident :: WithCors($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::cors::axum_cors::layer($v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithCors($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::cors::axum_cors::layer($v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, $m:ident :: WithCorsHeaders($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::cors::axum_cors::layer_allow_headers(Vec::new(), $v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithCorsHeaders($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::cors::axum_cors::layer_allow_headers(Vec::new(), $v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, $m:ident :: WithMaxBytes($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::max_bytes::axum_max_bytes::layer($v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithMaxBytes($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::max_bytes::axum_max_bytes::layer($v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, $m:ident :: WithNotFoundHandler($h:expr) $(, $($rest:tt)*)? ) => {{
        let r = $crate::middleware::not_found::axum_not_found::apply($router, $h);
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithNotFoundHandler($h:expr) $(, $($rest:tt)*)? ) => {{
        let r = $crate::middleware::not_found::axum_not_found::apply($router, $h);
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
}

// -------------------------
// axum expansion
// -------------------------

#[macro_export]
#[doc(hidden)]
macro_rules! __router_axum {
    ($($tt:tt)*) => {{
        $crate::__router_axum_build!(::axum::Router::new(), $($tt)*)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __router_axum_build {
    ($app:expr,) => { $app };
    ($app:expr) => { $app };

    ($app:expr, state $st:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!($app.layer(::axum::Extension($st)), $($rest)*)
    }};

    ($app:expr, middleware_fn $mw:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!($app.layer(::axum::middleware::from_fn($mw)), $($rest)*)
    }};

    ($app:expr, middleware_state $st:expr, $mw:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!(
            $app.layer(::axum::middleware::from_fn_with_state($st, $mw)),
            $($rest)*
        )
    }};

    ($app:expr, middleware $mw:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!($app.layer($mw), $($rest)*)
    }};

    ($app:expr, group $prefix:literal { $($inner:tt)* } $($rest:tt)*) => {{
        let sub = $crate::__router_axum_build!(::axum::Router::new(), $($inner)*);
        $crate::__router_axum_build!($app.nest($prefix, sub), $($rest)*)
    }};
    ($app:expr, group $prefix:literal { $($inner:tt)* }; $($rest:tt)*) => {{
        $crate::__router_axum_build!($app, group $prefix { $($inner)* } $($rest)*)
    }};

    ($app:expr, GET $path:literal => $handler:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!($app.route($path, ::axum::routing::get($handler)), $($rest)*)
    }};
    ($app:expr, POST $path:literal => $handler:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!(
            $app.route($path, ::axum::routing::post($handler)),
            $($rest)*
        )
    }};
    ($app:expr, PUT $path:literal => $handler:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!($app.route($path, ::axum::routing::put($handler)), $($rest)*)
    }};
    ($app:expr, DELETE $path:literal => $handler:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!(
            $app.route($path, ::axum::routing::delete($handler)),
            $($rest)*
        )
    }};
    ($app:expr, PATCH $path:literal => $handler:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!(
            $app.route($path, ::axum::routing::patch($handler)),
            $($rest)*
        )
    }};
}

// -------------------------
// actix expansion
// -------------------------

#[macro_export]
#[doc(hidden)]
macro_rules! __router_actix_factory {
    ($($tt:tt)*) => {{
        move || {
            let app = ::actix_web::App::new();
            $crate::__actix_apply_any!(app, $($tt)*)
        }
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __actix_apply_any {
    ($app:expr,) => { $app };
    ($app:expr) => { $app };

    ($app:expr, state $st:expr; $($rest:tt)*) => {{
        let app = $app.app_data(::actix_web::web::Data::new($st));
        $crate::__actix_apply_any!(app, $($rest)*)
    }};

    ($app:expr, middleware_fn $mw:expr; $($rest:tt)*) => {{
        let app = $app.wrap(::actix_web::middleware::from_fn($mw));
        $crate::__actix_apply_any!(app, $($rest)*)
    }};

    ($app:expr, middleware_state $st:expr, $mw:expr; $($rest:tt)*) => {{
        let _ = ($st, $mw);
        compile_error!("rest::router!: `middleware_state` is only supported for axum; for actix please use `middleware_fn` with a closure capturing state");
        $app
    }};

    ($app:expr, middleware $mw:expr; $($rest:tt)*) => {{
        let app = $app.wrap($mw);
        $crate::__actix_apply_any!(app, $($rest)*)
    }};

    ($app:expr, group $prefix:literal { $($inner:tt)* } $($rest:tt)*) => {{
        let scope = ::actix_web::web::scope($prefix);
        let scope = $crate::__actix_apply_any!(scope, $($inner)*);
        let app = $app.service(scope);
        $crate::__actix_apply_any!(app, $($rest)*)
    }};
    ($app:expr, group $prefix:literal { $($inner:tt)* }; $($rest:tt)*) => {{
        $crate::__actix_apply_any!($app, group $prefix { $($inner)* } $($rest)*)
    }};

    ($app:expr, GET $path:literal => $handler:expr; $($rest:tt)*) => {{
        let app = $app.route($path, ::actix_web::web::get().to($handler));
        $crate::__actix_apply_any!(app, $($rest)*)
    }};
    ($app:expr, POST $path:literal => $handler:expr; $($rest:tt)*) => {{
        let app = $app.route($path, ::actix_web::web::post().to($handler));
        $crate::__actix_apply_any!(app, $($rest)*)
    }};
    ($app:expr, PUT $path:literal => $handler:expr; $($rest:tt)*) => {{
        let app = $app.route($path, ::actix_web::web::put().to($handler));
        $crate::__actix_apply_any!(app, $($rest)*)
    }};
    ($app:expr, DELETE $path:literal => $handler:expr; $($rest:tt)*) => {{
        let app = $app.route($path, ::actix_web::web::delete().to($handler));
        $crate::__actix_apply_any!(app, $($rest)*)
    }};
    ($app:expr, PATCH $path:literal => $handler:expr; $($rest:tt)*) => {{
        let app = $app.route($path, ::actix_web::web::patch().to($handler));
        $crate::__actix_apply_any!(app, $($rest)*)
    }};
}
