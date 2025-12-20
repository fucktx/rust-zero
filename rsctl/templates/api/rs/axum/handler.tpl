{{if .HasDoc}}{{.Doc}}{{end}}
pub async fn {{.HandlerName}}(
    State(svc_ctx): State<Arc<ServiceContext>>,
    {{if .HasRequest}}Json(req): Json<crate::types::{{.RequestType}}>,{{end}}
) -> impl IntoResponse {
    let logic = crate::logic::{{.LogicName}}::{{.LogicType}}::new(svc_ctx);

    {{if .HasResp}}
    match logic.{{.Call}}({{if .HasRequest}}req{{end}}).await {
        Ok(resp) => Json(resp).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
    {{else}}
    match logic.{{.Call}}({{if .HasRequest}}req{{end}}).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
    {{end}}
}


