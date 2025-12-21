// placeholder (axum) - reuse `templates/api/rs/main.tpl`

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
    // 同时兼容环境变量 `CONFIG_FILE`，便于容器化部署。
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_file = parse_config_file();

    // conf.MustLoad(*configFile, &c)
    let c = config::load(&config_file)?;

    // ctx := svc.NewServiceContext(c)
    let svc_ctx = Arc::new(svc::ServiceContext::new(c.clone()));

    // server := rest.MustNewServer(c.RestConf)
    let server = rest::Server::must_new::<Arc<svc::ServiceContext>>(c.rest.clone());

    // handler.RegisterHandlers(server, ctx)
    let app = handler::register_handlers(svc_ctx.clone());

    println!("Starting server at {}:{}...", c.rest.host, c.rest.port);

    // server.Start()
    server.add_routes(app).with_state(svc_ctx).start().await?;
    Ok(())
}