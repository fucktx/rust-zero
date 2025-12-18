use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ModelMysqlOptions {
    pub out_dir: PathBuf,
    pub cache: bool,
    pub style: Option<String>,
    /// None => local workspace templates (`templates/`)
    /// Some => either a local path, or a remote git/http URL.
    pub remote: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelMysqlResolved {
    pub out_dir: PathBuf,
    pub cache: bool,
    pub style: Option<String>,
    /// Resolved template root directory (expected to contain `model/mysql/`).
    pub template_root: PathBuf,
}

pub fn run_mysql(opts: ModelMysqlOptions) -> Result<ModelMysqlResolved> {
    if opts.out_dir.as_os_str().is_empty() {
        return Err(anyhow!("--dir is required"));
    }

    fs::create_dir_all(&opts.out_dir)
        .with_context(|| format!("failed to create output dir: {}", opts.out_dir.display()))?;

    let template_root = crate::template_source::resolve_template_root(opts.remote.as_deref())?;

    // Minimal “proof of wiring”: write a marker file with the resolved config.
    let marker = opts.out_dir.join(".rsctl-run.json");
    let json = format!(
        "{{\n  \"kind\": \"model.mysql\",\n  \"out_dir\": \"{}\",\n  \"cache\": {},\n  \"style\": {},\n  \"template_root\": \"{}\"\n}}\n",
        crate::template_source::escape_json(&opts.out_dir.display().to_string()),
        if opts.cache { "true" } else { "false" },
        match &opts.style {
            Some(s) => format!("\"{}\"", crate::template_source::escape_json(s)),
            None => "null".to_string(),
        },
        crate::template_source::escape_json(&template_root.display().to_string()),
    );
    fs::write(&marker, json).with_context(|| format!("failed to write {}", marker.display()))?;

    tracing::info!(
        out_dir = %opts.out_dir.display(),
        template_root = %template_root.display(),
        cache = opts.cache,
        style = opts.style.as_deref().unwrap_or(""),
        "rsctl model mysql resolved"
    );

    Ok(ModelMysqlResolved {
        out_dir: opts.out_dir,
        cache: opts.cache,
        style: opts.style,
        template_root,
    })
}

#[derive(Debug, Clone)]
pub struct ModelPgOptions {
    pub out_dir: PathBuf,
    pub cache: bool,
    pub style: Option<String>,
    /// None => local workspace templates (`templates/`)
    /// Some => either a local path, or a remote git/http URL.
    pub remote: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelPgResolved {
    pub out_dir: PathBuf,
    pub cache: bool,
    pub style: Option<String>,
    /// Resolved template root directory (expected to contain `model/pg/`).
    pub template_root: PathBuf,
}

pub fn run_pg(opts: ModelPgOptions) -> Result<ModelPgResolved> {
    if opts.out_dir.as_os_str().is_empty() {
        return Err(anyhow!("--dir is required"));
    }

    fs::create_dir_all(&opts.out_dir)
        .with_context(|| format!("failed to create output dir: {}", opts.out_dir.display()))?;

    let template_root = crate::template_source::resolve_template_root(opts.remote.as_deref())?;

    // Minimal “proof of wiring”: write a marker file with the resolved config.
    let marker = opts.out_dir.join(".rsctl-run.json");
    let json = format!(
        "{{\n  \"kind\": \"model.pg\",\n  \"out_dir\": \"{}\",\n  \"cache\": {},\n  \"style\": {},\n  \"template_root\": \"{}\"\n}}\n",
        crate::template_source::escape_json(&opts.out_dir.display().to_string()),
        if opts.cache { "true" } else { "false" },
        match &opts.style {
            Some(s) => format!("\"{}\"", crate::template_source::escape_json(s)),
            None => "null".to_string(),
        },
        crate::template_source::escape_json(&template_root.display().to_string()),
    );
    fs::write(&marker, json).with_context(|| format!("failed to write {}", marker.display()))?;

    tracing::info!(
        out_dir = %opts.out_dir.display(),
        template_root = %template_root.display(),
        cache = opts.cache,
        style = opts.style.as_deref().unwrap_or(""),
        "rsctl model pg resolved"
    );

    Ok(ModelPgResolved {
        out_dir: opts.out_dir,
        cache: opts.cache,
        style: opts.style,
        template_root,
    })
}


