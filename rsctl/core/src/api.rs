//! API pipeline entrypoints (orchestration layer).

pub mod rs {
    use anyhow::{anyhow, Context, Result};
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

        // 3) gen -> artifacts
        let style = cfg.style.as_deref().unwrap_or("rust_zero");
        let service_name = cfg
            .api_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("api")
            .to_string();

        let artifacts = match web {
            "axum" => codegen::api::rs::axum::generate(
                &spec,
                &codegen::api::rs::axum::Options {
                    service_name,
                    merge: cfg.merge,
                    style: style.to_string(),
                    template_root: cfg.template_root.clone(),
                },
            )?,
            "actix" => codegen::api::rs::actix::generate(
                &spec,
                &codegen::api::rs::actix::Options {
                    service_name,
                    merge: cfg.merge,
                    style: style.to_string(),
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

