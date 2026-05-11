use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use sqlx::PgPool;
use tokio::sync::RwLock;
use serde::Serialize;

use crate::rcon_client::pool::RconPool;

// ═══ Server Health Status ═══

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Online,
    Degraded,  // RCON OK but agent disconnected
    Offline,   // RCON unreachable
    Unknown,
}

impl HealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Online => "online",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Offline => "offline",
            HealthStatus::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerHealth {
    pub server_id: i32,
    pub rcon_healthy: bool,
    pub agent_connected: bool,
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub last_rcon_error: Option<String>,
    pub player_count: Option<usize>,
    pub map_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnhancedServerInfo {
    pub id: i32,
    pub server_id: String,
    pub name: String,
    pub ip: String,
    pub rcon_port: i32,
    pub health: ServerHealth,
    pub stats_24h: Option<ServerStats24h>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerStats24h {
    pub player_count: i64,
    pub error_count: i64,
    pub warn_count: i64,
    pub match_count: i64,
    pub kill_count: i64,
    pub teamkill_count: i64,
    pub chat_count: i64,
}

// ═══ Server Monitor Service ═══

pub struct ServerMonitor {
    pool: PgPool,
    rcon_pool: RconPool,
    health_states: Arc<RwLock<HashMap<i32, ServerHealth>>>,
}

impl ServerMonitor {
    pub fn new(pool: PgPool, rcon_pool: RconPool) -> Self {
        Self {
            pool,
            rcon_pool,
            health_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn health_states(&self) -> Arc<RwLock<HashMap<i32, ServerHealth>>> {
        self.health_states.clone()
    }

    /// Start periodic health checking (every 60 seconds)
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            tracing::info!("ServerMonitor 服务已启动");
            // Initial check after 5 seconds
            tokio::time::sleep(Duration::from_secs(5)).await;
            loop {
                self.check_all_servers().await;
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        })
    }

    /// Check all servers
    async fn check_all_servers(&self) {
        let servers = match sqlx::query_as::<_, (i32,)>(
            "SELECT id FROM servers ORDER BY id"
        ).fetch_all(&self.pool).await {
            Ok(s) => s,
            Err(_) => return,
        };

        for (id,) in &servers {
            let health = self.check_server(*id).await;
            let mut states = self.health_states.write().await;
            states.insert(*id, health);
        }
    }

    /// Check a single server's health
    async fn check_server(&self, server_id: i32) -> ServerHealth {
        let now = chrono::Utc::now();
        // 通过 execute_by_server_id 测试连接健康
        let rcon_healthy = self.rcon_pool.execute_by_server_id(server_id, "ListPlayers").await.is_ok();

        let (player_count, map_name) = if rcon_healthy {
            self.get_live_info(server_id).await
        } else {
            (None, None)
        };

        let agent_connected = self.is_agent_connected(server_id).await;

        let status = if rcon_healthy && agent_connected {
            HealthStatus::Online
        } else if rcon_healthy {
            HealthStatus::Degraded
        } else {
            HealthStatus::Offline
        };

        ServerHealth {
            server_id,
            rcon_healthy,
            agent_connected,
            status,
            last_check: now,
            last_rcon_error: if rcon_healthy { None } else { Some("RCON 不可达".to_string()) },
            player_count,
            map_name,
        }
    }

    async fn get_live_info(&self, server_id: i32) -> (Option<usize>, Option<String>) {
        if let Ok(raw) = self.rcon_pool.execute_by_server_id(server_id, "ListPlayers").await {
            let player_count = raw.lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with("-----") && !l.starts_with("Active"))
                .count();
            return (Some(player_count), None);
        }
        (None, None)
    }

    async fn is_agent_connected(&self, server_id: i32) -> bool {
        // Check server_states via a separate approach
        // The agent pool is not directly available here, but we can check
        // if the server has received state updates recently
        // For now, report as connected if RCON is healthy
        true
    }

    /// Get health for a specific server
    pub async fn get_health(&self, server_id: i32) -> ServerHealth {
        let states = self.health_states.read().await;
        states.get(&server_id).cloned().unwrap_or(ServerHealth {
            server_id,
            rcon_healthy: false,
            agent_connected: false,
            status: HealthStatus::Unknown,
            last_check: chrono::Utc::now(),
            last_rcon_error: Some("尚未检查".to_string()),
            player_count: None,
            map_name: None,
        })
    }

    /// Get all health states
    pub async fn get_all_health(&self) -> HashMap<i32, ServerHealth> {
        let states = self.health_states.read().await;
        states.clone()
    }
}

// ═══ Enhanced Server List (with health + stats) ═══

pub async fn get_enhanced_server_list(
    pool: &PgPool,
    monitor: &ServerMonitor,
    player_tracker: Option<&crate::services::player_tracker::PlayerTracker>,
) -> Result<Vec<EnhancedServerInfo>, sqlx::Error> {
    let servers = sqlx::query_as::<_, (i32, String, String, String, i32, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, server_id, name, ip, rcon_port, created_at FROM servers ORDER BY id"
    ).fetch_all(pool).await?;

    let health_states = monitor.get_all_health().await;

    let mut results = Vec::new();
    for (id, server_id, name, ip, rcon_port, created_at) in servers {
        let health = health_states.get(&id).cloned().unwrap_or(ServerHealth {
            server_id: id,
            rcon_healthy: false,
            agent_connected: false,
            status: HealthStatus::Unknown,
            last_check: chrono::Utc::now(),
            last_rcon_error: None,
            player_count: None,
            map_name: None,
        });

        // Get 24h stats
        let stats = get_24h_stats(pool, id).await.ok();

        // Get live player count from tracker if available
        let mut health_with_players = health.clone();
        if let Some(ref pt) = player_tracker {
            if let Some(state) = pt.get_state(id).await {
                health_with_players.player_count = Some(state.player_count);
                if !state.map_name.is_empty() && state.map_name != "Unknown" {
                    health_with_players.map_name = Some(state.map_name.clone());
                }
            }
        }

        results.push(EnhancedServerInfo {
            id,
            server_id,
            name,
            ip,
            rcon_port,
            health: health_with_players,
            stats_24h: stats,
            created_at,
        });
    }

    Ok(results)
}

pub async fn get_24h_stats(pool: &PgPool, server_id: i32) -> Result<ServerStats24h, sqlx::Error> {
    let (player_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT steam64) FROM player_info WHERE server_id=$1 AND last_seen >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await?;

    let (error_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM server_logs WHERE server_id=$1 AND log_level='ERROR' AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await?;

    let (warn_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM server_logs WHERE server_id=$1 AND log_level='WARN' AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await?;

    let (match_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM match_info WHERE server_id=$1 AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await?;

    let (kill_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM kill_events WHERE server_id=$1 AND is_kill=true AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await?;

    let (tk_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM kill_events WHERE server_id=$1 AND is_teamkill=true AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await?;

    let (chat_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM chat_messages WHERE server_id=$1 AND logged_at >= NOW() - INTERVAL '24 hours'"
    ).bind(server_id).fetch_one(pool).await?;

    Ok(ServerStats24h {
        player_count,
        error_count,
        warn_count,
        match_count,
        kill_count,
        teamkill_count: tk_count,
        chat_count,
    })
}
