use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use crate::protocol::{AgentMessage, LogEntry, PlayerInfo, SquadInfo, TeamName};

pub fn start_rcon_listener(
    host: String, port: u16, password: String,
    msg_tx: mpsc::UnboundedSender<AgentMessage>,
    auto_broadcast_secs: u64, auto_broadcast_msg: String, welcome_msg: String,
) {
    std::thread::spawn(move || {
        let addr = format!("{}:{}", host, port);
        let parsed = match addr.parse() { Ok(a) => a, Err(_) => { eprintln!("[RCON] 地址错误: {}", addr); return; } };
        let mut backoff = 1u64;
        let mut last_bc = Instant::now();

        loop {
            let mut s = match TcpStream::connect_timeout(&parsed, Duration::from_secs(10)) {
                Ok(s) => { s.set_read_timeout(Some(Duration::from_secs(60))).ok(); s.set_write_timeout(Some(Duration::from_secs(10))).ok(); s }
                Err(_) => { std::thread::sleep(Duration::from_secs(backoff)); backoff = (backoff*2).min(60); continue; }
            };

            // 认证
            if s.write_all(&build(3, 0, &password)).is_err() { sleep(5); continue; }
            let mut rb = [0u8; 4096];
            if s.read(&mut rb).map_or(true, |n| n < 14 || i32::from_le_bytes([rb[4],rb[5],rb[6],rb[7]]) == -1) {
                sleep(5); continue;
            }

            backoff = 1;
            eprintln!("[RCON] 已连接");
            let mut buf = [0u8; 65536];
            let mut partial: Vec<u8> = Vec::new();
            let mut last_query = Instant::now();
            // 累积状态查询结果
            let mut players_raw = String::new();
            let mut squads_raw = String::new();
            let mut map_raw = String::new();

            loop {
                // 每 5 秒查询状态
                if last_query.elapsed().as_secs() >= 5 {
                    players_raw.clear(); squads_raw.clear(); map_raw.clear();
                    let _ = s.write_all(&build(2, 50, "ListPlayers"));
                    let _ = s.write_all(&build(2, 51, "ListSquads"));
                    let _ = s.write_all(&build(2, 52, "ShowNextMap"));
                    last_query = Instant::now();
                }

                // 定时广播
                if auto_broadcast_secs > 0 && !auto_broadcast_msg.is_empty() && last_bc.elapsed().as_secs() >= auto_broadcast_secs {
                    let _ = s.write_all(&build(2, 0, &format!("AdminBroadcast \"{}\"", auto_broadcast_msg)));
                    last_bc = Instant::now();
                }

                match s.read(&mut buf) {
                    Ok(0) => { eprintln!("[RCON] 断开"); break; }
                    Ok(n) => {
                        partial.extend_from_slice(&buf[..n]);
                        while let Some((pkt, rest)) = extract(&partial) {
                            partial = rest;
                            let Some(body) = body_str(&pkt) else { continue };
                            if body.is_empty() { continue; }

                            // ===== 状态查询响应（按内容前缀识别） =====
                            if body.starts_with("----- Active Players -----") {
                                players_raw = read_multi(&body, &mut partial);
                                eprintln!("[RCON] 玩家数据: {} 字节", players_raw.len());
                                continue;
                            }
                            if body.starts_with("----- Active Squads -----") {
                                squads_raw = read_multi(&body, &mut partial);
                                eprintln!("[RCON] 小队数据: {} 字节, 前100: {:?}", squads_raw.len(), &squads_raw[..squads_raw.len().min(100)]);
                                continue;
                            }
                            if body.contains("Current map is ") || body.contains("Next map is ") {
                                map_raw = body.clone();
                                eprintln!("[RCON] 地图数据: {:?}", &map_raw[..map_raw.len().min(100)]);
                            }

                            // ===== 聚合状态报告 =====
                            if !players_raw.is_empty() && !squads_raw.is_empty() {
                                let players = parse_players(&players_raw);
                                let squads = parse_squads(&squads_raw);
                                let team_names = parse_team_names(&squads_raw);
                                let map = map_raw.lines().next().unwrap_or("").to_string();
                                eprintln!("[RCON] 上报: {}人 {}队 阵营{:?} 地图{}", players.len(), squads.len(), team_names.iter().map(|t|&t.faction).collect::<Vec<_>>(), map);
                                if !players.is_empty() {
                                    let _ = msg_tx.send(AgentMessage::ServerStateReport {
                                        players, squads, team_names,
                                        map_name: map, game_mode: String::new(),
                                        server_name: String::new(), player_count: 0, max_players: 0,
                                        next_map: String::new(),
                                    });
                                }
                                players_raw.clear(); squads_raw.clear(); map_raw.clear();
                                continue;
                            }

                            // ===== 聊天 =====
                            if let Some(ev) = chat_event(&body) {
                                let _ = msg_tx.send(AgentMessage::Log { data: LogEntry {
                                    log_level: "INFO".into(), category: Some(format!("Chat-{}", ev.channel)),
                                    message: format!("{}: {}", ev.player, ev.msg),
                                    raw_line: Some(body), logged_at: chrono::Utc::now(),
                                }});
                                continue;
                            }

                            // ===== 玩家加入 =====
                            if body.contains("PlayerConnected") || body.contains("joined the server") {
                                if !welcome_msg.is_empty() {
                                    let name = body.split(':').last().map(|s| s.trim()).unwrap_or("");
                                    let _ = s.write_all(&build(2, 0, &format!("AdminBroadcast \"{}\"", welcome_msg.replace("{player}", name))));
                                }
                                let _ = msg_tx.send(AgentMessage::Log { data: LogEntry {
                                    log_level: "INFO".into(), category: Some("PlayerJoin".into()),
                                    message: body.clone(), raw_line: Some(body), logged_at: chrono::Utc::now(),
                                }});
                                continue;
                            }

                            // ===== 玩家离开 =====
                            if body.contains("PlayerDisconnected") || body.contains("left the server") {
                                let _ = msg_tx.send(AgentMessage::Log { data: LogEntry {
                                    log_level: "INFO".into(), category: Some("PlayerLeave".into()),
                                    message: body.clone(), raw_line: Some(body), logged_at: chrono::Utc::now(),
                                }});
                                continue;
                            }

                            // ===== 管理员镜头 =====
                            if body.contains("POSSESSED_ADMIN_CAMERA") || body.contains("UNPOSSESSED_ADMIN_CAMERA") {
                                let _ = msg_tx.send(AgentMessage::Log { data: LogEntry {
                                    log_level: "INFO".into(), category: Some("FlyEvent".into()),
                                    message: body.clone(), raw_line: Some(body), logged_at: chrono::Utc::now(),
                                }});
                                continue;
                            }

                            // ===== 管理员操作 =====
                            if body.contains("PLAYER_WARNED") || body.contains("PLAYER_KICKED") || body.contains("PLAYER_BANNED") {
                                let _ = msg_tx.send(AgentMessage::Log { data: LogEntry {
                                    log_level: "WARN".into(), category: Some("AdminAction".into()),
                                    message: body.clone(), raw_line: Some(body), logged_at: chrono::Utc::now(),
                                }});
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(_) => { eprintln!("[RCON] 读错误"); break; }
                }
            }
            sleep(backoff); backoff = (backoff*2).min(60);
        }
    });
}

// === 读取多包（空包终止） ===
fn read_multi(first: &str, partial: &mut Vec<u8>) -> String {
    let mut out = first.to_string();
    while let Some((p2, r2)) = extract(partial) {
        *partial = r2;
        if let Some(b2) = body_str(&p2) {
            if b2.is_empty() { break; }
            out.push_str(&b2);
        } else { break; }
    }
    out
}

// === RCON 协议 ===
fn build(t: i32, id: i32, body: &str) -> Vec<u8> {
    let b = body.as_bytes();
    let size = (10 + b.len()) as i32;
    let mut p = Vec::with_capacity(4 + size as usize);
    p.extend_from_slice(&size.to_le_bytes());
    p.extend_from_slice(&id.to_le_bytes());
    p.extend_from_slice(&t.to_le_bytes());
    p.extend_from_slice(b);
    p.push(0x00); p.push(0x00);
    p
}
fn extract(data: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    if data.len() < 12 { return None; }
    let size = i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if size < 10 { return None; }
    let total = 4 + size;
    if data.len() >= total { Some((data[..total].to_vec(), data[total..].to_vec())) } else { None }
}
fn body_str(pkt: &[u8]) -> Option<String> {
    if pkt.len() < 14 { return None; }
    let b = &pkt[12..pkt.len().saturating_sub(2)];
    let pos = b.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);
    String::from_utf8(b[..pos].to_vec()).ok()
}
fn sleep(s: u64) { std::thread::sleep(Duration::from_secs(s)); }

// === 聊天 ===
struct Chat { channel: String, player: String, msg: String }
fn chat_event(raw: &str) -> Option<Chat> {
    for line in raw.lines() {
        let line = line.trim();
        let (ch, rest) = if line.starts_with("[ChatAll]") { ("All", &line[9..]) }
        else if line.starts_with("[ChatTeam]") { ("Team", &line[11..]) }
        else if line.starts_with("[ChatSquad]") { ("Squad", &line[12..]) }
        else if line.starts_with("[ChatAdmin]") { ("Admin", &line[12..]) }
        else { continue };
        let rest = rest.trim();
        let after = if rest.starts_with("[Online IDs:") { rest.split(']').nth(1).unwrap_or(rest).trim() } else { rest };
        if let Some(pos) = after.find(" : ") {
            return Some(Chat { channel: ch.into(), player: after[..pos].trim().into(), msg: after[pos+3..].trim().into() });
        }
    }
    None
}

// === 解析 ===
fn parse_players(raw: &str) -> Vec<PlayerInfo> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("-----") { continue; }
        let mut name=String::new(); let mut steam=String::new(); let mut tid=0i32; let mut sq=None; let mut role=String::new();
        let mut k=0i32; let mut d=0i32; let mut sc=0i32; let mut ping=0i32; let mut admin=false;
        for part in line.split('|') {
            let v = part.trim();
            if let Some(x)=v.strip_prefix("Name: ") { name=x.to_string(); }
            else if let Some(x)=v.strip_prefix("Online IDs: ") {
                // "Online IDs: EOS: xxx steam: 7656xxx" → 提取 steam ID
                if let Some(pos)=x.find("steam: ") { steam = x[pos+7..].split_whitespace().next().unwrap_or("").to_string(); }
            }
            else if let Some(x)=v.strip_prefix("SteamID: ") { steam=x.to_string(); }
            else if let Some(x)=v.strip_prefix("Team ID: ") { tid=x.parse().unwrap_or(0); }
            else if let Some(x)=v.strip_prefix("Squad ID: ") { let x=x.trim(); if x!="N/A"&&!x.is_empty() { sq=Some(x.to_string()); } }
            else if let Some(x)=v.strip_prefix("Role: ") { role=x.to_string(); }
            else if let Some(x)=v.strip_prefix("Kills: ") { k=x.parse().unwrap_or(0); }
            else if let Some(x)=v.strip_prefix("Deaths: ") { d=x.parse().unwrap_or(0); }
            else if let Some(x)=v.strip_prefix("Score: ") { sc=x.parse().unwrap_or(0); }
            else if let Some(x)=v.strip_prefix("Ping: ") { ping=x.parse().unwrap_or(0); }
            else if let Some(x)=v.strip_prefix("Admin: ") { admin=x.trim()=="True"; }
            else if let Some(_)=v.strip_prefix("Is Leader: ") {}
        }
        if !name.is_empty() { out.push(PlayerInfo{name,steam_id:steam,team_id:tid,squad_id:sq,role,kills:k,deaths:d,score:sc,ping,is_admin:admin}); }
    }
    out
}
fn parse_squads(raw: &str) -> Vec<SquadInfo> {
    let mut out=Vec::new(); let mut ct=0i32;
    for line in raw.lines() {
        let line=line.trim();
        if line.is_empty()||line.starts_with("-----") { continue; }
        if line.to_lowercase().starts_with("team id:") { ct=line.split_whitespace().nth(2).and_then(|s|s.parse().ok()).unwrap_or(0); continue; }
        if line.to_lowercase().starts_with("squad ") {
            if let Some((_,rest))=line.split_once(": ") {
                let (name,creator)=rest.split_once(" - ").map_or((rest.to_string(),String::new()),|(a,b)|(a.trim().to_string(),b.trim().to_string()));
                out.push(SquadInfo{name,creator,team_id:ct});
            }
        }
    }
    out
}
fn parse_team_names(raw: &str) -> Vec<TeamName> {
    let mut out=Vec::new();
    for line in raw.lines() {
        let line=line.trim();
        if line.to_lowercase().starts_with("team id:") {
            let after=line.split(':').nth(1).unwrap_or("").trim();
            if let Some(p)=after.find('(') {
                if let Ok(id)=after[..p].trim().parse() {
                    out.push(TeamName{team_id:id,faction:after[p+1..].trim_end_matches(')').to_string()});
                }
            }
        }
    }
    out
}
