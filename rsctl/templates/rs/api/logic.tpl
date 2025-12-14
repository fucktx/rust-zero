// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

use std::sync::Arc;
use tracing::instrument;
use crate::svc::ServiceContext;
{{.imports}}

pub struct {{.logic}} {
    /// 请求级别的 tracing span / 日志上下文
    // 这里不直接存 context，通常从 handler 的参数里获取；如需可改成存 `Span` 或自定义 Context。
    svc_ctx: Arc<ServiceContext>,
}

impl {{.logic}} {
    pub fn new(svc_ctx: Arc<ServiceContext>) -> Self {
        Self { svc_ctx }
    }

    {{if .hasDoc}}{{.doc}}{{end}}
    #[instrument(skip(self{{if .request}}, req{{end}}))]
    pub async fn {{.function}}(
        &self{{if .request}},
        {{.request}}{{end}}
    ) {{.responseType}} {
        // todo: add your logic here and delete this line

        {{.returnString}}
    }
}
