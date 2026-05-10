use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sqlx::PgPool;
use tokio::sync::RwLock;
use serde::Serialize;

use crate::rcon_client::pool::RconPool;
use crate::services::rcon_server_info;
use crate::services::event_manager::{EventManager, GameEvent, PlayerListUpdatedData, EventType};

// ═══ Data Structures ═══

#[derive(Debug, Clone, Serialize)]
pub struct TrackedPlayer {
    pub name: String,
    pub steam_id: String,
    pub eos_id: String,
    pub team_id: i32,
    pub team_name: String,
    pub squad_id: Option<String>,
    pub squad_name: String,
    pub role: String,
    pub is_squad_leader: bool,
    pub player_controller: Option<String>,
    pub is_connected: bool,
    pub kills: i32,
    pub deaths: i32,
    pub score: i32,
    pub ping: i32,
    pub is_admin: bool,
    pub player_id: i32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackedTeam {
    pub team_id: i32,
    pub team_name: String,
    pub faction: String,
    pub player_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackedSquad {
    pub squad_id: String,
    pub squad_name: String,
    pub team_id: i32,
    pub team_name: String,
    pub size: usize,
    pub squad_leader: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerPlayerState {
    pub players: Vec<TrackedPlayer>,
    pub teams: Vec<TrackedTeam>,
    pub squads: Vec<TrackedSquad>,
    pub player_count: usize,
    pub max_players: i32,
    pub map_name: String,
    pub game_mode: String,
    pub last_refresh: chrono::DateTime<chrono::Utc>,
    pub refresh_error: Option<String>,
}

// ═══ PlayerTracker Service ═══

pub struct PlayerTracker {
    pool: PgPool,
    rcon_pool: RconPool,
    event_manager: Option<Arc<EventManager>>,
    states: Arc<RwLock<HashMap<i32, ServerPlayerState>>>,
}

impl PlayerTracker {
    pub fn new(pool: PgPool, rcon_pool: RconPool, event_manager: Option<Arc<EventManager>>) -> Self {
        Self {
            pool, rcon_pool, event_manager, states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a clone of the states map
    pub fn states(&self) -> Arc<RwLock<HashMap<i32, ServerPlayerState>>> {
        self.states.clone()
    }

    /// Start the background refresh loop
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            tracing::info!("PlayerTracker 服务已启动");
            loop {
                self.refresh_all_servers().await;
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        })
    }

    /// Force refresh a specific server
    pub async fn force_refresh(&self, server_id: i32) {
        self.refresh_server(server_id).await;
    }

    /// Get live player state for a server
    pub async fn get_state(&self, server_id: i32) -> Option<ServerPlayerState> {
        let states = self.states.read().await;
        states.get(&server_id).cloned()
    }

    /// Get all server states
    pub async fn get_all_states(&self) -> HashMap<i32, ServerPlayerState> {
        let states = self.states.read().await;
        states.clone()
    }

    /// Lookup a player by name across all tracked servers
    pub async fn find_player_by_name(&self, name: &str) -> Vec<(i32, TrackedPlayer)> {
        let states = self.states.read().await;
        let mut results = Vec::new();
        let lower = name.to_lowercase();
        for (&server_id, state) in states.iter() {
            for player in &state.players {
                if player.name.to_lowercase().contains(&lower) {
                    results.push((server_id, player.clone()));
                }
            }
        }
        results
    }

    /// Lookup a player by SteamID
    pub async fn find_player_by_steam(&self, steam_id: &str) -> Option<(i32, TrackedPlayer)> {
        let states = self.states.read().await;
        for (&server_id, state) in states.iter() {
            for player in &state.players {
                if player.steam_id == steam_id {
                    return Some((server_id, player.clone()));
                }
            }
        }
        None
    }

    /// Check if two players are on the same team (teamkill detection)
    pub async fn is_same_team(&self, server_id: i32, attacker_eos: &str, victim_eos: &str) -> Option<bool> {
        let states = self.states.read().await;
        let state = states.get(&server_id)?;
        let attacker = state.players.iter().find(|p| p.eos_id == attacker_eos || p.steam_id == attacker_eos)?;
        let victim = state.players.iter().find(|p| p.eos_id == victim_eos || p.steam_id == victim_eos)?;
        Some(attacker.team_id == victim.team_id && attacker.team_id != 0)
    }

    /// Get players by team
    pub async fn get_players_by_team(&self, server_id: i32, team_id: i32) -> Vec<TrackedPlayer> {
        let states = self.states.read().await;
        states.get(&server_id)
            .map(|s| s.players.iter().filter(|p| p.team_id == team_id).cloned().collect())
            .unwrap_or_default()
    }

    /// Get players by squad
    pub async fn get_players_by_squad(&self, server_id: i32, squad_id: &str) -> Vec<TrackedPlayer> {
        let states = self.states.read().await;
        states.get(&server_id)
            .map(|s| s.players.iter().filter(|p| p.squad_id.as_deref() == Some(squad_id)).cloned().collect())
            .unwrap_or_default()
    }

    // ═══ Internal ═══

    async fn refresh_all_servers(&self) {
        let servers = match sqlx::query_as::<_, (i32, String, i32, String)>(
            "SELECT id, ip, rcon_port, rcon_password FROM servers WHERE rcon_port > 0 AND rcon_password != ''"
        ).fetch_all(&self.pool).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "PlayerTracker 查询服务器列表失败");
                return;
            }
        };

        for (id, ip, port, password) in &servers {
            self.refresh_server_inner(*id, ip, *port as u16, password).await;
        }
    }

