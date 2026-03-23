import { motion, useReducedMotion } from "framer-motion";
import type { WalletActivityView } from "../lib/walletView";
import { ActivityRow } from "./ActivityRow";

interface ActivityPanelProps {
  activity: WalletActivityView[];
}

function EmptyPanelBody({
  title,
  copy,
}: {
  title: string;
  copy: string;
}) {
  return (
    <div className="mt-4 rounded-[1.5rem] border border-dashed border-white/10 bg-white/[0.02] p-5">
      <h3 className="wallet-heading text-sm font-medium text-slate-100">{title}</h3>
      <p className="wallet-copy mt-2 max-w-xl text-sm leading-6 text-slate-400">{copy}</p>
    </div>
  );
}

export function ActivityPanel({ activity }: ActivityPanelProps) {
  const prefersReducedMotion = useReducedMotion();

  return (
    <motion.section
      initial={prefersReducedMotion ? false : { opacity: 0.96, y: 10 }}
      whileInView={prefersReducedMotion ? undefined : { opacity: 1, y: 0 }}
      viewport={{ once: true, amount: 0.2 }}
      transition={{ duration: 0.24, ease: [0.16, 1, 0.3, 1] }}
      className="wallet-panel p-4 sm:p-5"
    >
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div className="space-y-1">
          <p className="wallet-kicker text-slate-500">History</p>
          <h2 className="wallet-heading text-xl font-semibold tracking-tight text-slate-50">
            Wallet activity lane
          </h2>
        </div>
        <p className="wallet-copy max-w-xl text-sm leading-6 text-slate-400">
          Follow deposits, refreshes, and withdrawals in one compact history
          lane without expanding into a full audit view.
        </p>
      </div>

      {activity.length === 0 ? (
        <EmptyPanelBody
          title="No activity is recorded"
          copy="This wallet preview has no deposits, refreshes, or withdrawals yet. The history lane stays visible so the empty state still feels intentional."
        />
      ) : (
        <div className="mt-4 grid gap-3 overflow-x-clip">
          {activity.map((item, index) => (
            <motion.div
              key={item.id}
              initial={prefersReducedMotion ? false : { opacity: 0.94, y: 8 }}
              whileInView={prefersReducedMotion ? undefined : { opacity: 1, y: 0 }}
              viewport={{ once: true, amount: 0.15 }}
              transition={{
                duration: 0.22,
                delay: prefersReducedMotion ? 0 : index * 0.04,
                ease: [0.16, 1, 0.3, 1],
              }}
              className="overflow-x-clip"
            >
              <ActivityRow activity={item} />
            </motion.div>
          ))}
        </div>
      )}
    </motion.section>
  );
}
