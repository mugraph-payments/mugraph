import type { WalletActivityView } from "../lib/walletView";
import { ActivityRow } from "./ActivityRow";

interface ActivityPanelProps {
  activity: WalletActivityView[];
}

export function ActivityPanel({ activity }: ActivityPanelProps) {
  return (
    <section className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Recent activity
          </p>
          <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-50">
            Deposits, refreshes, and withdrawals stay in one readable lane
          </h2>
        </div>
        <p className="max-w-xl text-sm leading-6 text-slate-400">
          The activity timeline keeps kind, status, amount, summary, reference,
          and relative timing visible without turning into a dense audit log.
        </p>
      </div>

      <div className="mt-5 grid gap-3">
        {activity.map((item) => (
          <ActivityRow key={item.id} activity={item} />
        ))}
      </div>
    </section>
  );
}
