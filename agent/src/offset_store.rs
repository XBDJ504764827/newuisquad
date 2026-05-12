use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Offset 持久化状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOffset {
    pub log_path: PathBuf,
    pub offset: u64,
    pub updated_at: DateTime<Utc>,
}

/// Offset 存储管理器
#[derive(Clone)]
pub struct OffsetStore {
    offset_file_path: Option<PathBuf>,
    offset: u64,
    log_path: Option<PathBuf>,
}

impl OffsetStore {
    /// 创建新的 Offset 存储管理器
    pub fn new(offset_file_path: Option<String>) -> Self {
        Self {
            offset_file_path: offset_file_path.map(|p| PathBuf::from(p)),
            offset: 0,
            log_path: None,
        }
    }

    /// 加载持久化的 offset
    pub fn load(&mut self, current_log_path: &PathBuf) -> anyhow::Result<u64> {
        if let Some(ref offset_file) = self.offset_file_path {
            if offset_file.exists() {
                match fs::read_to_string(offset_file) {
                    Ok(content) => {
                        if let Ok(offset_state) = serde_json::from_str::<LogOffset>(&content) {
                            // 检查日志路径是否匹配
                            if offset_state.log_path == *current_log_path {
                                self.offset = offset_state.offset;
                                self.log_path = Some(offset_state.log_path);
                                tracing::info!("恢复 offset: {} @ 文件: {}", self.offset, offset_file.display());
                                return Ok(self.offset);
                            } else {
                                tracing::warn!("日志路径不匹配，重置 offset: {} vs {}",
                                    offset_state.log_path.display(),
                                    current_log_path.display());
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("解析 offset 文件失败: {}", e);
                    }
                }
            }
        }

        self.offset = 0;
        self.log_path = Some(current_log_path.clone());
        Ok(0)
    }

    /// 保存当前的 offset
    pub fn persist(&self) -> anyhow::Result<()> {
        if let (Some(ref log_path), Some(ref offset_file)) = (&self.log_path, &self.offset_file_path) {
            let state = LogOffset {
                log_path: log_path.clone(),
                offset: self.offset,
                updated_at: Utc::now(),
            };

            let content = serde_json::to_string_pretty(&state)?;

            // 原子写入
            let tmp_path = offset_file.with_extension("tmp");
            fs::write(&tmp_path, content)?;

            // 替换原文件
            if offset_file.exists() {
                fs::remove_file(offset_file)?;
            }
            fs::rename(&tmp_path, offset_file)?;
        }
        Ok(())
    }

    /// 更新 offset
    pub fn set_offset(&mut self, new_offset: u64) {
        self.offset = new_offset;
    }

    /// 获取当前 offset
    pub fn get_offset(&self) -> u64 {
        self.offset
    }
}
