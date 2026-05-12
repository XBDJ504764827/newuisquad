use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::thread;
use tokio::sync::mpsc;
use crate::protocol::{AgentMessage, LogEntry};
use crate::offset_store::OffsetStore;
use crate::batch_uploader::{BatchUploader, SerializedEvent};
use crate::event_manager::EventManager;
use std::sync::Arc;
use chrono::Utc;

pub fn start_watching(
    file_path: PathBuf,
    msg_tx: mpsc::Sender<AgentMessage>,
    mut offset_store: OffsetStore,
    uploader: Arc<BatchUploader>,
    event_manager: Arc<EventManager>,
) -> thread::JoinHandle<()> {
    let path_display = file_path.display().to_string();
    tracing::info!("启动日志监听: {}", path_display);

    let (event_tx, _event_rx) = std::sync::mpsc::channel::<notify::Result<Event>>();

    let watcher = match RecommendedWatcher::new(
        move |res| {
            let _ = event_tx.send(res);
        },
        Config::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("创建文件监听器失败: {}", e);
            return thread::spawn(|| {});
        }
    };

    // watcher 需要在作用域内保持存活
    let _watcher = watcher;

    // 启动 offset 持久化定时刷新任务
    let offset_store_clone = offset_store.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(std::time::Duration::from_secs(10));
            let _ = offset_store_clone.persist();
        }
    });

    // 监听文件修改事件
    thread::spawn(move || {
        // 重新创建 watcher（上面的 watcher 已被 move）
        let (event_tx2, event_rx2) = std::sync::mpsc::channel::<notify::Result<Event>>();
        let mut watcher2 = match RecommendedWatcher::new(
            move |res| { let _ = event_tx2.send(res); },
            Config::default(),
        ) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("创建文件监听器失败: {}", e);
                return;
            }
        };

        if let Err(e) = watcher2.watch(&file_path, RecursiveMode::NonRecursive) {
            tracing::error!("监听文件失败: {}", e);
            return;
        }

        for event in event_rx2 {
            if let Ok(event) = event {
                if matches!(event.kind, EventKind::Modify(_)) {
                    if let Ok(mut file) = std::fs::File::open(&file_path) {
                        if let Ok(meta) = file.metadata() {
                            let file_len = meta.len();

                            let mut last_pos = offset_store.get_offset();

                            // 处理日志截断
                            if file_len < last_pos {
                                tracing::warn!("日志文件已截断，重置读取位置: {} -> {}", last_pos, file_len);
                                last_pos = 0;
                                offset_store.set_offset(last_pos);
                            }

                            if file_len <= last_pos {
                                continue;
                            }

                            if file.seek(SeekFrom::Start(last_pos)).is_err() {
                                tracing::error!("日志文件 seek 失败");
                                continue;
                            }

                            let mut buf = vec![0u8; (file_len - last_pos) as usize];
                            if file.read_exact(&mut buf).is_err() {
                                tracing::error!("日志文件读取失败");
                                continue;
                            }

                            let content = String::from_utf8_lossy(&buf);
                            let mut new_pos = last_pos;

                            for line in content.lines() {
                                let line = line.trim();
                                if line.is_empty() {
                                    continue;
                                }

                                new_pos += (line.len() + 1) as u64;

                                // 1. 先尝试结构化解析
                                if let Some(parsed) = crate::log_parser::process_log_line(line) {
                                    // 发布到 EventManager（内部消费）
                                    let event = crate::log_parser::parsed_to_event(parsed);
                                    event_manager.publish(event.clone());

                                    // 创建批量事件（上报后端）
                                    let batch_event = SerializedEvent {
                                        event_id: event.id.to_string(),
                                        event_type: format!("{:?}", event.event_type),
                                        timestamp: event.timestamp.timestamp(),
                                        data: event.data.clone(),
                                        raw_log: event.raw_log.clone(),
                                    };

                                    let uploader = uploader.clone();
                                    tokio::task::block_in_place(|| {
                                        let rt = tokio::runtime::Handle::current();
                                        rt.block_on(async {
                                            if let Err(e) = uploader.add_event(batch_event).await {
                                                tracing::error!("添加事件到批次失败: {}", e);
                                            }
                                        });
                                    });
                                }

                                // 2. 保留原有简单分类解析（兼容未匹配的行）
                                if let Some(entry) = parse_log_line(line) {
                                    if msg_tx.blocking_send(AgentMessage::Log { data: entry }).is_err() {
                                        tracing::error!("日志发送失败（通道已关闭）");
                                        break;
                                    }
                                }
                            }

                            offset_store.set_offset(new_pos);

                            tracing::trace!("处理完成，新位置: {} bytes", new_pos);
                        }
                    }
                }
            }
        }
    })
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
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

    let message = if category == "Chat" {
        parse_chat_message(line)
    } else {
        line.to_string()
    };

    Some(LogEntry {
        log_level: log_level.to_string(),
        category: Some(category.to_string()),
        message,
        raw_line: Some(line.to_string()),
        logged_at: Utc::now(),
    })
}

fn parse_chat_message(line: &str) -> String {
    let chat_prefixes = ["[ChatAll]", "[ChatTeam]", "[ChatSquad]", "[ChatAdmin]"];

    for prefix in &chat_prefixes {
        if let Some(rest) = line.strip_prefix(prefix) {
            let rest = rest.trim();
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
            if let Some(pos) = cleaned.find(": ") {
                let player = cleaned[..pos].trim();
                let msg = cleaned[pos + 3..].trim();
                if !player.is_empty() {
                    return format!("{}: {}", player, msg);
                }
            }
            break;
        }
    }
    line.to_string()
}
