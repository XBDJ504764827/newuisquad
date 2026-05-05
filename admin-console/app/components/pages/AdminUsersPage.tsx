'use client';

import { useState, useEffect, useCallback } from 'react';

import { api } from '../../lib/api';

interface AdminUser {
  id: number;
  username: string;
  role: string;
  steam_id64: string | null;
  notes: string | null;
  created_at: string;
}

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
  });

  const loadUsers = useCallback(async () => {
    const res = await api(`/admins`);
    const data = await res.json();
    setUsers(data.data || []);
    setLoading(false);
  }, []);

  useEffect(() => { loadUsers(); }, [loadUsers]);

  const openAdd = () => {
    setEditing(null);
    setForm({ username: '', password: '', role: '巡查员', steam_id64: '', notes: '' });
    setError('');
    setShowModal(true);
  };

  const openEdit = (u: AdminUser) => {
    setEditing(u);
    setForm({
      username: u.username, password: '', role: u.role,
      steam_id64: u.steam_id64 || '', notes: u.notes || '',
    });
    setError('');
    setShowModal(true);
  };

  const handleSubmit = async () => {
    setError('');
    const body: Record<string, unknown> = {
      username: form.username,
      role: form.role,
      steam_id64: form.steam_id64 || null,
      notes: form.notes || null,
    };
    if (form.password) body.password = form.password;

    const url = editing ? `/admins/${editing.id}` : `/admins`;
    const method = editing ? 'PUT' : 'POST';

    const res = await api(url, {
      method, headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    const data = await res.json();
    if (data.error) { setError(data.error); return; }
    setShowModal(false);
    setSuccess(editing ? '账户已更新' : '账户已创建');
    setTimeout(() => setSuccess(''), 3000);
    loadUsers();
  };

  const handleDelete = async (id: number) => {
    await api(`/admins/${id}`, { method: 'DELETE' });
    setSuccess('账户已删除');
    setTimeout(() => setSuccess(''), 3000);
    loadUsers();
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
          <div><div className="card-title">网站登录账户</div><div className="card-sub">管理网站登录账户，不含权限配置（权限请在用户权限设置中配置）</div></div>
          <button className="rcon-btn" style={{ padding: '6px 14px', fontSize: 12, width: 'auto' }} onClick={openAdd}>添加账户</button>
        </div>
        <div style={{ overflowX: 'auto' }}>
          <table>
            <thead>
              <tr>
                <th>账号</th>
                <th>角色</th>
                <th>SteamID64</th>
                <th>备注</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {users.length === 0 && (
                <tr><td colSpan={5} style={{ textAlign: 'center', color: 'var(--text3)', padding: 40 }}>暂无登录账户</td></tr>
              )}
              {users.map(u => (
                <tr key={u.id}>
                  <td><strong>{u.username}</strong></td>
                  <td><span className={`badge ${u.role === '超级管理员' ? 'green' : u.role === '服主协管' ? 'blue' : 'gray'}`}>{u.role}</span></td>
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
          <div style={{ background: 'var(--bg2)', border: '1px solid var(--border)', borderRadius: 'var(--radius)', padding: 24, width: 420, maxWidth: '90vw' }}
            onClick={e => e.stopPropagation()}>
            <h3 style={{ marginBottom: 20, fontSize: 16 }}>{editing ? '编辑账户' : '添加账户'}</h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              <label style={{ fontSize: 12, color: 'var(--text2)' }}>账号</label>
              <input className="rcon-input" value={form.username} onChange={e => setForm({...form, username: e.target.value})} placeholder="用户名" />
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
