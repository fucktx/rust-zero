//! API pipeline entrypoints (orchestration layer).

pub mod rs {
    use anyhow::{Context, Result, anyhow};
    use std::fs;
    use std::path::PathBuf;

    #[derive(Debug, Clone)]
    pub struct Options {
        pub out_dir: PathBuf,
        pub api_file: PathBuf,
        pub style: Option<String>,
        /// None => local workspace templates (`templates/`)
        /// Some => either a local path, or a remote git/http URL.
        pub remote: Option<String>,
        /// Merge handlers of the same group into one file.
        pub merge: bool,
        /// Whether to overwrite existing files when writing.
        pub overwrite: bool,
        /// Target web framework name (e.g. "axum").
        pub web: String,
    }

    #[derive(Debug, Clone)]
    pub struct Config {
        pub out_dir: PathBuf,
        pub api_file: PathBuf,
        pub style: Option<String>,
        /// Final template root directory (expected to contain `api/rs/`).
        pub template_root: PathBuf,
        pub merge: bool,
        pub overwrite: bool,
        pub web: String,
    }

    pub fn run(opts: Options) -> Result<Config> {
        let web = opts.web.clone();
        let cfg = config(opts, &web)?;
        pipeline(&cfg, &web)?;
        Ok(cfg)
    }

    /// Note: currently still contains side-effects (create dir, write marker) as a proof-of-wiring.
    pub(crate) fn config(mut opts: Options, web: &str) -> Result<Config> {
        // Ensure consistent web label for downstream.
        opts.web = web.to_string();

        if opts.out_dir.as_os_str().is_empty() {
            return Err(anyhow!("--dir is required"));
        }
        if opts.api_file.as_os_str().is_empty() {
            return Err(anyhow!("--api is required"));
        }

        fs::create_dir_all(&opts.out_dir)
            .with_context(|| format!("failed to create output dir: {}", opts.out_dir.display()))?;

        let template_root = utils::template::resolve_template_root(opts.remote.as_deref())?;

        tracing::info!(
            out_dir = %opts.out_dir.display(),
            api = %opts.api_file.display(),
            template_root = %template_root.display(),
            style = opts.style.as_deref().unwrap_or(""),
            web = %opts.web,
            merge = opts.merge,
            overwrite = opts.overwrite,
            "rsctl api rs resolved"
        );

        Ok(Config {
            out_dir: opts.out_dir,
            api_file: opts.api_file,
            style: opts.style,
            template_root,
            merge: opts.merge,
            overwrite: opts.overwrite,
            web: opts.web,
        })
    }

    pub(crate) fn pipeline(cfg: &Config, web: &str) -> Result<()> {
        // 1) parse
        let ast = parse::api::parse_file(&cfg.api_file).context("parse api file")?;

        // 2) semantic -> spec (stable IR)
        let spec = semantic::api::to_spec(&ast).context("semantic to spec")?;

        // 约束：如果存在多个 service 且名称不一致，直接报错（避免生成入口/工程名歧义）。
        if !spec.services.is_empty() {
            use std::collections::BTreeSet;
            let mut names: BTreeSet<String> = BTreeSet::new();
            for s in &spec.services {
                names.insert(s.name.clone());
            }
            if names.len() > 1 {
                return Err(anyhow!(
                    "multiple different service names found: {:?}. Please use the same service name across the .api file.",
                    names
                ));
            }
        }

        // 3) gen -> artifacts
        let style = cfg.style.as_deref().unwrap_or("rust_zero");
        // go-zero 风格：优先用 `.api` 里 `service <name>` 的名字作为工程/入口名。
        // 如果没有任何 service（只有顶层 routes），再回退到文件名。
        let service_name = spec
            .services
            .first()
            .map(|s| s.name.clone())
            .or_else(|| {
                cfg.api_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "api".to_string());

        let artifacts = match web {
            "axum" => codegen::api::rs::axum::generate(
                &spec,
                &codegen::api::rs::axum::Options {
                    service_name,
                    merge: cfg.merge,
                    style: style.to_string(),
                    out_dir: cfg.out_dir.clone(),
                    template_root: cfg.template_root.clone(),
                },
            )?,
            "actix" => codegen::api::rs::actix::generate(
                &spec,
                &codegen::api::rs::actix::Options {
                    service_name,
                    merge: cfg.merge,
                    style: style.to_string(),
                    out_dir: cfg.out_dir.clone(),
                    template_root: cfg.template_root.clone(),
                },
            )?,
            other => return Err(anyhow!("unsupported web for gen: {other}")),
        };

        // 4) write
        write_artifacts(cfg, &artifacts).context("write artifacts")?;

        Ok(())
    }

    fn write_artifacts(cfg: &Config, artifacts: &codegen::artifact::Artifacts) -> Result<()> {
        // 入口文件名规则：`<service>.rs`，若 service == "main" 则入口自然是 `main.rs`。
        // 因为 `--overwrite` 不会自动删除旧文件，这里在 overwrite=true 且入口不是 main.rs 时，
        // 主动清理残留的 `out_dir/main.rs`，避免误导用户。
        if cfg.overwrite {
            let has_entry_main = artifacts
                .files
                .iter()
                .any(|f| f.rel_path.as_os_str() == "main.rs");
            if !has_entry_main {
                let main_rs = cfg.out_dir.join("main.rs");
                if main_rs.exists() {
                    let _ = fs::remove_file(&main_rs);
                }
            }

            // 历史遗留：旧版本会生成 `middleware/jwt.rs`，现在 JWT 已下沉到 `rest` 内置中间件；
            // overwrite=true 时主动清理，避免残留文件导致编译错误/误导。
            let legacy_jwt = cfg.out_dir.join("middleware").join("jwt.rs");
            if legacy_jwt.exists() {
                let _ = fs::remove_file(&legacy_jwt);
            }

            // 历史遗留：旧版本会生成 `handler/routes.rs`，现在改为 `server.rs`；
            // overwrite=true 时主动清理，避免残留文件干扰理解。
            let legacy_routes = cfg.out_dir.join("handler").join("routes.rs");
            if legacy_routes.exists() {
                let _ = fs::remove_file(&legacy_routes);
            }

            // 历史遗留：之前生成过 `server.rs`，现在不再生成（启动走 rest::server）。
            let legacy_server_rs = cfg.out_dir.join("server.rs");
            if legacy_server_rs.exists() {
                let _ = fs::remove_file(&legacy_server_rs);
            }
        }

        for f in &artifacts.files {
            let path = cfg.out_dir.join(&f.rel_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create parent dir: {}", parent.display())
                })?;
            }

            if !cfg.overwrite && path.exists() {
                tracing::info!(path = %path.display(), "skip existing file (overwrite=false)");
                continue;
            }

            fs::write(&path, &f.content)
                .with_context(|| format!("failed to write {}", path.display()))?;
            tracing::info!(path = %path.display(), "write file");
        }
        Ok(())
    }
}
