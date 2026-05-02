'use client';

import { useState, useEffect, useCallback } from 'react';

const API_BASE = 'http://192.168.0.137:8000/api/v1';

interface AdminUser {
  id: number;
  username: string;
  role: string;
  permissions: Record<string, boolean>;
  steam_id64: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

const PERM_KEYS = ['控制面板', '日志查询', '修改配置', '玩家管理', '权限分配'] as const;
type PermKey = typeof PERM_KEYS[number];
type Permissions = Record<PermKey, boolean>;

const PERM_DESCRIPTIONS: Record<PermKey, string> = {
  '控制面板': '服务器控制、RCON命令、实时状态',
  '日志查询': '飞天/击倒/玩家信息等日志查看',
  '修改配置': '配置文件编辑、配置面板设置',
  '玩家管理': '警告/踢出/封禁等玩家操作',
  '权限分配': '添加/编辑/删除管理员账号',
};

const DEFAULT_PERMISSIONS: Permissions = {
  '控制面板': true,
  '日志查询': true,
  '修改配置': false,
  '玩家管理': false,
  '权限分配': false,
};

export default function AdminUsersPage() {
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [editing, setEditing] = useState<AdminUser | null>(null);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [form, setForm] = useState({
    username: '', password: '', role: '巡查员',
    steam_id64: '', notes: '',
    permissions: { ...DEFAULT_PERMISSIONS } as Permissions,
  });

  const loadUsers = useCallback(async () => {
    const res = await fetch(`${API_BASE}/admins`);
    const data = await res.json();
    setUsers(data.data || []);
    setLoading(false);
  }, []);

  useEffect(() => { loadUsers(); }, [loadUsers]);

  const openAdd = () => {
    setEditing(null);
    setForm({ username: '', password: '', role: '巡查员', steam_id64: '', notes: '', permissions: { ...DEFAULT_PERMISSIONS } });
    setError('');
    setShowModal(true);
  };

  const openEdit = (u: AdminUser) => {
    setEditing(u);
    setForm({
      username: u.username, password: '', role: u.role,
      steam_id64: u.steam_id64 || '', notes: u.notes || '',
      permissions: { ...DEFAULT_PERMISSIONS, ...u.permissions },
    });
    setError('');
    setShowModal(true);
  };

  const handleSubmit = async () => {
    setError('');
    const body: Record<string, unknown> = {
      username: form.username,
      role: form.role,
      permissions: form.permissions,
      steam_id64: form.steam_id64 || null,
      notes: form.notes || null,
    };
    if (form.password) body.password = form.password;

    const url = editing ? `${API_BASE}/admins/${editing.id}` : `${API_BASE}/admins`;
    const method = editing ? 'PUT' : 'POST';

    const res = await fetch(url, {
      method, headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    const data = await res.json();
    if (data.error) { setError(data.error); return; }
    setShowModal(false);
    setSuccess(editing ? '管理员已更新' : '管理员已创建');
    setTimeout(() => setSuccess(''), 3000);
    loadUsers();
  };

  const handleDelete = async (id: number) => {
    await fetch(`${API_BASE}/admins/${id}`, { method: 'DELETE' });
    setSuccess('管理员已删除');
    setTimeout(() => setSuccess(''), 3000);
    loadUsers();
  };

  const togglePerm = (key: PermKey) => {
    setForm(f => ({ ...f, permissions: { ...f.permissions, [key]: !f.permissions[key] } }));
  };

  if (loading) return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="card"><div className="card-body" style={{ textAlign: 'center', color: 'var(--text3)' }}>加载中...</div></div>
    </div>
  );

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {success && (
        <div style={{ padding: '8px 16px', background: 'rgba(34,197,94,0.12)', border: '1px solid rgba(34,197,94,0.3)', borderRadius: 'var(--radius)', color: '#22c55e', fontSize: 13, fontWeight: 500 }}>{success}</div>
      )}
      <div className="card">
        <div className="card-header">
          <div><div className="card-title">网站管理员列表</div><div className="card-sub">管理后台管理员账号与权限</div></div>
          <button className="rcon-btn" style={{ padding: '6px 14px', fontSize: 12, width: 'auto' }} onClick={openAdd}>添加管理员</button>
        </div>
        <div style={{ overflowX: 'auto' }}>
          <table>
            <thead>
              <tr>
                <th>账号</th>
                <th>角色</th>
                <th style={{ textAlign: 'center' }}>控制面板</th>
                <th style={{ textAlign: 'center' }}>日志查询</th>
                <th style={{ textAlign: 'center' }}>修改配置</th>
                <th style={{ textAlign: 'center' }}>玩家管理</th>
                <th style={{ textAlign: 'center' }}>权限分配</th>
                <th>SteamID64</th>
                <th>备注</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {users.length === 0 && (
                <tr><td colSpan={10} style={{ textAlign: 'center', color: 'var(--text3)', padding: 40 }}>暂无管理员账号</td></tr>
              )}
              {users.map(u => (
                <tr key={u.id}>
                  <td><strong>{u.username}</strong></td>
                  <td><span className={`badge ${u.role === '超级管理员' ? 'green' : u.role === '服主协管' ? 'blue' : 'gray'}`}>{u.role}</span></td>
                  {PERM_KEYS.map(k => (
                    <td key={k} style={{ textAlign: 'center' }}>
                      <span className={`badge ${(u.permissions as Permissions)?.[k] ? 'green' : 'gray'}`}>{(u.permissions as Permissions)?.[k] ? '是' : '否'}</span>
                    </td>
                  ))}
                  <td style={{ fontFamily: 'monospace', fontSize: 11 }}>{u.steam_id64 || '-'}</td>
                  <td style={{ maxWidth: 120, overflow: 'hidden', textOverflow: 'ellipsis' }}>{u.notes || '-'}</td>
                  <td>
                    <div style={{ display: 'flex', gap: 6 }}>
                      <span className="badge blue" style={{ cursor: 'pointer' }} onClick={() => openEdit(u)}>编辑</span>
                      <span className="badge red" style={{ cursor: 'pointer' }} onClick={() => { if (confirm('确认删除?')) handleDelete(u.id); }}>删除</span>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {showModal && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }}
          onClick={e => { if (e.target === e.currentTarget) setShowModal(false); }}>
          <div style={{ background: 'var(--bg2)', border: '1px solid var(--border)', borderRadius: 'var(--radius)', padding: 24, width: 480, maxWidth: '90vw', maxHeight: '90vh', overflowY: 'auto' }}
            onClick={e => e.stopPropagation()}>
            <h3 style={{ marginBottom: 20, fontSize: 16 }}>{editing ? '编辑管理员' : '添加管理员'}</h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              <label style={{ fontSize: 12, color: 'var(--text2)' }}>账号</label>
              <input className="rcon-input" value={form.username} onChange={e => setForm({...form, username: e.target.value})} placeholder="admin" />
              <label style={{ fontSize: 12, color: 'var(--text2)' }}>{editing ? '新密码（留空不修改）' : '密码'}</label>
              <input className="rcon-input" type="password" value={form.password} onChange={e => setForm({...form, password: e.target.value})} placeholder="••••••" />
              <label style={{ fontSize: 12, color: 'var(--text2)' }}>角色</label>
              <select className="rcon-input" value={form.role} onChange={e => setForm({...form, role: e.target.value})}>
                <option>超级管理员</option>
                <option>服主协管</option>
                <option>巡查员</option>
              </select>
              <label style={{ fontSize: 12, color: 'var(--text2)' }}>SteamID64</label>
              <input className="rcon-input" value={form.steam_id64} onChange={e => setForm({...form, steam_id64: e.target.value})} placeholder="76561198123456789" />
              <label style={{ fontSize: 12, color: 'var(--text2)' }}>备注</label>
              <input className="rcon-input" value={form.notes} onChange={e => setForm({...form, notes: e.target.value})} placeholder="备注信息" />
              <label style={{ fontSize: 12, color: 'var(--text2)', marginTop: 4 }}>模块权限</label>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {PERM_KEYS.map(k => (
                  <label key={k} style={{ display: 'flex', alignItems: 'flex-start', gap: 8, cursor: 'pointer', padding: '6px 8px', borderRadius: 'var(--radius)', background: 'var(--bg3)' }}>
                    <input type="checkbox" checked={!!form.permissions[k]} onChange={() => togglePerm(k)} style={{ marginTop: 2 }} />
                    <div>
                      <div style={{ fontSize: 13, fontWeight: 500 }}>{k}</div>
                      <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 1 }}>{PERM_DESCRIPTIONS[k]}</div>
                    </div>
                  </label>
                ))}
              </div>
              {error && <div style={{ color: 'var(--red)', fontSize: 12 }}>{error}</div>}
              <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
                <button className="rcon-btn" onClick={handleSubmit}>{editing ? '保存' : '创建'}</button>
                <button className="rcon-btn" style={{ background: 'var(--bg3)', color: 'var(--text2)' }} onClick={() => setShowModal(false)}>取消</button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
