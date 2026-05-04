use sqlx::PgPool;
use crate::models::team_switch::{TeamSwitchConfig, UpdateTeamSwitchConfigRequest};
use crate::repositories::team_switch_repo;

pub async fn get_config(pool: &PgPool, server_id: i32) -> Result<TeamSwitchConfig, sqlx::Error> {
    team_switch_repo::get_or_create(pool, server_id).await
}

pub async fn update_config(pool: &PgPool, server_id: i32, req: UpdateTeamSwitchConfigRequest) -> Result<TeamSwitchConfig, sqlx::Error> {
    team_switch_repo::update(pool, server_id, &req).await
}
