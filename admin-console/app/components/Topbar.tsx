'use client';

interface TopbarProps {
  category: string;
  page: string;
  username: string | null;
  onLogout: () => void;
  onToggleSidebar: () => void;
  onToggleTheme: () => void;
}

export default function Topbar({ category, page, username, onLogout, onToggleSidebar, onToggleTheme }: TopbarProps) {
  return (
    <header className="topbar">
      <button className="topbar-collapse-btn" onClick={onToggleSidebar}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <line x1="3" y1="12" x2="21" y2="12"/>
          <line x1="3" y1="6" x2="21" y2="6"/>
          <line x1="3" y1="18" x2="21" y2="18"/>
        </svg>
      </button>

      <div className="breadcrumb">
        <span>{category}</span>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <polyline points="9 18 15 12 9 6"/>
        </svg>
        <span className="current">{page}</span>
      </div>

      <div className="topbar-right" style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <span style={{ fontSize: 12, color: 'var(--text2)' }}>{username}</span>
        <button className="icon-btn" onClick={onToggleTheme} title="切换主题">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
          </svg>
        </button>
        <button className="icon-btn" onClick={onLogout} title="退出登录" style={{ color: 'var(--red)' }}>
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/>
          </svg>
        </button>
      </div>
    </header>
  );
}
