use crate::event_manager::{Event, EventType};
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use std::sync::LazyLock;

/// 解析结果
pub struct ParsedEvent {
    pub event_type: EventType,
    pub data: serde_json::Value,
    pub raw_log: String,
    pub timestamp: DateTime<Utc>,
}

/// 日志解析器
struct LogParser {
    regex: Regex,
    event_type: EventType,
    extract: fn(&Regex, &str) -> Option<serde_json::Value>,
}

/// Squad 日志时间戳格式: 2024.01.02-15.04.05:000
fn parse_squad_timestamp(raw: &str) -> Option<DateTime<Utc>> {
    // 尝试精确格式: 2024.01.02-15.04.05:000
    if let Ok(dt) = NaiveDateTime::parse_from_str(raw, "%Y.%m.%d-%H.%M.%S:%3f") {
        return Some(dt.and_utc());
    }
    // 尝试无毫秒格式: 2024.01.02-15.04.05
    if let Ok(dt) = NaiveDateTime::parse_from_str(raw, "%Y.%m.%d-%H.%M.%S") {
        return Some(dt.and_utc());
    }
    None
}

/// 从日志行中提取时间戳
fn extract_timestamp(line: &str) -> DateTime<Utc> {
    // 格式: [2024.01.02-15.04.05:000][...]
    if let Some(start) = line.find('[') {
        if let Some(end) = line[start + 1..].find(']') {
            let raw = &line[start + 1..start + 1 + end];
            if let Some(dt) = parse_squad_timestamp(raw) {
                return dt;
            }
        }
    }
    Utc::now()
}

/// 从日志行中提取 chain ID
fn extract_chain_id(line: &str) -> String {
    // 格式: [...][  123]...
    let bytes = line.as_bytes();
    let mut bracket_count = 0;
    let mut start = 0;
    let mut end = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'[' {
            bracket_count += 1;
            if bracket_count == 2 {
                start = i + 1;
            }
        } else if b == b']' && bracket_count == 2 {
            end = i;
            break;
        }
    }
    if end > start {
        line[start..end].trim().to_string()
    } else {
        String::new()
    }
}

/// 从 Online IDs 字符串中提取 EOS 和 Steam ID
fn extract_online_ids(ids_str: &str) -> (String, String) {
    let mut eos_id = String::new();
    let mut steam_id = String::new();

    // 尝试提取 EOS ID
    if let Some(pos) = ids_str.find("EOS: ") {
        let rest = &ids_str[pos + 5..];
        eos_id = rest.split_whitespace().next().unwrap_or("").to_string();
    } else if let Some(pos) = ids_str.find("EOS:") {
        let rest = &ids_str[pos + 4..];
        eos_id = rest.split_whitespace().next().unwrap_or("").to_string();
    }

    // 尝试提取 Steam ID
    if let Some(pos) = ids_str.find("steam: ") {
        let rest = &ids_str[pos + 7..];
        steam_id = rest.split_whitespace().next().unwrap_or("").to_string();
    } else if let Some(pos) = ids_str.find("steam:") {
        let rest = &ids_str[pos + 6..];
        steam_id = rest.split_whitespace().next().unwrap_or("").to_string();
    } else if let Some(pos) = ids_str.find("Steam: ") {
        let rest = &ids_str[pos + 7..];
        steam_id = rest.split_whitespace().next().unwrap_or("").to_string();
    } else if let Some(pos) = ids_str.find("Steam:") {
        let rest = &ids_str[pos + 6..];
        steam_id = rest.split_whitespace().next().unwrap_or("").to_string();
    }

    (eos_id, steam_id)
}

/// 规范化 actor 类名（去除 _C_12345 后缀）
fn normalize_actor_class_name(actor: &str) -> String {
    let actor = actor.trim();
    if let Some(pos) = actor.find("_C_") {
        if pos > 0 {
            return actor[..pos].to_string();
        }
    }
    actor.trim_end_matches("_C").to_string()
}

