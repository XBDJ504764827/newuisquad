use sqlx::PgPool;
use crate::models::team_settings::{TeamSettings, UpdateTeamSettings};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<TeamSettings, sqlx::Error> {
    let e = sqlx::query_as::<_, TeamSettings>("SELECT * FROM team_settings WHERE server_id=$1")
        .bind(server_id).fetch_optional(pool).await?;
    if let Some(s) = e { return Ok(s); }
    sqlx::query_as::<_, TeamSettings>("INSERT INTO team_settings (server_id) VALUES ($1) RETURNING *")
        .bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateTeamSettings) -> Result<TeamSettings, sqlx::Error> {
    let c = get_or_create(pool, server_id).await?;
    sqlx::query_as::<_, TeamSettings>(
        "UPDATE team_settings SET create_team_broadcast=$1,captain_time_check=$2,captain_min_playtime=$3,captain_check_min_players=$4,max_create_team_attempts=$5,updated_at=NOW() WHERE server_id=$6 RETURNING *"
    )
    .bind(req.create_team_broadcast.unwrap_or(c.create_team_broadcast))
    .bind(req.captain_time_check.unwrap_or(c.captain_time_check))
    .bind(req.captain_min_playtime.unwrap_or(c.captain_min_playtime))
    .bind(req.captain_check_min_players.unwrap_or(c.captain_check_min_players))
    .bind(req.max_create_team_attempts.unwrap_or(c.max_create_team_attempts))
    .bind(server_id).fetch_one(pool).await
}
