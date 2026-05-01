use axum::{Json, extract::{State, Path, Query}, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;
use crate::api::AppState;
use crate::protocol::AgentMessage;

#[derive(Deserialize)]
pub struct FilePathQuery {
    pub path: String,
}

#[derive(Deserialize)]
pub struct DirListQuery {
    pub dir: Option<String>,
}

#[derive(Deserialize)]
pub struct WriteFileRequest {
    pub path: String,
    pub content: String,
    pub admin_user: String,
}

pub async fn read_file(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<FilePathQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(ref agent_pool) = state.agent_pool else {
        return Ok(Json(serde_json::json!({ "error": "Agent 服务未启动" })));
    };

    let request_id = Uuid::new_v4().to_string();
    let cmd = AgentMessage::ReadFile {
        request_id: request_id.clone(),
        path: q.path.clone(),
    };

    match agent_pool
        .send_and_wait(&server_id.to_string(), cmd, &request_id)
        .await
    {
        Ok(AgentMessage::FileReadResult {
            content, error, ..
        }) => {
            if let Some(err) = error {
                Ok(Json(serde_json::json!({ "error": err })))
            } else {
                Ok(Json(serde_json::json!({ "path": q.path, "content": content })))
            }
        }
        Ok(_) => Ok(Json(serde_json::json!({ "error": "未知响应" }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}

pub async fn write_file(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Json(req): Json<WriteFileRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(ref agent_pool) = state.agent_pool else {
        return Ok(Json(serde_json::json!({ "error": "Agent 服务未启动" })));
    };

    let request_id = Uuid::new_v4().to_string();
    let cmd = AgentMessage::WriteFile {
        request_id: request_id.clone(),
        path: req.path.clone(),
        content: req.content.clone(),
    };

    match agent_pool
        .send_and_wait(&server_id.to_string(), cmd, &request_id)
        .await
    {
        Ok(AgentMessage::FileWriteResult {
            success, error, ..
        }) => {
            if success {
                let _ = sqlx::query(
                    "INSERT INTO file_ops (server_id, admin_user, operation, file_path, content) VALUES ($1, $2, 'WRITE', $3, $4)",
                )
                .bind(server_id)
                .bind(&req.admin_user)
                .bind(&req.path)
                .bind(&req.content)
                .execute(&state.pool)
                .await;
                Ok(Json(serde_json::json!({ "success": true })))
            } else {
                Ok(Json(serde_json::json!({ "error": error })))
            }
        }
        Ok(_) => Ok(Json(serde_json::json!({ "error": "未知响应" }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}

pub async fn list_files(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<DirListQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(ref agent_pool) = state.agent_pool else {
        return Ok(Json(serde_json::json!({ "error": "Agent 服务未启动" })));
    };

    let request_id = Uuid::new_v4().to_string();
    let dir = q.dir.unwrap_or_else(|| "/".to_string());
    let cmd = AgentMessage::ListFiles {
        request_id: request_id.clone(),
        dir,
    };

    match agent_pool
        .send_and_wait(&server_id.to_string(), cmd, &request_id)
        .await
    {
        Ok(AgentMessage::FileListResult { files, .. }) => {
            Ok(Json(serde_json::json!({ "files": files })))
        }
        Ok(_) => Ok(Json(serde_json::json!({ "error": "未知响应" }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}
