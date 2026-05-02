use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tokio::sync::mpsc;
use crate::protocol::{AgentMessage, LogEntry};

/// RCON 长连接监听 — SquadJS 兼容模式
/// 保持连接持续读取服务器推送的事件（聊天、管理员镜头等）
pub fn start_rcon_listener(
    host: String,
    port: u16,
    password: String,
    msg_tx: mpsc::UnboundedSender<AgentMessage>,
) {
    std::thread::spawn(move || {
        let addr = format!("{}:{}", host, port);
        eprintln!("[RCON] SquadJS 兼容模式 - 连接 {} (长连接)...", addr);

        // 持续重连
        loop {
            let mut stream = match TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(10)) {
                Ok(s) => {
                    let _ = s.set_read_timeout(Some(Duration::from_secs(300)));
                    s
                }
                Err(e) => {
                    eprintln!("[RCON] 连接失败: {}，5秒后重试", e);
                    std::thread::sleep(Duration::from_secs(5));
                    continue;
                }
            };

            // 认证
            let auth = build_packet(3, &password);
            if stream.write_all(&auth).is_err() {
                eprintln!("[RCON] 认证发送失败");
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }

            eprintln!("[RCON] 已连接，开始监听事件...");

            let mut buf = vec![0u8; 65536];
            let mut partial = String::new();

            loop {
                match stream.read(&mut buf) {
                    Ok(0) => {
                        eprintln!("[RCON] 连接关闭，5秒后重连");
                        break;
                    }
                    Ok(n) => {
                        let raw = String::from_utf8_lossy(&buf[..n]).to_string();
                        partial.push_str(&raw);

                        // 逐个处理多包数据
                        // Squad RCON 包格式: 4字节size + 4字节0 + 4字节type + body + 2字节0
                        while let Some((packet, rest)) = extract_packet(&partial) {
                            partial = rest;

                            // 跳过认证响应和命令响应（没有事件标记）
                            if packet.is_empty() || packet.len() < 10 { continue; }

                            // 查找并处理所有聊天事件
                            for line in packet.lines() {
                                let line = line.trim();
                                if line.is_empty() { continue; }

                                // SquadJS 聊天格式: [ChatAll] [Online IDs:...] Name : Message
                                if let Some(event) = parse_squadjs_chat(line) {
                                    let _ = msg_tx.send(AgentMessage::Log {
                                        data: LogEntry {
                                            log_level: "INFO".to_string(),
                                            category: Some(format!("Chat-{}", event.channel)),
                                            message: format!("{}: {}", event.player_name, event.message),
                                            raw_line: Some(line.to_string()),
                                            logged_at: chrono::Utc::now(),
                                        },
                                    });
                                    continue;
                                }

                                // 管理员镜头事件
                                if line.contains("POSSESSED_ADMIN_CAMERA") || line.contains("UNPOSSESSED_ADMIN_CAMERA") {
                                    let _ = msg_tx.send(AgentMessage::Log {
                                        data: LogEntry {
                                            log_level: "INFO".to_string(),
                                            category: Some("FlyEvent".to_string()),
                                            message: line.to_string(),
                                            raw_line: Some(line.to_string()),
                                            logged_at: chrono::Utc::now(),
                                        },
                                    });
                                    continue;
                                }

                                // 管理员操作事件
                                if line.contains("PLAYER_WARNED") || line.contains("PLAYER_KICKED") || line.contains("PLAYER_BANNED") {
                                    let _ = msg_tx.send(AgentMessage::Log {
                                        data: LogEntry {
                                            log_level: "WARN".to_string(),
                                            category: Some("AdminAction".to_string()),
                                            message: line.to_string(),
                                            raw_line: Some(line.to_string()),
                                            logged_at: chrono::Utc::now(),
                                        },
                                    });
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                        // 超时发送心跳保持连接
                        let ping = build_packet(2, "ping");
                        let _ = stream.write_all(&ping);
                        continue;
                    }
                    Err(e) => {
                        eprintln!("[RCON] 读取错误: {}", e);
                        break;
                    }
                }
            }
            std::thread::sleep(Duration::from_secs(5));
        }
    });
}

struct ChatEvent {
    channel: String,
    player_name: String,
    message: String,
}

/// 解析 SquadJS 兼容的 RCON 聊天格式:
/// [ChatAll] [Online IDs: EOS:xxx steam:xxx] PlayerName : Message
fn parse_squadjs_chat(line: &str) -> Option<ChatEvent> {
    // 匹配 [ChatAll|ChatTeam|ChatSquad|ChatAdmin]
    let chat_start = if line.starts_with("[ChatAll]") { ("All", &line[9..]) }
    else if line.starts_with("[ChatTeam]") { ("Team", &line[11..]) }
    else if line.starts_with("[ChatSquad]") { ("Squad", &line[12..]) }
    else if line.starts_with("[ChatAdmin]") { ("Admin", &line[12..]) }
    else { return None };

    let rest = chat_start.1.trim();

    // 跳过 [Online IDs:...] 部分
    let after_ids = if rest.starts_with("[Online IDs:") {
        if let Some(end) = rest.find(']') {
            &rest[end + 1..]
        } else { rest }
    } else { rest };

    let after_ids = after_ids.trim();

    // 格式: PlayerName : Message
    if let Some(colon_pos) = after_ids.find(" : ") {
        let player_name = after_ids[..colon_pos].trim().to_string();
        let message = after_ids[colon_pos + 3..].trim().to_string();
        if !player_name.is_empty() {
            return Some(ChatEvent { channel: chat_start.0.to_string(), player_name, message });
        }
    }

    // 也尝试旧格式: PlayerName (SteamID): message
    if let Some(paren_start) = after_ids.find('(') {
        let name = after_ids[..paren_start].trim().to_string();
        if let Some(paren_end) = after_ids[paren_start..].find(')') {
            let after = &after_ids[paren_start + paren_end + 1..];
            let msg = after.trim_start_matches(": ").trim().to_string();
            if !name.is_empty() {
                return Some(ChatEvent { channel: chat_start.0.to_string(), player_name: name, message: msg });
            }
        }
    }

    None
}

/// 从缓冲区中提取完整 RCON 包
fn extract_packet(data: &str) -> Option<(String, String)> {
    if data.len() < 12 { return None; }

    // 尝试找 RCON 包边界：4字节size LE
    let bytes = data.as_bytes();
    if bytes.len() < 4 { return None; }

    let size = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let total = 4 + size;
    if data.len() >= total {
        // 提取包体（跳过4字节size + 4字节padding + 4字节type = 12字节头部）
        let body_start = 12.min(total);
        let body = &data[body_start..total];
        let rest = if data.len() > total { data[total..].to_string() } else { String::new() };
        Some((body.trim_end_matches('\0').trim().to_string(), rest))
    } else {
        None
    }
}

fn build_packet(packet_type: i32, body: &str) -> Vec<u8> {
    let body_bytes = body.as_bytes();
    let size = (10 + body_bytes.len()) as i32;
    let mut packet = Vec::with_capacity(4 + size as usize);
    packet.extend_from_slice(&size.to_le_bytes());
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    packet.extend_from_slice(&packet_type.to_le_bytes());
    packet.extend_from_slice(body_bytes);
    packet.push(0x00);
    packet.push(0x00);
    packet
}
