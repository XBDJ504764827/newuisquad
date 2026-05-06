'use client';

interface TkForm { enabled: boolean; max_team_kills: number; apology_time_minutes: number; apology_keyword: string; notification_message: string; tk_broadcast_message: string; }

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
        <div style={{ display: 'flex', flexDirection: 'column', gap: 20, maxWidth: 520 }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer' }}>
            <input type="checkbox" checked={tkForm.enabled} onChange={e => onTkFormChange({...tkForm, enabled: e.target.checked})} />
            <div>
              <div style={{ fontWeight: 500 }}>开启误杀检测</div>
              <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>开启后将监控并记录玩家误杀队友行为</div>
            </div>
          </label>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>单局最大误杀人数</label>
            <input className="rcon-input" type="number" min={1} max={20} style={{ width: 100 }} value={tkForm.max_team_kills}
              onChange={e => onTkFormChange({...tkForm, max_team_kills: parseInt(e.target.value) || 0})} />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>玩家单局误杀队友数量超过此数值时，若未道歉将被踢出</p>
          </div>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>道歉关键字</label>
            <input className="rcon-input" style={{ width: 150 }} value={tkForm.apology_keyword}
              onChange={e => onTkFormChange({...tkForm, apology_keyword: e.target.value})}
              placeholder="sry" />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>玩家在聊天中发送此关键字即可道歉。例如设置成 sry，玩家发送 sry 后道歉成功</p>
          </div>

          <div>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>道歉时间（分钟）</label>
            <input className="rcon-input" type="number" min={1} max={60} style={{ width: 100 }} value={tkForm.apology_time_minutes}
              onChange={e => onTkFormChange({...tkForm, apology_time_minutes: parseInt(e.target.value) || 0})} />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>超过此时间未输入道歉关键字则被踢出服务器</p>
          </div>

          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>黄字广播消息（AdminBroadcast）</label>
            <textarea className="rcon-input" rows={2} style={{ resize: 'vertical' }} value={tkForm.tk_broadcast_message}
              onChange={e => onTkFormChange({...tkForm, tk_broadcast_message: e.target.value})}
              placeholder="默认: <玩家名称>误伤了<被击倒玩家名称>，输入sry道歉" />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>玩家误杀队友时，向全服发送的黄字广播消息。留空使用默认消息</p>
          </div>

          <div>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>黄色警告消息（AdminWarn）</label>
            <textarea className="rcon-input" rows={3} style={{ resize: 'vertical' }} value={tkForm.notification_message}
              onChange={e => onTkFormChange({...tkForm, notification_message: e.target.value})}
              placeholder="默认: 您误伤了友方XXX，请在X分钟内输入XXX道歉" />
            <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>发送给误杀玩家的黄字警告消息。{'{{time}'} 会被替换为实际道歉时间。留空使用默认消息</p>
          </div>

          <div style={{ padding: '12px 14px', background: 'rgba(59,130,246,0.06)', borderRadius: 8, border: '1px solid rgba(59,130,246,0.15)' }}>
            <div style={{ fontWeight: 500, fontSize: 13, marginBottom: 6 }}>RCON 命令说明</div>
            <div style={{ fontSize: 11, color: 'var(--text2)', lineHeight: 1.6 }}>
              <div>• 误杀发生时发送 <b>AdminBroadcast</b> 黄字广播（上方可自定义）</div>
              <div>• 同时发送 <b>AdminWarn</b> 黄字警告给误杀玩家</div>
              <div>• 玩家输入道歉关键字后发送 <b>AdminBroadcast "道歉成功"</b>（不可自定义）</div>
              <div>• 超时未道歉则执行 <b>AdminKick</b> 踢出玩家</div>
            </div>
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
