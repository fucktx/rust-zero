//! Timeout middleware（axum）。
//!
//! 对应 go-zero：`WithTimeout(...)`。

#[cfg(feature = "axum")]
pub mod axum_timeout {
    use axum::body::{boxed, Body};
    use axum::http::{Request, StatusCode};
    use axum::middleware::Next;
    use axum::response::Response;
    use std::sync::Arc;
    use std::time::Duration;

    pub fn state(timeout: Duration) -> Arc<Duration> {
        Arc::new(timeout)
    }

    pub async fn run(
        state: axum::extract::State<Arc<Duration>>,
        req: Request<Body>,
        next: Next<Body>,
    ) -> Response {
        let dur = *state.0;
        match tokio::time::timeout(dur, next.run(req)).await {
            Ok(resp) => resp,
            Err(_) => {
                let mut resp = Response::new(boxed(Body::from("timeout")));
                *resp.status_mut() = StatusCode::GATEWAY_TIMEOUT;
                resp
            }
        }
    }
}


