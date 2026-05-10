use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// ═══ Filter Categories ═══

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterCategory {
    Racial,
    Homophobic,
    Ableist,
    Chinese,
    Custom,
}

impl FilterCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterCategory::Racial => "racial",
            FilterCategory::Homophobic => "homophobic",
            FilterCategory::Ableist => "ableist",
            FilterCategory::Chinese => "chinese",
            FilterCategory::Custom => "custom",
        }
    }
}

// ═══ Escalation Actions ═══

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationAction {
    pub violation_count: i32,
    pub action: String, // WARN, KICK, BAN
    #[serde(default)]
    pub ban_duration_days: Option<i32>,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatModerationSettings {
    pub id: i32,
    pub server_id: i32,
    pub enabled: bool,
    pub enable_racial_slurs: bool,
    pub enable_homophobic_slurs: bool,
    pub enable_ableist_language: bool,
    pub enable_chinese_slurs: bool,
    pub custom_blacklist: Vec<String>,
    pub whitelist: Vec<String>,
    pub escalation_actions: Vec<EscalationAction>,
    pub violation_expiry_days: i32,
    pub exempt_admins: bool,
    pub log_detections: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationRecord {
    pub steam_id: String,
    pub player_name: String,
    pub message: String,
    pub category: String,
    pub matched_word: String,
    pub action_taken: String,
}

/// Match result from filter
#[derive(Debug, Clone)]
pub struct FilterMatch {
    pub category: FilterCategory,
    pub matched_word: String,
}

// ═══ Built-in Word Lists ═══

fn racial_slurs_chinese() -> Vec<&'static str> {
    vec![
        // 针对黑人的侮辱词
        "黑鬼", "尼哥", "nigger", "nigga", "negro", "n1gga", "n1gger",
        "黑猩猩", "黑皮", "非洲黑", "黑奴",
        // 针对白人的侮辱词
        "白皮猪", "白猪", "白鬼",
        // 针对亚洲人的侮辱词
        "chingchong", "chink", "chinaman",
        // 针对日本人的侮辱词
        "小日本", "日本鬼子", "倭寇", "jap",
        // 针对韩国人的侮辱词
        "棒子", "高丽棒子",
        // 针对印度人的
        "阿三", "咖喱",
    ]
}

fn homophobic_slurs_chinese() -> Vec<&'static str> {
    vec![
        "死基佬", "人妖", "娘娘腔", "娘炮", "变态",
        "faggot", "fag", "f4ggot", "f@g",
        "dyke", "tranny",
    ]
}

fn ableist_slurs_chinese() -> Vec<&'static str> {
    vec![
        "残废", "瘸子", "瞎子", "聋子", "傻子",
        "脑残", "弱智", "智障", "白痴", "低能",
        "retard", "ret4rd", "r3tard",
        "spaz", "sp4z",
    ]
}

fn chinese_extra_slurs() -> Vec<&'static str> {
    vec![
        // 问候家人的
        "草泥马", "艹你", "操你", "日你", "干你",
        "tmd", "t n d", "他妈", "你妈", "尼玛",
        "cnm", "c n m",
        // 人身攻击
        "sb", "傻逼", "傻x", "傻b",
        "垃圾", "废物", "辣鸡", "辣鸡",
        "狗东西", "狗日的", "狗比",
        "畜生", "杂种", "杂碎",
        // 滚相关
        "滚蛋", "滚出", "滚开",
    ]
}

// ═══ Leet Speak Normalization ═══

fn normalize_text(text: &str) -> String {
    let lower = text.to_lowercase();
    // Remove zero-width characters and common separators used for evasion
    let cleaned: String = lower.chars()
        .filter(|c| !c.is_whitespace() || *c == ' ')
        .map(|c| match c {
            '@' => 'a',
            '4' => 'a',
            '3' => 'e',
            '1' => 'i',
            '!' => 'i',
            '|' => 'i',
            '0' => 'o',
            '5' => 's',
            '$' => 's',
            '7' => 't',
            '+' => 't',
            '2' => 'z',
            '8' => 'b',
            '6' => 'g',
            '9' => 'g',
            _ => c,
        })
        .collect();
    // Reduce repeated chars: "niiigga" → "niga"
    let mut reduced = String::new();
    let mut prev = '\0';
    let mut count = 0u32;
    for c in cleaned.chars() {
        if c == prev {
            count += 1;
            if count <= 2 {
                reduced.push(c);
            }
        } else {
            count = 1;
            reduced.push(c);
        }
        prev = c;
    }
    reduced
}

