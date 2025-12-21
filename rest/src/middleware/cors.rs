//! CORS middleware（axum）。
//!
//! 对应 go-zero：`WithCors(...)` / `WithCorsHeaders(...)`。

#[cfg(feature = "axum")]
pub mod axum_cors {
    use http::HeaderValue;
    use http::header::HeaderName;
    use tower_http::cors::{AllowOrigin, Any, CorsLayer};

    pub fn layer(origins: Vec<String>) -> CorsLayer {
        if origins.is_empty() {
            return CorsLayer::new().allow_origin(Any);
        }

        let mut allow: Vec<HeaderValue> = Vec::new();
        for o in origins {
            if let Ok(v) = HeaderValue::from_str(&o) {
                allow.push(v);
            }
        }
        if allow.is_empty() {
            // fallback: allow all
            CorsLayer::new().allow_origin(Any)
        } else {
            CorsLayer::new().allow_origin(AllowOrigin::list(allow))
        }
    }

    pub fn layer_allow_headers(origins: Vec<String>, headers: Vec<String>) -> CorsLayer {
        let layer = layer(origins);
        if headers.is_empty() {
            return layer;
        }

        let mut allow: Vec<HeaderName> = Vec::new();
        for h in headers {
            if let Ok(v) = HeaderName::from_bytes(h.as_bytes()) {
                allow.push(v);
            }
        }
        if allow.is_empty() {
            layer
        } else {
            layer.allow_headers(allow)
        }
    }
}


