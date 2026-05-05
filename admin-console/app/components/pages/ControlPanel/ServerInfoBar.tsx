'use client';

interface ServerInfoBarProps {
  servers: any[];
  selectedServer: any;
  autoRefresh: boolean;
  serverInfo: {
    server_name: string;
    player_count: number;
    max_players: number;
    map_name: string;
    game_mode: string;
    next_map: string;
  } | null;
  serverState: any;
  onSelectServer: (s: any) => void;
  onAddServer: () => void;
  onToggleAutoRefresh: () => void;
  onManualRefresh: () => void;
}

export function factionFlag(f: string) {
  if (/pla|people.*liberation/i.test(f)) return '🇨🇳';
  if (/us\s*army|united\s*states/i.test(f)) return '🇺🇸';
  if (/british|baf/i.test(f)) return '🇬🇧';
  if (/canadian/i.test(f)) return '🇨🇦';
  if (/australian/i.test(f)) return '🇦🇺';
  if (/russian|rgf|vdv/i.test(f)) return '🇷🇺';
  if (/insurgent|irregular/i.test(f)) return '🏴';
  if (/turkish/i.test(f)) return '🇹🇷';
  if (/middle\s*eastern|mea/i.test(f)) return '🇸🇦';
  if (/marine/i.test(f)) return '🌎';
  return '🎖️';
}

export function ServerInfoBar({ servers, selectedServer, autoRefresh, serverInfo, serverState, onSelectServer, onAddServer, onToggleAutoRefresh, onManualRefresh }: ServerInfoBarProps) {
  const playerPct = serverInfo ? Math.round((serverInfo.player_count / Math.max(1, serverInfo.max_players)) * 100) : 0;

  return (
    <>
      {/* 服务器选择标签 */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, flexWrap: 'wrap' }}>
        <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
          {servers.map((s: any) => (
            <button
              key={s.id}
              onClick={() => onSelectServer(s)}
              style={{
                padding: '6px 14px', borderRadius: 6, border: 'none', cursor: 'pointer', fontSize: 12, fontWeight: 500,
                background: selectedServer?.id === s.id ? 'var(--text)' : 'var(--bg3)',
                color: selectedServer?.id === s.id ? 'var(--bg)' : 'var(--text2)',
                transition: 'all .15s',
              }}
            >{s.name}</button>
          ))}
          <button
            onClick={onAddServer}
            style={{ width: 28, height: 28, borderRadius: 6, border: '1px dashed var(--border2)', background: 'transparent', color: 'var(--text3)', cursor: 'pointer', fontSize: 16, display: 'flex', alignItems: 'center', justifyContent: 'center', transition: 'all .15s' }}
            title="添加服务器"
          >+</button>
        </div>
        <div style={{ display: 'flex', gap: 10, alignItems: 'center' }}>
          <label style={{ fontSize: 11, color: 'var(--text3)', display: 'flex', gap: 5, alignItems: 'center', cursor: 'pointer', userSelect: 'none' }}>
            <div style={{ width: 32, height: 18, borderRadius: 9, background: autoRefresh ? '#22c55e' : 'var(--border2)', position: 'relative', transition: 'background .2s' }}>
              <div style={{ position: 'absolute', top: 2, left: autoRefresh ? 16 : 2, width: 14, height: 14, borderRadius: '50%', background: '#fff', transition: 'left .2s', boxShadow: '0 1px 3px rgba(0,0,0,.3)' }} />
            </div>
            自动刷新
          </label>
          <button
            onClick={onManualRefresh}
            style={{ width: 28, height: 28, borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg3)', color: 'var(--text2)', cursor: 'pointer', fontSize: 13, display: 'flex', alignItems: 'center', justifyContent: 'center', transition: 'all .15s' }}
            title="手动刷新"
          >🔄</button>
        </div>
      </div>

      {/* 服务器状态卡片 */}
      {serverInfo && (
        <div style={{ background: 'var(--bg2)', border: '1px solid var(--border)', borderRadius: 10, overflow: 'hidden' }}>
          <div style={{ padding: '14px 20px', display: 'flex', gap: 20, flexWrap: 'wrap', alignItems: 'center' }}>
            <div style={{ minWidth: 0, flex: '0 0 auto' }}>
              <div style={{ fontSize: 13, fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: 260 }}>
                🖥️ {serverInfo.server_name || selectedServer?.name}
              </div>
            </div>
            <div style={{ width: 1, height: 24, background: 'var(--border)', flexShrink: 0 }} />
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexShrink: 0 }}>
              <span style={{ fontSize: 13, fontWeight: 600, whiteSpace: 'nowrap' }}>
                👥 {serverInfo.player_count}<span style={{ color: 'var(--text3)', fontWeight: 400 }}>/{serverInfo.max_players}</span>
              </span>
              <div style={{ width: 80, height: 6, borderRadius: 3, background: 'var(--bg3)', overflow: 'hidden' }}>
                <div style={{ height: '100%', borderRadius: 3, background: playerPct > 90 ? '#ef4444' : playerPct > 70 ? '#f59e0b' : '#22c55e', width: `${Math.min(100, playerPct)}%`, transition: 'width .5s ease' }} />
              </div>
            </div>
            <div style={{ width: 1, height: 24, background: 'var(--border)', flexShrink: 0 }} />
            <div style={{ display: 'flex', gap: 16, flexWrap: 'wrap', alignItems: 'center', flex: 1, minWidth: 0 }}>
              <span style={{ fontSize: 13, whiteSpace: 'nowrap' }}>
                🗺️ <strong>{serverInfo.map_name}</strong>
                <span style={{ color: 'var(--text3)', marginLeft: 6 }}>({serverInfo.game_mode})</span>
              </span>
              {serverInfo.next_map && (
                <span style={{ fontSize: 12, color: 'var(--text2)', whiteSpace: 'nowrap' }}>
                  → <span style={{ color: 'var(--text3)' }}>下一张:</span> {serverInfo.next_map}
                </span>
              )}
            </div>
            {serverState?.teams && serverState.teams.length >= 2 && (
              <>
                <div style={{ width: 1, height: 24, background: 'var(--border)', flexShrink: 0 }} />
                <div style={{ display: 'flex', gap: 12, alignItems: 'center', flexShrink: 0, fontSize: 12 }}>
                  <span>{factionFlag(serverState.teams[0]?.faction || '')} {serverState.teams[0]?.faction || '—'}</span>
                  <span style={{ color: 'var(--text3)', fontWeight: 700 }}>VS</span>
                  <span>{factionFlag(serverState.teams[1]?.faction || '')} {serverState.teams[1]?.faction || '—'}</span>
                </div>
              </>
            )}
          </div>
        </div>
      )}
    </>
  );
}
