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
    <section className="wallet-panel p-5 sm:p-6 lg:p-7">
      <div className="wallet-section-stack">
        <div className="wallet-section-intro">
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
          <p className="wallet-copy max-w-[40ch] text-base leading-7 text-slate-400">
            Follow the full transaction stream, then confirm or revert pending items directly from
            the ledger.
          </p>
        </div>

        {items.length === 0 ? (
          <div className="py-8 text-center">
            <div className="wallet-empty-illustration wallet-soft-float mx-auto mb-3">
              <span className="text-base">↺</span>
            </div>
            <p className="text-sm font-medium text-slate-300">No transactions yet</p>
            <p className="mt-1 text-sm text-slate-400">Your first handoff will show up here.</p>
          </div>
        ) : (
          <div className="wallet-list" role="list" aria-label="Transaction list">
            {items.map((item) => (
              <div key={item.id} role="listitem">
                <ActivityRow activity={item} onConfirm={handleConfirm} onRevert={handleRevert} />
              </div>
            ))}
          </div>
        )}
      </div>
    </section>
  );
}
