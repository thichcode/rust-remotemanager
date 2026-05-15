import { useParams, useNavigate } from 'react-router-dom';
import { useSessionStore } from '../stores/sessionStore';
import TerminalSessionComponent from '../components/terminal/TerminalSession';

export default function TerminalPage() {
  const { sessionId } = useParams<{ sessionId: string }>();
  const navigate = useNavigate();
  const { sessions } = useSessionStore();

  const session = sessions.find((s) => s.id === sessionId);

  if (!session) {
    return (
      <div className="flex items-center justify-center h-full bg-[var(--bg-primary)]">
        <div className="text-center">
          <p className="text-sm text-[var(--text-secondary)]">Session not found</p>
          <button
            onClick={() => navigate('/connections')}
            className="mt-4 inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--accent)] text-white text-sm font-medium hover:bg-[var(--accent-hover)] transition-colors"
          >
            Back to connections
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full w-full">
      <TerminalSessionComponent session={session} onClose={() => navigate('/connections')} />
    </div>
  );
}