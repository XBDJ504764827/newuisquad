'use client';

interface PaginationProps {
    page: number;
    total: number;
    perPage: number;
    onPageChange: (page: number) => void;
}

export default function Pagination({ page, total, perPage, onPageChange }: PaginationProps) {
    const totalPages = Math.max(1, Math.ceil(total / perPage));
    if (total <= perPage) return null;

    return (
        <div style={{ display: 'flex', justifyContent: 'center', gap: 8, padding: 16 }}>
            <button
                className="rcon-btn"
                style={{ width: 'auto', padding: '6px 14px', fontSize: 12 }}
                disabled={page <= 1}
                onClick={() => onPageChange(page - 1)}
            >
                上一页
            </button>
            <span style={{ fontSize: 12, color: 'var(--text2)', alignSelf: 'center' }}>
                第 {page} / {totalPages} 页
            </span>
            <button
                className="rcon-btn"
                style={{ width: 'auto', padding: '6px 14px', fontSize: 12 }}
                disabled={page >= totalPages}
                onClick={() => onPageChange(page + 1)}
            >
                下一页
            </button>
        </div>
    );
}
