use sqlx::PgPool;
use tokio::sync::mpsc;
use crate::models::server_log::LogEntry;

const BATCH_SIZE: usize = 100;
const FLUSH_INTERVAL_MS: u64 = 1000;

#[derive(Clone)]
pub struct LogBatcher {
    tx: mpsc::UnboundedSender<(i32, LogEntry)>,
}

impl LogBatcher {
    pub fn new(pool: PgPool) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<(i32, LogEntry)>();
        tokio::spawn(async move {
            let mut batch: Vec<(i32, LogEntry)> = Vec::with_capacity(BATCH_SIZE);
            loop {
                match tokio::time::timeout(
                    std::time::Duration::from_millis(FLUSH_INTERVAL_MS),
                    rx.recv(),
                )
                .await
                {
                    Ok(Some(entry)) => {
                        batch.push(entry);
                        // 继续排空 channel 中已有的消息
                        while let Ok(entry) = rx.try_recv() {
                            batch.push(entry);
                        }
                        if batch.len() >= BATCH_SIZE {
                            flush_batch(&pool, &mut batch).await;
                        }
                    }
                    Ok(None) => {
                        // channel 关闭，刷出剩余数据
                        if !batch.is_empty() {
                            flush_batch(&pool, &mut batch).await;
                        }
                        break;
                    }
                    Err(_) => {
                        // 超时，刷出当前批次
                        if !batch.is_empty() {
                            flush_batch(&pool, &mut batch).await;
                        }
                    }
                }
            }
        });
        Self { tx }
    }

    pub fn send(&self, server_id: i32, entry: LogEntry) {
        let _ = self.tx.send((server_id, entry));
    }
}

async fn flush_batch(pool: &PgPool, batch: &mut Vec<(i32, LogEntry)>) {
    if batch.is_empty() {
        return;
    }
    let entries = std::mem::take(batch);
    batch.clear();

    // 构建批量 INSERT: INSERT INTO server_logs (...) VALUES ($1,$2,...), ($N,$N+1,...), ...
    let mut query = String::from(
        "INSERT INTO server_logs (server_id, log_level, category, message, raw_line, logged_at) VALUES ",
    );
    let mut params: Vec<String> = Vec::new();
    for (i, (sid, entry)) in entries.iter().enumerate() {
        if i > 0 {
            query.push(',');
        }
        let base = i * 6;
        query.push_str(&format!(
            "(${},${},${},${},${},${})",
            base + 1,
            base + 2,
            base + 3,
            base + 4,
            base + 5,
            base + 6
        ));
        params.push(sid.to_string());
        params.push(entry.log_level.clone());
        params.push(entry.category.clone().unwrap_or_default());
        params.push(entry.message.clone());
        params.push(entry.raw_line.clone().unwrap_or_default());
        params.push(entry.logged_at.to_rfc3339());
    }

    // 使用原始 SQL 执行批量插入
    let mut db_query = sqlx::query(&query);
    for p in &params {
        db_query = db_query.bind(p);
    }
    if let Err(e) = db_query.execute(pool).await {
        tracing::error!(count = entries.len(), error = %e, "批量写入日志失败");
    } else {
        tracing::debug!(count = entries.len(), "批量写入日志成功");
    }
}
