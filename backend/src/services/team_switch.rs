use std::collections::HashMap;
use std::time::Instant;

const TIMEOUT_SECS: u64 = 60;

#[derive(Clone, Debug, PartialEq)]
enum SwitchState {
    AwaitingAdmin,
    AwaitingLeader,
}

#[derive(Clone, Debug)]
struct PendingSwitch {
    player_name: String,
    steam_id: String,
    state: SwitchState,
    created_at: Instant,
}

pub struct TeamSwitchManager {
    pending: HashMap<String, Vec<PendingSwitch>>,
}

impl TeamSwitchManager {
    pub fn new() -> Self {
        Self { pending: HashMap::new() }
    }

    /// 处理聊天消息，返回需要发送的 RCON 命令列表
    pub fn process_chat(
        &mut self,
        server_id: &str,
        player_name: &str,
        message: &str,
        state_json: Option<&serde_json::Value>,
    ) -> Vec<String> {
        tracing::info!(server_id, player = %player_name, msg = %message, "代码跳边: 收到聊天消息");
        self.cleanup(server_id);

        let msg = message.trim().to_lowercase();
        match msg.as_str() {
            "tb" | "跳边" => self.handle_tb(server_id, player_name, state_json),
            "ty" | "同意" => self.handle_admin_approve(server_id, player_name, state_json),
            "rl" | "认领" => self.handle_leader_claim(server_id, player_name, state_json),
            _ => vec![],
        }
    }

