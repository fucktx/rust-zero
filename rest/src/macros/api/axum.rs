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
