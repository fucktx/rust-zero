// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

use std::task::{Context, Poll};
use std::{future::Future, pin::Pin};
use tower::{Layer, Service};
use axum::response::Response;
use axum::http::{Request};

pub struct {{.name}};

impl {{.name}} {
    pub fn new() -> Self {
        {{.name}}
    }

    /// 返回一个可用于 axum 路由的 Layer：
    ///
    /// ```ignore
    /// let app = Router::new()
    ///     .route("/path", get(handler))
    ///     .layer({{.name}}::new().layer());
    /// ```
    pub fn layer<S>(&self) -> {{.name}}Layer<S> {
        {{.name}}Layer::new()
    }
}

/// 中间件的 Layer 类型，用于包裹 Service。
pub struct {{.name}}Layer<S> {
    inner: std::marker::PhantomData<S>,
}

impl<S> {{.name}}Layer<S> {
    pub fn new() -> Self {
        Self {
            inner: std::marker::PhantomData,
        }
    }
}

impl<S> Layer<S> for {{.name}}Layer<S> {
    type Service = {{.name}}Middleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        {{.name}}Middleware { inner }
    }
}

/// 实际的中间件实现：在这里编写你的逻辑。
pub struct {{.name}}Middleware<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for {{.name}}Middleware<S>
where
    S: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        // TODO generate middleware implement function, delete after code implementation

        let mut inner = self.inner.clone();

        Box::pin(async move {
            // 这里写你的前置逻辑，例如：
            // - 记录日志
            // - 校验 header / token
            // - 注入 trace id 到 extensions
            //
            // 示例：
            // println!("request path: {}", req.uri().path());

            // Passthrough to next service / handler if need
            let response = inner.call(req).await?;

            // 这里可以写后置逻辑，例如修改响应头
            // let mut response = response;
            // response.headers_mut().insert(...);

            Ok(response)
        })
    }
}


