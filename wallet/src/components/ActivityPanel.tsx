import { useState } from "react";
import type { WalletActivityView } from "../lib/walletView";
import { ActivityRow } from "./ActivityRow";

interface ActivityPanelProps {
  activity: WalletActivityView[];
}

type StatusOverride = "confirmed" | "reverted";

export function ActivityPanel({ activity }: ActivityPanelProps) {
  const [overrides, setOverrides] = useState<Record<string, StatusOverride>>({});

  function handleConfirm(id: string) {
    setOverrides((prev) => ({ ...prev, [id]: "confirmed" }));
  }

  function handleRevert(id: string) {
    setOverrides((prev) => ({ ...prev, [id]: "reverted" }));
  }

  const items: WalletActivityView[] = activity.map((item) => {
    const override = overrides[item.id];
    if (!override) return item;
    if (override === "confirmed") {
      return { ...item, statusLabel: "Completed", statusTone: "positive" };
    }
    return { ...item, statusLabel: "Reverted", statusTone: "critical" };
  });

  return (
    <section className="wallet-panel p-5 sm:p-6">
      <div className="flex items-end justify-between gap-3">
        <div className="space-y-1">
          <p className="wallet-kicker text-slate-500">Activity</p>
          <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
            Transactions
          </h2>
        </div>
        {items.length > 0 ? (
          <span className="text-sm text-slate-400">
            {items.length} {items.length === 1 ? "item" : "items"}
          </span>
        ) : null}
      </div>

      {items.length === 0 ? (
        <div className="mt-6 py-8 text-center">
          <p className="text-sm font-medium text-slate-300">No transactions yet</p>
          <p className="mt-1 text-sm text-slate-400">
            Deposits, withdrawals, and refreshes will appear here.
          </p>
        </div>
      ) : (
        <div
          className="mt-4 divide-y divide-white/[0.06]"
          role="list"
          aria-label="Transaction list"
        >
          {items.map((item) => (
            <div key={item.id} role="listitem">
              <ActivityRow activity={item} onConfirm={handleConfirm} onRevert={handleRevert} />
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