    async fn refresh_server(&self, server_id: i32) {
        let creds = match sqlx::query_as::<_, (String, i32, String)>(
            "SELECT ip, rcon_port, rcon_password FROM servers WHERE id=$1"
        ).bind(server_id).fetch_optional(&self.pool).await {
            Ok(Some(c)) => c,
            _ => return,
        };
        let (ip, port, password) = creds;
        self.refresh_server_inner(server_id, &ip, port as u16, &password).await;
    }

    async fn refresh_server_inner(&self, server_id: i32, ip: &str, port: u16, password: &str) {
        let now = chrono::Utc::now();

        // Get players, squads, and map info via RCON pool
        let players_fut = rcon_server_info::list_players(&self.rcon_pool, ip, port, password);
        let squads_fut = rcon_server_info::list_squads(&self.rcon_pool, ip, port, password);
        let map_fut = rcon_server_info::get_map(&self.rcon_pool, ip, port, password);

        let (players_result, squads_result, map_result) = tokio::join!(players_fut, squads_fut, map_fut);

        let players = players_result.unwrap_or_default();
        let squads = squads_result.unwrap_or_default();
        let (map_name, game_mode) = map_result.unwrap_or_default();

        // Build tracked players
        let tracked_players: Vec<TrackedPlayer> = players.iter().map(|p| TrackedPlayer {
            name: p.name.clone(),
            steam_id: p.steam_id.clone(),
            eos_id: String::new(), // RCON doesn't return EOSID for ListPlayers
            team_id: p.team_id,
            team_name: format!("队伍 {}", p.team_id),
            squad_id: p.squad_id.clone(),
            squad_name: String::new(),
            role: p.role.clone(),
            is_squad_leader: p.role.to_lowercase().contains("squad") || p.role.to_lowercase().contains("leader"),
            player_controller: None,
            is_connected: true,
            kills: p.kills,
            deaths: p.deaths,
            score: p.score,
            ping: p.ping,
            is_admin: p.is_admin,
            player_id: p.player_id,
            last_updated: now,
        }).collect();

        // Build tracked squads
        let tracked_squads: Vec<TrackedSquad> = squads.iter().map(|s| TrackedSquad {
            squad_id: s.squad_id.clone(),
            squad_name: s.name.clone(),
            team_id: s.team_id,
            team_name: format!("队伍 {}", s.team_id),
            size: 0, // Will be calculated
            squad_leader: if s.creator.is_empty() { None } else { Some(s.creator.clone()) },
        }).collect();

        // Build tracked teams
        let mut team_map: HashMap<i32, TrackedTeam> = HashMap::new();
        for p in &tracked_players {
            let entry = team_map.entry(p.team_id).or_insert_with(|| TrackedTeam {
                team_id: p.team_id,
                team_name: p.team_name.clone(),
                faction: String::new(),
                player_count: 0,
            });
            entry.player_count += 1;
        }
        let tracked_teams: Vec<TrackedTeam> = team_map.into_values().collect();

        // Update squad sizes
        let tracked_squads: Vec<TrackedSquad> = tracked_squads.into_iter().map(|mut s| {
            s.size = tracked_players.iter().filter(|p| p.squad_id.as_deref() == Some(&s.squad_id)).count();
            s
        }).collect();

        let state = ServerPlayerState {
            player_count: tracked_players.len(),
            max_players: 80,
            players: tracked_players,
            teams: tracked_teams,
            squads: tracked_squads,
            map_name,
            game_mode,
            last_refresh: now,
            refresh_error: None,
        };

        let mut states = self.states.write().await;
        states.insert(server_id, state);

        // Publish PlayerListUpdated event
        if let Some(ref em) = self.event_manager {
            let s = states.get(&server_id).unwrap();
            em.publish(GameEvent {
                player_list_updated: Some(PlayerListUpdatedData {
                    player_count: s.player_count,
                    team_count: s.teams.len(),
                    squad_count: s.squads.len(),
                    map_name: s.map_name.clone(),
                    timestamp: now,
                }),
                ..GameEvent::new(server_id, EventType::PlayerListUpdated)
            });
        }
    }
}

