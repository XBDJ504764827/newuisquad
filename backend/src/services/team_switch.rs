use std::collections::HashMap;
use std::time::Instant;

const TIMEOUT_SECS: u64 = 60;

#[derive(Clone, Debug, PartialEq)]
enum SwitchPhase {
    AwaitingClaim,
    AwaitingAdmin,
}

#[derive(Clone, Debug)]
struct PendingSwitch {
    tag: String,
    requester_name: String,
    requester_player_id: i32,
    requester_team_id: i32,
    claimer_name: Option<String>,
    claimer_player_id: Option<i32>,
    claimer_team_id: Option<i32>,
    phase: SwitchPhase,
    created_at: Instant,
}

pub struct TeamSwitchManager {
    pending: HashMap<String, Vec<PendingSwitch>>,
}

/// 解析标签消息，返回 (action, tag)
/// action: "tb" | "rl" | "ty"
/// tag: 标签内容（可能为空字符串）
fn parse_tagged_message(msg: &str) -> Option<(&'static str, String)> {
    let msg = msg.trim();

    // === English prefixes (case-insensitive) ===
    let lower = msg.to_lowercase();

    // tb<tag>
    if let Some(tag) = lower.strip_prefix("tb") {
        return Some(("tb", tag.to_string()));
    }
    // rl<tag>
    if let Some(tag) = lower.strip_prefix("rl") {
        return Some(("rl", tag.to_string()));
    }
    // ty<tag>
    if let Some(tag) = lower.strip_prefix("ty") {
        return Some(("ty", tag.to_string()));
    }

    // === Chinese prefixes ===
    if let Some(tag) = msg.strip_prefix("跳边") {
        return Some(("tb", tag.to_string()));
    }
    if let Some(tag) = msg.strip_prefix("认领") {
        return Some(("rl", tag.to_string()));
    }
    if let Some(tag) = msg.strip_prefix("同意") {
        return Some(("ty", tag.to_string()));
    }

    None
}

impl TeamSwitchManager {
    pub fn new() -> Self {
        Self { pending: HashMap::new() }
    }

    pub fn process_chat(
        &mut self,
        server_id: &str,
        player_name: &str,
        message: &str,
        state: Option<&serde_json::Value>,
    ) -> Vec<String> {
        tracing::info!(server_id, player = %player_name, msg = %message, "代码跳边: 收到聊天消息");
        self.cleanup(server_id);

        let Some((action, tag)) = parse_tagged_message(message) else {
            return vec![];
        };

        // 规范 tag：trim 后转小写
        let tag = tag.trim().to_lowercase();

        match action {
            "tb" => self.handle_tb(server_id, player_name, &tag, state),
            "rl" => self.handle_claim(server_id, player_name, &tag, state),
            "ty" => self.handle_admin_approve(server_id, player_name, &tag, state),
            _ => vec![],
        }
    }

