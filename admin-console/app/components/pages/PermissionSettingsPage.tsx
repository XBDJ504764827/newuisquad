'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';

interface PermGroup { id: number; server_id: number; group_name: string; permissions: string; created_at: string; updated_at: string; }
interface PermAdmin { id: number; server_id: number; steam_id: string; group_name: string; player_name: string; created_at: string; }

const ALL_PERMISSIONS: [string, string][] = [
  ['reserve', 'reserve (预留通道)'], ['balance', 'balance (平衡控制)'], ['canseeadminchat', 'canseeadminchat (查看管理聊天)'],
  ['manageserver', 'manageserver (服务器管理)'], ['teamchange', 'teamchange (换队权限)'], ['chat', 'chat (聊天控制)'],
  ['cameraman', 'cameraman (旁观/录像)'], ['kick', 'kick (踢出玩家)'], ['ban', 'ban (封禁玩家)'],
  ['forceteamchange', 'forceteamchange (强制换边)'], ['immune', 'immune (免疫踢封)'], ['changemap', 'changemap (更换地图)'],
  ['pause', 'pause (暂停比赛)'], ['cheat', 'cheat (作弊指令)'], ['private', 'private (私密设置)'],
  ['config', 'config (配置修改)'], ['featuretest', 'featuretest (功能测试)'], ['demos', 'demos (录像管理)'],
  ['disbandSquad', 'disbandSquad (解散小队)'], ['removeFromSquad', 'removeFromSquad (从小队移除)'],
  ['demoteCommander', 'demoteCommander (降职指挥官)'], ['debug', 'debug (调试模式)'],
];

