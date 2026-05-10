'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import { api } from '../../lib/api';

interface Server { id: number; server_id: string; name: string; ip: string; rcon_port: number; }
interface PermDef { id: string; code: string; category: string; name: string; description: string; }
interface PermGroup { id: string; name: string; permissions: string[]; is_admin: boolean; created_at?: string; }
interface AdminEntry { id: string; user_id?: string; steam_id?: string; username: string; group_id: string; group_name?: string; }

type TabKey = 'roles' | 'admins';

export default function ServerRolesPage() {
  const [servers, setServers] = useState<Server[]>([]);
  const [serverId, setServerId] = useState<number | null>(null);
  const [activeTab, setActiveTab] = useState<TabKey>('roles');
  const [groups, setGroups] = useState<PermGroup[]>([]);
  const [admins, setAdmins] = useState<AdminEntry[]>([]);
  const [permCatalog, setPermCatalog] = useState<PermDef[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showGroupDialog, setShowGroupDialog] = useState(false);
  const [showAdminDialog, setShowAdminDialog] = useState(false);
  const [editingGroup, setEditingGroup] = useState<PermGroup | null>(null);
  const [savingGroup, setSavingGroup] = useState(false);
  const [savingAdmin, setSavingAdmin] = useState(false);
  // Group form
  const [groupName, setGroupName] = useState('');
  const [groupIsAdmin, setGroupIsAdmin] = useState(false);
  const [groupPerms, setGroupPerms] = useState<string[]>([]);
  // Admin form
  const [adminUsername, setAdminUsername] = useState('');
  const [adminSteamId, setAdminSteamId] = useState('');
  const [adminGroupId, setAdminGroupId] = useState('');
  const [adminType, setAdminType] = useState<'user' | 'steam'>('user');

  useEffect(() => {
    api('/servers').then(r => r.json()).then(d => {
      const list = d.data || [];
      setServers(list);
      if (list.length > 0) setServerId(list[0].id);
    }).catch(() => {});
    // Load permission catalog
    api('/permission-catalog').then(r => r.json()).then(d => {
      setPermCatalog((d.data || d.catalog || []).map((c: any) => ({
        id: c.id || c.code || '', code: c.code || c.id || '', category: c.category || '', name: c.name || c.code || '', description: c.description || '',
      })));
    }).catch(() => {});
  }, []);

  const loadData = useCallback(async (sid: number) => {
    setLoading(true);
    setError(null);
    try {
      const [gRes, aRes] = await Promise.all([
        api(`/servers/${sid}/permission-groups`),
        api(`/servers/${sid}/permission-admins`),
      ]);
      const gData = await gRes.json();
      const aData = await aRes.json();
      const rawGroups = (gData.data || gData.groups || []).map((g: any) => {
        let perms: string[] = [];
        if (Array.isArray(g.permissions)) {
          perms = g.permissions;
        } else if (typeof g.permissions === 'string') {
          try { perms = JSON.parse(g.permissions); } catch { perms = []; }
        }
        return { ...g, permissions: perms };
      });
      setGroups(rawGroups);
      const adminList = (aData.data || aData.admins || []).map((a: any) => ({
        id: a.id || '', user_id: a.user_id || '', steam_id: a.steam_id || '',
        username: a.username || a.steam_id || a.user_id || '未知',
        group_id: a.group_id || a.permission_group_id || '',
        group_name: a.group_name || '',
      }));
      setAdmins(adminList);
    } catch (e: any) { setError(e.message); }
    setLoading(false);
  }, []);

  useEffect(() => { if (serverId) loadData(serverId); }, [serverId, loadData]);

  const saveGroup = async () => {
    if (!serverId || !groupName.trim()) return;
    setSavingGroup(true);
    setError(null);
    try {
      if (editingGroup) {
        await api(`/servers/${serverId}/permission-groups/${editingGroup.id}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name: groupName, is_admin: groupIsAdmin, permissions: groupPerms }),
        });
      } else {
        await api(`/servers/${serverId}/permission-groups`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name: groupName, is_admin: groupIsAdmin, permissions: groupPerms }),
        });
      }
      setShowGroupDialog(false);
      loadData(serverId);
    } catch (e: any) { setError(e.message); }
    setSavingGroup(false);
  };

  const deleteGroup = async (gid: string) => {
    if (!serverId) return;
    try {
      await api(`/servers/${serverId}/permission-groups/${gid}`, { method: 'DELETE' });
      loadData(serverId);
    } catch (e: any) { setError(e.message); }
  };

  const openEditGroup = (g: PermGroup) => {
    setEditingGroup(g);
    setGroupName(g.name);
    setGroupIsAdmin(g.is_admin);
    setGroupPerms(g.permissions || []);
    setShowGroupDialog(true);
  };

  const openNewGroup = () => {
    setEditingGroup(null);
    setGroupName('');
    setGroupIsAdmin(false);
    setGroupPerms([]);
    setShowGroupDialog(true);
  };

  const saveAdmin = async () => {
    if (!serverId) return;
    setSavingAdmin(true);
    setError(null);
    try {
      const body: any = { group_id: adminGroupId };
      if (adminType === 'user') body.user_id = adminUsername;
      else body.steam_id = adminSteamId;
      await api(`/servers/${serverId}/permission-admins`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
      setShowAdminDialog(false);
      loadData(serverId);
    } catch (e: any) { setError(e.message); }
    setSavingAdmin(false);
  };

  const deleteAdmin = async (aid: string) => {
    if (!serverId) return;
    try {
      await api(`/servers/${serverId}/permission-admins/${aid}`, { method: 'DELETE' });
      loadData(serverId);
    } catch (e: any) { setError(e.message); }
  };

  const togglePerm = (code: string) => {
    setGroupPerms(prev => prev.includes(code) ? prev.filter(p => p !== code) : [...prev, code]);
  };

  // Group permissions by category
  const permByCategory: Record<string, PermDef[]> = {};
  permCatalog.forEach(p => {
    const cat = p.category || '其他';
    if (!permByCategory[cat]) permByCategory[cat] = [];
    permByCategory[cat].push(p);
  });

  const styles = {
    container: { padding: 20 },
    header: { display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16, flexWrap: 'wrap' as const },
    select: { padding: '8px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', fontSize: 14, minWidth: 200 },
    btn: { padding: '8px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--text)', color: 'var(--bg)', cursor: 'pointer', fontWeight: 500, fontSize: 13 },
    btnSm: { padding: '4px 10px', borderRadius: 4, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', cursor: 'pointer', fontSize: 11 },
    btnDanger: { padding: '4px 10px', borderRadius: 4, border: '1px solid rgba(239,68,68,0.3)', background: 'rgba(239,68,68,0.1)', color: '#ef4444', cursor: 'pointer', fontSize: 11 },
    tabs: { display: 'flex', gap: 2, marginBottom: 16, borderBottom: '1px solid var(--border)' },
    tabBtn: (active: boolean) => ({ padding: '10px 20px', border: 'none', background: active ? 'var(--text)' : 'transparent', color: active ? 'var(--bg)' : 'var(--text2)', cursor: 'pointer', borderRadius: '8px 8px 0 0', fontSize: 13, fontWeight: active ? 600 : 400 }),
    card: { background: 'var(--bg2)', borderRadius: 10, border: '1px solid var(--border)', padding: 16, marginBottom: 12 },
    table: { width: '100%', borderCollapse: 'collapse' as const, marginTop: 8 },
    th: { textAlign: 'left' as const, padding: '10px 12px', borderBottom: '1px solid var(--border)', fontSize: 12, color: 'var(--text2)' },
    td: { padding: '10px 12px', borderBottom: '1px solid var(--border)', fontSize: 13 },
    input: { padding: '8px 12px', borderRadius: 6, border: '1px solid var(--border)', background: '#1e1e1e', color: 'var(--text)', fontSize: 13, width: '100%', boxSizing: 'border-box' as const },
    badge: (bg: string, color: string) => ({ display: 'inline-block', padding: '3px 10px', borderRadius: 10, fontSize: 11, fontWeight: 600, background: bg, color }),
    modalOverlay: { position: 'fixed' as const, inset: 0, background: 'rgba(0,0,0,0.6)', zIndex: 1000, display: 'flex', alignItems: 'center', justifyContent: 'center' },
    modal: { background: 'var(--bg)', borderRadius: 12, border: '1px solid var(--border)', width: '90vw', maxWidth: 650, maxHeight: '80vh', display: 'flex', flexDirection: 'column' as const, overflow: 'hidden' },
    permCheck: (checked: boolean) => ({ display: 'inline-flex', alignItems: 'center', gap: 6, padding: '6px 10px', borderRadius: 6, border: checked ? '1px solid var(--accent)' : '1px solid var(--border)', background: checked ? 'rgba(59,130,246,0.1)' : 'var(--bg2)', cursor: 'pointer', fontSize: 12, color: checked ? 'var(--text)' : 'var(--text3)', margin: '0 4px 4px 0' }),
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h2 style={{ margin: 0, fontSize: 20, fontWeight: 600 }}>用户与角色管理</h2>
        <select value={serverId || ''} onChange={e => setServerId(Number(e.target.value))} style={styles.select}>
          {servers.map(s => <option key={s.id} value={s.id}>{s.name} ({s.ip}:{s.rcon_port})</option>)}
        </select>
        <button onClick={() => loadData(serverId!)} style={{ ...styles.btn, background: 'var(--bg2)', color: 'var(--text)' }}>刷新</button>
      </div>

      <div style={styles.tabs}>
        <button onClick={() => setActiveTab('roles')} style={styles.tabBtn(activeTab === 'roles')}>权限组（角色）</button>
        <button onClick={() => setActiveTab('admins')} style={styles.tabBtn(activeTab === 'admins')}>管理员分配</button>
      </div>

      {error && <div style={{ padding: '8px 12px', background: 'rgba(239,68,68,0.1)', color: '#ef4444', borderRadius: 6, marginBottom: 12, fontSize: 13 }}>{error}</div>}
      {loading && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>加载中...</div>}

      {/* Roles Tab */}
      {activeTab === 'roles' && (
        <>
          <div style={{ marginBottom: 12 }}>
            <button onClick={openNewGroup} style={styles.btn}>新建权限组</button>
          </div>
          {groups.length === 0 && !loading && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>暂无权限组</div>}
          {groups.map(g => (
            <div key={g.id} style={styles.card}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: 8 }}>
                <div>
                  <strong style={{ fontSize: 14, color: 'var(--text)' }}>{g.name}</strong>
                  <span style={styles.badge(g.is_admin ? 'rgba(34,197,94,0.15)' : 'rgba(156,163,175,0.15)', g.is_admin ? '#22c55e' : '#9ca3af')} >
                    {g.is_admin ? '管理员组' : '普通组'}
                  </span>
                </div>
                <div style={{ display: 'flex', gap: 6 }}>
                  <button onClick={() => openEditGroup(g)} style={styles.btnSm}>编辑</button>
                  <button onClick={() => deleteGroup(g.id)} style={styles.btnDanger}>删除</button>
                </div>
              </div>
              <div style={{ marginTop: 8, fontSize: 12, color: 'var(--text2)' }}>
                权限数量：{(g.permissions || []).length}
              </div>
              <div style={{ marginTop: 6, display: 'flex', flexWrap: 'wrap', gap: 4 }}>
                {(g.permissions || []).slice(0, 8).map(p => {
                  const def = permCatalog.find(d => d.code === p);
                  return <span key={p} style={{ fontSize: 10, padding: '2px 6px', borderRadius: 4, background: 'rgba(59,130,246,0.1)', color: '#3b82f6' }}>{def?.name || p}</span>;
                })}
                {(g.permissions || []).length > 8 && <span style={{ fontSize: 10, color: 'var(--text3)' }}>+{g.permissions.length - 8} 更多...</span>}
              </div>
            </div>
          ))}
        </>
      )}

      {/* Admins Tab */}
      {activeTab === 'admins' && (
        <>
          <div style={{ marginBottom: 12 }}>
            <button onClick={() => { setAdminUsername(''); setAdminSteamId(''); setAdminGroupId(groups[0]?.id || ''); setAdminType('user'); setShowAdminDialog(true); }} style={styles.btn}>添加管理员</button>
          </div>
          {admins.length === 0 && !loading && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>暂无管理员</div>}
          {admins.length > 0 && (
            <table style={styles.table}>
              <thead><tr>
                <th style={styles.th}>用户标识</th><th style={styles.th}>类型</th><th style={styles.th}>所属权限组</th><th style={styles.th}>操作</th>
              </tr></thead>
              <tbody>
                {admins.map(a => (
                  <tr key={a.id}>
                    <td style={{ ...styles.td, fontFamily: 'monospace' }}>{a.username}</td>
                    <td style={styles.td}>
                      <span style={styles.badge(a.steam_id ? 'rgba(234,179,8,0.15)' : 'rgba(59,130,246,0.15)', a.steam_id ? '#eab308' : '#3b82f6')}>
                        {a.steam_id ? 'Steam ID' : '用户'}
                      </span>
                    </td>
                    <td style={styles.td}>{a.group_name || a.group_id}</td>
                    <td style={styles.td}>
                      <button onClick={() => deleteAdmin(a.id)} style={styles.btnDanger}>移除</button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </>
      )}

      {/* Group Dialog */}
      {showGroupDialog && (
        <div style={styles.modalOverlay} onClick={() => setShowGroupDialog(false)}>
          <div style={styles.modal} onClick={e => e.stopPropagation()}>
            <div style={{ padding: '16px 20px', borderBottom: '1px solid var(--border)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <h3 style={{ margin: 0, fontSize: 16 }}>{editingGroup ? '编辑权限组' : '新建权限组'}</h3>
              <button onClick={() => setShowGroupDialog(false)} style={styles.btnSm}>关闭</button>
            </div>
            <div style={{ padding: 20, overflow: 'auto', flex: 1 }}>
              <div style={{ marginBottom: 14 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 4 }}>组名</label>
                <input value={groupName} onChange={e => setGroupName(e.target.value)} style={styles.input} placeholder="权限组名称" />
              </div>
              <div style={{ marginBottom: 14 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)', display: 'flex', alignItems: 'center', gap: 8 }}>
                  <input type="checkbox" checked={groupIsAdmin} onChange={e => setGroupIsAdmin(e.target.checked)} />
                  此组为管理员组
                </label>
              </div>
              <div style={{ marginBottom: 8 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 8 }}>权限分配</label>
                {Object.entries(permByCategory).map(([cat, perms]) => (
                  <div key={cat} style={{ marginBottom: 12 }}>
                    <div style={{ fontSize: 11, fontWeight: 600, color: 'var(--text3)', marginBottom: 6, textTransform: 'uppercase' }}>{cat}</div>
                    <div style={{ display: 'flex', flexWrap: 'wrap' }}>
                      {perms.map(p => (
                        <label key={p.code} style={styles.permCheck(groupPerms.includes(p.code))} onClick={() => togglePerm(p.code)}>
                          <input type="checkbox" checked={groupPerms.includes(p.code)} onChange={() => {}} style={{ display: 'none' }} />
                          {p.name || p.code}
                        </label>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
            <div style={{ padding: '12px 20px', borderTop: '1px solid var(--border)', display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button onClick={() => setShowGroupDialog(false)} style={{ ...styles.btn, background: 'var(--bg2)', color: 'var(--text)' }}>取消</button>
              <button onClick={saveGroup} disabled={!groupName.trim() || savingGroup} style={{ ...styles.btn, opacity: (!groupName.trim() || savingGroup) ? 0.5 : 1 }}>
                {savingGroup ? '保存中...' : '保存'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Admin Dialog */}
      {showAdminDialog && (
        <div style={styles.modalOverlay} onClick={() => setShowAdminDialog(false)}>
          <div style={styles.modal} onClick={e => e.stopPropagation()}>
            <div style={{ padding: '16px 20px', borderBottom: '1px solid var(--border)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <h3 style={{ margin: 0, fontSize: 16 }}>添加管理员</h3>
              <button onClick={() => setShowAdminDialog(false)} style={styles.btnSm}>关闭</button>
            </div>
            <div style={{ padding: 20 }}>
              <div style={{ marginBottom: 14, display: 'flex', gap: 12 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)', display: 'flex', alignItems: 'center', gap: 6 }}>
                  <input type="radio" checked={adminType === 'user'} onChange={() => setAdminType('user')} /> 按用户ID
                </label>
                <label style={{ fontSize: 12, color: 'var(--text2)', display: 'flex', alignItems: 'center', gap: 6 }}>
                  <input type="radio" checked={adminType === 'steam'} onChange={() => setAdminType('steam')} /> 按Steam ID
                </label>
              </div>
              <div style={{ marginBottom: 14 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 4 }}>
                  {adminType === 'user' ? '用户ID' : 'Steam ID'}
                </label>
                <input
                  value={adminType === 'user' ? adminUsername : adminSteamId}
                  onChange={e => adminType === 'user' ? setAdminUsername(e.target.value) : setAdminSteamId(e.target.value)}
                  style={styles.input}
                  placeholder={adminType === 'user' ? '输入用户ID' : '输入17位Steam ID'}
                />
              </div>
              <div style={{ marginBottom: 14 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 4 }}>所属权限组</label>
                <select value={adminGroupId} onChange={e => setAdminGroupId(e.target.value)} style={{ ...styles.input }}>
                  <option value="">选择权限组...</option>
                  {groups.map(g => <option key={g.id} value={g.id}>{g.name}</option>)}
                </select>
              </div>
            </div>
            <div style={{ padding: '12px 20px', borderTop: '1px solid var(--border)', display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button onClick={() => setShowAdminDialog(false)} style={{ ...styles.btn, background: 'var(--bg2)', color: 'var(--text)' }}>取消</button>
              <button onClick={saveAdmin} disabled={!adminGroupId || savingAdmin} style={{ ...styles.btn, opacity: (!adminGroupId || savingAdmin) ? 0.5 : 1 }}>
                {savingAdmin ? '保存中...' : '添加'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
