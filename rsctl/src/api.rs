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
    #[arg(short = 'd', long = "dir", default_value = ".")]
    pub dir: PathBuf,

    /// Path of API description file
    #[arg(short = 'a', long = "api")]
    pub api: PathBuf,

    /// 生成文件命名风格（影响生成的 .rs 文件名）
    ///
    /// 可选值：
    /// - rust_zero  : snake_case（默认）
    /// - rustZero   : lowerCamelCase（仅影响文件名；模块名仍为 snake_case）
    /// - RustZero   : UpperCamelCase（仅影响文件名；模块名仍为 snake_case）
    #[arg(short = 's', long = "style", default_value = "rust_zero", value_parser = ["rust_zero", "rustZero", "RustZero"])]
    pub style: Option<String>,

    /// Template source:
    /// - git/http URL => clone to temp and use it
    /// - /xxx or relative path => local template directory
    /// - omitted => use local `templates/` under current working dir
    #[arg(short = 'r', long = "remote")]
    pub remote: Option<String>,

    /// Merge handlers of the same group into one file.
    ///
    /// - `true`: merge into `handler/<group>/handler.rs` and `logic/<group>/logic.rs`
    /// - `false`: split one handler per file under the group dir
    #[arg(short = 'm', long = "merge", default_value_t = true, action = clap::ArgAction::Set)]
    pub merge: bool,

    /// Overwrite existing files when writing to disk (default: false).
    #[arg(short = 'o', long = "overwrite", action = clap::ArgAction::SetTrue)]
    pub overwrite: bool,

    /// - 推荐：`--web axum`
    #[arg(short = 'w', long = "web", default_value = "axum")]
    pub web: String,
}

pub fn run(api: Api) -> Result<()> {
    match api {
        Api::Rs(args) => {
            crate::pipeline::api::rs::run(crate::pipeline::api::rs::Options {
                out_dir: args.dir,
                api_file: args.api,
                style: args.style,
                remote: args.remote,
                merge: args.merge,
                overwrite: args.overwrite,
                web: args.web,
            })
            .context("rsctl api rs failed")?;
        }
    }
    Ok(())
}
