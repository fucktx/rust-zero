// Flattened: keep everything under `gen/src/api/rs.rs` (no `api/rs/` directory).

#[derive(Debug, Clone)]
pub struct Options {
    pub service_name: String,
    pub merge: bool,
    pub style: String,
    /// 输出目录（用于计算对 monorepo `rest` crate 的相对路径）。
    pub out_dir: std::path::PathBuf,
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
        &opts.out_dir,
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

    fn relative_path(from: &Path, to: &Path) -> PathBuf {
        // A small, dependency-free relative path helper.
        // - If paths have different prefixes (e.g. different drive letters), fallback to absolute `to`.
        let from = from.components().collect::<Vec<_>>();
        let to = to.components().collect::<Vec<_>>();
        let mut i = 0usize;
        while i < from.len() && i < to.len() && from[i] == to[i] {
            i += 1;
        }
        // If the common prefix is empty and either begins with a prefix/root, just return `to`.
        if i == 0 {
            return to.iter().map(|c| c.as_os_str()).collect::<PathBuf>();
        }
        let mut out = PathBuf::new();
        for _ in i..from.len() {
            out.push("..");
        }
        for c in &to[i..] {
            out.push(c.as_os_str());
        }
        if out.as_os_str().is_empty() {
            out.push(".");
        }
        out
    }

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

    fn effective_group(service: &spec::api::Service) -> String {
        // go-zero 习惯：没写 group 时，默认用 service 名作为 group。
        service_group_name(service).unwrap_or_else(|| service.name.clone())
    }

    fn service_prefix(service: &spec::api::Service) -> Option<String> {
        let Some(srv) = find_annotation(&service.annotations, "server") else {
            return None;
        };
        find_kv(srv, "prefix").map(|s| s.to_string())
    }

    fn service_middleware(service: &spec::api::Service) -> Option<String> {
        let Some(srv) = find_annotation(&service.annotations, "server") else {
            return None;
        };
        find_kv(srv, "middleware").map(|s| s.to_string())
    }

