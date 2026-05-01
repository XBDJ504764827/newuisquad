use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::SeekFrom;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::broadcast;
use crate::models::server_log::LogEntry;

pub fn start_watching(
    file_path: PathBuf,
    db_pool: sqlx::PgPool,
    server_id: i32,
) -> broadcast::Sender<LogEntry> {
    let (tx, _) = broadcast::channel::<LogEntry>(256);
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<notify::Result<Event>>(128);

        let mut watcher = match RecommendedWatcher::new(
            move |res| { let _ = event_tx.blocking_send(res); },
            Config::default()
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

        let mut last_pos = 0u64;
        while let Some(Ok(event)) = event_rx.recv().await {
            if matches!(event.kind, EventKind::Modify(_)) {
                let Ok(mut file) = tokio::fs::File::open(&file_path).await else {
                    continue;
                };

                let Ok(metadata) = file.metadata().await else { continue };
                let file_len = metadata.len();

                if file_len > last_pos {
                    let _ = file.seek(SeekFrom::Start(last_pos)).await;
                    let mut buf = vec![0u8; (file_len - last_pos) as usize];
                    let _ = file.read_exact(&mut buf).await;
                    last_pos = file_len;

                    let Ok(content) = String::from_utf8(buf) else { continue };

                    for line in content.lines() {
                        let line = line.trim();
                        if line.is_empty() { continue; }
                        let entry = parse_log_line(line);
                        let _ = tx_clone.send(entry.clone());
                        let _ = crate::repositories::server_log_repo::insert_log_entry(
                            &db_pool, server_id, &entry,
                        ).await;
                    }
                }
            }
        }
    });

    tx
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

    let category = if line.contains("[RCON]") { "RCON" }
        else if line.contains("[Chat]") { "Chat" }
        else if line.contains("[Player]") { "Player" }
        else if line.contains("[Server]") { "Server" }
        else if line.contains("[Anti-Cheat]") { "Anti-Cheat" }
        else if line.contains("[Broadcast]") { "Broadcast" }
        else { "General" };

    LogEntry {
        log_level: log_level.to_string(),
        category: Some(category.to_string()),
        message: line.to_string(),
        raw_line: Some(line.to_string()),
        logged_at: chrono::Utc::now(),
    }
}
