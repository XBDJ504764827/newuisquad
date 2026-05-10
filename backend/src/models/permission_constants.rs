// ════════════════════════════════════════════
//  细粒度权限常量与 Squad 权限映射
//  参考 Super-Boy 项目的 PBAC 权限系统
// ════════════════════════════════════════════

// ═══ Wildcard ═══
pub const WILDCARD: &str = "*";

// ═══ UI Permissions (管理面板前端权限) ═══
pub const UI_DASHBOARD_VIEW: &str = "ui:dashboard:view";
pub const UI_AUDIT_LOGS_VIEW: &str = "ui:audit_logs:view";
pub const UI_METRICS_VIEW: &str = "ui:metrics:view";
pub const UI_FEEDS_VIEW: &str = "ui:feeds:view";
pub const UI_CONSOLE_VIEW: &str = "ui:console:view";
pub const UI_CONSOLE_EXECUTE: &str = "ui:console:execute";
pub const UI_PLUGINS_VIEW: &str = "ui:plugins:view";
pub const UI_PLUGINS_MANAGE: &str = "ui:plugins:manage";
pub const UI_WORKFLOWS_VIEW: &str = "ui:workflows:view";
pub const UI_WORKFLOWS_MANAGE: &str = "ui:workflows:manage";
pub const UI_SETTINGS_VIEW: &str = "ui:settings:view";
pub const UI_SETTINGS_MANAGE: &str = "ui:settings:manage";
pub const UI_USERS_MANAGE: &str = "ui:users:manage";
pub const UI_ROLES_MANAGE: &str = "ui:roles:manage";
pub const UI_BANS_VIEW: &str = "ui:bans:view";
pub const UI_BANS_CREATE: &str = "ui:bans:create";
pub const UI_BANS_EDIT: &str = "ui:bans:edit";
pub const UI_BANS_DELETE: &str = "ui:bans:delete";
pub const UI_PLAYERS_VIEW: &str = "ui:players:view";
pub const UI_PLAYERS_KICK: &str = "ui:players:kick";
pub const UI_PLAYERS_WARN: &str = "ui:players:warn";
pub const UI_PLAYERS_MOVE: &str = "ui:players:move";
pub const UI_RULES_VIEW: &str = "ui:rules:view";
pub const UI_RULES_MANAGE: &str = "ui:rules:manage";
pub const UI_BAN_LISTS_VIEW: &str = "ui:ban_lists:view";
pub const UI_BAN_LISTS_MANAGE: &str = "ui:ban_lists:manage";
pub const UI_MOTD_VIEW: &str = "ui:motd:view";
pub const UI_MOTD_MANAGE: &str = "ui:motd:manage";
pub const UI_CONFIG_VIEW: &str = "ui:config:view";
pub const UI_CONFIG_MANAGE: &str = "ui:config:manage";

// ═══ RCON Permissions (映射到 Squad admin.cfg) ═══
pub const RCON_RESERVE: &str = "rcon:reserve";
pub const RCON_BALANCE: &str = "rcon:balance";
pub const RCON_CAN_SEE_ADMIN_CHAT: &str = "rcon:canseeadminchat";
pub const RCON_MANAGE_SERVER: &str = "rcon:manageserver";
pub const RCON_TEAM_CHANGE: &str = "rcon:teamchange";
pub const RCON_CHAT: &str = "rcon:chat";
pub const RCON_CAMERAMAN: &str = "rcon:cameraman";
pub const RCON_KICK: &str = "rcon:kick";
pub const RCON_BAN: &str = "rcon:ban";
pub const RCON_FORCE_TEAM_CHANGE: &str = "rcon:forceteamchange";
pub const RCON_IMMUNE: &str = "rcon:immune";
pub const RCON_CHANGE_MAP: &str = "rcon:changemap";
pub const RCON_PAUSE: &str = "rcon:pause";
pub const RCON_CHEAT: &str = "rcon:cheat";
pub const RCON_PRIVATE: &str = "rcon:private";
pub const RCON_CONFIG: &str = "rcon:config";
pub const RCON_FEATURE_TEST: &str = "rcon:featuretest";
pub const RCON_DEMOS: &str = "rcon:demos";
pub const RCON_DISBAND_SQUAD: &str = "rcon:disbandsquad";
pub const RCON_REMOVE_FROM_SQUAD: &str = "rcon:removefromsquad";
pub const RCON_DEMOTE_COMMANDER: &str = "rcon:demotecommander";
pub const RCON_DEBUG: &str = "rcon:debug";

