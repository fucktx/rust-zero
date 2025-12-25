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
            if dir.join("rest").join("Cargo.toml").is_file() {
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

    /// 将渲染后的 goctl `context.tpl`（Go 代码风格）转换为 Rust 的 `ServiceContext`。
    ///
    /// 约束：
    /// - 模板文件尽量保持 goctl 原样（变量名/结构不动）；
    /// - 生成器侧仅把其中的“语义信息”（config 类型 + middleware 字段/赋值）映射到 Rust；
    /// - `package/import` 段落不会进入输出。
    fn render_goctl_context_to_rust(rendered: &str) -> Result<String> {
        let mut config_ty: Option<String> = None;
        let mut middleware_fields: Vec<String> = Vec::new();
        let mut middleware_assign: Vec<String> = Vec::new();

        // 解析 struct 区块中的字段（我们只关心 Config + middleware 占位）。
        let mut in_struct = false;
        for raw in rendered.lines() {
            let line = raw.trim();
            if line.starts_with("type ServiceContext struct") {
                in_struct = true;
                continue;
            }
            if in_struct {
                if line == "}" {
                    in_struct = false;
                    continue;
                }
                if line.is_empty() {
                    continue;
                }
                // Go 风格：`Config <type>`
                if let Some(rest) = line.strip_prefix("Config ") {
                    let ty = rest.trim();
                    if !ty.is_empty() {
                        config_ty = Some(ty.to_string());
                    }
                    continue;
                }
                // 其余视为 middleware 字段（允许模板直接塞 Rust 片段）。
                middleware_fields.push(raw.to_string());
            }
        }

        // 解析 NewServiceContext 的 struct literal 区块里的赋值。
        // Go 风格：
        // return &ServiceContext{
        //     Config: c,
        //     {{.middlewareAssignment}}
        // }
        let mut in_literal = false;
        for raw in rendered.lines() {
            let line = raw.trim();
            if line.starts_with("return &ServiceContext{") {
                in_literal = true;
                continue;
            }
            if in_literal {
                if line == "}" {
                    in_literal = false;
                    continue;
                }
                if line.is_empty() {
                    continue;
                }
                if line.starts_with("Config:") {
                    continue;
                }
                middleware_assign.push(raw.to_string());
            }
        }

        let config_ty = config_ty.unwrap_or_else(|| "crate::config::Config".to_string());
        let (config_use, config_ty) = if config_ty == "crate::config::Config" {
            ("use crate::config::Config;\n\n", "Config".to_string())
        } else {
            ("", config_ty)
        };

        // 输出 Rust
        let mut out = String::new();
        out.push_str("// Code scaffolded by rsctl. Safe to edit.\n");
        out.push_str("// rsctl 0.01\n\n");
        out.push_str(config_use);
        out.push_str(&format!(
            "pub struct ServiceContext {{\n    pub config: {config_ty},\n"
        ));
        for l in middleware_fields {
            if l.trim().is_empty() {
                continue;
            }
            out.push_str("    ");
            out.push_str(l.trim());
            if !l.trim().ends_with(',') {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("}\n\n");
        out.push_str("impl ServiceContext {\n");
        out.push_str(&format!("    pub fn new(config: {config_ty}) -> Self {{\n"));
        out.push_str("        Self {\n");
        out.push_str("            config,\n");
        for l in middleware_assign {
            let t = l.trim();
            if t.is_empty() {
                continue;
            }
            out.push_str("            ");
            out.push_str(t);
            if !t.ends_with(',') {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n");
        Ok(out)
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
        // 只依赖少量“非 Rust 源码”的模板文件：
        // - `etc.tpl`: config.yaml 的最小模板
        // - `context.tpl`: 为了保留 goctl 变量命名，但生成器侧会转换为 Rust `svc.rs`
        let _ = must_exist(&tmpl_dir, "context.tpl")?;
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

        let context_tpl = read_tpl(&tmpl_dir, "context.tpl")?;
        let etc_tpl = read_tpl(&tmpl_dir, "etc.tpl")?;

        // Project root
        // NOTE:
        // - 生成的工程默认依赖当前仓库内的 `rest` crate（用于 `rest::router!` DSL）。
        // - `template_root` 可能来自 `~/.rsctl/<version>/`（用户安装目录），因此不能再用它推导 monorepo 根。
        // - 这里改为从 `out_dir` 向上查找 `rest/Cargo.toml` 来定位 monorepo 根（更稳）。
        let monorepo_root = find_monorepo_root(out_dir).ok_or_else(|| {
            anyhow!(
                "cannot locate monorepo root from out_dir={}, expected to find rest/Cargo.toml in parent dirs",
                out_dir.display()
            )
        })?;
        let rest_crate_path = monorepo_root.join("rest");
        let rest_rel = relative_path(out_dir, &rest_crate_path);
        // Normalize Windows backslashes to forward slashes for Cargo.toml.
        let rest_rel = rest_rel.to_string_lossy().replace('\\', "/");

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
{validator_dep}

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
            content: {
                // Rust 入口：按代码层生成，避免依赖 goctl 的 Go 模板。
                r#"//! Generated by rsctl. Safe to edit.

use std::sync::Arc;

use anyhow::Context;

mod config;
mod handler;
mod logic;
mod svc;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cfg = config::load("etc/config.yaml").context("load config")?;
    let addr = cfg.rest.addr_string();
    let addr = addr.parse().context("parse bind addr")?;

    let svc_ctx = Arc::new(svc::ServiceContext::new(cfg));
    let app = handler::register_handlers(svc_ctx);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("axum serve")?;

    Ok(())
}}
"#
                .to_string()
            },
        });

        // 注意：不再“额外”生成 main.rs（stub）。
        // 入口文件名规则：`<service>.rs`；若 service == "main" 则入口自然是 `main.rs`。

        // config.rs：按代码层生成（不使用 goctl 的 Go 模板）。
        // 若使用了 @server(jwt: Xxx) 则生成对应字段（Xxx.access_secret）。
        files.push(Artifact {
            rel_path: "config.rs".into(),
            content: {
                let mut cfg_rs = String::new();
                cfg_rs.push_str("// Code scaffolded by rsctl. Safe to edit.\n");
                cfg_rs.push_str("// rsctl 0.01\n\n");
                cfg_rs.push_str("use anyhow::Context;\n");
                cfg_rs.push_str("use rest::RestConf;\n");
                cfg_rs.push_str("use serde::Deserialize;\n");
                cfg_rs.push_str("use std::fs;\n\n");

                if !jwt_names.is_empty() {
                    cfg_rs.push_str("#[derive(Debug, Clone, Deserialize)]\n");
                    cfg_rs.push_str("pub struct JwtConf {\n");
                    cfg_rs.push_str("    #[serde(rename = \"AccessSecret\")]\n");
                    cfg_rs.push_str("    pub access_secret: String,\n");
                    cfg_rs.push_str("}\n\n");
                }

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
            content: {
                let rendered =
                    render::render(&context_tpl, &base_ctx).context("render context.tpl")?;
                render_goctl_context_to_rust(&rendered)?
            },
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
        handler_root
            .push_str("pub fn register_handlers(svc_ctx: Arc<ServiceContext>) -> Router {\n");
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
            routes_rs.push_str("use std::collections::BTreeMap;\n");
            routes_rs.push_str("use crate::svc::ServiceContext;\n\n");
            routes_rs
                .push_str("pub fn register_routes(svc_ctx: Arc<ServiceContext>) -> Router {\n");
            routes_rs.push_str("    let mut app = Router::<Arc<ServiceContext>>::new();\n");

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

                routes_rs.push_str("    app = app.merge(rest::add_routes!(\n");
                routes_rs.push_str("        [\n");

                let mut any = false;
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
                // `rest::add_routes!` 返回的 Router state 类型会根据 handler 的 MethodRouter 推导出来，
                // 这里无需再做 `.with_state::<...>(...)` 转换。
                routes_rs.push_str("    ) );\n");

                let _ = any; // 允许空块（即便没有 routes），保持结构简单
            }

            // svc_ctx 已用于构造 handler；Router 的真实 state 值由上层 `rest::Server::with_state(svc_ctx)` 注入。
            routes_rs.push_str("    // erase state type so the returned Router can be served via `into_make_service()`\n");
            routes_rs.push_str("    app.with_state::<()>(svc_ctx)\n");
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
                    // 选择 extractor：Json / Path / Form（按 request struct 字段 tag 决定）
                    let mut need_json = false;
                    let mut need_path = false;
                    let mut need_form = false;
                    let mut need_validate = false;

                    let type_map: std::collections::BTreeMap<String, &spec::api::TypeDef> =
                        spec.types.iter().map(|t| (t.name.clone(), t)).collect();

                    for service in &spec.services {
                        let gg = effective_group(service);
                        if snake(&gg) != *g {
                            continue;
                        }
                        for r in &service.routes {
                            let Some(req) = &r.request else { continue };
                            if let Some(td) = type_map.get(req) {
                                for f in &td.fields {
                                    let Some(tag) = &f.tag else { continue };
                                    let m = tag_kv_map(tag);
                                    if m.contains_key("path") {
                                        need_path = true;
                                    }
                                    if m.contains_key("form") {
                                        need_form = true;
                                    }
                                    if m.contains_key("json") {
                                        need_json = true;
                                    }
                                    if m.contains_key("validate") {
                                        need_validate = true;
                                    }
                                }
                            } else {
                                // 未知 type：默认按 Json 处理
                                need_json = true;
                            }
                        }
                    }

                    s.push_str("use axum::{\n");
                    s.push_str("    extract::{\n");
                    if need_json {
                        s.push_str("        Json,\n");
                    }
                    if need_path {
                        s.push_str("        Path,\n");
                    }
                    if need_form {
                        s.push_str("        Form,\n");
                    }
                    s.push_str("    },\n");
                    s.push_str("    http::StatusCode,\n");
                    s.push_str("    response::IntoResponse,\n");
                    s.push_str("    routing::{delete, get, patch, post, put, MethodRouter},\n");
                    s.push_str("};\n");
                    s.push_str("use crate::svc::ServiceContext;\n\n");
                    if need_validate {
                        s.push_str("use validator::Validate;\n\n");
                    }
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
                                let Some(hn) = route_handler_name(r) else {
                                    continue;
                                };
                                if snake(&hn) != *h {
                                    continue;
                                }
                                req = r.request.clone();
                                resp = r.response.clone();
                                http_method = Some(r.method.clone());
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

                        // extractor kind + 是否需要 validate
                        let mut extractor = "Json".to_string();
                        let mut needs_validate = false;
                        if let Some(req_name) = req.as_deref()
                            && let Some(td) = type_map.get(req_name)
                        {
                            let mut has_path = false;
                            let mut has_form = false;
                            let mut has_json = false;
                            for f in &td.fields {
                                let Some(tag) = &f.tag else { continue };
                                let m = tag_kv_map(tag);
                                if m.contains_key("path") {
                                    has_path = true;
                                }
                                if m.contains_key("form") {
                                    has_form = true;
                                }
                                if m.contains_key("json") {
                                    has_json = true;
                                }
                                if m.contains_key("validate") {
                                    needs_validate = true;
                                }
                            }
                            // 简化规则：path-only -> Path；form-only -> Form；否则 Json。
                            if has_path && !has_form && !has_json {
                                extractor = "Path".into();
                            } else if has_form && !has_path && !has_json {
                                extractor = "Form".into();
                            } else {
                                extractor = "Json".into();
                            }
                        }

                        // Rust handler（按代码层生成）：模板中的 Go `package/import` 段不做处理，也不参与生成。
                        let doc_str = doc
                            .as_ref()
                            .map(|d| format!("/// {}\n", d))
                            .unwrap_or_default();
                        s.push_str(&doc_str);
                        // builder fn name 与 @handler 一致（snake_case）
                        let inner_fn = format!("handle_{h}");
                        s.push_str(&format!(
                            "pub fn {name}(svc_ctx: Arc<ServiceContext>) -> MethodRouter<Arc<ServiceContext>> {{\n",
                            name = h
                        ));
                        s.push_str(&format!(
                            "    {}({inner_fn}).with_state(svc_ctx)\n",
                            match http_method.unwrap_or(spec::api::HttpMethod::Get) {
                                spec::api::HttpMethod::Get => "get",
                                spec::api::HttpMethod::Post => "post",
                                spec::api::HttpMethod::Put => "put",
                                spec::api::HttpMethod::Delete => "delete",
                                spec::api::HttpMethod::Patch => "patch",
                            }
                        ));
                        s.push_str("}\n\n");

                        // inner handler fn
                        let has_req = req.is_some();
                        let has_resp = resp.is_some();
                        let req_ty = req_rust.as_str();
                        let extractor = extractor.as_str();

                        s.push_str(&format!("async fn {inner_fn}(\n"));
                        s.push_str("    axum::extract::State(svc_ctx): axum::extract::State<Arc<ServiceContext>>,\n");
                        if has_req {
                            let pat = match extractor {
                                "Path" => "axum::extract::Path(req): axum::extract::Path<",
                                "Form" => "axum::extract::Form(req): axum::extract::Form<",
                                _ => "axum::Json(req): axum::Json<",
                            };
                            s.push_str("    ");
                            s.push_str(pat);
                            s.push_str(req_ty);
                            s.push_str(">,\n");
                        }
                        s.push_str(") -> impl IntoResponse {\n");
                        if has_req && needs_validate {
                            s.push_str("    if let Err(e) = req.validate() {\n");
                            s.push_str("        return (StatusCode::BAD_REQUEST, e.to_string()).into_response();\n");
                            s.push_str("    }\n\n");
                        }
                        // call logic
                        let logic_struct = format!("{}Logic", pascal(h));
                        s.push_str(&format!(
                            "    let l = crate::logic::{g}::logic::{logic_struct}::new(svc_ctx.clone());\n",
                            g = g
                        ));
                        if has_req {
                            if has_resp {
                                s.push_str(&format!(
                                    "    match l.{call}(req).await {{\n        Ok(resp) => axum::Json(resp).into_response(),\n        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),\n    }}\n",
                                    call = h
                                ));
                            } else {
                                s.push_str(&format!(
                                    "    match l.{call}(req).await {{\n        Ok(()) => StatusCode::OK.into_response(),\n        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),\n    }}\n",
                                    call = h
                                ));
                            }
                        } else if has_resp {
                            s.push_str(&format!(
                                "    match l.{call}().await {{\n        Ok(resp) => axum::Json(resp).into_response(),\n        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),\n    }}\n",
                                call = h
                            ));
                        } else {
                            s.push_str(&format!(
                                "    match l.{call}().await {{\n        Ok(()) => StatusCode::OK.into_response(),\n        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),\n    }}\n",
                                call = h
                            ));
                        }
                        s.push_str("}\n\n");
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

                        s.push_str(&format!(
                            "pub struct {logic_name} {{\n    svc_ctx: Arc<ServiceContext>,\n}}\n\n",
                        ));
                        s.push_str(&format!(
                            "impl {logic_name} {{\n    pub fn new(svc_ctx: Arc<ServiceContext>) -> Self {{\n        Self {{ svc_ctx }}\n    }}\n\n"
                        ));
                        s.push_str("    #[instrument(skip_all)]\n");
                        if has_req {
                            s.push_str(&format!(
                                "    pub async fn {func}(&self, {req}) -> {resp} {{\n        let _ = &self.svc_ctx;\n        // todo: add your logic here and delete this line\n\n        {ret}\n    }}\n",
                                func = h,
                                req = req_param,
                                resp = response_type,
                                ret = return_string
                            ));
                        } else {
                            s.push_str(&format!(
                                "    pub async fn {func}(&self) -> {resp} {{\n        let _ = &self.svc_ctx;\n        // todo: add your logic here and delete this line\n\n        {ret}\n    }}\n",
                                func = h,
                                resp = response_type,
                                ret = return_string
                            ));
                        }
                        s.push_str("}\n\n");
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
                    let mut out = String::new();
                    out.push_str("// Code generated by rsctl. DO NOT EDIT.\n");
                    out.push_str("// rsctl 0.01\n\n");
                    out.push_str("use serde::{Deserialize, Serialize};\n");
                    if need_validate {
                        out.push_str("use validator::Validate;\n");
                    }
                    out.push('\n');
                    out.push_str(&decls.join("\n"));
                    out
                },
            });
        }

        // NOTE: /healthz 等默认路由应由 api.api 显式声明，避免生成器暗含行为。

        Ok(Artifacts { files })
    }
}
