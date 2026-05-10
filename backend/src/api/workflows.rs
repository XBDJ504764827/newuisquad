use axum::{Json, extract::{State, Path, Query}, http::StatusCode};
use serde::Deserialize;
use crate::api::AppState;
use crate::models::workflow::{self, CreateWorkflowRequest, UpdateWorkflowRequest};

/// GET /api/v1/servers/{id}/workflows
pub async fn list(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows = sqlx::query_as::<_, crate::models::workflow::Workflow>(
        "SELECT id, server_id, name, description, enabled, definition, created_by, created_at, updated_at FROM workflows WHERE server_id=$1 ORDER BY updated_at DESC"
    ).bind(server_id).fetch_all(&state.pool).await.map_err(|e| {
        tracing::error!("查询工作流失败: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({ "data": rows, "total": rows.len() })))
}

/// GET /api/v1/servers/{id}/workflows/{wid}
pub async fn get_one(
    State(state): State<AppState>,
    Path((server_id, wid)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row = sqlx::query_as::<_, crate::models::workflow::Workflow>(
        "SELECT id, server_id, name, description, enabled, definition, created_by, created_at, updated_at FROM workflows WHERE id=$1 AND server_id=$2"
    ).bind(wid).bind(server_id).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(w) => Ok(Json(serde_json::json!({ "data": w }))),
        None => Ok(Json(serde_json::json!({ "error": "工作流不存在" }))),
    }
}

/// POST /api/v1/servers/{id}/workflows
pub async fn create(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if req.name.trim().is_empty() {
        return Ok(Json(serde_json::json!({ "error": "名称不能为空" })));
    }

    let definition_json = serde_json::to_value(&req.definition).unwrap_or_default();
    let user = "admin"; // TODO: 从认证获取真实用户名

    let row = sqlx::query_as::<_, crate::models::workflow::Workflow>(
        "INSERT INTO workflows (server_id, name, description, enabled, definition, created_by) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, server_id, name, description, enabled, definition, created_by, created_at, updated_at"
    ).bind(server_id).bind(req.name.trim()).bind(req.description).bind(req.enabled).bind(&definition_json).bind(user)
    .fetch_one(&state.pool).await.map_err(|e| {
        tracing::error!("创建工作流失败: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({ "data": row })))
}

/// PUT /api/v1/servers/{id}/workflows/{wid}
pub async fn update(
    State(state): State<AppState>,
    Path((server_id, wid)): Path<(i32, i32)>,
    Json(req): Json<UpdateWorkflowRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 获取现有记录
    let existing = sqlx::query_as::<_, crate::models::workflow::Workflow>(
        "SELECT id, server_id, name, description, enabled, definition, created_by, created_at, updated_at FROM workflows WHERE id=$1 AND server_id=$2"
    ).bind(wid).bind(server_id).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let w = match existing {
        Some(w) => w,
        None => return Ok(Json(serde_json::json!({ "error": "工作流不存在" }))),
    };

    let name = req.name.unwrap_or(w.name);
    let description = req.description.unwrap_or(w.description);
    let enabled = req.enabled.unwrap_or(w.enabled);
    let definition = req.definition.map(|d| serde_json::to_value(d).unwrap_or_default()).unwrap_or(w.definition);

    let row = sqlx::query_as::<_, crate::models::workflow::Workflow>(
        "UPDATE workflows SET name=$1, description=$2, enabled=$3, definition=$4, updated_at=NOW() WHERE id=$5 AND server_id=$6 RETURNING id, server_id, name, description, enabled, definition, created_by, created_at, updated_at"
    ).bind(&name).bind(&description).bind(enabled).bind(&definition).bind(wid).bind(server_id)
    .fetch_one(&state.pool).await.map_err(|e| {
        tracing::error!("更新工作流失败: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({ "data": row })))
}

/// DELETE /api/v1/servers/{id}/workflows/{wid}
pub async fn delete(
    State(state): State<AppState>,
    Path((server_id, wid)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query("DELETE FROM workflows WHERE id=$1 AND server_id=$2")
        .bind(wid).bind(server_id).execute(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Ok(Json(serde_json::json!({ "error": "工作流不存在" })));
    }
    Ok(Json(serde_json::json!({ "success": true })))
}

/// POST /api/v1/servers/{id}/workflows/{wid}/toggle
pub async fn toggle(
    State(state): State<AppState>,
    Path((server_id, wid)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row = sqlx::query_as::<_, (bool,)>(
        "UPDATE workflows SET enabled=NOT enabled, updated_at=NOW() WHERE id=$1 AND server_id=$2 RETURNING enabled"
    ).bind(wid).bind(server_id).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some((enabled,)) => Ok(Json(serde_json::json!({ "success": true, "enabled": enabled }))),
        None => Ok(Json(serde_json::json!({ "error": "工作流不存在" }))),
    }
}

/// GET /api/v1/servers/{id}/workflows/{wid}/executions
#[derive(Deserialize, Default)]
pub struct ExecutionQuery { pub page: Option<i64>, pub per_page: Option<i64> }

pub async fn executions(
    State(state): State<AppState>,
    Path((server_id, wid)): Path<(i32, i32)>,
    Query(q): Query<ExecutionQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (total,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM workflow_executions WHERE workflow_id=$1"
    ).bind(wid).fetch_one(&state.pool).await.unwrap_or((0,));

    let rows = sqlx::query_as::<_, crate::models::workflow::WorkflowExecution>(
        "SELECT id, workflow_id, status, trigger_event_type, trigger_data, started_at, completed_at, error_message FROM workflow_executions WHERE workflow_id=$1 ORDER BY started_at DESC LIMIT $2 OFFSET $3"
    ).bind(wid).bind(per_page).bind(offset).fetch_all(&state.pool).await.unwrap_or_default();

    Ok(Json(serde_json::json!({ "data": rows, "total": total, "page": page, "per_page": per_page })))
}
