use crate::protocol::{AgentMessage, FileInfo};
use tokio::sync::mpsc;

pub async fn handle_command(
    cmd: AgentMessage,
    msg_tx: &mpsc::UnboundedSender<AgentMessage>,
    game_dir: &str,
) {
    match cmd {
        AgentMessage::ReadFile { request_id, path } => {
            let full_path = safe_join(game_dir, &path);
            match tokio::fs::read_to_string(&full_path).await {
                Ok(content) => {
                    let _ = msg_tx.send(AgentMessage::FileReadResult {
                        request_id,
                        success: true,
                        path,
                        content: Some(content),
                        error: None,
                    });
                }
                Err(e) => {
                    let _ = msg_tx.send(AgentMessage::FileReadResult {
                        request_id,
                        success: false,
                        path,
                        content: None,
                        error: Some(format!("读取失败: {}", e)),
                    });
                }
            }
        }
        AgentMessage::WriteFile {
            request_id,
            path,
            content,
        } => {
            let full_path = safe_join(game_dir, &path);
            match tokio::fs::write(&full_path, &content).await {
                Ok(_) => {
                    let _ = msg_tx.send(AgentMessage::FileWriteResult {
                        request_id,
                        success: true,
                        path,
                        error: None,
                    });
                }
                Err(e) => {
                    let _ = msg_tx.send(AgentMessage::FileWriteResult {
                        request_id,
                        success: false,
                        path,
                        error: Some(format!("写入失败: {}", e)),
                    });
                }
            }
        }
        AgentMessage::ListFiles { request_id, dir } => {
            let full_dir = safe_join(game_dir, &dir);
            match list_dir(&full_dir).await {
                Ok(files) => {
                    let _ = msg_tx.send(AgentMessage::FileListResult { request_id, files });
                }
                Err(e) => {
                    let _ = msg_tx.send(AgentMessage::FileListResult {
                        request_id,
                        files: vec![FileInfo {
                            name: format!("错误: {}", e),
                            size: 0,
                        }],
                    });
                }
            }
        }
        _ => {}
    }
}

fn safe_join(base: &str, path: &str) -> String {
    let base = base.trim_end_matches(['\\', '/']);
    let path = path.trim_start_matches(['\\', '/']);
    format!("{}\\{}", base, path)
}

async fn list_dir(dir: &str) -> Result<Vec<FileInfo>, String> {
    let mut entries = tokio::fs::read_dir(dir)
        .await
        .map_err(|e| format!("{}", e))?;
    let mut files = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Ok(meta) = entry.metadata().await {
            if meta.is_file() {
                files.push(FileInfo {
                    name: entry.file_name().to_string_lossy().to_string(),
                    size: meta.len(),
                });
            }
        }
    }
    Ok(files)
}
