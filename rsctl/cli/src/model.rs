use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum Model {
    /// Generate model code for MySQL
    Mysql(MysqlArgs),
    /// Generate model code for PostgreSQL
    Pg(PgArgs),
}

#[derive(Debug, Clone, ClapArgs)]
pub struct MysqlArgs {
    /// Generate code with cache
    #[arg(short = 'c', long = "cache")]
    pub cache: bool,

    /// The target dir
    #[arg(short = 'd', long = "dir")]
    pub dir: PathBuf,

    /// Template source:
    /// - git/http URL => clone to temp and use it
    /// - /xxx or relative path => local template directory
    /// - omitted => use local `templates/` under current working dir
    #[arg(long = "remote")]
    pub remote: Option<String>,

    /// The file naming format (reserved; affects output filenames)
    #[arg(long = "style")]
    pub style: Option<String>,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct PgArgs {
    /// Generate code with cache
    #[arg(short = 'c', long = "cache")]
    pub cache: bool,

    /// The target dir
    #[arg(short = 'd', long = "dir")]
    pub dir: PathBuf,

    /// Template source:
    /// - git/http URL => clone to temp and use it
    /// - /xxx or relative path => local template directory
    /// - omitted => use local `templates/` under current working dir
    #[arg(long = "remote")]
    pub remote: Option<String>,

    /// The file naming format (reserved; affects output filenames)
    #[arg(long = "style")]
    pub style: Option<String>,
}

pub fn run(model: Model) -> Result<()> {
    match model {
        Model::Mysql(args) => {
            core::model::run_mysql(core::model::ModelMysqlOptions {
                out_dir: args.dir,
                cache: args.cache,
                style: args.style,
                remote: args.remote,
            })
            .context("rsctl model mysql failed")?;
        }
        Model::Pg(args) => {
            core::model::run_pg(core::model::ModelPgOptions {
                out_dir: args.dir,
                cache: args.cache,
                style: args.style,
                remote: args.remote,
            })
            .context("rsctl model pg failed")?;
        }
    }
    Ok(())
}


