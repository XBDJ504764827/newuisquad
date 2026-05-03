use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use tokio::sync::mpsc;
use crate::protocol::{AgentMessage, LogEntry};

pub fn start_watching(file_path: PathBuf, msg_tx: mpsc::UnboundedSender<AgentMessage>) {
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
                                    let line_count = content.lines().count();
                                    eprintln!("[LogWatcher] 检测到文件变化, 新增 {} 行", line_count);
                                    for line in content.lines() {
                                        let line = line.trim();
                                        if line.is_empty() {
                                            continue;
                                        }
                                        let entry = parse_log_line(line);
                                        eprintln!("[LogWatcher] -> [{}] {}", entry.log_level, entry.message);
                                        if msg_tx.send(AgentMessage::Log { data: entry }).is_err() {
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

    LogEntry {
        log_level: log_level.to_string(),
        category: Some(category.to_string()),
        message: line.to_string(),
        raw_line: Some(line.to_string()),
        logged_at: chrono::Utc::now(),
    }
}
