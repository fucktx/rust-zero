//! 编译期 DSL（宏展开）：
//! - **零运行时抽象成本**（无 dyn/BoxFuture/统一 Request 转换）
//! - 表面写法保持一致，但在不同 feature 下展开为原生框架调用
//!
//! 用法示例：
//! ```no_run
//! # use rust_zero::rest::{Engine, RestConf};
//! # // 注意：workspace `--all-features` 可能同时启用 axum+actix；此时 router! 会拒绝展开。
//! # #[cfg(all(feature="axum", not(feature="actix")))]
//! async fn demo() -> anyhow::Result<()> {
//! let engine = Engine::new(RestConf::default());
//! let app = rust_zero::rest::router! {
//!   group "/api" {
//!     GET "/ping" => ping;
//!   }
//!   GET "/ping" => ping;
//! };
//! rust_zero::rest::native::axum::run(engine, app).await
//! }
//! # #[cfg(all(feature="axum", not(feature="actix")))]
//! async fn ping() -> &'static str { "ok" }
//! ```

// 重要：宏展开里的 `cfg(feature=...)` 会在**调用方 crate**里生效，而不是 rest crate。
// 为了让 `rest::router!` 只依赖 rest 自己的 feature，我们把宏定义本身放到 cfg 下。

/// 统一 DSL：展开为原生框架代码。
///
/// 说明：
/// - 当只开启一个框架 feature 时（axum 或 actix），可正常展开；
/// - 当 **同时** 开启 `axum+actix`（例如 workspace `--all-features`）时，crate 允许编译，
///   但如果实际调用 `router!` 会给出明确的编译错误（避免类型/语义歧义）。
#[macro_export]
macro_rules! router {
    ($($tt:tt)*) => {{
        // 两个框架同时启用：只有在真正用到 router! 时才报错（避免影响 workspace all-features 编译）。
        #[cfg(all(feature = "axum", feature = "actix"))]
        {
            compile_error!("rest::router!: enable only one of features `axum` or `actix`");
        }

        #[cfg(all(feature = "axum", not(feature = "actix")))]
        {
            $crate::__router_axum!($($tt)*)
        }

        #[cfg(all(feature = "actix", not(feature = "axum")))]
        {
            $crate::__router_actix_factory!($($tt)*)
        }

        #[cfg(not(any(feature = "axum", feature = "actix")))]
        {
            compile_error!(
                "rest::router!: please enable one feature on the `rest` crate: `axum` or `actix`"
            );
        }
    }};
}

// -------------------------
// go-zero style: add_routes! (axum only for now)
// -------------------------

/// go-zero 风格的“路由表 + options”写法（零运行时抽象成本）。
///
/// 语法示例：
/// ```ignore
/// let app = Router::new()
///   .route("/healthz", axum::routing::get(|| async { "ok" }))
///   .merge(rest::add_routes!(
///     [
///       { method: http::Method::GET,  path: "/route/isRouteExist", handler: route::is_route_exist(ctx.clone()) },
///       { method: http::Method::POST, path: "/route/getUserRoutes", handler: route::get_user_routes(ctx.clone()) },
///     ],
///     rest::WithPrefix("/api/v1"),
///     rest::WithMiddleware(JwtLayer::new(secret)),
///   ));
/// ```
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

    // 允许写成 `add_routes!([ ... ],)`（便于生成器统一输出）
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
        let _ = $method; // 仅用于可读性/对齐 go-zero，不影响运行时。
        $router.route($path, $handler)
    }};
}

