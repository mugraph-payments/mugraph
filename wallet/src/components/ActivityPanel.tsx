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
    <div className="mt-5 rounded-[1.5rem] border border-dashed border-white/10 bg-white/[0.02] p-5">
      <h3 className="text-sm font-medium text-slate-100">{title}</h3>
      <p className="mt-2 max-w-xl text-sm leading-6 text-slate-400">{copy}</p>
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
      transition={{ duration: 0.26, ease: [0.16, 1, 0.3, 1] }}
      className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur"
    >
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

      {activity.length === 0 ? (
        <EmptyPanelBody
          title="No activity is recorded in this preview"
          copy="Use the ready preview to inspect the timeline. The empty preview keeps the activity lane intentional when there are no deposits, refreshes, or withdrawals to show."
        />
      ) : (
        <div className="mt-5 grid gap-3 overflow-x-clip">
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
