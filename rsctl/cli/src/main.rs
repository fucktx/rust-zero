use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "rsctl", version, about = "Code generator CLI")]
struct Args {
    /// 模板根目录（默认：rsctl/templates）
    #[arg(long)]
    templates: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let _args = Args::parse();
    Ok(())
}


