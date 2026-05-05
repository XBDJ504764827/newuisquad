'use client';

import { InfoRow } from './InfoRow';

const QUICK_COMMANDS = [
  { label: '列出玩家', cmd: 'ListPlayers', icon: '👥' },
  { label: '列出小队', cmd: 'ListSquads', icon: '🛡️' },
  { label: '下张地图', cmd: 'ShowNextMap', icon: '🗺️' },
  { label: '服务器信息', cmd: 'ShowServerInfo', icon: '📊' },
  { label: '结束对局', cmd: 'AdminEndMatch', icon: '🏁' },
  { label: '换图确认', cmd: 'AdminSlomo 1', icon: '⏱️' },
];

const WARMUP_ITEMS = [
  { key: 'novehicleclaim', label: '取消载具认领权限', cmd: 'AdminDisableVehicleClaiming' },
  { key: 'forcevehicle', label: '始终填满所有载具刷新位置', cmd: 'AdminForceAllVehicleAvailability' },
  { key: 'forcedeploy', label: '取消部署要求限制', cmd: 'AdminForceAllDeployableAvailability' },
  { key: 'forcerole', label: '取消装具人数限制', cmd: 'AdminForceAllRoleAvailability' },
  { key: 'noenemylimit', label: '可以使用敌方载具', cmd: 'AdminDisableVehicleTeamRequirement' },
  { key: 'nokitreq', label: '取消坦克飞机载具要求', cmd: 'AdminDisableVehicleKitRequirement' },
  { key: 'norespawn', label: '取消复活时间', cmd: 'AdminNoRespawnTimer' },
];

interface LeftSidePanelProps {
  selectedServer: any;
  rconCommand: string;
  rconResult: string;
  broadcastMsg: string;
  warmupToggles: Record<string, boolean | null>;
  onRconCommandChange: (v: string) => void;
  onSendRcon: (cmd?: string) => void;
  onDeleteServer: (s: any) => void;
  onBroadcastMsgChange: (v: string) => void;
  onSendBroadcast: () => void;
  onWarmupToggle: (key: string, value: boolean) => void;
}