// ═══ Match Summary (增强版比赛追踪) ═══

#[derive(Debug, Clone, Serialize)]
pub struct MatchSummary {
    pub match_id: String,
    pub server_id: i32,
    pub chain_id: Option<String>,     // Links consecutive matches
    pub map_name: String,
    pub layer_name: String,
    pub team1_faction: String,
    pub team2_faction: String,
    pub winner_team: Option<i32>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ended_at: chrono::DateTime<chrono::Utc>,
    pub duration_minutes: Option<i32>,
    pub player_count: i32,
    pub kill_count: i32,
    pub teamkill_count: i32,
}

/// Build a match chain ID from server_id + timestamp prefix
pub fn build_chain_id(server_id: i32, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    format!("srv{}-{}", server_id, timestamp.format("%Y%m%d"))
}

/// Query match summary for a server
pub async fn get_match_summaries(
    pool: &PgPool,
    server_id: i32,
    page: i64,
    per_page: i64,
) -> Result<(Vec<MatchSummary>, i64), sqlx::Error> {
    let offset = (page - 1) * per_page;

    let (total,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM match_info WHERE server_id=$1"
    ).bind(server_id).fetch_one(pool).await?;

    let rows = sqlx::query_as::<_, (i32, String, String, String, String, Option<i32>, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, map_name, layer_name, team1_faction, team2_faction, winner_team, event_type, logged_at \
         FROM match_info WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    ).bind(server_id).bind(per_page).bind(offset).fetch_all(pool).await?;

    let mut summaries = Vec::new();
    for (id, map_name, layer_name, t1, t2, winner, event_type, logged_at) in rows {
        // Get kill/tk stats for this match
        let (kill_count,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM kill_events WHERE server_id=$1 AND is_kill=true AND logged_at <= $2 \
             AND logged_at >= $2 - INTERVAL '2 hours'"
        ).bind(server_id).bind(logged_at).fetch_one(pool).await.unwrap_or((0,));

        let (tk_count,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM kill_events WHERE server_id=$1 AND is_teamkill=true AND logged_at <= $2 \
             AND logged_at >= $2 - INTERVAL '2 hours'"
        ).bind(server_id).bind(logged_at).fetch_one(pool).await.unwrap_or((0,));

        // Get player count for this match
        let (player_count,) = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(DISTINCT player_name) FROM kill_events WHERE server_id=$1 \
             AND logged_at <= $2 AND logged_at >= $2 - INTERVAL '2 hours'"
        ).bind(server_id).bind(logged_at).fetch_one(pool).await.unwrap_or((0,));

        // Calculate duration from previous match
        let started_at = sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>,)>(
            "SELECT logged_at FROM match_info WHERE server_id=$1 AND logged_at < $2 ORDER BY logged_at DESC LIMIT 1"
        ).bind(server_id).bind(logged_at).fetch_optional(pool).await.ok().flatten()
            .map(|(t,)| t);

        let duration = started_at.as_ref().map(|start| {
            (logged_at - *start).num_minutes() as i32
        });

        let chain_id = build_chain_id(server_id, &logged_at);

        // Check if previous chain ID matches (continuation)
        let chain_id = if let Some(ref start) = started_at {
            let prev_chain = build_chain_id(server_id, start);
            if prev_chain == chain_id {
                Some(chain_id)
            } else {
                Some(format!("{}", id))
            }
        } else {
            Some(format!("{}", id))
        };

        summaries.push(MatchSummary {
            match_id: format!("{}", id),
            server_id,
            chain_id,
            map_name,
            layer_name,
            team1_faction: t1,
            team2_faction: t2,
            winner_team: winner,
            started_at,
            ended_at: logged_at,
            duration_minutes: duration,
            player_count: player_count as i32,
            kill_count: kill_count as i32,
            teamkill_count: tk_count as i32,
        });
    }

    Ok((summaries, total))
}