    // ====== 玩家请求跳边 ======
    fn handle_tb(
        &mut self,
        server_id: &str,
        player_name: &str,
        state: Option<&serde_json::Value>,
    ) -> Vec<String> {
        let (steam_id, is_admin) = self.extract_player_info(player_name, state);

        let player_count = state
            .and_then(|s| s["players"].as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let has_squadjs = state
            .and_then(|s| s["squadjs_squads"].as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        tracing::info!(server_id, player = %player_name, has_state = state.is_some(), steam_id = %steam_id, is_admin, player_count, has_squadjs, "代码跳边: handle_tb");

        // 管理员自己跳边直接进入认领阶段
        if is_admin {
            return self.promote_to_leader_claim(server_id, player_name, &steam_id, state);
        }

        // 如果 steam_id 为空（没找到玩家或 state 为 None），仍广播提示
        if steam_id.is_empty() {
            return vec![format!(
                "AdminBroadcast \"{} 发送了 tb，但系统暂未获取到服务器状态，请稍后再试\"",
                player_name
            )];
        }

        // 检查是否已有未完成的请求
        let entries = self.pending.entry(server_id.to_string()).or_default();
        if entries.iter().any(|r| r.player_name == player_name) {
            return vec![format!(
                "AdminBroadcast \"{} 你的跳边请求已在处理中，请耐心等待\"",
                player_name
            )];
        }

        entries.push(PendingSwitch {
            player_name: player_name.to_string(),
            steam_id,
            state: SwitchState::AwaitingAdmin,
            created_at: Instant::now(),
        });

        // 查找在线管理员
        let admin_names: Vec<&str> = state
            .and_then(|s| s["players"].as_array())
            .map(|players| {
                players.iter()
                    .filter(|p| p["is_admin"].as_bool().unwrap_or(false))
                    .filter_map(|p| p["name"].as_str())
                    .collect()
            })
            .unwrap_or_default();

        if admin_names.is_empty() {
            vec![format!(
                "AdminBroadcast \"玩家 {} 请求跳边！但是没有管理员在线，请稍后再试\"",
                player_name
            )]
        } else {
            let admin_list = admin_names.join(", ");
            vec![
                format!(
                    "AdminBroadcast \"玩家 {} 请求跳边！在线管理员({}) 请发送 ty 或 同意 批准（60秒有效）\"",
                    player_name, admin_list
                ),
            ]
        }
    }

    // ====== 管理员同意 ======
    fn handle_admin_approve(
        &mut self,
        server_id: &str,
        admin_name: &str,
        state: Option<&serde_json::Value>,
    ) -> Vec<String> {
        // 验证说话的人是否是管理员
        let is_admin = state
            .and_then(|s| s["players"].as_array())
            .map(|players| {
                players.iter().any(|p| {
                    p["name"].as_str() == Some(admin_name)
                        && p["is_admin"].as_bool().unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if !is_admin {
            return vec![];
        }

        let entries = self.pending.get_mut(server_id);
        let Some(entries) = entries else { return vec![] };

        let request = entries
            .iter_mut()
            .find(|r| r.state == SwitchState::AwaitingAdmin);
        let Some(request) = request else { return vec![] };

        request.state = SwitchState::AwaitingLeader;

        let player_name = request.player_name.clone();
        // 直接通知对应小队长
        let leader_names: Vec<String> = state
            .and_then(|s| s["squad_leaders"].as_array())
            .map(|leaders| {
                leaders.iter()
                    .filter_map(|l| l["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let mut cmds = vec![format!(
            "AdminBroadcast \"管理员 {} 已批准！玩家 {} 可以跳边。小队长请发送 rl 或 认领 将其编入小队\"",
            admin_name, player_name,
        )];

        // 如果有已知的小队长，专门呼叫
        for ln in &leader_names {
            cmds.push(format!(
                "AdminBroadcast \"@{} 请发送 rl 或 认领 来接收玩家 {} 到你的小队\"",
                ln, player_name,
            ));
        }

        cmds
    }

    // ====== 小队长认领 ======
    fn handle_leader_claim(
        &mut self,
        server_id: &str,
        leader_name: &str,
        state: Option<&serde_json::Value>,
    ) -> Vec<String> {
        let Some(state) = state else { return vec![] };

        // 验证是否是队长（通过 squad_leaders 或 is_leader 字段）
        let is_verified_leader = state["squad_leaders"].as_array()
            .map(|leaders| leaders.iter().any(|l| l["name"].as_str() == Some(leader_name)))
            .unwrap_or(false)
            ||
            state["players"].as_array()
            .map(|players| players.iter().any(|p| p["name"].as_str() == Some(leader_name) && p["is_leader"].as_bool().unwrap_or(false)))
            .unwrap_or(false);

        if !is_verified_leader {
            return vec![];
        }

        // 找到认领者在玩家列表中的信息
        let claimer = state["players"].as_array()
            .and_then(|players| players.iter().find(|p| p["name"].as_str() == Some(leader_name)));

        let Some(claimer) = claimer else { return vec![] };

        let claimer_team_id = claimer["team_id"].as_i64().unwrap_or(0);
        let claimer_squad_id = claimer["squad_id"].as_str().and_then(|s| {
            if s.is_empty() || s == "N/A" || s == "null" { None } else { Some(s.to_string()) }
        });

        let Some(ref squad_id) = claimer_squad_id else {
            return vec![];
        };

        // 找到待认领的跳边请求
        let entries = self.pending.get_mut(server_id);
        let Some(entries) = entries else { return vec![] };

        let req_idx = entries
            .iter()
            .position(|r| r.state == SwitchState::AwaitingLeader);
        let Some(req_idx) = req_idx else { return vec![] };

        let request = entries.remove(req_idx);
        let target_steam_id = request.steam_id.clone();

        let squad_name = state["squads"].as_array()
            .and_then(|squads| squads.iter()
                .find(|s| s["squad_id"].as_str() == Some(squad_id))
                .and_then(|s| s["name"].as_str().map(String::from))
            )
            .unwrap_or_else(|| format!("小队 {}", squad_id));

        vec![
            format!("AdminForceTeamChange {}", target_steam_id),
            format!("AdminSetTeam {} {} {}", target_steam_id, claimer_team_id, squad_id),
            format!(
                "AdminBroadcast \"玩家 {} 已跳边至 {} 队 [{}] 的小队（队长：{}）\"",
                request.player_name, claimer_team_id, squad_name, leader_name
            ),
        ]
    }

    // ====== 跳过管理员审批，直接进入队长认领（管理员自己跳边） ======
    fn promote_to_leader_claim(
        &mut self,
        server_id: &str,
        player_name: &str,
        steam_id: &str,
        state: Option<&serde_json::Value>,
    ) -> Vec<String> {
        // 清理同名的旧请求
        if let Some(entries) = self.pending.get_mut(server_id) {
            entries.retain(|r| r.player_name != player_name);
        }

        let entries = self.pending.entry(server_id.to_string()).or_default();
        entries.push(PendingSwitch {
            player_name: player_name.to_string(),
            steam_id: steam_id.to_string(),
            state: SwitchState::AwaitingLeader,
            created_at: Instant::now(),
        });

        let leader_names: Vec<String> = state
            .and_then(|s| s["squad_leaders"].as_array())
            .map(|leaders| {
                leaders.iter()
                    .filter_map(|l| l["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let mut cmds = vec![format!(
            "AdminBroadcast \"管理员 {} 请求跳边！小队长请发送 rl 或 认领 将其编入小队\"",
            player_name,
        )];

        for ln in &leader_names {
            cmds.push(format!(
                "AdminBroadcast \"@{} 请发送 rl 或 认领 来接收管理员 {} 到你的小队\"",
                ln, player_name,
            ));
        }

        cmds
    }

    // ====== 从 state 中提取玩家 steam_id 和 is_admin ======
    fn extract_player_info<'a>(&self, player_name: &str, state: Option<&'a serde_json::Value>) -> (String, bool) {
        match state {
            Some(s) => s["players"].as_array()
                .and_then(|players| {
                    players.iter()
                        .find(|p| p["name"].as_str() == Some(player_name))
                        .map(|p| (
                            p["steam_id"].as_str().unwrap_or("").to_string(),
                            p["is_admin"].as_bool().unwrap_or(false),
                        ))
                })
                .unwrap_or_default(),
            None => (String::new(), false),
        }
    }

    // ====== 过期清理 ======
    fn cleanup(&mut self, server_id: &str) {
        if let Some(entries) = self.pending.get_mut(server_id) {
            let now = Instant::now();
            entries.retain(|r| now.duration_since(r.created_at).as_secs() < TIMEOUT_SECS);
        }
    }
}
