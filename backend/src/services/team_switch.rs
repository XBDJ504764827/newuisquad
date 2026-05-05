use std::collections::HashMap;
use std::time::Instant;

const TIMEOUT_SECS: u64 = 60;

#[derive(Clone, Debug)]
struct PendingSwitch {
    tag: String,
    requester_name: String,
    created_at: Instant,
}

pub struct TeamSwitchManager {
    pending: HashMap<String, Vec<PendingSwitch>>,
}

/// 生成 4 位随机标识
fn generate_tag() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{:04}", nanos % 10000)
}

/// 解析消息，返回 (action, tag)
/// action: "sq_tb" (玩家发起) | "approve" (管理员同意并执行)
fn parse_tagged_message(msg: &str) -> Option<(&'static str, String)> {
    let msg = msg.trim();
    let lower = msg.to_lowercase();

    // sqtb: 玩家发起跳边
    if lower == "sqtb" {
        return Some(("sq_tb", String::new()));
    }

    // tb<tag>: 管理员批准并执行
    if let Some(tag) = lower.strip_prefix("tb") {
        let tag = tag.trim().to_string();
        if !tag.is_empty() {
            return Some(("approve", tag));
        }
        return None;
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

        let tag = tag.trim().to_lowercase();

        match action {
            "sq_tb" => self.handle_sqtb(server_id, player_name, &tag, state),
            "approve" => self.handle_approve(server_id, player_name, &tag, state),
            _ => vec![],
        }
    }

    // ====== 从 state 中查找玩家 ======
    fn find_player<'a>(&self, player_name: &str, state: &'a serde_json::Value) -> Option<&'a serde_json::Value> {
        let players = state["players"].as_array()?;
        if let Some(p) = players.iter().find(|p| p["name"].as_str() == Some(player_name)) {
            return Some(p);
        }
        let lower = player_name.to_lowercase();
        players.iter().find(|p| {
            p["name"].as_str().map(|n| n.to_lowercase() == lower).unwrap_or(false)
        })
    }

    // ====== 获取在线管理员列表 ======
    fn get_admin_players<'a>(&self, state: &'a serde_json::Value) -> Vec<&'a serde_json::Value> {
        let admin_ids: Vec<&str> = state["admin_steam_ids"]
            .as_array()
            .map(|ids| ids.iter().filter_map(|id| id.as_str()).collect())
            .unwrap_or_default();

        state["players"].as_array().map(|players| {
            players.iter().filter(|p| {
                if p["is_admin"].as_bool().unwrap_or(false) { return true; }
                let sid = p["steam_id"].as_str().unwrap_or("");
                !sid.is_empty() && admin_ids.iter().any(|id| *id == sid)
            }).collect()
        }).unwrap_or_default()
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

    // ====== 玩家发起 (sqtb) ======
    fn handle_sqtb(
        &mut self,
        server_id: &str,
        player_name: &str,
        _tag: &str,
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

        // 检查是否已有进行中的请求
        {
            let entries = self.pending.entry(server_id.to_string()).or_default();
            if let Some(existing) = entries.iter_mut().find(|r| r.requester_name == requester_name) {
                existing.created_at = Instant::now();
                let tag = existing.tag.clone();

                let mut cmds = vec![
                    format!(
                        "AdminWarn \"{}\" \"您已经在一分钟之内申请过了，请耐心等待\"",
                        requester_name
                    ),
                    format!(
                        "AdminBroadcast \"{} 重新申请跳边，请管理员输入 tb{} 同意，拒绝则忽略\"",
                        requester_name, tag
                    ),
                ];

                for admin in self.get_admin_players(state) {
                    if let Some(admin_name) = admin["name"].as_str() {
                        cmds.push(format!(
                            "AdminWarn \"{}\" \"{} 重新申请跳边，同意请在聊天框输入 tb{}，拒绝则忽略\"",
                            admin_name, requester_name, tag
                        ));
                    }
                }

                return cmds;
            }
        }

        // 新请求：生成随机 tag
        let entries = self.pending.entry(server_id.to_string()).or_default();
        let tag = loop {
            let t = generate_tag();
            if !entries.iter().any(|r| r.tag == t) {
                break t;
            }
        };

        entries.push(PendingSwitch {
            tag: tag.clone(),
            requester_name: requester_name.clone(),
            created_at: Instant::now(),
        });

        let mut cmds = vec![
            format!(
                "AdminBroadcast \"{} 申请跳边，请管理员输入 tb{} 审批跳边申请\"",
                requester_name, tag
            ),
        ];

        // 对每个在线管理员发送 AdminWarn
        for admin in self.get_admin_players(state) {
            if let Some(admin_name) = admin["name"].as_str() {
                cmds.push(format!(
                    "AdminWarn \"{}\" \"{} 申请跳边，同意请在聊天框输入 tb{}，拒绝则忽略\"",
                    admin_name, requester_name, tag
                ));
            }
        }

        cmds
    }

    // ====== 管理员批准 (tbxxx) ======
    fn handle_approve(
        &mut self,
        server_id: &str,
        admin_name: &str,
        tag: &str,
        state: Option<&serde_json::Value>,
    ) -> Vec<String> {
        let Some(state) = state else { return vec![] };
        if !self.check_is_admin(admin_name, state) { return vec![]; }

        let entries = self.pending.get_mut(server_id);
        let Some(entries) = entries else { return vec![] };

        let req_idx = entries.iter().position(|r| r.tag == tag);
        let Some(req_idx) = req_idx else { return vec![] };
        let req = entries.remove(req_idx);

        vec![
            format!("AdminForceTeamChange \"{}\"", req.requester_name),
            format!(
                "AdminBroadcast \"管理员 {} 已同意 {} 的跳边申请，执行跳边\"",
                admin_name, req.requester_name
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