/// 规范化伤害来源
fn normalize_damage_causer(causer: &str) -> String {
    let causer = causer.trim();
    if causer.is_empty() || causer.eq_ignore_ascii_case("nullptr") {
        return String::new();
    }
    normalize_actor_class_name(causer)
}

// 预编译正则表达式
static RE_PLAYER_CONNECTED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquad: PostLogin: NewPlayer: BP_PlayerController_C .+PersistentLevel\.([^\s]+) \(IP: ([^|)]+?)\s*\|\s*Online IDs:(?:\s*EOS: ([^ )]+))?(?:\s*steam: ([^ )]+))?\)").unwrap()
});

static RE_PLAYER_CONNECTED_ALT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"has connected\. \(Online IDs: (.+)\)").unwrap()
});

static RE_PLAYER_DISCONNECTED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquad: (.+) \(Online IDs: (.+)\) has disconnected\.").unwrap()
});

static RE_PLAYER_DISCONNECTED_ALT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"has disconnected\. \(Online IDs: (.+)\)").unwrap()
});

static RE_PLAYER_WOUNDED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquad: Player:(.+) ActualDamage=([0-9.]+) from (.+) \(Online IDs:(?:(?: EOS: ([^ )|]+))?(?: steam: ([^ )|]+))?| INVALID)\s*\|\s*Player Controller ID: ([^ )]+)\)caused by ([A-Za-z0-9_-]+(?:_C(?:_[0-9]+)?)?|nullptr)").unwrap()
});

static RE_PLAYER_DIED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquadTrace: \[DedicatedServer\](?:ASQSoldier::)?Die\(\): Player:(.+) KillingDamage=(?:-)*([0-9.]+) from (.+?) \(Online IDs:(?:(?: EOS: ([^ )|]+))?(?: steam: ([^ )|]+))?| INVALID)\s*\| Cont(?:r)?oller ID: ([\w\d]+)\) caused by ([A-Za-z0-9_-]+(?:_C(?:_[0-9]+)?)?|nullptr)").unwrap()
});

static RE_PLAYER_REVIVED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquad: (.+) revived by (.+)").unwrap()
});

static RE_PLAYER_POSSESS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquad: Possess actor (.+) by (.+)").unwrap()
});

static RE_DEPLOYABLE_DAMAGED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquadTrace: \[DedicatedServer\](?:ASQDeployable::)?TakeDamage\(\): ([A-Za-z0-9_-]+)_C_[0-9]+: ([0-9.]+) damage attempt by causer ([A-Za-z0-9_-]+)_C_[0-9]+ instigator (.+) with damage type ([A-Za-z0-9_-]+)_C health remaining ([0-9.]+)").unwrap()
});

static RE_ADMIN_BROADCAST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquad: ADMIN COMMAND: Message broadcasted <(.+)> from (.+)").unwrap()
});

static RE_TICK_RATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquadTrace: \[DedicatedServer\] TickRate = ([0-9.]+)").unwrap()
});

static RE_GAME_EVENT_TICKET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquadGameEvents: Display: Team ([0-9]), (.*) \( ?(.*?) ?\) has (won|lost) the match with ([0-9]+) Tickets on layer (.*) \(level (.*)\)!").unwrap()
});

static RE_GAME_EVENT_MATCH_WINNER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquadTrace: \[DedicatedServer\](?:ASQGameMode::)?DetermineMatchWinner\(\): (.+) won on (.+)").unwrap()
});

static RE_GAME_EVENT_ROUND_ENDED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogGameState: Match State Changed from InProgress to WaitingPostMatch").unwrap()
});

static RE_GAME_EVENT_NEW_GAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogWorld: Bringing World /([A-z0-9]+)/([A-z0-9-]+)/([A-z0-9-]+)").unwrap()
});

