use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::broadcast;

use crate::rcon_client::pool::RconPool;
use crate::services::event_manager::{GameEvent, EventType};

/// Ban Enforcer — automatically kicks banned players on connection
pub struct BanEnforcer {
    pool: PgPool,
    rcon_pool: RconPool,
}

impl BanEnforcer {
    pub fn new(pool: PgPool, rcon_pool: RconPool) -> Self {
        Self { pool, rcon_pool }
    }

    /// Start listening for player connection events
    pub fn start(self: Arc<Self>, mut event_rx: broadcast::Receiver<GameEvent>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            tracing::info!("BanEnforcer 已启动，等待玩家连接事件...");
            loop {
                match event_rx.recv().await {
                    Ok(event) => {
                        if event.event_type == EventType::PlayerConnected {
                            if let Some(ref data) = event.player_connected {
                                self.handle_connection(event.server_id, data).await;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "BanEnforcer 事件滞后");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            tracing::info!("BanEnforcer 已停止");
        })
    }

    async fn handle_connection(&self, server_id: i32, data: &crate::services::event_manager::PlayerConnectedData) {
        let steam_id = &data.steam_id;
        if steam_id.len() < 10 { return; }

        // Check if this player has an active ban
        let ban = sqlx::query_as::<_, (i32, String, String)>(
            "SELECT duration, reason, player_name FROM bans WHERE server_id=$1 AND steam_id=$2"
        ).bind(server_id).bind(steam_id).fetch_optional(&self.pool).await;

        let (duration, reason, player_name) = match ban {
            Ok(Some(b)) => b,
            _ => return, // Not banned
        };

        // Check if ban is active (0 = permanent, >0 = minutes remaining check)
        if duration < 0 { return; } // Expired

        // Get server RCON credentials
        let creds = match sqlx::query_as::<_, (String, i32, String)>(
            "SELECT ip, rcon_port, rcon_password FROM servers WHERE id=$1"
        ).bind(server_id).fetch_optional(&self.pool).await {
            Ok(Some(c)) => c,
            _ => return,
        };
        let (ip, port, password) = creds;

        let duration_str = if duration == 0 { "永久封禁".to_string() } else { format!("{}分钟", duration) };
        let kick_reason = format!("您已被封禁 ({}): {}", duration_str, reason);

        // Kick the player
        let kick_cmd = format!("AdminKick \"{}\" \"{}\"", steam_id, kick_reason);
        match self.rcon_pool.execute(&ip, port as u16, &password, &kick_cmd).await {
            Ok(_) => {
                tracing::info!(server_id, steam_id, player = %data.player_name, ban_reason = %reason, "BanEnforcer 已踢出被封玩家");
            }
            Err(e) => {
                tracing::warn!(server_id, steam_id, error = %e, "BanEnforcer 踢出失败");
            }
        }
    }
}
