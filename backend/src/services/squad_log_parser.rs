use chrono::NaiveDateTime;

/// Squad 日志行解析结果
pub enum ParsedEvent {
    PlayerLogin {
        player_name: String,
        eos_id: String,
        steam64: String,
        ip: String,
        logged_at: NaiveDateTime,
    },
    FlyEvent {
        player_name: String,
        eos_id: String,
        steam64: String,
        event_type: String, // "possess", "unpossess", "spectate"
        logged_at: NaiveDateTime,
    },
    KillEvent {
        attacker_name: String,
        attacker_eos: String,
        attacker_steam64: String,
        victim_name: String,
        damage: f64,
        weapon: String,
        is_kill: bool,
        is_teamkill: bool,
        logged_at: NaiveDateTime,
    },
    TeamAssignment {
        player_name: String,
        steam64: String,
        team_id: i32,
        logged_at: NaiveDateTime,
    },
    SquadCreation {
        player_name: String,
        steam64: String,
        squad_id: String,
        squad_name: String,
        faction: String,
        logged_at: NaiveDateTime,
    },
    MatchEvent {
        map_name: String,
        layer_name: String,
        team1_faction: String,
        team2_faction: String,
        winner_team: Option<i32>,
        event_type: String,
        logged_at: NaiveDateTime,
    },
    DeployRole {
        player_name: String,
        steam64: String,
        role: String,
        logged_at: NaiveDateTime,
    },
    ReviveEvent {
        reviver_name: String,
        reviver_steam64: String,
        revived_name: String,
        revived_steam64: String,
        logged_at: NaiveDateTime,
    },
    VehicleEvent {
        player_name: String,
        steam64: String,
        vehicle_name: String,
        event_type: String,
        logged_at: NaiveDateTime,
    },
    AdminAction {
        admin_name: String,
        action_type: String,
        target: String,
        message: String,
        raw_line: String,
        logged_at: NaiveDateTime,
    },
    PlayerDeath {
        player_name: String,
        steam64: String,
        killer_steam64: String,
        weapon: String,
        logged_at: NaiveDateTime,
    },
    ChatMessage {
        player_name: String,
        steam64: String,
        message: String,
        channel: String,
        logged_at: NaiveDateTime,
    },
}

/// 解析 Squad 日志行的时间戳 [2026.05.01-06.17.03:441]
fn parse_timestamp(line: &str) -> Option<NaiveDateTime> {
    let start = line.find('[')?;
    let end = line[start..].find(']')?;
    let ts = &line[start + 1..start + end];
    // 格式: 2026.05.01-06.17.03:441
    let normalized = ts.replace('.', "").replace(':', "").replace('-', "");
    if normalized.len() < 14 { return None; }
    NaiveDateTime::parse_from_str(&format!("{}000", &normalized[..14]), "%Y%m%d%H%M%S%3f").ok()
}

/// 从 (Online IDs: EOS: xxx steam: xxx) 中提取 EOS 和 steam64
fn extract_online_ids(text: &str) -> (String, String) {
    let mut eos = String::new();
    let mut steam = String::new();
    if let Some(eos_start) = text.find("EOS: ") {
        let rest = &text[eos_start + 5..];
        if let Some(end) = rest.find(|c: char| c == ' ' || c == '|' || c == ')') {
            eos = rest[..end].trim().to_string();
        } else {
            eos = rest.trim().to_string();
        }
    }
    if let Some(st_start) = text.find("steam: ") {
        let rest = &text[st_start + 7..];
        if let Some(end) = rest.find(|c: char| c == ' ' || c == '|' || c == ')') {
            steam = rest[..end].trim().to_string();
        } else {
            steam = rest.trim().to_string();
        }
    }
    (eos, steam)
}

