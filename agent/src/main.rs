mod ban_enforcer;
mod batch_uploader;
mod config;
mod event_manager;
mod file_ops;
mod log_parser;
mod log_watcher;
mod offset_store;
mod player_tracker;
mod protocol;
mod rcon_listener;
mod ws_client;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let env_path = exe_dir.join(".env");
            eprintln!("[Env] 尝试加载: {}", env_path.display());
            match dotenvy::from_path(&env_path) {
                Ok(_) => eprintln!("[Env] 加载成功"),
                Err(e) => eprintln!("[Env] 加载失败: {}", e),
            }
        }
    }

    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();
    tracing::info!("Agent 启动, token: {}...", &config.token[..16.min(config.token.len())]);

    // 取消令牌，用于优雅关闭
    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    // 监听 Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("收到关闭信号，开始优雅关闭...");
        cancel_clone.cancel();
    });

    // Channel A: log_watcher/file_ops → ws_client → WebSocket → 后端
    let (to_ws_tx, to_ws_rx) = mpsc::channel::<protocol::AgentMessage>(256);

    // Channel B: WebSocket(后端消息) → ws_client → file_ops
    let (to_agent_tx, to_agent_rx) = mpsc::channel::<protocol::AgentMessage>(64);

    // 创建 offset 存储
    let mut offset_store = offset_store::OffsetStore::new(config.offset_file_path.clone());

    // 创建事件管理器
    let event_manager = Arc::new(event_manager::EventManager::new(4096));

    // 创建批量上传器
    let (batch_tx, batch_rx) = mpsc::channel::<batch_uploader::SerializedEvent>(4096);
    let uploader = Arc::new(batch_uploader::BatchUploader::new(
        config.batch_max_size,
        config.batch_max_delay_ms,
        batch_tx,
        config.compression_enabled,
        config.compression_level,
        config.compression_min_bytes,
    ));

    // 启动批量上传定时刷新任务
    let uploader_clone = Arc::clone(&uploader);
    tokio::spawn(async move {
        uploader_clone.start_flush_task().await;
    });

    // 处理批量上传后的事件发送到 WebSocket
    let to_ws_tx_clone = to_ws_tx.clone();
    tokio::spawn(async move {
        let mut batch_rx = batch_rx;
        while let Some(event) = batch_rx.recv().await {
            let msg = protocol::AgentMessage::EventBatch {
                batch_id: uuid::Uuid::new_v4().to_string(),
                events: vec![protocol::EventData {
                    event_id: event.event_id,
                    event_type: event.event_type,
                    timestamp: event.timestamp,
                    data: event.data,
                    raw_log: event.raw_log,
                }],
                compression: None,
            };
            if let Err(e) = to_ws_tx_clone.send(msg).await {
                tracing::error!("发送事件到后端失败: {}", e);
            }
        }
    });

    // Channel C: 后端 RCON 命令 → rcon_listener
    let (rcon_cmd_tx, rcon_cmd_rx) = mpsc::channel::<String>(32);

    // 创建玩家追踪器
    let player_tracker = Arc::new(player_tracker::PlayerTracker::new(event_manager.clone()));
    player_tracker.start().await;

    // 创建封禁执行器
    let ban_enforcer = Arc::new(ban_enforcer::BanEnforcer::new(
        rcon_cmd_tx.clone(),
        event_manager.clone(),
        player_tracker.clone(),
    ));
    ban_enforcer.start().await;

    // WebSocket 客户端
    let ws_tx = to_agent_tx.clone();
    let ws_rx = Arc::new(Mutex::new(to_ws_rx));
    let ws_url = format!("{}?token={}", config.backend_ws_url, config.token);
    let ws_handle = tokio::spawn(async move {
        ws_client::run(ws_url, ws_tx, ws_rx).await;
    });

    // RCON 长连接
    if !config.rcon_password.is_empty() {
        rcon_listener::start_rcon_listener(
            config.rcon_host.clone(),
            config.rcon_port,
            config.rcon_password.clone(),
            to_ws_tx.clone(),
            rcon_cmd_rx,
            config.rcon_poll_interval_secs,
            config.auto_broadcast_interval_secs,
            config.auto_broadcast_message.clone(),
            config.welcome_message.clone(),
            config.admins_cfg_path.clone(),
        );
    }

    // 日志文件监听
    let log_path = find_squad_log_file(&config.log_file_path, &config.game_dir);
    if let Some(path) = log_path {
        let _ = offset_store.load(&path);
        log_watcher::start_watching(
            path.clone(),
            to_ws_tx.clone(),
            offset_store,
            uploader,
            event_manager.clone(),
        );
        tracing::info!("日志监听已启动: {}", path.display());
    } else {
        tracing::warn!("找不到 Squad 日志文件（LOG_FILE_PATH={:?}, GAME_DIR={:?}）", config.log_file_path, config.game_dir);
    }

    // 处理来自后端的命令
    let config_dir = config.config_dir.clone();
    let cmd_tx = to_ws_tx.clone();
    let cmd_handle = tokio::spawn(async move {
        let mut rx = to_agent_rx;
        while let Some(cmd) = rx.recv().await {
            match cmd {
                protocol::AgentMessage::SendRcon { command } => {
                    let _ = rcon_cmd_tx.send(command).await;
                }
                other => file_ops::handle_command(other, &cmd_tx, &config_dir).await,
            }
        }
    });

    // 等待取消信号
    cancel_token.cancelled().await;
    tracing::info!("Agent 正在关闭...");

    // 等待任务完成
    let _ = tokio::join!(ws_handle, cmd_handle);
    tracing::info!("Agent 已关闭");

    Ok(())
}

fn find_squad_log_file(configured: &str, game_dir: &str) -> Option<PathBuf> {
    let candidates = [
        Some(PathBuf::from(configured)),
        (!game_dir.is_empty() && game_dir != r"C:\game").then(|| {
            PathBuf::from(game_dir).join("SquadGame").join("ServerLog").join("SquadGame.log")
        }),
        (!game_dir.is_empty() && game_dir != r"C:\game").then(|| {
            PathBuf::from(game_dir).join("SquadGame").join("Saved").join("Logs").join("SquadGame.log")
        }),
        Some(PathBuf::from("./SquadGame/ServerLog/SquadGame.log")),
        Some(PathBuf::from("./SquadGame/Saved/Logs/SquadGame.log")),
    ];

    for path in candidates.into_iter().flatten() {
        eprintln!("[Log] 检查日志路径: {}", path.display());
        if path.exists() {
            eprintln!("[Log] 发现日志文件: {}", path.display());
            return Some(path);
        }
    }
    None
}