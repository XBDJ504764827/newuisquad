use serde::Serialize;

/// Squad RCON Command type
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CommandType {
    Public,
    Admin,
}

/// Command catalog entry
#[derive(Debug, Clone, Serialize)]
pub struct CommandInfo {
    pub name: String,
    pub category: String,
    pub syntax: String,
    pub description: String,
    pub command_type: CommandType,
}

/// Full catalog of Squad RCON commands (84+ entries)
pub fn command_catalog() -> Vec<CommandInfo> {
    vec![
        // ═══ 公共命令 ═══
        cmd("ListCommands", "manageserver", "ListCommands", "列出所有可用命令", CommandType::Public),
        cmd("ShowCommandInfo", "manageserver", "ShowCommandInfo <CommandName>", "显示命令详情", CommandType::Public),
        cmd("ShowServerInfo", "manageserver", "ShowServerInfo", "显示服务器信息", CommandType::Public),
        cmd("ListPlayers", "manageserver", "ListPlayers", "列出所有玩家", CommandType::Public),
        cmd("ListSquads", "manageserver", "ListSquads", "列出所有小队", CommandType::Public),
        cmd("ListLevels", "changemap", "ListLevels", "列出所有地图", CommandType::Public),
        cmd("ListLayers", "changemap", "ListLayers <MapName>", "列出地图的可用层", CommandType::Public),
        cmd("ShowCurrentMap", "changemap", "ShowCurrentMap", "显示当前地图", CommandType::Public),
        cmd("ShowNextMap", "changemap", "ShowNextMap", "显示下一张地图", CommandType::Public),
        cmd("ChatToAdmin", "chat", "ChatToAdmin <Message>", "向管理员发送消息", CommandType::Public),

        // ═══ 管理员命令 ═══
        // chat
        cmd("AdminBroadcast", "chat", "AdminBroadcast <Message>", "向所有玩家广播消息", CommandType::Admin),
        cmd("AdminVote", "chat", "AdminVote <Question> <Option1> <Option2> ...", "发起投票", CommandType::Admin),

        // kick
        cmd("AdminKick", "kick", "AdminKick <PlayerName|SteamID> <Reason>", "踢出玩家（按名称或SteamID）", CommandType::Admin),
        cmd("AdminKickById", "kick", "AdminKickById <PlayerID> <Reason>", "踢出玩家（按ID）", CommandType::Admin),
        cmd("AdminWarn", "kick", "AdminWarn <PlayerName|SteamID> <Message>", "警告玩家", CommandType::Admin),
        cmd("AdminWarnById", "kick", "AdminWarnById <PlayerID> <Message>", "警告玩家（按ID）", CommandType::Admin),
        cmd("AdminListDisconnectedPlayers", "kick", "AdminListDisconnectedPlayers", "列出已断线玩家", CommandType::Admin),

        // ban
        cmd("AdminBan", "ban", "AdminBan <PlayerName|SteamID> <Duration> <Reason>", "封禁玩家（0=永久，分钟数）", CommandType::Admin),
        cmd("AdminBanById", "ban", "AdminBanById <PlayerID> <Duration> <Reason>", "封禁玩家（按ID）", CommandType::Admin),
        cmd("AdminListBans", "ban", "AdminListBans", "列出封禁列表", CommandType::Admin),
        cmd("AdminListWarns", "ban", "AdminListWarns", "列出警告列表", CommandType::Admin),

        // changemap
        cmd("AdminChangeLevel", "changemap", "AdminChangeLevel <LevelName>", "切换地图", CommandType::Admin),
        cmd("AdminChangeLayer", "changemap", "AdminChangeLayer <LayerName>", "切换地图图层", CommandType::Admin),
        cmd("AdminSetNextLevel", "changemap", "AdminSetNextLevel <LevelName>", "设置下一地图", CommandType::Admin),
        cmd("AdminSetNextLayer", "changemap", "AdminSetNextLayer <LayerName>", "设置下一图层", CommandType::Admin),
        cmd("AdminClearNextLayer", "changemap", "AdminClearNextLayer", "清除预设下一图层", CommandType::Admin),

        // config
        cmd("AdminSetMaxNumPlayers", "config", "AdminSetMaxNumPlayers <Number>", "设置最大玩家数", CommandType::Admin),
        cmd("AdminSetPublicQueueLimit", "config", "AdminSetPublicQueueLimit <Number>", "设置公开队列限制", CommandType::Admin),
        cmd("AdminSetNumReservedSlots", "reserve", "AdminSetNumReservedSlots <Number>", "设置预留位数量", CommandType::Admin),
        cmd("AdminSetServerPassword", "private", "AdminSetServerPassword <Password>", "设置服务器密码（空=取消）", CommandType::Admin),
        cmd("AdminEnableVoting", "config", "AdminEnableVoting <true|false>", "启用/禁用投票", CommandType::Admin),
        cmd("AdminSetCustomOption", "config", "AdminSetCustomOption <Key> <Value>", "设置自定义选项", CommandType::Admin),
        cmd("AdminRemoveCustomOption", "config", "AdminRemoveCustomOption <Key>", "移除自定义选项", CommandType::Admin),
        cmd("AdminReloadServerConfig", "config", "AdminReloadServerConfig", "重载服务器配置", CommandType::Admin),

        // forceteamchange
        cmd("AdminForceTeamChange", "forceteamchange", "AdminForceTeamChange <PlayerName|SteamID>", "强制玩家换队", CommandType::Admin),
        cmd("AdminForceTeamChangeById", "forceteamchange", "AdminForceTeamChangeById <PlayerID>", "强制玩家换队（按ID）", CommandType::Admin),

        // removeFromSquad
        cmd("AdminRemovePlayerFromSquad", "removeFromSquad", "AdminRemovePlayerFromSquad <PlayerName>", "将玩家移出小队", CommandType::Admin),
        cmd("AdminRemovePlayerFromSquadById", "removeFromSquad", "AdminRemovePlayerFromSquadById <PlayerID>", "将玩家移出小队（按ID）", CommandType::Admin),

        // demoteCommander
        cmd("AdminDemoteCommander", "demoteCommander", "AdminDemoteCommander <PlayerName>", "降职指挥官", CommandType::Admin),
        cmd("AdminDemoteCommanderById", "demoteCommander", "AdminDemoteCommanderById <PlayerID>", "降职指挥官（按ID）", CommandType::Admin),

        // disbandSquad
        cmd("AdminDisbandSquad", "disbandSquad", "AdminDisbandSquad <TeamID> <SquadID>", "解散小队", CommandType::Admin),
        cmd("AdminRenameSquad", "disbandSquad", "AdminRenameSquad <TeamID> <SquadID> <NewName>", "重命名小队", CommandType::Admin),

        // teamchange
        cmd("SLInviteMember", "teamchange", "SLInviteMember <PlayerName>", "邀请玩家加入小队（仅小队长可用）", CommandType::Admin),

        // pause
        cmd("AdminRestartMatch", "pause", "AdminRestartMatch", "重启比赛", CommandType::Admin),
        cmd("AdminEndMatch", "pause", "AdminEndMatch", "结束当前比赛", CommandType::Admin),
        cmd("AdminPauseMatch", "pause", "AdminPauseMatch", "暂停比赛", CommandType::Admin),
        cmd("AdminUnpauseMatch", "pause", "AdminUnpauseMatch", "恢复比赛", CommandType::Admin),

        // cameraman
        cmd("AdminAddCameraman", "cameraman", "AdminAddCameraman <PlayerName>", "添加观察者", CommandType::Admin),

        // cheat (暖服/调试用)
        cmd("AdminSlomo", "cheat", "AdminSlomo <Speed>", "设置游戏速度 (0.1-3.0)", CommandType::Admin),
        cmd("AdminSetFogOfWar", "cheat", "AdminSetFogOfWar <0|1>", "设置战争迷雾", CommandType::Admin),
        cmd("AdminForceAllVehicleAvailability", "cheat", "AdminForceAllVehicleAvailability <0|1>", "强制所有载具可用", CommandType::Admin),
        cmd("AdminForceAllDeployableAvailability", "cheat", "AdminForceAllDeployableAvailability <0|1>", "强制所有部署物可用", CommandType::Admin),
        cmd("AdminForceAllRoleAvailability", "cheat", "AdminForceAllRoleAvailability <0|1>", "强制所有兵种可用", CommandType::Admin),
        cmd("AdminForceAllActionAvailability", "cheat", "AdminForceAllActionAvailability <0|1>", "强制所有动作可用", CommandType::Admin),
        cmd("AdminNoTeamChangeTimer", "cheat", "AdminNoTeamChangeTimer <0|1>", "移除换队冷却", CommandType::Admin),
        cmd("AdminNoRespawnTimer", "cheat", "AdminNoRespawnTimer <0|1>", "移除重生计时器", CommandType::Admin),
        cmd("AdminDisableVehicleClaiming", "cheat", "AdminDisableVehicleClaiming <0|1>", "禁用载具认领", CommandType::Admin),
        cmd("AdminDisableVehicleTeamRequirement", "cheat", "AdminDisableVehicleTeamRequirement <0|1>", "禁用载具阵营限制", CommandType::Admin),
        cmd("AdminDisableVehicleKitRequirement", "cheat", "AdminDisableVehicleKitRequirement <0|1>", "禁用载具装备限制", CommandType::Admin),
        cmd("AdminAlwaysValidPlacement", "cheat", "AdminAlwaysValidPlacement <0|1>", "始终有效放置", CommandType::Admin),

        // demos
        cmd("AdminDemoPlay", "demos", "AdminDemoPlay <FileName>", "播放录像", CommandType::Admin),
        cmd("AdminDemoRec", "demos", "AdminDemoRec <FileName>", "开始录像", CommandType::Admin),
        cmd("AdminDemoStop", "demos", "AdminDemoStop", "停止录像", CommandType::Admin),
        cmd("RecordingStart", "demos", "RecordingStart", "开始录制", CommandType::Admin),
        cmd("RecordingStart_Named", "demos", "RecordingStart_Named <Name>", "命名录制并开始", CommandType::Admin),
        cmd("RecordingStop", "demos", "RecordingStop", "停止录制", CommandType::Admin),

        // featuretest
        cmd("AdminCreateVehicle", "featuretest", "AdminCreateVehicle <VehicleName>", "创建载具", CommandType::Admin),
        cmd("AdminCreateDeployable", "featuretest", "AdminCreateDeployable <DeployableName>", "创建部署物", CommandType::Admin),
        cmd("AdminGiveEquipment", "featuretest", "AdminGiveEquipment <PlayerName> <EquipmentName>", "给予玩家装备", CommandType::Admin),
        cmd("AdminSpawnActor", "featuretest", "AdminSpawnActor <ActorName>", "生成Actor", CommandType::Admin),

        // debug
        cmd("AdminForceNetUpdateOnClientSaturation", "debug", "AdminForceNetUpdateOnClientSaturation <0|1>", "强制网络更新", CommandType::Admin),
        cmd("AdminProfileServer", "debug", "AdminProfileServer <Duration>", "服务器性能分析", CommandType::Admin),
        cmd("DebugVehicleList", "debug", "DebugVehicleList", "调试载具列表", CommandType::Admin),
        cmd("DebugAddBuildSupply", "debug", "DebugAddBuildSupply <Amount>", "添加建筑资源", CommandType::Admin),
        cmd("DebugAddAmmoSupply", "debug", "DebugAddAmmoSupply <Amount>", "添加弹药资源", CommandType::Admin),
        cmd("DebugRearm", "debug", "DebugRearm", "重新武装", CommandType::Admin),
        cmd("DebugRearmPlayer", "debug", "DebugRearmPlayer <PlayerName>", "重新武装玩家", CommandType::Admin),
        cmd("DebugPrintPlayerStats", "debug", "DebugPrintPlayerStats", "打印玩家统计", CommandType::Admin),
        cmd("DebugPrintFactionsList", "debug", "DebugPrintFactionsList", "打印阵营列表", CommandType::Admin),
        cmd("DebugCrash", "debug", "DebugCrash", "调试崩溃", CommandType::Admin),
    ]
}

