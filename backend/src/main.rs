mod api;
mod config;
mod db;
mod log_watcher;
mod models;
mod rcon_client;
mod repositories;
mod services;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let _ = dotenvy::dotenv();

    let config = config::Config::from_env();
    tracing::info!("启动管理控制台后端...");

    let pool = db::create_pool(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("数据库迁移完成");

    let log_broadcast = {
        let path = PathBuf::from(&config.log_file_path);
        if path.exists() {
            let tx = services::log_service::start_log_watcher(path, pool.clone(), 1);
            tracing::info!("日志监听器已启动: {}", config.log_file_path);
            Some(Arc::new(tx))
        } else {
            tracing::warn!("日志文件不存在: {}，跳过日志监听", config.log_file_path);
            None
        }
    };

    let state = api::AppState { pool, log_broadcast };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = api::build_router(state).layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;

    tracing::info!("服务器监听: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
