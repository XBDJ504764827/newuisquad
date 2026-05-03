use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use crate::protocol::{AgentMessage, LogEntry, PlayerInfo, SquadInfo, TeamName};

pub fn start_rcon_listener(
    host: String, port: u16, password: String,
    msg_tx: mpsc::UnboundedSender<AgentMessage>,
    mut rcon_cmd_rx: mpsc::UnboundedReceiver<String>,
    auto_broadcast_secs: u64, auto_broadcast_msg: String, welcome_msg: String,
) {
    std::thread::spawn(move || {
        let addr = format!("{}:{}", host, port);
        let parsed = match addr.parse() { Ok(a) => a, Err(_) => { eprintln!("[RCON] 地址错误: {}", addr); return; } };
        let mut backoff = 1u64;
        let mut last_bc = Instant::now();

        loop {
            let mut s = match TcpStream::connect_timeout(&parsed, Duration::from_secs(10)) {
                Ok(s) => { s.set_read_timeout(Some(Duration::from_millis(200))).ok(); s.set_write_timeout(Some(Duration::from_secs(10))).ok(); s }
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
            let mut next_map = String::new();
            let mut server_name = String::new();
            let mut player_count = 0i32;
            let mut max_players = 0i32;
            let mut game_mode = String::new();

            loop {
                // 每 3 秒查询状态
                if last_query.elapsed().as_secs() >= 3 {
                    // 主动上报累积的查询结果（不依赖外来事件触发）
                    if !players_raw.is_empty() {
                        let players = parse_players(&players_raw);
                        let squads = parse_squads(&squads_raw);
                        let mut team_names = parse_team_names(&squads_raw);
                        if team_names.is_empty() {
                            let mut seen = std::collections::HashSet::new();
                            for p in &players {
                                if p.team_id != 0 && seen.insert(p.team_id) {
                                    team_names.push(TeamName { team_id: p.team_id, faction: format!("队伍 {}", p.team_id) });
                                }
                            }
                        }
                        let map = map_raw.lines().next().unwrap_or("").to_string();
                        let actual_player_count = player_count.max(players.len() as i32);
                        eprintln!("[RCON] 上报: {}人/{}队 阵营{:?} 地图{}", players.len(), squads.len(), team_names.iter().map(|t|&t.faction).collect::<Vec<_>>(), map);
                        if !players.is_empty() {
                            let _ = msg_tx.send(AgentMessage::ServerStateReport {
                                players, squads, team_names,
                                map_name: map, game_mode: game_mode.clone(),
                                server_name: server_name.clone(), player_count: actual_player_count, max_players,
                                next_map: next_map.clone(),
                            });
                        }
                    }
                    players_raw.clear(); squads_raw.clear(); map_raw.clear(); next_map.clear();
                    server_name.clear(); player_count = 0; max_players = 0; game_mode.clear();
                    let _ = s.write_all(&build(2, 50, "ListPlayers"));
                    let _ = s.write_all(&build(2, 51, "ListSquads"));
                    let _ = s.write_all(&build(2, 52, "ShowServerInfo"));
                    let _ = s.write_all(&build(2, 53, "ShowNextMap"));
                    let _ = s.write_all(&build(2, 54, "ShowCurrentMap"));
                    last_query = Instant::now();
                }

                // 定时广播
                if auto_broadcast_secs > 0 && !auto_broadcast_msg.is_empty() && last_bc.elapsed().as_secs() >= auto_broadcast_secs {
                    let _ = s.write_all(&build(2, 0, &format!("AdminBroadcast \"{}\"", auto_broadcast_msg)));
                    last_bc = Instant::now();
                }

                // 执行来自后端的 RCON 命令（如跳边、广播等）
                loop {
                    match rcon_cmd_rx.try_recv() {
                        Ok(cmd) => {
                            eprintln!("[RCON] 执行后端命令: {}", cmd);
                            let _ = s.write_all(&build(2, 200, &cmd));
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            eprintln!("[RCON] 后端命令通道已关闭");
                            break;
                        }
                    }
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
                                let id = pkt_id(&pkt).unwrap_or(0);
                                players_raw = read_multi(&body, &mut partial, id);
                                eprintln!("[RCON] 玩家数据: {} 字节", players_raw.len());
                                continue;
                            }
                            if body.starts_with("----- Active Squads -----") {
                                let id = pkt_id(&pkt).unwrap_or(0);
                                squads_raw = read_multi(&body, &mut partial, id);
                                let preview: String = squads_raw.chars().take(100).collect();
                                eprintln!("[RCON] 小队数据: {} 字节, 前100: {:?}", squads_raw.len(), preview);
                                continue;
                            }
                            if body.contains("Current map is ") || body.contains("Current map:") || body.contains("Current level is ") || body.contains("Next map is ")
                                || body.contains("Map: ") || body.contains("Next Map: ") || body.contains("Server Name:")
                                || body.contains("Server name:") || body.contains("Game Mode:") || body.contains("Game mode:")
                                || body.contains("Player count:") || body.contains("Max players:")
                                || body.contains("layer is ") || body.contains("factions ") {
                                // 解析当前地图和下一张地图（兼容多种格式）
                                for line in body.lines() {
                                    let line = line.trim();
                                    // 格式1: "Current map is Narva_RAAS_v1, Next map is ..."
                                    if let Some(v) = line.strip_prefix("Current map is ") {
                                        map_raw = v.split(',').next().unwrap_or(v).trim().to_string();
                                    }
                                    if let Some(v) = line.strip_prefix("Next map is ") {
                                        next_map = v.split(',').next().unwrap_or(v).trim().to_string();
                                    }
                                    // 兼容 "Current map is X, Next map is Y" 格式
                                    if line.contains(", Next map is ") {
                                        if let Some(pos) = line.find(", Next map is ") {
                                            next_map = line[pos + 14..].split(',').next().unwrap_or("").trim().to_string();
                                        }
                                    }
                                    // 格式2 (ShowNextMap): "Current map: X" / "Next map: Y"
                                    if let Some(v) = line.strip_prefix("Current map: ") {
                                        map_raw = v.split(',').next().unwrap_or(v).trim().to_string();
                                    }
                                    if let Some(v) = line.strip_prefix("Next map: ") {
                                        next_map = v.split(',').next().unwrap_or(v).trim().to_string();
                                    }
                                    // 格式3 (ShowServerInfo): "Map: Narva_RAAS_v1"
                                    if let Some(v) = line.strip_prefix("Map: ") {
                                        if map_raw.is_empty() { map_raw = v.to_string(); }
                                    }
                                    if let Some(v) = line.strip_prefix("Next Map: ") {
                                        if next_map.is_empty() { next_map = v.to_string(); }
                                    }
                                    // 格式4 (ShowCurrentMap): "Current level is Narva, layer is Narva_Invasion_v1, factions ..."
                                    if let Some(v) = line.strip_prefix("layer is ") {
                                        map_raw = v.split(',').next().unwrap_or(v).trim().to_string();
                                    }
                                    if let Some(v) = line.strip_prefix("Current level is ") {
                                        // 作为兜底，仅在 map_raw 为空时使用
                                        if map_raw.is_empty() { map_raw = v.split(',').next().unwrap_or(v).trim().to_string(); }
                                    }
                                    // 解析 ShowServerInfo 附加字段
                                    if let Some(v) = line.strip_prefix("Server name: ") {
                                        server_name = v.to_string();
                                    }
                                    if let Some(v) = line.strip_prefix("Server Name: ") {
                                        if server_name.is_empty() { server_name = v.to_string(); }
                                    }
                                    if let Some(v) = line.strip_prefix("Player count: ") {
                                        player_count = v.trim().parse().unwrap_or(0);
                                    }
                                    if let Some(v) = line.strip_prefix("Max players: ") {
                                        max_players = v.trim().parse().unwrap_or(0);
                                    }
                                    if let Some(v) = line.strip_prefix("Game mode: ") {
                                        game_mode = v.to_string();
                                    }
                                    if let Some(v) = line.strip_prefix("Game Mode: ") {
                                        if game_mode.is_empty() { game_mode = v.to_string(); }
                                    }
                                }
                                eprintln!("[RCON] 地图数据: current={:?} next={:?}", map_raw, next_map);
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
                                    message: body.clone(), raw_line: Some(body.clone()), logged_at: chrono::Utc::now(),
                                }});
                            }

                            // ===== 诊断：打印所有未被其他 handler 匹配的 body（前 500 字符）=====
                            if body.len() > 10 && !body.contains("-----") && !body.contains("Current map is") && !body.contains("Current level is") && !body.contains("Next map is") && !body.contains("Map:") && !body.contains("Server Name:") && !body.contains("Server name:") && !body.contains("Game Mode:") && !body.contains("Game mode:") && !body.contains("Players:") && !body.contains("Player count:") && !body.contains("Max players:") && !body.contains("layer is") && !body.contains("factions") && !body.starts_with("Message broadcasted") {
                                let preview: &str = &body[..body.len().min(500)];
                                eprintln!("[RCON] 未匹配: {:?}", preview);
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

// === 读取多包（同 packet ID 继续，不同 ID 停止；空包可能是段间分隔符，跳过） ===
fn read_multi(first: &str, partial: &mut Vec<u8>, first_pkt_id: i32) -> String {
    let mut out = first.to_string();
    while let Some((p2, r2)) = extract(partial) {
        // 检查 packet ID：不同 ID 说明是下一个命令的响应，停止消费
        if let Some(next_id) = pkt_id(&p2) {
            if next_id != first_pkt_id {
                break;
            }
        }
        *partial = r2;
        if let Some(b2) = body_str(&p2) {
            if b2.is_empty() {
                // 同 ID 空包：可能是 Active/RecentlyDisconnected 之间的分隔符，跳过继续读
                continue;
            }
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
fn pkt_id(pkt: &[u8]) -> Option<i32> {
    if pkt.len() < 8 { return None; }
    Some(i32::from_le_bytes([pkt[4], pkt[5], pkt[6], pkt[7]]))
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
    let mut debug_printed = false;
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("-----") { continue; }
        let mut name=String::new(); let mut steam=String::new(); let mut tid=0i32; let mut sq=None; let mut role=String::new();
        let mut k=0i32; let mut d=0i32; let mut sc=0i32; let mut ping=0i32; let mut admin=false; let mut leader=false;
        for part in line.split('|') {
            let v = part.trim();
            if let Some(x)=v.strip_prefix("Name: ") { name=x.to_string(); }
            else if let Some(x)=v.strip_prefix("Online IDs: ") {
                // "Online IDs: EOS: xxx steam: 7656xxx" → 提取 steam ID
                // 兼容 steam:/Steam: 大小写
                if let Some(pos)=x.find("steam: ") { steam = x[pos+7..].split_whitespace().next().unwrap_or("").to_string(); }
                else if let Some(pos)=x.find("Steam: ") { steam = x[pos+7..].split_whitespace().next().unwrap_or("").to_string(); }
                else if let Some(pos)=x.find("steam:") { steam = x[pos+6..].split_whitespace().next().unwrap_or("").to_string(); }
                else if let Some(pos)=x.find("Steam:") { steam = x[pos+6..].split_whitespace().next().unwrap_or("").to_string(); }
            }
            else if let Some(x)=v.strip_prefix("SteamID: ") { steam=x.to_string(); }
            else if let Some(x)=v.strip_prefix("Steam ID: ") { steam=x.to_string(); }
            else if let Some(x)=v.strip_prefix("Player UID: ") { steam=x.to_string(); }
            else if let Some(x)=v.strip_prefix("EOS ID: ") {
                // 只记 EOS ID 作为备用，但优先用 SteamID
                if steam.is_empty() { steam=x.to_string(); }
            }
            else if let Some(x)=v.strip_prefix("Team ID: ") { tid=x.parse().unwrap_or(0); }
            else if let Some(x)=v.strip_prefix("Squad ID: ") { let x=x.trim(); if x!="N/A"&&!x.is_empty() { sq=Some(x.to_string()); } }
            else if let Some(x)=v.strip_prefix("Role: ") { role=x.to_string(); }
            else if let Some(x)=v.strip_prefix("Kills: ") { k=x.parse().unwrap_or(0); }
            else if let Some(x)=v.strip_prefix("Deaths: ") { d=x.parse().unwrap_or(0); }
            else if let Some(x)=v.strip_prefix("Score: ") { sc=x.parse().unwrap_or(0); }
            else if let Some(x)=v.strip_prefix("Ping: ") { ping=x.parse().unwrap_or(0); }
            else if let Some(x)=v.strip_prefix("Admin: ") { admin=x.trim()=="True"; }
            else if let Some(x)=v.strip_prefix("Is Leader: ") { leader=x.trim()=="True"; }
        }
        // 兜底：从整行中查找 17 位 SteamID64（7656119 开头）
        if steam.is_empty() || !steam.chars().all(|c| c.is_ascii_digit()) {
            let fallback = find_steam64_in_line(line);
            if !fallback.is_empty() { steam = fallback; }
        }
        // 跳过 Recently Disconnected 玩家（没有 Team ID，不参与游戏内操作）
        let is_disconnected = line.contains("Since Disconnect:");
        if !name.is_empty() && !is_disconnected {
            if !debug_printed && steam.is_empty() {
                eprintln!("[RCON] ⚠ SteamID 解析失败，玩家行示例(前200字符): {:?}", &line[..line.len().min(200)]);
                debug_printed = true;
            }
            out.push(PlayerInfo{name,steam_id:steam,team_id:tid,squad_id:sq,role,kills:k,deaths:d,score:sc,ping,is_admin:admin,is_leader:leader});
        }
    }
    out
}
/// 从行中查找 17 位 SteamID64（以 7656119 开头）
fn find_steam64_in_line(line: &str) -> String {
    let mut i = 0;
    while i < line.len() {
        let rest = &line[i..];
        // 找连续的 17 位数字
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.len() == 17 && digits.starts_with("7656119") {
            // 检查数字前后是否为非数字（确保是独立数字，非更长数字的一部分）
            let before_digit = i == 0 || !line[..i].chars().last().map_or(false, |c| c.is_ascii_digit());
            let after_pos = i + 17;
            let after_digit = after_pos >= line.len() || !line[after_pos..].chars().next().map_or(false, |c| c.is_ascii_digit());
            if before_digit && after_digit {
                return digits;
            }
        }
        if digits.len() > 0 { i += digits.len(); } else { i += rest.chars().next().map_or(0, |c| c.len_utf8()); }
    }
    String::new()
}
fn parse_squads(raw: &str) -> Vec<SquadInfo> {
    let mut out=Vec::new(); let mut ct=0i32;
    for line in raw.lines() {
        let line=line.trim();
        if line.is_empty()||line.starts_with("-----") { continue; }
        if line.to_lowercase().starts_with("team id:") { ct=line.split_whitespace().nth(2).and_then(|s|s.parse().ok()).unwrap_or(0); continue; }
        // 兼容两种格式：
        //   旧格式: "Squad 1: Alpha - CreatorName"
        //   新格式: "ID: 1 | Name: 老年团 | Size: 1 |"
        if line.to_lowercase().starts_with("squad ") {
            // 旧格式解析
            if let Some((left,rest)) = line.split_once(": ") {
                let sid = left.strip_prefix("Squad ").unwrap_or(left).trim().to_string();
                let (name,creator) = rest.split_once(" - ").map_or((rest.to_string(),String::new()),|(a,b)|(a.trim().to_string(),b.trim().to_string()));
                out.push(SquadInfo{name,creator,team_id:ct,squad_id:sid,leader_name:None,leader_steam_id:None});
            }
        } else if line.starts_with("ID: ") && !line.to_lowercase().starts_with("team id:") {
            // 新格式: "ID: 1 | Name: 老年团 | Size: 1 | Leader: PlayerA" 或 "ID: 1 | Name: 老年团 | Size: 1 |"
            let mut squad_id = String::new();
            let mut name = String::new();
            let mut leader_name: Option<String> = None;
            let parts: Vec<&str> = line.split('|').collect();
            for part in &parts {
                let v = part.trim();
                if let Some(id_val) = v.strip_prefix("ID: ") {
                    squad_id = id_val.to_string();
                } else if let Some(n) = v.strip_prefix("Name: ") {
                    name = n.to_string();
                } else if let Some(l) = v.strip_prefix("Leader: ") {
                    let l = l.to_string();
                    if !l.is_empty() { leader_name = Some(l); }
                }
            }
            if !squad_id.is_empty() {
                out.push(SquadInfo { name, creator: String::new(), team_id: ct, squad_id, leader_name, leader_steam_id: None });
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
