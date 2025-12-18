use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ApiRsOptions {
    pub out_dir: PathBuf,
    pub api_file: PathBuf,
    pub style: Option<String>,
    /// None => local workspace templates (`templates/`)
    /// Some => either a local path, or a remote git/http URL.
    pub remote: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiRsResolved {
    pub out_dir: PathBuf,
    pub api_file: PathBuf,
    pub style: Option<String>,
    /// Resolved template root directory (expected to contain `api/rs/`).
    pub template_root: PathBuf,
}

pub fn run_rs(opts: ApiRsOptions) -> Result<ApiRsResolved> {
    if opts.out_dir.as_os_str().is_empty() {
        return Err(anyhow!("--dir is required"));
    }
    if opts.api_file.as_os_str().is_empty() {
        return Err(anyhow!("--api is required"));
    }

    fs::create_dir_all(&opts.out_dir)
        .with_context(|| format!("failed to create output dir: {}", opts.out_dir.display()))?;

    let template_root = crate::template_source::resolve_template_root(opts.remote.as_deref())?;

    // Minimal “proof of wiring”: write a marker file with the resolved config.
    let marker = opts.out_dir.join(".rsctl-run.json");
    let json = format!(
        "{{\n  \"kind\": \"api.rs\",\n  \"out_dir\": \"{}\",\n  \"api\": \"{}\",\n  \"style\": {},\n  \"template_root\": \"{}\"\n}}\n",
        crate::template_source::escape_json(&opts.out_dir.display().to_string()),
        crate::template_source::escape_json(&opts.api_file.display().to_string()),
        match &opts.style {
            Some(s) => format!("\"{}\"", crate::template_source::escape_json(s)),
            None => "null".to_string(),
        },
        crate::template_source::escape_json(&template_root.display().to_string()),
    );
    fs::write(&marker, json).with_context(|| format!("failed to write {}", marker.display()))?;

    tracing::info!(
        out_dir = %opts.out_dir.display(),
        api = %opts.api_file.display(),
        template_root = %template_root.display(),
        style = opts.style.as_deref().unwrap_or(""),
        "rsctl api rs resolved"
    );

    Ok(ApiRsResolved {
        out_dir: opts.out_dir,
        api_file: opts.api_file,
        style: opts.style,
        template_root,
    })
}


