use std::collections::HashMap;
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
            // 设为 3 秒前，让首次状态查询立即执行
            let mut last_query = Instant::now() - Duration::from_secs(3);
            // RCON 命令 ID 计数器（每个命令唯一 ID）
            let mut cmd_id: i32 = 100;
            // 按 ID 累积响应: id → (type, data)
            let mut pending: HashMap<i32, (String, String)> = HashMap::new();
            // 已完成的查询结果
            let mut players_data = String::new();
            let mut squads_data = String::new();
            let mut map_data = String::new();
            let mut server_info_json = String::new();

            loop {
                // 每 3 秒查询状态
                if last_query.elapsed().as_secs() >= 3 {
                    // 将 pending 中已完成的响应转移到对应的数据区
                    let mut new_players = String::new();
                    let mut new_squads = String::new();
                    let mut new_map = String::new();
                    let mut new_info = String::new();
                    for (_id, (typ, data)) in pending.drain() {
                        if data.is_empty() { continue; }
                        match typ.as_str() {
                            "players" => { new_players = data; }
                            "squads" => { new_squads = data; }
                            "map" => { new_map = data; }
                            "info" => { new_info = data; }
                            _ => {}
                        }
                    }
                    // 保留最新数据
                    if !new_players.is_empty() { players_data = new_players; }
                    if !new_squads.is_empty() { squads_data = new_squads; }
                    if !new_map.is_empty() { map_data = new_map; }
                    if !new_info.is_empty() { server_info_json = new_info; }

                    // 处理所有已完成的累积响应
                    if !players_data.is_empty() || !squads_data.is_empty() || !map_data.is_empty() || !server_info_json.is_empty() {
                        let players = parse_players(&players_data);
                        let squads = parse_squads(&squads_data);
                        let mut team_names = parse_team_names(&squads_data);
                        if team_names.is_empty() {
                            let mut seen = std::collections::HashSet::new();
                            for p in &players {
                                if p.team_id != 0 && seen.insert(p.team_id) {
                                    team_names.push(TeamName { team_id: p.team_id, faction: format!("队伍 {}", p.team_id) });
                                }
                            }
                        }
                        // 从累积的地图数据和 server_info_json 中提取信息
                        let mut map_name = String::new();
                        let mut next_map_name = String::new();
                        let mut server_name = String::new();
                        let mut player_count = 0i32;
                        let mut max_players = 0i32;
                        let mut game_mode = String::new();
                        // 合并所有地图相关数据一起解析
                        let all_map_data = format!("{}\n{}", map_data, server_info_json);
                        parse_map_info(&all_map_data, &mut map_name, &mut next_map_name, &mut server_name, &mut player_count, &mut max_players, &mut game_mode);
                        let actual_player_count = player_count.max(players.len() as i32);
                        eprintln!("[RCON] 上报: {}人/{}队 阵营{:?} 地图{}", players.len(), squads.len(), team_names.iter().map(|t|&t.faction).collect::<Vec<_>>(), map_name);
                        if !players.is_empty() {
                            let _ = msg_tx.send(AgentMessage::ServerStateReport {
                                players, squads, team_names,
                                map_name, game_mode,
                                server_name, player_count: actual_player_count, max_players,
                                next_map: next_map_name,
                            });
                        }
                    }
                    players_data.clear(); squads_data.clear(); map_data.clear(); server_info_json.clear();
                    pending.clear();

                    // 发送新一轮查询（每个命令用唯一 ID）
                    cmd_id += 1; let pid_players = cmd_id;
                    cmd_id += 1; let pid_squads = cmd_id;
                    cmd_id += 1; let pid_info = cmd_id;
                    cmd_id += 1; let pid_next = cmd_id;
                    cmd_id += 1; let pid_cur = cmd_id;
                    let _ = s.write_all(&build(2, pid_players, "ListPlayers"));
                    let _ = s.write_all(&build(2, pid_squads, "ListSquads"));
                    let _ = s.write_all(&build(2, pid_info, "ShowServerInfo"));
                    let _ = s.write_all(&build(2, pid_next, "ShowNextMap"));
                    let _ = s.write_all(&build(2, pid_cur, "ShowCurrentMap"));
                    pending.insert(pid_players, ("players".into(), String::new()));
                    pending.insert(pid_squads, ("squads".into(), String::new()));
                    pending.insert(pid_info, ("info".into(), String::new()));
                    pending.insert(pid_next, ("map".into(), String::new()));
                    pending.insert(pid_cur, ("map".into(), String::new()));
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

                            // 按 packet ID 路由响应到对应的累积器
                            let pid = pkt_id(&pkt).unwrap_or(0);
                            if pid != 0 {
                                if let Some((ref _typ, ref mut acc)) = pending.get_mut(&pid) {
                                    acc.push_str(&body);
                                    // 标注类型方便调试
                                    if body.starts_with("----- Active Players -----") {
                                        eprintln!("[RCON] 玩家响应开始，ID={}", pid);
                                    }
                                    if body.starts_with("----- Active Squads -----") {
                                        eprintln!("[RCON] 小队响应开始，ID={}", pid);
                                    }
                                    continue;
                                }
                            }

                            // 非查询命令的响应（聊天、玩家加入/离开等实时事件）

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

                            // ===== 未匹配包：仅对长度为 5-200 的非系统包做诊断 =====
                            if body.len() > 5 && body.len() < 200 && !body.contains("-----") && !body.contains("Online IDs:") && !body.starts_with("{") {
                                let preview: &str = &body[..body.len().min(150)];
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

/// 从 ShowCurrentMap / ShowNextMap / ShowServerInfo 的累积响应中提取地图信息
fn parse_map_info(data: &str, map_name: &mut String, next_map: &mut String, server_name: &mut String, player_count: &mut i32, max_players: &mut i32, game_mode: &mut String) {
    for line in data.lines() {
        let line = line.trim();
        // ShowCurrentMap / ShowNextMap 格式
        if let Some(v) = line.strip_prefix("Current map is ") {
            *map_name = v.split(',').next().unwrap_or(v).trim().to_string();
        }
        if let Some(v) = line.strip_prefix("Next map is ") {
            *next_map = v.split(',').next().unwrap_or(v).trim().to_string();
        }
        if line.contains(", Next map is ") {
            if let Some(pos) = line.find(", Next map is ") {
                *next_map = line[pos + 14..].split(',').next().unwrap_or("").trim().to_string();
            }
        }
        if let Some(v) = line.strip_prefix("Current map: ") {
            *map_name = v.split(',').next().unwrap_or(v).trim().to_string();
        }
        if let Some(v) = line.strip_prefix("Next map: ") {
            *next_map = v.split(',').next().unwrap_or(v).trim().to_string();
        }
        if let Some(v) = line.strip_prefix("layer is ") {
            *map_name = v.split(',').next().unwrap_or(v).trim().to_string();
        }
        if let Some(v) = line.strip_prefix("Current level is ") {
            if map_name.is_empty() { *map_name = v.split(',').next().unwrap_or(v).trim().to_string(); }
        }
        // ShowServerInfo JSON 字段（虽为 JSON，但可能夹杂文本行）
        if let Some(v) = line.strip_prefix("Map: ") {
            if map_name.is_empty() { *map_name = v.to_string(); }
        }
        if let Some(v) = line.strip_prefix("Next Map: ") {
            if next_map.is_empty() { *next_map = v.to_string(); }
        }
        if let Some(v) = line.strip_prefix("Server name: ") {
            *server_name = v.to_string();
        }
        if let Some(v) = line.strip_prefix("Server Name: ") {
            if server_name.is_empty() { *server_name = v.to_string(); }
        }
        if let Some(v) = line.strip_prefix("Player count: ") {
            *player_count = v.trim().parse().unwrap_or(0);
        }
        if let Some(v) = line.strip_prefix("Max players: ") {
            *max_players = v.trim().parse().unwrap_or(0);
        }
        if let Some(v) = line.strip_prefix("Game mode: ") {
            *game_mode = v.to_string();
        }
        if let Some(v) = line.strip_prefix("Game Mode: ") {
            if game_mode.is_empty() { *game_mode = v.to_string(); }
        }
        // ShowServerInfo JSON 中的字段
        // 格式: "MapName_s":"Gorodok_Invasion_v1","ServerName_s":"name","GameMode_s":"mode"
        // 提取 JSON 中 "key":"value" 的值部分
        let extract_json_val = |data: &str, key: &str| -> Option<String> {
            let search = format!("\"{}\":", key);
            let pos = data.find(&search)?;
            let rest = &data[pos + search.len()..];
            let rest = rest.trim_start();
            if rest.starts_with('"') {
                // 字符串值: "value" → 找下一个无转义的 "
                let inner = &rest[1..];
                let mut chars = inner.chars();
                let mut val = String::new();
                loop {
                    match chars.next() {
                        Some('\\') => {
                            val.push('\\');
                            if let Some(c) = chars.next() { val.push(c); }
                        }
                        Some('"') => break,
                        Some(c) => val.push(c),
                        None => break,
                    }
                }
                Some(val)
            } else {
                // 数字/布尔值: 123 → 找下一个 , 或 }
                let end = rest.find(|c: char| c == ',' || c == '}').unwrap_or(rest.len());
                Some(rest[..end].trim().to_string())
            }
        };
        if map_name.is_empty() {
            if let Some(v) = extract_json_val(line, "MapName_s") { *map_name = v; }
        }
        if server_name.is_empty() {
            if let Some(v) = extract_json_val(line, "ServerName_s") { *server_name = v; }
        }
        if game_mode.is_empty() {
            if let Some(v) = extract_json_val(line, "GameMode_s") { *game_mode = v; }
        }
        if *player_count == 0 {
            if let Some(v) = extract_json_val(line, "PlayerCount_I") {
                *player_count = v.parse().unwrap_or(0);
            }
        }
        if *max_players == 0 {
            if let Some(v) = extract_json_val(line, "MaxPlayers") {
                *max_players = v.parse().unwrap_or(0);
            }
        }
    }
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
