{{if .HasDoc}}{{.Doc}}{{end}}
pub fn {{.HandlerName}}(svc_ctx: Arc<ServiceContext>) -> MethodRouter {
    // go-zero 风格：Handler 由 ctx “构造”出来，路由表里写 `handler(ctx.clone())`。
    {{.AxumMethodFn}}(move |{{if .HasRequest}}Json(req): Json<crate::types::{{.RequestType}}>{{end}}| {
        let svc_ctx = svc_ctx.clone();
        async move {
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
    })
}


