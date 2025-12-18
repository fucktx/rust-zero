use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum Api {
    /// Generate Rust API scaffold (Axum-style templates)
    Rs(RsArgs),
}

#[derive(Debug, Clone, ClapArgs)]
pub struct RsArgs {
    /// Output directory of generated project/files
    #[arg(long = "dir")]
    pub dir: PathBuf,

    /// Path of API description file
    #[arg(long = "api")]
    pub api: PathBuf,

    /// The file naming format (reserved; affects output filenames)
    #[arg(long = "style")]
    pub style: Option<String>,

    /// Template source:
    /// - git/http URL => clone to temp and use it
    /// - /xxx or relative path => local template directory
    /// - omitted => use local `templates/` under current working dir
    #[arg(long = "remote")]
    pub remote: Option<String>,
}

pub fn run(api: Api) -> Result<()> {
    match api {
        Api::Rs(args) => {
            core::api::run_rs(core::api::ApiRsOptions {
                out_dir: args.dir,
                api_file: args.api,
                style: args.style,
                remote: args.remote,
            })
            .context("rsctl api rs failed")?;
        }
    }
    Ok(())
}


