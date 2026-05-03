use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AgentMessage {
    #[serde(rename = "log")]
    Log { data: LogEntry },
    #[serde(rename = "file_read_result")]
    FileReadResult {
        request_id: String,
        success: bool,
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    #[serde(rename = "file_write_result")]
    FileWriteResult {
        request_id: String,
        success: bool,
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    #[serde(rename = "file_list_result")]
    FileListResult {
        request_id: String,
        files: Vec<FileInfo>,
    },

    #[serde(rename = "server_state_report")]
    ServerStateReport {
        players: Vec<PlayerState>,
        squads: Vec<SquadState>,
        team_names: Vec<TeamState>,
        map_name: String,
        game_mode: String,
        server_name: String,
        player_count: i32,
        max_players: i32,
        next_map: String,
    },

    #[serde(rename = "read_file")]
    ReadFile { request_id: String, path: String },
    #[serde(rename = "write_file")]
    WriteFile { request_id: String, path: String, content: String },
    #[serde(rename = "list_files")]
    ListFiles { request_id: String, dir: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub name: String,
    pub steam_id: String,
    pub team_id: i32,
    pub squad_id: Option<String>,
    pub role: String,
    pub kills: i32,
    pub deaths: i32,
    pub score: i32,
    pub ping: i32,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadState {
    pub name: String,
    pub creator: String,
    pub team_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamState {
    pub team_id: i32,
    pub faction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub log_level: String,
    pub category: Option<String>,
    pub message: String,
    pub raw_line: Option<String>,
    pub logged_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
}
