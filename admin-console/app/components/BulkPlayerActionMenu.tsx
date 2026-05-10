'use client';

import { useState } from 'react';
import { api } from '../lib/api';

interface Player { name: string; steam_id: string; }

interface Props {
  selectedPlayers: Player[];
  serverId: number;
  onActionComplete?: () => void;
  onClearSelection?: () => void;
  children?: React.ReactNode;
}

export default function BulkPlayerActionMenu({ selectedPlayers, serverId, onActionComplete, onClearSelection, children }: Props) {
  const [dialogAction, setDialogAction] = useState<'kick' | 'ban' | 'warn' | 'move' | null>(null);
  const [reason, setReason] = useState('');
  const [duration, setDuration] = useState(0);
  const [loading, setLoading] = useState(false);
  const [contextOpen, setContextOpen] = useState(false);
  const [ctxPos, setCtxPos] = useState({ x: 0, y: 0 });

  const close = () => { setDialogAction(null); setReason(''); setDuration(0); setContextOpen(false); };

  const handleContextMenu = (e: React.MouseEvent) => {
    if (selectedPlayers.length === 0) return;
    e.preventDefault();
    setCtxPos({ x: e.clientX, y: e.clientY });
    setContextOpen(true);
  };

  const execute = async () => {
    if (!dialogAction) return;
    setLoading(true);
    let success = 0, fail = 0;
    for (const p of selectedPlayers) {
      try {
        let body: any = { steam_id: p.steam_id, action: dialogAction };
        if (dialogAction === 'ban') { body.duration = duration; body.reason = reason; }
        else if (dialogAction === 'warn') body.message = reason;
        else if (dialogAction === 'kick') body.reason = reason;
        await api(`/servers/${serverId}/player-action`, {
          method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body),
        });
        success++;
      } catch { fail++; }
    }
    onActionComplete?.();
    onClearSelection?.();
    setLoading(false);
    close();
  };

  const s = {
    ctxMenu: { position: 'fixed' as const, zIndex: 999, left: ctxPos.x, top: ctxPos.y, background: 'var(--bg2)', borderRadius: 8, border: '1px solid var(--border)', minWidth: 200, padding: '4px 0', boxShadow: '0 8px 24px rgba(0,0,0,0.4)' },
    item: (c: string) => ({ display: 'flex', alignItems: 'center', gap: 8, padding: '8px 14px', cursor: 'pointer', fontSize: 13, color: c, background: 'none', border: 'none', width: '100%', textAlign: 'left' as const }),
    overlay: { position: 'fixed' as const, inset: 0, background: 'rgba(0,0,0,0.4)', zIndex: 1000, display: 'flex', alignItems: 'center', justifyContent: 'center' },
    dialog: { background: 'var(--bg)', borderRadius: 12, border: '1px solid var(--border)', padding: 20, width: '90vw', maxWidth: 440 },
    input: { width: '100%', padding: '8px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', fontSize: 13, boxSizing: 'border-box' as const, marginTop: 6 },
    btnSm: { padding: '6px 14px', borderRadius: 6, border: '1px solid var(--border)', cursor: 'pointer', fontSize: 12 },
  };

  const labels: Record<string, string> = { kick: '踢出', ban: '封禁', warn: '警告', move: '转移' };

  return (
    <div onContextMenu={handleContextMenu}>
      {children}

      {contextOpen && selectedPlayers.length > 0 && (
        <>
          <div style={{ position: 'fixed', inset: 0, zIndex: 998 }} onClick={() => setContextOpen(false)} />
          <div style={s.ctxMenu}>
            <div style={{ padding: '6px 14px', fontSize: 11, color: 'var(--text3)', fontWeight: 600 }}>已选择 {selectedPlayers.length} 名玩家</div>
            <div style={{ height: 1, background: 'var(--border)', margin: '4px 0' }} />
            <button style={s.item('#eab308')} onClick={() => { setDialogAction('warn'); setContextOpen(false); }}>⚠ 警告选中玩家</button>
            <button style={s.item('#3b82f6')} onClick={() => { setDialogAction('move'); setContextOpen(false); }}>⇄ 转移到对方队伍</button>
            <button style={s.item('#f97316')} onClick={() => { setDialogAction('kick'); setContextOpen(false); }}>⏏ 踢出选中玩家</button>
            <button style={s.item('#ef4444')} onClick={() => { setDialogAction('ban'); setContextOpen(false); }}>⊘ 封禁选中玩家</button>
            <div style={{ height: 1, background: 'var(--border)', margin: '4px 0' }} />
            <button style={s.item('var(--text2)')} onClick={() => { onClearSelection?.(); setContextOpen(false); }}>✕ 清空选择</button>
          </div>
        </>
      )}

      {dialogAction && (
        <div style={s.overlay} onClick={close}>
          <div style={s.dialog} onClick={e => e.stopPropagation()}>
            <h3 style={{ margin: '0 0 8px 0', fontSize: 16 }}>{labels[dialogAction]} {selectedPlayers.length} 名玩家</h3>
            <p style={{ margin: '0 0 16px 0', fontSize: 12, color: 'var(--text3)' }}>将对选中的 {selectedPlayers.length} 名玩家执行批量{labels[dialogAction]}操作</p>
            {dialogAction === 'ban' && (
              <div style={{ marginBottom: 12 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)' }}>封禁天数（0=永久）</label>
                <input type="number" value={duration} onChange={e => setDuration(Number(e.target.value))} style={s.input} min={0} />
              </div>
            )}
            {dialogAction !== 'move' && (
              <div style={{ marginBottom: 12 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)' }}>{dialogAction === 'warn' ? '警告消息' : '原因'}</label>
                <textarea value={reason} onChange={e => setReason(e.target.value)} rows={3} style={{ ...s.input, resize: 'vertical' }} />
              </div>
            )}
            <div style={{ maxHeight: 100, overflow: 'auto', marginBottom: 12, fontSize: 12, color: 'var(--text2)' }}>
              {selectedPlayers.map(p => <div key={p.steam_id} style={{ padding: '2px 0' }}>{p.name}</div>)}
            </div>
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button onClick={close} style={{ ...s.btnSm, background: 'var(--bg2)', color: 'var(--text)' }}>取消</button>
              <button onClick={execute} disabled={loading} style={{ ...s.btnSm, background: dialogAction === 'warn' || dialogAction === 'move' ? 'var(--accent)' : '#ef4444', color: '#fff', opacity: loading ? 0.6 : 1 }}>
                {loading ? '执行中...' : `${labels[dialogAction]} ${selectedPlayers.length} 名玩家`}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
