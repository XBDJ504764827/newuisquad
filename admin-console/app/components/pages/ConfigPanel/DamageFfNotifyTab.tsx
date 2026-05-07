'use client';

interface TkForm {
  enabled: boolean;
  max_team_kills: number;
  apology_time_minutes: number;
  apology_keyword: string;
  notification_message: string;
  tk_broadcast_message: string;
}

interface DamageNotifyForm {
  enabled: boolean;
  notify_kill: boolean;
  notify_damage: boolean;
}

interface Props {
  selectedServerId: number | null;
  tkLoading: boolean;
  damageNotifyLoading: boolean;
  tkForm: TkForm;
  damageNotifyForm: DamageNotifyForm;
  tkError: string;
  damageNotifyError: string;
  tkSaving: boolean;
  damageNotifySaving: boolean;
  onTkFormChange: (f: TkForm) => void;
  onDamageNotifyFormChange: (f: DamageNotifyForm) => void;
  onSaveTk: () => void;
  onSaveDamageNotify: () => void;
  onSaveAll: () => void;
}

export function DamageFfNotifyTab({
  selectedServerId,
  tkLoading, damageNotifyLoading,
  tkForm, damageNotifyForm,
  tkError, damageNotifyError,
  tkSaving, damageNotifySaving,
  onTkFormChange, onDamageNotifyFormChange,
  onSaveTk, onSaveDamageNotify,
  onSaveAll,
}: Props) {
  const loading = tkLoading || damageNotifyLoading;

  if (!selectedServerId) {
    return (
      <div className="tab-content" style={{ display: 'block' }}>
        <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="tab-content" style={{ display: 'block' }}>
        <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p>
      </div>
    );
  }

  return (
    <div className="tab-content" style={{ display: 'block' }}>
      <h4 style={{ marginBottom: 20 }}>伤害与误伤通知</h4>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 24, maxWidth: 560 }}>

        {/* ═══ 敌方伤害通知 ═══ */}
        <div style={{ padding: 14, background: 'var(--bg3)', borderRadius: 8, border: '1px solid var(--border)' }}>
          <div style={{ fontWeight: 600, fontSize: 14, marginBottom: 12 }}>敌方伤害通知</div>

          <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer', marginBottom: 12 }}>
            <input type="checkbox" checked={damageNotifyForm.enabled} onChange={e => onDamageNotifyFormChange({ ...damageNotifyForm, enabled: e.target.checked })} />
            <div>
              <div style={{ fontWeight: 500 }}>开启敌方伤害通知</div>
              <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>对敌方造成伤害时，通知攻击者造成的伤害数值</div>
            </div>
          </label>

          <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer', marginLeft: 28 }}>
            <input type="checkbox" checked={damageNotifyForm.notify_damage} onChange={e => onDamageNotifyFormChange({ ...damageNotifyForm, notify_damage: e.target.checked })} disabled={!damageNotifyForm.enabled} />
            <div>
              <div style={{ fontWeight: 500, fontSize: 13 }}>伤害数值通知</div>
              <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>每次造成伤害时通过 AdminWarn 通知攻击者伤害数值</div>
            </div>
          </label>

          <div style={{ marginTop: 10, marginLeft: 28, padding: '8px 12px', background: 'rgba(0,0,0,0.15)', borderRadius: 6, fontFamily: 'monospace', fontSize: 11, color: 'var(--text2)' }}>
            AdminWarn &lt;PlayerID&gt; 你对&lt;被攻击玩家&gt;造成了&lt;伤害数值&gt;点伤害
          </div>

          {damageNotifyError && <div style={{ color: 'var(--red)', fontSize: 12, marginTop: 8 }}>{damageNotifyError}</div>}
          <button className="rcon-btn" style={{ width: 'auto', padding: '8px 18px', fontSize: 12, marginTop: 12 }}
            onClick={onSaveDamageNotify} disabled={damageNotifySaving}>
            {damageNotifySaving ? '保存中...' : '保存伤害通知设置'}
          </button>
        </div>

        {/* ═══ 友方误伤处理 ═══ */}
        <div style={{ padding: 14, background: 'var(--bg3)', borderRadius: 8, border: '1px solid var(--border)' }}>
          <div style={{ fontWeight: 600, fontSize: 14, marginBottom: 12 }}>友方误伤处理</div>

          <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer', marginBottom: 16 }}>
            <input type="checkbox" checked={tkForm.enabled} onChange={e => onTkFormChange({ ...tkForm, enabled: e.target.checked })} />
            <div>
              <div style={{ fontWeight: 500 }}>开启误伤检测</div>
              <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>开启后将监控玩家误伤队友行为，发送警告并要求道歉</div>
            </div>
          </label>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>单局最大误伤人数</label>
            <input className="rcon-input" type="number" min={1} max={20} style={{ width: 100 }} value={tkForm.max_team_kills}
              onChange={e => onTkFormChange({ ...tkForm, max_team_kills: parseInt(e.target.value) || 0 })} />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>玩家单局误伤队友超过此次数且未道歉将被踢出</p>
          </div>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16, marginTop: 16 }}>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>道歉关键字</label>
            <input className="rcon-input" style={{ width: 150 }} value={tkForm.apology_keyword}
              onChange={e => onTkFormChange({ ...tkForm, apology_keyword: e.target.value })}
              placeholder="sry" />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>玩家在聊天中发送此关键字即视为道歉。例如设为 sry，玩家发送 sry 即道歉成功</p>
          </div>

          <div style={{ marginTop: 16 }}>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>道歉时间（分钟）</label>
            <input className="rcon-input" type="number" min={1} max={60} style={{ width: 100 }} value={tkForm.apology_time_minutes}
              onChange={e => onTkFormChange({ ...tkForm, apology_time_minutes: parseInt(e.target.value) || 0 })} />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>超过此时间未输入道歉关键字将被踢出服务器</p>
          </div>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16, marginTop: 16 }}>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>黄字广播消息（AdminBroadcast）</label>
            <textarea className="rcon-input" rows={2} style={{ resize: 'vertical' }} value={tkForm.tk_broadcast_message}
              onChange={e => onTkFormChange({ ...tkForm, tk_broadcast_message: e.target.value })}
              placeholder="默认: &lt;攻击玩家&gt;误伤了队友&lt;被攻击玩家&gt;，输入&lt;道歉关键字&gt;道歉，否则将在&lt;道歉时间&gt;分钟后被踢出" />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>玩家误伤队友时向全服发送的黄字广播。留空使用默认消息</p>
          </div>

          <div style={{ marginTop: 16 }}>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>警告消息（AdminWarn）</label>
            <textarea className="rcon-input" rows={3} style={{ resize: 'vertical' }} value={tkForm.notification_message}
              onChange={e => onTkFormChange({ ...tkForm, notification_message: e.target.value })}
              placeholder="默认: 你对队友&lt;被攻击玩家&gt;造成了&lt;伤害数值&gt;点伤害" />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>发送给误伤玩家的警告消息。留空使用默认消息</p>
          </div>

          <div style={{ marginTop: 16, padding: '8px 12px', background: 'rgba(0,0,0,0.15)', borderRadius: 6, fontFamily: 'monospace', fontSize: 11, color: 'var(--text2)', lineHeight: 1.6 }}>
            <div>AdminWarn &lt;PlayerID&gt; 你对队友&lt;被攻击玩家&gt;造成了&lt;伤害数值&gt;点伤害</div>
            <div style={{ marginTop: 4 }}>AdminBroadcast "&lt;攻击玩家&gt;误伤了队友&lt;被攻击玩家&gt;，输入&lt;道歉关键字&gt;道歉，否则将在&lt;道歉时间&gt;分钟后被踢出"</div>
          </div>

          {tkError && <div style={{ color: 'var(--red)', fontSize: 12, marginTop: 8 }}>{tkError}</div>}
          <button className="rcon-btn" style={{ width: 'auto', padding: '8px 18px', fontSize: 12, marginTop: 12 }}
            onClick={onSaveTk} disabled={tkSaving}>
            {tkSaving ? '保存中...' : '保存误伤设置'}
          </button>
        </div>

        {/* ═══ RCON 命令说明 ═══ */}
        <div style={{ padding: '12px 14px', background: 'rgba(59,130,246,0.06)', borderRadius: 8, border: '1px solid rgba(59,130,246,0.15)' }}>
          <div style={{ fontWeight: 500, fontSize: 13, marginBottom: 6 }}>RCON 命令说明</div>
          <div style={{ fontSize: 11, color: 'var(--text2)', lineHeight: 1.8 }}>
            <div>敌方伤害：<b>AdminWarn &lt;PlayerID&gt;</b> 通知攻击者伤害数值</div>
            <div>友方误伤：每次造成伤害立即 <b>AdminWarn</b> + <b>AdminBroadcast</b> 全服黄字广播</div>
            <div>每次误伤（重新）启动道歉倒计时，玩家输入道歉关键字后取消计时器</div>
            <div>玩家输入道歉关键字后发送 <b>AdminBroadcast "道歉成功"</b></div>
            <div>倒计时结束未道歉则执行 <b>AdminKick &lt;名称&gt;</b> 踢出玩家</div>
          </div>
        </div>

        <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={onSaveAll} disabled={tkSaving || damageNotifySaving}>
          {tkSaving || damageNotifySaving ? '保存中...' : '保存全部设置'}
        </button>
      </div>
    </div>
  );
}
