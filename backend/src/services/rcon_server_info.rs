use crate::rcon_client::squad::SquadRcon;

pub struct PlayerInfo {
    pub name: String,
    pub steam_id: String,
    pub team_id: i32,
    pub squad_id: Option<String>,
    pub role: String,
    pub kills: i32,
    pub deaths: i32,
    pub score: i32,
    pub ping: i32,
    pub is_admin: bool,
}

pub struct SquadInfo {
    pub name: String,
    pub creator: String,
    pub team_id: i32,
}

pub struct ServerState {
    pub players: Vec<PlayerInfo>,
    pub squads: Vec<SquadInfo>,
    pub teams: Vec<String>,
    pub map_name: String,
    pub game_mode: String,
}

/// 执行 ListPlayers 并解析
pub async fn list_players(ip: &str, port: u16, password: &str) -> Result<Vec<PlayerInfo>, String> {
    let mut rcon = SquadRcon::connect(ip, port, password).await?;
    let raw = rcon.execute("ListPlayers").await?;
    Ok(parse_list_players(&raw))
}

/// 执行 ListSquads 并解析
pub async fn list_squads(ip: &str, port: u16, password: &str) -> Result<Vec<SquadInfo>, String> {
    let mut rcon = SquadRcon::connect(ip, port, password).await?;
    let raw = rcon.execute("ListSquads").await?;
    Ok(parse_list_squads(&raw))
}

/// 获取当前地图
pub async fn get_map(ip: &str, port: u16, password: &str) -> Result<(String, String), String> {
    let mut rcon = SquadRcon::connect(ip, port, password).await?;
    let raw = rcon.execute("ShowNextMap").await?;
    Ok(parse_map(&raw))
}

/// Ban/Warn 条目
#[derive(serde::Serialize)]
pub struct BanEntry {
    pub player_name: String,
    pub steam_id: String,
    pub duration: String,
    pub reason: String,
    pub admin: String,
}

#[derive(serde::Serialize)]
pub struct WarnEntry {
    pub player_name: String,
    pub steam_id: String,
    pub reason: String,
    pub admin: String,
}

#[derive(serde::Serialize)]
pub struct ServerInfo {
    pub server_name: String,
    pub player_count: i32,
    pub max_players: i32,
    pub map_name: String,
    pub game_mode: String,
    pub next_map: String,
    pub next_layer: String,
}

/// 获取 Ban 列表
pub async fn get_bans(ip: &str, port: u16, password: &str) -> Result<Vec<BanEntry>, String> {
    let mut rcon = SquadRcon::connect(ip, port, password).await?;
    let raw = rcon.execute("AdminListBans").await?;
    Ok(parse_ban_list(&raw))
}

/// 获取 Warn 列表
pub async fn get_warns(ip: &str, port: u16, password: &str) -> Result<Vec<WarnEntry>, String> {
    let mut rcon = SquadRcon::connect(ip, port, password).await?;
    let raw = rcon.execute("AdminListWarns").await?;
    Ok(parse_warn_list(&raw))
}

/// 获取服务器基本信息
pub async fn get_server_info(ip: &str, port: u16, password: &str) -> Result<ServerInfo, String> {
    let mut rcon = SquadRcon::connect(ip, port, password).await?;
    let raw = rcon.execute("ShowServerInfo").await?;
    let (next_map, next_layer) = get_map(ip, port, password).await.unwrap_or_default();
    Ok(parse_server_info(&raw, &next_map, &next_layer))
}

/// 获取完整服务器状态（含 ban/warn/server info）
pub async fn get_server_state(ip: &str, port: u16, password: &str) -> Result<ServerState, String> {
    let players = list_players(ip, port, password).await.unwrap_or_default();
    let squads = list_squads(ip, port, password).await.unwrap_or_default();
    let (map_name, game_mode) = get_map(ip, port, password).await.unwrap_or_default();

    let teams = players.iter()
        .filter_map(|p| if p.team_id == 1 { Some("US Army".to_string()) } else if p.team_id == 2 { Some("RUS".to_string()) } else { None })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    Ok(ServerState { players, squads, teams, map_name, game_mode })
}

fn parse_list_players(raw: &str) -> Vec<PlayerInfo> {
    let mut players = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("-----") || line.starts_with("Active") || line.starts_with("Recently") {
            continue;
        }
        // 格式: ID: 0 | Name: Player1 | SteamID: 7656xxx | Team ID: 1 | Squad ID: 1 | Role: Rifleman | ...
        let mut name = String::new();
        let mut steam_id = String::new();
        let mut team_id = 0i32;
        let mut squad_id = None;
        let mut role = String::new();
        let mut kills = 0i32;
        let mut deaths = 0i32;
        let mut score = 0i32;
        let mut ping = 0i32;
        let mut is_admin = false;

        for part in line.split('|') {
            let part = part.trim();
            if let Some(v) = part.strip_prefix("Name: ") { name = v.trim().to_string(); }
            else if let Some(v) = part.strip_prefix("SteamID: ") { steam_id = v.trim().to_string(); }
            else if let Some(v) = part.strip_prefix("Team ID: ") { team_id = v.trim().parse().unwrap_or(0); }
            else if let Some(v) = part.strip_prefix("Squad ID: ") { let s = v.trim(); if s != "N/A" && !s.is_empty() { squad_id = Some(s.to_string()); } }
            else if let Some(v) = part.strip_prefix("Role: ") { role = v.trim().to_string(); }
            else if let Some(v) = part.strip_prefix("Kills: ") { kills = v.trim().parse().unwrap_or(0); }
            else if let Some(v) = part.strip_prefix("Deaths: ") { deaths = v.trim().parse().unwrap_or(0); }
            else if let Some(v) = part.strip_prefix("Score: ") { score = v.trim().parse().unwrap_or(0); }
            else if let Some(v) = part.strip_prefix("Ping: ") { ping = v.trim().parse().unwrap_or(0); }
            else if let Some(v) = part.strip_prefix("Admin: ") { is_admin = v.trim() == "True"; }
        }

        if !name.is_empty() {
            players.push(PlayerInfo { name, steam_id, team_id, squad_id, role, kills, deaths, score, ping, is_admin });
        }
    }
    players
}

