'use client';

import { useState, useEffect, useCallback } from 'react';

const API_BASE = '/api/v1';

interface FileInfo {
  name: string;
  size: number;
}

export default function ConfigFilePage() {
  const [servers, setServers] = useState<{ id: number; name: string }[]>([]);
  const [selectedServerId, setSelectedServerId] = useState<number | null>(null);
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [content, setContent] = useState('');
  const [originalContent, setOriginalContent] = useState('');
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [modified, setModified] = useState(false);

  useEffect(() => {
    fetch(`${API_BASE}/servers`)
      .then(r => r.json())
      .then(data => {
        setServers(data.data || []);
        if (data.data?.length > 0) setSelectedServerId(data.data[0].id);
      })
      .catch(() => {});
  }, []);

  const fetchFiles = useCallback(async () => {
    if (!selectedServerId) return;
    setLoading(true);
    setError('');
    try {
      const res = await fetch(`${API_BASE}/servers/${selectedServerId}/files/list`);
      const data = await res.json();
      if (data.error) {
        setError(data.error);
        setFiles([]);
      } else {
        setFiles(data.files || []);
      }
    } catch {
      setError('获取文件列表失败');
    }
    setLoading(false);
  }, [selectedServerId]);

  useEffect(() => {
    if (selectedServerId) {
      fetchFiles();
      setSelectedFile(null);
      setContent('');
      setOriginalContent('');
      setModified(false);
    }
  }, [selectedServerId, fetchFiles]);

  const openFile = useCallback(async (filename: string) => {
    if (!selectedServerId) return;
    setLoading(true);
    setError('');
    setSuccess('');
    try {
      const res = await fetch(`${API_BASE}/servers/${selectedServerId}/files?path=${encodeURIComponent(filename)}`);
      const data = await res.json();
      if (data.error) {
        setError(data.error);
      } else {
        setSelectedFile(filename);
        const text = data.content || '';
        setContent(text);
        setOriginalContent(text);
        setModified(false);
      }
    } catch {
      setError('读取文件失败');
    }
    setLoading(false);
  }, [selectedServerId]);

  const saveFile = useCallback(async () => {
    if (!selectedServerId || !selectedFile) return;
    setSaving(true);
    setError('');
    setSuccess('');
    try {
      const res = await fetch(`${API_BASE}/servers/${selectedServerId}/files`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: selectedFile, content, admin_user: 'Admin' }),
      });
      const data = await res.json();
      if (data.error) {
        setError(data.error);
      } else {
        setSuccess('保存成功');
        setOriginalContent(content);
        setModified(false);
        setTimeout(() => setSuccess(''), 2000);
      }
    } catch {
      setError('保存失败');
    }
    setSaving(false);
  }, [selectedServerId, selectedFile, content]);

  const handleContentChange = useCallback((value: string) => {
    setContent(value);
    setModified(value !== originalContent);
  }, [originalContent]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      saveFile();
    }
  }, [saveFile]);

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {servers.length > 0 && (
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
          <select className="rcon-input" style={{ width: 'auto', padding: '6px 10px' }} value={selectedServerId || ''}
            onChange={e => setSelectedServerId(parseInt(e.target.value))}>
            {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
          </select>
          <button className="rcon-btn" style={{ width: 'auto', padding: '6px 12px', fontSize: 12 }} onClick={fetchFiles}>刷新列表</button>
        </div>
      )}

      <div className="card">
        <div className="card-header">
          <div>
            <div className="card-title">配置文件编辑器</div>
            <div className="card-sub">直接在网页上修改游戏服务器的配置文件</div>
          </div>
          {selectedFile && (
            <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
              {modified && <span style={{ fontSize: 11, color: '#f59e0b' }}>已修改</span>}
              <button className="rcon-btn" style={{ padding: '6px 16px', fontSize: 12, width: 'auto' }}
                onClick={saveFile} disabled={saving || !modified}>
                {saving ? '保存中...' : '保存修改'}
              </button>
            </div>
          )}
        </div>

        {error && (
          <div style={{ padding: '8px 16px', background: 'rgba(239,68,68,0.1)', borderBottom: '1px solid var(--border)', color: 'var(--red)', fontSize: 12 }}>{error}</div>
        )}
        {success && (
          <div style={{ padding: '8px 16px', background: 'rgba(34,197,94,0.1)', borderBottom: '1px solid var(--border)', color: '#22c55e', fontSize: 12 }}>{success}</div>
        )}

        <div className="file-editor-layout">
          <div className="file-tree">
            {!selectedServerId ? (
              <div style={{ padding: 20, color: 'var(--text3)', fontSize: 12 }}>请先选择服务器</div>
            ) : loading && files.length === 0 ? (
              <div style={{ padding: 20, color: 'var(--text3)', fontSize: 12 }}>加载中...</div>
            ) : files.length === 0 ? (
              <div style={{ padding: 20, color: 'var(--text3)', fontSize: 12 }}>
                暂无配置文件<br/>
                <span style={{ fontSize: 11 }}>请在 agent .env 中设置 CONFIG_DIR</span>
              </div>
            ) : (
              files.map(f => (
                <div key={f.name}
                  className={`file-item${selectedFile === f.name ? ' active' : ''}`}
                  onClick={() => openFile(f.name)}>
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/>
                    <polyline points="13 2 13 9 20 9"/>
                  </svg>
                  <span style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{f.name}</span>
                  <span style={{ fontSize: 10, color: 'var(--text3)' }}>{formatSize(f.size)}</span>
                </div>
              ))
            )}
          </div>

          <div className="editor-area">
            {!selectedFile ? (
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--text3)', fontSize: 13 }}>
                请从左侧选择要编辑的文件
              </div>
            ) : loading ? (
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--text3)', fontSize: 13 }}>
                加载文件中...
              </div>
            ) : (
              <>
                <div className="editor-header">
                  <span style={{ fontWeight: 500, fontSize: 12 }}>{selectedFile}</span>
                  <span style={{ fontSize: 11, color: 'var(--text3)' }}>
                    {content.split('\n').length} 行 · {content.length} 字符
                  </span>
                </div>
                <textarea className="editor-textarea"
                  value={content}
                  onChange={e => handleContentChange(e.target.value)}
                  onKeyDown={handleKeyDown}
                  spellCheck={false}
                  placeholder="文件内容为空" />
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
