use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use tokio::sync::mpsc;
use crate::protocol::{AgentMessage, LogEntry};

pub fn start_watching(file_path: PathBuf, msg_tx: mpsc::Sender<AgentMessage>) {
    let path_display = file_path.display().to_string();
    eprintln!("[LogWatcher] 启动监听: {}", path_display);
    std::thread::spawn(move || {
        let mut last_pos = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
        eprintln!("[LogWatcher] 初始文件大小: {} bytes, 起始位置: {}", std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0), last_pos);
        let (event_tx, event_rx) = std::sync::mpsc::channel::<notify::Result<Event>>();

        let mut watcher = match RecommendedWatcher::new(
            move |res| {
                let _ = event_tx.send(res);
            },
            Config::default(),
        ) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("创建文件监听器失败: {}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(&file_path, RecursiveMode::NonRecursive) {
            tracing::error!("监听文件失败: {}", e);
            return;
        }

        for event in event_rx {
            if let Ok(event) = event {
                if matches!(event.kind, EventKind::Modify(_)) {
                    if let Ok(mut file) = std::fs::File::open(&file_path) {
                        if let Ok(meta) = file.metadata() {
                            let file_len = meta.len();
                            if file_len > last_pos {
                                if file.seek(SeekFrom::Start(last_pos)).is_err() {
                                    tracing::error!("日志文件 seek 失败");
                                    continue;
                                }
                                let mut buf = vec![0u8; (file_len - last_pos) as usize];
                                if file.read_exact(&mut buf).is_err() {
                                    tracing::error!("日志文件读取失败");
                                    continue;
                                }
                                last_pos = file_len;

                                if let Ok(content) = String::from_utf8(buf) {
                                    for line in content.lines() {
                                        let line = line.trim();
                                        if line.is_empty() {
                                            continue;
                                        }
                                        let entry = parse_log_line(line);
                                        if msg_tx.blocking_send(AgentMessage::Log { data: entry }).is_err() {
                                                            tracing::error!("日志行发送失败（通道已关闭）");
                                                            return;
                                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

fn parse_log_line(line: &str) -> LogEntry {
    let log_level = if line.contains("[Error]") || line.contains("error") {
        "ERROR"
    } else if line.contains("[Warn]") || line.contains("Warning") {
        "WARN"
    } else if line.contains("success") || line.contains("Success") {
        "SUCCESS"
    } else {
        "INFO"
    };

    let category = if line.contains("[RCON]") {
        "RCON"
    } else if line.contains("[Chat]") {
        "Chat"
    } else if line.contains("[Player]") {
        "Player"
    } else if line.contains("[Server]") {
        "Server"
    } else if line.contains("[Anti-Cheat]") {
        "Anti-Cheat"
    } else if line.contains("[Broadcast]") {
        "Broadcast"
    } else {
        "General"
    };

    // 对于聊天行，清理格式以便后端正确解析玩家名和消息
    let message = if category == "Chat" {
        parse_chat_message(line)
    } else {
        line.to_string()
    };

    LogEntry {
        log_level: log_level.to_string(),
        category: Some(category.to_string()),
        message,
        raw_line: Some(line.to_string()),
        logged_at: chrono::Utc::now(),
    }
}

/// 解析聊天日志行，输出 "PlayerName: Message" 格式（去除 [Online IDs:] 等干扰）
fn parse_chat_message(line: &str) -> String {
    let chat_prefixes = ["[ChatAll]", "[ChatTeam]", "[ChatSquad]", "[ChatAdmin]"];
    for prefix in &chat_prefixes {
        if let Some(rest) = line.strip_prefix(prefix) {
            let rest = rest.trim();
            // 剥离 [Online IDs: ...] 块
            let cleaned = if let Some(start) = rest.find("[Online IDs:") {
                if let Some(end) = rest[start..].find(']') {
                    let end_pos = start + end + 1;
                    format!("{}{}", &rest[..start], &rest[end_pos..])
                } else {
                    rest.to_string()
                }
            } else {
                rest.to_string()
            };
            // 查找 " : " 分隔符提取玩家名和消息
            if let Some(pos) = cleaned.find(" : ") {
                let player = cleaned[..pos].trim();
                let msg = cleaned[pos + 3..].trim();
                if !player.is_empty() {
                    return format!("{}: {}", player, msg);
                }
            }
            break;
        }
    }
    // 无法解析时回退到原始行
    line.to_string()
}
