use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tokio::sync::mpsc;
use crate::protocol::{AgentMessage, LogEntry};

/// RCON 主动轮询 + 被动监听（聊天、玩家列表等）
pub fn start_rcon_listener(
    host: String,
    port: u16,
    password: String,
    msg_tx: mpsc::UnboundedSender<AgentMessage>,
) {
    std::thread::spawn(move || {
        let addr = format!("{}:{}", host, port);
        eprintln!("[RCON] 开始主动轮询 {}...", addr);

        loop {
            // 每次轮询建新连接（避免长连接不稳定）
            let mut stream = match TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(5)) {
                Ok(s) => { let _ = s.set_read_timeout(Some(Duration::from_secs(10))); s }
                Err(e) => { eprintln!("[RCON] 连接失败: {}，2秒后重试", e); std::thread::sleep(Duration::from_secs(2)); continue; }
            };

            // 认证
            let auth = build_packet(3, &password);
            if stream.write_all(&auth).is_err() { std::thread::sleep(Duration::from_secs(2)); continue; }
            std::thread::sleep(Duration::from_millis(200));

            // 读取认证响应
            let _ = read_response(&mut stream);

            // 1. 查询玩家列表
            let cmd = build_packet(2, "ListPlayers");
            if stream.write_all(&cmd).is_ok() {
                std::thread::sleep(Duration::from_millis(500));
                if let Ok(resp) = read_response(&mut stream) {
                    for line in resp.lines() {
                        let line = line.trim();
                        if !line.is_empty() && !line.starts_with("-----") {
                            let _ = msg_tx.send(AgentMessage::Log {
                                data: LogEntry {
                                    log_level: "INFO".to_string(),
                                    category: Some("PlayerList".to_string()),
                                    message: format!("[Player] {}", line),
                                    raw_line: Some(line.to_string()),
                                    logged_at: chrono::Utc::now(),
                                },
                            });
                        }
                    }
                }
            }

            // 2. 尝试查询聊天（如果服务器版本支持）
            std::thread::sleep(Duration::from_millis(100));
            let chat_cmd = build_packet(2, "ChatList");
            if stream.write_all(&chat_cmd).is_ok() {
                std::thread::sleep(Duration::from_millis(300));
                if let Ok(resp) = read_response(&mut stream) {
                    let chat_lines: Vec<String> = resp.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty() && l.len() > 5).collect();
                    for line in &chat_lines {
                        if let Some((channel, name, _steam, msg)) = parse_chat_event(line) {
                            let _ = msg_tx.send(AgentMessage::Log {
                                data: LogEntry {
                                    log_level: "INFO".to_string(),
                                    category: Some(format!("Chat-{}", channel)),
                                    message: format!("{}: {}", name, msg),
                                    raw_line: Some(line.clone()),
                                    logged_at: chrono::Utc::now(),
                                },
                            });
                        }
                    }
                }
            }

            // 3. 查询服务器信息
            std::thread::sleep(Duration::from_millis(100));
            let info_cmd = build_packet(2, "ShowServerInfo");
            if stream.write_all(&info_cmd).is_ok() {
                std::thread::sleep(Duration::from_millis(300));
                if let Ok(resp) = read_response(&mut stream) {
                    let resp_clone = resp.clone();
                    let _ = msg_tx.send(AgentMessage::Log {
                        data: LogEntry {
                            log_level: "INFO".to_string(),
                            category: Some("ServerInfo".to_string()),
                            message: resp,
                            raw_line: Some(resp_clone),
                            logged_at: chrono::Utc::now(),
                        },
                    });
                }
            }

            // 等待 3 秒后再次轮询
            std::thread::sleep(Duration::from_secs(3));
        }
    });
}

fn parse_chat_event(line: &str) -> Option<(String, String, String, String)> {
    let (channel, rest) = if let Some(s) = line.strip_prefix("[ChatAll] ") { ("All", s.to_string()) }
    else if let Some(s) = line.strip_prefix("[ChatTeam] ") { ("Team", s.to_string()) }
    else if let Some(s) = line.strip_prefix("[ChatSquad] ") { ("Squad", s.to_string()) }
    else if let Some(s) = line.strip_prefix("[ChatAdmin] ") { ("Admin", s.to_string()) }
    else if line.contains("ChatAll:") || line.contains("ChatTeam:") {
        let ch = if line.contains("ChatAll") { "All" } else { "Team" };
        let rest = line.splitn(2, ": ").nth(1).unwrap_or("").to_string();
        (ch, rest)
    } else { return None; };

    if let Some(paren_start) = rest.find('(') {
        let name = rest[..paren_start].trim().to_string();
        if let Some(paren_end) = rest[paren_start..].find(')') {
            let steam = rest[paren_start + 1..paren_start + paren_end].to_string();
            let msg = rest[paren_start + paren_end + 1..].trim_start_matches(": ").trim().to_string();
            if steam.len() >= 10 && steam.chars().all(|c| c.is_ascii_digit()) {
                return Some((channel.to_string(), name, steam, msg));
            }
        }
    }
    None
}

fn read_response(stream: &mut TcpStream) -> Result<String, String> {
    let mut buf = vec![0u8; 16384];
    match stream.read(&mut buf) {
        Ok(n) if n > 0 => {
            // Squad RCON 响应格式：跳过 12 字节头部
            if n > 12 {
                Ok(String::from_utf8_lossy(&buf[12..n]).to_string())
            } else {
                Ok(String::from_utf8_lossy(&buf[..n]).to_string())
            }
        }
        Ok(_) => Ok(String::new()),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
            Ok(String::new())
        }
        Err(e) => Err(format!("{}", e)),
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
