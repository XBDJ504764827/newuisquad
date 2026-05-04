use sqlx::PgPool;
use crate::models::team_switch::{TeamSwitchConfig, UpdateTeamSwitchConfigRequest};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<TeamSwitchConfig, sqlx::Error> {
    let existing = sqlx::query_as::<_, TeamSwitchConfig>(
        "SELECT * FROM team_switch_config WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await?;
    if let Some(s) = existing { return Ok(s); }
    sqlx::query_as::<_, TeamSwitchConfig>(
        "INSERT INTO team_switch_config (server_id) VALUES ($1) RETURNING *"
    ).bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateTeamSwitchConfigRequest) -> Result<TeamSwitchConfig, sqlx::Error> {
    let current = get_or_create(pool, server_id).await?;
    let enabled = req.enabled.unwrap_or(current.enabled);
    sqlx::query_as::<_, TeamSwitchConfig>(
        "UPDATE team_switch_config SET enabled=$1, updated_at=NOW() WHERE server_id=$2 RETURNING *"
    )
    .bind(enabled)
    .bind(server_id)
    .fetch_one(pool).await
}
