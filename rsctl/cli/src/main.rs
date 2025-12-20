use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

mod api;
mod model;
mod version;

#[derive(Debug, Parser)]
#[command(name = "rsctl", version, disable_version_flag = true, about = "Code generator CLI")]
struct App {
    #[arg(short = 'v', long = "version", global = true, action = clap::ArgAction::SetTrue)]
    version: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// API generation
    Api {
        #[command(subcommand)]
        api: api::Api,
    },
    /// Model generation
    Model {
        #[command(subcommand)]
        model: model::Model,
    },
    // Rpc {
    //     #[command(subcommand)]
    //
    // }
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


    match app.command {
        Some(Command::Api { api }) => api::run(api)?,
        Some(Command::Model { model }) => model::run(model)?,
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
