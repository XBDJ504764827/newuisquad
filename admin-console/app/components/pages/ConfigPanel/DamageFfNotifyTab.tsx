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

        {/* ═══ 全局开关 + 敌方伤害通知 ═══ */}
        <div style={{ padding: 14, background: 'var(--bg3)', borderRadius: 8, border: '1px solid var(--border)' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer', marginBottom: 16 }}>
            <input type="checkbox" checked={damageNotifyForm.enabled} onChange={e => onDamageNotifyFormChange({ ...damageNotifyForm, enabled: e.target.checked })} />
            <div>
              <div style={{ fontWeight: 600, fontSize: 14 }}>开启伤害通知功能</div>
              <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>总开关，开启后敌方伤害通知和友方误伤通知才会生效</div>
            </div>
          </label>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
            <div style={{ fontWeight: 600, fontSize: 14, marginBottom: 12 }}>敌方伤害通知</div>

            <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer' }}>
              <input type="checkbox" checked={damageNotifyForm.notify_damage} onChange={e => onDamageNotifyFormChange({ ...damageNotifyForm, notify_damage: e.target.checked })} disabled={!damageNotifyForm.enabled} />
              <div>
                <div style={{ fontWeight: 500, fontSize: 13 }}>开启敌方伤害通知</div>
                <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>每次对敌方造成伤害时通过 AdminWarn 通知攻击者伤害数值</div>
              </div>
            </label>

            <div style={{ marginTop: 10, marginLeft: 28, padding: '8px 12px', background: 'rgba(0,0,0,0.15)', borderRadius: 6, fontFamily: 'monospace', fontSize: 11, color: 'var(--text2)' }}>
              AdminWarn &lt;玩家编号&gt; 你对&lt;被攻击者&gt;造成了&lt;伤害数值&gt;点伤害
            </div>
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
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>黄字广播消息格式</label>
            <div style={{ padding: '8px 12px', background: 'rgba(0,0,0,0.15)', borderRadius: 6, fontFamily: 'monospace', fontSize: 11, color: 'var(--text2)', lineHeight: 1.6 }}>
              AdminBroadcast &lt;攻击者&gt;误伤了队友&lt;被攻击者&gt;，请输入&lt;道歉关键字&gt;道歉否则将在&lt;道歉时间&gt;分钟后被踢出
            </div>
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>广播消息由系统根据上方配置自动生成，无需手动输入</p>
          </div>

          <div style={{ marginTop: 16 }}>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>警告消息预览</label>
            <div style={{ padding: '8px 12px', background: 'rgba(0,0,0,0.15)', borderRadius: 6, fontFamily: 'monospace', fontSize: 11, color: 'var(--text2)', lineHeight: 1.6 }}>
              AdminBroadcast &lt;攻击者&gt;误伤了队友&lt;被攻击者&gt;，请输入{tkForm.apology_keyword || '<道歉关键字>'}道歉否则将在{tkForm.apology_time_minutes || '<道歉时间>'}分钟后被踢出
            </div>
          </div>

          <div style={{ marginTop: 16, padding: '8px 12px', background: 'rgba(0,0,0,0.15)', borderRadius: 6, fontFamily: 'monospace', fontSize: 11, color: 'var(--text2)', lineHeight: 1.6 }}>
            <div>AdminBroadcast &lt;攻击者&gt;误伤了队友&lt;被攻击者&gt;，请输入&lt;道歉关键字&gt;道歉否则将在&lt;道歉时间&gt;分钟后被踢出</div>
            <div style={{ marginTop: 4 }}>超时未道歉: AdminKickById &lt;玩家编号&gt; &lt;踢出理由&gt;</div>
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
            <div>敌方伤害：<b>AdminWarn &lt;玩家编号&gt;</b> 通知攻击者伤害数值</div>
            <div>友方误伤：<b>AdminBroadcast</b> 黄字广播通知全服</div>
            <div>每次误伤（重新）启动道歉倒计时，玩家输入道歉关键字后取消计时器</div>
            <div>玩家输入道歉关键字后广播"道歉成功，已取消踢出"</div>
            <div>倒计时结束未道歉则执行 <b>AdminKickById &lt;玩家编号&gt;</b> 踢出玩家</div>
            <div>通过服务器状态缓存中的队伍信息（team_id）自动判断敌友</div>
          </div>
        </div>

        <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={onSaveAll} disabled={tkSaving || damageNotifySaving}>
          {tkSaving || damageNotifySaving ? '保存中...' : '保存全部设置'}
        </button>
      </div>
    </div>
  );
}