fn parse_list_squads(raw: &str) -> Vec<SquadInfo> {
    let mut squads = Vec::new();
    let mut current_team = 0i32;
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        // 团队行: Team 1 (US Army):
        if line.to_lowercase().starts_with("team ") {
            if let Some(t) = line.split_whitespace().nth(1) {
                current_team = t.trim_end_matches(':').parse().unwrap_or(0);
            }
            continue;
        }
        // 小队行: Squad 1: Alpha - CreatorName
        if line.to_lowercase().starts_with("squad ") || line.contains(':') {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let name_part = parts[1].trim();
                let (squad_name, creator) = if let Some(pos) = name_part.find(" - ") {
                    (name_part[..pos].trim().to_string(), name_part[pos + 3..].trim().to_string())
                } else {
                    (name_part.to_string(), String::new())
                };
                squads.push(SquadInfo { name: squad_name, creator, team_id: current_team });
            }
        }
    }
    squads
}

fn parse_map(raw: &str) -> (String, String) {
    let mut map_name = String::new();
    let mut game_mode = String::new();
    for line in raw.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("Map: ") { map_name = v.to_string(); }
        else if let Some(v) = line.strip_prefix("Game Mode: ") { game_mode = v.to_string(); }
        else if line.starts_with("Current map") || line.starts_with("Next map") {
            if let Some(v) = line.strip_prefix("Current map is ") { map_name = v.to_string(); }
            else if let Some(v) = line.strip_prefix("Next map is ") { map_name = v.to_string(); }
        }
    }
    if map_name.is_empty() { map_name = "Unknown".to_string(); }
    if game_mode.is_empty() { game_mode = "Unknown".to_string(); }
    (map_name, game_mode)
}

fn parse_ban_list(raw: &str) -> Vec<BanEntry> {
    let mut bans = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("-----") { continue; }
        let mut name = String::new();
        let mut steam_id = String::new();
        let mut duration = String::new();
        let mut reason = String::new();
        let mut admin = String::new();
        for part in line.split('|') {
            let part = part.trim();
            if let Some(v) = part.strip_prefix("Name: ") { name = v.to_string(); }
            else if let Some(v) = part.strip_prefix("SteamID: ") { steam_id = v.to_string(); }
            else if let Some(v) = part.strip_prefix("Duration: ") { duration = v.to_string(); }
            else if let Some(v) = part.strip_prefix("Reason: ") { reason = v.to_string(); }
            else if let Some(v) = part.strip_prefix("Admin: ") { admin = v.to_string(); }
        }
        if !name.is_empty() { bans.push(BanEntry { player_name: name, steam_id, duration, reason, admin }); }
    }
    bans
}

fn parse_warn_list(raw: &str) -> Vec<WarnEntry> {
    let mut warns = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("-----") { continue; }
        let mut name = String::new();
        let mut steam_id = String::new();
        let mut reason = String::new();
        let mut admin = String::new();
        for part in line.split('|') {
            let part = part.trim();
            if let Some(v) = part.strip_prefix("Name: ") { name = v.to_string(); }
            else if let Some(v) = part.strip_prefix("SteamID: ") { steam_id = v.to_string(); }
            else if let Some(v) = part.strip_prefix("Reason: ") { reason = v.to_string(); }
            else if let Some(v) = part.strip_prefix("Admin: ") { admin = v.to_string(); }
        }
        if !name.is_empty() { warns.push(WarnEntry { player_name: name, steam_id, reason, admin }); }
    }
    warns
}

fn parse_server_info(raw: &str, next_map: &str, next_layer: &str) -> ServerInfo {
    let mut server_name = String::new();
    let mut player_count = 0i32;
    let mut max_players = 0i32;
    let mut map_name = String::new();
    let mut game_mode = String::new();

    for line in raw.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("Server name: ") { server_name = v.to_string(); }
        else if let Some(v) = line.strip_prefix("Player count: ") { player_count = v.trim().parse().unwrap_or(0); }
        else if let Some(v) = line.strip_prefix("Max players: ") { max_players = v.trim().parse().unwrap_or(0); }
        else if let Some(v) = line.strip_prefix("Map: ") { map_name = v.to_string(); }
        else if let Some(v) = line.strip_prefix("Game Mode: ") { game_mode = v.to_string(); }
    }

    ServerInfo {
        server_name,
        player_count,
        max_players,
        map_name: if map_name.is_empty() { "Unknown".to_string() } else { map_name },
        game_mode: if game_mode.is_empty() { "Unknown".to_string() } else { game_mode },
        next_map: next_map.to_string(),
        next_layer: next_layer.to_string(),
    }
}
