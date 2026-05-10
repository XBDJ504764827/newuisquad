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
    services::system_log::backend_info(&pool, "main", "数据库迁移完成，后端启动中").await;

    // Agent 连接池（生产环境日志来源）
    let agent_pool = api::agent_ws::AgentPool::new();
    let log_tx = agent_pool.log_tx();
    let log_rx1 = log_tx.subscribe();
    let log_rx2 = log_tx.subscribe();

    // 初始化默认管理员
    init_admin(&pool, &config).await?;

    // 初始化批量日志写入器
    let log_batcher = services::log_batcher::LogBatcher::new(pool.clone());

    // 启动 RCON 连接池（每连接独立命令队列、优先级、健康检查、自动重连）
    let rcon_pool = rcon_client::pool::RconPool::with_db(pool.clone());
    // 创建事件管理器（所有服务通过它解耦通信）
    let event_manager = Arc::new(services::event_manager::EventManager::new(10000));
    // 启动实时玩家追踪服务
    let player_tracker = Arc::new(services::player_tracker::PlayerTracker::new(pool.clone(), rcon_pool.clone(), Some(event_manager.clone())));
    let pt_handle = player_tracker.clone().start();
    // 初始化聊天审核服务
    let chat_automod = Arc::new(tokio::sync::RwLock::new(services::chat_automod::ChatAutomod::new()));
    {
        let mut ca = chat_automod.write().await;
        ca.load_settings(&pool).await;
        ca.refresh_admin_cache(&pool).await;
    }
    // 启动服务器健康监控
    let server_monitor = Arc::new(services::server_monitor::ServerMonitor::new(pool.clone(), rcon_pool.clone()));
    let sm_handle = server_monitor.clone().start();
    // 启动播种模式服务
    let seeding_service = Arc::new(services::seeding_service::SeedModeService::new(pool.clone(), rcon_pool.clone()));
    let seed_handle = seeding_service.clone().start();
    // 初始化队伍平衡服务
    let team_balance = Arc::new(services::team_balance_service::TeamBalanceService::new(pool.clone(), rcon_pool.clone()));
    // 启动 AFK 管理服务
    let afk_service = Arc::new(services::afk_service::AfkService::new(pool.clone(), rcon_pool.clone()));
    let afk_handle = afk_service.clone().start();
    // 启动 Ban 强制执行器（订阅玩家连接事件自动踢出）
    let ban_enforcer = Arc::new(services::ban_enforcer::BanEnforcer::new(pool.clone(), rcon_pool.clone()));
    let ban_rx = event_manager.subscribe();
    let ban_handle = ban_enforcer.start(ban_rx);

    let state = api::AppState {
        pool: pool.clone(),
        log_broadcast: Some(Arc::new(log_tx)),
        agent_pool: Some(agent_pool),
        steam_api_key: config.steam_api_key.clone(),
        jwt_secret: config.jwt_secret.clone(),
        server_states: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        team_switch_cache: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        log_batcher,
        rate_limiter: api::rate_limiter::RateLimiterState::new(),
        rcon_pool: rcon_pool.clone(),
        player_tracker: Some(player_tracker.clone()),
        chat_automod: Some(chat_automod.clone()),
        server_monitor: Some(server_monitor.clone()),
        seeding_service: Some(seeding_service.clone()),
        team_balance: Some(team_balance.clone()),
        afk_service: Some(afk_service.clone()),
        event_manager: Some(event_manager.clone()),
        permission_version_cache: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    // 启动广播处理服务
    let bc_handle = services::broadcast_handler::start_broadcast_handler(pool.clone(), log_rx1, rcon_pool.clone(), chat_automod.clone());
    // 启动统一伤害与误伤通知服务
    let server_states_dn = state.server_states.clone();
    let dn_rcon_pool = rcon_pool.clone();
    let dn_handle = services::damage_notify_service::start_damage_notify(pool.clone(), log_rx2, server_states_dn, dn_rcon_pool);
    let log_pool = pool;

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
    services::system_log::backend_info(&log_pool, "main", &format!("后端服务已启动，监听 {}", addr)).await;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // 优雅关闭：捕获 SIGTERM/Ctrl+C，等待后台任务完成
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await
    });

    tokio::select! {
        result = server_handle => {
            result??;
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("收到关闭信号，等待后台任务完成...");
        }
    }

    // 等待后台服务任务结束（最多 10 秒超时）
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        async {
            let _ = tokio::join!(bc_handle, dn_handle, pt_handle, sm_handle, seed_handle, afk_handle, ban_handle);
        }
    ).await;
    tracing::info!("服务已关闭");
    services::system_log::backend_info(&log_pool, "main", "后端服务已关闭").await;

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