// ═══ Category Wildcards ═══
pub const UI_WILDCARD: &str = "ui:*";
pub const RCON_WILDCARD: &str = "rcon:*";

/// All UI permissions (for frontend reference)
pub fn all_ui_permissions() -> Vec<(&'static str, &'static str)> {
    vec![
        (UI_DASHBOARD_VIEW, "查看仪表盘"),
        (UI_AUDIT_LOGS_VIEW, "查看审计日志"),
        (UI_METRICS_VIEW, "查看指标"),
        (UI_FEEDS_VIEW, "查看事件流"),
        (UI_CONSOLE_VIEW, "查看控制台"),
        (UI_CONSOLE_EXECUTE, "执行RCON命令"),
        (UI_PLUGINS_VIEW, "查看插件"),
        (UI_PLUGINS_MANAGE, "管理插件"),
        (UI_WORKFLOWS_VIEW, "查看工作流"),
        (UI_WORKFLOWS_MANAGE, "管理工作流"),
        (UI_SETTINGS_VIEW, "查看设置"),
        (UI_SETTINGS_MANAGE, "管理设置"),
        (UI_USERS_MANAGE, "管理用户"),
        (UI_ROLES_MANAGE, "管理角色"),
        (UI_BANS_VIEW, "查看封禁"),
        (UI_BANS_CREATE, "创建封禁"),
        (UI_BANS_EDIT, "编辑封禁"),
        (UI_BANS_DELETE, "删除封禁"),
        (UI_PLAYERS_VIEW, "查看玩家"),
        (UI_PLAYERS_KICK, "踢出玩家"),
        (UI_PLAYERS_WARN, "警告玩家"),
        (UI_PLAYERS_MOVE, "移动玩家"),
        (UI_RULES_VIEW, "查看规则"),
        (UI_RULES_MANAGE, "管理规则"),
        (UI_BAN_LISTS_VIEW, "查看封禁列表"),
        (UI_BAN_LISTS_MANAGE, "管理封禁列表"),
        (UI_MOTD_VIEW, "查看MOTD"),
        (UI_MOTD_MANAGE, "管理MOTD"),
        (UI_CONFIG_VIEW, "查看配置"),
        (UI_CONFIG_MANAGE, "管理配置"),
    ]
}

/// All RCON permissions (for frontend reference)
pub fn all_rcon_permissions() -> Vec<(&'static str, &'static str)> {
    vec![
        (RCON_RESERVE, "预留位"),
        (RCON_BALANCE, "平衡"),
        (RCON_CAN_SEE_ADMIN_CHAT, "查看管理聊天"),
        (RCON_MANAGE_SERVER, "管理服务器"),
        (RCON_TEAM_CHANGE, "换队"),
        (RCON_CHAT, "聊天"),
        (RCON_CAMERAMAN, "观察者"),
        (RCON_KICK, "踢出"),
        (RCON_BAN, "封禁"),
        (RCON_FORCE_TEAM_CHANGE, "强制换队"),
        (RCON_IMMUNE, "免疫"),
        (RCON_CHANGE_MAP, "换图"),
        (RCON_PAUSE, "暂停"),
        (RCON_CHEAT, "作弊"),
        (RCON_PRIVATE, "私密"),
        (RCON_CONFIG, "配置"),
        (RCON_FEATURE_TEST, "功能测试"),
        (RCON_DEMOS, "录像"),
        (RCON_DISBAND_SQUAD, "解散小队"),
        (RCON_REMOVE_FROM_SQUAD, "移除小队成员"),
        (RCON_DEMOTE_COMMANDER, "降职指挥官"),
        (RCON_DEBUG, "调试"),
    ]
}

