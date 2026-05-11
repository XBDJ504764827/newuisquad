'use client';

import { useState, useEffect, useCallback } from 'react';
import dynamic from 'next/dynamic';
import Sidebar from './components/Sidebar';
import Topbar from './components/Topbar';
import LoginPage from './components/LoginPage';
import { PageId } from './types';
import { AUTH_EXPIRED_EVENT, api, clearAuthStorage } from './lib/api';

const pageComponents: Record<PageId, React.ComponentType> = {
  'summary': dynamic(() => import('./components/pages/SummaryPage'), { loading: () => <LoadingPlaceholder /> }),
  'control-panel': dynamic(() => import('./components/pages/ControlPanelPage'), { loading: () => <LoadingPlaceholder /> }),
  'chat-logs': dynamic(() => import('./components/pages/ChatLogsPage'), { loading: () => <LoadingPlaceholder /> }),
  'fly-logs': dynamic(() => import('./components/pages/FlyLogsPage'), { loading: () => <LoadingPlaceholder /> }),
  'kill-logs': dynamic(() => import('./components/pages/KillLogsPage'), { loading: () => <LoadingPlaceholder /> }),
  'match-logs': dynamic(() => import('./components/pages/MatchLogsPage'), { loading: () => <LoadingPlaceholder /> }),
  'config-file': dynamic(() => import('./components/pages/ConfigFilePage'), { loading: () => <LoadingPlaceholder /> }),
  'config-panel': dynamic(() => import('./components/pages/ConfigPanelPage'), { loading: () => <LoadingPlaceholder /> }),
  'player-info': dynamic(() => import('./components/pages/PlayerInfoPage'), { loading: () => <LoadingPlaceholder /> }),
  'admin-users': dynamic(() => import('./components/pages/AdminUsersPage'), { loading: () => <LoadingPlaceholder /> }),
  'ban-management': dynamic(() => import('./components/pages/BanManagementPage'), { loading: () => <LoadingPlaceholder /> }),
  'audit-dashboard': dynamic(() => import('./components/pages/AuditDashboardPage'), { loading: () => <LoadingPlaceholder /> }),
  'player-detail': dynamic(() => import('./components/pages/PlayerDetailPage'), { loading: () => <LoadingPlaceholder /> }),
  'motd': dynamic(() => import('./components/pages/MotdPage'), { loading: () => <LoadingPlaceholder /> }),
  'workflows': dynamic(() => import('./components/pages/WorkflowsPage'), { loading: () => <LoadingPlaceholder /> }),
  'rcon-console': dynamic(() => import('./components/pages/RconConsolePage'), { loading: () => <LoadingPlaceholder /> }),
  'server-metrics': dynamic(() => import('./components/pages/ServerMetricsPage'), { loading: () => <LoadingPlaceholder /> }),
  'server-feeds': dynamic(() => import('./components/pages/ServerFeedsPage'), { loading: () => <LoadingPlaceholder /> }),
  'server-rules': dynamic(() => import('./components/pages/ServerRulesPage'), { loading: () => <LoadingPlaceholder /> }),
  'server-roles': dynamic(() => import('./components/pages/ServerRolesPage'), { loading: () => <LoadingPlaceholder /> }),
  'identity-alts': dynamic(() => import('./components/pages/IdentityAltsPage'), { loading: () => <LoadingPlaceholder /> }),
  'deployable-events': dynamic(() => import('./components/pages/ServerEventsPage'), { loading: () => <LoadingPlaceholder /> }),
  'tick-rate': dynamic(() => import('./components/pages/ServerEventsPage'), { loading: () => <LoadingPlaceholder /> }),
  'vehicle-events': dynamic(() => import('./components/pages/ServerEventsPage'), { loading: () => <LoadingPlaceholder /> }),
};

function LoadingPlaceholder() {
  return <div className="page-view" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '60vh' }}>
    <div style={{ color: 'var(--text3)', fontSize: 14 }}>加载中...</div>
  </div>;
}

const breadcrumbMap: Record<PageId, { cat: string; page: string }> = {
  'summary': { cat: '主页', page: '概要' },
  'control-panel': { cat: '主页', page: '控制面板' },
  'chat-logs': { cat: '日志系统', page: '聊天记录' },
  'fly-logs': { cat: '日志系统', page: '飞天记录' },
  'kill-logs': { cat: '日志系统', page: '击倒记录' },
  'match-logs': { cat: '日志系统', page: '比赛记录' },
  'player-info': { cat: '玩家管理', page: '玩家信息' },
  'admin-users': { cat: '玩家管理', page: '网站管理员' },
  'ban-management': { cat: '玩家管理', page: '玩家封禁' },
  'audit-dashboard': { cat: '日志系统', page: '审计仪表盘' },
  'config-file': { cat: '系统配置', page: '配置文件' },
  'config-panel': { cat: '系统配置', page: '配置面板' },
  'player-detail': { cat: '玩家管理', page: '玩家详情' },
  'motd': { cat: '系统配置', page: 'MOTD 配置' },
  'workflows': { cat: '系统配置', page: '工作流管理' },
  'rcon-console': { cat: '服务器管理', page: 'RCON控制台' },
  'server-metrics': { cat: '服务器管理', page: '性能指标' },
  'server-feeds': { cat: '服务器管理', page: '实时信息' },
  'server-rules': { cat: '服务器管理', page: '规则管理' },
  'server-roles': { cat: '服务器管理', page: '用户与角色' },
  'identity-alts': { cat: '玩家管理', page: '小号检测' },
  'deployable-events': { cat: '日志系统', page: '工事&事件' },
  'tick-rate': { cat: '日志系统', page: '服务器性能' },
  'vehicle-events': { cat: '日志系统', page: '载具记录' },
};

