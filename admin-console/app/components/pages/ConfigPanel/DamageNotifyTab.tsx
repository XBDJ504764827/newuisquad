'use client';

interface DamageNotifyForm {
  enabled: boolean; keyword: string;
  min_damage: number; notify_tk: boolean; notify_damage: boolean;
  notify_high_damage: boolean; high_damage_threshold: number;
}

interface Props {
  selectedServerId: number | null;
  damageNotifyLoading: boolean;
  damageNotifyForm: DamageNotifyForm;
  damageNotifyError: string;
  damageNotifySaving: boolean;
  onFormChange: (f: DamageNotifyForm) => void;
  onSave: () => void;
}

export function DamageNotifyTab({ selectedServerId, damageNotifyLoading, damageNotifyForm, damageNotifyError, damageNotifySaving, onFormChange, onSave }: Props) {
  return (
    <div className="tab-content" style={{ display: 'block' }}>
      <h4 style={{ marginBottom: 20 }}>伤害 / TK 通知</h4>
      {!selectedServerId ? (
        <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
      ) : damageNotifyLoading ? (
        <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 20, maxWidth: 520 }}>
          <label style={{ display: 'flex', alignItems: 'flex-start', gap: 12, cursor: 'pointer', padding: 14, background: 'var(--bg3)', borderRadius: 8, border: '1px solid var(--border)' }}>
            <input type="checkbox" checked={damageNotifyForm.enabled} onChange={e => onFormChange({ ...damageNotifyForm, enabled: e.target.checked })} style={{ marginTop: 2 }} />
            <div>
              <div style={{ fontWeight: 600, fontSize: 13 }}>启用伤害通知服务</div>
              <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4, lineHeight: 1.5 }}>
                开启后，游戏内的伤害和 TK 事件将通过 AdminBroadcast 实时广播到聊天框
              </div>
            </div>
          </label>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }} />

          <div style={{ padding: '12px 14px', background: 'rgba(239,68,68,0.05)', borderRadius: 8, border: '1px solid rgba(239,68,68,0.15)' }}>
            <label style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: 13 }}>💀 误杀 (TK) 通知</div>
                <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>任何友军伤害事件都会广播</div>
              </div>
              <input type="checkbox" checked={damageNotifyForm.notify_tk}
                onChange={e => onFormChange({ ...damageNotifyForm, notify_tk: e.target.checked })}
                disabled={!damageNotifyForm.enabled} />
            </label>
          </div>

          <div style={{ padding: '12px 14px', background: 'var(--bg3)', borderRadius: 8, border: '1px solid var(--border)' }}>
            <label style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: 13 }}>💥 高伤害击杀通知</div>
                <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>对敌方造成一击必杀（伤害 ≥ 阈值）时广播</div>
              </div>
              <input type="checkbox" checked={damageNotifyForm.notify_high_damage}
                onChange={e => onFormChange({ ...damageNotifyForm, notify_high_damage: e.target.checked })}
                disabled={!damageNotifyForm.enabled} />
            </label>
            {damageNotifyForm.notify_high_damage && (
              <div style={{ marginTop: 10, display: 'flex', alignItems: 'center', gap: 10 }}>
                <span style={{ fontSize: 12, color: 'var(--text3)' }}>阈值：</span>
                <input type="number" className="rcon-input" style={{ width: 80, textAlign: 'center' }}
                  value={damageNotifyForm.high_damage_threshold}
                  onChange={e => onFormChange({ ...damageNotifyForm, high_damage_threshold: parseInt(e.target.value) || 0 })}
                  disabled={!damageNotifyForm.enabled} />
                <span style={{ fontSize: 12, color: 'var(--text3)' }}>伤害值</span>
              </div>
            )}
          </div>

          <div style={{ padding: '12px 14px', background: 'var(--bg3)', borderRadius: 8, border: '1px solid var(--border)' }}>
            <label style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: 13 }}>🔫 普通伤害通知</div>
                <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>非 TK 伤害超过阈值时广播（⚠️ 可能大量刷屏）</div>
              </div>
              <input type="checkbox" checked={damageNotifyForm.notify_damage}
                onChange={e => onFormChange({ ...damageNotifyForm, notify_damage: e.target.checked })}
                disabled={!damageNotifyForm.enabled} />
            </label>
            {damageNotifyForm.notify_damage && (
              <div style={{ marginTop: 10, display: 'flex', alignItems: 'center', gap: 10 }}>
                <span style={{ fontSize: 12, color: 'var(--text3)' }}>最低伤害：</span>
                <input type="number" className="rcon-input" style={{ width: 80, textAlign: 'center' }}
                  value={damageNotifyForm.min_damage}
                  onChange={e => onFormChange({ ...damageNotifyForm, min_damage: parseInt(e.target.value) || 0 })}
                  disabled={!damageNotifyForm.enabled} />
                <span style={{ fontSize: 12, color: 'var(--text3)' }}>伤害值</span>
              </div>
            )}
          </div>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }} />

          <div>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>玩家触发关键字（HUD 查询，预留功能）</label>
            <input className="rcon-input" style={{ width: 200 }} value={damageNotifyForm.keyword}
              onChange={e => onFormChange({ ...damageNotifyForm, keyword: e.target.value })}
              placeholder="!damage" />
          </div>

          {damageNotifyError && <div style={{ color: 'var(--red)', fontSize: 12 }}>{damageNotifyError}</div>}

          <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={onSave} disabled={damageNotifySaving}>
            {damageNotifySaving ? '保存中...' : '保存设置'}
          </button>
        </div>
      )}
    </div>
  );
}