    fn service_jwt(service: &spec::api::Service) -> Option<String> {
        let Some(srv) = find_annotation(&service.annotations, "server") else {
            return None;
        };
        find_kv(srv, "jwt").map(|s| s.to_string())
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
        out_dir: &Path,
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
        // 入口文件名：对齐 go-zero 的“服务名即入口”（例如 `user.rs`）。
        let entry_file = format!("{service_name}.rs");

        // group -> list of handler names (de-duplicated per service)
        let mut group_handlers: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for service in &spec.services {
            let group = effective_group(service);
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
        // NOTE:
        // - 生成的工程默认依赖当前仓库内的 `rest` crate（用于 `rest::router!` DSL）。
        // - 依赖路径通过 `template_root` 推导出 monorepo 根，再计算相对路径，保证 rsctl/test/out 默认可用。
        let monorepo_root = template_root
            .parent()
            .and_then(|p| p.parent())
            .ok_or_else(|| anyhow!("invalid template_root: {}", template_root.display()))?;
        let rest_crate_path = monorepo_root.join("rest");
        let rest_rel = relative_path(out_dir, &rest_crate_path);
        let rest_rel = rest_rel.to_string_lossy().to_string();

        // 收集 @server(jwt: Xxx) 的 jwt 名称（用于生成 config 字段）
        use std::collections::BTreeSet;
        let mut jwt_names: BTreeSet<String> = BTreeSet::new();
        for s in &spec.services {
            if let Some(j) = service_jwt(s) {
                jwt_names.insert(snake(&j));
            }
        }

        // 是否需要启用 rest 的 jwt feature（由 @server(jwt: ...) 决定）
        let use_jwt = !jwt_names.is_empty();
        let rest_features = if use_jwt {
            "[\"axum\", \"jwt\"]"
        } else {
            "[\"axum\"]"
        };

        files.push(Artifact {
            rel_path: "Cargo.toml".into(),
            content: format!(
                r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "{project_name}"
path = "{entry_file}"

[dependencies]
anyhow = "1"
axum = "0.6"
http = "0.2"
serde = {{ version = "1", features = ["derive"] }}
serde_yaml = "0.9"
tokio = {{ version = "1", features = ["macros", "rt-multi-thread", "signal"] }}
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
rest = {{ path = "{rest_rel}", features = {rest_features} }}

[workspace]

"#
            ),
        });

        files.push(Artifact {
            rel_path: "etc/config.yaml".into(),
            content: {
                let mut c = render::render(&etc_tpl, &base_ctx).context("render etc.tpl")?;
                // 追加 jwt 配置段落（按注解名生成）
                // 形如：
                // Auth:
                //   AccessSecret: ...
                if !jwt_names.is_empty() {
                    c.push('\n');
                    for j in &jwt_names {
                        // config.yaml 采用 PascalCase key
                        let key = {
                            let mut ch = j.chars();
                            match ch.next() {
                                None => j.clone(),
                                Some(f) => f.to_ascii_uppercase().to_string() + ch.as_str(),
                            }
                        };
                        c.push_str(&format!("{key}:\n  AccessSecret: \"changeme\"\n"));
                    }
                }
                c
            },
        });

        // src root files
        files.push(Artifact {
            rel_path: entry_file.clone().into(),
            content: render::render(&main_tpl, &base_ctx).context("render main.tpl")?,
        });

        // 注意：不再“额外”生成 main.rs（stub）。
        // 入口文件名规则：`<service>.rs`；若 service == "main" 则入口自然是 `main.rs`。

        // config.rs：若使用了 @server(jwt: Xxx) 则生成对应字段（Xxx.access_secret）。
        // 为避免对模板做脆弱的字符串替换，jwt_names 非空时直接生成完整文件内容。
        files.push(Artifact {
            rel_path: "config.rs".into(),
            content: if jwt_names.is_empty() {
                render::render(&config_tpl, &base_ctx).context("render config.tpl")?
            } else {
                let mut cfg_rs = String::new();
                cfg_rs.push_str("// Code scaffolded by rsctl. Safe to edit.\n");
                cfg_rs.push_str("// rsctl 0.01\n\n");
                cfg_rs.push_str("use serde::Deserialize;\n");
                cfg_rs.push_str("use rest::RestConf;\n");
                cfg_rs.push_str("use anyhow::Context;\n");
                cfg_rs.push_str("use std::fs;\n\n");

                cfg_rs.push_str("#[derive(Debug, Clone, Deserialize)]\n");
                cfg_rs.push_str("pub struct JwtConf {\n");
                cfg_rs.push_str("    #[serde(rename = \"AccessSecret\")]\n");
                cfg_rs.push_str("    pub access_secret: String,\n");
                cfg_rs.push_str("}\n\n");

                cfg_rs.push_str("#[derive(Debug, Clone, Deserialize)]\n");
                cfg_rs.push_str("pub struct Config {\n");
                cfg_rs.push_str("    #[serde(flatten)]\n");
                cfg_rs.push_str("    pub rest: RestConf,\n");
                for j in &jwt_names {
                    let key = {
                        let mut ch = j.chars();
                        match ch.next() {
                            None => j.clone(),
                            Some(f) => f.to_ascii_uppercase().to_string() + ch.as_str(),
                        }
                    };
                    cfg_rs.push_str(&format!("    #[serde(rename = \"{key}\")]\n"));
                    cfg_rs.push_str(&format!("    pub {j}: JwtConf,\n"));
                }
                cfg_rs.push_str("}\n\n");

                cfg_rs.push_str("impl Default for Config {\n");
                cfg_rs.push_str("    fn default() -> Self {\n");
                cfg_rs.push_str("        Self {\n");
                cfg_rs.push_str("            rest: RestConf::default(),\n");
                for j in &jwt_names {
                    cfg_rs.push_str(&format!(
                        "            {j}: JwtConf {{ access_secret: String::new() }},\n"
                    ));
                }
                cfg_rs.push_str("        }\n");
                cfg_rs.push_str("    }\n");
                cfg_rs.push_str("}\n\n");

                cfg_rs.push_str("pub fn load_config() -> Config {\n");
                cfg_rs.push_str("    Config::default()\n");
                cfg_rs.push_str("}\n\n");

                cfg_rs.push_str("/// 从 YAML 配置文件加载。\n");
                cfg_rs.push_str("pub fn load(path: &str) -> anyhow::Result<Config> {\n");
                cfg_rs.push_str(
                    "    let raw = fs::read_to_string(path).with_context(|| format!(\"read config: {path}\"))?;\n",
                );
                cfg_rs.push_str(
                    "    let cfg: Config = serde_yaml::from_str(&raw).context(\"parse yaml config\")?;\n",
                );
                cfg_rs.push_str("    Ok(cfg)\n");
                cfg_rs.push_str("}\n");

                cfg_rs
            },
        });
        files.push(Artifact {
            rel_path: "svc.rs".into(),
            content: render::render(&svc_tpl, &base_ctx).context("render svc.tpl")?,
        });

        // handler root + routes
        let mut handler_root = "/// generated by rsctl\n".to_string();
        handler_root.push_str("use std::sync::Arc;\n\n");
        handler_root.push_str("use axum::Router;\n");
        handler_root.push_str("use crate::svc::ServiceContext;\n\n");
        handler_root.push_str("pub mod routes;\n");
        for g in group_handlers.keys() {
            let file_base = group_file_base(g, style);
            handler_root.push_str(&mod_decl_with_path("handler", g, &file_base));
        }
        handler_root.push('\n');
        handler_root.push_str("pub fn register_handlers(svc_ctx: Arc<ServiceContext>) -> Router {\n");
        handler_root.push_str("    routes::register_routes(svc_ctx)\n");
        handler_root.push_str("}\n");
        files.push(Artifact {
            rel_path: "handler.rs".into(),
            content: handler_root,
        });

        // routes.rs：路由表（go-zero 风格：集中定义路由 + options）
        {
            let mut routes_rs = String::new();
            routes_rs.push_str("/// generated by rsctl\n");
            routes_rs.push_str("use std::sync::Arc;\n\n");
            routes_rs.push_str("use axum::Router;\n");
            routes_rs.push_str("use crate::svc::ServiceContext;\n\n");
            routes_rs.push_str("pub fn register_routes(svc_ctx: Arc<ServiceContext>) -> Router {\n");
            routes_rs.push_str("    let mut app = Router::new().route(\"/healthz\", axum::routing::get(|| async { \"ok\" }));\n");

            // 逐个 service 生成 add_routes，才能保留 @server(middleware/jwt/...) 的 option 信息。
            // 但如果同一个 method+full_path 在多个 service 块里重复出现，axum 会在启动时 panic。
            // 这里做一个 deterministic 的“择优”：
            // - 优先选择声明了 jwt/middleware 的 service 块（更接近 go-zero 的意图）
            // - 再其次选择声明了 prefix 的
            // - 若优先级相同但来源不同，则直接报错提示用户去重
            #[derive(Clone)]
            struct Winner {
                score: i32,
                service_idx: usize,
            }
            let mut winners: BTreeMap<String, Winner> = BTreeMap::new();

            let score_of = |prefix: &str, mw: &Option<String>, jwt: &Option<String>| -> i32 {
                let mut s = 0;
                if jwt.is_some() { s += 100; }
                if mw.is_some() { s += 10; }
                if !prefix.trim().is_empty() { s += 1; }
                s
            };

            for (idx, service) in spec.services.iter().enumerate() {
                let prefix = service_prefix(service).unwrap_or_default();
                let mw = service_middleware(service);
                let jwt = service_jwt(service);
                let score = score_of(&prefix, &mw, &jwt);

                for r in &service.routes {
                    let Some(_h) = route_handler_name(r) else { continue };
                    let method = match r.method {
                        spec::api::HttpMethod::Get => "GET",
                        spec::api::HttpMethod::Post => "POST",
                        spec::api::HttpMethod::Put => "PUT",
                        spec::api::HttpMethod::Delete => "DELETE",
                        spec::api::HttpMethod::Patch => "PATCH",
                    };
                    let full_path = join_paths(&prefix, &r.path);
                    let route_key = format!("{method} {full_path}");

                    match winners.get(&route_key) {
                        None => {
                            winners.insert(route_key, Winner { score, service_idx: idx });
                        }
                        Some(prev) if score > prev.score => {
                            winners.insert(route_key, Winner { score, service_idx: idx });
                        }
                        Some(prev) if score == prev.score && prev.service_idx != idx => {
                            return Err(anyhow::anyhow!(
                                "duplicate route detected with same priority: `{}`.\nPlease remove duplicates or keep only one @server(...) variant.",
                                route_key
                            ));
                        }
                        _ => {}
                    }
                }
            }

            for (idx, service) in spec.services.iter().enumerate() {
                let group = effective_group(service);
                let group = snake(&group);
                let prefix = service_prefix(service).unwrap_or_default();
                let mw = service_middleware(service);
                let jwt = service_jwt(service);

                routes_rs.push_str("    app = app.merge(rest::add_routes!(\n");
                routes_rs.push_str("        [\n");

                let mut any = false;
                for r in &service.routes {
                    let Some(h) = route_handler_name(r) else { continue };
                    let h = snake(&h);
                    let method = match r.method {
                        spec::api::HttpMethod::Get => "GET",
                        spec::api::HttpMethod::Post => "POST",
                        spec::api::HttpMethod::Put => "PUT",
                        spec::api::HttpMethod::Delete => "DELETE",
                        spec::api::HttpMethod::Patch => "PATCH",
                    };

                    let full_path = join_paths(&prefix, &r.path);
                    let route_key = format!("{method} {full_path}");
                    let Some(w) = winners.get(&route_key) else { continue };
                    if w.service_idx != idx {
                        continue;
                    }

                    any = true;
                    routes_rs.push_str(&format!(
                        "            {{ method: http::Method::{method}, path: \"{}\", handler: crate::handler::{}::handler::{}(svc_ctx.clone()) }},\n",
                        r.path, group, h
                    ));
                }

                routes_rs.push_str("        ],\n");
                if !prefix.trim().is_empty() {
                    routes_rs.push_str(&format!("        rest::WithPrefix(\"{prefix}\"),\n"));
                }
                if let Some(mw_name) = mw {
                    let mw_mod = snake(&mw_name);
                    routes_rs.push_str(&format!(
                        "        // @server(middleware: {mw_name})\n        rest::WithMiddleware(axum::middleware::from_fn(crate::middleware::{mw_mod}::handle)),\n"
                    ));
                }
                if let Some(jwt_name) = jwt {
                    let jwt_field = snake(&jwt_name);
                    routes_rs.push_str(&format!(
                        "        // @server(jwt: {jwt_name})\n        rest::WithJwt(svc_ctx.config.{jwt_field}.access_secret.clone()),\n"
                    ));
                }
                routes_rs.push_str("    ));\n");

                let _ = any; // 允许空块（即便没有 routes），保持结构简单
            }

            routes_rs.push_str("    app\n");
            routes_rs.push_str("}\n");

            files.push(Artifact {
                rel_path: "handler/routes.rs".into(),
                content: routes_rs,
            });
        }

        // middleware 模块：仅当 api 里声明了 @server(middleware: ...) 才生成占位文件
        {
            use std::collections::BTreeSet;
            let mut mws: BTreeSet<String> = BTreeSet::new();
            for s in &spec.services {
                if let Some(m) = service_middleware(s) {
                    mws.insert(snake(&m));
                }
            }
            if !mws.is_empty() {
                let mut root = String::new();
                root.push_str("/// generated by rsctl\n");
                root.push_str("pub mod prelude;\n");
                for m in &mws {
                    root.push_str(&format!("pub mod {m};\n"));
                }
                files.push(Artifact {
                    rel_path: "middleware.rs".into(),
                    content: root,
                });
                files.push(Artifact {
                    rel_path: "middleware/prelude.rs".into(),
                    content: "// generated by rsctl\n".to_string(),
                });
                for m in &mws {
                    files.push(Artifact {
                        rel_path: format!("middleware/{m}.rs").into(),
                        content: format!(
                            r#"// Code scaffolded by rsctl. Safe to edit.
// rsctl 0.01
use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;

/// {m} middleware handler（占位实现）：后续把鉴权/日志等逻辑写在这里。
pub async fn handle(req: Request<Body>, next: Next<Body>) -> Response {{
    // TODO: {m} middleware logic
    next.run(req).await
}}
"#,
                            m = m
                        ),
                    });
                }
            }
        }

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
                    s.push_str("    extract::Json,\n");
                    s.push_str("    http::StatusCode,\n");
                    s.push_str("    response::IntoResponse,\n");
                    s.push_str("    routing::{delete, get, patch, post, put, MethodRouter},\n");
                    s.push_str("};\n");
                    s.push_str("use crate::svc::ServiceContext;\n\n");
                    for h in hs {
                        // Find a representative route in this group with this handler.
                        // We only use it to populate request/response/doc metadata.
                        let mut req: Option<String> = None;
                        let mut resp: Option<String> = None;
                        let mut http_method: Option<spec::api::HttpMethod> = None;
                        let mut doc: Option<String> = None;
                        'outer: for service in &spec.services {
                            let gg = effective_group(service);
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
                                http_method = Some(r.method.clone());
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
                            .set_str("Call", h)
                            .set_str(
                                "AxumMethodFn",
                                match http_method.unwrap_or(spec::api::HttpMethod::Get) {
                                    spec::api::HttpMethod::Get => "get",
                                    spec::api::HttpMethod::Post => "post",
                                    spec::api::HttpMethod::Put => "put",
                                    spec::api::HttpMethod::Delete => "delete",
                                    spec::api::HttpMethod::Patch => "patch",
                                },
                            );

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
                            let gg = effective_group(service);
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
                        let gg = effective_group(service);
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


