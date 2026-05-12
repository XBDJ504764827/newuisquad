use std::time::{Duration, Instant};
use tokio::sync::{Mutex, mpsc};
use serde::{Deserialize, Serialize};

/// 事件批次上传器
pub struct BatchUploader {
    batch: Mutex<Vec<SerializedEvent>>,
    max_size: usize,
    max_delay_ms: u64,
    last_flush: Mutex<Instant>,
    event_tx: mpsc::Sender<SerializedEvent>,
    compression_enabled: bool,
    compression_level: i32,
    compression_min_bytes: usize,
}

/// 序列化后的事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEvent {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: i64,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_log: Option<String>,
}

impl BatchUploader {
    pub fn new(
        max_size: usize,
        max_delay_ms: u64,
        event_tx: mpsc::Sender<SerializedEvent>,
        compression_enabled: bool,
        compression_level: i32,
        compression_min_bytes: usize,
    ) -> Self {
        Self {
            batch: Mutex::new(Vec::with_capacity(max_size)),
            max_size,
            max_delay_ms,
            last_flush: Mutex::new(Instant::now()),
            event_tx,
            compression_enabled,
            compression_level,
            compression_min_bytes,
        }
    }

    /// 添加事件到批次中，返回是否已立即发送
    pub async fn add_event(&self, event: SerializedEvent) -> anyhow::Result<bool> {
        let mut batch = self.batch.lock().await;

        // 检查单个大事件是否需要压缩单独发送
        if self.compression_enabled {
            let estimated_size = self.estimate_event_size(&event);
            if estimated_size >= self.compression_min_bytes && !batch.is_empty() {
                let events: Vec<SerializedEvent> = std::mem::take(&mut *batch);
                drop(batch);
                self.flush_batch(&events).await?;
                // 大事件单独发送
                self.event_tx.send(event).await?;
                return Ok(true);
            }
        }

        batch.push(event.clone());

        // 达到批次大小上限时刷新
        if batch.len() >= self.max_size {
            let events: Vec<SerializedEvent> = std::mem::take(&mut *batch);
            drop(batch);
            self.flush_batch(&events).await?;
            return Ok(true);
        }

        Ok(false)
    }

    /// 刷新批次：逐个发送事件到 channel
    async fn flush_batch(&self, events: &[SerializedEvent]) -> anyhow::Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        tracing::debug!("刷新批次: {} 个事件", events.len());

        for event in events {
            self.event_tx.send(event.clone()).await?;
        }

        let mut last_flush = self.last_flush.lock().await;
        *last_flush = Instant::now();

        Ok(())
    }

    /// 启动定时刷新任务
    pub async fn start_flush_task(&self) {
        let interval = Duration::from_millis(self.max_delay_ms);
        let mut ticker = tokio::time::interval(interval);

        loop {
            ticker.tick().await;

            let last_flush = self.last_flush.lock().await;
            if last_flush.elapsed() >= interval {
                drop(last_flush);
                let mut batch = self.batch.lock().await;
                if !batch.is_empty() {
                    let events: Vec<SerializedEvent> = std::mem::take(&mut *batch);
                    drop(batch);
                    if let Err(e) = self.flush_batch(&events).await {
                        tracing::error!("定时刷新批次失败: {}", e);
                    }
                }
            }
        }
    }

    fn estimate_event_size(&self, event: &SerializedEvent) -> usize {
        serde_json::to_vec(event).map(|v| v.len()).unwrap_or(0)
    }
}