// ═══ Chat Automod Service ═══

pub struct ChatAutomod {
    settings_cache: HashMap<i32, ChatModerationSettings>,
    admin_cache: HashMap<i32, Vec<String>>, // server_id → admin steam_ids
    compiled_blacklist: HashMap<i32, Vec<Regex>>,
}

impl ChatAutomod {
    pub fn new() -> Self {
        Self {
            settings_cache: HashMap::new(),
            admin_cache: HashMap::new(),
            compiled_blacklist: HashMap::new(),
        }
    }

    /// Load settings for all servers
    pub async fn load_settings(&mut self, pool: &PgPool) {
        let rows = sqlx::query_as::<_, (i32, bool, bool, bool, bool, bool, Vec<String>, Vec<String>, serde_json::Value, i32, bool, bool)>(
            "SELECT server_id, enabled, enable_racial_slurs, enable_homophobic_slurs, enable_ableist_language, \
             enable_chinese_slurs, custom_blacklist, whitelist, escalation_actions, \
             violation_expiry_days, exempt_admins, log_detections \
             FROM chat_moderation_settings WHERE enabled=true"
        ).fetch_all(pool).await.unwrap_or_default();

        self.settings_cache.clear();
        self.compiled_blacklist.clear();

        for (sid, enabled, racial, homo, ableist, chinese, blacklist, whitelist, actions, expiry, exempt, log) in rows {
            let escalation: Vec<EscalationAction> = serde_json::from_value(actions).unwrap_or_default();

            // Compile custom blacklist regexes
            let mut compiled = Vec::new();
            for pattern in &blacklist {
                if let Ok(re) = Regex::new(&format!("(?i){}", regex::escape(pattern))) {
                    compiled.push(re);
                }
            }

            self.settings_cache.insert(sid, ChatModerationSettings {
                id: 0, server_id: sid, enabled,
                enable_racial_slurs: racial,
                enable_homophobic_slurs: homo,
                enable_ableist_language: ableist,
                enable_chinese_slurs: chinese,
                custom_blacklist: blacklist,
                whitelist,
                escalation_actions: escalation,
                violation_expiry_days: expiry,
                exempt_admins: exempt,
                log_detections: log,
            });
            self.compiled_blacklist.insert(sid, compiled);
        }
    }

    /// Refresh admin cache
    pub async fn refresh_admin_cache(&mut self, pool: &PgPool) {
        let rows = sqlx::query_as::<_, (i32, String)>(
            "SELECT server_id, steam_id FROM permission_admins"
        ).fetch_all(pool).await.unwrap_or_default();

        self.admin_cache.clear();
        for (sid, steam_id) in rows {
            self.admin_cache.entry(sid).or_default().push(steam_id);
        }
    }

    /// Check a chat message for violations.
    /// Returns (filter_match, current_violation_count) if a violation is found.
    pub async fn check_message(
        &self,
        pool: &PgPool,
        server_id: i32,
        player_name: &str,
        steam_id: &str,
        message: &str,
        chat_channel: &str,
    ) -> Option<(FilterMatch, i32)> {
        let settings = self.settings_cache.get(&server_id)?;

        // Skip admin/admin chat
        if chat_channel.contains("Admin") || chat_channel.contains("admin") {
            return None;
        }

        // Admin exemption
        if settings.exempt_admins {
            if let Some(admins) = self.admin_cache.get(&server_id) {
                if admins.contains(&steam_id.to_string()) {
                    return None;
                }
            }
        }

        // Whitelist check
        let has_whitelist = settings.whitelist.iter().any(|w| {
            message.to_lowercase().contains(&w.to_lowercase())
        });
        if has_whitelist {
            return None;
        }

        let normalized = normalize_text(message);

        // Check filters in priority order
        if settings.enable_racial_slurs {
            if let Some(m) = self.check_category(&normalized, &racial_slurs_chinese(), FilterCategory::Racial) {
                let violations = self.count_violations(pool, server_id, steam_id).await;
                return Some((m, violations + 1));
            }
        }
        if settings.enable_homophobic_slurs {
            if let Some(m) = self.check_category(&normalized, &homophobic_slurs_chinese(), FilterCategory::Homophobic) {
                let violations = self.count_violations(pool, server_id, steam_id).await;
                return Some((m, violations + 1));
            }
        }
        if settings.enable_ableist_language {
            if let Some(m) = self.check_category(&normalized, &ableist_slurs_chinese(), FilterCategory::Ableist) {
                let violations = self.count_violations(pool, server_id, steam_id).await;
                return Some((m, violations + 1));
            }
        }
        if settings.enable_chinese_slurs {
            if let Some(m) = self.check_category(&normalized, &chinese_extra_slurs(), FilterCategory::Chinese) {
                let violations = self.count_violations(pool, server_id, steam_id).await;
                return Some((m, violations + 1));
            }
        }

        // Custom blacklist
        if let Some(patterns) = self.compiled_blacklist.get(&server_id) {
            for re in patterns {
                if let Some(cap) = re.find(message) {
                    let m = FilterMatch {
                        category: FilterCategory::Custom,
                        matched_word: cap.as_str().to_string(),
                    };
                    let violations = self.count_violations(pool, server_id, steam_id).await;
                    return Some((m, violations + 1));
                }
            }
        }

        None
    }

