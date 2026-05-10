use serde::{Deserialize, Serialize};

/// Server rule (hierarchical tree)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRule {
    pub id: i32,
    pub server_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i32>,
    pub display_order: i32,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub actions: Vec<ServerRuleAction>,
    pub sub_rules: Vec<ServerRule>,
}

/// Action taken when a rule is violated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRuleAction {
    pub id: i32,
    pub violation_count: i32,
    pub action_type: String, // WARN, KICK, BAN
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_days: Option<i32>,
    #[serde(default)]
    pub message: String,
}

/// MOTD configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotdConfig {
    pub server_id: i32,
    #[serde(default)]
    pub prefix_text: String,
    #[serde(default)]
    pub suffix_text: String,
    #[serde(default)]
    pub auto_generate_from_rules: bool,
    #[serde(default)]
    pub include_rule_descriptions: bool,
}

impl Default for MotdConfig {
    fn default() -> Self {
        Self {
            server_id: 0,
            prefix_text: "=== 服务器规则 ===\n".to_string(),
            suffix_text: "\n=== 祝您游戏愉快 ===".to_string(),
            auto_generate_from_rules: true,
            include_rule_descriptions: true,
        }
    }
}

/// MOTD Generator — generates formatted MOTD text from hierarchical rules
pub struct MotdGenerator;

impl MotdGenerator {
    /// Generate full MOTD text
    pub fn generate(config: &MotdConfig, rules: &[ServerRule]) -> String {
        let mut result = String::new();

        // Prefix
        if !config.prefix_text.is_empty() {
            result.push_str(&config.prefix_text);
            if !config.prefix_text.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
        }

        // Rules
        if config.auto_generate_from_rules {
            Self::write_rules(&mut result, rules, config.include_rule_descriptions, "", 1);
        }

        // Suffix
        if !config.suffix_text.is_empty() {
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
            result.push_str(&config.suffix_text);
        }

        result
    }

    /// Recursively write rules with hierarchical numbering
    fn write_rules(
        builder: &mut String,
        rules: &[ServerRule],
        include_descriptions: bool,
        prefix: &str,
        depth: u32,
    ) {
        for (i, rule) in rules.iter().enumerate() {
            let number = format!("{}{}", prefix, i + 1);

            // Rule title with indentation
            let indent = "  ".repeat((depth - 1) as usize);
            builder.push_str(&format!("{}{}. {}\n", indent, number, rule.title));

            // Description as bullet
            if include_descriptions && !rule.description.is_empty() {
                builder.push_str(&format!("{}   • {}\n", indent, rule.description));
            }

            // Action descriptions
            for action in &rule.actions {
                let action_desc = match action.action_type.as_str() {
                    "WARN" => format!("警告"),
                    "KICK" => format!("踢出"),
                    "BAN" => {
                        if let Some(days) = action.duration_days {
                            if days == 0 { "永久封禁".to_string() } else { format!("封禁 {} 天", days) }
                        } else { "封禁".to_string() }
                    }
                    _ => action.action_type.clone(),
                };
                let msg = if action.message.is_empty() { String::new() } else { format!(": {}", action.message) };
                builder.push_str(&format!("{}     └ 第{}次违规 → {}{}{}\n",
                    indent, action.violation_count, action_desc, msg,
                    if action.violation_count > 1 { " (升级)" } else { "" }
                ));
            }

            // Recursive sub-rules
            if !rule.sub_rules.is_empty() {
                builder.push('\n');
                Self::write_rules(builder, &rule.sub_rules, include_descriptions, &format!("{}.", number), depth + 1);
            }

            builder.push('\n');
        }
    }

    /// Count total rules (including sub-rules)
    pub fn count_rules(rules: &[ServerRule]) -> usize {
        rules.iter().map(|r| 1 + Self::count_rules(&r.sub_rules)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rules() -> Vec<ServerRule> {
        vec![
            ServerRule {
                id: 1, server_id: 1, parent_id: None, display_order: 1,
                title: "禁止使用外挂或作弊软件".to_string(),
                description: "任何形式的作弊行为都将被永久封禁".to_string(),
                actions: vec![
                    ServerRuleAction { id: 1, violation_count: 1, action_type: "BAN".into(), duration_days: Some(0), message: "使用外挂".into() },
                ],
                sub_rules: vec![],
            },
            ServerRule {
                id: 2, server_id: 1, parent_id: None, display_order: 2,
                title: "禁止恶意破坏游戏体验".to_string(),
                description: "包括但不限于恶意TK、故意送人头、恶意摧毁友方建筑".to_string(),
                actions: vec![
                    ServerRuleAction { id: 2, violation_count: 1, action_type: "WARN".into(), duration_days: None, message: "请停止破坏行为".into() },
                    ServerRuleAction { id: 3, violation_count: 3, action_type: "KICK".into(), duration_days: None, message: "多次违规".into() },
                    ServerRuleAction { id: 4, violation_count: 5, action_type: "BAN".into(), duration_days: Some(7), message: "反复违规".into() },
                ],
                sub_rules: vec![
                    ServerRule {
                        id: 3, server_id: 1, parent_id: Some(2), display_order: 1,
                        title: "禁止恶意TK（击杀队友）".to_string(),
                        description: "故意击杀队友将被警告，多次违规将踢出或封禁".to_string(),
                        actions: vec![],
                        sub_rules: vec![],
                    },
                    ServerRule {
                        id: 4, server_id: 1, parent_id: Some(2), display_order: 2,
                        title: "禁止故意摧毁友方建筑".to_string(),
                        description: "故意摧毁友方FOB、弹药箱等建筑将被处罚".to_string(),
                        actions: vec![],
                        sub_rules: vec![],
                    },
                ],
            },
            ServerRule {
                id: 5, server_id: 1, parent_id: None, display_order: 3,
                title: "请遵守小队长指令".to_string(),
                description: "无特殊理由请配合小队长完成战术目标".to_string(),
                actions: vec![],
                sub_rules: vec![],
            },
        ]
    }

    #[test]
    fn test_generate_motd() {
        let config = MotdConfig {
            server_id: 1,
            prefix_text: "=== 欢迎来到 TCS 战队服务器 ===\n请遵守以下规则:".to_string(),
            suffix_text: "违规举报请加群: xxxxxx\n=== 祝您游戏愉快 ===".to_string(),
            auto_generate_from_rules: true,
            include_rule_descriptions: true,
        };
        let rules = sample_rules();
        let motd = MotdGenerator::generate(&config, &rules);
        assert!(motd.contains("欢迎来到"));
        assert!(motd.contains("禁止使用外挂"));
        assert!(motd.contains("2.1. 禁止恶意TK"));
        assert!(motd.contains("祝您游戏愉快"));
    }

    #[test]
    fn test_count_rules() {
        let rules = sample_rules();
        assert_eq!(MotdGenerator::count_rules(&rules), 5); // 3 top-level + 2 sub-rules
    }

    #[test]
    fn test_generate_empty() {
        let config = MotdConfig::default();
        let motd = MotdGenerator::generate(&config, &[]);
        assert!(motd.contains("服务器规则"));
        assert!(motd.contains("祝您游戏愉快"));
    }
}
