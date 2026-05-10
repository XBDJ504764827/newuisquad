use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub triggers: Vec<WorkflowTrigger>,
    #[serde(default)]
    pub steps: Vec<WorkflowStep>,
}

fn default_version() -> String { "1.0".to_string() }

impl Default for WorkflowDefinition {
    fn default() -> Self {
        Self { version: "1.0".into(), triggers: vec![], steps: vec![] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTrigger {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub event_type: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub step_type: String, // condition, action, delay, loop
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub config: serde_json::Value,
}

/// 工作流记录（数据库行）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Workflow {
    pub id: i32,
    pub server_id: i32,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub definition: serde_json::Value,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 工作流执行记录
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowExecution {
    pub id: i32,
    pub workflow_id: i32,
    pub status: String,
    pub trigger_event_type: String,
    pub trigger_data: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// 创建工作流请求
#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub definition: WorkflowDefinition,
}

fn default_enabled() -> bool { true }

/// 更新工作流请求
#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub definition: Option<WorkflowDefinition>,
}
