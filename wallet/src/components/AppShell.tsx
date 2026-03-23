import type { ReactNode } from "react";

interface AppShellProps {
  header: ReactNode;
  workspace: ReactNode;
}

export function AppShell({ header, workspace }: AppShellProps) {
  return (
    <div className="min-h-[100dvh] text-slate-50">
      <div className="mx-auto flex min-h-[100dvh] w-full max-w-7xl flex-col gap-4 px-4 py-4 sm:px-6 lg:px-8">
        {header}

        <main className="flex min-h-0 flex-1 flex-col gap-4">{workspace}</main>
      </div>
    </div>
  );
}
