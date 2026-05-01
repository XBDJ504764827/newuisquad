'use client';

interface TopbarProps {
  category: string;
  page: string;
  onToggleSidebar: () => void;
  onToggleTheme: () => void;
}

export default function Topbar({ category, page, onToggleSidebar, onToggleTheme }: TopbarProps) {
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

      <div className="topbar-right">
        <button className="icon-btn" onClick={onToggleTheme}>
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
          </svg>
        </button>
        <div className="avatar">AD</div>
      </div>
    </header>
  );
}
