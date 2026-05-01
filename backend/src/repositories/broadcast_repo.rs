use sqlx::PgPool;
use crate::models::broadcast_settings::{BroadcastSettings, UpdateBroadcastRequest};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<BroadcastSettings, sqlx::Error> {
    let e = sqlx::query_as::<_, BroadcastSettings>("SELECT * FROM broadcast_settings WHERE server_id = $1")
        .bind(server_id).fetch_optional(pool).await?;
    if let Some(s) = e { return Ok(s); }
    sqlx::query_as::<_, BroadcastSettings>("INSERT INTO broadcast_settings (server_id) VALUES ($1) RETURNING *")
        .bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateBroadcastRequest) -> Result<BroadcastSettings, sqlx::Error> {
    let c = get_or_create(pool, server_id).await?;
    sqlx::query_as::<_, BroadcastSettings>(
        "UPDATE broadcast_settings SET join_message_enabled=$1,join_message=$2,gameop_list_enabled=$3,gameop_list_message=$4,announcement_enabled=$5,announcement_content=$6,announcement_interval=$7,updated_at=NOW() WHERE server_id=$8 RETURNING *"
    )
    .bind(req.join_message_enabled.unwrap_or(c.join_message_enabled))
    .bind(req.join_message.as_deref().unwrap_or(&c.join_message))
    .bind(req.gameop_list_enabled.unwrap_or(c.gameop_list_enabled))
    .bind(req.gameop_list_message.as_deref().unwrap_or(&c.gameop_list_message))
    .bind(req.announcement_enabled.unwrap_or(c.announcement_enabled))
    .bind(req.announcement_content.as_deref().or(c.announcement_content.as_deref()))
    .bind(req.announcement_interval.unwrap_or(c.announcement_interval))
    .bind(server_id).fetch_one(pool).await
}
