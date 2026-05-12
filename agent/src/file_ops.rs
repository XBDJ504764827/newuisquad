use crate::protocol::{AgentMessage, FileInfo};
use tokio::sync::mpsc;

pub async fn handle_command(
    cmd: AgentMessage,
    msg_tx: &mpsc::Sender<AgentMessage>,
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
                            }).await;
                        }
                        Err(e) => {
                            let _ = msg_tx.send(AgentMessage::FileReadResult {
                                request_id,
                                success: false,
                                path,
                                content: None,
                                error: Some(format!("读取失败: {}", e)),
                            }).await;
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
                    }).await;
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
                    match atomic_write_with_backup(&full_path, &content, &request_id).await {
                        Ok(_) => {
                            let _ = msg_tx.send(AgentMessage::FileWriteResult {
                                request_id,
                                success: true,
                                path,
                                error: None,
                            }).await;
                        }
                        Err(e) => {
                            let _ = msg_tx.send(AgentMessage::FileWriteResult {
                                request_id,
                                success: false,
                                path,
                                error: Some(format!("写入失败: {}", e)),
                            }).await;
                        }
                    }
                }
                Err(e) => {
                    let _ = msg_tx.send(AgentMessage::FileWriteResult {
                        request_id,
                        success: false,
                        path,
                        error: Some(e),
                    }).await;
                }
            }
        }
        AgentMessage::ListFiles { request_id, dir } => {
            match safe_join(game_dir, &dir) {
                Ok(full_dir) => {
                    match list_dir(&full_dir).await {
                        Ok(files) => {
                            let _ = msg_tx.send(AgentMessage::FileListResult { request_id, files }).await;
                        }
                        Err(e) => {
                            let _ = msg_tx.send(AgentMessage::FileListResult {
                                request_id,
                                files: vec![FileInfo {
                                    name: format!("错误: {}", e),
                                    size: 0,
                                }],
                            }).await;
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
                    }).await;
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

/// 原子写入：备份 + 临时文件 + rename
async fn atomic_write_with_backup(target_path: &str, content: &str, request_id: &str) -> Result<(), String> {
    use std::path::Path;

    let target = Path::new(target_path);
    let target_dir = target.parent().ok_or_else(|| "无效目标路径".to_string())?;

    // 确保目录存在
    tokio::fs::create_dir_all(target_dir)
        .await
        .map_err(|e| format!("创建目录失败: {}", e))?;

    // 备份现有文件
    if tokio::fs::metadata(target).await.is_ok() {
        let backup_path = format!("{}.bak.{}", target_path, chrono::Utc::now().timestamp());
        if let Err(e) = tokio::fs::copy(target, &backup_path).await {
            tracing::warn!("创建备份失败: {}", e);
        }
    }

    // 写入临时文件
    let tmp_path = format!("{}.tmp.{}", target_path, request_id.replace('-', ""));
    tokio::fs::write(&tmp_path, content)
        .await
        .map_err(|e| format!("写入临时文件失败: {}", e))?;

    // 删除旧文件（如果存在）
    if tokio::fs::metadata(target).await.is_ok() {
        tokio::fs::remove_file(target)
            .await
            .map_err(|e| format!("删除旧文件失败: {}", e))?;
    }

    // 原子 rename
    tokio::fs::rename(&tmp_path, target)
        .await
        .map_err(|e| {
            let _ = std::fs::remove_file(&tmp_path);
            format!("重命名失败: {}", e)
        })?;

    // 清理旧备份（保留最近 1 个）
    if let Ok(mut entries) = tokio::fs::read_dir(target_dir).await {
        let base_name = target.file_name().unwrap_or_default().to_string_lossy();
        let backup_prefix = format!("{}.bak.", base_name);
        let mut backups: Vec<(String, i64)> = Vec::new();

        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&backup_prefix) {
                if let Ok(ts) = name[backup_prefix.len()..].parse::<i64>() {
                    backups.push((format!("{}/{}", target_dir.display(), name), ts));
                }
            }
        }

        backups.sort_by(|a, b| b.1.cmp(&a.1));
        for backup in backups.iter().skip(1) {
            let _ = tokio::fs::remove_file(&backup.0).await;
        }
    }

    Ok(())
}