/// Map granular rcon:* permissions to Squad admin.cfg format
pub fn rcon_to_squad_permission(granular: &str) -> Option<&'static str> {
    match granular {
        RCON_RESERVE => Some("reserve"),
        RCON_BALANCE => Some("balance"),
        RCON_CAN_SEE_ADMIN_CHAT => Some("canseeadminchat"),
        RCON_MANAGE_SERVER => Some("manageserver"),
        RCON_TEAM_CHANGE => Some("teamchange"),
        RCON_CHAT => Some("chat"),
        RCON_CAMERAMAN => Some("cameraman"),
        RCON_KICK => Some("kick"),
        RCON_BAN => Some("ban"),
        RCON_FORCE_TEAM_CHANGE => Some("forceteamchange"),
        RCON_IMMUNE => Some("immune"),
        RCON_CHANGE_MAP => Some("changemap"),
        RCON_PAUSE => Some("pause"),
        RCON_CHEAT => Some("cheat"),
        RCON_PRIVATE => Some("private"),
        RCON_CONFIG => Some("config"),
        RCON_FEATURE_TEST => Some("featuretest"),
        RCON_DEMOS => Some("demos"),
        RCON_DISBAND_SQUAD => Some("disbandSquad"),
        RCON_REMOVE_FROM_SQUAD => Some("removeFromSquad"),
        RCON_DEMOTE_COMMANDER => Some("demoteCommander"),
        RCON_DEBUG => Some("debug"),
        _ => None,
    }
}

/// Normalize a legacy flat permission to granular format
/// e.g., "kick" → "rcon:kick", "ban" → "rcon:ban"
pub fn normalize_legacy_permission(perm: &str) -> String {
    let perm = perm.trim().to_lowercase();
    if perm.contains(':') {
        return perm;
    }
    // Known Squad permissions
    let squad_perms = [
        "reserve", "balance", "canseeadminchat", "manageserver", "teamchange",
        "chat", "cameraman", "kick", "ban", "forceteamchange", "immune",
        "changemap", "pause", "cheat", "private", "config", "featuretest",
        "demos", "disbandsquad", "removefromsquad", "demotecommander", "debug",
    ];
    if squad_perms.contains(&perm.as_str()) {
        format!("rcon:{}", perm)
    } else {
        perm
    }
}

/// Check if a user's permissions grant access to a required permission.
/// Supports:
///   - `*` wildcard (super admin)
///   - Category wildcards like `ui:*`, `rcon:*`
///   - Exact match
///   - Parent→child permission matching (e.g., "ui:bans" matches "ui:bans:create")
#[allow(dead_code)]
pub fn evaluate_permission(user_permissions: &[String], required: &str) -> bool {
    if user_permissions.contains(&WILDCARD.to_string()) {
        return true;
    }

    let required_lower = required.to_lowercase();

    for perm in user_permissions {
        let perm = perm.trim().to_lowercase();

        // Exact match
        if perm == required_lower {
            return true;
        }

        // Category wildcard: ui:* or rcon:*
        if perm.ends_with(":*") {
            let category = perm.trim_end_matches(":*");
            if required_lower.starts_with(category) {
                return true;
            }
        }

        // Parent permission matches child: ui:bans → ui:bans:create
        if required_lower.starts_with(&perm) && required_lower.len() > perm.len()
            && required_lower.as_bytes()[perm.len()] == b':'
        {
            return true;
        }
    }

    false
}