export default function PermissionSettingsPage() {
  const { servers } = useServers();
  const [serverId, setServerId] = useState<number | null>(null);
  const [msg, setMsg] = useState('');

  const [groups, setGroups] = useState<PermGroup[]>([]);
  const [gLoading, setGLoading] = useState(false);
  const [admins, setAdmins] = useState<PermAdmin[]>([]);
  const [aLoading, setALoading] = useState(false);

  // Group modal
  const [showGroupModal, setShowGroupModal] = useState(false);
  const [editingGid, setEditingGid] = useState<number | null>(null);
  const [gName, setGName] = useState('');
  const [gPerms, setGPerms] = useState<Set<string>>(new Set());
  const [gSaving, setGSaving] = useState(false);

  // Admin modal
  const [showAdminModal, setShowAdminModal] = useState(false);
  const [editingAid, setEditingAid] = useState<number | null>(null);
  const [aSteamId, setASteamId] = useState('');
  const [aGroupName, setAGroupName] = useState('');
  const [aName, setAName] = useState('');
  const [aLookupName, setALookupName] = useState('');
  const [aLooking, setALooking] = useState(false);
  const [aSaving, setASaving] = useState(false);

  const groupOverlayRef = useRef<HTMLDivElement>(null);
  const adminOverlayRef = useRef<HTMLDivElement>(null);

  useEffect(() => { if (servers.length > 0 && !serverId) setServerId(servers[0].id); }, [servers, serverId]);
  const showMsg = (text: string) => { setMsg(text); setTimeout(() => setMsg(''), 3000); };

  const fetchGroups = useCallback(() => {
    if (!serverId) return; setGLoading(true);
    api(`/servers/${serverId}/permission-groups`).then(r => r.json())
      .then(d => { setGroups(d.data || []); setGLoading(false); }).catch(() => setGLoading(false));
  }, [serverId]);
  const fetchAdmins = useCallback(() => {
    if (!serverId) return; setALoading(true);
    api(`/servers/${serverId}/permission-admins`).then(r => r.json())
      .then(d => { setAdmins(d.data || []); setALoading(false); }).catch(() => setALoading(false));
  }, [serverId]);
  useEffect(() => { fetchGroups(); fetchAdmins(); }, [fetchGroups, fetchAdmins]);

  // ── Group modal ──
  const openGroupModal = (g?: PermGroup) => {
    if (g) { setEditingGid(g.id); setGName(g.group_name); setGPerms(new Set(g.permissions.split(',').filter(Boolean))); }
    else { setEditingGid(null); setGName(''); setGPerms(new Set()); }
    setShowGroupModal(true);
  };
  const closeGroupModal = () => setShowGroupModal(false);
  const toggleGPerm = (p: string) => setGPerms(prev => { const s = new Set(prev); s.has(p) ? s.delete(p) : s.add(p); return s; });
  const saveGroup = () => {
    if (!serverId || !gName) return;
    setGSaving(true);
    const permsStr = Array.from(gPerms).join(',');
    const req = editingGid
      ? api(`/servers/${serverId}/permission-groups/${editingGid}`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ group_name: gName, permissions: permsStr }) })
      : api(`/servers/${serverId}/permission-groups`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ group_name: gName, permissions: permsStr }) });
    req.then(r => r.json()).then(d => { setGSaving(false); if (d.error) showMsg(d.error); else { fetchGroups(); closeGroupModal(); showMsg(editingGid ? '权限组已更新' : '权限组已创建'); } });
  };
  const deleteGroup = (gid: number) => {
    if (!serverId) return;
    api(`/servers/${serverId}/permission-groups/${gid}`, { method: 'DELETE' }).then(() => { fetchGroups(); showMsg('权限组已删除'); });
  };

  // ── Admin modal ──
  const openAdminModal = (a?: PermAdmin) => {
    if (a) { setEditingAid(a.id); setASteamId(a.steam_id); setAGroupName(a.group_name); setAName(a.player_name); setALookupName(''); }
    else { setEditingAid(null); setASteamId(''); setAGroupName(''); setAName(''); setALookupName(''); }
    setShowAdminModal(true);
  };
  const closeAdminModal = () => setShowAdminModal(false);
  const handleSteamLookup = () => {
    if (!aSteamId || aSteamId.length < 10) return;
    setALooking(true); setALookupName('');
    api(`/steam-player/${aSteamId}`).then(r => r.json())
      .then(d => { setALookupName(d.player_name || ''); setAName(d.player_name || ''); setALooking(false); }).catch(() => setALooking(false));
  };
  const saveAdmin = () => {
    if (!serverId || !aSteamId || !aGroupName) return;
    setASaving(true);
    const req = editingAid
      ? api(`/servers/${serverId}/permission-admins/${editingAid}`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ group_name: aGroupName, player_name: aName || aLookupName }) })
      : api(`/servers/${serverId}/permission-admins`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ steam_id: aSteamId, group_name: aGroupName, player_name: aName || aLookupName }) });
    req.then(r => r.json()).then(d => { setASaving(false); if (d.error) showMsg(d.error); else { fetchAdmins(); closeAdminModal(); showMsg(editingAid ? '管理员已更新' : '管理员已添加'); } });
  };
  const deleteAdmin = (aid: number) => {
    if (!serverId) return;
    api(`/servers/${serverId}/permission-admins/${aid}`, { method: 'DELETE' }).then(() => { fetchAdmins(); showMsg('管理员已删除'); });
  };

  const cfgBaseUrl = typeof window !== 'undefined' ? `${window.location.origin}/api/v1/servers` : '';

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {servers.length > 0 && (
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
          <select className="form-select" style={{ width: 'auto', padding: '6px 10px' }} value={serverId || ''}
            onChange={e => setServerId(parseInt(e.target.value))}>
            {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
          </select>
        </div>
      )}

      {msg && (
        <div style={{ padding: '8px 16px', background: 'rgba(34,197,94,0.12)', border: '1px solid rgba(34,197,94,0.3)', borderRadius: 'var(--radius)', color: '#22c55e', fontSize: 13, fontWeight: 500 }}>{msg}</div>
      )}

      <div className="card">
        <div className="card-header">
          <div><div className="card-title">游戏管理员列表</div><div className="card-sub">为指定玩家分配服务器内指令的操作权限</div></div>
          <div style={{ display: 'flex', gap: 10 }}>
            <button className="btn btn-outline" onClick={() => openGroupModal()}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              创建权限组
            </button>
            <button className="btn btn-primary" onClick={() => openAdminModal()}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              添加管理员
            </button>
          </div>
        </div>
        <div style={{ overflowX: 'auto' }}>
          {aLoading ? <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
          : admins.length === 0 ? <div className="empty-state"><h3>暂无管理员</h3><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 4 }}>点击"添加管理员"为玩家分配服务器权限</p></div>
          : <table>
            <thead>
              <tr><th>添加时间</th><th>SteamID64</th><th>身份标识/名称</th><th>拥有权限/权限组</th><th>操作</th></tr>
            </thead>
            <tbody>{admins.map(a => {
              const group = groups.find(g => g.group_name === a.group_name);
              const permLabels = group ? group.permissions.split(',').filter(Boolean).slice(0, 4).join(', ') : '';
              return (
                <tr key={a.id}>
                  <td style={{ fontSize: 12 }}>{a.created_at ? new Date(a.created_at).toISOString().slice(0, 10) : '-'}</td>
                  <td style={{ fontFamily: 'monospace', fontSize: 12, color: 'var(--text2)' }}>{a.steam_id}</td>
                  <td style={{ fontWeight: 500 }}>{a.player_name || '-'}</td>
                  <td><span className="badge blue">组: {a.group_name}</span>{permLabels ? <span style={{ fontSize: 12, color: 'var(--text3)', marginLeft: 6 }}>({permLabels})</span> : ''}</td>
                  <td>
                    <div style={{ display: 'flex', gap: 6 }}>
                      <button className="btn btn-outline" style={{ padding: '4px 10px', fontSize: 11 }} onClick={() => openAdminModal(a)}>编辑</button>
                      <button className="btn btn-outline" style={{ padding: '4px 10px', fontSize: 11, color: 'var(--red)', borderColor: 'rgba(239,68,68,0.3)' }} onClick={() => deleteAdmin(a.id)}>删除</button>
                    </div>
                  </td>
                </tr>
              );
            })}</tbody>
          </table>}
        </div>
      </div>

      {/* 权限组列表 */}
      <div className="card">
        <div className="card-header">
          <div><div className="card-title">权限组列表</div><div className="card-sub">已创建的权限组及其包含的权限（共 {groups.length} 组）</div></div>
          <button className="btn btn-outline" style={{ padding: '6px 12px' }} onClick={() => openGroupModal()}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
            新建
          </button>
        </div>
        <div style={{ overflowX: 'auto' }}>
          {gLoading ? <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
          : groups.length === 0 ? <div className="empty-state"><h3>暂无权限组</h3><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 4 }}>创建权限组来定义一组权限，方便批量分配给管理员</p></div>
          : <table>
            <thead>
              <tr><th>组名</th><th>权限数量</th><th>包含的权限</th><th>操作</th></tr>
            </thead>
            <tbody>{groups.map(g => {
              const perms = g.permissions.split(',').filter(Boolean);
              return (
                <tr key={g.id}>
                  <td><span className="badge blue" style={{ fontSize: 13 }}>{g.group_name}</span></td>
                  <td style={{ color: 'var(--text2)' }}>{perms.length} 项</td>
                  <td style={{ fontSize: 12, color: 'var(--text3)', maxWidth: 400, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                    {perms.slice(0, 6).join(', ')}{perms.length > 6 ? ` ...等${perms.length}项` : ''}
                  </td>
                  <td>
                    <div style={{ display: 'flex', gap: 6 }}>
                      <button className="btn btn-outline" style={{ padding: '4px 10px', fontSize: 11 }} onClick={() => openGroupModal(g)}>编辑</button>
                      <button className="btn btn-outline" style={{ padding: '4px 10px', fontSize: 11, color: 'var(--red)', borderColor: 'rgba(239,68,68,0.3)' }} onClick={() => deleteGroup(g.id)}>删除</button>
                    </div>
                  </td>
                </tr>
              );
            })}</tbody>
          </table>}
        </div>
      </div>

      <div className="card">
        <div className="card-header"><div><div className="card-title">Admins.cfg 下载地址</div><div className="card-sub">填入 RemoteAdminListHosts.cfg 使游戏服务器自动同步</div></div></div>
        <div className="card-body">
          {serverId && <div className="cfg-url-box"><code>{cfgBaseUrl}/{serverId}/Admins.cfg</code></div>}
          <p style={{ fontSize: 12, color: 'var(--text3)', marginTop: 8 }}>公开端点，无需认证。</p>
        </div>
      </div>

      {/* 创建/编辑权限组 Modal */}
      {showGroupModal && (
        <div className="modal-overlay" ref={groupOverlayRef} onClick={e => { if (e.target === groupOverlayRef.current) closeGroupModal(); }}>
          <div className="modal-box large">
            <div className="modal-header">
              <div className="modal-title">{editingGid ? '编辑权限组' : '创建新权限组'}</div>
              <button className="modal-close" onClick={closeGroupModal}><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label className="form-label">权限组名称 <span style={{ color: 'var(--red)' }}>*</span></label>
                <input type="text" className="form-input" placeholder="例如：Moderator, VIP, Tester" value={gName}
                  onChange={e => setGName(e.target.value)} />
              </div>
              <div className="form-group">
                <label className="form-label">该组包含的权限 (勾选)</label>
                <div className="checkbox-grid">
                  {ALL_PERMISSIONS.map(([key, label]) => (
                    <label key={key} className="checkbox-label">
                      <input type="checkbox" checked={gPerms.has(key)} onChange={() => toggleGPerm(key)} />
                      {label}
                    </label>
                  ))}
                </div>
              </div>
            </div>
            <div className="modal-footer">
              <button className="btn btn-outline" onClick={closeGroupModal} type="button">取消</button>
              <button className="btn btn-primary" onClick={saveGroup} disabled={gSaving || !gName} type="button">
                {gSaving ? '保存中...' : editingGid ? '保存修改' : '创建组'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 添加/编辑管理员 Modal */}
      {showAdminModal && (
        <div className="modal-overlay" ref={adminOverlayRef} onClick={e => { if (e.target === adminOverlayRef.current) closeAdminModal(); }}>
          <div className="modal-box large">
            <div className="modal-header">
              <div className="modal-title">{editingAid ? '编辑管理员' : '添加游戏管理员'}</div>
              <button className="modal-close" onClick={closeAdminModal}><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label className="form-label">管理员 SteamID64 <span style={{ color: 'var(--red)' }}>*</span></label>
                <input type="text" className="form-input" placeholder="例如：76561198000000000" value={aSteamId} disabled={!!editingAid}
                  onChange={e => { setASteamId(e.target.value); setALookupName(''); }} />
                <button className="btn btn-outline" style={{ alignSelf: 'flex-start', marginTop: 4, padding: '4px 10px' }}
                  onClick={handleSteamLookup} disabled={aLooking || aSteamId.length < 10} type="button">
                  {aLooking ? '查询中...' : '查询 Steam 名称'}
                </button>
                {aLookupName && <div style={{ fontSize: 12, color: 'var(--text2)', marginTop: 4 }}>Steam 名称：<strong>{aLookupName}</strong></div>}
              </div>
              <div className="form-group">
                <label className="form-label">玩家名称 (选填)</label>
                <input type="text" className="form-input" placeholder="备注名称" value={aName}
                  onChange={e => setAName(e.target.value)} />
              </div>
              <div className="form-group">
                <label className="form-label">权限组 <span style={{ color: 'var(--red)' }}>*</span></label>
                <select className="form-select" value={aGroupName} onChange={e => setAGroupName(e.target.value)}>
                  <option value="">-- 选择权限组 --</option>
                  {groups.map(g => <option key={g.id} value={g.group_name}>{g.group_name} ({g.permissions.split(',').filter(Boolean).length} 项权限)</option>)}
                </select>
              </div>
            </div>
            <div className="modal-footer">
              <button className="btn btn-outline" onClick={closeAdminModal} type="button">取消</button>
              <button className="btn btn-primary" onClick={saveAdmin} disabled={aSaving || !aSteamId || !aGroupName} type="button">
                {aSaving ? '保存中...' : editingAid ? '保存修改' : '保存管理员'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
