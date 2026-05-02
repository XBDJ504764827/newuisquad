mod config;
mod file_ops;
mod log_watcher;
mod protocol;
mod ws_client;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let ws_url = format!("{}?token={}", config.backend_ws_url, config.token);

    // WebSocket 客户端
    let ws_tx = to_agent_tx.clone();
    let ws_rx = Arc::new(Mutex::new(to_ws_rx));
    let ws_handle = tokio::spawn(async move {
        ws_client::run(ws_url, ws_tx, ws_rx).await;
    });

    // 日志文件监听 → Channel A
    let log_path = PathBuf::from(&config.log_file_path);
    if log_path.exists() {
        log_watcher::start_watching(log_path, to_ws_tx.clone());
        tracing::info!("日志监听已启动: {}", config.log_file_path);
    } else {
        tracing::warn!("日志文件不存在: {}", config.log_file_path);
    }

    // 处理来自后端的命令（Channel B → file_ops，结果 → Channel A）
    let config_dir = config.config_dir.clone();
    let cmd_tx = to_ws_tx.clone();
    let cmd_handle = tokio::spawn(async move {
        let mut rx = to_agent_rx;
        while let Some(cmd) = rx.recv().await {
            file_ops::handle_command(cmd, &cmd_tx, &config_dir).await;
        }
    });

    let _ = tokio::join!(ws_handle, cmd_handle);
    Ok(())
}
