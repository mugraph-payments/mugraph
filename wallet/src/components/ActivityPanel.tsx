import { motion, useReducedMotion } from "framer-motion";
import type { WalletActivityView } from "../lib/walletView";
import { ActivityRow } from "./ActivityRow";

interface ActivityPanelProps {
  activity: WalletActivityView[];
}

function EmptyPanelBody({ title, copy }: { title: string; copy: string }) {
  return (
    <div className="wallet-card mt-5 p-5">
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
      className="wallet-panel p-5 sm:p-6 xl:p-7"
    >
      <div className="flex flex-col gap-2">
        <div>
          <p className="wallet-kicker text-slate-500">Activity</p>
          <h2 className="wallet-heading mt-2 text-2xl font-semibold tracking-tight text-slate-50">
            Activity history
          </h2>
        </div>
        <p className="wallet-copy max-w-2xl text-sm leading-6 text-slate-400">
          Browse your wallet activity in one mobile list with the latest items first.
        </p>
      </div>

      {activity.length === 0 ? (
        <EmptyPanelBody
          title="No activity is recorded"
          copy="This wallet preview has no deposits, refreshes, or withdrawals yet. The transaction list stays visible so the empty state still feels intentional."
        />
      ) : (
        <div
          className="mt-5 grid gap-3 overflow-x-clip xl:grid-cols-2 2xl:grid-cols-3"
          aria-label="Activity list"
        >
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