#[cfg(feature = "axum")]
#[macro_export]
#[doc(hidden)]
macro_rules! __add_routes_axum_opts {
    ($router:expr, ) => { $router };
    ($router:expr) => { $router };

    // WithPrefix("/x"): 将本次 routes 整体 nest 到指定前缀下（等价 go-zero WithPrefix）
    ($router:expr, $crate::WithPrefix($prefix:expr) $(, $($rest:tt)*)? ) => {{
        let nested = ::axum::Router::new().nest($prefix, $router);
        $crate::__add_routes_axum_opts!(nested $(, $($rest)*)?)
    }};
    // 允许调用方写 `rest::WithPrefix(...)`
    ($router:expr, $m:ident :: WithPrefix($prefix:expr) $(, $($rest:tt)*)? ) => {{
        let nested = ::axum::Router::new().nest($prefix, $router);
        $crate::__add_routes_axum_opts!(nested $(, $($rest)*)?)
    }};
    // 允许调用方写 `::rest::WithPrefix(...)`
    ($router:expr, :: $m:ident :: WithPrefix($prefix:expr) $(, $($rest:tt)*)? ) => {{
        let nested = ::axum::Router::new().nest($prefix, $router);
        $crate::__add_routes_axum_opts!(nested $(, $($rest)*)?)
    }};

    // WithMiddleware(layer): 对本次 routes 整体加 Layer（等价 go-zero WithJwt/WithMiddleware 一类）
    ($router:expr, $crate::WithMiddleware($mw:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($mw);
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    // 允许调用方写 `rest::WithMiddleware(...)`
    ($router:expr, $m:ident :: WithMiddleware($mw:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($mw);
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    // 允许调用方写 `::rest::WithMiddleware(...)`
    ($router:expr, :: $m:ident :: WithMiddleware($mw:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($mw);
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};

    // WithJwt(secret): 内置 JWT 校验（需要启用 rest features: ["axum", "jwt"]）
    ($router:expr, $crate::WithJwt($secret:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer(::axum::middleware::from_fn_with_state(
            $crate::middleware::jwt::axum_jwt::state($secret),
            $crate::middleware::jwt::axum_jwt::auth,
        ));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, $m:ident :: WithJwt($secret:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer(::axum::middleware::from_fn_with_state(
            $crate::middleware::jwt::axum_jwt::state($secret),
            $crate::middleware::jwt::axum_jwt::auth,
        ));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithJwt($secret:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer(::axum::middleware::from_fn_with_state(
            $crate::middleware::jwt::axum_jwt::state($secret),
            $crate::middleware::jwt::axum_jwt::auth,
        ));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};

    // WithTimeout(dur): axum 版实现
    ($router:expr, $crate::WithTimeout($dur:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer(::axum::middleware::from_fn_with_state(
            $crate::middleware::timeout::axum_timeout::state($dur),
            $crate::middleware::timeout::axum_timeout::run,
        ));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, $m:ident :: WithTimeout($dur:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer(::axum::middleware::from_fn_with_state(
            $crate::middleware::timeout::axum_timeout::state($dur),
            $crate::middleware::timeout::axum_timeout::run,
        ));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithTimeout($dur:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer(::axum::middleware::from_fn_with_state(
            $crate::middleware::timeout::axum_timeout::state($dur),
            $crate::middleware::timeout::axum_timeout::run,
        ));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};

    // 下面这些 WithXxx 先把接口补齐：如果你在 routes 里用到了，会给出明确的编译期提示，不会 silent no-op。
    // WithCors(origins): axum 版实现
    ($router:expr, $m:ident :: WithCors($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::cors::axum_cors::layer($v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithCors($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::cors::axum_cors::layer($v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    // WithCorsHeaders(headers): axum 版实现（这里默认允许所有 origin；若需要按 origin 配置，用 WithCors + WithCorsHeaders 的组合后续再细化）
    ($router:expr, $m:ident :: WithCorsHeaders($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::cors::axum_cors::layer_allow_headers(Vec::new(), $v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithCorsHeaders($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::cors::axum_cors::layer_allow_headers(Vec::new(), $v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    // WithMaxBytes(max): axum 版实现
    ($router:expr, $m:ident :: WithMaxBytes($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::max_bytes::axum_max_bytes::layer($v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithMaxBytes($v:expr) $(, $($rest:tt)*)? ) => {{
        let r = $router.layer($crate::middleware::max_bytes::axum_max_bytes::layer($v));
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};

    // WithNotFoundHandler(handler): axum 版实现
    ($router:expr, $m:ident :: WithNotFoundHandler($h:expr) $(, $($rest:tt)*)? ) => {{
        let r = $crate::middleware::not_found::axum_not_found::apply($router, $h);
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};
    ($router:expr, :: $m:ident :: WithNotFoundHandler($h:expr) $(, $($rest:tt)*)? ) => {{
        let r = $crate::middleware::not_found::axum_not_found::apply($router, $h);
        $crate::__add_routes_axum_opts!(r $(, $($rest)*)?)
    }};

    ($router:expr, $m:ident :: WithMaxConns($v:expr) $(, $($rest:tt)*)? ) => {{
        let _ = ($m, $v);
        compile_error!("rest::WithMaxConns: 已预留接口，暂未实现");
        $router
    }};
    ($router:expr, $m:ident :: WithRecover $(, $($rest:tt)*)? ) => {{
        let _ = $m;
        compile_error!("rest::WithRecover: 已预留接口，暂未实现");
        $router
    }};
    ($router:expr, $m:ident :: WithGunzip $(, $($rest:tt)*)? ) => {{
        let _ = $m;
        compile_error!("rest::WithGunzip: 已预留接口，暂未实现");
        $router
    }};
    ($router:expr, $m:ident :: WithMetrics $(, $($rest:tt)*)? ) => {{
        let _ = $m;
        compile_error!("rest::WithMetrics: 已预留接口，暂未实现");
        $router
    }};
    ($router:expr, $m:ident :: WithPrometheus $(, $($rest:tt)*)? ) => {{
        let _ = $m;
        compile_error!("rest::WithPrometheus: 已预留接口，暂未实现");
        $router
    }};
    ($router:expr, $m:ident :: WithBreaker $(, $($rest:tt)*)? ) => {{
        let _ = $m;
        compile_error!("rest::WithBreaker: 已预留接口，暂未实现");
        $router
    }};
    ($router:expr, $m:ident :: WithShedding $(, $($rest:tt)*)? ) => {{
        let _ = $m;
        compile_error!("rest::WithShedding: 已预留接口，暂未实现");
        $router
    }};
    ($router:expr, $m:ident :: WithSSE $(, $($rest:tt)*)? ) => {{
        let _ = $m;
        compile_error!("rest::WithSSE: 已预留接口，暂未实现");
        $router
    }};
    ($router:expr, $m:ident :: WithSignature($v:expr) $(, $($rest:tt)*)? ) => {{
        let _ = ($m, $v);
        compile_error!("rest::WithSignature: 已预留接口，暂未实现");
        $router
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

    // state：通过 axum::Extension 注入共享状态（保持 Router<()>，避免 state 泛型污染）
    ($app:expr, state $st:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!($app.layer(::axum::Extension($st)), $($rest)*)
    }};

    // middleware_fn：传入 `async fn(req, next) -> Response`，由宏展开成 `from_fn(...)`
    ($app:expr, middleware_fn $mw:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!($app.layer(::axum::middleware::from_fn($mw)), $($rest)*)
    }};

    // middleware_state：传入 (state_expr, async_fn)，由宏展开成 `from_fn_with_state(...)`
    ($app:expr, middleware_state $st:expr, $mw:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!(
            $app.layer(::axum::middleware::from_fn_with_state($st, $mw)),
            $($rest)*
        )
    }};

    ($app:expr, middleware $mw:expr; $($rest:tt)*) => {{
        $crate::__router_axum_build!($app.layer($mw), $($rest)*)
    }};

    // group/nest：用 axum::Router::nest 实现前缀 + 作用域中间件（middleware 写在 group 内即可）
    ($app:expr, group $prefix:literal { $($inner:tt)* } $($rest:tt)*) => {{
        let sub = $crate::__router_axum_build!(::axum::Router::new(), $($inner)*);
        $crate::__router_axum_build!($app.nest($prefix, sub), $($rest)*)
    }};
    ($app:expr, group $prefix:literal { $($inner:tt)* }; $($rest:tt)*) => {{
        $crate::__router_axum_build!($app, group $prefix { $($inner)* } $($rest)*)
    }};
    ($app:expr, nest $prefix:literal { $($inner:tt)* } $($rest:tt)*) => {{
        $crate::__router_axum_build!($app, group $prefix { $($inner)* } $($rest)*)
    }};
    ($app:expr, nest $prefix:literal { $($inner:tt)* }; $($rest:tt)*) => {{
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
        // 重要：HttpServer::new(factory) 需要 factory: Clone + 'static。
        // 这里用 move closure，要求被捕获的 middleware/handler 表达式具备 Clone（或不捕获）。
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

    // state：通过 actix_web::web::Data 注入共享状态
    ($app:expr, state $st:expr; $($rest:tt)*) => {{
        let app = $app.app_data(::actix_web::web::Data::new($st));
        $crate::__actix_apply_any!(app, $($rest)*)
    }};

    // middleware_fn：传入 `async fn(req, next) -> Result<...>` 或 closure，由宏展开成 `from_fn(...)`
    ($app:expr, middleware_fn $mw:expr; $($rest:tt)*) => {{
        let app = $app.wrap(::actix_web::middleware::from_fn($mw));
        $crate::__actix_apply_any!(app, $($rest)*)
    }};

    // middleware_state：actix 目前走 closure 捕获（由生成器侧直接输出 closure），这里给出明确错误避免 silent。
    ($app:expr, middleware_state $st:expr, $mw:expr; $($rest:tt)*) => {{
        let _ = ($st, $mw);
        compile_error!("rest::router!: `middleware_state` is only supported for axum; for actix please use `middleware_fn` with a closure capturing state");
        $app
    }};

    ($app:expr, middleware $mw:expr; $($rest:tt)*) => {{
        let app = $app.wrap($mw);
        $crate::__actix_apply_any!(app, $($rest)*)
    }};

    // group/nest：用 actix_web::web::scope 实现前缀 + 作用域中间件
    ($app:expr, group $prefix:literal { $($inner:tt)* } $($rest:tt)*) => {{
        let scope = ::actix_web::web::scope($prefix);
        let scope = $crate::__actix_apply_any!(scope, $($inner)*);
        let app = $app.service(scope);
        $crate::__actix_apply_any!(app, $($rest)*)
    }};
    ($app:expr, group $prefix:literal { $($inner:tt)* }; $($rest:tt)*) => {{
        $crate::__actix_apply_any!($app, group $prefix { $($inner)* } $($rest)*)
    }};
    ($app:expr, nest $prefix:literal { $($inner:tt)* } $($rest:tt)*) => {{
        $crate::__actix_apply_any!($app, group $prefix { $($inner)* } $($rest)*)
    }};
    ($app:expr, nest $prefix:literal { $($inner:tt)* }; $($rest:tt)*) => {{
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
