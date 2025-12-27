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
