'use client';

import { useState } from 'react';

const API_BASE = 'http://192.168.0.137:8000/api/v1';

interface Props {
  onLogin: (token: string, username: string, role: string) => void;
}

export default function LoginPage({ onLogin }: Props) {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function handleLogin(e: React.FormEvent) {
    e.preventDefault();
    if (!username || !password) return;
    setLoading(true);
    setError('');
    try {
      const res = await fetch(`${API_BASE}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      });
      const data = await res.json();
      if (data.token) {
        localStorage.setItem('token', data.token);
        localStorage.setItem('username', data.username);
        localStorage.setItem('role', data.role);
        localStorage.setItem('permissions', JSON.stringify(data.permissions || {}));
        onLogin(data.token, data.username, data.role);
      } else {
        setError(data.error || '登录失败');
      }
    } catch {
      setError('网络错误，请检查后端服务');
    }
    setLoading(false);
  }

  return (
    <div style={{
      position: 'fixed', inset: 0, display: 'flex', alignItems: 'center', justifyContent: 'center',
      background: 'var(--bg)', color: 'var(--text)', zIndex: 9999
    }}>
      <div style={{
        width: 380, maxWidth: '90vw', padding: 32,
        background: 'var(--bg2)', border: '1px solid var(--border)', borderRadius: 'var(--radius)',
      }}>
        <div style={{ textAlign: 'center', marginBottom: 28 }}>
          <div style={{ fontSize: 28, fontWeight: 700, letterSpacing: 2, marginBottom: 8 }}>AC</div>
          <div style={{ fontSize: 14, color: 'var(--text2)' }}>管理控制台</div>
        </div>

        <form onSubmit={handleLogin} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          <div>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>用户名</label>
            <input className="rcon-input" style={{ width: '100%', boxSizing: 'border-box' }}
              value={username} onChange={e => setUsername(e.target.value)}
              placeholder="请输入用户名" autoFocus />
          </div>
          <div>
            <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>密码</label>
            <input className="rcon-input" type="password" style={{ width: '100%', boxSizing: 'border-box' }}
              value={password} onChange={e => setPassword(e.target.value)}
              placeholder="请输入密码" />
          </div>

          {error && <div style={{ color: 'var(--red)', fontSize: 12, textAlign: 'center' }}>{error}</div>}

          <button type="submit" disabled={loading}
            style={{
              width: '100%', padding: '10px 0', marginTop: 4,
              background: 'var(--text)', color: 'var(--bg)', border: 'none',
              borderRadius: 'var(--radius)', cursor: 'pointer', fontSize: 14, fontWeight: 500,
              opacity: loading ? 0.6 : 1,
            }}>
            {loading ? '登录中...' : '登 录'}
          </button>
        </form>
      </div>
    </div>
  );
}
