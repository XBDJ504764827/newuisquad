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
  | 'permissions';

export interface NavItemDef {
  id: PageId;
  label: string;
  icon: React.ReactNode;
}

export interface NavSectionDef {
  label: string;
  items: NavItemDef[];
}
