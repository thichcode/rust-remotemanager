import { type ReactNode } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import { ThemeProvider } from './themes/theme';
import Sidebar from './components/layout/Sidebar';
import MainArea from './components/layout/MainArea';
import StatusBar from './components/layout/StatusBar';
import Dashboard from './pages/Dashboard';
import Connections from './pages/Connections';
import TerminalPage from './pages/TerminalPage';
import Settings from './pages/Settings';
import TerminalTab from './components/terminal/TerminalTab';
import { useSessionStore } from './stores/sessionStore';

// ─── Layout ─────────────────────────────────────────────────────────────────

function AppLayout({ children }: { children: ReactNode }) {
  const { sessions } = useSessionStore();

  const handleNewConnection = () => {
    window.location.href = '/connections';
  };

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[var(--bg-primary)]">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0">
        {sessions.length > 0 && (
          <TerminalTab onNewConnection={handleNewConnection} />
        )}
        <MainArea>{children}</MainArea>
        <StatusBar />
      </div>
    </div>
  );
}

// ─── App Shell ──────────────────────────────────────────────────────────────

function AppShell() {
  return (
    <ThemeProvider>
      <BrowserRouter>
        <AppLayout>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/connections" element={<Connections />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/terminal/:sessionId" element={<TerminalPage />} />
          </Routes>
        </AppLayout>
      </BrowserRouter>
      <Toaster
        position="bottom-right"
        toastOptions={{
          duration: 4000,
          style: {
            borderRadius: '12px',
            border: '1px solid var(--border)',
            background: 'var(--bg-secondary)',
            color: 'var(--text-primary)',
          },
        }}
      />
    </ThemeProvider>
  );
}

export default AppShell;
