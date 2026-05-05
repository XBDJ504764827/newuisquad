'use client';

import { Component, ErrorInfo, ReactNode } from 'react';

interface Props { children: ReactNode; }
interface State { hasError: boolean; error: Error | null; }

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          position: 'fixed', inset: 0, display: 'flex', flexDirection: 'column',
          alignItems: 'center', justifyContent: 'center',
          background: 'var(--bg)', color: 'var(--text)', zIndex: 9999, gap: 16,
        }}>
          <div style={{ fontSize: 48 }}>⚠️</div>
          <h2 style={{ margin: 0, fontSize: 20 }}>页面发生错误</h2>
          <p style={{ color: 'var(--text3)', fontSize: 13, maxWidth: 400, textAlign: 'center' }}>
            {this.state.error?.message || '未知错误'}
          </p>
          <button
            onClick={() => { this.setState({ hasError: false, error: null }); window.location.hash = '#summary'; }}
            style={{
              padding: '10px 24px', background: 'var(--text)', color: 'var(--bg)',
              border: 'none', borderRadius: 6, cursor: 'pointer', fontSize: 14, fontWeight: 500,
            }}
          >
            返回主页
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
