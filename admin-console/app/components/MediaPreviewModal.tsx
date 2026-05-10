'use client';

import { useState, useEffect, useCallback } from 'react';

interface EvidenceFile {
  file_path?: string | null; file_name?: string | null;
  file_size?: number | null; file_type?: string | null;
}

interface Props {
  open: boolean;
  file: EvidenceFile | null;
  files: EvidenceFile[];
  currentIndex: number;
  previewUrl: string;
  onClose: () => void;
  onNavigate: (direction: 'prev' | 'next') => void;
}

function getMediaCategory(mimeType?: string | null): 'image' | 'video' | 'pdf' | 'text' | 'unknown' {
  if (!mimeType) return 'unknown';
  const t = mimeType.toLowerCase();
  if (t.startsWith('image/')) return 'image';
  if (t.startsWith('video/')) return 'video';
  if (t === 'application/pdf') return 'pdf';
  if (t.startsWith('text/') || t.includes('json') || t.includes('xml') || t.includes('log') || t.includes('cfg')) return 'text';
  return 'unknown';
}

function fmtSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export default function MediaPreviewModal({ open, file, files, currentIndex, previewUrl, onClose, onNavigate }: Props) {
  const [textContent, setTextContent] = useState('');
  const [loadingText, setLoadingText] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const category = getMediaCategory(file?.file_type);

  const hasPrev = currentIndex > 0;
  const hasNext = currentIndex < files.length - 1;

  useEffect(() => {
    if (!open) { setTextContent(''); setLoadError(null); return; }
    if (category === 'text') {
      setLoadingText(true);
      setLoadError(null);
      fetch(previewUrl, { credentials: 'include' })
        .then(r => { if (!r.ok) throw new Error('fail'); return r.text(); })
        .then(t => setTextContent(t))
        .catch(() => setLoadError('加载文本内容失败'))
        .finally(() => setLoadingText(false));
    }
    setLoadError(null);
  }, [open, file, category, previewUrl]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!open) return;
      if (e.key === 'Escape') onClose();
      if (e.key === 'ArrowLeft' && hasPrev) onNavigate('prev');
      if (e.key === 'ArrowRight' && hasNext) onNavigate('next');
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [open, hasPrev, hasNext, onClose, onNavigate]);

  if (!open) return null;

  const s = {
    overlay: { position: 'fixed' as const, inset: 0, zIndex: 2000, background: 'rgba(0,0,0,0.7)', display: 'flex', alignItems: 'center', justifyContent: 'center' },
    modal: { background: 'var(--bg)', borderRadius: 12, border: '1px solid var(--border)', width: '92vw', maxWidth: 900, maxHeight: '90vh', display: 'flex', flexDirection: 'column' as const, overflow: 'hidden' },
    header: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '14px 18px', borderBottom: '1px solid var(--border)' },
    content: { flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', minHeight: 300, background: 'rgba(0,0,0,0.2)', position: 'relative' as const, overflow: 'hidden' },
    footer: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '10px 18px', borderTop: '1px solid var(--border)' },
    btnSm: { padding: '6px 14px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', cursor: 'pointer', fontSize: 12 },
    navBtn: (side: 'left' | 'right') => ({ position: 'absolute' as const, [side]: 10, top: '50%', transform: 'translateY(-50%)', zIndex: 10, width: 40, height: 40, borderRadius: 20, background: 'rgba(0,0,0,0.5)', border: 'none', color: '#fff', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 18, opacity: 0.8 }),
  };

  return (
    <div style={s.overlay} onClick={onClose}>
      <div style={s.modal} onClick={e => e.stopPropagation()}>
        <div style={s.header}>
          <div>
            <h4 style={{ margin: 0, fontSize: 14, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: 500 }}>{file?.file_name || '预览'}</h4>
            <span style={{ fontSize: 11, color: 'var(--text3)' }}>{file?.file_size ? fmtSize(file.file_size) : ''} · {file?.file_type || '未知类型'}</span>
          </div>
          <button onClick={onClose} style={{ ...s.btnSm, fontSize: 14, padding: '4px 10px' }}>×</button>
        </div>

        <div style={s.content}>
          {loadError && (
            <div style={{ textAlign: 'center', padding: 30, color: '#ef4444' }}>
              <p>{loadError}</p>
            </div>
          )}

          {!loadError && category === 'image' && (
            <img src={previewUrl} alt={file?.file_name || ''} style={{ maxWidth: '100%', maxHeight: '70vh', objectFit: 'contain' }}
              onError={() => setLoadError('加载图片失败')} />
          )}

          {!loadError && category === 'video' && (
            <video src={previewUrl} controls style={{ maxWidth: '100%', maxHeight: '70vh' }}
              onError={() => setLoadError('加载视频失败')}>不支持视频播放</video>
          )}

          {!loadError && category === 'pdf' && (
            <iframe src={previewUrl} style={{ width: '100%', height: '70vh', border: 'none' }} title="PDF预览" />
          )}

          {!loadError && category === 'text' && (
            <div style={{ width: '100%', height: '70vh', overflow: 'auto', padding: 14 }}>
              {loadingText ? <div style={{ textAlign: 'center', color: 'var(--text3)', padding: 30 }}>加载中...</div> :
                <pre style={{ margin: 0, fontSize: 12, fontFamily: 'monospace', whiteSpace: 'pre-wrap', wordBreak: 'break-all', color: 'var(--text)' }}>{textContent}</pre>}
            </div>
          )}

          {!loadError && category === 'unknown' && (
            <div style={{ textAlign: 'center', color: 'var(--text3)', padding: 30 }}>
              <p>该文件类型暂不支持预览</p>
            </div>
          )}

          {hasPrev && !loadError && (
            <button style={s.navBtn('left')} onClick={() => onNavigate('prev')}>◀</button>
          )}
          {hasNext && !loadError && (
            <button style={s.navBtn('right')} onClick={() => onNavigate('next')}>▶</button>
          )}
        </div>

        <div style={s.footer}>
          <span style={{ fontSize: 11, color: 'var(--text3)' }}>{files.length > 1 ? `第 ${currentIndex + 1} / ${files.length} 个` : ''}</span>
          <button onClick={onClose} style={s.btnSm}>关闭</button>
        </div>
      </div>
    </div>
  );
}
