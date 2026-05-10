'use client';

import { useState } from 'react';
import { api } from '../lib/api';

export interface Player {
  name: string; steam_id: string; eos_id?: string;
  team_id?: number; squad_id?: string | number; role?: string;
  player_id?: number; is_leader?: boolean; is_admin?: boolean;
}

interface Props {
  player: Player;
  serverId: number;
  onActionComplete?: () => void;
  onViewProfile?: (steamId: string) => void;
}

export default function PlayerActionMenu({ player, serverId, onActionComplete, onViewProfile }: Props) {
  const [open, setOpen] = useState(false);
  const [dialogAction, setDialogAction] = useState<'kick' | 'ban' | 'warn' | 'move' | 'remove-squad' | null>(null);
  const [reason, setReason] = useState('');
  const [duration, setDuration] = useState(0);
  const [loading, setLoading] = useState(false);

  const close = () => { setDialogAction(null); setReason(''); setDuration(0); setOpen(false); };
  const openDialog = (action: typeof dialogAction) => { setDialogAction(action); setReason(''); setDuration(0); setOpen(false); };

  const execute = async () => {
    if (!dialogAction) return;
    setLoading(true);
    try {
      let endpoint = `/servers/${serverId}/player-action`;
      let body: any = { steam_id: player.steam_id, action: dialogAction === 'remove-squad' ? 'remove-from-squad' : dialogAction };
      if (dialogAction === 'ban') { body.duration = duration; body.reason = reason; }
      else if (dialogAction === 'warn') body.message = reason;
      else if (dialogAction === 'kick') body.reason = reason;
      await api(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
      onActionComplete?.();
    } catch {}
    setLoading(false);
    close();
  };

  const copyToClipboard = (text: string) => { navigator.clipboard?.writeText(text).catch(() => {}); };

  const s = {
    trigger: { background: 'none', border: 'none', cursor: 'pointer', padding: '4px 6px', borderRadius: 4, color: 'var(--text2)', fontSize: 16, lineHeight: 1 },
    menu: { position: 'fixed' as const, zIndex: 999, background: 'var(--bg2)', borderRadius: 8, border: '1px solid var(--border)', minWidth: 180, padding: '4px 0', boxShadow: '0 8px 24px rgba(0,0,0,0.4)' },
    item: { display: 'flex', alignItems: 'center', gap: 8, padding: '8px 14px', cursor: 'pointer', fontSize: 13, color: 'var(--text)', background: 'none', border: 'none', width: '100%', textAlign: 'left' as const },
    itemDanger: (color: string) => ({ display: 'flex', alignItems: 'center', gap: 8, padding: '8px 14px', cursor: 'pointer', fontSize: 13, color, background: 'none', border: 'none', width: '100%', textAlign: 'left' as const }),
    overlay: { position: 'fixed' as const, inset: 0, background: 'rgba(0,0,0,0.4)', zIndex: 1000, display: 'flex', alignItems: 'center', justifyContent: 'center' },
    dialog: { background: 'var(--bg)', borderRadius: 12, border: '1px solid var(--border)', padding: 20, width: '90vw', maxWidth: 440 },
    input: { width: '100%', padding: '8px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', fontSize: 13, boxSizing: 'border-box' as const, marginTop: 6 },
    btnSm: { padding: '6px 14px', borderRadius: 6, border: '1px solid var(--border)', cursor: 'pointer', fontSize: 12 },
  };

  const labelMap: Record<string, { label: string; icon: string; color: string; desc: string }> = {
    warn: { label: '警告玩家', icon: '⚠', color: '#eab308', desc: '向该玩家发送警告消息' },
    move: { label: '转至对方队伍', icon: '⇄', color: '#3b82f6', desc: '强制将该玩家转至另一支队伍' },
    'remove-squad': { label: '移出小队', icon: '✕', color: '#f59e0b', desc: '将该玩家移出其当前小队' },
    kick: { label: '踢出玩家', icon: '⏏', color: '#f97316', desc: '将该玩家踢出服务器，玩家仍可重新加入' },
    ban: { label: '封禁玩家', icon: '⊘', color: '#ef4444', desc: '按指定时长封禁该玩家' },
  };

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      <button style={s.trigger} onClick={(e) => { e.stopPropagation(); setOpen(!open); }} title="操作菜单">⋮</button>

      {open && (
        <>
          <div style={{ position: 'fixed', inset: 0, zIndex: 998 }} onClick={() => setOpen(false)} />
          <div style={{ ...s.menu, position: 'absolute', right: 0, top: '100%' }}>
            <button style={s.item} onClick={() => { onViewProfile?.(player.steam_id); setOpen(false); }}>👤 查看玩家档案</button>
            <button style={s.item} onClick={() => { copyToClipboard(player.steam_id); setOpen(false); }}>📋 复制 Steam ID</button>
            {player.eos_id && <button style={s.item} onClick={() => { copyToClipboard(player.eos_id!); setOpen(false); }}>📋 复制 EOS ID</button>}
            <div style={{ height: 1, background: 'var(--border)', margin: '4px 0' }} />
            <button style={s.itemDanger('#eab308')} onClick={() => openDialog('warn')}>⚠ 警告玩家</button>
            <button style={s.itemDanger('#3b82f6')} onClick={() => openDialog('move')}>⇄ 转至对方队伍</button>
            <button style={s.itemDanger('#f59e0b')} onClick={() => openDialog('remove-squad')}>✕ 移出小队</button>
            <button style={s.itemDanger('#f97316')} onClick={() => openDialog('kick')}>⏏ 踢出玩家</button>
            <button style={s.itemDanger('#ef4444')} onClick={() => openDialog('ban')}>⊘ 封禁玩家</button>
          </div>
        </>
      )}

      {/* Action Dialog */}
      {dialogAction && (
        <div style={s.overlay} onClick={close}>
          <div style={s.dialog} onClick={e => e.stopPropagation()}>
            <h3 style={{ margin: '0 0 8px 0', fontSize: 16 }}>{labelMap[dialogAction].label}：{player.name}</h3>
            <p style={{ margin: '0 0 16px 0', fontSize: 12, color: 'var(--text3)' }}>{labelMap[dialogAction].desc}</p>

            {dialogAction === 'ban' && (
              <div style={{ marginBottom: 12 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)' }}>封禁天数（0=永久）</label>
                <input type="number" value={duration} onChange={e => setDuration(Number(e.target.value))} style={s.input} min={0} />
              </div>
            )}
            {dialogAction !== 'move' && dialogAction !== 'remove-squad' && (
              <div style={{ marginBottom: 12 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)' }}>{dialogAction === 'warn' ? '警告消息' : '原因'}</label>
                <textarea value={reason} onChange={e => setReason(e.target.value)} rows={3} style={{ ...s.input, resize: 'vertical' }} placeholder={dialogAction === 'warn' ? '警告消息内容' : '操作原因'} />
              </div>
            )}

            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button onClick={close} style={{ ...s.btnSm, background: 'var(--bg2)', color: 'var(--text)' }}>取消</button>
              <button onClick={execute} disabled={loading} style={{ ...s.btnSm, background: dialogAction === 'warn' || dialogAction === 'move' ? 'var(--accent)' : '#ef4444', color: '#fff', opacity: loading ? 0.6 : 1 }}>
                {loading ? '执行中...' : labelMap[dialogAction].label}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
