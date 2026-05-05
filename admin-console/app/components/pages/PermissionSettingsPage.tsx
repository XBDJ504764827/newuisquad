'use client';

import { useState, useEffect, useCallback } from 'react';

import { api } from '../../lib/api';

interface AdminUser {
  id: number;
  username: string;
  role: string;
  permissions: Record<string, boolean>;
  steam_id64: string | null;
}

const PERM_KEYS = ['控制面板', '日志查询', '修改配置', '玩家管理', '权限分配'] as const;

const PERM_DESCRIPTIONS: Record<string, string> = {
  '控制面板': '服务器控制、RCON命令、实时状态',
  '日志查询': '飞天/击倒/玩家信息等日志查看',
  '修改配置': '配置文件编辑、配置面板设置',
  '玩家管理': '警告/踢出/封禁等玩家操作',
  '权限分配': '添加/编辑用户账户与权限',
};

export default function PermissionSettingsPage() {
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [loading, setLoading] = useState(true);
  const [success, setSuccess] = useState('');
  const [error, setError] = useState('');
  const [savingId, setSavingId] = useState<number | null>(null);
  // 本地编辑缓存: { [userId]: { permissions, steam_id64 } }
  const [edits, setEdits] = useState<Record<number, { permissions: Record<string, boolean>; steam_id64: string }>>({});

  const loadUsers = useCallback(async () => {
    const res = await api(`/admins`);
    const data = await res.json();
    const list = (data.data || []) as AdminUser[];
    setUsers(list);
    // 初始化编辑缓存
    const init: Record<number, { permissions: Record<string, boolean>; steam_id64: string }> = {};
    for (const u of list) {
      init[u.id] = {
        permissions: { ...u.permissions },
        steam_id64: u.steam_id64 || '',
      };
    }
    setEdits(init);
    setLoading(false);
  }, []);

  useEffect(() => { loadUsers(); }, [loadUsers]);

  const showSuccess = (msg: string) => { setSuccess(msg); setTimeout(() => setSuccess(''), 3000); };

  const togglePerm = (userId: number, key: string) => {
    setEdits(prev => {
      const e = prev[userId];
      if (!e) return prev;
      return { ...prev, [userId]: { ...e, permissions: { ...e.permissions, [key]: !e.permissions[key] } } };
    });
  };

  const setSteamId = (userId: number, v: string) => {
    setEdits(prev => {
      const e = prev[userId];
      if (!e) return prev;
      return { ...prev, [userId]: { ...e, steam_id64: v } };
    });
  };

  const saveUser = async (userId: number) => {
    setError('');
    const edit = edits[userId];
    if (!edit) return;
    // 过滤空 SteamID64
    const steam_id64 = edit.steam_id64.trim() || null;

    setSavingId(userId);
    const res = await api(`/admins/${userId}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ permissions: edit.permissions, steam_id64 }),
    });
    const data = await res.json();
    setSavingId(null);
    if (data.error) { setError(`保存失败: ${data.error}`); return; }
    showSuccess('权限已保存');
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
      {error && (
        <div style={{ padding: '8px 16px', background: 'rgba(239,68,68,0.12)', border: '1px solid rgba(239,68,68,0.3)', borderRadius: 'var(--radius)', color: 'var(--red)', fontSize: 13 }}>{error}</div>
      )}
      <div className="card">
        <div className="card-header">
          <div><div className="card-title">用户权限设置</div><div className="card-sub">配置每个用户的网站功能权限和游戏内 SteamID64 绑定</div></div>
        </div>
        <div style={{ overflowX: 'auto' }}>
          <table>
            <thead>
              <tr>
                <th style={{ minWidth: 80 }}>账号</th>
                <th style={{ minWidth: 80 }}>角色</th>
                <th style={{ minWidth: 140 }}>SteamID64</th>
                {PERM_KEYS.map(k => (
                  <th key={k} style={{ textAlign: 'center', minWidth: 72, fontSize: 10 }} title={PERM_DESCRIPTIONS[k]}>{k}</th>
                ))}
                <th style={{ minWidth: 60 }}>操作</th>
              </tr>
            </thead>
            <tbody>
              {users.length === 0 && (
                <tr><td colSpan={8 + PERM_KEYS.length} style={{ textAlign: 'center', color: 'var(--text3)', padding: 40 }}>暂无用户</td></tr>
              )}
              {users.map(u => {
                const isSuper = u.role === '超级管理员';
                const edit = edits[u.id];
                if (!edit) return null;
                return (
                  <tr key={u.id}>
                    <td><strong>{u.username}</strong></td>
                    <td><span className={`badge ${isSuper ? 'green' : u.role === '服主协管' ? 'blue' : 'gray'}`}>{u.role}</span></td>
                    <td>
                      <input
                        className="rcon-input"
                        value={edit.steam_id64}
                        onChange={e => setSteamId(u.id, e.target.value)}
                        placeholder="76561198123456789"
                        style={{ fontSize: 11, padding: '3px 6px', width: '100%' }}
                      />
                    </td>
                    {PERM_KEYS.map(k => (
                      <td key={k} style={{ textAlign: 'center' }}>
                        {isSuper ? (
                          <span className="badge green">是</span>
                        ) : (
                          <input
                            type="checkbox"
                            checked={!!edit.permissions[k]}
                            onChange={() => togglePerm(u.id, k)}
                            style={{ cursor: 'pointer' }}
                          />
                        )}
                      </td>
                    ))}
                    <td>
                      <button
                        className="rcon-btn"
                        style={{ padding: '4px 12px', fontSize: 11, width: 'auto' }}
                        onClick={() => saveUser(u.id)}
                        disabled={savingId === u.id}
                      >
                        {savingId === u.id ? '...' : '保存'}
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
