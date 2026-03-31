import { motion, useReducedMotion } from "framer-motion";
import type { WalletTone } from "../lib/walletView";

interface StatusChipProps {
  label: string;
  value: string;
  tone?: WalletTone;
  compact?: boolean;
}

const toneClasses: Record<WalletTone, { container: string; label: string; value: string }> = {
  neutral: {
    container: "wallet-chip-neutral",
    label: "wallet-chip-label-neutral",
    value: "wallet-chip-value-neutral",
  },
  positive: {
    container: "border-teal-300/25 bg-teal-400/12",
    label: "text-teal-100/80",
    value: "text-teal-50",
  },
  warning: {
    container: "border-amber-300/30 bg-amber-400/14",
    label: "text-amber-100/80",
    value: "text-amber-50",
  },
  critical: {
    container: "border-rose-300/30 bg-rose-400/14",
    label: "text-rose-100/80",
    value: "text-rose-50",
  },
};

export function StatusChip({ label, value, tone = "neutral", compact = false }: StatusChipProps) {
  const classes = toneClasses[tone];
  const prefersReducedMotion = useReducedMotion();

  return (
    <motion.div
      initial={prefersReducedMotion ? false : { opacity: 0.96, y: 4 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
      className={`border text-sm will-change-transform ${classes.container} ${
        compact
          ? "grid gap-1.5 rounded-2xl px-3.5 py-3"
          : "inline-flex items-center gap-2.5 rounded-full px-3.5 py-2"
      }`}
      style={{ boxShadow: "inset 0 1px 0 rgba(255,255,255,0.03)" }}
    >
      <span className={`wallet-kicker ${classes.label}`}>{label}</span>
      <span className={`wallet-data text-sm font-semibold tracking-[0.01em] ${classes.value}`}>
        {value}
      </span>
    </motion.div>
  );
}
