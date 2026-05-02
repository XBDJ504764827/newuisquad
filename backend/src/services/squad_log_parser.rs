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
