'use client';

import { useState } from 'react';
import { InfoRow } from './InfoRow';
import { ConfirmModal } from './ConfirmModal';

const QUICK_COMMANDS = [
  { label: '立即结束当前对局', cmd: 'AdminEndMatch', icon: '🏁' },
  { label: '立即暂停当前对局', cmd: 'AdminPauseMatch', icon: '⏸️' },
  { label: '继续当前对局', cmd: 'AdminUnpauseMatch', icon: '▶️' },
  { label: '重新开始当前对局', cmd: 'AdminRestartMatch', icon: '🔄' },
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
  slomoValue: number;
  onRconCommandChange: (v: string) => void;
  onSendRcon: (cmd?: string) => void;
  onDeleteServer: (s: any) => void;
  onBroadcastMsgChange: (v: string) => void;
  onSendBroadcast: () => void;
  onWarmupToggle: (key: string, value: boolean) => void;
  onSlomoChange: (v: number) => void;
}

export function LeftSidePanel({
  selectedServer, rconCommand, rconResult, broadcastMsg, warmupToggles, slomoValue,
  onRconCommandChange, onSendRcon, onDeleteServer,
  onBroadcastMsgChange, onSendBroadcast, onWarmupToggle, onSlomoChange,
}: LeftSidePanelProps) {
  const [cmdConfirm, setCmdConfirm] = useState<string | null>(null);
  if (!selectedServer) return null;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      {cmdConfirm && (
        <ConfirmModal
          title="执行快捷命令"
          message={`确认执行「${cmdConfirm}」？此操作将立即生效。`}
          confirmLabel="确认执行"
          danger={true}
          onConfirm={() => { onSendRcon(QUICK_COMMANDS.find(q => q.label === cmdConfirm)?.cmd); setCmdConfirm(null); }}
          onCancel={() => setCmdConfirm(null)}
        />
      )}
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
            <button key={qc.cmd} onClick={() => setCmdConfirm(qc.label)}
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

          {/* 服务器时间倍数 */}
          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 10, marginTop: 4 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
              <span style={{ fontSize: 12, fontWeight: 500, color: 'var(--text2)' }}>⏱️ 时间倍数</span>
              <span style={{ fontSize: 12, fontWeight: 700, color: slomoValue !== 1 ? '#f59e0b' : 'var(--text2)', fontFamily: 'monospace' }}>{slomoValue}x</span>
            </div>
            <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
              <input
                type="range"
                min={0} max={20} step={0.1}
                value={slomoValue}
                onChange={e => onSlomoChange(parseFloat(e.target.value))}
                style={{ flex: 1, accentColor: 'var(--blue)', height: 4 }}
              />
              <input
                type="number"
                min={0} max={20} step={0.1}
                value={slomoValue}
                onChange={e => { const v = parseFloat(e.target.value); if (!isNaN(v) && v >= 0 && v <= 20) onSlomoChange(v); }}
                style={{
                  width: 52, padding: '3px 6px', fontSize: 11, textAlign: 'center',
                  background: 'var(--bg2)', border: '1px solid var(--border)',
                  borderRadius: 4, color: 'var(--text)', fontFamily: 'monospace',
                }}
              />
              <button
                onClick={() => onSendRcon(`AdminSlomo ${slomoValue}`)}
                style={{
                  padding: '4px 10px', borderRadius: 4, border: 'none', cursor: 'pointer',
                  fontSize: 10, fontWeight: 600, background: 'var(--blue)', color: '#fff',
                  whiteSpace: 'nowrap', transition: 'all .1s',
                }}
              >应用</button>
            </div>
            <div style={{ fontSize: 10, color: 'var(--text3)', marginTop: 4 }}>
              0=暂停 1=正常 0-20可调
            </div>
          </div>
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
