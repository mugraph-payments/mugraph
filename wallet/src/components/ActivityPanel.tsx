import { motion, useReducedMotion } from "framer-motion";
import type { WalletActivityView } from "../lib/walletView";
import { ActivityRow } from "./ActivityRow";

interface ActivityPanelProps {
  activity: WalletActivityView[];
}

export function ActivityPanel({ activity }: ActivityPanelProps) {
  const prefersReducedMotion = useReducedMotion();

  return (
    <motion.section
      initial={prefersReducedMotion ? false : { opacity: 0.96, y: 10 }}
      whileInView={prefersReducedMotion ? undefined : { opacity: 1, y: 0 }}
      viewport={{ once: true, amount: 0.2 }}
      transition={{ duration: 0.24, ease: [0.16, 1, 0.3, 1] }}
      className="wallet-panel p-5 sm:p-6"
    >
      <div className="flex items-end justify-between gap-3">
        <div className="space-y-1">
          <p className="wallet-kicker text-slate-500">Activity</p>
          <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
            Transactions
          </h2>
        </div>
        {activity.length > 0 ? (
          <span className="text-sm text-slate-400">
            {activity.length} {activity.length === 1 ? "item" : "items"}
          </span>
        ) : null}
      </div>

      {activity.length === 0 ? (
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
          {activity.map((item) => (
            <div key={item.id} role="listitem">
              <ActivityRow activity={item} />
            </div>
          ))}
        </div>
      )}
    </motion.section>
  );
}
