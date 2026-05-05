// 服务器
export interface ServerInfo {
  id: number; server_id: string; name: string; ip: string;
  rcon_port: number; rcon_password?: string; created_at: string; token?: string;
}

// 服务器实时状态
export interface PlayerState {
  name: string; steam_id: string; team_id: number;
  squad_id: string | null; role: string;
  kills: number; deaths: number; score: number; ping: number;
  is_admin: boolean; is_leader: boolean; player_id: number;
}

export interface SquadState {
  name: string; creator: string; team_id: number; squad_id: string;
  leader_name?: string; leader_steam_id?: string;
}

export interface TeamState {
  team_id: number; faction: string;
}

export interface ServerState {
  players: PlayerState[];
  squads: SquadState[];
  teams: TeamState[];
  admin_steam_ids: string[];
  map_name: string;
  game_mode: string;
  server_name: string;
  player_count: number;
  max_players: number;
  next_map: string;
}

// 日志/聊天
export interface ServerInfoDisplay {
  server_name: string; player_count: number; max_players: number;
  map_name: string; game_mode: string; next_map: string; next_layer: string;
}

export type PageId =
  | 'summary'
  | 'control-panel'
  | 'chat-logs'
  | 'fly-logs'
  | 'kill-logs'
  | 'match-logs'
  | 'config-file'
  | 'config-panel'
  | 'action-logs'
  | 'player-info'
  | 'admin-users'
  | 'permission-settings';

export interface NavItemDef {
  id: PageId;
  label: string;
  icon: React.ReactNode;
}

export interface NavSectionDef {
  label: string;
  items: NavItemDef[];
}
