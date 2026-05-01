use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamSettings {
    pub id: i32, pub server_id: i32,
    pub create_team_broadcast: bool,
    pub captain_time_check: bool,
    pub captain_min_playtime: i32,
    pub captain_check_min_players: i32,
    pub max_create_team_attempts: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamSettings {
    pub create_team_broadcast: Option<bool>,
    pub captain_time_check: Option<bool>,
    pub captain_min_playtime: Option<i32>,
    pub captain_check_min_players: Option<i32>,
    pub max_create_team_attempts: Option<i32>,
}
