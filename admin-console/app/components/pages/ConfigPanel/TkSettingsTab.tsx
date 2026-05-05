'use client';

interface TkForm { enabled: boolean; max_team_kills: number; apology_time_minutes: number; notification_message: string; }

interface Props {
  selectedServerId: number | null;
  tkLoading: boolean;
  tkForm: TkForm;
  tkError: string;
  tkSaving: boolean;
  onTkFormChange: (f: TkForm) => void;
  onSave: () => void;
}

export function TkSettingsTab({ selectedServerId, tkLoading, tkForm, tkError, tkSaving, onTkFormChange, onSave }: Props) {
  return (
    <div className="tab-content" style={{ display: 'block' }}>
      <h4 style={{ marginBottom: 20 }}>误杀设置 (TK)</h4>
      {!selectedServerId ? (
        <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
      ) : tkLoading ? (
        <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 20, maxWidth: 500 }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer' }}>
            <input type="checkbox" checked={tkForm.enabled} onChange={e => onTkFormChange({...tkForm, enabled: e.target.checked})} />
            <div>
              <div style={{ fontWeight: 500 }}>开启误杀检测</div>
              <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>开启后将监控玩家误杀队友行为</div>
            </div>
          </label>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>单局最大误杀人数</label>
            <input className="rcon-input" type="number" min={1} max={20} style={{ width: 100 }} value={tkForm.max_team_kills}
              onChange={e => onTkFormChange({...tkForm, max_team_kills: parseInt(e.target.value) || 0})} />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>超过此数值后玩家将被踢出服务器</p>
          </div>

          <div>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>道歉时间（分钟）</label>
            <input className="rcon-input" type="number" min={1} max={60} style={{ width: 100 }} value={tkForm.apology_time_minutes}
              onChange={e => onTkFormChange({...tkForm, apology_time_minutes: parseInt(e.target.value) || 0})} />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>超过此时间未道歉则被踢出服务器</p>
          </div>

          <div>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>自定义通知消息</label>
            <textarea className="rcon-input" rows={3} style={{ resize: 'vertical' }} value={tkForm.notification_message}
              onChange={e => onTkFormChange({...tkForm, notification_message: e.target.value})}
              placeholder="误杀队友将被踢出服务器，请在 {time} 分钟内输入 !sorry 道歉。" />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>{'{time}'} 会被自动替换为实际道歉时间</p>
          </div>

          {tkError && <div style={{ color: 'var(--red)', fontSize: 12 }}>{tkError}</div>}

          <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={onSave} disabled={tkSaving}>
            {tkSaving ? '保存中...' : '保存设置'}
          </button>
        </div>
      )}
    </div>
  );
}