/// 解析单行日志
pub fn parse_line(line: &str) -> Option<ParsedEvent> {
    let ts = parse_timestamp(line)?;

    // 1. PostLogin: 最完整的玩家信息
    if line.contains("PostLogin:") {
        let (eos, steam) = extract_online_ids(line);
        let ip = if let Some(ip_start) = line.find("IP: ") {
            let rest = &line[ip_start + 4..];
            if let Some(end) = rest.find(|c: char| c == ' ' || c == '|') {
                rest[..end].trim().to_string()
            } else {
                String::new()
            }
        } else { String::new() };

        let player_name = if let Some(pn_start) = line.find("PostLogin: NewPlayer: ") {
            let rest = &line[pn_start + 23..];
            // 跳过 BP_PlayerController 部分，找到 (IP: ... 之前的内容
            if let Some(_ip_pos) = rest.find(" (IP:") {
                // 往前找到 "BP_PlayerController" 之后的部分... 太复杂了
                // 先提取 PostLogin 中的名字，通常在 "NewPlayer: " 之后的路径中
                String::new()
            } else {
                String::new()
            }
        } else { String::new() };

        if !eos.is_empty() {
            return Some(ParsedEvent::PlayerLogin {
                player_name,
                eos_id: eos,
                steam64: steam,
                ip,
                logged_at: ts,
            });
        }
    }

    // 2. Login request: ?Name=玩家名
    if line.contains("Login request:") {
        let player_name = if let Some(name_start) = line.find("?Name=") {
            let rest = &line[name_start + 6..];
            if let Some(end) = rest.find(|c: char| c == ' ' || c == '?') {
                rest[..end].to_string()
            } else {
                rest.trim().to_string()
            }
        } else { String::new() };

        if !player_name.is_empty() {
            let eos = if let Some(eos_start) = line.find("RedpointEOS:") {
                let rest = &line[eos_start + 13..];
                if let Some(end) = rest.find(|c: char| c == ' ' || c == '?') {
                    rest[..end].to_string()
                } else {
                    rest.trim().to_string()
                }
            } else { String::new() };

            return Some(ParsedEvent::PlayerLogin {
                player_name,
                eos_id: eos,
                steam64: String::new(),
                ip: String::new(),
                logged_at: ts,
            });
        }
    }

    // 3. 管理员镜头 (飞天)
    if line.contains("BP_DeveloperAdminCam_C") || line.contains("Admin Camera possessed") {
        let event_type = if line.contains("OnPossess") { "possess" }
            else if line.contains("OnUnPossess") { "unpossess" }
            else { "admin_camera" };

        let player_name = if let Some(pn_start) = line.find("PC=") {
            let rest = &line[pn_start + 3..];
            if let Some(end) = rest.find(|c: char| c == ' ' || c == '(') {
                rest[..end].trim().to_string()
            } else { String::new() }
        } else { String::new() };

        let (eos, steam) = extract_online_ids(line);

        return Some(ParsedEvent::FlyEvent {
            player_name,
            eos_id: eos,
            steam64: steam,
            event_type: event_type.to_string(),
            logged_at: ts,
        });
    }

    // 4. 观战
    if line.contains("ChangeState") && line.contains("Spectating") {
        let player_name = if let Some(pn_start) = line.find("PC=") {
            let rest = &line[pn_start + 3..];
            if let Some(end) = rest.find(|c: char| c == ' ' || c == '(') {
                rest[..end].trim().to_string()
            } else { String::new() }
        } else { String::new() };

        let (eos, steam) = extract_online_ids(line);

        return Some(ParsedEvent::FlyEvent {
            player_name,
            eos_id: eos,
            steam64: steam,
            event_type: "spectate".to_string(),
            logged_at: ts,
        });
    }

    // 5. 伤害事件: Player: victim ActualDamage=X from attacker
    if line.contains("ActualDamage=") && line.contains("LogSquad: Player:") {
        let victim = if let Some(v_start) = line.find("Player: ") {
            let rest = &line[v_start + 8..];
            if let Some(end) = rest.find("ActualDamage=") {
                rest[..end].trim().to_string()
            } else { String::new() }
        } else { String::new() };

        let damage = if let Some(d_start) = line.find("ActualDamage=") {
            let rest = &line[d_start + 13..];
            if let Some(end) = rest.find(|c: char| c == ' ' || c == 'f') {
                rest[..end].trim().parse::<f64>().unwrap_or(0.0)
            } else { 0.0 }
        } else { 0.0 };

        let attacker = if let Some(a_start) = line.find("from ") {
            let rest = &line[a_start + 5..];
            // attacker ends at " (Online IDs:" or "caused by"
            if let Some(end) = rest.find(" (Online IDs:") {
                rest[..end].trim().to_string()
            } else if let Some(end) = rest.find("caused by") {
                rest[..end].trim().to_string()
            } else {
                rest.trim().to_string()
            }
        } else { String::new() };

        let (attacker_eos, attacker_steam64) = extract_online_ids(line);

        let weapon = if let Some(w_start) = line.find("caused by ") {
            let rest = &line[w_start + 10..];
            if let Some(_end) = rest.find(|c: char| c == '_' || c == ' ') {
                // 武器名在第一个下划线之前
                let full = &rest[..rest.len().min(50)];
                full.split('_').next().unwrap_or(full).trim().to_string()
            } else {
                rest.trim().to_string()
            }
        } else { "Unknown".to_string() };

        let is_kill = line.contains("KillingDamage=") || line.contains("Wound()");
        let is_teamkill = line.to_lowercase().contains("teamkill") || line.contains("友军") || line.contains("队友");

        return Some(ParsedEvent::KillEvent {
            attacker_name: attacker,
            attacker_eos,
            attacker_steam64,
            victim_name: victim,
            damage,
            weapon,
            is_kill,
            is_teamkill,
            logged_at: ts,
        });
    }

    // 6. Kill/Wound 事件: Wound(): Player: victim KillingDamage=X from ...
    if line.contains("Wound()") && line.contains("KillingDamage=") {
        let victim = if let Some(v_start) = line.find("Player: ") {
            let rest = &line[v_start + 8..];
            if let Some(end) = rest.find("KillingDamage=") {
                rest[..end].trim().to_string()
            } else { String::new() }
        } else { String::new() };

        let damage = if let Some(d_start) = line.find("KillingDamage=") {
            let rest = &line[d_start + 14..];
            if let Some(end) = rest.find(|c: char| c == ' ' || c == 'f') {
                rest[..end].trim().parse::<f64>().unwrap_or(0.0)
            } else { 0.0 }
        } else { 0.0 };

        let (attacker_eos, attacker_steam64) = extract_online_ids(line);

        let weapon = if let Some(w_start) = line.find("caused by ") {
            let rest = &line[w_start + 10..];
            let full = &rest[..rest.len().min(50)];
            full.split('_').next().unwrap_or(full).trim().to_string()
        } else { "Unknown".to_string() };

        return Some(ParsedEvent::KillEvent {
            attacker_name: String::new(),
            attacker_eos,
            attacker_steam64,
            victim_name: victim,
            damage,
            weapon,
            is_kill: true,
            is_teamkill: false,
            logged_at: ts,
        });
    }

    // 7. TK 通知
    if line.contains("你TK了") || (line.contains("击杀了队友") && line.contains("ADMIN COMMAND")) {
        let _is_broadcast = line.contains("broadcasted");
        let player_name = if let Some(_pn_start) = line.find("你TK了") {
            String::new() // 无法直接获取名字
        } else if let Some(_pn_start) = line.find("击杀了队友 ") {
            // 格式: <玩家名 击杀了队友 队友名>
            // 从 broadcasted < 之后提取
            if let Some(lt) = line.find('<') {
                let rest = &line[lt + 1..];
                if let Some(gt) = rest.find('>') {
                    rest[..gt].split_whitespace().next().unwrap_or("").to_string()
                } else { String::new() }
            } else { String::new() }
        } else { String::new() };

        // 尝试从行中提取玩家名和队友名
        if !player_name.is_empty() {
            let victim_name = line.split("击杀了队友").nth(1)
                .and_then(|s| s.split('>').next())
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            return Some(ParsedEvent::KillEvent {
                attacker_name: player_name,
                attacker_eos: String::new(),
                attacker_steam64: String::new(),
                victim_name,
                damage: 0.0,
                weapon: "TeamKill".to_string(),
                is_kill: true,
                is_teamkill: true,
                logged_at: ts,
            });
        }
    }

    // 8. 队伍分配
    if line.contains("has been added to Team ") {
        let player_name = line.split("Player").nth(1).and_then(|s| s.split("has been added").next()).map(|s| s.trim().to_string()).unwrap_or_default();
        let team_str = line.split("has been added to Team ").nth(1).map(|s| s.trim().to_string()).unwrap_or_default();
        let team_id: i32 = team_str.parse().unwrap_or(0);
        let (_, steam) = extract_online_ids(line);
        return Some(ParsedEvent::TeamAssignment { player_name, steam64: steam, team_id, logged_at: ts });
    }

    // 9. 换图/设下一图
    if line.contains("ADMIN COMMAND:") && line.contains("Change layer to ") {
        let rest = line.split("Change layer to ").nth(1).unwrap_or("");
        let parts: Vec<&str> = rest.split_whitespace().collect();
        let layer_name = parts.first().map(|s| s.to_string()).unwrap_or_default();
        let factions: String = parts.iter().skip(1).map(|s| s.to_string()).collect::<Vec<_>>().join(" ");
        let mut team1 = String::new();
        let mut team2 = String::new();
        for part in &parts[1..] {
            if part.contains('+') {
                let code = part.split('+').next().unwrap_or("").to_string();
                if team1.is_empty() { team1 = code; } else { team2 = code; }
            }
        }
        let admin_name = line.split("  ").last().map(|s| s.trim().to_string()).unwrap_or_default();
        let raw = line.to_string();
        return Some(ParsedEvent::AdminAction { admin_name, action_type: "change_layer".to_string(), target: layer_name.clone(), message: factions, raw_line: raw, logged_at: ts });
    }

    // 10. 对局结算
    if line.contains("has won the match") && line.contains("on layer") {
        let team_id: Option<i32> = if line.contains("Team 1,") { Some(1) } else if line.contains("Team 2,") { Some(2) } else { None };
        let layer_name = line.split("on layer ").nth(1).and_then(|s| s.split('(').next()).map(|s| s.trim().to_string()).unwrap_or_default();
        let map_name = line.split("(level ").nth(1).and_then(|s| s.split(')').next()).map(|s| s.trim().to_string()).unwrap_or_default();
        let team1_faction = line.split("Team 1, ").nth(1).and_then(|s| s.split('(').next()).map(|s| s.trim().to_string()).unwrap_or_default();
        let team2_faction = line.split("Team 2, ").nth(1).and_then(|s| s.split('(').next()).map(|s| s.trim().to_string()).unwrap_or_default();
        if !layer_name.is_empty() {
            return Some(ParsedEvent::MatchEvent { map_name, layer_name, team1_faction, team2_faction, winner_team: team_id, event_type: "match_end".to_string(), logged_at: ts });
        }
    }

    // 11. 小队创建
    if line.contains("has created Squad ") {
        let player_name = line.split(" (Online IDs:").next().unwrap_or("").trim().to_string();
        let player_name = if let Some(pos) = player_name.rfind(". ") { player_name[pos+2..].to_string() } else { player_name };
        let squad_id = line.split("has created Squad ").nth(1).and_then(|s| s.split_whitespace().next()).map(|s| s.to_string()).unwrap_or_default();
        let squad_name = line.split("(Squad Name: ").nth(1).and_then(|s| s.split(')').next()).map(|s| s.to_string()).unwrap_or_default();
        let faction = line.split("on ").last().map(|s| s.trim().to_string()).unwrap_or_default();
        let (_, steam) = extract_online_ids(line);
        return Some(ParsedEvent::SquadCreation { player_name, steam64: steam, squad_id, squad_name, faction, logged_at: ts });
    }

    // 12. 部署/兵种
    if line.contains("DeployRole=") || (line.contains("OnPossess") && line.contains("Pawn=BP_Soldier")) {
        let player_name = if let Some(n) = line.find("PC=") {
            let r = &line[n+3..]; r.split(|c: char| c == ' ' || c == '(').next().unwrap_or("").to_string()
        } else { String::new() };
        let (_, steam) = extract_online_ids(line);
        let role = if let Some(r) = line.find("DeployRole=") {
            line[r+11..].split_whitespace().next().unwrap_or("").to_string()
        } else if let Some(r) = line.find("Pawn=BP_Soldier_") {
            let rest = &line[r+15..]; rest.split(|c: char| c == '_' || c == ' ').next().unwrap_or("").to_string()
        } else { "Unknown".to_string() };
        return Some(ParsedEvent::DeployRole { player_name, steam64: steam, role, logged_at: ts });
    }

    // 13. 救人
    if line.contains("has revived ") {
        let reviver = line.split(" has revived ").next().unwrap_or("").trim().to_string();
        let reviver_name = reviver.split(" (Online IDs:").next().unwrap_or(&reviver).trim().to_string();
        let (_, reviver_steam) = extract_online_ids(&reviver);
        let after = line.split("has revived ").nth(1).unwrap_or("");
        let revived_name = after.split(" (Online IDs:").next().unwrap_or("").trim().to_string();
        let (_, revived_steam) = extract_online_ids(after);
        return Some(ParsedEvent::ReviveEvent { reviver_name, reviver_steam64: reviver_steam, revived_name, revived_steam64: revived_steam, logged_at: ts });
    }

    // 14. 载具
    if line.contains("Entered Vehicle") || line.contains("Exited Vehicle") {
        let event_type = if line.contains("Entered") { "enter" } else { "exit" };
        let player_name = if let Some(p) = line.find("PC=") {
            line[p+3..].split(|c: char| c == ' ' || c == '(').next().unwrap_or("").to_string()
        } else { String::new() };
        let (_, steam) = extract_online_ids(line);
        let vehicle_name = if let Some(v) = line.find("Asset Name = ") {
            line[v+13..].split(')').next().unwrap_or("").trim().to_string()
        } else if let Some(v) = line.find("Pawn=") {
            let rest = &line[v+5..]; rest.split(|c: char| c == '_' || c == ' ').next().unwrap_or("").to_string()
        } else { "Unknown".to_string() };
        return Some(ParsedEvent::VehicleEvent { player_name, steam64: steam, vehicle_name, event_type: event_type.to_string(), logged_at: ts });
    }

    // 15. 管理员操作审计
    if line.contains("ADMIN COMMAND:") {
        let after_admin = line.split("ADMIN COMMAND:").nth(1).unwrap_or("").trim();
        let (action_type, target) = if after_admin.contains("Remote admin has warned") {
            ("warn".to_string(), after_admin.split("warned player").nth(1).and_then(|s| s.split('.').next()).map(|s| s.trim().to_string()).unwrap_or_default())
        } else if after_admin.contains("Change layer to") {
            ("change_layer".to_string(), String::new())
        } else if after_admin.contains("broadcasted") {
            ("broadcast".to_string(), String::new())
        } else {
            ("other".to_string(), String::new())
        };
        let admin_name = line.split("from RCON").next().and_then(|s| {
            let parts: Vec<&str> = s.rsplitn(3, ' ').collect();
            if parts.len() >= 2 { Some(parts[1].to_string()) } else { None }
        }).unwrap_or_default();
        let message = after_admin.split("Message was \"").nth(1).and_then(|s| s.split('"').next()).map(|s| s.to_string()).unwrap_or_default();
        return Some(ParsedEvent::AdminAction { admin_name, action_type, target, message, raw_line: line.to_string(), logged_at: ts });
    }

    // 16. 玩家死亡
    if line.contains("Die():") && line.contains("KillingDamage=") {
        let player_name = line.split("Player: ").nth(1).and_then(|s| s.split("KillingDamage=").next()).map(|s| s.trim().to_string()).unwrap_or_default();
        let (_, steam) = extract_online_ids(line);
        let weapon = line.split("caused by ").last().map(|s| s.split('_').next().unwrap_or(s).trim().to_string()).unwrap_or_default();
        let killer_steam = extract_online_ids(line).1;
        return Some(ParsedEvent::PlayerDeath { player_name, steam64: steam, killer_steam64: killer_steam, weapon, logged_at: ts });
    }

    // 17. 聊天消息: [Chat] PlayerName (SteamID): message
    // 也支持 RCON 事件格式: [ChatAll] / [ChatTeam] / ChatAll: 等
    if line.contains("[Chat") || line.to_lowercase().contains("chatall") || line.to_lowercase().contains("chatteam") {
        let content = if let Some(pos) = line.find("[Chat]") { &line[pos + 6..] }
            else if let Some(pos) = line.find("Chat:") { &line[pos + 5..] }
            else { line };
        let content = content.trim();
        if let Some(colon_pos) = content.find(": ") {
            let header = &content[..colon_pos];
            let message = content[colon_pos + 2..].trim().to_string();
            if let Some(paren_start) = header.rfind('(') {
                if let Some(paren_end) = header.rfind(')') {
                    let sid_str = &header[paren_start + 1..paren_end];
                    let player_name = header[..paren_start].trim().to_string();
                    if sid_str.chars().all(|c| c.is_ascii_digit()) && sid_str.len() >= 10 {
                        let channel = if line.to_lowercase().contains("team") { "Team" } else { "All" };
                        return Some(ParsedEvent::ChatMessage { player_name, steam64: sid_str.to_string(), message, channel: channel.to_string(), logged_at: ts });
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_login_request() {
        let line = "[2026.05.01-06.02.59:848][583]LogNet: Login request: ?Name=傻逼 userId: RedpointEOS:00024fa80525424f8b0f58481ba85f51 platform: RedpointEOS";
        let result = parse_line(line);
        assert!(matches!(result, Some(ParsedEvent::PlayerLogin { .. })));
    }

    #[test]
    fn test_parse_fly_event() {
        let line = "[2026.05.01-06.17.03:441][243]LogSquadTrace: [DedicatedServer]OnPossess(): PC=傻逼 (Online IDs: EOS: 00024fa80525424f8b0f58481ba85f51 steam: 76561198444088167) Pawn=BP_DeveloperAdminCam_C_2147477376";
        let result = parse_line(line);
        assert!(matches!(result, Some(ParsedEvent::FlyEvent { .. })));
    }

    #[test]
    fn test_parse_damage() {
        let line = "[2026.05.01-06.26.19:104][333]LogSquad: Player: axqr7078 ActualDamage=74.154121 from 黄金方便面 (Online IDs: EOS: 0002647b6c164f78957355de49c0e2a5 steam: 76561198844042513 | Player Controller ID: BP_PlayerController_C_2147476840)caused by BP_M67Frag_C_2147472997";
        let result = parse_line(line);
        assert!(matches!(result, Some(ParsedEvent::KillEvent { .. })));
    }
}
