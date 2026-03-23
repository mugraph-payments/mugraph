import type { ReactNode } from "react";

interface AppShellProps {
  header: ReactNode;
  primary: ReactNode;
  secondary: ReactNode;
}

export function AppShell({ header, primary, secondary }: AppShellProps) {
  return (
    <div className="min-h-[100dvh] text-slate-50">
      <div className="mx-auto flex min-h-[100dvh] w-full max-w-7xl flex-col gap-4 px-4 py-4 sm:px-6 lg:px-8">
        {header}

        <main className="grid flex-1 gap-4 lg:grid-cols-[minmax(0,1.2fr)_minmax(18rem,0.8fr)]">
          <section className="grid content-start gap-4">{primary}</section>
          <aside className="grid content-start gap-4">{secondary}</aside>
        </main>
      </div>
    </div>
  );
}
