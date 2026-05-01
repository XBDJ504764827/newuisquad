use axum::extract::{ws::{WebSocket, WebSocketUpgrade, Message}, Query, State};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};
use crate::models::server_log::LogEntry;
use sqlx::Row;
use crate::protocol::AgentMessage;
use crate::api::AppState;

#[derive(Deserialize)]
pub struct AgentQuery {
    pub token: String,
}

#[derive(Clone)]
pub struct AgentPool {
    agents: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<AgentMessage>>>>,
    log_tx: broadcast::Sender<LogEntry>,
    pending: Arc<RwLock<HashMap<String, oneshot::Sender<AgentMessage>>>>,
}

impl AgentPool {
    pub fn new() -> Self {
        let (log_tx, _) = broadcast::channel::<LogEntry>(1024);
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            log_tx,
            pending: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn log_tx(&self) -> broadcast::Sender<LogEntry> {
        self.log_tx.clone()
    }

    pub async fn send_and_wait(
        &self,
        server_id: &str,
        cmd: AgentMessage,
        request_id: &str,
    ) -> Result<AgentMessage, String> {
        let agents = self.agents.read().await;
        let tx = agents.get(server_id).ok_or("Agent 未连接")?;
        let (resp_tx, resp_rx) = oneshot::channel();
        self.pending
            .write()
            .await
            .insert(request_id.to_string(), resp_tx);
        tx.send(cmd).map_err(|e| format!("发送失败: {}", e))?;
        tokio::time::timeout(std::time::Duration::from_secs(10), resp_rx)
            .await
            .map_err(|_| "响应超时".to_string())?
            .map_err(|_| "Agent 断开".to_string())
    }
}

pub async fn handler(
    State(state): State<AppState>,
    Query(q): Query<AgentQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let token = q.token.clone();
    let agent_pool = state.agent_pool.clone().unwrap();

    // 验证 token
    let result = sqlx::query("SELECT id FROM servers WHERE token = $1")
        .bind(&token)
        .fetch_optional(&pool)
        .await;

    match result {
        Ok(Some(row)) => {
            let server_id: i32 = row.get(0);
            ws.on_upgrade(move |socket| handle_socket(socket, pool, agent_pool, server_id.to_string()))
        }
        _ => {
            ws.on_upgrade(|mut socket| async move {
                let _ = socket.send(Message::Text("{\"error\":\"无效的 token\"}".into())).await;
                let _ = socket.close().await;
            })
        }
    }
}

async fn handle_socket(
    socket: WebSocket,
    db_pool: sqlx::PgPool,
    agent_pool: AgentPool,
    server_id: String,
) {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<AgentMessage>();
    agent_pool
        .agents
        .write()
        .await
        .insert(server_id.clone(), cmd_tx);
    tracing::info!("Agent 已连接: {}", server_id);

    let (mut ws_sender, mut ws_receiver) = socket.split();

    let pool = db_pool.clone();
    let log_tx = agent_pool.log_tx();
    let pending = agent_pool.pending.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Ok(text) = msg.to_text() {
                if let Ok(agent_msg) = serde_json::from_str::<AgentMessage>(text) {
                    match &agent_msg {
                        AgentMessage::Log { data } => {
                            let entry = LogEntry {
                                log_level: data.log_level.clone(),
                                category: data.category.clone(),
                                message: data.message.clone(),
                                raw_line: data.raw_line.clone(),
                                logged_at: data.logged_at,
                            };
                            let _ = log_tx.send(entry.clone());
                            let _ = crate::repositories::server_log_repo::insert_log_entry(
                                &pool, 1, &entry,
                            )
                            .await;
                        }
                        AgentMessage::FileReadResult { request_id, .. }
                        | AgentMessage::FileWriteResult { request_id, .. }
                        | AgentMessage::FileListResult { request_id, .. } => {
                            let rid = request_id.clone();
                            if let Some(tx) = pending.write().await.remove(&rid) {
                                let _ = tx.send(agent_msg.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            let json = serde_json::to_string(&cmd).unwrap();
            if ws_sender
                .send(Message::Text(json.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut recv_task => {}
        _ = &mut send_task => {}
    }

    agent_pool.agents.write().await.remove(&server_id);
    tracing::info!("Agent 已断开: {}", server_id);
}
