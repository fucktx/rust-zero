use std::sync::Arc;
use axum::{
    extract::{State{{if .HasRequest}}, Json{{end}}},
    response::{IntoResponse, Response},
};
use crate::svc::ServiceContext;
{{.ImportPackages}}

{{if .HasDoc}}{{.Doc}}{{end}}
pub async fn {{.HandlerName}}(
    State(svc_ctx): State<Arc<ServiceContext>>,
    {{if .HasRequest}}Json(req): Json<crate::types::{{.RequestType}}>,{{end}}
) -> Response {
    let logic = crate::logic::{{.LogicName}}::{{.LogicType}}::new(svc_ctx);

    {{if .HasResp}}
    match logic.{{.Call}}({{if .HasRequest}}req{{end}}).await {
        Ok(resp) => resp.into_response(),
        Err(err) => err.into_response(),
    }
    {{else}}
    match logic.{{.Call}}({{if .HasRequest}}req{{end}}).await {
        Ok(()) => ().into_response(), // 这里你可以自定义一个 OK 的 IntoResponse 实现
        Err(err) => err.into_response(),
    }
    {{end}}
}
