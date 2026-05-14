import type { ReactNode } from 'react';

interface MainAreaProps {
  children: ReactNode;
}

export default function MainArea({ children }: MainAreaProps) {
  return (
    <main className="flex-1 overflow-y-auto bg-[var(--bg-primary)]">
      <div className="h-full">{children}</div>
    </main>
  );
}
