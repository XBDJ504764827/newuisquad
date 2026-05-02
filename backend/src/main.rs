mod api;
mod config;
mod db;
mod models;
mod protocol;
mod repositories;
mod rcon_client;
mod services;

use std::net::SocketAddr;
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

    // Agent 连接池（生产环境日志来源）
    let agent_pool = api::agent_ws::AgentPool::new();
    let log_tx = agent_pool.log_tx();
    let log_rx1 = log_tx.subscribe();
    let log_rx2 = log_tx.subscribe();

    let state = api::AppState {
        pool: pool.clone(),
        log_broadcast: Some(Arc::new(log_tx)),
        agent_pool: Some(agent_pool),
    };

    // 启动误杀检测服务
    services::tk_service::start_tk_monitor(pool.clone(), log_rx1);
    // 启动广播处理服务
    services::broadcast_handler::start_broadcast_handler(pool, log_rx2);

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
