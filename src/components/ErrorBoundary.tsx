import { Component, type ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export default class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('[ErrorBoundary] Caught error:', error.message);
    console.error('[ErrorBoundary] Component stack:', info.componentStack);
    console.error('[ErrorBoundary] Digest:', info.digest);
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;
      return (
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            height: '100%',
            color: '#f1f5f9',
            background: '#0f172a',
            fontFamily: 'system-ui, sans-serif',
            gap: '12px',
            padding: '24px',
            textAlign: 'center',
          }}
        >
          <div style={{ fontSize: '48px' }}>⚠️</div>
          <h2 style={{ margin: 0, fontSize: '18px', color: '#ef4444' }}>
            React Render Error
          </h2>
          <p style={{ margin: 0, fontSize: '14px', color: '#94a3b8', maxWidth: '480px' }}>
            {this.state.error?.message ?? 'An unknown error occurred during rendering.'}
          </p>
          {this.state.error?.stack && (
            <pre
              style={{
                fontSize: '11px',
                color: '#64748b',
                textAlign: 'left',
                maxWidth: '600px',
                overflow: 'auto',
                maxHeight: '200px',
                background: '#1e293b',
                padding: '12px',
                borderRadius: '8px',
                border: '1px solid #334155',
              }}
            >
              {this.state.error.stack}
            </pre>
          )}
          <button
            onClick={() => window.location.reload()}
            style={{
              padding: '8px 16px',
              background: '#6366f1',
              color: 'white',
              border: 'none',
              borderRadius: '8px',
              cursor: 'pointer',
              fontSize: '14px',
            }}
          >
            Reload App
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}