/// Get per-match player stats
#[derive(Debug, Clone, Serialize)]
pub struct MatchPlayerStat {
    pub player_name: String,
    pub steam64: String,
    pub kills: i32,
    pub deaths: i32,
    pub teamkills: i32,
    pub revives: i32,
    pub damage_dealt: f64,
    pub damage_taken: f64,
    pub kd_ratio: f64,
}

pub async fn get_match_player_stats(
    pool: &PgPool,
    server_id: i32,
    match_id: i32,
) -> Result<Vec<MatchPlayerStat>, sqlx::Error> {
    // Get match time range
    let match_time = match sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>,)>(
        "SELECT logged_at FROM match_info WHERE id=$1 AND server_id=$2"
    ).bind(match_id).bind(server_id).fetch_optional(pool).await? {
        Some((t,)) => t,
        None => return Ok(vec![]),
    };

    let start = match_time - chrono::Duration::hours(2);

    let rows = sqlx::query_as::<_, (String, String, Option<i64>, Option<i64>, Option<i64>, Option<f64>, Option<f64>)>(
        "SELECT attacker_name, attacker_steam64, \
         COUNT(*) FILTER (WHERE is_kill=true) AS kills, \
         COUNT(*) FILTER (WHERE is_kill=false) AS hits, \
         COUNT(*) FILTER (WHERE is_teamkill=true) AS teamkills, \
         SUM(damage) FILTER (WHERE is_kill=true OR is_teamkill=true) AS damage_dealt, \
         0::float8 AS damage_taken \
         FROM kill_events \
         WHERE server_id=$1 AND logged_at BETWEEN $2 AND $3 AND attacker_name != '' \
         GROUP BY attacker_name, attacker_steam64 \
         ORDER BY kills DESC NULLS LAST LIMIT 100"
    ).bind(server_id).bind(start).bind(match_time).fetch_all(pool).await?;

    let stats: Vec<MatchPlayerStat> = rows.into_iter().map(|(name, steam, kills, hits, tks, dmg_dealt, _dmg_taken)| {
        let k = kills.unwrap_or(0) as f64;
        let d = hits.unwrap_or(0) as f64;
        let kd = if d > 0.0 { k / d } else if k > 0.0 { k } else { 0.0 };
        MatchPlayerStat {
            player_name: name,
            steam64: steam,
            kills: kills.unwrap_or(0) as i32,
            deaths: (hits.unwrap_or(0) - kills.unwrap_or(0)) as i32,
            teamkills: tks.unwrap_or(0) as i32,
            revives: 0,
            damage_dealt: dmg_dealt.unwrap_or(0.0),
            damage_taken: 0.0,
            kd_ratio: kd,
        }
    }).collect();

    Ok(stats)
}
