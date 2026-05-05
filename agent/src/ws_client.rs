use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use crate::protocol::AgentMessage;

const PING_INTERVAL_SECS: u64 = 30;
const INITIAL_RECONNECT_DELAY_SECS: u64 = 1;
const MAX_RECONNECT_DELAY_SECS: u64 = 60;

pub async fn run(
    ws_url: String,
    msg_tx: mpsc::Sender<AgentMessage>,
    msg_rx: Arc<Mutex<mpsc::Receiver<AgentMessage>>>,
) {
    let mut reconnect_delay = INITIAL_RECONNECT_DELAY_SECS;
    loop {
        tracing::info!("连接后端: {}", ws_url);

        match connect_async(ws_url.as_str()).await {
            Ok((ws_stream, resp)) => {
                tracing::info!("WebSocket 已连接 (状态: {:?})", resp.status());
                reconnect_delay = INITIAL_RECONNECT_DELAY_SECS;
                let (mut write, mut read) = ws_stream.split();

                let send_tx = msg_tx.clone();
                let recv_handle = tokio::spawn(async move {
                    let mut msg_count = 0u64;
                    while let Some(Ok(msg)) = read.next().await {
                        match msg {
                            Message::Text(text) => {
                                if let Ok(cmd) = serde_json::from_str::<AgentMessage>(&text) {
                                    msg_count += 1;
                                    tracing::debug!("收到后端命令 #{}: {:?}", msg_count, std::mem::discriminant(&cmd));
                                    if send_tx.send(cmd).await.is_err() {
                                        tracing::error!("命令转发失败（通道已关闭）");
                                        break;
                                    }
                                } else if !text.is_empty() {
                                    tracing::warn!("收到非协议消息: {}", text);
                                    if text.contains("error") || text.contains("无效") {
                                        tracing::error!("后端返回错误，请检查 TOKEN 是否正确");
                                    }
                                }
                            }
                            Message::Ping(data) => {
                                // tungstenite 自动回复 pong，无需手动处理
                                tracing::debug!("收到 Ping ({} bytes)", data.len());
                            }
                            Message::Pong(_) => {
                                tracing::debug!("收到 Pong");
                            }
                            Message::Close(_) => {
                                tracing::info!("收到关闭帧");
                                break;
                            }
                            _ => {}
                        }
                    }
                    tracing::info!("WebSocket 读端关闭，共接收 {} 条命令", msg_count);
                });

                let mut sent_count = 0u64;
                loop {
                    tokio::select! {
                        // 从通道接收消息发送到 WebSocket
                        msg = async {
                            let mut rx = msg_rx.lock().await;
                            rx.recv().await
                        } => {
                            match msg {
                                Some(msg) => {
                                    let json = match serde_json::to_string(&msg) {
                                        Ok(s) => s,
                                        Err(e) => {
                                            tracing::error!("消息序列化失败: {}", e);
                                            continue;
                                        }
                                    };
                                    sent_count += 1;
                                    if write.send(Message::Text(json.into())).await.is_err() {
                                        tracing::error!("WebSocket 发送失败（共发送 {} 条）", sent_count);
                                        break;
                                    }
                                }
                                None => {
                                    tracing::info!("发送通道关闭");
                                    break;
                                }
                            }
                        }
                        // 心跳：每 30 秒发送 Ping
                        _ = tokio::time::sleep(Duration::from_secs(PING_INTERVAL_SECS)) => {
                            if write.send(Message::Ping(tokio_tungstenite::tungstenite::Bytes::new())).await.is_err() {
                                tracing::warn!("Ping 发送失败，连接可能已断开");
                                break;
                            }
                            tracing::debug!("已发送 Ping");
                        }
                    }
                }
                recv_handle.abort();
                tracing::warn!("WebSocket 连接断开（发送 {} 条），{} 秒后重连...", sent_count, reconnect_delay);
            }
            Err(e) => {
                tracing::error!("连接失败: {}，{} 秒后重连...", e, reconnect_delay);
            }
        }
        tokio::time::sleep(Duration::from_secs(reconnect_delay)).await;
        reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY_SECS);
    }
}
