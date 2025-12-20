use anyhow::Result;
use clap::{Parser, Subcommand};

mod api;
mod model;
mod version;

#[derive(Debug, Parser)]
#[command(name = "rsctl", version, about = "Code generator CLI")]
struct App {
    #[command(subcommand)]
    command: Command,
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
    /// Print RS generator version (template/version for generated Rust scaffold)
    Info {
        /// Print RS generator version
        #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
        version: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let argv = preprocess_argv_for_legacy_flags(std::env::args_os().collect());
    let app = App::parse_from(argv);

    match app.command {
        Command::Api { lang } => api::run(lang)?,
        Command::Model { driver } => model::run(driver)?,
        Command::Info { version: _ } => println!("{}", version::VERSION),
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
