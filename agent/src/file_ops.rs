use crate::protocol::{AgentMessage, FileInfo};
use tokio::sync::mpsc;

pub async fn handle_command(
    cmd: AgentMessage,
    msg_tx: &mpsc::UnboundedSender<AgentMessage>,
    game_dir: &str,
) {
    match cmd {
        AgentMessage::ReadFile { request_id, path } => {
            match safe_join(game_dir, &path) {
                Ok(full_path) => {
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
                Err(e) => {
                    let _ = msg_tx.send(AgentMessage::FileReadResult {
                        request_id,
                        success: false,
                        path,
                        content: None,
                        error: Some(e),
                    });
                }
            }
        }
        AgentMessage::WriteFile {
            request_id,
            path,
            content,
        } => {
            match safe_join(game_dir, &path) {
                Ok(full_path) => {
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
                Err(e) => {
                    let _ = msg_tx.send(AgentMessage::FileWriteResult {
                        request_id,
                        success: false,
                        path,
                        error: Some(e),
                    });
                }
            }
        }
        AgentMessage::ListFiles { request_id, dir } => {
            match safe_join(game_dir, &dir) {
                Ok(full_dir) => {
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

/// 安全路径拼接：规范化后验证结果路径在 base 目录内，防止路径遍历攻击
fn safe_join(base: &str, path: &str) -> Result<String, String> {
    let base = base.trim_end_matches(['\\', '/']);
    let path = path.trim_start_matches(['\\', '/']);
    let joined = format!("{}\\{}", base, path);

    let canonical = std::path::Path::new(&joined)
        .canonicalize()
        .map_err(|e| format!("路径无效: {}", e))?;

    let canonical_base = std::path::Path::new(base)
        .canonicalize()
        .map_err(|e| format!("基础路径无效: {}", e))?;

    if !canonical.starts_with(&canonical_base) {
        return Err("路径遍历攻击被阻止".to_string());
    }

    Ok(canonical.to_string_lossy().to_string())
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