    // ====== 从 state 中查找玩家 ======
    fn find_player<'a>(&self, player_name: &str, state: &'a serde_json::Value) -> Option<&'a serde_json::Value> {
        let players = state["players"].as_array()?;
        // 精确匹配
        if let Some(p) = players.iter().find(|p| p["name"].as_str() == Some(player_name)) {
            return Some(p);
        }
        // 大小写不敏感
        let lower = player_name.to_lowercase();
        if let Some(p) = players.iter().find(|p| {
            p["name"].as_str().map(|n| n.to_lowercase() == lower).unwrap_or(false)
        }) {
            return Some(p);
        }
        // 未找到：打印调试信息
        let names: Vec<&str> = players.iter()
            .filter_map(|p| p["name"].as_str())
            .take(10)
            .collect();
        tracing::warn!(
            target_player = %player_name,
            player_count = players.len(),
            state_player_names = ?names,
            "代码跳边: find_player 未找到玩家"
        );
        None
    }

    // ====== 检查是否是管理员 ======
    fn check_is_admin(&self, player_name: &str, state: &serde_json::Value) -> bool {
        let admin_ids: Vec<&str> = state["admin_steam_ids"]
            .as_array()
            .map(|ids| ids.iter().filter_map(|id| id.as_str()).collect())
            .unwrap_or_default();

        state["players"].as_array().map(|players| {
            players.iter().any(|p| {
                if p["name"].as_str() != Some(player_name) { return false; }
                if p["is_admin"].as_bool().unwrap_or(false) { return true; }
                let sid = p["steam_id"].as_str().unwrap_or("");
                if sid.is_empty() { return false; }
                admin_ids.iter().any(|id| *id == sid)
            })
        }).unwrap_or(false)
    }

    // ====== Phase 1: 玩家请求跳边 (tb) ======
    fn handle_tb(
        &mut self,
        server_id: &str,
        player_name: &str,
        tag: &str,
        state: Option<&serde_json::Value>,
    ) -> Vec<String> {
        let Some(state) = state else {
            return vec![format!(
                "AdminBroadcast \"{} 发送了跳边请求，系统正在初始化，请稍后重试\"",
                player_name
            )];
        };

        let Some(player) = self.find_player(player_name, state) else {
            return vec![format!(
                "AdminBroadcast \"{} 发送了跳边请求，但在玩家列表中未找到你，请等待3秒后重试\"",
                player_name
            )];
        };

        let requester_name = player["name"].as_str().unwrap_or(player_name).to_string();
        let requester_player_id = player["player_id"].as_i64().unwrap_or(0) as i32;
        let requester_team_id = player["team_id"].as_i64().unwrap_or(0) as i32;

        if requester_player_id == 0 {
            return vec![format!(
                "AdminBroadcast \"{} 发送了跳边请求，但未能获取你的玩家ID，请等待状态刷新后重试\"",
                requester_name
            )];
        }

        let effective_tag = if tag.is_empty() {
            requester_name.to_lowercase()
        } else {
            tag.to_string()
        };

        // 检查同 tag 是否有进行中的请求
        let entries = self.pending.entry(server_id.to_string()).or_default();
        if entries.iter().any(|r| r.tag == effective_tag) {
            return vec![format!(
                "AdminBroadcast \"标签 '{}' 已有进行中的跳边请求，请使用其他标签重试\"",
                effective_tag
            )];
        }

        entries.push(PendingSwitch {
            tag: effective_tag.clone(),
            requester_name: requester_name.clone(),
            requester_player_id,
            requester_team_id,
            claimer_name: None,
            claimer_player_id: None,
            claimer_team_id: None,
            phase: SwitchPhase::AwaitingClaim,
            created_at: Instant::now(),
        });

        vec![format!(
            "AdminBroadcast \"{} 申请跳边，请对面阵营玩家发送 rl{} 或 认领{} 认领该玩家，如一分钟内无认领则失效\"",
            requester_name, effective_tag, effective_tag
        )]
    }

    // ====== Phase 2: 对面玩家认领 (rl) ======
    fn handle_claim(
        &mut self,
        server_id: &str,
        claimer_name: &str,
        tag: &str,
        state: Option<&serde_json::Value>,
    ) -> Vec<String> {
        let Some(state) = state else { return vec![] };
        let Some(claimer) = self.find_player(claimer_name, state) else { return vec![] };

        let claimer_display_name = claimer["name"].as_str().unwrap_or(claimer_name).to_string();
        let claimer_team_id = claimer["team_id"].as_i64().unwrap_or(0) as i32;
        let claimer_player_id = claimer["player_id"].as_i64().unwrap_or(0) as i32;

        let entries = self.pending.get_mut(server_id);
        let Some(entries) = entries else { return vec![] };

        // 查找匹配的 AwaitingClaim 请求
        let req_idx = entries.iter().position(|r| {
            let tag_match = if tag.is_empty() {
                true // 无 tag 则匹配第一个
            } else {
                r.tag == tag
            };
            tag_match
                && r.phase == SwitchPhase::AwaitingClaim
                && claimer_team_id != r.requester_team_id
                && claimer_team_id != 0
        });

        let Some(req_idx) = req_idx else { return vec![] };
        let req = &mut entries[req_idx];

        let requester_name = req.requester_name.clone();
        req.claimer_name = Some(claimer_display_name.clone());
        req.claimer_player_id = Some(claimer_player_id);
        req.claimer_team_id = Some(claimer_team_id);
        req.phase = SwitchPhase::AwaitingAdmin;
        req.created_at = Instant::now();

        let effective_tag = &req.tag;

        vec![format!(
            "AdminBroadcast \"{} 已被 {} 认领，请服务器内管理员发送 ty{} 或 同意{} 同意玩家跳边申请，拒绝则忽略，该申请在一分钟内失效\"",
            requester_name, claimer_display_name, effective_tag, effective_tag
        )]
    }

    // ====== Phase 3: 管理员批准 (ty) ======
    fn handle_admin_approve(
        &mut self,
        server_id: &str,
        admin_name: &str,
        tag: &str,
        state: Option<&serde_json::Value>,
    ) -> Vec<String> {
        let Some(state) = state else { return vec![] };

        if !self.check_is_admin(admin_name, state) {
            return vec![];
        }

        let entries = self.pending.get_mut(server_id);
        let Some(entries) = entries else { return vec![] };

        let req_idx = entries.iter().position(|r| {
            let tag_match = if tag.is_empty() {
                true
            } else {
                r.tag == tag
            };
            tag_match && r.phase == SwitchPhase::AwaitingAdmin && r.claimer_name.is_some()
        });

        let Some(req_idx) = req_idx else { return vec![] };
        let req = entries.remove(req_idx);

        let requester_name = req.requester_name;
        let requester_player_id = req.requester_player_id;
        let claimer_name = req.claimer_name.unwrap_or_default();

        vec![
            format!("AdminForceTeamChange {}", requester_player_id),
            format!(
                "AdminBroadcast \"{} 已跳边至 {} 所在队伍\"",
                requester_name, claimer_name
            ),
        ]
    }

    // ====== 过期清理 ======
    fn cleanup(&mut self, server_id: &str) {
        if let Some(entries) = self.pending.get_mut(server_id) {
            let now = Instant::now();
            entries.retain(|r| now.duration_since(r.created_at).as_secs() < TIMEOUT_SECS);
        }
    }
}