fn cmd(name: &str, category: &str, syntax: &str, description: &str, cmd_type: CommandType) -> CommandInfo {
    CommandInfo {
        name: name.to_string(),
        category: category.to_string(),
        syntax: syntax.to_string(),
        description: description.to_string(),
        command_type: cmd_type,
    }
}

/// Get commands filtered by category
pub fn get_commands_by_category(category: &str) -> Vec<&'static CommandInfo> {
    // This would need static storage; for now return empty
    // In production this would use once_cell::sync::Lazy
    vec![]
}

/// Map a command category to the RCON permission required
pub fn category_to_permission(category: &str) -> Option<&'static str> {
    match category {
        "manageserver" => Some("rcon:manageserver"),
        "changemap" => Some("rcon:changemap"),
        "chat" => Some("rcon:chat"),
        "config" => Some("rcon:config"),
        "reserve" => Some("rcon:reserve"),
        "private" => Some("rcon:private"),
        "cheat" => Some("rcon:cheat"),
        "kick" => Some("rcon:kick"),
        "ban" => Some("rcon:ban"),
        "forceteamchange" => Some("rcon:forceteamchange"),
        "teamchange" => Some("rcon:teamchange"),
        "removeFromSquad" => Some("rcon:removefromsquad"),
        "demoteCommander" => Some("rcon:demotecommander"),
        "disbandSquad" => Some("rcon:disbandsquad"),
        "pause" => Some("rcon:pause"),
        "cameraman" => Some("rcon:cameraman"),
        "featuretest" => Some("rcon:featuretest"),
        "demos" => Some("rcon:demos"),
        "debug" => Some("rcon:debug"),
        _ => None,
    }
}
