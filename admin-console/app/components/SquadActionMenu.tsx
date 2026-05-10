'use client';

import { useState } from 'react';
import { api } from '../lib/api';

export interface Squad {
  id: number; name: string; size: number; locked: boolean;
  leader: { name: string } | null; players: { name: string; steam_id: string }[];
  teamId: number;
}

interface Props {
  squad: Squad;
  serverId: number;
  onActionComplete?: () => void;
}

export default function SquadActionMenu({ squad, serverId, onActionComplete }: Props) {
  const [open, setOpen] = useState(false);
  const [dialogAction, setDialogAction] = useState<'disband' | 'swap-team' | null>(null);
  const [loading, setLoading] = useState(false);

  const close = () => { setDialogAction(null); setOpen(false); };

  const execute = async () => {
    if (!dialogAction) return;
    setLoading(true);
    try {
      if (dialogAction === 'disband') {
        await api(`/servers/${serverId}/disband-squad/${squad.teamId}/${squad.id}`, { method: 'DELETE' });
      } else if (dialogAction === 'swap-team') {
        // Move each player in the squad
        for (const p of squad.players) {
          await api(`/servers/${serverId}/player-action`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ steam_id: p.steam_id, action: 'move' }),
          });
        }
      }
      onActionComplete?.();
    } catch {}
    setLoading(false);
    close();
  };

  const leaderName = squad.leader?.name || squad.players.find(p => squad.leader)?.name || '无队长';

  const s = {
    trigger: { background: 'none', border: 'none', cursor: 'pointer', padding: '4px 6px', borderRadius: 4, color: 'var(--text2)', fontSize: 16 },
    menu: { position: 'fixed' as const, zIndex: 999, background: 'var(--bg2)', borderRadius: 8, border: '1px solid var(--border)', minWidth: 180, padding: '4px 0', boxShadow: '0 8px 24px rgba(0,0,0,0.4)' },
    item: (c: string) => ({ display: 'flex', alignItems: 'center', gap: 8, padding: '8px 14px', cursor: 'pointer', fontSize: 13, color: c, background: 'none', border: 'none', width: '100%', textAlign: 'left' as const }),
    overlay: { position: 'fixed' as const, inset: 0, background: 'rgba(0,0,0,0.4)', zIndex: 1000, display: 'flex', alignItems: 'center', justifyContent: 'center' },
    dialog: { background: 'var(--bg)', borderRadius: 12, border: '1px solid var(--border)', padding: 20, width: '90vw', maxWidth: 420 },
    btnSm: { padding: '6px 14px', borderRadius: 6, border: '1px solid var(--border)', cursor: 'pointer', fontSize: 12 },
  };

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      <button style={s.trigger} onClick={(e) => { e.stopPropagation(); setOpen(!open); }} title="小队操作">⋮</button>

      {open && (
        <>
          <div style={{ position: 'fixed', inset: 0, zIndex: 998 }} onClick={() => setOpen(false)} />
          <div style={{ ...s.menu, position: 'absolute', right: 0, top: '100%' }}>
            <button style={s.item('#3b82f6')} onClick={() => { setDialogAction('swap-team'); setOpen(false); }}>⇄ 整队转移到对方队伍</button>
            <button style={s.item('#ef4444')} onClick={() => { setDialogAction('disband'); setOpen(false); }}>⊘ 解散小队</button>
          </div>
        </>
      )}

      {dialogAction && (
        <div style={s.overlay} onClick={close}>
          <div style={s.dialog} onClick={e => e.stopPropagation()}>
            <h3 style={{ margin: '0 0 8px 0', fontSize: 16 }}>
              {dialogAction === 'disband' ? `解散小队 ${squad.id}: ${squad.name}` : `转移小队 ${squad.id}: ${squad.name}`}
            </h3>
            <p style={{ margin: '0 0 16px 0', fontSize: 12, color: 'var(--text3)' }}>
              {dialogAction === 'disband'
                ? `确定要解散该小队吗？${squad.players.length} 名玩家将被移出小队。此操作不可撤销。`
                : `确定要将该小队 ${squad.players.length} 名玩家全部转移到对方队伍吗？`}
            </p>
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button onClick={close} style={{ ...s.btnSm, background: 'var(--bg2)', color: 'var(--text)' }}>取消</button>
              <button onClick={execute} disabled={loading} style={{ ...s.btnSm, background: dialogAction === 'swap-team' ? 'var(--accent)' : '#ef4444', color: '#fff', opacity: loading ? 0.6 : 1 }}>
                {loading ? '执行中...' : dialogAction === 'disband' ? '解散小队' : '转移队伍'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
