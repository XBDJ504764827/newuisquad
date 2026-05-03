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
        let parsed_addr = match addr.parse() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("[RCON] 地址解析失败 {}: {}", addr, e);
                return;
            }
        };

        eprintln!("[RCON] SquadJS 兼容模式 - 连接 {} (长连接)...", addr);

        loop {
            let mut stream = match TcpStream::connect_timeout(&parsed_addr, Duration::from_secs(10)) {
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

            // 认证 (type=3 SERVERDATA_AUTH)
            let auth = build_packet(3, 0, &password);
            if stream.write_all(&auth).is_err() {
                eprintln!("[RCON] 认证发送失败");
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }

            // 读取认证响应
            let mut resp_buf = [0u8; 4096];
            let auth_ok = match stream.read(&mut resp_buf) {
                Ok(n) => {
                    if let Some(body) = parse_packet_body(&resp_buf[..n]) {
                        // 认证失败时服务器返回 ID=-1
                        !body.is_empty() || n > 14
                    } else {
                        true // 假设成功
                    }
                }
                Err(_) => false,
            };

            if !auth_ok {
                eprintln!("[RCON] 认证失败，5秒后重试");
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }

            eprintln!("[RCON] 已连接并认证成功，开始监听事件...");

            let mut buf = [0u8; 65536];
            let mut partial: Vec<u8> = Vec::new();

            loop {
                match stream.read(&mut buf) {
                    Ok(0) => {
                        eprintln!("[RCON] 连接关闭，5秒后重连");
                        break;
                    }
                    Ok(n) => {
                        partial.extend_from_slice(&buf[..n]);

                        // 逐个提取完整的 RCON 包（基于二进制长度前缀）
                        while let Some((packet, rest)) = extract_packet(&partial) {
                            partial = rest;

                            if let Some(body) = parse_packet_body(&packet) {
                                if body.is_empty() { continue; }

                                // 处理包体中的每一行事件
                                for line in body.lines() {
                                    let line = line.trim();
                                    if line.is_empty() { continue; }

                                    // SquadJS 聊天格式
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
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                        // 超时发送心跳
                        let ping = build_packet(2, 0, "ping");
                        let _ = stream.write_all(&ping);
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

/// 从二进制缓冲区提取完整 RCON 包
/// 返回 (包数据, 剩余数据)
fn extract_packet(data: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    if data.len() < 12 { return None; }

    let size = i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    // RCON size 至少为 10 (id+type+2 null bytes)
    if size < 10 { return None; }

    let total = 4 + size;
    if data.len() >= total {
        let packet = data[..total].to_vec();
        let rest = data[total..].to_vec();
        Some((packet, rest))
    } else {
        None
    }
}

/// 从二进制 RCON 包中提取文本 body
/// 包结构: [4字节size][4字节id][4字节type][body字符串\0][\0]
fn parse_packet_body(packet: &[u8]) -> Option<String> {
    if packet.len() < 14 { return None; }
    // body 从第12字节开始，到末尾2字节之前
    let body_end = packet.len().saturating_sub(2);
    if body_end <= 12 { return None; }
    let body_bytes = &packet[12..body_end];
    // 去除尾部 null 字节
    let body_bytes = match body_bytes.iter().rposition(|&b| b != 0) {
        Some(pos) => &body_bytes[..=pos],
        None => return None,
    };
    String::from_utf8(body_bytes.to_vec()).ok()
}

/// 构建 Source RCON 协议包
fn build_packet(packet_type: i32, packet_id: i32, body: &str) -> Vec<u8> {
    let body_bytes = body.as_bytes();
    // size = id(4) + type(4) + body + null(1) + null(1)
    let size = (10 + body_bytes.len()) as i32;
    let mut packet = Vec::with_capacity(4 + size as usize);
    packet.extend_from_slice(&size.to_le_bytes());
    packet.extend_from_slice(&packet_id.to_le_bytes());
    packet.extend_from_slice(&packet_type.to_le_bytes());
    packet.extend_from_slice(body_bytes);
    packet.push(0x00);
    packet.push(0x00);
    packet
}