    /// Determine what action to take based on violation count
    pub fn determine_action(&self, server_id: i32, violation_count: i32) -> Option<EscalationAction> {
        let settings = self.settings_cache.get(&server_id)?;
        let actions = &settings.escalation_actions;

        // Find exact match first
        if let Some(a) = actions.iter().find(|a| a.violation_count == violation_count) {
            return Some(a.clone());
        }
        // Find highest action below current count
        let mut best: Option<&EscalationAction> = None;
        for a in actions {
            if a.violation_count <= violation_count {
                match best {
                    None => best = Some(a),
                    Some(ref b) if a.violation_count > b.violation_count => best = Some(a),
                    _ => {}
                }
            }
        }
        best.cloned()
    }

    /// Build RCON command for the action
    pub fn build_rcon_command(&self, action: &EscalationAction, player_name: &str, steam_id: &str) -> String {
        let msg = &action.message;
        match action.action.as_str() {
            "WARN" => format!("AdminWarn \"{}\" \"{}\"", steam_id, msg),
            "KICK" => format!("AdminKick \"{}\" \"{}\"", steam_id, msg),
            "BAN" => {
                let days = action.ban_duration_days.unwrap_or(1);
                let minutes = days * 24 * 60;
                format!("AdminBan \"{}\" {} \"{}\"", steam_id, minutes, msg)
            }
            _ => format!("AdminWarn \"{}\" \"{}\"", steam_id, msg),
        }
    }

    /// Record a violation
    pub async fn record_violation(
        &self,
        pool: &PgPool,
        server_id: i32,
        steam_id: &str,
        player_name: &str,
        message: &str,
        category: &str,
        matched_word: &str,
        action: &str,
    ) {
        let _ = sqlx::query(
            "INSERT INTO chat_violations (server_id, steam_id, player_name, message, category, matched_word, action_taken) \
             VALUES ($1,$2,$3,$4,$5,$6,$7)"
        ).bind(server_id).bind(steam_id).bind(player_name).bind(message)
         .bind(category).bind(matched_word).bind(action)
         .execute(pool).await;
    }

    /// Count active (non-expired) violations for a player
    async fn count_violations(&self, pool: &PgPool, server_id: i32, steam_id: &str) -> i32 {
        let expiry = self.settings_cache.get(&server_id)
            .map(|s| s.violation_expiry_days)
            .unwrap_or(30);

        if expiry == 0 {
            // Never expire
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM chat_violations WHERE server_id=$1 AND steam_id=$2"
            ).bind(server_id).bind(steam_id).fetch_one(pool).await.unwrap_or(0) as i32
        } else {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM chat_violations WHERE server_id=$1 AND steam_id=$2 \
                 AND logged_at > NOW() - INTERVAL '1 day' * $3"
            ).bind(server_id).bind(steam_id).bind(expiry as i32)
             .fetch_one(pool).await.unwrap_or(0) as i32
        }
    }

    // ═══ Internal ═══

    fn check_category(&self, text: &str, words: &[&str], category: FilterCategory) -> Option<FilterMatch> {
        for &word in words {
            if text.contains(&word.to_lowercase()) {
                return Some(FilterMatch {
                    category,
                    matched_word: word.to_string(),
                });
            }
        }
        None
    }
}

// ═══ API: List violations ═══

