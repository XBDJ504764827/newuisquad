'use client';

interface DamageNotifyForm {
  enabled: boolean;
  notify_kill: boolean;
  notify_damage: boolean;
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
      <h4 style={{ marginBottom: 20 }}>伤害通知</h4>
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
                开启后，玩家造成击倒或伤害时将收到 AdminWarn 警告信息
              </div>
            </div>
          </label>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }} />

          {/* 击倒通知 */}
          <div style={{ padding: '12px 14px', background: 'var(--bg3)', borderRadius: 8, border: '1px solid var(--border)' }}>
            <label style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: 13 }}>击倒通知</div>
                <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>
                  玩家击倒另一名玩家时，发送 AdminWarn 警告攻击者
                </div>
              </div>
              <input type="checkbox" checked={damageNotifyForm.notify_kill}
                onChange={e => onFormChange({ ...damageNotifyForm, notify_kill: e.target.checked })}
                disabled={!damageNotifyForm.enabled} />
            </label>
            {damageNotifyForm.notify_kill && (
              <div style={{ marginTop: 10, padding: '8px 12px', background: 'rgba(0,0,0,0.15)', borderRadius: 6, fontFamily: 'monospace', fontSize: 12, color: 'var(--text2)' }}>
                AdminWarn "玩家名" "击倒了被击倒玩家"
              </div>
            )}
          </div>

          {/* 伤害通知 */}
          <div style={{ padding: '12px 14px', background: 'var(--bg3)', borderRadius: 8, border: '1px solid var(--border)' }}>
            <label style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: 13 }}>伤害通知</div>
                <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>
                  玩家造成伤害时，发送 AdminWarn 警告攻击者（⚠️ 可能大量刷屏）
                </div>
              </div>
              <input type="checkbox" checked={damageNotifyForm.notify_damage}
                onChange={e => onFormChange({ ...damageNotifyForm, notify_damage: e.target.checked })}
                disabled={!damageNotifyForm.enabled} />
            </label>
            {damageNotifyForm.notify_damage && (
              <div style={{ marginTop: 10, padding: '8px 12px', background: 'rgba(0,0,0,0.15)', borderRadius: 6, fontFamily: 'monospace', fontSize: 12, color: 'var(--text2)' }}>
                AdminWarn "玩家名" "对目标玩家造成了XX点伤害"
              </div>
            )}
          </div>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
            <div style={{ fontWeight: 500, fontSize: 13, marginBottom: 6 }}>RCON 命令说明</div>
            <div style={{ fontSize: 11, color: 'var(--text2)', lineHeight: 1.6 }}>
              <div>• 击倒通知使用 <b>AdminWarn</b> 对攻击者发送黄字警告</div>
              <div>• 伤害通知使用 <b>AdminWarn</b> 对攻击者发送黄字警告</div>
              <div>• 队友伤害由误杀设置功能独立处理，不在此处重复</div>
            </div>
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
