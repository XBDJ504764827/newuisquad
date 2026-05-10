'use client';

interface BreadcrumbItem { label: string; href?: string; }

interface Props { items: BreadcrumbItem[]; onNavigate?: (href: string) => void; }

export default function Breadcrumbs({ items, onNavigate }: Props) {
  return (
    <nav style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 13, flexWrap: 'wrap' }}>
      {items.map((item, idx) => {
        const isLast = idx === items.length - 1;
        return (
          <span key={idx} style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            {idx > 0 && (
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--text3)" strokeWidth="2">
                <path d="m9 18 6-6-6-6"/>
              </svg>
            )}
            {isLast ? (
              <span style={{ color: 'var(--text)', fontWeight: 500 }}>{item.label}</span>
            ) : (
              <button
                style={{ background: 'none', border: 'none', color: 'var(--text3)', cursor: 'pointer', padding: 0, fontSize: 13 }}
                onClick={() => item.href && onNavigate?.(item.href)}
              >
                {item.label}
              </button>
            )}
          </span>
        );
      })}
    </nav>
  );
}
