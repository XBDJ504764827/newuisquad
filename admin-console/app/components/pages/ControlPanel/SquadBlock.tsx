'use client';

import { useState } from 'react';
import { ActionBtn } from './ActionBtn';
import { ConfirmModal } from './ConfirmModal';

interface SquadBlockProps {
  squad: any; members: any[]; onAction: (name: string, action: string, msg?: string, playerId?: number) => void;
  onBan: (player: any) => void;
  onDisband: (() => void) | null; adminSteamIds?: string[]; collapsed?: boolean;
}

interface ConfirmState {
  title: string;
  message: string;
  confirmLabel: string;
  danger: boolean;
  onConfirm: () => void;
}

export function SquadBlock({ squad, members, onAction, onBan, onDisband, adminSteamIds, collapsed: forceCollapsed }: SquadBlockProps) {
  const [collapsed, setCollapsed] = useState(forceCollapsed ?? (members.length > 8));
  const [confirm, setConfirm] = useState<ConfirmState | null>(null);
  const leader = members.find((m: any) => m.is_leader);

  return (
    <div style={{ borderBottom: '1px solid var(--border)' }}>
      {confirm && (
        <ConfirmModal
          title={confirm.title}
          message={confirm.message}
          confirmLabel={confirm.confirmLabel}
          danger={confirm.danger}
          onConfirm={() => { confirm.onConfirm(); setConfirm(null); }}
          onCancel={() => setConfirm(null)}
        />
      )}

      <div
        onClick={() => setCollapsed(!collapsed)}
        style={{
          display: 'flex', justifyContent: 'space-between', alignItems: 'center',
          padding: '8px 14px', background: 'var(--bg3)', cursor: 'pointer',
          userSelect: 'none', transition: 'background .1s',
          fontSize: 12,
        }}
        onMouseEnter={e => { e.currentTarget.style.background = 'var(--bg4)'; }}
        onMouseLeave={e => { e.currentTarget.style.background = 'var(--bg3)'; }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ fontSize: 10, color: 'var(--text3)', transition: 'transform .15s', transform: collapsed ? 'rotate(-90deg)' : 'rotate(0)' }}>▼</span>
          <strong>{squad.name}</strong>
          {leader && <span style={{ fontSize: 10, color: '#f59e0b' }}>👑 {leader.name}</span>}
        </div>
        <div style={{ display: 'flex', gap: 10, alignItems: 'center' }}>
          <span style={{ fontSize: 10, color: 'var(--text3)' }}>{squad.creator || ''}</span>
          <span style={{ fontSize: 10, color: 'var(--text3)', background: 'var(--bg2)', padding: '1px 7px', borderRadius: 10 }}>{members.length}</span>
          {onDisband && (
            <span
              onClick={e => { e.stopPropagation(); onDisband(); }}
              style={{ fontSize: 10, cursor: 'pointer', color: 'var(--red)', padding: '2px 6px', borderRadius: 4, background: 'rgba(239,68,68,0.08)' }}
              title="解散小队"
            >解散</span>
          )}
        </div>
      </div>
      {!collapsed && members.length > 0 && (
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 11 }}>
          <thead>
            <tr style={{ background: 'var(--bg2)' }}>
              <th style={{ padding: '5px 14px', color: 'var(--text3)', fontWeight: 500, textAlign: 'left', fontSize: 10 }}>玩家</th>
              <th style={{ padding: '5px 6px', color: 'var(--text3)', fontWeight: 500, textAlign: 'left', fontSize: 10 }}>ID</th>
              <th style={{ padding: '5px 6px', color: 'var(--text3)', fontWeight: 500, textAlign: 'left', fontSize: 10 }}>职业</th>
              <th style={{ padding: '5px 14px', color: 'var(--text3)', fontWeight: 500, textAlign: 'right', fontSize: 10 }}>操作</th>
            </tr>
          </thead>
          <tbody>
            {members.map((p: any) => (
              <tr key={p.name + (p.steam_id || '')} style={{ borderBottom: '1px solid var(--border)', transition: 'background .1s' }}
                onMouseEnter={e => { e.currentTarget.style.background = 'var(--bg3)'; }}
                onMouseLeave={e => { e.currentTarget.style.background = 'transparent'; }}
              >
                <td style={{ padding: '5px 14px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                    <span style={{ fontWeight: 600, fontSize: 12 }}>{p.name}</span>
                    {(p.is_admin || (adminSteamIds && p.steam_id && adminSteamIds.includes(p.steam_id))) && <span style={{ color: '#f59e0b', fontSize: 9, background: 'rgba(245,158,11,0.15)', padding: '1px 5px', borderRadius: 3, fontWeight: 700, letterSpacing: '0.02em' }}>OP</span>}
                    {p.is_leader && <span style={{ color: '#f59e0b', fontSize: 9 }}>👑</span>}
                  </div>
                </td>
                <td style={{ padding: '5px 6px', color: 'var(--text2)', fontSize: 10, fontFamily: 'monospace' }}>{p.player_id}</td>
                <td style={{ padding: '5px 6px', color: 'var(--text2)', fontSize: 10 }}>{p.role}</td>
                <td style={{ padding: '5px 14px', textAlign: 'right' }}>
                  <div style={{ display: 'flex', gap: 3, justifyContent: 'flex-end' }}>
                    <ActionBtn color="var(--text2)" bg="var(--bg4)" onClick={() => setConfirm({
                      title: '警告玩家', message: `确认警告 ${p.name}？`, confirmLabel: '确认警告', danger: false,
                      onConfirm: () => onAction(p.name, 'warn', undefined, p.player_id),
                    })}>警告</ActionBtn>
                    <ActionBtn color="var(--red)" bg="rgba(239,68,68,0.08)" onClick={() => setConfirm({
                      title: '踢出玩家', message: `确认踢出 ${p.name}？该玩家可重新加入服务器。`, confirmLabel: '确认踢出', danger: true,
                      onConfirm: () => onAction(p.name, 'kick', '管理员操作', p.player_id),
                    })}>踢出</ActionBtn>
                    <ActionBtn color="var(--red)" bg="rgba(239,68,68,0.12)" onClick={() => onBan(p)}>封禁</ActionBtn>
                    <ActionBtn color="var(--blue)" bg="rgba(59,130,246,0.08)" onClick={() => setConfirm({
                      title: '强制跳边', message: `确认强制 ${p.name} 跳边到对方阵营？`, confirmLabel: '确认跳边', danger: false,
                      onConfirm: () => onAction(p.name, 'team_change', undefined, p.player_id),
                    })}>跳边</ActionBtn>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