pub async fn list_violations(
    pool: &PgPool,
    server_id: i32,
    page: i64,
    per_page: i64,
) -> Result<(Vec<ViolationRecord>, i64), sqlx::Error> {
    let offset = (page - 1) * per_page;
    let (total,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM chat_violations WHERE server_id=$1"
    ).bind(server_id).fetch_one(pool).await?;

    let rows = sqlx::query_as::<_, (String, String, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT steam_id, player_name, message, category, matched_word, action_taken, logged_at \
         FROM chat_violations WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    ).bind(server_id).bind(per_page).bind(offset).fetch_all(pool).await?;

    let records = rows.into_iter().map(|(sid, name, msg, cat, word, action, _ts)| {
        ViolationRecord { steam_id: sid, player_name: name, message: msg, category: cat, matched_word: word, action_taken: action }
    }).collect();

    Ok((records, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_leetspeak() {
        assert_eq!(normalize_text("n1gg3r"), "nigger");
        assert_eq!(normalize_text("f@g"), "fag");
        assert_eq!(normalize_text("r3t4rd"), "retard");
        assert_eq!(normalize_text("sh1t"), "shit");
    }

    #[test]
    fn test_normalize_repeated_chars() {
        assert_eq!(normalize_text("niiiggggeeerrr"), "niiggeerr");
        assert_eq!(normalize_text("fuuuuck"), "fuuck");
    }

    #[test]
    fn test_racial_slur_detection() {
        let automod = ChatAutomod::new();
        let text = normalize_text("nigger");
        assert!(automod.check_category(&text, &racial_slurs_chinese(), FilterCategory::Racial).is_some());
    }

    #[test]
    fn test_chinese_slur_detection() {
        let automod = ChatAutomod::new();
        let text = normalize_text("傻逼");
        assert!(automod.check_category(&text, &chinese_extra_slurs(), FilterCategory::Chinese).is_some());
    }

    #[test]
    fn test_clean_message_passes() {
        let automod = ChatAutomod::new();
        let text = normalize_text("hello world");
        assert!(automod.check_category(&text, &racial_slurs_chinese(), FilterCategory::Racial).is_none());
        assert!(automod.check_category(&text, &chinese_extra_slurs(), FilterCategory::Chinese).is_none());
    }

    #[test]
    fn test_evasion_detection() {
        // n1gg3r with leet speak should still match
        let text = normalize_text("n1gg3r");
        assert!(check_racial(&text));
    }

    fn check_racial(text: &str) -> bool {
        let automod = ChatAutomod::new();
        automod.check_category(text, &racial_slurs_chinese(), FilterCategory::Racial).is_some()
    }

    #[test]
    fn test_determine_action_exact() {
        let mut automod = ChatAutomod::new();
        automod.settings_cache.insert(1, ChatModerationSettings {
            id: 0, server_id: 1, enabled: true,
            enable_racial_slurs: true,
            enable_homophobic_slurs: false,
            enable_ableist_language: false,
            enable_chinese_slurs: false,
            custom_blacklist: vec![],
            whitelist: vec![],
            escalation_actions: vec![
                EscalationAction { violation_count: 1, action: "WARN".into(), ban_duration_days: None, message: "警告".into() },
                EscalationAction { violation_count: 3, action: "KICK".into(), ban_duration_days: None, message: "踢出".into() },
                EscalationAction { violation_count: 5, action: "BAN".into(), ban_duration_days: Some(1), message: "封禁".into() },
            ],
            violation_expiry_days: 30,
            exempt_admins: true,
            log_detections: true,
        });

        let action = automod.determine_action(1, 1).unwrap();
        assert_eq!(action.action, "WARN");

        let action = automod.determine_action(1, 2).unwrap();
        assert_eq!(action.action, "WARN"); // falls back to highest below

        let action = automod.determine_action(1, 3).unwrap();
        assert_eq!(action.action, "KICK");

        let action = automod.determine_action(1, 10).unwrap();
        assert_eq!(action.action, "BAN");
    }

    #[test]
    fn test_build_rcon_commands() {
        let automod = ChatAutomod::new();
        let warn = EscalationAction { violation_count: 1, action: "WARN".into(), ban_duration_days: None, message: "请文明用语".into() };
        let kick = EscalationAction { violation_count: 3, action: "KICK".into(), ban_duration_days: None, message: "被踢出".into() };
        let ban = EscalationAction { violation_count: 5, action: "BAN".into(), ban_duration_days: Some(3), message: "被封禁".into() };

        assert!(automod.build_rcon_command(&warn, "test", "123").contains("AdminWarn"));
        assert!(automod.build_rcon_command(&kick, "test", "123").contains("AdminKick"));
        assert!(automod.build_rcon_command(&ban, "test", "123").contains("AdminBan"));
        assert!(automod.build_rcon_command(&ban, "test", "123").contains("4320")); // 3 days in minutes
    }
}
