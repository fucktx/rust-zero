use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

mod api;
mod model;
mod version;

#[derive(Debug, Parser)]
#[command(name = "rsctl", version, disable_version_flag = true, about = "Code generator CLI")]
struct App {
    /// RS 生成器版本号（模板/生成器版本），等价于旧的 `rsctl info -v`
    ///
    /// 注意：CLI 自身版本号可用 `-V/--cli-version`。
    #[arg(short = 'v', long = "version", global = true, action = clap::ArgAction::SetTrue)]
    version: bool,

    /// CLI 自身版本号（Cargo package version）
    #[arg(short = 'V', long = "cli-version", global = true, action = clap::ArgAction::SetTrue)]
    cli_version: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// API generation
    Api {
        #[command(subcommand)]
        lang: api::Api,
    },
    /// Model generation
    Model {
        #[command(subcommand)]
        driver: model::Model,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let argv = preprocess_argv_for_legacy_flags(std::env::args_os().collect());
    let app = App::parse_from(argv);

    if app.version {
        println!("{}", version::VERSION);
        return Ok(());
    }

    if app.cli_version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    match app.command {
        Some(Command::Api { lang }) => api::run(lang)?,
        Some(Command::Model { driver }) => model::run(driver)?,
        None => {
            // No subcommand, no -v: show help
            let mut cmd = App::command();
            cmd.print_help()?;
            println!();
        }
    }

    Ok(())
}

/// Compatibility shim:
/// Allow `rsctl api -rs ...` by rewriting it to `rsctl api rs ...` before clap parsing.
fn preprocess_argv_for_legacy_flags(mut argv: Vec<std::ffi::OsString>) -> Vec<std::ffi::OsString> {
    // Example input:
    //   ["rsctl", "api", "-rs", "--dir", "...", "--api", "..."]
    // Rewrite to:
    //   ["rsctl", "api", "rs", "--dir", "...", "--api", "..."]
    for i in 0..argv.len().saturating_sub(2) {
        if argv[i] == "api" && argv[i + 1] == "-rs" {
            argv[i + 1] = "rs".into();
            break;
        }
    }
    argv
}
