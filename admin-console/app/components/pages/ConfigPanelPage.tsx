'use client';

import { useState, useEffect, useCallback } from 'react';

const API_BASE = '/api/v1';

const configTabs = [
  { id: 'tab-1', label: '快捷设置' },
  { id: 'tab-2', label: '误杀设置' },
  { id: 'tab-3', label: '挂机设置' },
  { id: 'tab-4', label: '跳边设置' },
  { id: 'tab-5', label: '广播设置' },
  { id: 'tab-6', label: '队伍设置' },
  { id: 'tab-7', label: '悬赏设置' },
  { id: 'tab-8', label: '暖服设置' },
  { id: 'tab-9', label: '伤害通知' },
  { id: 'tab-10', label: '异常伤害' },
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
  notification_message: string | null;
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
  const [tkForm, setTkForm] = useState({ enabled: false, max_team_kills: 3, apology_time_minutes: 5, notification_message: '' });
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
  // Seed settings state
  const [seedLoading, setSeedLoading] = useState(false);
  const [seedSaving, setSeedSaving] = useState(false);
  const [seedError, setSeedError] = useState('');
  const [seedForm, setSeedForm] = useState({ enabled: false, player_threshold: 20, vehicle_claim: true, vehicle_fill: true, deploy_restrict: false, kit_restrict: false, heavy_vehicle_require: false, respawn_timer: true, use_enemy_vehicle: false });
  // Damage notify settings state
  const [damageNotifyLoading, setDamageNotifyLoading] = useState(false);
  const [damageNotifySaving, setDamageNotifySaving] = useState(false);
  const [damageNotifyError, setDamageNotifyError] = useState('');
  const [damageNotifyForm, setDamageNotifyForm] = useState({ enabled: false, keyword: '!damage' });
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
  const [successMsg, setSuccessMsg] = useState('');

  const showSuccess = (msg: string) => { setSuccessMsg(msg); setTimeout(() => setSuccessMsg(''), 3000); };

  useEffect(() => {
    fetch(`${API_BASE}/servers`)
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
    fetch(`${API_BASE}/servers/${selectedServerId}/tk-settings`)
      .then(r => r.json())
      .then(data => {
        setTkSettings(data);
        setTkForm({ enabled: data.enabled, max_team_kills: data.max_team_kills, apology_time_minutes: data.apology_time_minutes, notification_message: data.notification_message || '' });
        setTkLoading(false);
      })
      .catch(() => setTkLoading(false));
    // 加载 AFK 设置
    setAfkLoading(true);
    fetch(`${API_BASE}/servers/${selectedServerId}/afk-settings`)
      .then(r => r.json())
      .then(data => { setAfkForm({ enabled: data.enabled, min_players_to_check: data.min_players_to_check, max_afk_minutes: data.max_afk_minutes }); setAfkLoading(false); })
      .catch(() => setAfkLoading(false));
    // 加载广播设置
    setBcLoading(true);
    fetch(`${API_BASE}/servers/${selectedServerId}/broadcast-settings`)
      .then(r => r.json())
      .then(data => { setBcForm({ join_message_enabled: data.join_message_enabled, join_message: data.join_message, gameop_list_enabled: data.gameop_list_enabled, gameop_list_message: data.gameop_list_message, announcement_enabled: data.announcement_enabled, announcement_content: data.announcement_content || '', announcement_interval: data.announcement_interval }); setBcLoading(false); })
      .catch(() => setBcLoading(false));
    fetch(`${API_BASE}/servers/${selectedServerId}/announcements`).then(r => r.json()).then(d => setAnnouncements(d.data || [])).catch(() => {});
    fetch(`${API_BASE}/servers/${selectedServerId}/auto-replies`).then(r => r.json()).then(d => setAutoReplies(d.data || [])).catch(() => {});
    // 加载队伍设置
    setTeamLoading(true);
    fetch(`${API_BASE}/servers/${selectedServerId}/team-settings`).then(r => r.json())
      .then(d => { setTeamForm({ create_team_broadcast: d.create_team_broadcast, captain_time_check: d.captain_time_check, captain_min_playtime: d.captain_min_playtime, captain_check_min_players: d.captain_check_min_players, max_create_team_attempts: d.max_create_team_attempts }); setTeamLoading(false); })
      .catch(() => setTeamLoading(false));
    // 加载暖服设置
    setSeedLoading(true);
    fetch(`${API_BASE}/servers/${selectedServerId}/seed-settings`).then(r => r.json())
      .then(d => { const f: Record<string, boolean | number> = { enabled: d.enabled, player_threshold: d.player_threshold, vehicle_claim: d.vehicle_claim, vehicle_fill: d.vehicle_fill, deploy_restrict: d.deploy_restrict, kit_restrict: d.kit_restrict, heavy_vehicle_require: d.heavy_vehicle_require, respawn_timer: d.respawn_timer, use_enemy_vehicle: d.use_enemy_vehicle }; setSeedForm(f as any); setSeedLoading(false); })
      .catch(() => setSeedLoading(false));
    // 加载伤害通知设置
    setDamageNotifyLoading(true);
    fetch(`${API_BASE}/servers/${selectedServerId}/damage-notify-settings`).then(r => r.json())
      .then(d => { setDamageNotifyForm({ enabled: d.enabled, keyword: d.keyword || '!damage' }); setDamageNotifyLoading(false); })
      .catch(() => setDamageNotifyLoading(false));
    // 加载异常伤害设置
    setAbDamageLoading(true);
    fetch(`${API_BASE}/servers/${selectedServerId}/abnormal-damage-config`).then(r => r.json())
      .then(d => { setAbDamageEnabled(d.enabled); setAbDamageLoading(false); })
      .catch(() => setAbDamageLoading(false));
    fetch(`${API_BASE}/servers/${selectedServerId}/abnormal-damage-rules`).then(r => r.json())
      .then(d => setAbDamageRules(d.data || []))
      .catch(() => {});
    // 加载异常伤害日志
    fetchAbDamageLogs();
  }, [selectedServerId]);

  const saveTkSettings = useCallback(async () => {
    if (!selectedServerId) return;
    setTkSaving(true);
    setTkError('');
    const res = await fetch(`${API_BASE}/servers/${selectedServerId}/tk-settings`, {
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
    const res = await fetch(`${API_BASE}/servers/${selectedServerId}/afk-settings`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(afkForm) });
    const data = await res.json(); setAfkSaving(false);
    if (data.error) { setAfkError(data.error); } else { showSuccess('挂机设置已保存'); }
  }, [selectedServerId, afkForm]);

  const saveBroadcast = useCallback(async () => {
    if (!selectedServerId) return; setBcSaving(true); setBcError('');
    const res = await fetch(`${API_BASE}/servers/${selectedServerId}/broadcast-settings`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(bcForm) });
    const data = await res.json(); setBcSaving(false);
    if (data.error) { setBcError(data.error); } else { showSuccess('广播设置已保存'); }
  }, [selectedServerId, bcForm]);

  const saveTeamSettings = useCallback(async () => {
    if (!selectedServerId) return; setTeamSaving(true); setTeamError('');
    const res = await fetch(`${API_BASE}/servers/${selectedServerId}/team-settings`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(teamForm) });
    const data = await res.json(); setTeamSaving(false);
    if (data.error) setTeamError(data.error); else showSuccess('队伍设置已保存');
  }, [selectedServerId, teamForm]);

  const saveSeedSettings = useCallback(async () => {
    if (!selectedServerId) return; setSeedSaving(true); setSeedError('');
    const res = await fetch(`${API_BASE}/servers/${selectedServerId}/seed-settings`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(seedForm) });
    const data = await res.json(); setSeedSaving(false);
    if (data.error) setSeedError(data.error); else showSuccess('暖服设置已保存');
  }, [selectedServerId, seedForm]);

  const saveDamageNotifySettings = useCallback(async () => {
    if (!selectedServerId) return; setDamageNotifySaving(true); setDamageNotifyError('');
    const res = await fetch(`${API_BASE}/servers/${selectedServerId}/damage-notify-settings`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(damageNotifyForm) });
    const data = await res.json(); setDamageNotifySaving(false);
    if (data.error) setDamageNotifyError(data.error); else showSuccess('伤害通知设置已保存');
  }, [selectedServerId, damageNotifyForm]);

  const saveAbDamageConfig = useCallback(async () => {
    if (!selectedServerId) return; setAbDamageSaving(true); setAbDamageError('');
    const res = await fetch(`${API_BASE}/servers/${selectedServerId}/abnormal-damage-config`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ enabled: abDamageEnabled }) });
    const data = await res.json(); setAbDamageSaving(false);
    if (data.error) setAbDamageError(data.error); else showSuccess('异常伤害设置已保存');
  }, [selectedServerId, abDamageEnabled]);

  const addAbDamageRule = useCallback(async () => {
    if (!selectedServerId || !newAbDamage) return;
    const v = parseInt(newAbDamage);
    if (!v || v <= 0) return;
    const res = await fetch(`${API_BASE}/servers/${selectedServerId}/abnormal-damage-rules`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ max_damage: v }) });
    const data = await res.json();
    setAbDamageRules(prev => [...prev, data]);
    setNewAbDamage('');
    showSuccess('伤害阈值已添加');
  }, [selectedServerId, newAbDamage]);

  const delAbDamageRule = useCallback(async (id: number) => {
    await fetch(`${API_BASE}/servers/${selectedServerId}/abnormal-damage-rules/${id}`, { method: 'DELETE' });
    setAbDamageRules(prev => prev.filter(r => r.id !== id));
    showSuccess('伤害阈值已删除');
  }, [selectedServerId]);

  const fetchAbDamageLogs = useCallback(async (playerName?: string) => {
    if (!selectedServerId) return;
    setAbDamageLogsLoading(true);
    const params = new URLSearchParams({ limit: '200' });
    if (playerName) params.set('player_name', playerName);
    try {
      const res = await fetch(`${API_BASE}/servers/${selectedServerId}/abnormal-damage-logs?${params}`);
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
    const res = await fetch(`${API_BASE}/servers/${selectedServerId}/announcements`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(newAnn) });
    const data = await res.json();
    setAnnouncements(prev => [...prev, data]);
    setNewAnn({ content: '', interval_minutes: 10 });
    showSuccess('通告已添加');
  }, [selectedServerId, newAnn]);

  const delAnnouncement = useCallback(async (id: number) => {
    await fetch(`${API_BASE}/servers/${selectedServerId}/announcements/${id}`, { method: 'DELETE' });
    setAnnouncements(prev => prev.filter(a => a.id !== id));
    showSuccess('通告已删除');
  }, [selectedServerId]);

  const addAutoReply = useCallback(async () => {
    if (!selectedServerId || !newReply.keyword || !newReply.reply_message) return;
    const res = await fetch(`${API_BASE}/servers/${selectedServerId}/auto-replies`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(newReply) });
    const data = await res.json();
    setAutoReplies(prev => [...prev, data]);
    setNewReply({ keyword: '', reply_message: '' });
    showSuccess('自动回复规则已添加');
  }, [selectedServerId, newReply]);

  const delAutoReply = useCallback(async (id: number) => {
    await fetch(`${API_BASE}/servers/${selectedServerId}/auto-replies/${id}`, { method: 'DELETE' });
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

      <div className="card">
        <div className="tabs-header">
          {configTabs.map((tab) => (
            <button key={tab.id} className={`tab-btn${activeTab === tab.id ? ' active' : ''}`} onClick={() => setActiveTab(tab.id)}>
              {tab.label}
            </button>
          ))}
        </div>

        {activeTab === 'tab-1' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 16 }}>快捷设置</h4>
            <p style={{ color: 'var(--text3)', fontSize: 12 }}>此处可快速开关常用的插件和服务器基础功能。</p>
            <div style={{ marginTop: 20, display: 'flex', flexDirection: 'column', gap: 16 }}>
              <label style={{ display: 'flex', alignItems: 'center', gap: 10 }}><input type="checkbox" defaultChecked /> 启用服务器密码保护</label>
              <label style={{ display: 'flex', alignItems: 'center', gap: 10 }}><input type="checkbox" defaultChecked /> 开启反作弊模块</label>
              <label style={{ display: 'flex', alignItems: 'center', gap: 10 }}><input type="checkbox" /> 允许全员语音跨队</label>
            </div>
          </div>
        )}

        {activeTab === 'tab-2' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 20 }}>误杀设置 (TK)</h4>
            {!selectedServerId ? (
              <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
            ) : tkLoading ? (
              <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 20, maxWidth: 500 }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer' }}>
                  <input type="checkbox" checked={tkForm.enabled} onChange={e => setTkForm({...tkForm, enabled: e.target.checked})} />
                  <div>
                    <div style={{ fontWeight: 500 }}>开启误杀检测</div>
                    <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>开启后将监控玩家误杀队友行为</div>
                  </div>
                </label>

                <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>单局最大误杀人数</label>
                  <input className="rcon-input" type="number" min={1} max={20} style={{ width: 100 }} value={tkForm.max_team_kills}
                    onChange={e => setTkForm({...tkForm, max_team_kills: parseInt(e.target.value) || 0})} />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>超过此数值后玩家将被踢出服务器</p>
                </div>

                <div>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>道歉时间（分钟）</label>
                  <input className="rcon-input" type="number" min={1} max={60} style={{ width: 100 }} value={tkForm.apology_time_minutes}
                    onChange={e => setTkForm({...tkForm, apology_time_minutes: parseInt(e.target.value) || 0})} />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>超过此时间未道歉则被踢出服务器</p>
                </div>

                <div>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>自定义通知消息</label>
                  <textarea className="rcon-input" rows={3} style={{ resize: 'vertical' }} value={tkForm.notification_message}
                    onChange={e => setTkForm({...tkForm, notification_message: e.target.value})}
                    placeholder="误杀队友将被踢出服务器，请在 {time} 分钟内输入 !sorry 道歉。" />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>{'{time}'} 会被自动替换为实际道歉时间</p>
                </div>

                {tkError && <div style={{ color: 'var(--red)', fontSize: 12 }}>{tkError}</div>}

                <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={saveTkSettings} disabled={tkSaving}>
                  {tkSaving ? '保存中...' : '保存设置'}
                </button>
              </div>
            )}
          </div>
        )}

        {activeTab === 'tab-3' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 20 }}>挂机设置 (AFK)</h4>
            {!selectedServerId ? <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
            : afkLoading ? <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p> : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 20, maxWidth: 500 }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer' }}>
                  <input type="checkbox" checked={afkForm.enabled} onChange={e => setAfkForm({...afkForm, enabled: e.target.checked})} />
                  <div><div style={{ fontWeight: 500 }}>开启挂机检测</div><div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>开启后，当服务器人数达到指定数量时启动检测</div></div>
                </label>
                <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>启动检测人数</label>
                  <input className="rcon-input" type="number" min={1} max={100} style={{ width: 100 }} value={afkForm.min_players_to_check}
                    onChange={e => setAfkForm({...afkForm, min_players_to_check: parseInt(e.target.value) || 0})} />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>服务器人数达到此数量后开始挂机检测</p>
                </div>
                <div>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>最大挂机时长（分钟）</label>
                  <input className="rcon-input" type="number" min={1} max={120} style={{ width: 100 }} value={afkForm.max_afk_minutes}
                    onChange={e => setAfkForm({...afkForm, max_afk_minutes: parseInt(e.target.value) || 0})} />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>玩家持续挂机超过此时长将被踢出服务器</p>
                </div>
                {afkError && <div style={{ color: 'var(--red)', fontSize: 12 }}>{afkError}</div>}
                <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={saveAfkSettings} disabled={afkSaving}>
                  {afkSaving ? '保存中...' : '保存设置'}
                </button>
              </div>
            )}
          </div>
        )}
        {activeTab === 'tab-4' && <div className="tab-content" style={{ display: 'block' }}><h4>跳边设置</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>限制玩家频繁更换阵营。（功能UI待实现）</p></div>}
        {activeTab === 'tab-5' && (
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
                      <span className="badge red" style={{ cursor: 'pointer', fontSize: 10 }} onClick={() => delAnnouncement(a.id)}>删除</span>
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
                      <span className="badge red" style={{ cursor: 'pointer', fontSize: 10 }} onClick={() => delAutoReply(r.id)}>删除</span>
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
        {activeTab === 'tab-6' && (
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
        {activeTab === 'tab-7' && <div className="tab-content" style={{ display: 'block' }}><h4>悬赏设置</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>击杀连杀玩家的赏金系统设置。（功能UI待实现）</p></div>}
        {activeTab === 'tab-8' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 20 }}>暖服设置</h4>
            {!selectedServerId ? <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
            : seedLoading ? <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p> : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 24, maxWidth: 560 }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer' }}>
                  <input type="checkbox" checked={seedForm.enabled} onChange={e => setSeedForm({...seedForm, enabled: e.target.checked})} />
                  <div><div style={{ fontWeight: 500 }}>开启暖服功能</div><div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>在线人数低于阈值时自动触发下方暖服规则</div></div>
                </label>
                <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>暖服人数阈值</label>
                  <input className="rcon-input" type="number" min={1} max={100} style={{ width: 100 }} value={seedForm.player_threshold}
                    onChange={e => setSeedForm({...seedForm, player_threshold: parseInt(e.target.value) || 0})} />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>在线人数小于此值时触发暖服</p>
                </div>

                <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
                  <div style={{ fontWeight: 500, marginBottom: 14 }}>暖服规则</div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                    {[
                      { k: 'vehicle_claim', label: '载具认领权限', desc: '开启后允许玩家认领载具' },
                      { k: 'vehicle_fill', label: '载具刷新位置填满', desc: '始终填满所有载具刷新位置' },
                      { k: 'deploy_restrict', label: '部署要求限制', desc: '限制沙袋、机枪、前哨等部署条件' },
                      { k: 'kit_restrict', label: '兵种人数限制', desc: '对工兵、医生、迫击炮等职业数量限制' },
                      { k: 'heavy_vehicle_require', label: '坦克飞机载具要求', desc: '限制重型载具所需套件/人员要求' },
                      { k: 'respawn_timer', label: '复活时间', desc: '开启=启用计时/更慢复活' },
                      { k: 'use_enemy_vehicle', label: '使用敌方载具', desc: '允许或禁止使用敌方载具' },
                    ].map(({ k, label, desc }) => (
                      <label key={k} style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer' }}>
                        <input type="checkbox" checked={!!(seedForm as any)[k]} onChange={e => setSeedForm({...seedForm, [k]: e.target.checked})} />
                        <div><div style={{ fontSize: 13 }}>{label}</div><div style={{ fontSize: 11, color: 'var(--text3)' }}>{desc}</div></div>
                      </label>
                    ))}
                  </div>
                </div>

                {seedError && <div style={{ color: 'var(--red)', fontSize: 12 }}>{seedError}</div>}
                <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={saveSeedSettings} disabled={seedSaving}>
                  {seedSaving ? '保存中...' : '保存设置'}
                </button>
              </div>
            )}
          </div>
        )}
        {activeTab === 'tab-9' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 20 }}>伤害通知</h4>
            {!selectedServerId ? (
              <p style={{ color: 'var(--text3)', fontSize: 12 }}>请先添加游戏服务器。</p>
            ) : damageNotifyLoading ? (
              <p style={{ color: 'var(--text3)', fontSize: 12 }}>加载中...</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 24, maxWidth: 500 }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: 12, cursor: 'pointer' }}>
                  <input type="checkbox" checked={damageNotifyForm.enabled} onChange={e => setDamageNotifyForm({...damageNotifyForm, enabled: e.target.checked})} />
                  <div>
                    <div style={{ fontWeight: 500 }}>开启伤害通知功能</div>
                    <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>开启后，玩家可通过关键字查询 HUD 伤害通知</div>
                  </div>
                </label>

                <div style={{ borderTop: '1px solid var(--border)', paddingTop: 16 }}>
                  <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>玩家触发关键字</label>
                  <input className="rcon-input" style={{ width: 200 }} value={damageNotifyForm.keyword}
                    onChange={e => setDamageNotifyForm({...damageNotifyForm, keyword: e.target.value})}
                    placeholder="!damage" />
                  <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>
                    玩家在聊天中发送此关键字即可开启伤害通知。{!damageNotifyForm.enabled && <span style={{ color: 'var(--red)' }}>当前功能已关闭，关键字不会生效。</span>}
                  </p>
                </div>

                {damageNotifyError && <div style={{ color: 'var(--red)', fontSize: 12 }}>{damageNotifyError}</div>}

                <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px' }} onClick={saveDamageNotifySettings} disabled={damageNotifySaving}>
                  {damageNotifySaving ? '保存中...' : '保存设置'}
                </button>
              </div>
            )}
          </div>
        )}
        {activeTab === 'tab-10' && (
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
                      <span className="badge red" style={{ cursor: 'pointer', fontSize: 10 }} onClick={() => delAbDamageRule(r.id)}>删除</span>
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
