use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::connect_async;
use crate::protocol::AgentMessage;

pub async fn run(
    ws_url: String,
    msg_tx: mpsc::UnboundedSender<AgentMessage>,
    msg_rx: Arc<Mutex<mpsc::UnboundedReceiver<AgentMessage>>>,
) {
    loop {
        tracing::info!("连接后端: {}", ws_url);

        match connect_async(ws_url.as_str()).await {
            Ok((ws_stream, resp)) => {
                tracing::info!("WebSocket 已连接 (状态: {:?})", resp.status());
                let (mut write, mut read) = ws_stream.split();

                let send_tx = msg_tx.clone();
                let recv_handle = tokio::spawn(async move {
                    let mut msg_count = 0u64;
                    while let Some(Ok(msg)) = read.next().await {
                        if let Ok(text) = msg.to_text() {
                            if let Ok(cmd) = serde_json::from_str::<AgentMessage>(text) {
                                msg_count += 1;
                                tracing::debug!("收到后端命令 #{}: {:?}", msg_count, std::mem::discriminant(&cmd));
                                if send_tx.send(cmd).is_err() {
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
                    }
                    tracing::info!("WebSocket 读端关闭，共接收 {} 条命令", msg_count);
                });

                let mut locked_rx = msg_rx.lock().await;
                let mut sent_count = 0u64;
                loop {
                    tokio::select! {
                        msg = locked_rx.recv() => {
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
                                    if write.send(tokio_tungstenite::tungstenite::Message::Text(json.into())).await.is_err() {
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
                    }
                }
                drop(locked_rx);
                recv_handle.abort();
                tracing::warn!("WebSocket 连接断开（发送 {} 条），5秒后重连...", sent_count);
            }
            Err(e) => {
                tracing::error!("连接失败: {}，5秒后重连...", e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
