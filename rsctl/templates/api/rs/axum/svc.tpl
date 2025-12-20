// placeholder (axum) - reuse `templates/api/rs/svc.tpl`

// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

{{.configImport}}

pub struct ServiceContext {
    pub config: {{.config}},
    {{.middleware}}
}

impl ServiceContext {
    pub fn new(config: {{.config}}) -> Self {
        Self {
            config,
            {{.middlewareAssignment}}
        }
    }
}


