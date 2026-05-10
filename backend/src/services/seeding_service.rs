use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use sqlx::PgPool;
use serde::Serialize;

use crate::rcon_client::pool::RconPool;
use crate::models::seed_settings::SeedSettings;

/// Seeding mode state per server
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SeedMode {
    Seeding,
    Live,
    Waiting, // Waiting for new game to start
}

#[derive(Debug, Clone)]
struct ServerSeedState {
    mode: SeedMode,
    last_broadcast: chrono::DateTime<chrono::Utc>,
    new_game_pending: bool,
}

/// Seeding mode service — auto-switches between seeding/live based on player count
pub struct SeedModeService {
    pool: PgPool,
    rcon_pool: RconPool,
    states: Arc<tokio::sync::RwLock<HashMap<i32, ServerSeedState>>>,
}

impl SeedModeService {
    pub fn new(pool: PgPool, rcon_pool: RconPool) -> Self {
        Self {
            pool,
            rcon_pool,
            states: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub fn states(&self) -> Arc<tokio::sync::RwLock<HashMap<i32, ServerSeedState>>> {
        self.states.clone()
    }

    /// Start the seeding mode background service (runs every 30 seconds)
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            tracing::info!("SeedMode 服务已启动");
            tokio::time::sleep(Duration::from_secs(10)).await;
            loop {
                self.tick().await;
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        })
    }

    async fn tick(&self) {
        let servers = match sqlx::query_as::<_, (i32, String, i32, String)>(
            "SELECT id, ip, rcon_port, rcon_password FROM servers WHERE rcon_port > 0 AND rcon_password != ''"
        ).fetch_all(&self.pool).await {
            Ok(s) => s,
            Err(_) => return,
        };

        for (server_id, ip, rcon_port, password) in servers {
            if let Err(e) = self.process_server(server_id, &ip, rcon_port as u16, &password).await {
                tracing::warn!(server_id, error = %e, "播种模式处理失败");
            }
        }
    }

    async fn process_server(&self, server_id: i32, ip: &str, port: u16, password: &str) -> Result<(), String> {
        let settings = match sqlx::query_as::<_, (bool, i32, bool, bool, bool, bool, bool, bool, bool)>(
            "SELECT enabled, player_threshold, vehicle_claim, respawn_timer, \
             deploy_restrict, kit_restrict, heavy_vehicle_require, use_enemy_vehicle, vehicle_fill \
             FROM seed_settings WHERE server_id=$1"
        ).bind(server_id).fetch_optional(&self.pool).await.map_err(|e| e.to_string())? {
            Some((enabled, threshold, vc, rt, dr, kr, hvr, uev, vf)) => {
                if !enabled { return Ok(()); }
                SeedSettings {
                    id: 0, server_id, enabled, player_threshold: threshold,
                    vehicle_claim: vc, vehicle_fill: vf, deploy_restrict: dr,
                    kit_restrict: kr, heavy_vehicle_require: hvr,
                    respawn_timer: rt, use_enemy_vehicle: uev,
                    updated_at: chrono::Utc::now(),
                }
            }
            None => return Ok(()),
        };

        // Get current player count
        let player_count = match self.rcon_pool.execute(ip, port, password, "ListPlayers").await {
            Ok(raw) => {
                raw.lines().filter(|l| {
                    let t = l.trim();
                    !t.is_empty() && !t.starts_with("-----") && !t.starts_with("Active")
                }).count()
            }
            Err(_) => return Ok(()),
        };

        let mut states = self.states.write().await;
        let state = states.entry(server_id).or_insert_with(|| ServerSeedState {
            mode: SeedMode::Waiting,
            last_broadcast: chrono::Utc::now() - chrono::Duration::minutes(5),
            new_game_pending: false,
        });

        let threshold = settings.player_threshold as usize;
        let new_mode = if player_count < threshold {
            SeedMode::Seeding
        } else {
            SeedMode::Live
        };

        // Mode transition
        if state.mode != new_mode {
            tracing::info!(server_id, from = ?state.mode, to = ?new_mode, player_count, threshold, "播种模式切换");
            state.mode = new_mode.clone();

            match new_mode {
                SeedMode::Seeding => self.apply_seeding_rules(ip, port, password, &settings).await?,
                SeedMode::Live => self.revert_seeding_rules(ip, port, password, &settings).await?,
                SeedMode::Waiting => {}
            }

            // Broadcast mode change
            let msg = match new_mode {
                SeedMode::Seeding => format!("暖服规则已生效！当前 {} 人，满 {} 人后自动切换正式模式", player_count, threshold),
                SeedMode::Live => "正式开局！暖服规则已取消，祝各位游戏愉快！".to_string(),
                SeedMode::Waiting => String::new(),
            };
            if !msg.is_empty() {
                let _ = self.rcon_pool.execute(ip, port, password, &format!("AdminBroadcast {}", msg)).await;
            }
        }

        // Periodic broadcast during seeding
        if state.mode == SeedMode::Seeding {
            let now = chrono::Utc::now();
            let since_last = (now - state.last_broadcast).num_seconds();
            if since_last >= 150 {
                // Broadcast every 2.5 minutes
                let msg = format!("暖服模式进行中 — 当前 {} 人，满 {} 人后自动切换正式模式", player_count, threshold);
                if let Err(e) = self.rcon_pool.execute(ip, port, password, &format!("AdminBroadcast {}", msg)).await {
                    tracing::warn!(server_id, error = %e, "播种消息广播失败");
                }
                state.last_broadcast = now;
            }
        }

        Ok(())
    }

