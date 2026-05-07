'use client';

import { useState, useEffect, useCallback } from 'react';

import { api } from '../../lib/api';
import { DamageFfNotifyTab } from './ConfigPanel/DamageFfNotifyTab';
import { ConfirmModal } from './ControlPanel/ConfirmModal';

const configTabs = [
  { id: 'tab-1', label: '伤害与误伤通知' },
  { id: 'tab-2', label: '挂机设置' },
  { id: 'tab-3', label: '代码跳边设置' },
  { id: 'tab-4', label: '广播设置' },
  { id: 'tab-5', label: '队伍设置' },
  { id: 'tab-7', label: '异常伤害' },
];

interface AfkSettings {
  id: number; server_id: number; enabled: boolean;
  min_players_to_check: number; max_afk_minutes: number;
}

interface TkSettings {
  id: number;
  server_id: number;
  enabled: boolean;
  max_team_kills: number;
  apology_time_minutes: number;
  apology_keyword: string;
  notification_message: string | null;
  tk_broadcast_message: string | null;
}

export default function ConfigPanelPage() {
  const [activeTab, setActiveTab] = useState('tab-1');
  const [servers, setServers] = useState<{ id: number; name: string }[]>([]);
  const [selectedServerId, setSelectedServerId] = useState<number | null>(null);

  // TK settings state
  const [tkSettings, setTkSettings] = useState<TkSettings | null>(null);
  const [tkLoading, setTkLoading] = useState(false);
  const [tkSaving, setTkSaving] = useState(false);
  const [tkError, setTkError] = useState('');
  const [tkForm, setTkForm] = useState({ enabled: false, max_team_kills: 3, apology_time_minutes: 5, apology_keyword: 'sry', notification_message: '', tk_broadcast_message: '' });
  // AFK settings state
  const [afkLoading, setAfkLoading] = useState(false);
  const [afkSaving, setAfkSaving] = useState(false);
  const [afkError, setAfkError] = useState('');
  const [afkForm, setAfkForm] = useState({ enabled: false, min_players_to_check: 10, max_afk_minutes: 15 });
  // Broadcast settings state
  const [bcLoading, setBcLoading] = useState(false);
  const [bcSaving, setBcSaving] = useState(false);
  const [bcError, setBcError] = useState('');
  const [bcForm, setBcForm] = useState({
    join_message_enabled: true, join_message: '欢迎 {player} 加入服务器',
    gameop_list_enabled: false, gameop_list_message: '在线管理员: {oplist}',
    announcement_enabled: false, announcement_content: '', announcement_interval: 10,
  });
  // Announcements list
  const [announcements, setAnnouncements] = useState<{ id: number; content: string; interval_minutes: number; enabled: boolean }[]>([]);
  const [newAnn, setNewAnn] = useState({ content: '', interval_minutes: 10 });
  // Auto-replies list
  const [autoReplies, setAutoReplies] = useState<{ id: number; keyword: string; reply_message: string; enabled: boolean }[]>([]);
  const [newReply, setNewReply] = useState({ keyword: '', reply_message: '' });
  // Team settings state
  const [teamLoading, setTeamLoading] = useState(false);
  const [teamSaving, setTeamSaving] = useState(false);
  const [teamError, setTeamError] = useState('');
  const [teamForm, setTeamForm] = useState({ create_team_broadcast: true, captain_time_check: false, captain_min_playtime: 30, captain_check_min_players: 20, max_create_team_attempts: 3 });
  // Damage notify settings state
  const [damageNotifyLoading, setDamageNotifyLoading] = useState(false);
  const [damageNotifySaving, setDamageNotifySaving] = useState(false);
  const [damageNotifyError, setDamageNotifyError] = useState('');
  const [damageNotifyForm, setDamageNotifyForm] = useState({
    enabled: false, notify_kill: true, notify_damage: true,
  });
  // Abnormal damage state
  const [abDamageLoading, setAbDamageLoading] = useState(false);
  const [abDamageSaving, setAbDamageSaving] = useState(false);
  const [abDamageError, setAbDamageError] = useState('');
  const [abDamageEnabled, setAbDamageEnabled] = useState(false);
  const [abDamageRules, setAbDamageRules] = useState<{ id: number; server_id: number; max_damage: number; created_at: string }[]>([]);
  const [newAbDamage, setNewAbDamage] = useState('');
  const [abDamageLogs, setAbDamageLogs] = useState<{
    id: number; player_name: string; player_steamid64: string; victim_name: string;
    victim_steamid64: string; weapon: string; damage: number;
    attacker_faction: string; victim_faction: string; logged_at: string;
  }[]>([]);
  const [abDamageLogsLoading, setAbDamageLogsLoading] = useState(false);
  const [abDamageLogsQuery, setAbDamageLogsQuery] = useState('');
  // Team switch settings state
  const [tsLoading, setTsLoading] = useState(false);
  const [tsSaving, setTsSaving] = useState(false);
  const [tsEnabled, setTsEnabled] = useState(false);

  const [deleteConfirm, setDeleteConfirm] = useState<{ title: string; message: string; onConfirm: () => void } | null>(null);
  const [successMsg, setSuccessMsg] = useState('');

  const showSuccess = (msg: string) => { setSuccessMsg(msg); setTimeout(() => setSuccessMsg(''), 3000); };

  useEffect(() => {
    api(`/servers`)
      .then(r => r.json())
      .then(data => {
        setServers(data.data || []);
        if (data.data?.length > 0) setSelectedServerId(data.data[0].id);
      })
      .catch(() => {});
  }, []);

  // 加载 TK 设置
  useEffect(() => {
    if (!selectedServerId) return;
    setTkLoading(true);
    api(`/servers/${selectedServerId}/tk-settings`)
      .then(r => r.json())
      .then(data => {
        setTkSettings(data);
        setTkForm({ enabled: data.enabled, max_team_kills: data.max_team_kills, apology_time_minutes: data.apology_time_minutes, apology_keyword: data.apology_keyword || 'sry', notification_message: data.notification_message || '', tk_broadcast_message: data.tk_broadcast_message || '' });
        setTkLoading(false);
      })
      .catch(() => setTkLoading(false));
    // 加载广播设置
    setBcLoading(true);
    api(`/servers/${selectedServerId}/broadcast-settings`)
      .then(r => r.json())
      .then(data => { setBcForm({ join_message_enabled: data.join_message_enabled, join_message: data.join_message, gameop_list_enabled: data.gameop_list_enabled, gameop_list_message: data.gameop_list_message, announcement_enabled: data.announcement_enabled, announcement_content: data.announcement_content || '', announcement_interval: data.announcement_interval }); setBcLoading(false); })
      .catch(() => setBcLoading(false));
    api(`/servers/${selectedServerId}/announcements`).then(r => r.json()).then(d => setAnnouncements(d.data || [])).catch(() => {});
    api(`/servers/${selectedServerId}/auto-replies`).then(r => r.json()).then(d => setAutoReplies(d.data || [])).catch(() => {});
    // 加载队伍设置
    setTeamLoading(true);
    api(`/servers/${selectedServerId}/team-settings`).then(r => r.json())
      .then(d => { setTeamForm({ create_team_broadcast: d.create_team_broadcast, captain_time_check: d.captain_time_check, captain_min_playtime: d.captain_min_playtime, captain_check_min_players: d.captain_check_min_players, max_create_team_attempts: d.max_create_team_attempts }); setTeamLoading(false); })
      .catch(() => setTeamLoading(false));
    // 加载伤害通知设置
    setDamageNotifyLoading(true);
    api(`/servers/${selectedServerId}/damage-notify-settings`).then(r => r.json())
      .then(d => { setDamageNotifyForm({ enabled: d.enabled, notify_kill: d.notify_kill ?? true, notify_damage: d.notify_damage ?? true }); setDamageNotifyLoading(false); })
      .catch(() => setDamageNotifyLoading(false));
    // 加载异常伤害设置
    setAbDamageLoading(true);
    api(`/servers/${selectedServerId}/abnormal-damage-config`).then(r => r.json())
      .then(d => { setAbDamageEnabled(d.enabled); setAbDamageLoading(false); })
      .catch(() => setAbDamageLoading(false));
    api(`/servers/${selectedServerId}/abnormal-damage-rules`).then(r => r.json())
      .then(d => setAbDamageRules(d.data || []))
      .catch(() => {});
    // 加载异常伤害日志
    fetchAbDamageLogs();
    // 加载代码跳边设置
    setTsLoading(true);
    api(`/servers/${selectedServerId}/team-switch-config`).then(r => r.json())
      .then(d => { setTsEnabled(d.enabled); setTsLoading(false); })
      .catch(() => setTsLoading(false));
  }, [selectedServerId]);

  const saveTkSettings = useCallback(async () => {
    if (!selectedServerId) return;
    setTkSaving(true);
    setTkError('');
    const res = await api(`/servers/${selectedServerId}/tk-settings`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(tkForm),
    });
    const data = await res.json();
    setTkSaving(false);
    if (data.error) { setTkError(data.error); } else { setTkSettings(data); showSuccess('误杀设置已保存'); }
  }, [selectedServerId, tkForm]);

  const saveAfkSettings = useCallback(async () => {
    if (!selectedServerId) return; setAfkSaving(true); setAfkError('');
    const res = await api(`/servers/${selectedServerId}/afk-settings`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(afkForm) });
    const data = await res.json(); setAfkSaving(false);
    if (data.error) { setAfkError(data.error); } else { showSuccess('挂机设置已保存'); }
  }, [selectedServerId, afkForm]);

  const saveBroadcast = useCallback(async () => {
    if (!selectedServerId) return; setBcSaving(true); setBcError('');
    const res = await api(`/servers/${selectedServerId}/broadcast-settings`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(bcForm) });
    const data = await res.json(); setBcSaving(false);
    if (data.error) { setBcError(data.error); } else { showSuccess('广播设置已保存'); }
  }, [selectedServerId, bcForm]);

  const saveTeamSettings = useCallback(async () => {
    if (!selectedServerId) return; setTeamSaving(true); setTeamError('');
    const res = await api(`/servers/${selectedServerId}/team-settings`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(teamForm) });
    const data = await res.json(); setTeamSaving(false);
    if (data.error) setTeamError(data.error); else showSuccess('队伍设置已保存');
  }, [selectedServerId, teamForm]);

  const saveDamageNotifySettings = useCallback(async () => {
    if (!selectedServerId) return; setDamageNotifySaving(true); setDamageNotifyError('');
    const res = await api(`/servers/${selectedServerId}/damage-notify-settings`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(damageNotifyForm) });
    const data = await res.json(); setDamageNotifySaving(false);
    if (data.error) setDamageNotifyError(data.error); else showSuccess('伤害通知设置已保存');
  }, [selectedServerId, damageNotifyForm]);

  const saveAllDamageFf = useCallback(async () => {
    await saveTkSettings();
    await saveDamageNotifySettings();
  }, [saveTkSettings, saveDamageNotifySettings]);

  const saveAbDamageConfig = useCallback(async () => {
    if (!selectedServerId) return; setAbDamageSaving(true); setAbDamageError('');
    const res = await api(`/servers/${selectedServerId}/abnormal-damage-config`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ enabled: abDamageEnabled }) });
    const data = await res.json(); setAbDamageSaving(false);
    if (data.error) setAbDamageError(data.error); else showSuccess('异常伤害设置已保存');
  }, [selectedServerId, abDamageEnabled]);

  const saveTsConfig = useCallback(async () => {
    if (!selectedServerId) return; setTsSaving(true);
    const res = await api(`/servers/${selectedServerId}/team-switch-config`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ enabled: tsEnabled }) });
    const data = await res.json(); setTsSaving(false);
    if (data.error) showSuccess('代码跳边设置保存失败'); else showSuccess('代码跳边设置已保存');
  }, [selectedServerId, tsEnabled]);

  const addAbDamageRule = useCallback(async () => {
    if (!selectedServerId || !newAbDamage) return;
    const v = parseInt(newAbDamage);
    if (!v || v <= 0) return;
    const res = await api(`/servers/${selectedServerId}/abnormal-damage-rules`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ max_damage: v }) });
    const data = await res.json();
    setAbDamageRules(prev => [...prev, data]);
    setNewAbDamage('');
    showSuccess('伤害阈值已添加');
  }, [selectedServerId, newAbDamage]);

  const delAbDamageRule = useCallback(async (id: number) => {
    await api(`/servers/${selectedServerId}/abnormal-damage-rules/${id}`, { method: 'DELETE' });
    setAbDamageRules(prev => prev.filter(r => r.id !== id));
    showSuccess('伤害阈值已删除');
  }, [selectedServerId]);

  const fetchAbDamageLogs = useCallback(async (playerName?: string) => {
    if (!selectedServerId) return;
    setAbDamageLogsLoading(true);
    const params = new URLSearchParams({ limit: '200' });
    if (playerName) params.set('player_name', playerName);
    try {
      const res = await api(`/servers/${selectedServerId}/abnormal-damage-logs?${params}`);
      const data = await res.json();
      setAbDamageLogs(data.data || []);
    } catch {}
    setAbDamageLogsLoading(false);
  }, [selectedServerId]);

  const handleAbDamageLogsSearch = useCallback(() => {
    fetchAbDamageLogs(abDamageLogsQuery || undefined);
  }, [fetchAbDamageLogs, abDamageLogsQuery]);

  const addAnnouncement = useCallback(async () => {
    if (!selectedServerId || !newAnn.content) return;
    const res = await api(`/servers/${selectedServerId}/announcements`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(newAnn) });
    const data = await res.json();
    setAnnouncements(prev => [...prev, data]);
    setNewAnn({ content: '', interval_minutes: 10 });
    showSuccess('通告已添加');
  }, [selectedServerId, newAnn]);

  const delAnnouncement = useCallback(async (id: number) => {
    await api(`/servers/${selectedServerId}/announcements/${id}`, { method: 'DELETE' });
    setAnnouncements(prev => prev.filter(a => a.id !== id));
    showSuccess('通告已删除');
  }, [selectedServerId]);

  const addAutoReply = useCallback(async () => {
    if (!selectedServerId || !newReply.keyword || !newReply.reply_message) return;
    const res = await api(`/servers/${selectedServerId}/auto-replies`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(newReply) });
    const data = await res.json();
    setAutoReplies(prev => [...prev, data]);
    setNewReply({ keyword: '', reply_message: '' });
    showSuccess('自动回复规则已添加');
  }, [selectedServerId, newReply]);

  const delAutoReply = useCallback(async (id: number) => {
    await api(`/servers/${selectedServerId}/auto-replies/${id}`, { method: 'DELETE' });
    setAutoReplies(prev => prev.filter(r => r.id !== id));
    showSuccess('自动回复规则已删除');
  }, [selectedServerId]);

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {servers.length > 0 && (
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
          <select className="rcon-input" style={{ width: 'auto', padding: '6px 10px' }} value={selectedServerId || ''} onChange={e => setSelectedServerId(parseInt(e.target.value))}>
            {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
          </select>
        </div>
      )}

      {successMsg && (
        <div style={{ padding: '8px 16px', background: 'rgba(34,197,94,0.12)', border: '1px solid rgba(34,197,94,0.3)', borderRadius: 'var(--radius)', color: '#22c55e', fontSize: 13, fontWeight: 500 }}>
          {successMsg}
        </div>
      )}

      {deleteConfirm && (
        <ConfirmModal
          title={deleteConfirm.title}
          message={deleteConfirm.message}
          confirmLabel="确认删除"
          danger={true}
          onConfirm={() => { deleteConfirm.onConfirm(); setDeleteConfirm(null); }}
          onCancel={() => setDeleteConfirm(null)}
        />
      )}

      <div className="card">
        <div className="tabs-header">
          {configTabs.map((tab) => (
            <button key={tab.id} className={`tab-btn${activeTab === tab.id ? ' active' : ''}`} onClick={() => setActiveTab(tab.id)}>
              {tab.label}
            </button>
          ))}
        </div>

        {activeTab === 'tab-1' && (
          <DamageFfNotifyTab
            selectedServerId={selectedServerId}
            tkLoading={tkLoading}
            damageNotifyLoading={damageNotifyLoading}
            tkForm={tkForm}
            damageNotifyForm={damageNotifyForm}
            tkError={tkError}
            damageNotifyError={damageNotifyError}
            tkSaving={tkSaving}
            damageNotifySaving={damageNotifySaving}
            onTkFormChange={setTkForm}
            onDamageNotifyFormChange={setDamageNotifyForm}
            onSaveTk={saveTkSettings}
            onSaveDamageNotify={saveDamageNotifySettings}
            onSaveAll={saveAllDamageFf}
          />
        )}

        {activeTab === 'tab-2' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 20 }}>挂机设置 (AFK)</h4>
            <div style={{ padding: '60px 0', textAlign: 'center', color: 'var(--text3)', fontSize: 14 }}>
              功能待开发
            </div>
          </div>
        )}
        {activeTab === 'tab-3' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 20 }}>代码跳边设置</h4>
            {!selectedServerId ? (
              <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
            ) : tsLoading ? (
              <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 24, maxWidth: 500 }}>
                <label style={{ display: 'flex', alignItems: 'flex-start', gap: 12, cursor: 'pointer' }}>
                  <input type="checkbox" checked={tsEnabled} onChange={e => setTsEnabled(e.target.checked)} style={{ marginTop: 2 }} />
                  <div>
                    <div style={{ fontWeight: 500 }}>开启代码跳边</div>
                    <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4, lineHeight: 1.6 }}>
                      开启后，玩家可在公屏发送 <code>tb&lt;tag&gt;</code> 或 <code>跳边&lt;tag&gt;</code> 发起跳边请求。<br />
                      对方队伍玩家可发送 <code>rl&lt;tag&gt;</code> 或 <code>认领&lt;tag&gt;</code> 认领该玩家。<br />
                      管理员发送 <code>ty&lt;tag&gt;</code> 或 <code>同意&lt;tag&gt;</code> 批准后自动执行跳边。<br />
                      每个阶段有效期为 60 秒，超时自动失效。
                    </div>
                  </div>
                </label>
                <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={saveTsConfig} disabled={tsSaving}>
                  {tsSaving ? '保存中...' : '保存设置'}
                </button>
              </div>
            )}
          </div>
        )}
        {activeTab === 'tab-4' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 20 }}>广播设置</h4>
            {!selectedServerId ? <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
            : bcLoading ? <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p> : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 24, maxWidth: 560 }}>
                {/* 进入提醒 */}
                <div style={{ borderBottom: '1px solid var(--border)', paddingBottom: 20 }}>
                  <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer', marginBottom: 12 }}>
                    <input type="checkbox" checked={bcForm.join_message_enabled} onChange={e => setBcForm({...bcForm, join_message_enabled: e.target.checked})} />
                    <div><div style={{ fontWeight: 500 }}>玩家进入提醒</div><div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>玩家加入服务器时广播欢迎消息</div></div>
                  </label>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>欢迎消息</label>
                  <input className="rcon-input" value={bcForm.join_message} onChange={e => setBcForm({...bcForm, join_message: e.target.value})} />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>{'{player}'} 会被替换为玩家名称</p>
                </div>

                {/* OP列表 */}
                <div style={{ borderBottom: '1px solid var(--border)', paddingBottom: 20 }}>
                  <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer', marginBottom: 12 }}>
                    <input type="checkbox" checked={bcForm.gameop_list_enabled} onChange={e => setBcForm({...bcForm, gameop_list_enabled: e.target.checked})} />
                    <div><div style={{ fontWeight: 500 }}>在线 OP 列表</div><div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>玩家呼唤 OP 时显示在线管理员列表</div></div>
                  </label>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>OP 列表消息格式</label>
                  <input className="rcon-input" value={bcForm.gameop_list_message} onChange={e => setBcForm({...bcForm, gameop_list_message: e.target.value})} />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>{'{oplist}'} 会被替换为在线 OP 名称列表</p>
                </div>

                {/* 定时通告（多条） */}
                <div style={{ borderBottom: '1px solid var(--border)', paddingBottom: 20 }}>
                  <div style={{ fontWeight: 500, marginBottom: 12 }}>定时通告列表</div>
                  {announcements.length === 0 && <p style={{ fontSize: 12, color: 'var(--text3)', marginBottom: 12 }}>暂无通告</p>}
                  {announcements.map(a => (
                    <div key={a.id} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
                      <span style={{ flex: 1, fontSize: 13 }}>{a.content}</span>
                      <span className="badge gray" style={{ fontSize: 10 }}>每{a.interval_minutes}{a.interval_minutes === 0 ? '(连续)' : '分钟'}</span>
                      <span className="badge red" style={{ cursor: 'pointer', fontSize: 10 }} onClick={() => setDeleteConfirm({ title: '删除通告', message: `确认删除此通告「${a.content}」？`, onConfirm: () => delAnnouncement(a.id) })}>删除</span>
                    </div>
                  ))}
                  <div style={{ display: 'flex', gap: 8, marginTop: 12 }}>
                    <input className="rcon-input" style={{ flex: 1 }} placeholder="通告内容" value={newAnn.content} onChange={e => setNewAnn({...newAnn, content: e.target.value})}
                      onKeyDown={e => e.key === 'Enter' && addAnnouncement()} />
                    <input className="rcon-input" type="number" min={0} max={120} style={{ width: 70 }} value={newAnn.interval_minutes}
                      onChange={e => setNewAnn({...newAnn, interval_minutes: parseInt(e.target.value) || 0})} title="间隔分钟" />
                    <button className="rcon-btn" style={{ width: 'auto', padding: '8px 14px', fontSize: 12 }} onClick={addAnnouncement}>添加</button>
                  </div>
                </div>

                {/* 自动回复（多条） */}
                <div>
                  <div style={{ fontWeight: 500, marginBottom: 12 }}>自动回复规则</div>
                  {autoReplies.length === 0 && <p style={{ fontSize: 12, color: 'var(--text3)', marginBottom: 12 }}>暂无自动回复规则</p>}
                  {autoReplies.map(r => (
                    <div key={r.id} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
                      <span className="badge blue" style={{ fontSize: 10 }}>{r.keyword}</span>
                      <span style={{ flex: 1, fontSize: 13 }}>{r.reply_message}</span>
                      <span className="badge red" style={{ cursor: 'pointer', fontSize: 10 }} onClick={() => setDeleteConfirm({ title: '删除自动回复', message: `确认删除自动回复规则「${r.keyword}」？`, onConfirm: () => delAutoReply(r.id) })}>删除</span>
                    </div>
                  ))}
                  <div style={{ display: 'flex', gap: 8, marginTop: 12 }}>
                    <input className="rcon-input" style={{ width: 120 }} placeholder="关键字" value={newReply.keyword} onChange={e => setNewReply({...newReply, keyword: e.target.value})}
                      onKeyDown={e => e.key === 'Enter' && addAutoReply()} />
                    <input className="rcon-input" style={{ flex: 1 }} placeholder="回复消息" value={newReply.reply_message} onChange={e => setNewReply({...newReply, reply_message: e.target.value})}
                      onKeyDown={e => e.key === 'Enter' && addAutoReply()} />
                    <button className="rcon-btn" style={{ width: 'auto', padding: '8px 14px', fontSize: 12 }} onClick={addAutoReply}>添加</button>
                  </div>
                </div>

                {bcError && <div style={{ color: 'var(--red)', fontSize: 12 }}>{bcError}</div>}
                <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={saveBroadcast} disabled={bcSaving}>
                  {bcSaving ? '保存中...' : '保存设置'}
                </button>
              </div>
            )}
          </div>
        )}
        {activeTab === 'tab-5' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 20 }}>队伍设置</h4>
            {!selectedServerId ? <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
            : teamLoading ? <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p> : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 24, maxWidth: 500 }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer' }}>
                  <input type="checkbox" checked={teamForm.create_team_broadcast} onChange={e => setTeamForm({...teamForm, create_team_broadcast: e.target.checked})} />
                  <div><div style={{ fontWeight: 500 }}>建队广播</div><div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>玩家创建队伍时向全服广播</div></div>
                </label>

                <div style={{ borderTop: '1px solid var(--border)', paddingTop: 20 }}>
                  <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer', marginBottom: 16 }}>
                    <input type="checkbox" checked={teamForm.captain_time_check} onChange={e => setTeamForm({...teamForm, captain_time_check: e.target.checked})} />
                    <div><div style={{ fontWeight: 500 }}>队长游戏时长检测</div><div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>检测队长是否满足最小游戏时长要求</div></div>
                  </label>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>队长最小游戏时长（分钟）</label>
                  <input className="rcon-input" type="number" min={1} max={9999} style={{ width: 100 }} value={teamForm.captain_min_playtime}
                    onChange={e => setTeamForm({...teamForm, captain_min_playtime: parseInt(e.target.value) || 0})} />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>队长游戏时长未达到此数值时不可建队</p>
                </div>

                <div>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>时长检查生效人数</label>
                  <input className="rcon-input" type="number" min={1} max={100} style={{ width: 100 }} value={teamForm.captain_check_min_players}
                    onChange={e => setTeamForm({...teamForm, captain_check_min_players: parseInt(e.target.value) || 0})} />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>服务器人数达到此数量后才开始检查队长时长</p>
                </div>

                <div>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>最大建队次数</label>
                  <input className="rcon-input" type="number" min={1} max={50} style={{ width: 100 }} value={teamForm.max_create_team_attempts}
                    onChange={e => setTeamForm({...teamForm, max_create_team_attempts: parseInt(e.target.value) || 0})} />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>玩家不满足时长要求重复建队超过此次数将被踢出</p>
                </div>

                {teamError && <div style={{ color: 'var(--red)', fontSize: 12 }}>{teamError}</div>}
                <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={saveTeamSettings} disabled={teamSaving}>
                  {teamSaving ? '保存中...' : '保存设置'}
                </button>
              </div>
            )}
          </div>
        )}
        {activeTab === 'tab-7' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 20 }}>异常伤害</h4>
            {!selectedServerId ? (
              <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
            ) : abDamageLoading ? (
              <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 24, maxWidth: 560 }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer' }}>
                  <input type="checkbox" checked={abDamageEnabled} onChange={e => setAbDamageEnabled(e.target.checked)} />
                  <div>
                    <div style={{ fontWeight: 500 }}>开启异常伤害检测</div>
                    <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>开启后，将监控玩家造成的伤害是否超过设定的最高值</div>
                  </div>
                </label>

                <div style={{ borderTop: '1px solid var(--border)', paddingTop: 20 }}>
                  <div style={{ fontWeight: 500, marginBottom: 12 }}>伤害最高值规则</div>
                  {abDamageRules.length === 0 && (
                    <p style={{ fontSize: 12, color: 'var(--text3)', marginBottom: 12 }}>暂无规则，请添加伤害最高值</p>
                  )}
                  {abDamageRules.map(r => (
                    <div key={r.id} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
                      <span style={{ flex: 1, fontSize: 13 }}>最高伤害值：<strong>{r.max_damage}</strong></span>
                      <span className="badge red" style={{ cursor: 'pointer', fontSize: 10 }} onClick={() => setDeleteConfirm({ title: '删除伤害阈值', message: `确认删除伤害阈值规则（最大值: ${r.max_damage}）？`, onConfirm: () => delAbDamageRule(r.id) })}>删除</span>
                    </div>
                  ))}
                  <div style={{ display: 'flex', gap: 8, marginTop: 12 }}>
                    <input className="rcon-input" type="number" min={1} style={{ flex: 1 }} placeholder="输入伤害最高值" value={newAbDamage}
                      onChange={e => setNewAbDamage(e.target.value)}
                      onKeyDown={e => e.key === 'Enter' && addAbDamageRule()} />
                    <button className="rcon-btn" style={{ width: 'auto', padding: '8px 14px', fontSize: 12 }} onClick={addAbDamageRule}>添加</button>
                  </div>
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>当玩家造成的单次伤害超过该值时，将被记录为异常伤害事件</p>
                </div>

                {abDamageError && <div style={{ color: 'var(--red)', fontSize: 12 }}>{abDamageError}</div>}

                <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={saveAbDamageConfig} disabled={abDamageSaving}>
                  {abDamageSaving ? '保存中...' : '保存设置'}
                </button>

                <div style={{ borderTop: '1px solid var(--border)', paddingTop: 24, marginTop: 8 }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
                    <h4 style={{ margin: 0, fontSize: 14 }}>异常伤害记录</h4>
                    <div style={{ display: 'flex', gap: 8 }}>
                      <input className="rcon-input" style={{ width: 180, fontSize: 12 }} placeholder="搜索玩家名称..."
                        value={abDamageLogsQuery} onChange={e => setAbDamageLogsQuery(e.target.value)}
                        onKeyDown={e => e.key === 'Enter' && handleAbDamageLogsSearch()} />
                      <button className="rcon-btn" style={{ width: 'auto', padding: '6px 12px', fontSize: 12 }} onClick={handleAbDamageLogsSearch}>搜索</button>
                      <span className="badge gray" style={{ cursor: 'pointer' }} onClick={() => { setAbDamageLogsQuery(''); fetchAbDamageLogs(); }}>全部</span>
                    </div>
                  </div>

                  {abDamageLogsLoading ? (
                    <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p>
                  ) : abDamageLogs.length === 0 ? (
                    <div style={{ padding: '32px 0', textAlign: 'center', color: 'var(--text3)', fontSize: 13 }}>
                      暂无异常伤害记录
                    </div>
                  ) : (
                    <div style={{ overflowX: 'auto' }}>
                      <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 12 }}>
                        <thead>
                          <tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
                            <th style={{ padding: '8px 6px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap' }}>时间</th>
                            <th style={{ padding: '8px 6px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap' }}>攻击者</th>
                            <th style={{ padding: '8px 6px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap' }}>SteamID64</th>
                            <th style={{ padding: '8px 6px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap' }}>阵营</th>
                            <th style={{ padding: '8px 6px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap' }}>武器</th>
                            <th style={{ padding: '8px 6px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap' }}>伤害值</th>
                            <th style={{ padding: '8px 6px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap' }}>被伤害者</th>
                            <th style={{ padding: '8px 6px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap' }}>对方阵营</th>
                          </tr>
                        </thead>
                        <tbody>
                          {abDamageLogs.map(log => (
                            <tr key={log.id} style={{ borderBottom: '1px solid var(--border)' }}>
                              <td style={{ padding: '6px', whiteSpace: 'nowrap', fontSize: 11 }}>
                                {new Date(log.logged_at).toLocaleString()}
                              </td>
                              <td style={{ padding: '6px', fontWeight: 500 }}>{log.player_name}</td>
                              <td style={{ padding: '6px', fontFamily: 'monospace', fontSize: 11, color: 'var(--text2)' }}>{log.player_steamid64}</td>
                              <td style={{ padding: '6px', fontSize: 11 }}>{log.attacker_faction || '-'}</td>
                              <td style={{ padding: '6px' }}><span className="badge blue" style={{ fontSize: 10 }}>{log.weapon}</span></td>
                              <td style={{ padding: '6px', color: 'var(--red)', fontWeight: 600 }}>{log.damage}</td>
                              <td style={{ padding: '6px', fontWeight: 500 }}>{log.victim_name}</td>
                              <td style={{ padding: '6px', fontSize: 11 }}>{log.victim_faction || '-'}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