export function LeftSidePanel({
  selectedServer, rconCommand, rconResult, broadcastMsg, warmupToggles,
  onRconCommandChange, onSendRcon, onDeleteServer,
  onBroadcastMsgChange, onSendBroadcast, onWarmupToggle,
}: LeftSidePanelProps) {
  if (!selectedServer) return null;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      {/* 连接信息 */}
      <div className="card">
        <div className="card-header" style={{ padding: '10px 14px' }}>
          <div className="card-title" style={{ fontSize: 13 }}>📡 连接信息</div>
        </div>
        <div className="card-body" style={{ padding: '12px 14px', display: 'flex', flexDirection: 'column', gap: 8, fontSize: 12 }}>
          <InfoRow label="服务器 ID" value={String(selectedServer.server_id)} />
          <InfoRow label="地址" value={`${selectedServer.ip}:${selectedServer.rcon_port}`} />
          <button
            onClick={() => onDeleteServer(selectedServer)}
            style={{
              width: '100%', marginTop: 4, padding: '7px 0', background: 'transparent',
              border: '1px solid rgba(239,68,68,0.3)', borderRadius: 6,
              color: 'var(--red)', cursor: 'pointer', fontSize: 11, fontWeight: 500,
              transition: 'all .15s',
            }}
            onMouseEnter={e => { e.currentTarget.style.background = 'rgba(239,68,68,0.08)'; }}
            onMouseLeave={e => { e.currentTarget.style.background = 'transparent'; }}
          >删除服务器</button>
        </div>
      </div>

      {/* RCON 命令 */}
      <div className="card">
        <div className="card-header" style={{ padding: '10px 14px' }}>
          <div className="card-title" style={{ fontSize: 13 }}>⌨️ RCON 命令</div>
        </div>
        <div className="card-body" style={{ padding: '10px 14px', display: 'flex', flexDirection: 'column', gap: 8 }}>
          <div style={{ display: 'flex', gap: 6 }}>
            <input type="text" className="rcon-input" placeholder="输入 RCON 指令..."
              value={rconCommand} onChange={e => onRconCommandChange(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && onSendRcon()}
              style={{ flex: 1, fontSize: 12, padding: '8px 10px' }} />
            <button className="rcon-btn" onClick={() => onSendRcon()}
              style={{ width: 'auto', padding: '8px 14px', fontSize: 12 }}>发送</button>
          </div>
          {rconResult && (
            <div className="terminal" style={{ maxHeight: 140, overflowY: 'auto', fontSize: 11, padding: 10, whiteSpace: 'pre-wrap', wordBreak: 'break-all', borderRadius: 6 }}>
              {rconResult}
            </div>
          )}
        </div>
      </div>

      {/* 快捷命令 */}
      <div className="card">
        <div className="card-header" style={{ padding: '10px 14px' }}>
          <div className="card-title" style={{ fontSize: 13 }}>⚡ 快捷命令</div>
        </div>
        <div className="card-body" style={{ padding: '8px 14px', display: 'flex', flexDirection: 'column', gap: 4 }}>
          {QUICK_COMMANDS.map(qc => (
            <button key={qc.cmd} onClick={() => onSendRcon(qc.cmd)}
              style={{
                width: '100%', padding: '7px 12px', border: '1px solid var(--border)',
                borderRadius: 6, background: 'var(--bg3)', color: 'var(--text2)',
                cursor: 'pointer', fontSize: 12, textAlign: 'left',
                transition: 'all .12s', display: 'flex', gap: 8, alignItems: 'center',
              }}
              onMouseEnter={e => { e.currentTarget.style.background = 'var(--bg4)'; e.currentTarget.style.color = 'var(--text)'; }}
              onMouseLeave={e => { e.currentTarget.style.background = 'var(--bg3)'; e.currentTarget.style.color = 'var(--text2)'; }}
            ><span style={{ fontSize: 14 }}>{qc.icon}</span> {qc.label}</button>
          ))}
        </div>
      </div>

      {/* 暖服功能 */}
      <div className="card">
        <div className="card-header" style={{ padding: '10px 14px' }}>
          <div className="card-title" style={{ fontSize: 13 }}>🔥 暖服功能</div>
          <div className="card-sub">快速开关暖服作弊选项</div>
        </div>
        <div className="card-body" style={{ padding: '8px 14px', display: 'flex', flexDirection: 'column', gap: 6 }}>
          {WARMUP_ITEMS.map(item => {
            const state = warmupToggles[item.key];
            return (
              <div key={item.key} style={{
                display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                padding: '6px 10px', borderRadius: 6,
                background: state === true ? 'rgba(34,197,94,0.06)' : state === false ? 'rgba(239,68,68,0.04)' : 'var(--bg3)',
                border: `1px solid ${state === true ? 'rgba(34,197,94,0.2)' : state === false ? 'rgba(239,68,68,0.15)' : 'var(--border)'}`,
                transition: 'all .15s',
              }}>
                <span style={{ fontSize: 12, fontWeight: 500, color: state === true ? '#22c55e' : state === false ? 'var(--red)' : 'var(--text2)' }}>{item.label}</span>
                <div style={{ display: 'flex', gap: 4, flexShrink: 0, marginLeft: 8 }}>
                  <button onClick={() => onWarmupToggle(item.key, true)} style={{
                    padding: '2px 10px', borderRadius: 4, border: 'none', cursor: 'pointer', fontSize: 10, fontWeight: 700,
                    background: state === true ? '#22c55e' : 'rgba(34,197,94,0.12)',
                    color: state === true ? '#fff' : 'rgba(34,197,94,0.6)',
                    transition: 'all .1s',
                  }}>开启</button>
                  <button onClick={() => onWarmupToggle(item.key, false)} style={{
                    padding: '2px 10px', borderRadius: 4, border: 'none', cursor: 'pointer', fontSize: 10, fontWeight: 700,
                    background: state === false ? 'var(--red)' : 'rgba(239,68,68,0.12)',
                    color: state === false ? '#fff' : 'rgba(239,68,68,0.5)',
                    transition: 'all .1s',
                  }}>关闭</button>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* 广播 */}
      <div className="card">
        <div className="card-header" style={{ padding: '10px 14px' }}>
          <div className="card-title" style={{ fontSize: 13 }}>📢 游戏广播</div>
        </div>
        <div className="card-body" style={{ padding: '10px 14px', display: 'flex', flexDirection: 'column', gap: 8 }}>
          <input type="text" className="rcon-input" placeholder="输入广播内容..."
            value={broadcastMsg} onChange={e => onBroadcastMsgChange(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && onSendBroadcast()}
            style={{ fontSize: 12, padding: '8px 10px' }} />
          <button className="rcon-btn" onClick={onSendBroadcast} disabled={!broadcastMsg}
            style={{ fontSize: 12, padding: '8px 14px', opacity: broadcastMsg ? 1 : 0.4 }}>发送广播</button>
        </div>
      </div>
    </div>
  );
}
