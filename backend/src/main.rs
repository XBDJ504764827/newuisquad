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
    let log_rx3 = log_tx.subscribe();

    // 初始化默认管理员
    init_admin(&pool, &config).await?;

    let state = api::AppState {
        pool: pool.clone(),
        log_broadcast: Some(Arc::new(log_tx)),
        agent_pool: Some(agent_pool),
        steam_api_key: config.steam_api_key.clone(),
        jwt_secret: config.jwt_secret.clone(),
        server_states: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
    };

    // 启动误杀检测服务
    services::tk_service::start_tk_monitor(pool.clone(), log_rx1);
    // 启动广播处理服务
    services::broadcast_handler::start_broadcast_handler(pool.clone(), log_rx2);
    // 启动伤害/TK通知服务
    services::damage_notify_service::start_damage_notify(pool, log_rx3);

    let cors = if config.allowed_origin == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(tower_http::cors::AllowOrigin::list(config.allowed_origin.split(',').map(|s| s.trim().parse().unwrap())))
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let router = api::build_router(state).layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;

    tracing::info!("服务器监听: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

async fn init_admin(pool: &sqlx::PgPool, config: &config::Config) -> anyhow::Result<()> {
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM admin_users WHERE username=$1")
        .bind(&config.init_admin_username).fetch_one(pool).await?;
    if exists == 0 {
        let hash = bcrypt::hash(&config.init_admin_password, 12)?;
        sqlx::query("INSERT INTO admin_users (username, password_hash, role, permissions) VALUES ($1,$2,$3,$4::jsonb)")
            .bind(&config.init_admin_username).bind(&hash).bind("超级管理员")
            .bind(serde_json::json!({"all": true}))
            .execute(pool).await?;
        tracing::info!("已创建默认管理员: {}", config.init_admin_username);
    }
    Ok(())
}
