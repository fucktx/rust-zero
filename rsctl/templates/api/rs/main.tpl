// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

use std::{env, net::SocketAddr, sync::Arc};
use tokio::signal;
use axum::Router;

{{.importPackages}}

/// 配置文件路径，默认：`etc/{{.serviceName}}.yaml`
/// 支持通过环境变量覆盖：`CONFIG_FILE=/path/to/config.yaml`
const DEFAULT_CONFIG_FILE: &str = "etc/{{.serviceName}}.yaml";

#[tokio::main]
async fn main() {
    // 解析配置文件路径：优先环境变量，其次默认值
    let config_file = env::var("CONFIG_FILE").unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string());

    // 加载配置：这里假设你有一个 config 模块和 Config 结构体，类似 go-zero 的 Config
    // 你可以在 config.tpl 里定义具体结构，这里只用占位符形式描述。
    let c = match config::load(&config_file) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("failed to load config from {}: {}", config_file, e);
            std::process::exit(1);
        }
    };

    // 组合监听地址：等价 Go 的 c.Host, c.Port
    let addr: SocketAddr = format!("{}:{}", c.host, c.port)
        .parse()
        .expect("invalid host/port in config");

    // 初始化全局 ServiceContext
    let svc_ctx = Arc::new(svc::ServiceContext::new(c.clone()));

    // 构建路由：等价 Go 里的 handler.RegisterHandlers(server, ctx)
    let app: Router = handler::register_handlers(svc_ctx);

    println!("Starting server at {}:{}...", c.host, c.port);

    // 启动服务，并支持 ctrl+c 优雅退出
    if let Err(err) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("server error: {}", err);
    }
}

async fn shutdown_signal() {
    // Ctrl+C
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term =
            signal(SignalKind::terminate()).expect("failed to install signal handler");
        term.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}


