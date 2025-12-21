// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

mod config;
mod handler;
mod logic;
mod middleware;
mod svc;
mod types;

use std::{env, sync::Arc};

{{.importPackages}}

/// 配置文件路径，默认：`etc/config.yaml`
const DEFAULT_CONFIG_FILE: &str = "etc/config.yaml";

fn parse_config_file() -> String {
    // 对齐 go-zero: `-f <configFile>`
    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        if arg == "-f" || arg == "--config" {
            if let Some(v) = it.next() {
                return v;
            }
        }
    }
    env::var("CONFIG_FILE").unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_file = parse_config_file();
    let c = match config::load(&config_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load config from {}: {}", config_file, e);
            std::process::exit(1);
        }
    };

    let svc_ctx = Arc::new(svc::ServiceContext::new(c.clone()));
    let engine = rest::Engine::new(c.rest.clone());

    println!("Starting server at {}:{}...", c.rest.host, c.rest.port);

    // actix：native::run 需要 factory closure
    let factory = move || handler::register_handlers(svc_ctx.clone());
    rest::native::run(engine, factory).await
}