/// Resolve effective permissions for a role, including parent inheritance.
/// `all_groups`: all permission_groups for the server
/// `group_name`: the role to resolve
pub fn resolve_effective_permissions(
    all_groups: &[crate::models::permission::PermissionGroupRow],
    group_name: &str,
    visited: &mut std::collections::HashSet<String>,
) -> Vec<String> {
    if !visited.insert(group_name.to_string()) {
        return vec![]; // circular inheritance, stop
    }

    let mut perms = Vec::new();

    if let Some(group) = all_groups.iter().find(|g| g.group_name == group_name) {
        // Add own permissions (normalize legacy format)
        for p in group.permissions.split(',') {
            let p = p.trim();
            if !p.is_empty() {
                perms.push(normalize_legacy_permission(p));
            }
        }

        // Inherit parent permissions
        if let Some(parent_id) = group.parent_group_id {
            if let Some(parent) = all_groups.iter().find(|g| g.id == parent_id) {
                let inherited = resolve_effective_permissions(
                    all_groups,
                    &parent.group_name,
                    visited,
                );
                perms.extend(inherited);
            }
        }
    }

    // Deduplicate
    perms.sort();
    perms.dedup();
    perms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::permission::PermissionGroupRow;

    fn make_group(id: i32, name: &str, perms: &str, parent: Option<i32>, is_admin: bool) -> PermissionGroupRow {
        PermissionGroupRow {
            id, server_id: 1, group_name: name.to_string(),
            permissions: perms.to_string(), parent_group_id: parent,
            is_admin, is_template: false,
        }
    }

    #[test]
    fn test_evaluate_wildcard() {
        assert!(evaluate_permission(&["*".to_string()], "ui:dashboard:view"));
        assert!(evaluate_permission(&["*".to_string()], "rcon:kick"));
    }

    #[test]
    fn test_evaluate_exact_match() {
        let perms = vec!["ui:dashboard:view".to_string(), "rcon:kick".to_string()];
        assert!(evaluate_permission(&perms, "ui:dashboard:view"));
        assert!(evaluate_permission(&perms, "rcon:kick"));
        assert!(!evaluate_permission(&perms, "ui:bans:create"));
    }

    #[test]
    fn test_evaluate_category_wildcard() {
        let perms = vec!["ui:*".to_string()];
        assert!(evaluate_permission(&perms, "ui:dashboard:view"));
        assert!(evaluate_permission(&perms, "ui:bans:create"));
        assert!(!evaluate_permission(&perms, "rcon:kick"));
    }

    #[test]
    fn test_evaluate_parent_permission() {
        let perms = vec!["ui:bans".to_string()];
        assert!(evaluate_permission(&perms, "ui:bans:create"));
        assert!(evaluate_permission(&perms, "ui:bans:edit"));
        assert!(!evaluate_permission(&perms, "ui:players:view"));
    }

    #[test]
    fn test_normalize_legacy() {
        assert_eq!(normalize_legacy_permission("kick"), "rcon:kick");
        assert_eq!(normalize_legacy_permission("ban"), "rcon:ban");
        assert_eq!(normalize_legacy_permission("chat"), "rcon:chat");
        assert_eq!(normalize_legacy_permission("rcon:kick"), "rcon:kick");
        assert_eq!(normalize_legacy_permission("ui:dashboard:view"), "ui:dashboard:view");
        assert_eq!(normalize_legacy_permission("unknown"), "unknown");
    }

    #[test]
    fn test_rcon_to_squad_mapping() {
        assert_eq!(rcon_to_squad_permission("rcon:kick"), Some("kick"));
        assert_eq!(rcon_to_squad_permission("rcon:disbandsquad"), Some("disbandSquad"));
        assert_eq!(rcon_to_squad_permission("ui:dashboard:view"), None);
    }

    #[test]
    fn test_resolve_effective_permissions_no_inheritance() {
        let groups = vec![
            make_group(1, "admin", "rcon:kick,rcon:ban,ui:dashboard:view", None, true),
        ];
        let mut visited = std::collections::HashSet::new();
        let perms = resolve_effective_permissions(&groups, "admin", &mut visited);
        assert!(perms.contains(&"rcon:kick".to_string()));
        assert!(perms.contains(&"rcon:ban".to_string()));
        assert_eq!(perms.len(), 3);
    }

    #[test]
    fn test_resolve_effective_permissions_with_inheritance() {
        let groups = vec![
            make_group(1, "base", "ui:dashboard:view,rcon:chat", None, false),
            make_group(2, "admin", "rcon:kick,rcon:ban", Some(1), true),
        ];
        let mut visited = std::collections::HashSet::new();
        let perms = resolve_effective_permissions(&groups, "admin", &mut visited);
        assert!(perms.contains(&"rcon:kick".to_string()));
        assert!(perms.contains(&"rcon:ban".to_string()));
        assert!(perms.contains(&"ui:dashboard:view".to_string()));
        assert!(perms.contains(&"rcon:chat".to_string()));
        assert_eq!(perms.len(), 4);
    }

    #[test]
    fn test_resolve_effective_permissions_legacy_normalization() {
        let groups = vec![
            make_group(1, "old_admin", "kick,ban,chat,cameraman", None, true),
        ];
        let mut visited = std::collections::HashSet::new();
        let perms = resolve_effective_permissions(&groups, "old_admin", &mut visited);
        assert!(perms.contains(&"rcon:kick".to_string()));
        assert!(perms.contains(&"rcon:ban".to_string()));
        assert!(perms.contains(&"rcon:chat".to_string()));
        assert!(perms.contains(&"rcon:cameraman".to_string()));
    }

    #[test]
    fn test_resolve_effective_permissions_circular_stops() {
        let groups = vec![
            make_group(1, "a", "rcon:kick", Some(2), true),
            make_group(2, "b", "rcon:ban", Some(1), true),
        ];
        let mut visited = std::collections::HashSet::new();
        let perms = resolve_effective_permissions(&groups, "a", &mut visited);
        // Should include a's own + b's, but not recurse infinitely
        assert!(perms.contains(&"rcon:kick".to_string()));
        assert!(perms.contains(&"rcon:ban".to_string()));
    }
}
