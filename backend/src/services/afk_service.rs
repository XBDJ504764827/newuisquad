use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use sqlx::PgPool;
use serde::Serialize;

use crate::rcon_client::pool::RconPool;
use crate::models::afk_settings::AfkSettings;

/// AFK tracking per player
#[derive(Debug, Clone)]
pub(crate) struct AfkPlayerState {
    pub player_name: String,
    pub player_id: i32,
    pub unassigned_since: chrono::DateTime<chrono::Utc>,
    pub last_warning_at: Option<chrono::DateTime<chrono::Utc>>,
    pub warning_count: i32,
    pub kicked: bool,
}

/// AFK detection service — warns and kicks unassigned/idle players
pub struct AfkService {
    pool: PgPool,
    rcon_pool: RconPool,
    afk_states: Arc<tokio::sync::RwLock<HashMap<i32, HashMap<i32, AfkPlayerState>>>>, // server_id → player_id → state
}

impl AfkService {
    pub fn new(pool: PgPool, rcon_pool: RconPool) -> Self {
        Self {
            pool,
            rcon_pool,
            afk_states: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Start background AFK checking (every 60 seconds)
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            tracing::info!("AFK 管理服务已启动");
            tokio::time::sleep(Duration::from_secs(30)).await;
            loop {
                self.check_all().await;
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        })
    }

    async fn check_all(&self) {
        let servers = match sqlx::query_as::<_, (i32,)>(
            "SELECT id FROM servers WHERE rcon_port > 0 AND rcon_password != ''"
        ).fetch_all(&self.pool).await {
            Ok(s) => s,
            Err(_) => return,
        };

        for (server_id,) in servers {
            if let Err(e) = self.check_server(server_id).await {
                tracing::debug!(server_id, error = %e, "AFK 检查失败");
            }
        }
    }

    async fn check_server(&self, server_id: i32) -> Result<(), String> {
        let settings = match sqlx::query_as::<_, (bool, i32, i32)>(
            "SELECT enabled, min_players_to_check, max_afk_minutes FROM afk_settings WHERE server_id=$1"
        ).bind(server_id).fetch_optional(&self.pool).await.map_err(|e| e.to_string())? {
            Some((enabled, min_players, max_afk)) => {
                if !enabled { return Ok(()); }
                (min_players, max_afk)
            }
            None => return Ok(()),
        };

        // Get current players
        let raw = self.rcon_pool.execute_by_server_id(server_id, "ListPlayers").await?;
        let players = parse_players(&raw);

        // Only check if enough players online
        if players.len() < settings.0 as usize {
            // Below threshold — reset all tracking
            self.afk_states.write().await.remove(&server_id);
            return Ok(());
        }

        let now = chrono::Utc::now();
        let max_afk = settings.1;
        let mut states = self.afk_states.write().await;
        let server_states = states.entry(server_id).or_default();

        // Track current player IDs for cleanup
        let current_ids: std::collections::HashSet<i32> = players.iter().map(|p| p.0).collect();

        // Check each player
        for (pid, name, squad_id) in &players {
            let pid = *pid;
            let squad_id = squad_id.clone();
            if squad_id.is_some() {
                // Player is in a squad — not AFK, remove from tracking
                server_states.remove(&pid);
                continue;
            }

            // Unassigned player
            let entry = server_states.entry(pid).or_insert_with(|| AfkPlayerState {
                player_name: name.clone(),
                player_id: pid,
                unassigned_since: now,
                last_warning_at: None,
                warning_count: 0,
                kicked: false,
            });

            if entry.kicked {
                continue;
            }

            let afk_minutes = (now - entry.unassigned_since).num_minutes() as i32;

            if afk_minutes >= max_afk {
                // Auto-kick
                let kick_cmd = format!(
                    "AdminKickById {} 您因超过{}分钟未加入小队被自动踢出",
                    pid, max_afk
                );
                if self.rcon_pool.execute_by_server_id(server_id, &kick_cmd).await.is_ok() {
                    entry.kicked = true;
                    tracing::info!(server_id, player = %name, pid, afk_minutes, "AFK 自动踢出");
                }
            } else if afk_minutes >= max_afk / 2 && afk_minutes >= 5 {
                // Send warning (at half the limit, and at least every 3 minutes)
                let should_warn = match entry.last_warning_at {
                    Some(last) => (now - last).num_minutes() >= 3,
                    None => true,
                };
                if should_warn {
                    let remaining = max_afk - afk_minutes;
                    let warn_cmd = format!(
                        "AdminWarn \"{}\" 您未加入小队已有{}分钟，请在{}分钟内加入小队否则将被自动踢出",
                        name, afk_minutes, remaining
                    );
                    let _ = self.rcon_pool.execute_by_server_id(server_id, &warn_cmd).await;
                    entry.last_warning_at = Some(now);
                    entry.warning_count += 1;
                    tracing::info!(server_id, player = %name, pid, afk_minutes, remaining, "AFK 警告");
                }
            }
        }

        // Cleanup players who left
        server_states.retain(|pid, state| {
            current_ids.contains(pid) || state.kicked
        });

        Ok(())
    }

    /// Get AFK state for a server
    pub async fn get_state(&self, server_id: i32) -> Vec<AfkPlayerState> {
        let states = self.afk_states.read().await;
        states.get(&server_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }
}

/// Parse ListPlayers output into (player_id, name, squad_id_option)
fn parse_players(raw: &str) -> Vec<(i32, String, Option<String>)> {
    let mut players = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("-----") || line.starts_with("Active") || line.starts_with("Recently") {
            continue;
        }
        let mut pid = 0i32;
        let mut name = String::new();
        let mut squad_id: Option<String> = None;
        for part in line.split('|') {
            let part = part.trim();
            if let Some(v) = part.strip_prefix("ID: ") { pid = v.parse().unwrap_or(0); }
            else if let Some(v) = part.strip_prefix("Name: ") { name = v.to_string(); }
            else if let Some(v) = part.strip_prefix("Squad ID: ") {
                let s = v.trim();
                if s != "N/A" && !s.is_empty() { squad_id = Some(s.to_string()); }
            }
        }
        if pid > 0 && !name.is_empty() {
            players.push((pid, name, squad_id));
        }
    }
    players
}

#[derive(Debug, Clone, Serialize)]
pub struct AfkStatus {
    pub server_id: i32,
    pub players: Vec<AfkPlayerInfo>,
    pub total_afk: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AfkPlayerInfo {
    pub player_name: String,
    pub player_id: i32,
    pub afk_minutes: i32,
    pub warning_count: i32,
    pub kicked: bool,
}
