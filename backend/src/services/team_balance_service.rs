use std::collections::HashMap;
use std::sync::Arc;
use serde::Serialize;
use sqlx::PgPool;

use crate::rcon_client::pool::RconPool;

/// Team balancer configuration
#[derive(Debug, Clone, Serialize)]
pub struct TeamBalanceConfig {
    pub enabled: bool,
    pub max_win_streak: i32,
    pub enable_single_round_scramble: bool,
    pub single_round_scramble_threshold: i32,
    pub min_tickets_dominant_win: i32,
    pub scramble_announcement_delay: i32,
    pub scramble_percentage: f64,
    pub warn_on_swap: bool,
}

impl Default for TeamBalanceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_win_streak: 2,
            enable_single_round_scramble: false,
            single_round_scramble_threshold: 250,
            min_tickets_dominant_win: 150,
            scramble_announcement_delay: 30,
            scramble_percentage: 0.5,
            warn_on_swap: true,
        }
    }
}

/// Per-server team balance state
#[derive(Debug, Clone)]
pub struct TeamBalanceState {
    pub config: TeamBalanceConfig,
    pub win_streak_team: Option<i32>, // team_id on winning streak
    pub win_streak_count: i32,
    pub last_scramble: Option<chrono::DateTime<chrono::Utc>>,
    pub scramble_in_progress: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScrambleResult {
    pub success: bool,
    pub players_moved: i32,
    pub message: String,
}

/// Team Balancer Service — win streak tracking and auto-scramble
pub struct TeamBalanceService {
    pool: PgPool,
    rcon_pool: RconPool,
    states: Arc<tokio::sync::RwLock<HashMap<i32, TeamBalanceState>>>,
}

impl TeamBalanceService {
    pub fn new(pool: PgPool, rcon_pool: RconPool) -> Self {
        Self {
            pool,
            rcon_pool,
            states: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub fn states(&self) -> Arc<tokio::sync::RwLock<HashMap<i32, TeamBalanceState>>> {
        self.states.clone()
    }

    /// Load config for a server (auto-creates default if not configured)
    pub async fn load_config(&self, server_id: i32) -> TeamBalanceConfig {
        // Read from team_balance_config table (fallback to defaults)
        let row = sqlx::query_as::<_, (bool,)>("SELECT enabled FROM team_switch_config WHERE server_id=$1")
            .bind(server_id).fetch_optional(&self.pool).await.ok().flatten();

        let mut config = TeamBalanceConfig::default();
        if let Some((enabled,)) = row {
            config.enabled = enabled;
        }
        config
    }

    /// Initialize state for a server
    pub async fn init_server(&self, server_id: i32) {
        let config = self.load_config(server_id).await;
        let mut states = self.states.write().await;
        states.entry(server_id).or_insert_with(|| TeamBalanceState {
            config,
            win_streak_team: None,
            win_streak_count: 0,
            last_scramble: None,
            scramble_in_progress: false,
        });
    }

    /// Handle round ended event — track win streak
    pub async fn on_round_ended(&self, server_id: i32, winner_team: Option<i32>, tickets_gap: i32, is_invasion: bool) {
        let mut states = self.states.write().await;
        let state = match states.get_mut(&server_id) {
            Some(s) => s,
            None => return,
        };

        if !state.config.enabled {
            return;
        }

        let winner = match winner_team {
            Some(t) if t > 0 => t,
            _ => return,
        };

        // Determine if dominant win
        let threshold = if is_invasion {
            // Invasion: much larger gap expected
            state.config.min_tickets_dominant_win * 3
        } else {
            state.config.min_tickets_dominant_win
        };

        if tickets_gap < threshold {
            // Not dominant — reset streak
            state.win_streak_team = None;
            state.win_streak_count = 0;
            return;
        }

        // Check single-round mercy scramble
        if state.config.enable_single_round_scramble
            && tickets_gap >= state.config.single_round_scramble_threshold
        {
            tracing::info!(server_id, winner, tickets_gap, "触发单局碾压重组");
            state.win_streak_count = state.config.max_win_streak; // Force scramble
        }

        // Update streak
        if state.win_streak_team == Some(winner) {
            state.win_streak_count += 1;
        } else {
            state.win_streak_team = Some(winner);
            state.win_streak_count = 1;
        }

        tracing::info!(server_id, winner, streak = state.win_streak_count, tickets_gap, "连胜追踪");

        // Trigger scramble if streak exceeds max
        if state.win_streak_count >= state.config.max_win_streak {
            // Drop write lock before scramble (which needs RCON)
            drop(states);
            self.execute_scramble(server_id).await;
        }
    }

    /// Execute team scramble
    pub async fn execute_scramble(&self, server_id: i32) -> ScrambleResult {
        let creds = match sqlx::query_as::<_, (String, i32, String)>(
            "SELECT ip, rcon_port, rcon_password FROM servers WHERE id=$1"
        ).bind(server_id).fetch_optional(&self.pool).await.ok().flatten() {
            Some(c) => c,
            None => return ScrambleResult { success: false, players_moved: 0, message: "服务器不存在".into() },
        };
        let (ip, port, password) = creds;

        // Announce scramble
        let _ = self.rcon_pool.execute(&ip, port as u16, &password,
            "AdminBroadcast 阵营洗牌即将开始！系统检测到实力不均衡，将在15秒后重组队伍...").await;

        // Count down
        for remaining in [10, 5, 3].iter() {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let _ = self.rcon_pool.execute(&ip, port as u16, &password,
                &format!("AdminBroadcast 阵营洗牌倒计时 {} 秒...", remaining)).await;
        }

        // Get players via RCON
        let players_raw = match self.rcon_pool.execute(&ip, port as u16, &password, "ListPlayers").await {
            Ok(r) => r,
            Err(e) => return ScrambleResult { success: false, players_moved: 0, message: format!("获取玩家列表失败: {}", e) },
        };

        // Parse players: gather team 1 and team 2 player IDs
        let mut team1: Vec<(i32, String)> = Vec::new();
        let mut team2: Vec<(i32, String)> = Vec::new();

        for line in players_raw.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("-----") || line.starts_with("Active") {
                continue;
            }
            let mut pid = 0i32;
            let mut name = String::new();
            let mut tid = 0i32;
            for part in line.split('|') {
                let part = part.trim();
                if let Some(v) = part.strip_prefix("ID: ") { pid = v.parse().unwrap_or(0); }
                else if let Some(v) = part.strip_prefix("Name: ") { name = v.to_string(); }
                else if let Some(v) = part.strip_prefix("Team ID: ") { tid = v.parse().unwrap_or(0); }
            }
            if pid > 0 && tid > 0 {
                match tid {
                    1 => team1.push((pid, name)),
                    2 => team2.push((pid, name)),
                    _ => {}
                }
            }
        }

        // Select players to swap (half of the smaller team rounded up)
        let swap_count = (team1.len().min(team2.len()) as f64 * 0.5).ceil() as usize;
        if swap_count == 0 {
            let _ = self.rcon_pool.execute(&ip, port as u16, &password,
                "AdminBroadcast 阵营洗牌失败：玩家不足").await;
            return ScrambleResult { success: false, players_moved: 0, message: "玩家不足".into() };
        }

        let team1_swap: Vec<_> = team1.iter().take(swap_count).collect();
        let team2_swap: Vec<_> = team2.iter().take(swap_count).collect();

        // Execute swaps: team1 → team2, team2 → team1
        let mut moved = 0i32;
        for (pid, _name) in team1_swap.iter().chain(team2_swap.iter()) {
            let cmd = format!("AdminForceTeamChangeById {}", pid);
            if self.rcon_pool.execute(&ip, port as u16, &password, &cmd).await.is_ok() {
                moved += 1;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        // Broadcast result
        let result_msg = format!("阵营洗牌完成！已调整 {} 名玩家", moved);
        let _ = self.rcon_pool.execute(&ip, port as u16, &password,
            &format!("AdminBroadcast {}", result_msg)).await;

        // Reset state
        let mut states = self.states.write().await;
        if let Some(state) = states.get_mut(&server_id) {
            state.win_streak_team = None;
            state.win_streak_count = 0;
            state.last_scramble = Some(chrono::Utc::now());
            state.scramble_in_progress = false;
        }

        tracing::info!(server_id, moved, "阵营洗牌完成");
        ScrambleResult { success: true, players_moved: moved, message: result_msg }
    }

    /// Get state for a server
    pub async fn get_state(&self, server_id: i32) -> Option<TeamBalanceState> {
        let states = self.states.read().await;
        states.get(&server_id).cloned()
    }

    /// Manual trigger
    pub async fn manual_scramble(&self, server_id: i32) -> ScrambleResult {
        self.execute_scramble(server_id).await
    }
}
