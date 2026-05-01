'use client';

import { useState } from 'react';
import Sidebar from './components/Sidebar';
import Topbar from './components/Topbar';
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
import PermissionsPage from './components/pages/PermissionsPage';
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
  'permissions': PermissionsPage,
};

export default function Home() {
  const [activePage, setActivePage] = useState<PageId>('summary');
  const [breadcrumbCat, setBreadcrumbCat] = useState('主页');
  const [breadcrumbPage, setBreadcrumbPage] = useState('概要');
  const [collapsed, setCollapsed] = useState(false);
  const [dark, setDark] = useState(true);

  function handleNavigate(pageId: PageId, category: string, pageName: string) {
    setActivePage(pageId);
    setBreadcrumbCat(category);
    setBreadcrumbPage(pageName);
  }

  function handleToggleSidebar() {
    setCollapsed(!collapsed);
  }

  function handleToggleTheme() {
    const nextDark = !dark;
    setDark(nextDark);
    document.documentElement.className = nextDark ? 'dark' : 'light';
  }

  const ActivePageComponent = pageComponents[activePage];

  return (
    <>
      <Sidebar collapsed={collapsed} activePage={activePage} onNavigate={handleNavigate} />
      <div className="main-area">
        <Topbar
          category={breadcrumbCat}
          page={breadcrumbPage}
          onToggleSidebar={handleToggleSidebar}
          onToggleTheme={handleToggleTheme}
        />
        <main className="content">
          <ActivePageComponent />
        </main>
        <div style={{ position: 'fixed', bottom: 8, right: 8, zIndex: 9999, display: 'flex', gap: 8 }}>
          <span style={{ color: 'var(--text2)', fontSize: 11, lineHeight: '24px' }}>当前: {activePage}</span>
          <button
            onClick={() => setActivePage(activePage === 'summary' ? 'control-panel' : 'summary')}
            style={{ padding: '2px 10px', fontSize: 11, background: 'var(--text)', color: 'var(--bg)', border: 'none', borderRadius: 4, cursor: 'pointer' }}
          >
            测试切换
          </button>
        </div>
      </div>
    </>
  );
}
