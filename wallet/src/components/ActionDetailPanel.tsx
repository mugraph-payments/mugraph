import type { WalletActionView } from "../lib/walletView";

interface ActionDetailPanelProps {
  action: WalletActionView;
}

export function ActionDetailPanel({ action }: ActionDetailPanelProps) {
  return (
    <section className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Detail region
          </p>
          <h2 className="mt-2 text-xl font-semibold tracking-tight text-slate-50">
            {action.label} is selected
          </h2>
        </div>
        <span className="self-start rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[11px] uppercase tracking-[0.22em] text-slate-300">
          {action.id}
        </span>
      </div>

      <div className="mt-4 rounded-[1.5rem] border border-dashed border-white/10 bg-white/[0.02] p-4">
        <p className="text-sm leading-6 text-slate-300">{action.helper}</p>
        <p className="mt-3 text-sm leading-6 text-slate-400">
          This framework keeps the selected action detail visible without
          replacing the action grid. The next steps will swap this placeholder
          for action-specific receive, deposit, send, and withdraw surfaces.
        </p>
      </div>
    </section>
  );
}
