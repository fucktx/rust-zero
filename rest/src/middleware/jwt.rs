//! JWT middleware（axum）。
//!
//! 启用方式：
//! - `rest = { ..., features = ["axum", "jwt"] }`
//! - 路由 options：`rest::WithJwt("secret")`

#[cfg(all(feature = "axum", feature = "jwt"))]
pub mod axum_jwt {
    use axum::body::Body;
    use axum::body::boxed;
    use axum::extract::State;
    use axum::http::{Request, StatusCode};
    use axum::middleware::Next;
    use axum::response::Response;
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
    use serde::Deserialize;
    use std::sync::Arc;

    #[derive(Debug, Deserialize)]
    struct Claims {
        // 标准字段：exp（秒）
        #[serde(default)]
        #[allow(dead_code)]
        exp: Option<u64>,
    }

    pub fn state(secret: impl Into<String>) -> Arc<String> {
        Arc::new(secret.into())
    }

    pub async fn auth(
        State(secret): State<Arc<String>>,
        req: Request<Body>,
        next: Next<Body>,
    ) -> Response {
        // Authorization: Bearer <token>
        let Some(authz) = req.headers().get(http::header::AUTHORIZATION) else {
            return unauthorized("missing Authorization");
        };
        let Ok(authz) = authz.to_str() else {
            return unauthorized("invalid Authorization");
        };
        let token = authz
            .strip_prefix("Bearer ")
            .or_else(|| authz.strip_prefix("bearer "));
        let Some(token) = token else {
            return unauthorized("invalid Authorization scheme");
        };

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        let key = DecodingKey::from_secret(secret.as_bytes());
        if decode::<Claims>(token, &key, &validation).is_err() {
            return unauthorized("invalid token");
        }

        next.run(req).await
    }

    fn unauthorized(msg: &'static str) -> Response {
        let mut resp = Response::new(boxed(Body::from(msg)));
        *resp.status_mut() = StatusCode::UNAUTHORIZED;
        resp
    }
}
