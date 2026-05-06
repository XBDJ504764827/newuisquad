'use client';

import { useState, useEffect, useCallback } from 'react';
import dynamic from 'next/dynamic';
import Sidebar from './components/Sidebar';
import Topbar from './components/Topbar';
import LoginPage from './components/LoginPage';
import { PageId } from './types';

const pageComponents: Record<PageId, React.ComponentType> = {
  'summary': dynamic(() => import('./components/pages/SummaryPage'), { loading: () => <LoadingPlaceholder /> }),
  'control-panel': dynamic(() => import('./components/pages/ControlPanelPage'), { loading: () => <LoadingPlaceholder /> }),
  'chat-logs': dynamic(() => import('./components/pages/ChatLogsPage'), { loading: () => <LoadingPlaceholder /> }),
  'fly-logs': dynamic(() => import('./components/pages/FlyLogsPage'), { loading: () => <LoadingPlaceholder /> }),
  'kill-logs': dynamic(() => import('./components/pages/KillLogsPage'), { loading: () => <LoadingPlaceholder /> }),
  'match-logs': dynamic(() => import('./components/pages/MatchLogsPage'), { loading: () => <LoadingPlaceholder /> }),
  'config-file': dynamic(() => import('./components/pages/ConfigFilePage'), { loading: () => <LoadingPlaceholder /> }),
  'config-panel': dynamic(() => import('./components/pages/ConfigPanelPage'), { loading: () => <LoadingPlaceholder /> }),
  'action-logs': dynamic(() => import('./components/pages/ActionLogsPage'), { loading: () => <LoadingPlaceholder /> }),
  'player-info': dynamic(() => import('./components/pages/PlayerInfoPage'), { loading: () => <LoadingPlaceholder /> }),
  'admin-users': dynamic(() => import('./components/pages/AdminUsersPage'), { loading: () => <LoadingPlaceholder /> }),
  'permission-settings': dynamic(() => import('./components/pages/PermissionSettingsPage'), { loading: () => <LoadingPlaceholder /> }),
  'ban-management': dynamic(() => import('./components/pages/BanManagementPage'), { loading: () => <LoadingPlaceholder /> }),
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
  'action-logs': { cat: '日志系统', page: '操作记录' },
  'player-info': { cat: '玩家管理', page: '玩家信息' },
  'admin-users': { cat: '玩家管理', page: '网站管理员' },
  'permission-settings': { cat: '玩家管理', page: '用户权限设置' },
  'ban-management': { cat: '玩家管理', page: '玩家封禁' },
  'config-file': { cat: '系统配置', page: '配置文件' },
  'config-panel': { cat: '系统配置', page: '配置面板' },
};

function getHashPage(): PageId | null {
  if (typeof window === 'undefined') return null;
  const hash = window.location.hash.replace('#', '');
  return (hash in pageComponents) ? hash as PageId : null;
}

export default function Home() {
  const [activePage, setActivePage] = useState<PageId | null>(null);
  const [breadcrumb, setBreadcrumb] = useState(breadcrumbMap['summary']);
  const [collapsed, setCollapsed] = useState(false);
  const [dark, setDark] = useState(true);
  const [token, setToken] = useState<string | null>(null);
  const [username, setUsername] = useState<string | null>(null);
  const [permissions, setPermissions] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const saved = localStorage.getItem('token');
    if (saved) {
      setToken(saved);
      setUsername(localStorage.getItem('username'));
      try { setPermissions(JSON.parse(localStorage.getItem('permissions') || '{}')); } catch {}
    }
  }, []);

  function handleLogin(t: string, u: string, _role: string) {
    setToken(t);
    setUsername(u);
    try { setPermissions(JSON.parse(localStorage.getItem('permissions') || '{}')); } catch {}
  }

  function handleLogout() {
    localStorage.removeItem('token');
    localStorage.removeItem('username');
    localStorage.removeItem('role');
    localStorage.removeItem('permissions');
    setToken(null);
    setUsername(null);
    setPermissions({});
  }

  // 首次挂载时从 URL hash 恢复页面（仅客户端），无闪烁
  useEffect(() => {
    const page = getHashPage() || 'summary';
    setActivePage(page);
    setBreadcrumb(breadcrumbMap[page]);
  }, []);

  // 页面切换 → 同步到 hash
  useEffect(() => {
    if (activePage) window.location.hash = activePage;
  }, [activePage]);

  const handleNavigate = useCallback((pageId: PageId, _category: string, _pageName: string) => {
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