static RE_JOIN_SUCCEEDED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[([0-9.:-]+)\]\[([ 0-9]*)\]LogSquad: (.+) \(Online IDs: (.+)\) joined the server successfully\.").unwrap()
});

/// 处理日志行，返回结构化事件
pub fn process_log_line(line: &str) -> Option<ParsedEvent> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // 去除 NUL 字节（Windows 日志传输可能包含）
    let line = if line.contains('\0') {
        line.replace('\0', "")
    } else {
        line.to_string()
    };
    let line = line.trim();

    let timestamp = extract_timestamp(line);

    // 按优先级尝试各解析器

    // PlayerConnected (PostLogin)
    if let Some(caps) = RE_PLAYER_CONNECTED.captures(&line) {
        let controller = caps.get(3).map_or("", |m| m.as_str());
        let ip = caps.get(4).map_or("", |m| m.as_str());
        let eos_id = caps.get(5).map_or("", |m| m.as_str());
        let steam_id = caps.get(6).map_or("", |m| m.as_str());
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        return Some(ParsedEvent {
            event_type: EventType::LogPlayerConnected,
            data: serde_json::json!({
                "chain_id": chain_id,
                "player_controller": controller,
                "ip": ip,
                "eos_id": eos_id,
                "steam_id": steam_id,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // PlayerDisconnected
    if let Some(caps) = RE_PLAYER_DISCONNECTED.captures(&line) {
        let name = caps.get(3).map_or("", |m| m.as_str());
        let ids_str = caps.get(4).map_or("", |m| m.as_str());
        let (eos_id, steam_id) = extract_online_ids(ids_str);
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        return Some(ParsedEvent {
            event_type: EventType::LogPlayerDisconnected,
            data: serde_json::json!({
                "chain_id": chain_id,
                "name": name,
                "eos_id": eos_id,
                "steam_id": steam_id,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // PlayerDied
    if let Some(caps) = RE_PLAYER_DIED.captures(&line) {
        let victim_name = caps.get(3).map_or("", |m| m.as_str()).trim();
        let damage = caps.get(4).map_or("", |m| m.as_str());
        let attacker_name = caps.get(5).map_or("", |m| m.as_str()).trim();
        let attacker_eos = caps.get(6).map_or("", |m| m.as_str());
        let attacker_steam = caps.get(7).map_or("", |m| m.as_str());
        let controller_id = caps.get(8).map_or("", |m| m.as_str());
        let weapon_raw = caps.get(9).map_or("", |m| m.as_str());
        let weapon = normalize_damage_causer(weapon_raw);
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        // 检查 INVALID
        if attacker_steam.contains("INVALID") {
            return None;
        }

        return Some(ParsedEvent {
            event_type: EventType::LogPlayerDied,
            data: serde_json::json!({
                "chain_id": chain_id,
                "victim_name": victim_name,
                "damage": damage,
                "attacker_name": attacker_name,
                "attacker_eos_id": attacker_eos,
                "attacker_steam_id": attacker_steam,
                "attacker_controller": controller_id,
                "weapon": weapon,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // PlayerWounded (PlayerDamaged)
    if let Some(caps) = RE_PLAYER_WOUNDED.captures(&line) {
        let victim_name = caps.get(3).map_or("", |m| m.as_str()).trim();
        let damage = caps.get(4).map_or("", |m| m.as_str());
        let attacker_name = caps.get(5).map_or("", |m| m.as_str()).trim();
        let attacker_eos = caps.get(6).map_or("", |m| m.as_str());
        let attacker_steam = caps.get(7).map_or("", |m| m.as_str());
        let controller_id = caps.get(8).map_or("", |m| m.as_str());
        let weapon_raw = caps.get(9).map_or("", |m| m.as_str());
        let weapon = normalize_damage_causer(weapon_raw);
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        if attacker_steam.contains("INVALID") {
            return None;
        }

        return Some(ParsedEvent {
            event_type: EventType::LogPlayerDamaged,
            data: serde_json::json!({
                "chain_id": chain_id,
                "victim_name": victim_name,
                "damage": damage,
                "attacker_name": attacker_name,
                "attacker_eos_id": attacker_eos,
                "attacker_steam_id": attacker_steam,
                "attacker_controller": controller_id,
                "weapon": weapon,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // PlayerRevived
    if let Some(caps) = RE_PLAYER_REVIVED.captures(&line) {
        let victim = caps.get(3).map_or("", |m| m.as_str());
        let reviver = caps.get(4).map_or("", |m| m.as_str());
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        return Some(ParsedEvent {
            event_type: EventType::LogPlayerRevived,
            data: serde_json::json!({
                "chain_id": chain_id,
                "victim": victim,
                "reviver": reviver,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // PlayerPossess
    if let Some(caps) = RE_PLAYER_POSSESS.captures(&line) {
        let actor = caps.get(3).map_or("", |m| m.as_str());
        let player = caps.get(4).map_or("", |m| m.as_str());
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        return Some(ParsedEvent {
            event_type: EventType::LogPlayerPossess,
            data: serde_json::json!({
                "chain_id": chain_id,
                "actor": actor,
                "player": player,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // DeployableDamaged
    if let Some(caps) = RE_DEPLOYABLE_DAMAGED.captures(&line) {
        let deployable = caps.get(3).map_or("", |m| m.as_str());
        let damage = caps.get(4).map_or("", |m| m.as_str());
        let weapon = caps.get(5).map_or("", |m| m.as_str());
        let instigator = caps.get(6).map_or("", |m| m.as_str());
        let damage_type = caps.get(7).map_or("", |m| m.as_str());
        let health = caps.get(8).map_or("", |m| m.as_str());
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        return Some(ParsedEvent {
            event_type: EventType::LogDeployableDamaged,
            data: serde_json::json!({
                "chain_id": chain_id,
                "deployable": deployable,
                "damage": damage,
                "weapon": weapon,
                "instigator": instigator,
                "damage_type": damage_type,
                "health_remaining": health,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // AdminBroadcast
    if let Some(caps) = RE_ADMIN_BROADCAST.captures(&line) {
        let message = caps.get(3).map_or("", |m| m.as_str());
        let from = caps.get(4).map_or("", |m| m.as_str());
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        return Some(ParsedEvent {
            event_type: EventType::LogAdminBroadcast,
            data: serde_json::json!({
                "chain_id": chain_id,
                "message": message,
                "from": from,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // TickRate
    if let Some(caps) = RE_TICK_RATE.captures(&line) {
        let tick_rate = caps.get(3).map_or("", |m| m.as_str());
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        return Some(ParsedEvent {
            event_type: EventType::LogTickRate,
            data: serde_json::json!({
                "chain_id": chain_id,
                "tick_rate": tick_rate,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // GameEvent: Ticket update
    if let Some(caps) = RE_GAME_EVENT_TICKET.captures(&line) {
        let team = caps.get(3).map_or("", |m| m.as_str());
        let subfaction = caps.get(4).map_or("", |m| m.as_str());
        let faction = caps.get(5).map_or("", |m| m.as_str());
        let action = caps.get(6).map_or("", |m| m.as_str());
        let tickets = caps.get(7).map_or("", |m| m.as_str());
        let layer = caps.get(8).map_or("", |m| m.as_str());
        let level = caps.get(9).map_or("", |m| m.as_str());
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        return Some(ParsedEvent {
            event_type: EventType::LogGameEventUnified,
            data: serde_json::json!({
                "chain_id": chain_id,
                "event_type": "TICKET_UPDATE",
                "team": team,
                "subfaction": subfaction,
                "faction": faction,
                "action": action,
                "tickets": tickets,
                "layer": layer,
                "level": level,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // GameEvent: Match winner
    if let Some(caps) = RE_GAME_EVENT_MATCH_WINNER.captures(&line) {
        let winner = caps.get(3).map_or("", |m| m.as_str());
        let layer = caps.get(4).map_or("", |m| m.as_str());
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        return Some(ParsedEvent {
            event_type: EventType::LogGameEventUnified,
            data: serde_json::json!({
                "chain_id": chain_id,
                "event_type": "MATCH_WINNER",
                "winner": winner,
                "layer": layer,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // GameEvent: Round ended
    if RE_GAME_EVENT_ROUND_ENDED.is_match(&line) {
        let chain_id = extract_chain_id(&line);
        return Some(ParsedEvent {
            event_type: EventType::LogGameEventUnified,
            data: serde_json::json!({
                "chain_id": chain_id,
                "event_type": "ROUND_ENDED",
                "from_state": "InProgress",
                "to_state": "WaitingPostMatch",
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // GameEvent: New game
    if let Some(caps) = RE_GAME_EVENT_NEW_GAME.captures(&line) {
        let dlc = caps.get(3).map_or("", |m| m.as_str());
        let map_classname = caps.get(4).map_or("", |m| m.as_str());
        let layer_classname = caps.get(5).map_or("", |m| m.as_str());
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        if layer_classname == "TransitionMap" {
            return None;
        }

        return Some(ParsedEvent {
            event_type: EventType::LogGameEventUnified,
            data: serde_json::json!({
                "chain_id": chain_id,
                "event_type": "NEW_GAME",
                "dlc": dlc,
                "map_classname": map_classname,
                "layer_classname": layer_classname,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // JoinSucceeded
    if let Some(caps) = RE_JOIN_SUCCEEDED.captures(&line) {
        let name = caps.get(3).map_or("", |m| m.as_str());
        let ids_str = caps.get(4).map_or("", |m| m.as_str());
        let (eos_id, steam_id) = extract_online_ids(ids_str);
        let chain_id = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

        return Some(ParsedEvent {
            event_type: EventType::LogJoinSucceeded,
            data: serde_json::json!({
                "chain_id": chain_id,
                "name": name,
                "eos_id": eos_id,
                "steam_id": steam_id,
            }),
            raw_log: line.to_string(),
            timestamp,
        });
    }

    // 备用格式：PlayerConnected (简单格式)
    if RE_PLAYER_CONNECTED_ALT.is_match(&line) {
        if let Some(caps) = RE_PLAYER_CONNECTED_ALT.captures(&line) {
            let ids_str = caps.get(1).map_or("", |m| m.as_str());
            let (eos_id, steam_id) = extract_online_ids(ids_str);
            // 尝试从行中提取玩家名
            let name = line.split("has connected").next().unwrap_or("").trim().to_string();

            return Some(ParsedEvent {
                event_type: EventType::LogPlayerConnected,
                data: serde_json::json!({
                    "name": name,
                    "eos_id": eos_id,
                    "steam_id": steam_id,
                }),
                raw_log: line.to_string(),
                timestamp,
            });
        }
    }

    // 备用格式：PlayerDisconnected (简单格式)
    if RE_PLAYER_DISCONNECTED_ALT.is_match(&line) {
        if let Some(caps) = RE_PLAYER_DISCONNECTED_ALT.captures(&line) {
            let ids_str = caps.get(1).map_or("", |m| m.as_str());
            let (eos_id, steam_id) = extract_online_ids(ids_str);
            let name = line.split("has disconnected").next().unwrap_or("").trim().to_string();

            return Some(ParsedEvent {
                event_type: EventType::LogPlayerDisconnected,
                data: serde_json::json!({
                    "name": name,
                    "eos_id": eos_id,
                    "steam_id": steam_id,
                }),
                raw_log: line.to_string(),
                timestamp,
            });
        }
    }

    None
}

/// 将 ParsedEvent 转换为 EventManager 的 Event
pub fn parsed_to_event(parsed: ParsedEvent) -> Event {
    Event::new(parsed.event_type, parsed.data, Some(parsed.raw_log))
}