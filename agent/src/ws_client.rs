use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::http::Request;
use crate::protocol::AgentMessage;

pub async fn run(
    ws_url: String,
    token: String,
    msg_tx: mpsc::UnboundedSender<AgentMessage>,
    msg_rx: Arc<Mutex<mpsc::UnboundedReceiver<AgentMessage>>>,
) {
    loop {
        tracing::info!("连接后端: {}", ws_url);

        let request = Request::builder()
            .uri(&ws_url)
            .header("X-Auth-Token", &token)
            .body(())
            .expect("构建 WebSocket 请求失败");

        match connect_async(request).await {
            Ok((ws_stream, _)) => {
                tracing::info!("已连接至后端");
                let (mut write, mut read) = ws_stream.split();

                let send_tx = msg_tx.clone();
                let recv_handle = tokio::spawn(async move {
                    while let Some(Ok(msg)) = read.next().await {
                        if let Ok(text) = msg.to_text() {
                            if let Ok(cmd) = serde_json::from_str::<AgentMessage>(text) {
                                let _ = send_tx.send(cmd);
                            }
                        }
                    }
                });

                let mut locked_rx = msg_rx.lock().await;
                loop {
                    tokio::select! {
                        msg = locked_rx.recv() => {
                            match msg {
                                Some(msg) => {
                                    let json = serde_json::to_string(&msg).unwrap();
                                    if write.send(tokio_tungstenite::tungstenite::Message::Text(json.into())).await.is_err() {
                                        break;
                                    }
                                }
                                None => break,
                            }
                        }
                    }
                }
                drop(locked_rx);
                recv_handle.abort();
                tracing::warn!("WebSocket 连接断开，5秒后重连...");
            }
            Err(e) => {
                tracing::error!("连接失败: {}，5秒后重连...", e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
