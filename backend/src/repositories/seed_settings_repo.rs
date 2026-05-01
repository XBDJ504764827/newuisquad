use sqlx::PgPool;
use crate::models::seed_settings::{SeedSettings, UpdateSeedSettings};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<SeedSettings, sqlx::Error> {
    let e = sqlx::query_as::<_, SeedSettings>("SELECT * FROM seed_settings WHERE server_id=$1")
        .bind(server_id).fetch_optional(pool).await?;
    if let Some(s) = e { return Ok(s); }
    sqlx::query_as::<_, SeedSettings>("INSERT INTO seed_settings (server_id) VALUES ($1) RETURNING *")
        .bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateSeedSettings) -> Result<SeedSettings, sqlx::Error> {
    let c = get_or_create(pool, server_id).await?;
    sqlx::query_as::<_, SeedSettings>(
        "UPDATE seed_settings SET enabled=$1,player_threshold=$2,vehicle_claim=$3,vehicle_fill=$4,deploy_restrict=$5,kit_restrict=$6,heavy_vehicle_require=$7,respawn_timer=$8,use_enemy_vehicle=$9,updated_at=NOW() WHERE server_id=$10 RETURNING *"
    )
    .bind(req.enabled.unwrap_or(c.enabled)).bind(req.player_threshold.unwrap_or(c.player_threshold))
    .bind(req.vehicle_claim.unwrap_or(c.vehicle_claim)).bind(req.vehicle_fill.unwrap_or(c.vehicle_fill))
    .bind(req.deploy_restrict.unwrap_or(c.deploy_restrict)).bind(req.kit_restrict.unwrap_or(c.kit_restrict))
    .bind(req.heavy_vehicle_require.unwrap_or(c.heavy_vehicle_require)).bind(req.respawn_timer.unwrap_or(c.respawn_timer))
    .bind(req.use_enemy_vehicle.unwrap_or(c.use_enemy_vehicle)).bind(server_id).fetch_one(pool).await
}
