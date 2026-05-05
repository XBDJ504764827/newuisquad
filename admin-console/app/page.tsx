'use client';

import { useState, useEffect, useCallback } from 'react';
import Sidebar from './components/Sidebar';
import Topbar from './components/Topbar';
import LoginPage from './components/LoginPage';
import SummaryPage from './components/pages/SummaryPage';
import ControlPanelPage from './components/pages/ControlPanelPage';
import ChatLogsPage from './components/pages/ChatLogsPage';
import FlyLogsPage from './components/pages/FlyLogsPage';
import KillLogsPage from './components/pages/KillLogsPage';
import MatchLogsPage from './components/pages/MatchLogsPage';
import ConfigFilePage from './components/pages/ConfigFilePage';
import ConfigPanelPage from './components/pages/ConfigPanelPage';
import ActionLogsPage from './components/pages/ActionLogsPage';
import PlayerInfoPage from './components/pages/PlayerInfoPage';
import AdminUsersPage from './components/pages/AdminUsersPage';
import PermissionSettingsPage from './components/pages/PermissionSettingsPage';
import { PageId } from './types';

const pageComponents: Record<PageId, React.ComponentType> = {
  'summary': SummaryPage,
  'control-panel': ControlPanelPage,
  'chat-logs': ChatLogsPage,
  'fly-logs': FlyLogsPage,
  'kill-logs': KillLogsPage,
  'match-logs': MatchLogsPage,
  'config-file': ConfigFilePage,
  'config-panel': ConfigPanelPage,
  'action-logs': ActionLogsPage,
  'player-info': PlayerInfoPage,
  'admin-users': AdminUsersPage,
  'permission-settings': PermissionSettingsPage,
};

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
