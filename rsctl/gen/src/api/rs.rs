// Flattened: keep everything under `gen/src/api/rs.rs` (no `api/rs/` directory).

#[derive(Debug, Clone)]
pub struct Options {
    pub service_name: String,
    pub merge: bool,
    pub style: String,
    pub template_root: std::path::PathBuf,
}

fn generate_by_framework(
    framework: &str,
    spec: &spec::api::Spec,
    opts: &Options,
) -> anyhow::Result<crate::artifact::Artifacts> {
    shared::generate_go_like_tree(
        framework,
        spec,
        &opts.service_name,
        opts.merge,
        &opts.style,
        &opts.template_root,
    )
}

pub mod actix {
    pub type Options = super::Options;

    pub fn generate(
        spec: &spec::api::Spec,
        opts: &Options,
    ) -> anyhow::Result<crate::artifact::Artifacts> {
        super::generate_by_framework("actix", spec, opts)
    }
}

pub mod axum {
    pub type Options = super::Options;

    pub fn generate(
        spec: &spec::api::Spec,
        opts: &Options,
    ) -> anyhow::Result<crate::artifact::Artifacts> {
        super::generate_by_framework("axum", spec, opts)
    }
}

pub mod shared {
    use anyhow::{anyhow, Context, Result};
    use crate::artifact::{Artifact, Artifacts};
    use std::path::{Path, PathBuf};
    use utils::render;

    fn snake(s: &str) -> String {
        s.trim().to_ascii_lowercase()
    }

    fn service_name_from_input(service_name: &str) -> String {
        snake(service_name)
    }

