mod config;
mod file_ops;
mod log_watcher;
mod protocol;
mod rcon_listener;
mod ws_client;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 安装 rustls 加密提供程序（wss:// 必需）
    let _ = rustls::crypto::ring::default_provider().install_default();

    // 从 exe 同目录加载 .env
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

    // 不兜底加载当前目录的 .env，避免意外覆盖

    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();
    tracing::info!("Agent 启动, token: {}...", &config.token[..16.min(config.token.len())]);

    // Channel A: log_watcher/file_ops → ws_client → WebSocket → 后端
    let (to_ws_tx, to_ws_rx) = mpsc::unbounded_channel::<protocol::AgentMessage>();

    // Channel B: WebSocket(后端消息) → ws_client → file_ops
    let (to_agent_tx, to_agent_rx) = mpsc::unbounded_channel::<protocol::AgentMessage>();

    // WebSocket 客户端（token 通过 URL query param 传递，兼容 tungstenite 自定义请求限制）
    let ws_tx = to_agent_tx.clone();
    let ws_rx = Arc::new(Mutex::new(to_ws_rx));
    let ws_url = format!("{}?token={}", config.backend_ws_url, config.token);
    let ws_handle = tokio::spawn(async move {
        ws_client::run(ws_url, ws_tx, ws_rx).await;
    });

    // Channel C: 后端 RCON 命令 → rcon_listener
    let (rcon_cmd_tx, rcon_cmd_rx) = mpsc::unbounded_channel::<String>();

    // RCON 长连接 → Channel A（事件监听 + 状态查询 + 自动广播 + 欢迎消息）
    if !config.rcon_password.is_empty() {
        rcon_listener::start_rcon_listener(
            config.rcon_host.clone(),
            config.rcon_port,
            config.rcon_password.clone(),
            to_ws_tx.clone(),
            rcon_cmd_rx,
            config.auto_broadcast_interval_secs,
            config.auto_broadcast_message.clone(),
            config.welcome_message.clone(),
        );
    }

    // 日志文件监听 → Channel A
    let log_path = find_squad_log_file(&config.log_file_path, &config.game_dir);
    if let Some(path) = log_path {
        log_watcher::start_watching(path.clone(), to_ws_tx.clone());
        tracing::info!("日志监听已启动: {}", path.display());
    } else {
        tracing::warn!("找不到 Squad 日志文件（LOG_FILE_PATH={:?}, GAME_DIR={:?}, 将使用 RCON 推送作为聊天来源）", config.log_file_path, config.game_dir);
    }

    // 处理来自后端的命令（Channel B → file_ops / rcon_cmd_tx，结果 → Channel A）
    let config_dir = config.config_dir.clone();
    let cmd_tx = to_ws_tx.clone();
    let cmd_handle = tokio::spawn(async move {
        let mut rx = to_agent_rx;
        while let Some(cmd) = rx.recv().await {
            match cmd {
                protocol::AgentMessage::SendRcon { command } => {
                    let _ = rcon_cmd_tx.send(command);
                }
                other => file_ops::handle_command(other, &cmd_tx, &config_dir).await,
            }
        }
    });

    let _ = tokio::join!(ws_handle, cmd_handle);
    Ok(())
}

/// 查找 Squad 日志文件：优先使用配置路径，失败则从 GAME_DIR 推断
fn find_squad_log_file(configured: &str, game_dir: &str) -> Option<PathBuf> {
    let candidates = [
        // 1. 用户配置的路径
        Some(PathBuf::from(configured)),
        // 2. 从 GAME_DIR 推导 Squad 日志目录
        (!game_dir.is_empty() && game_dir != r"C:\game").then(|| {
            PathBuf::from(game_dir).join("SquadGame").join("ServerLog").join("SquadGame.log")
        }),
        // 3. 从 GAME_DIR/SquadGame/Saved/Logs 推导（UE5 默认路径）
        (!game_dir.is_empty() && game_dir != r"C:\game").then(|| {
            PathBuf::from(game_dir).join("SquadGame").join("Saved").join("Logs").join("SquadGame.log")
        }),
        // 4. 当前目录下的 SquadGame
        Some(PathBuf::from("./SquadGame/ServerLog/SquadGame.log")),
        Some(PathBuf::from("./SquadGame/Saved/Logs/SquadGame.log")),
    ];

    for path in candidates.into_iter().flatten() {
        eprintln!("[Log] 检查日志路径: {}", path.display());
        if path.exists() {
            eprintln!("[Log] ✅ 发现日志文件: {}", path.display());
            return Some(path);
        }
    }
    None
}
