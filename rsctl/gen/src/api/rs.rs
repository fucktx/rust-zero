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
    use crate::artifact::{Artifact, Artifacts};
    use anyhow::{Context, Result, anyhow};
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

    fn find_monorepo_root(out_dir: &Path) -> Option<PathBuf> {
        // `out_dir` 可能是相对路径（如 `tests/out`），且生成时目录未必已存在；
        // 因此用当前工作目录拼成一个可向上遍历的绝对/半绝对路径。
        let mut dir = std::env::current_dir().ok()?.join(out_dir);
        loop {
            if dir
                .join("crates")
                .join("rust-zero")
                .join("Cargo.toml")
                .is_file()
            {
                return Some(dir);
            }
            if !dir.pop() {
                break;
            }
        }
        None
    }

    fn snake(s: &str) -> String {
        s.trim().to_ascii_lowercase()
    }

    fn service_name_from_input(service_name: &str) -> String {
        snake(service_name)
    }

    fn find_kv<'a>(ann: &'a spec::api::Annotation, key: &str) -> Option<&'a str> {
        match &ann.args {
            spec::api::AnnotationArgs::Map(kvs) => {
                kvs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
            }
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
        let a = find_annotation(&route.annotations, "handler")?;
        match &a.args {
            spec::api::AnnotationArgs::Str(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn service_group_name(service: &spec::api::Service) -> Option<String> {
        let srv = find_annotation(&service.annotations, "server")?;
        find_kv(srv, "group").map(|s| s.to_string())
    }

    fn effective_group(service: &spec::api::Service) -> String {
        // go-zero 习惯：没写 group 时，默认用 service 名作为 group。
        service_group_name(service).unwrap_or_else(|| service.name.clone())
    }

    fn service_prefix(service: &spec::api::Service) -> Option<String> {
        let srv = find_annotation(&service.annotations, "server")?;
        find_kv(srv, "prefix").map(|s| s.to_string())
    }

    fn service_middleware(service: &spec::api::Service) -> Option<String> {
        let srv = find_annotation(&service.annotations, "server")?;
        find_kv(srv, "middleware").map(|s| s.to_string())
    }

    fn service_jwt(service: &spec::api::Service) -> Option<String> {
        let srv = find_annotation(&service.annotations, "server")?;
        find_kv(srv, "jwt").map(|s| s.to_string())
    }

    fn tag_kv_map(tag: &str) -> std::collections::BTreeMap<String, String> {
        // Parse `json:"x",validate:"min=1,max=2"` (order-insensitive, best-effort).
        let mut out = std::collections::BTreeMap::new();
        let mut i = 0usize;
        let b = tag.as_bytes();
        while i < b.len() {
            while i < b.len() && (b[i] == b' ' || b[i] == b',' || b[i] == b'\t') {
                i += 1;
            }
            let start = i;
            while i < b.len() && (b[i] as char).is_ascii_alphanumeric() {
                i += 1;
            }
            if i == start || i >= b.len() || b[i] != b':' {
                break;
            }
            let key = &tag[start..i];
            i += 1; // skip ':'
            if i >= b.len() || b[i] != b'"' {
                break;
            }
            i += 1; // skip '"'
            let val_start = i;
            while i < b.len() && b[i] != b'"' {
                i += 1;
            }
            if i >= b.len() {
                break;
            }
            let val = &tag[val_start..i];
            i += 1; // skip closing '"'
            out.insert(key.to_string(), val.to_string());
        }
        out
    }

    fn api_ty_to_rust(g: &str, api_ty: &str) -> String {
        // `[]T` -> `Vec<T>`
        if let Some(inner) = api_ty.strip_prefix("[]") {
            return format!("Vec<{}>", api_ty_to_rust(g, inner));
        }

        match api_ty {
            "string" => "String".into(),
            "bool" => "bool".into(),
            "int" | "int64" => "i64".into(),
            "int32" => "i32".into(),
            "uint" | "uint64" => "u64".into(),
            "uint32" => "u32".into(),
            "float32" => "f32".into(),
            "float64" => "f64".into(),
            other => format!("crate::types::{g}::types::{other}"),
        }
    }

    fn validate_attrs(field_rust_ty: &str, validate_tag: &str) -> Vec<String> {
        // Map a small, useful subset of go-validate tags to `validator` crate:
        // - string: min/max -> length(min/max)
        // - numeric: min/max -> range(min/max)
        // - email/url/required -> corresponding validators
        let mut out = Vec::new();
        let items = validate_tag
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        let is_string = field_rust_ty == "String";
        let is_num = matches!(field_rust_ty, "i64" | "i32" | "u64" | "u32" | "f64" | "f32");

        let mut min_v: Option<i64> = None;
        let mut max_v: Option<i64> = None;
        for it in &items {
            if *it == "email" {
                out.push("#[validate(email)]".into());
                continue;
            }
            if *it == "url" {
                out.push("#[validate(url)]".into());
                continue;
            }
            if *it == "required" {
                out.push("#[validate(required)]".into());
                continue;
            }
            if let Some((k, v)) = it.split_once('=') {
                let v = v.parse::<i64>().ok();
                match (k.trim(), v) {
                    ("min", Some(n)) => min_v = Some(n),
                    ("max", Some(n)) => max_v = Some(n),
                    _ => {}
                }
            }
        }

        if is_string {
            let mut parts = Vec::new();
            if let Some(n) = min_v
                && n > 0
            {
                parts.push(format!("min = {n}"));
            }
            if let Some(n) = max_v
                && n > 0
            {
                parts.push(format!("max = {n}"));
            }
            if !parts.is_empty() {
                out.push(format!("#[validate(length({}))]", parts.join(", ")));
            }
        } else if is_num {
            let mut parts = Vec::new();
            if let Some(n) = min_v {
                parts.push(format!("min = {n}"));
            }
            if let Some(n) = max_v {
                parts.push(format!("max = {n}"));
            }
            if !parts.is_empty() {
                out.push(format!("#[validate(range({}))]", parts.join(", ")));
            }
        }

        out
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

    fn indent_block(s: &str, spaces: usize) -> String {
        let pad = " ".repeat(spaces);
        s.lines()
            .map(|l| {
                if l.trim().is_empty() {
                    String::new()
                } else {
                    format!("{pad}{l}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    fn template_dir(template_root: &Path) -> PathBuf {
        // 模板目录固定为 `templates/api/rs/`（不按框架分目录）。
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
        Snake,
        LowerCamel,
        UpperCamel,
    }

    fn parse_style(style: &str) -> Result<Style> {
        match style {
            "rust_zero" => Ok(Style::Snake),
            "rustZero" => Ok(Style::LowerCamel),
            "RustZero" => Ok(Style::UpperCamel),
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
            Style::Snake => group_snake.to_string(),
            Style::LowerCamel => lower_camel_from_snake(group_snake),
            Style::UpperCamel => pascal(group_snake),
        }
    }

    fn mod_decl_with_path(parent_dir: &str, module: &str, file_base: &str) -> String {
        // `module` stays snake_case for idiomatic rust modules.
        // Only the filename is styled; use `#[path="..."]` when they differ.
        if module == file_base {
            format!("pub mod {module};\n")
        } else {
            format!("#[path = \"{parent_dir}/{file_base}.rs\"]\npub mod {module};\n")
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
        // 模板目录固定为 `<template_root>/api/rs/`（不按框架分目录）。
        //
        // NOTE:
        // - 模板保持 goctl 的变量命名，以便未来直接替换为 goctl 官方模板。
        // - 框架差异在代码层（axum/actix）处理。
        let tmpl_dir = template_dir(template_root);
        // 依赖模板文件（变量名保持 goctl 风格，但内容已调整为 Rust/YAML）：
        // - `main.tpl`: 入口（Rust）
        // - `config.tpl`: 配置（Rust）
        // - `context.tpl`: ServiceContext（Rust，历史命名保留）
        // - `routes.tpl`: 路由表（Rust）
        // - `handler.tpl`: handler（Rust）
        // - `logic.tpl`: logic（Rust）
        // - `types.tpl`: types（Rust）
        // - `middleware.tpl`: middleware（Rust）
        // - `etc.tpl`: 默认配置文件（YAML）
        let _ = must_exist(&tmpl_dir, "context.tpl")?;
        let _ = must_exist(&tmpl_dir, "etc.tpl")?;
        let _ = must_exist(&tmpl_dir, "main.tpl")?;
        let _ = must_exist(&tmpl_dir, "config.tpl")?;
        let _ = must_exist(&tmpl_dir, "routes.tpl")?;
        let _ = must_exist(&tmpl_dir, "handler.tpl")?;
        let _ = must_exist(&tmpl_dir, "logic.tpl")?;
        let _ = must_exist(&tmpl_dir, "types.tpl")?;
        let _ = must_exist(&tmpl_dir, "middleware.tpl")?;

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
        let main_attr = if framework == "actix" {
            "#[actix_web::main]"
        } else {
            "#[tokio::main]"
        };
        let handler_imports = if framework == "actix" {
            "use actix_web::HttpResponse;\nuse actix_web::web;\n"
        } else {
            "use axum::http::StatusCode;\nuse axum::response::IntoResponse;\nuse axum::Extension;\nuse axum::Json;\n"
        };
        let server_start = if framework == "actix" {
            "    rust_zero::rest::server::start(engine, app)\n        .await\n        .map_err(anyhow::Error::from)\n        .context(\"actix serve\")?;"
        } else {
            "    rust_zero::rest::server::start(engine, app).await.context(\"axum serve\")?;"
        };

        let rsctl_version = std::env::var("RSCTL_VERSION").unwrap_or_else(|_| "unknown".into());
        let base_ctx = render::Context::new()
            .set_str("version", &rsctl_version)
            .set_str("serviceName", &service_name)
            .set_str("host", "0.0.0.0")
            .set_str("port", "8888")
            .set_str("importPackages", "")
            .set_str("ImportPackages", "")
            .set_str("imports", "")
            .set_str("configImport", "")
            .set_str("config", "crate::config::Config")
            .set_str("middleware", "")
            .set_str("middlewareAssignment", "")
            .set_str("mainAttr", main_attr)
            .set_str("handlerImports", handler_imports)
            .set_str("serverStart", server_start);

        let context_tpl = read_tpl(&tmpl_dir, "context.tpl")?;
        let etc_tpl = read_tpl(&tmpl_dir, "etc.tpl")?;
        let main_tpl = read_tpl(&tmpl_dir, "main.tpl")?;
        let config_tpl = read_tpl(&tmpl_dir, "config.tpl")?;
        let routes_tpl = read_tpl(&tmpl_dir, "routes.tpl")?;
        let handler_tpl = read_tpl(&tmpl_dir, "handler.tpl")?;
        let logic_tpl = read_tpl(&tmpl_dir, "logic.tpl")?;
        let types_tpl = read_tpl(&tmpl_dir, "types.tpl")?;
        let middleware_tpl = read_tpl(&tmpl_dir, "middleware.tpl")?;

        // Project root
        // NOTE:
        // - 生成的工程默认依赖当前仓库内的 `rust-zero` crate（用于 `rust_zero::rest::router!` DSL）。
        // - `template_root` 可能来自 `~/.rsctl/<version>/`（用户安装目录），因此不能再用它推导 monorepo 根。
        // - 这里改为从 `out_dir` 向上查找 `rust-zero/Cargo.toml` 来定位 monorepo 根（更稳）。
        let monorepo_root = find_monorepo_root(out_dir).ok_or_else(|| {
            anyhow!(
                "cannot locate monorepo root from out_dir={}, expected to find rust-zero/Cargo.toml in parent dirs",
                out_dir.display()
            )
        })?;
        let rust_zero_crate_path = monorepo_root.join("crates").join("rust-zero");
        let rust_zero_rel = relative_path(out_dir, &rust_zero_crate_path);
        let rust_zero_rel = rust_zero_rel.to_string_lossy().replace('\\', "/");

        // 收集 @server(jwt: Xxx) 的 jwt 名称（用于生成 config 字段）
        use std::collections::BTreeSet;
        let mut jwt_names: BTreeSet<String> = BTreeSet::new();
        for s in &spec.services {
            if let Some(j) = service_jwt(s) {
                jwt_names.insert(snake(&j));
            }
        }

        // 生成后的项目选择原生框架依赖（axum/actix-web），并把 `rest` 的 feature 对齐到同一框架。
        // 若 api 里声明了 @server(jwt: ...) 则额外启用 `rest/jwt`（JWT 作为 rest 内置中间件）。
        let (framework_dep, rest_features) = if framework == "actix" {
            if jwt_names.is_empty() {
                ("actix-web = \"4\"", "[\"actix\"]")
            } else {
                ("actix-web = \"4\"", "[\"actix\", \"jwt\"]")
            }
        } else if jwt_names.is_empty() {
            ("axum = \"0.6\"", "[\"axum\"]")
        } else {
            ("axum = \"0.6\"", "[\"axum\", \"jwt\"]")
        };

        // validator（按 type 字段 tag 是否包含 validate:"..." 决定）
        let use_validator = spec.types.iter().any(|t| {
            t.fields.iter().any(|f| {
                f.tag
                    .as_ref()
                    .is_some_and(|tag| tag.contains("validate:\""))
            })
        });

        let validator_dep = if use_validator {
            // validator crate: see https://github.com/Keats/validator
            "validator = { version = \"0.19\", features = [\"derive\"] }"
        } else {
            ""
        };

        let jsonwebtoken_dep = "";

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
{framework_dep}
http = "0.2"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tokio = {{ version = "1", features = ["macros", "rt-multi-thread", "signal"] }}
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
rust_zero = {{ package = "rust-zero", path = "{rust_zero_rel}", features = {rest_features} }}
{validator_dep}
{jsonwebtoken_dep}

[workspace]

"#
            ),
        });

        let default_cfg_path = format!("etc/{service_name}.yaml");

        files.push(Artifact {
            rel_path: default_cfg_path.clone().into(),
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
            content: { render::render(&main_tpl, &base_ctx).context("render main.tpl")? },
        });

        // 注意：不再“额外”生成 main.rs（stub）。
        // 入口文件名规则：`<service>.rs`；若 service == "main" 则入口自然是 `main.rs`。

        // config.rs：渲染 Rust 模板（config.tpl）。
        files.push(Artifact {
            rel_path: "config.rs".into(),
            content: {
                let (jwt_conf, jwt_field, jwt_default) = if jwt_names.is_empty() {
                    (String::new(), String::new(), String::new())
                } else {
                    let jwt_conf = r#"#[derive(Debug, Clone, Deserialize, Default)]
pub struct JwtConf {
    #[serde(rename = "AccessSecret")]
    pub access_secret: String,
}
"#
                    .to_string();

                    let mut fields: Vec<String> = Vec::new();
                    let mut defaults: Vec<String> = Vec::new();
                    for j in &jwt_names {
                        let key = {
                            let mut ch = j.chars();
                            match ch.next() {
                                None => j.clone(),
                                Some(f) => f.to_ascii_uppercase().to_string() + ch.as_str(),
                            }
                        };
                        fields.push(format!(
                            "#[serde(rename = \"{key}\")]\n    pub {j}: JwtConf,"
                        ));
                        defaults.push(format!("{j}: JwtConf {{ access_secret: String::new() }},"));
                    }

                    (
                        jwt_conf,
                        fields.join("\n\n"),
                        defaults.join("\n            "),
                    )
                };

                let ctx = base_ctx
                    .clone()
                    .set_str("jwtConf", jwt_conf)
                    .set_str("jwtField", jwt_field)
                    .set_str("jwtDefault", jwt_default);
                render::render(&config_tpl, &ctx).context("render config.tpl")?
            },
        });
        files.push(Artifact {
            rel_path: "svc.rs".into(),
            content: { render::render(&context_tpl, &base_ctx).context("render context.tpl")? },
        });

        // handler root（handler 子模块 + routes）
        let mut handler_root = "/// generated by rsctl\n".to_string();
        handler_root.push('\n');
        handler_root.push_str("pub mod routes;\n");
        for g in group_handlers.keys() {
            let file_base = group_file_base(g, style);
            handler_root.push_str(&mod_decl_with_path("handler", g, &file_base));
        }
        files.push(Artifact {
            rel_path: "handler.rs".into(),
            content: handler_root,
        });

        // routes.rs：路由构建（Rust 模板：routes.tpl）
        {
            use std::collections::BTreeMap;
            let mut routes_additions = String::new();
            routes_additions.push_str("state $svc_ctx.clone();\n");

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
                if jwt.is_some() {
                    s += 100;
                }
                if mw.is_some() {
                    s += 10;
                }
                if !prefix.trim().is_empty() {
                    s += 1;
                }
                s
            };

            for (idx, service) in spec.services.iter().enumerate() {
                let prefix = service_prefix(service).unwrap_or_default();
                let mw = service_middleware(service);
                let jwt = service_jwt(service);
                let score = score_of(&prefix, &mw, &jwt);

                for r in &service.routes {
                    let Some(_h) = route_handler_name(r) else {
                        continue;
                    };
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
                            winners.insert(
                                route_key,
                                Winner {
                                    score,
                                    service_idx: idx,
                                },
                            );
                        }
                        Some(prev) if score > prev.score => {
                            winners.insert(
                                route_key,
                                Winner {
                                    score,
                                    service_idx: idx,
                                },
                            );
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

                let mut any = false;
                let mut block = String::new();
                if let Some(mw_name) = &mw {
                    let mw_mod = snake(mw_name);
                    block.push_str(&format!(
                        "middleware_fn crate::middleware::{mw_mod}::handle;\n"
                    ));
                }
                if let Some(jwt_name) = &jwt {
                    let jwt_field = snake(jwt_name);
                    if framework == "actix" {
                        // JWT 内置于 rest：生成器只负责把 secret 注入到 closure（满足 'static）
                        block.push_str(&format!(
                            "middleware_fn ({{
    let secret = $svc_ctx.config.{jwt_field}.access_secret.clone();
    move |req, next| {{
        let secret = secret.clone();
        async move {{ rust_zero::rest::middleware::jwt::actix_jwt::handle(req, next, secret).await }}
    }}
}});\n"
                        ));
                    } else {
                        block.push_str(&format!(
                            "middleware_state rust_zero::rest::middleware::jwt::axum_jwt::state($svc_ctx.config.{jwt_field}.access_secret.clone()), rust_zero::rest::middleware::jwt::axum_jwt::auth;\n"
                        ));
                    }
                }
                for r in &service.routes {
                    let Some(h) = route_handler_name(r) else {
                        continue;
                    };
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
                    let Some(w) = winners.get(&route_key) else {
                        continue;
                    };
                    if w.service_idx != idx {
                        continue;
                    }

                    any = true;
                    block.push_str(&format!(
                        "{method} \"{}\" => crate::handler::{group}::handler::{h};\n",
                        r.path
                    ));
                }

                if any {
                    if !prefix.trim().is_empty() {
                        routes_additions.push_str(&format!(
                            "group \"{prefix}\" {{\n{block}}}\n",
                            prefix = prefix,
                            block = indent_block(&block, 4)
                        ));
                    } else {
                        routes_additions.push_str(&block);
                    }
                }

                let _ = group; // keep group var to preserve handler path calculation
            }

            let routes_ctx = base_ctx
                .clone()
                .set_str("routesAdditions", routes_additions);
            files.push(Artifact {
                rel_path: "handler/routes.rs".into(),
                content: render::render(&routes_tpl, &routes_ctx).context("render routes.tpl")?,
            });
        }

        // middleware 模块：
        // - 若 api 里声明了 @server(middleware: ...) 则生成占位 middleware；
        // - JWT 由 rest 内置提供（不再生成 jwt.rs）。
        {
            use std::collections::BTreeSet;
            let mut mws: BTreeSet<String> = BTreeSet::new();
            for s in &spec.services {
                if let Some(m) = service_middleware(s) {
                    mws.insert(snake(&m));
                }
            }
            // 为了匹配模板中的 `mod middleware;`，即使没有任何自定义 middleware 也生成一个空模块文件。
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
                    content: {
                        let (imports, handle) = if framework == "actix" {
                            (
                                "use actix_web::body::BoxBody;\nuse actix_web::dev::{ServiceRequest, ServiceResponse};\nuse actix_web::middleware::Next;\nuse actix_web::Error;\n",
                                "pub async fn handle(req: ServiceRequest, next: Next<BoxBody>) -> Result<ServiceResponse<BoxBody>, Error> {\n    // TODO: {{.name}} middleware logic\n    next.call(req).await\n}\n",
                            )
                        } else {
                            (
                                "use axum::body::Body;\nuse axum::http::Request;\nuse axum::middleware::Next;\nuse axum::response::Response;\n",
                                "pub async fn handle(req: Request<Body>, next: Next<Body>) -> Response {\n    // TODO: {{.name}} middleware logic\n    next.run(req).await\n}\n",
                            )
                        };
                        let ctx = base_ctx
                            .clone()
                            .set_str("name", m)
                            .set_str("middlewareImports", imports)
                            .set_str("middlewareHandle", handle);
                        render::render(&middleware_tpl, &ctx)
                            .context("render middleware.tpl")?
                    },
                });
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
                    let mut handlers_body = String::new();

                    let type_map: std::collections::BTreeMap<String, &spec::api::TypeDef> =
                        spec.types.iter().map(|t| (t.name.clone(), t)).collect();

                    for h in hs {
                        // Find a representative route in this group with this handler.
                        // We only use it to populate request/response/doc metadata.
                        let mut req: Option<String> = None;
                        let mut resp: Option<String> = None;
                        let mut doc: Option<String> = None;
                        'outer: for service in &spec.services {
                            let gg = effective_group(service);
                            if snake(&gg) != *g {
                                continue;
                            }
                            for r in &service.routes {
                                let Some(hn) = route_handler_name(r) else {
                                    continue;
                                };
                                if snake(&hn) != *h {
                                    continue;
                                }
                                req = r.request.clone();
                                resp = r.response.clone();
                                // doc annotation
                                if let Some(a) = r.annotations.iter().find(|a| a.name == "doc")
                                    && let spec::api::AnnotationArgs::Str(s) = &a.args
                                {
                                    doc = Some(s.clone());
                                }
                                break 'outer;
                            }
                        }

                        let req_rust = req
                            .as_deref()
                            .map(|t| api_ty_to_rust(g, t))
                            .unwrap_or_default();
                        let _resp_rust = resp
                            .as_deref()
                            .map(|t| {
                                // 返回类型可能是 `[]T`，逻辑层也需要返回 Vec<T>
                                if let Some(inner) = t.strip_prefix("[]") {
                                    format!("anyhow::Result<Vec<{}>>", api_ty_to_rust(g, inner))
                                } else {
                                    format!("anyhow::Result<{}>", api_ty_to_rust(g, t))
                                }
                            })
                            .unwrap_or_else(|| "anyhow::Result<()>".to_string());

                        let mut needs_validate = false;
                        if let Some(req_name) = req.as_deref()
                            && let Some(td) = type_map.get(req_name)
                        {
                            for f in &td.fields {
                                let Some(tag) = &f.tag else { continue };
                                let m = tag_kv_map(tag);
                                if m.contains_key("validate") {
                                    needs_validate = true;
                                }
                            }
                        }

                        // Rust handler（rest::web 风格，框架无关）
                        let doc_str = doc
                            .as_ref()
                            .map(|d| format!("/// {}\n", d))
                            .unwrap_or_default();
                        handlers_body.push_str(&doc_str);

                        let has_req = req.is_some();
                        let has_resp = resp.is_some();
                        let req_ty = req_rust.as_str();

                        if framework == "actix" {
                            // actix-web handler
                            handlers_body.push_str(&format!(
                                "pub async fn {name}(\n    svc_ctx: web::Data<Arc<ServiceContext>>,\n",
                                name = h
                            ));
                            if has_req {
                                handlers_body.push_str(&format!(
                                    "    req: web::Json<{req_ty}>,\n",
                                    req_ty = req_ty
                                ));
                            }
                            handlers_body.push_str(") -> HttpResponse {\n");
                            if has_req && needs_validate {
                                handlers_body.push_str("    if let Err(e) = req.validate() {\n        return HttpResponse::BadRequest().body(e.to_string());\n    }\n\n");
                            }
                            let logic_struct = format!("{}Logic", pascal(h));
                            handlers_body.push_str(&format!(
                                "    let l = crate::logic::{g}::logic::{logic_struct}::new(svc_ctx.get_ref().clone());\n",
                                g = g
                            ));
                            if has_req {
                                if has_resp {
                                    handlers_body.push_str(&format!(
                                        "    match l.{call}(req.into_inner()).await {{\n        Ok(resp) => HttpResponse::Ok().json(resp),\n        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),\n    }}\n",
                                        call = h
                                    ));
                                } else {
                                    handlers_body.push_str(&format!(
                                        "    match l.{call}(req.into_inner()).await {{\n        Ok(()) => HttpResponse::Ok().finish(),\n        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),\n    }}\n",
                                        call = h
                                    ));
                                }
                            } else if has_resp {
                                handlers_body.push_str(&format!(
                                    "    match l.{call}().await {{\n        Ok(resp) => HttpResponse::Ok().json(resp),\n        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),\n    }}\n",
                                    call = h
                                ));
                            } else {
                                handlers_body.push_str(&format!(
                                    "    match l.{call}().await {{\n        Ok(()) => HttpResponse::Ok().finish(),\n        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),\n    }}\n",
                                    call = h
                                ));
                            }
                            handlers_body.push_str("}\n\n");
                        } else {
                            // axum handler
                            handlers_body.push_str(&format!(
                                "pub async fn {name}(\n    Extension(svc_ctx): Extension<Arc<ServiceContext>>,\n",
                                name = h
                            ));
                            if has_req {
                                handlers_body.push_str(&format!("    Json(req): Json<{req_ty}>,\n"));
                            }
                            handlers_body.push_str(") -> impl IntoResponse {\n");
                            if has_req && needs_validate {
                                handlers_body.push_str("    if let Err(e) = req.validate() {\n        return (StatusCode::BAD_REQUEST, e.to_string()).into_response();\n    }\n\n");
                            }
                            let logic_struct = format!("{}Logic", pascal(h));
                            handlers_body.push_str(&format!(
                                "    let l = crate::logic::{g}::logic::{logic_struct}::new(svc_ctx.clone());\n",
                                g = g
                            ));
                            if has_req {
                                if has_resp {
                                    handlers_body.push_str(&format!(
                                        "    match l.{call}(req).await {{\n        Ok(resp) => Json(resp).into_response(),\n        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),\n    }}\n",
                                        call = h
                                    ));
                                } else {
                                    handlers_body.push_str(&format!(
                                        "    match l.{call}(req).await {{\n        Ok(()) => StatusCode::OK.into_response(),\n        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),\n    }}\n",
                                        call = h
                                    ));
                                }
                            } else if has_resp {
                                handlers_body.push_str(&format!(
                                    "    match l.{call}().await {{\n        Ok(resp) => Json(resp).into_response(),\n        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),\n    }}\n",
                                    call = h
                                ));
                            } else {
                                handlers_body.push_str(&format!(
                                    "    match l.{call}().await {{\n        Ok(()) => StatusCode::OK.into_response(),\n        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),\n    }}\n",
                                    call = h
                                ));
                            }
                            handlers_body.push_str("}\n\n");
                        }
                    }
                    if !merge {
                        // Minimal behavior: if merge=false, still keep one file per group for now.
                        // (Future: split into per-handler module files.)
                    }
                    let ctx = base_ctx.clone().set_str("handlers", handlers_body);
                    render::render(&handler_tpl, &ctx).context("render handler.tpl")?
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
                    let mut logics_body = String::new();
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
                                let Some(hn) = route_handler_name(r) else {
                                    continue;
                                };
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
                            .map(|t| format!("req: {}", api_ty_to_rust(g, t)))
                            .unwrap_or_default();

                        let resp_ty = resp
                            .as_deref()
                            .map(|t| api_ty_to_rust(g, t))
                            .unwrap_or_else(|| "()".to_string());

                        let response_type = format!("anyhow::Result<{resp_ty}>");
                        let return_string = if has_resp {
                            if !has_req && h == "healthz" && resp_ty.ends_with("::HealthzResp") {
                                format!(
                                    "Ok({resp_ty} {{ code: 0, message: \"ok\".to_string(), data: Default::default() }})"
                                )
                            } else if resp.as_deref().is_some_and(|t| t.starts_with("[]")) {
                                "Ok(Vec::new())".to_string()
                            } else {
                                format!("Ok({resp_ty}::default())")
                            }
                        } else {
                            "Ok(())".to_string()
                        };

                        logics_body.push_str(&format!(
                            "pub struct {logic_name} {{\n    svc_ctx: Arc<ServiceContext>,\n}}\n\n",
                        ));
                        logics_body.push_str(&format!(
                            "impl {logic_name} {{\n    pub fn new(svc_ctx: Arc<ServiceContext>) -> Self {{\n        Self {{ svc_ctx }}\n    }}\n\n"
                        ));
                        logics_body.push_str("    #[instrument(skip_all)]\n");
                        if has_req {
                            logics_body.push_str(&format!(
                                "    pub async fn {func}(&self, {req}) -> {resp} {{\n        let _ = &self.svc_ctx;\n        // todo: add your logic here and delete this line\n\n        {ret}\n    }}\n",
                                func = h,
                                req = req_param,
                                resp = response_type,
                                ret = return_string
                            ));
                        } else {
                            logics_body.push_str(&format!(
                                "    pub async fn {func}(&self) -> {resp} {{\n        let _ = &self.svc_ctx;\n        // todo: add your logic here and delete this line\n\n        {ret}\n    }}\n",
                                func = h,
                                resp = response_type,
                                ret = return_string
                            ));
                        }
                        logics_body.push_str("}\n\n");
                    }
                    if !merge {
                        // Minimal behavior: keep group-level file even when merge=false.
                    }
                    let ctx = base_ctx.clone().set_str("logics", logics_body);
                    render::render(&logic_tpl, &ctx).context("render logic.tpl")?
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
                    // generate structs for referenced request/response types in this group
                    let mut decls: Vec<String> = Vec::new();
                    let mut required = std::collections::BTreeSet::<String>::new();
                    let mut queue = std::collections::VecDeque::<String>::new();
                    let type_map: std::collections::BTreeMap<String, &spec::api::TypeDef> =
                        spec.types.iter().map(|t| (t.name.clone(), t)).collect();
                    let mut need_validate = false;

                    for service in &spec.services {
                        let gg = effective_group(service);
                        if snake(&gg) != *g {
                            continue;
                        }
                        for r in &service.routes {
                            if let Some(t) = &r.request {
                                if let Some(inner) = t.strip_prefix("[]") {
                                    queue.push_back(inner.to_string());
                                } else {
                                    queue.push_back(t.clone());
                                }
                            }
                            if let Some(t) = &r.response {
                                if let Some(inner) = t.strip_prefix("[]") {
                                    queue.push_back(inner.to_string());
                                } else {
                                    queue.push_back(t.clone());
                                }
                            }
                        }
                    }

                    // 递归收集：把 response/request 结构体字段中引用到的自定义类型也一并生成。
                    while let Some(t) = queue.pop_front() {
                        let t = t.strip_prefix("[]").unwrap_or(&t).to_string();
                        if !required.insert(t.clone()) {
                            continue;
                        }

                        if let Some(td) = type_map.get(&t) {
                            for f in &td.fields {
                                let ft = f.ty.strip_prefix("[]").unwrap_or(&f.ty).to_string();
                                if type_map.contains_key(&ft) && !required.contains(&ft) {
                                    queue.push_back(ft);
                                }
                            }
                        }
                    }

                    for t in required {
                        // array type in returns: `[]T` is not a struct, skip
                        if t.starts_with("[]") {
                            continue;
                        }

                        if let Some(td) = type_map.get(&t) {
                            let mut s = String::new();
                            // decide derives: request/response structs both derive Deserialize/Serialize
                            // and optionally Validate when any field has validate tag.
                            let mut has_validate = false;
                            for f in &td.fields {
                                if let Some(tag) = &f.tag {
                                    let m = tag_kv_map(tag);
                                    if m.contains_key("validate") {
                                        has_validate = true;
                                    }
                                }
                            }
                            if has_validate {
                                need_validate = true;
                                s.push_str("#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]\n");
                            } else {
                                s.push_str("#[derive(Debug, Clone, Serialize, Deserialize, Default)]\n");
                            }
                            s.push_str(&format!("pub struct {} {{\n", td.name));

                            for f in &td.fields {
                                let rust_ty = api_ty_to_rust(g, &f.ty);
                                if let Some(tag) = &f.tag {
                                    let m = tag_kv_map(tag);
                                    // serde rename for json/form/path
                                    if let Some(v) = m.get("json").or_else(|| m.get("form")).or_else(|| m.get("path")) {
                                        s.push_str(&format!("    #[serde(rename = \"{}\")]\n", v));
                                    }
                                    if let Some(v) = m.get("validate") {
                                        for a in validate_attrs(rust_ty.as_str(), v) {
                                            s.push_str(&format!("    {}\n", a));
                                        }
                                    }
                                }
                                s.push_str(&format!("    pub {}: {},\n", snake(&f.name), rust_ty));
                            }
                            s.push_str("}\n");
                            decls.push(s);
                        } else {
                        decls.push(format!(
                            "#[derive(Debug, Clone, Serialize, Deserialize, Default)]\npub struct {} {{}}\n",
                            t
                        ));
                    }
                    }
                    let types_ctx = base_ctx
                        .clone()
                        .set_str("types", decls.join("\n"))
                        .set_bool("needValidate", need_validate);
                    render::render(&types_tpl, &types_ctx).context("render types.tpl")?
                },
            });
        }

        // NOTE: /healthz 等默认路由应由 api.api 显式声明，避免生成器暗含行为。

        Ok(Artifacts { files })
    }
}