    fn find_kv<'a>(ann: &'a spec::api::Annotation, key: &str) -> Option<&'a str> {
        match &ann.args {
            spec::api::AnnotationArgs::Map(kvs) => kvs
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str()),
            _ => None,
        }
    }

    fn find_annotation<'a>(
        anns: &'a [spec::api::Annotation],
        name: &str,
    ) -> Option<&'a spec::api::Annotation> {
        anns.iter().find(|a| a.name == name)
    }

    fn route_handler_name(route: &spec::api::Route) -> Option<String> {
        let Some(a) = find_annotation(&route.annotations, "handler") else {
            return None;
        };
        match &a.args {
            spec::api::AnnotationArgs::Str(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn service_group_name(service: &spec::api::Service) -> Option<String> {
        let Some(srv) = find_annotation(&service.annotations, "server") else {
            return None;
        };
        find_kv(srv, "group").map(|s| s.to_string())
    }

    fn service_prefix(service: &spec::api::Service) -> Option<String> {
        let Some(srv) = find_annotation(&service.annotations, "server") else {
            return None;
        };
        find_kv(srv, "prefix").map(|s| s.to_string())
    }

    fn join_paths(prefix: &str, path: &str) -> String {
        let pfx = prefix.trim_end_matches('/');
        let mut p = path.to_string();
        if !p.starts_with('/') {
            p.insert(0, '/');
        }
        if pfx.is_empty() {
            return p;
        }
        if p == "/" {
            return pfx.to_string();
        }
        format!("{pfx}{p}")
    }

    fn template_dir(template_root: &Path, framework: &str) -> PathBuf {
        let fw = template_root.join("api").join("rs").join(framework);
        if fw.is_dir() {
            return fw;
        }
        template_root.join("api").join("rs")
    }

    fn must_exist(dir: &Path, file: &str) -> Result<PathBuf> {
        let p = dir.join(file);
        if p.is_file() {
            Ok(p)
        } else {
            Err(anyhow!("missing template file: {}", p.display()))
        }
    }

    fn read_tpl(dir: &Path, file: &str) -> Result<String> {
        let p = must_exist(dir, file)?;
        std::fs::read_to_string(&p).with_context(|| format!("read template: {}", p.display()))
    }

    fn pascal(s: &str) -> String {
        let mut out = String::new();
        let mut upper = true;
        for ch in s.chars() {
            if ch == '_' || ch == '-' {
                upper = true;
                continue;
            }
            if upper {
                out.extend(ch.to_uppercase());
                upper = false;
            } else {
                out.push(ch);
            }
        }
        out
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Style {
        RustZeroSnake,
        RustZeroLowerCamel,
        RustZeroUpperCamel,
    }

    fn parse_style(style: &str) -> Result<Style> {
        match style {
            "rust_zero" => Ok(Style::RustZeroSnake),
            "rustZero" => Ok(Style::RustZeroLowerCamel),
            "RustZero" => Ok(Style::RustZeroUpperCamel),
            other => Err(anyhow!(
                "unsupported style: {other} (expected rust_zero|rustZero|RustZero)"
            )),
        }
    }

    fn lower_camel_from_snake(s: &str) -> String {
        let p = pascal(s);
        let mut it = p.chars();
        let Some(first) = it.next() else {
            return String::new();
        };
        first.to_lowercase().collect::<String>() + it.as_str()
    }

    fn group_file_base(group_snake: &str, style: Style) -> String {
        match style {
            Style::RustZeroSnake => group_snake.to_string(),
            Style::RustZeroLowerCamel => lower_camel_from_snake(group_snake),
            Style::RustZeroUpperCamel => pascal(group_snake),
        }
    }

    fn mod_decl_with_path(parent_dir: &str, module: &str, file_base: &str) -> String {
        // `module` stays snake_case for idiomatic rust modules.
        // Only the filename is styled; use `#[path="..."]` when they differ.
        if module == file_base {
            format!("pub mod {module};\n")
        } else {
            format!(
                "#[path = \"{parent_dir}/{file_base}.rs\"]\npub mod {module};\n"
            )
        }
    }

    pub fn generate_go_like_tree(
        framework: &str,
        spec: &spec::api::Spec,
        service_name: &str,
        merge: bool,
        style: &str,
        template_root: &Path,
    ) -> Result<Artifacts> {
        // Template root resolution:
        // Prefer `<template_root>/api/rs/<framework>/`, fallback to `<template_root>/api/rs/`.
        let tmpl_dir = template_dir(template_root, framework);
        let _ = must_exist(&tmpl_dir, "main.tpl")?;
        let _ = must_exist(&tmpl_dir, "config.tpl")?;
        let _ = must_exist(&tmpl_dir, "svc.tpl")?;
        let _ = must_exist(&tmpl_dir, "types.tpl")?;
        let _ = must_exist(&tmpl_dir, "handler.tpl")?;
        let _ = must_exist(&tmpl_dir, "logic.tpl")?;
        let _ = must_exist(&tmpl_dir, "etc.tpl")?;

        use std::collections::BTreeMap;

        let service_name = service_name_from_input(service_name);
        let style = parse_style(style)?;

        // `style` 是“命名风格”，不是包名的一部分；包名保持稳定且符合 Cargo 习惯（全小写 + 下划线）。
        let project_name = format!("rust_zero_{service_name}_{framework}");

        // group -> list of handler names (de-duplicated per service)
        let mut group_handlers: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for service in &spec.services {
            let group = service_group_name(service).unwrap_or_else(|| "index".to_string());
            let group = snake(&group);
            let mut hs: Vec<String> = Vec::new();
            for r in &service.routes {
                if let Some(h) = route_handler_name(r) {
                    hs.push(snake(&h));
                }
            }
            hs.sort();
            hs.dedup();
            group_handlers.entry(group).or_default().extend(hs);
        }
        for v in group_handlers.values_mut() {
            v.sort();
            v.dedup();
        }

        let mut files: Vec<Artifact> = Vec::new();

        // Base template context
        let base_ctx = render::Context::new()
            .set_str("version", "0.01")
            .set_str("serviceName", &service_name)
            .set_str("host", "0.0.0.0")
            .set_str("port", "8888")
            .set_str("importPackages", "")
            .set_str("ImportPackages", "")
            .set_str("imports", "")
            .set_str("configImport", "")
            .set_str("config", "crate::config::Config")
            .set_str("middleware", "")
            .set_str("middlewareAssignment", "");

        let main_tpl = read_tpl(&tmpl_dir, "main.tpl")?;
        let config_tpl = read_tpl(&tmpl_dir, "config.tpl")?;
        let svc_tpl = read_tpl(&tmpl_dir, "svc.tpl")?;
        let types_tpl = read_tpl(&tmpl_dir, "types.tpl")?;
        let handler_tpl = read_tpl(&tmpl_dir, "handler.tpl")?;
        let logic_tpl = read_tpl(&tmpl_dir, "logic.tpl")?;
        let etc_tpl = read_tpl(&tmpl_dir, "etc.tpl")?;

        // Project root
        files.push(Artifact {
            rel_path: "Cargo.toml".into(),
            content: format!(
                r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "{project_name}"
path = "main.rs"

[dependencies]
anyhow = "1"
axum = "0.6"
serde = {{ version = "1", features = ["derive"] }}
serde_yaml = "0.9"
tokio = {{ version = "1", features = ["macros", "rt-multi-thread", "signal"] }}
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
rest = {{ path = "rest" }}

[workspace]

"#
            ),
        });

        // local rest crate (minimal)
        files.push(Artifact {
            rel_path: "rest/Cargo.toml".into(),
            content: r#"[package]
                    name = "rest"
                    version = "0.1.0"
                    edition = "2024"

                    [dependencies]
                    serde = { version = "1", features = ["derive"] }
                    "#
            .to_string(),
        });
        files.push(Artifact {
            rel_path: "rest/src/lib.rs".into(),
            content: r#"use serde::{Deserialize, Serialize};

                    /// 最小 Rest 配置（对应 `etc/config.yaml` 的 Name/Host/Port）。
                    #[derive(Debug, Clone, Serialize, Deserialize)]
                    #[serde(rename_all = "PascalCase")]
                    pub struct RestConf {
                        pub name: String,
                        pub host: String,
                        pub port: u16,
                    }

                    impl Default for RestConf {
                        fn default() -> Self {
                            Self {
                                name: "api".to_string(),
                                host: "0.0.0.0".to_string(),
                                port: 8888,
                            }
                        }
                    }
                    "#
            .to_string(),
        });

        files.push(Artifact {
            rel_path: "etc/config.yaml".into(),
            content: render::render(&etc_tpl, &base_ctx).context("render etc.tpl")?,
        });

        // src root files
        files.push(Artifact {
            rel_path: "main.rs".into(),
            content: render::render(&main_tpl, &base_ctx).context("render main.tpl")?,
        });

        files.push(Artifact {
            rel_path: "config.rs".into(),
            content: render::render(&config_tpl, &base_ctx).context("render config.tpl")?,
        });
        files.push(Artifact {
            rel_path: "svc.rs".into(),
            content: render::render(&svc_tpl, &base_ctx).context("render svc.tpl")?,
        });

        // handler root + routes
        let mut handler_root = "/// generated by rsctl\n".to_string();
        handler_root.push_str("use std::sync::Arc;\n\n");
        handler_root.push_str("use axum::{\n");
        handler_root.push_str("    routing::{delete, get, patch, post, put},\n");
        handler_root.push_str("    Router,\n");
        handler_root.push_str("};\n");
        handler_root.push_str("use crate::svc::ServiceContext;\n\n");
        handler_root.push_str("pub mod routes;\n");
        for g in group_handlers.keys() {
            let file_base = group_file_base(g, style);
            handler_root.push_str(&mod_decl_with_path("handler", g, &file_base));
        }
        handler_root.push('\n');
        handler_root.push_str("pub fn register_handlers(svc_ctx: Arc<ServiceContext>) -> Router {\n");
        handler_root.push_str("    let mut app = Router::new().route(\"/healthz\", get(|| async { \"ok\" }));\n");
        let mut seen_routes = std::collections::BTreeSet::<(String, String)>::new();
        for service in &spec.services {
            let group = service_group_name(service).unwrap_or_else(|| "index".to_string());
            let group = snake(&group);
            let prefix = service_prefix(service).unwrap_or_default();
            for r in &service.routes {
                let Some(h) = route_handler_name(r) else { continue };
                let h = snake(&h);
                let full_path = join_paths(&prefix, &r.path);
                let method = match r.method {
                    spec::api::HttpMethod::Get => "get",
                    spec::api::HttpMethod::Post => "post",
                    spec::api::HttpMethod::Put => "put",
                    spec::api::HttpMethod::Delete => "delete",
                    spec::api::HttpMethod::Patch => "patch",
                };
                let key = (method.to_string(), full_path.clone());
                if !seen_routes.insert(key) {
                    continue;
                }
                handler_root.push_str(&format!(
                    "    app = app.route(\"{full_path}\", {method}(crate::handler::{group}::handler::{h}));\n"
                ));
            }
        }
        handler_root.push_str("    app.with_state(svc_ctx)\n");
        handler_root.push_str("}\n");
        files.push(Artifact {
            rel_path: "handler.rs".into(),
            content: handler_root,
        });
        files.push(Artifact {
            rel_path: "handler/routes.rs".into(),
            content: "/// generated by rsctl\npub fn register_routes() {}\n".to_string(),
        });

        // logic root
        let mut logic_root = "/// generated by rsctl\n".to_string();
        for g in group_handlers.keys() {
            let file_base = group_file_base(g, style);
            logic_root.push_str(&mod_decl_with_path("logic", g, &file_base));
        }
        files.push(Artifact {
            rel_path: "logic.rs".into(),
            content: logic_root,
        });

        // types root + module dir
        let mut types_root = "/// generated by rsctl\n".to_string();
        for g in group_handlers.keys() {
            let file_base = group_file_base(g, style);
            types_root.push_str(&mod_decl_with_path("types", g, &file_base));
        }
        files.push(Artifact {
            rel_path: "types.rs".into(),
            content: types_root,
        });

        // per-group modules
        for (g, hs) in &group_handlers {
            let g_file = group_file_base(g, style);
            files.push(Artifact {
                rel_path: format!("handler/{g_file}.rs").into(),
                content: format!(
                    "/// generated by rsctl\n#[path = \"{g}/handler.rs\"]\npub mod handler;\n"
                ),
            });
            files.push(Artifact {
                rel_path: format!("handler/{g}/handler.rs").into(),
                content: {
                    let mut s = String::new();
                    s.push_str("// Code scaffolded by rsctl. Safe to edit.\n");
                    s.push_str("// rsctl 0.01\n\n");
                    s.push_str("use std::sync::Arc;\n");
                    s.push_str("use axum::{\n");
                    s.push_str("    extract::{State, Json},\n");
                    s.push_str("    http::StatusCode,\n");
                    s.push_str("    response::IntoResponse,\n");
                    s.push_str("};\n");
                    s.push_str("use crate::svc::ServiceContext;\n\n");
                    for h in hs {
                        // Find a representative route in this group with this handler.
                        // We only use it to populate request/response/doc metadata.
                        let mut req: Option<String> = None;
                        let mut resp: Option<String> = None;
                        let mut doc: Option<String> = None;
                        'outer: for service in &spec.services {
                            let gg = service_group_name(service).unwrap_or_else(|| "index".to_string());
                            if snake(&gg) != *g {
                                continue;
                            }
                            for r in &service.routes {
                                let Some(hn) = route_handler_name(r) else { continue };
                                if snake(&hn) != *h {
                                    continue;
                                }
                                req = r.request.clone();
                                resp = r.response.clone();
                                // doc annotation
                                if let Some(a) = r.annotations.iter().find(|a| a.name == "doc") {
                                    if let spec::api::AnnotationArgs::Str(s) = &a.args {
                                        doc = Some(s.clone());
                                    }
                                }
                                break 'outer;
                            }
                        }

                        let req_ty = req
                            .as_deref()
                            .map(|t| format!("{g}::types::{}", t))
                            .unwrap_or_default();
                        let resp_ty = resp
                            .as_deref()
                            .map(|t| format!("{g}::types::{}", t))
                            .unwrap_or_default();

                        let ctx = base_ctx
                            .clone()
                            .set_bool("HasRequest", req.is_some())
                            .set_bool("HasResp", resp.is_some())
                            .set_bool("HasDoc", doc.is_some())
                            .set_str(
                                "Doc",
                                doc.as_ref()
                                    .map(|d| format!("/// {}\n", d))
                                    .unwrap_or_default(),
                            )
                            .set_str("HandlerName", h)
                            .set_str("RequestType", req_ty)
                            .set_str("ResponseType", resp_ty)
                            .set_str("LogicName", g)
                            .set_str("LogicType", format!("logic::{}Logic", pascal(h)))
                            .set_str("Call", h);

                        s.push_str(&render::render(&handler_tpl, &ctx)?);
                        s.push('\n');
                    }
                    if !merge {
                        // Minimal behavior: if merge=false, still keep one file per group for now.
                        // (Future: split into per-handler module files.)
                    }
                    s
                },
            });

            files.push(Artifact {
                rel_path: format!("logic/{g_file}.rs").into(),
                content: format!(
                    "/// generated by rsctl\n#[path = \"{g}/logic.rs\"]\npub mod logic;\n"
                ),
            });
            files.push(Artifact {
                rel_path: format!("logic/{g}/logic.rs").into(),
                content: {
                    let mut s = String::new();
                    s.push_str("// Code scaffolded by rsctl. Safe to edit.\n");
                    s.push_str("// rsctl 0.01\n\n");
                    s.push_str("use std::sync::Arc;\n");
                    s.push_str("use tracing::instrument;\n");
                    s.push_str("use crate::svc::ServiceContext;\n\n");
                    for h in hs {
                        // Find route meta again for request/response.
                        let mut req: Option<String> = None;
                        let mut resp: Option<String> = None;
                        'outer2: for service in &spec.services {
                            let gg = service_group_name(service).unwrap_or_else(|| "index".to_string());
                            if snake(&gg) != *g {
                                continue;
                            }
                            for r in &service.routes {
                                let Some(hn) = route_handler_name(r) else { continue };
                                if snake(&hn) != *h {
                                    continue;
                                }
                                req = r.request.clone();
                                resp = r.response.clone();
                                break 'outer2;
                            }
                        }

                        let logic_name = format!("{}Logic", pascal(h));
                        let has_req = req.is_some();
                        let has_resp = resp.is_some();
                        let req_param = req
                            .as_deref()
                            .map(|t| format!("req: crate::types::{g}::types::{t}"))
                            .unwrap_or_default();
                        let resp_ty = resp
                            .as_deref()
                            .map(|t| format!("crate::types::{g}::types::{t}"))
                            .unwrap_or_else(|| "()".to_string());

                        let response_type = format!("anyhow::Result<{resp_ty}>");
                        let return_string = if has_resp {
                            format!("Ok({resp_ty} {{}})")
                        } else {
                            "Ok(())".to_string()
                        };

                        let ctx = base_ctx
                            .clone()
                            .set_str("logic", logic_name)
                            .set_str("function", h)
                            .set_str("responseType", response_type)
                            .set_str("returnString", return_string)
                            .set_str("imports", "")
                            .set_bool("hasDoc", false)
                            .set_bool("request", has_req)
                            .set_str("request", req_param);
                        s.push_str(&render::render(&logic_tpl, &ctx)?);
                        s.push('\n');
                    }
                    if !merge {
                        // Minimal behavior: keep group-level file even when merge=false.
                    }
                    s
                },
            });

            files.push(Artifact {
                rel_path: format!("types/{g_file}.rs").into(),
                content: format!(
                    "/// generated by rsctl\n#[path = \"{g}/types.rs\"]\npub mod types;\n"
                ),
            });
            files.push(Artifact {
                rel_path: format!("types/{g}/types.rs").into(),
                content: {
                    // minimal type stubs for referenced request/response types in this group
                    let mut decls: Vec<String> = Vec::new();
                    let mut seen = std::collections::BTreeSet::<String>::new();
                    for service in &spec.services {
                        let gg = service_group_name(service).unwrap_or_else(|| "index".to_string());
                        if snake(&gg) != *g {
                            continue;
                        }
                        for r in &service.routes {
                            if let Some(t) = &r.request {
                                seen.insert(t.clone());
                            }
                            if let Some(t) = &r.response {
                                seen.insert(t.clone());
                            }
                        }
                    }
                    for t in seen {
                        decls.push(format!(
                            "#[derive(Debug, Clone, Serialize, Deserialize, Default)]\npub struct {} {{}}\n",
                            t
                        ));
                    }
                    let ctx = base_ctx.clone().set_bool("containsTime", false).set_str("types", decls.join("\n"));
                    render::render(&types_tpl, &ctx)?
                },
            });
        }

        Ok(Artifacts { files })
    }
}