function getHashPage(): PageId | null {
  if (typeof window === 'undefined') return null;
  const hash = window.location.hash.replace('#', '');
  const page = hash.split('?')[0];  // 去掉查询参数
  return (page in pageComponents) ? page as PageId : null;
}

export default function Home() {
  const [activePage, setActivePage] = useState<PageId | null>(null);
  const [breadcrumb, setBreadcrumb] = useState(breadcrumbMap['summary']);
  const [collapsed, setCollapsed] = useState(false);
  const [dark, setDark] = useState(true);
  const [token, setToken] = useState<string | null>(null);
  const [username, setUsername] = useState<string | null>(null);
  const [permissions, setPermissions] = useState<Record<string, boolean>>({});
  const [authChecked, setAuthChecked] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const saved = localStorage.getItem('token');
    if (!saved) {
      const timer = window.setTimeout(() => {
        if (!cancelled) setAuthChecked(true);
      }, 0);
      return () => {
        cancelled = true;
        window.clearTimeout(timer);
      };
    }
    api('/auth/verify', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token: saved }),
    }).then(res => res.json()).then(data => {
      if (cancelled) return;
      if (data.valid) {
        setToken(saved);
        setUsername(data.username || localStorage.getItem('username'));
        const nextPermissions = data.permissions || JSON.parse(localStorage.getItem('permissions') || '{}');
        setPermissions(nextPermissions);
        localStorage.setItem('permissions', JSON.stringify(nextPermissions));
        if (data.username) localStorage.setItem('username', data.username);
        if (data.role) localStorage.setItem('role', data.role);
      } else {
        clearAuthStorage();
      }
    }).catch(() => {
      if (!cancelled) clearAuthStorage();
    }).finally(() => {
      if (!cancelled) setAuthChecked(true);
    });
    return () => { cancelled = true; };
  }, []);

  function handleLogin(t: string, u: string) {
    setToken(t);
    setUsername(u);
    try { setPermissions(JSON.parse(localStorage.getItem('permissions') || '{}')); } catch {}
  }

  const handleLogout = useCallback(() => {
    clearAuthStorage();
    setToken(null);
    setUsername(null);
    setPermissions({});
  }, []);

  useEffect(() => {
    window.addEventListener(AUTH_EXPIRED_EVENT, handleLogout);
    return () => window.removeEventListener(AUTH_EXPIRED_EVENT, handleLogout);
  }, [handleLogout]);

  // 首次挂载时从 URL hash 恢复页面（仅客户端），无闪烁
  useEffect(() => {
    const timer = window.setTimeout(() => {
      const page = getHashPage() || 'summary';
      setActivePage(page);
      setBreadcrumb(breadcrumbMap[page]);
    }, 0);
    return () => window.clearTimeout(timer);
  }, []);

  // 页面切换 → 同步到 hash
  useEffect(() => {
    if (activePage) window.location.hash = activePage;
  }, [activePage]);

  const handleNavigate = useCallback((pageId: PageId) => {
    setActivePage(pageId);
    setBreadcrumb(breadcrumbMap[pageId]);
  }, []);

  function handleToggleSidebar() {
    setCollapsed(!collapsed);
  }

  function handleToggleTheme() {
    const nextDark = !dark;
    setDark(nextDark);
    document.documentElement.className = nextDark ? 'dark' : 'light';
  }

  if (!authChecked) return null;

  if (!token) {
    return <LoginPage onLogin={handleLogin} />;
  }

  if (!activePage) return null;

  const ActivePageComponent = pageComponents[activePage];

  return (
    <>
      <Sidebar collapsed={collapsed} activePage={activePage} permissions={permissions} onNavigate={handleNavigate} />
      <div className="main-area">
        <Topbar
          category={breadcrumb.cat}
          page={breadcrumb.page}
          username={username}
          onLogout={handleLogout}
          onToggleSidebar={handleToggleSidebar}
          onToggleTheme={handleToggleTheme}
        />
        <main className="content">
          <ActivePageComponent />
        </main>
      </div>
    </>
  );
}
