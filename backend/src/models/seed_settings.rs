use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeedSettings {
    pub id: i32, pub server_id: i32, pub enabled: bool, pub player_threshold: i32,
    pub vehicle_claim: bool, pub vehicle_fill: bool, pub deploy_restrict: bool,
    pub kit_restrict: bool, pub heavy_vehicle_require: bool,
    pub respawn_timer: bool, pub use_enemy_vehicle: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSeedSettings {
    pub enabled: Option<bool>, pub player_threshold: Option<i32>,
    pub vehicle_claim: Option<bool>, pub vehicle_fill: Option<bool>,
    pub deploy_restrict: Option<bool>, pub kit_restrict: Option<bool>,
    pub heavy_vehicle_require: Option<bool>, pub respawn_timer: Option<bool>,
    pub use_enemy_vehicle: Option<bool>,
}
