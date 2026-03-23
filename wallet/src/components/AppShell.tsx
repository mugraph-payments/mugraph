import type { ReactNode } from "react";

interface AppShellProps {
  header: ReactNode;
  workspace: ReactNode;
}

export function AppShell({ header, workspace }: AppShellProps) {
  return (
    <div className="min-h-[100dvh] overflow-x-clip text-slate-50">
      <div className="mx-auto flex min-h-[100dvh] w-full max-w-[72rem] flex-col gap-4 overflow-x-clip px-4 py-4 sm:px-5 lg:px-6">
        {header}

        <main className="flex min-h-0 flex-1 flex-col gap-4 overflow-x-clip">
          {workspace}
        </main>
      </div>
    </div>
  );
}