    async fn apply_seeding_rules(&self, ip: &str, port: u16, password: &str, settings: &SeedSettings) -> Result<(), String> {
        let cmds: Vec<Option<&str>> = vec![
            if settings.respawn_timer { Some("AdminNoRespawnTimer 1") } else { None },
            if settings.vehicle_claim { Some("AdminDisableVehicleClaiming 1") } else { None },
            if settings.deploy_restrict { Some("AdminForceAllDeployableAvailability 1") } else { None },
            if settings.kit_restrict { Some("AdminForceAllRoleAvailability 1") } else { None },
            if settings.heavy_vehicle_require { Some("AdminDisableVehicleKitRequirement 1") } else { None },
            if settings.use_enemy_vehicle { Some("AdminDisableVehicleTeamRequirement 1") } else { None },
        ];

        for cmd in cmds.into_iter().flatten() {
            let _ = self.rcon_pool.execute(ip, port, password, cmd).await;
        }
        Ok(())
    }

    async fn revert_seeding_rules(&self, ip: &str, port: u16, password: &str, settings: &SeedSettings) -> Result<(), String> {
        let cmds: Vec<Option<&str>> = vec![
            if settings.respawn_timer { Some("AdminNoRespawnTimer 0") } else { None },
            if settings.vehicle_claim { Some("AdminDisableVehicleClaiming 0") } else { None },
            if settings.deploy_restrict { Some("AdminAlwaysValidPlacement 0") } else { None },
            if settings.kit_restrict { Some("AdminForceAllRoleAvailability 0") } else { None },
            if settings.heavy_vehicle_require { Some("AdminDisableVehicleKitRequirement 0") } else { None },
            if settings.use_enemy_vehicle { Some("AdminDisableVehicleTeamRequirement 0") } else { None },
        ];

        for cmd in cmds.into_iter().flatten() {
            let _ = self.rcon_pool.execute(ip, port, password, cmd).await;
        }
        Ok(())
    }

    /// Notify that a new game has started
    pub async fn on_new_game(&self, server_id: i32) {
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(&server_id) {
            state.new_game_pending = false;
        }
    }

    /// Get current seeding mode for a server
    pub async fn get_mode(&self, server_id: i32) -> Option<SeedMode> {
        let states = self.states.read().await;
        states.get(&server_id).map(|s| s.mode.clone())
    }
}
