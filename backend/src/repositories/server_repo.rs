use sqlx::PgPool;
use crate::models::server::{Server, UpdateServerRequest};

pub async fn list_servers(pool: &PgPool) -> Result<Vec<Server>, sqlx::Error> {
    sqlx::query_as::<_, Server>("SELECT * FROM servers ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn get_server(pool: &PgPool, id: i32) -> Result<Option<Server>, sqlx::Error> {
    sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn update_server(pool: &PgPool, id: i32, req: &UpdateServerRequest) -> Result<Option<Server>, sqlx::Error> {
    let server = get_server(pool, id).await?;
    let Some(current) = server else { return Ok(None) };

    let name = req.name.as_deref().unwrap_or(&current.name);
    let ip = req.ip.as_deref().unwrap_or(&current.ip);
    let rcon_port = req.rcon_port.unwrap_or(current.rcon_port);
    let rcon_password = req.rcon_password.as_deref().unwrap_or(&current.rcon_password);

    sqlx::query_as::<_, Server>(
        "UPDATE servers SET name=$1, ip=$2, rcon_port=$3, rcon_password=$4, updated_at=NOW() WHERE id=$5 RETURNING *"
    )
    .bind(name).bind(ip).bind(rcon_port).bind(rcon_password).bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_server(pool: &PgPool, id: i32) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM rcon_logs WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM server_logs WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM file_ops WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM tk_settings WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM afk_settings WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM broadcast_settings WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM announcements WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM auto_replies WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM team_settings WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM seed_settings WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM damage_notify_settings WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM abnormal_damage_rules WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM abnormal_damage_logs WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM abnormal_damage_config WHERE server_id = $1").bind(id).execute(&mut *tx).await?;
    let result = sqlx::query("DELETE FROM servers WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}
