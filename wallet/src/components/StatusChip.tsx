import type { WalletTone } from "../lib/walletView";

interface StatusChipProps {
  label: string;
  value: string;
  tone?: WalletTone;
}

const toneClasses: Record<
  WalletTone,
  { container: string; label: string; value: string }
> = {
  neutral: {
    container: "border-white/10 bg-white/5",
    label: "text-slate-500",
    value: "text-slate-200",
  },
  positive: {
    container: "border-teal-400/20 bg-teal-400/10",
    label: "text-teal-200/75",
    value: "text-teal-100",
  },
  warning: {
    container: "border-amber-400/25 bg-amber-400/12",
    label: "text-amber-200/75",
    value: "text-amber-100",
  },
  critical: {
    container: "border-rose-400/25 bg-rose-400/12",
    label: "text-rose-200/75",
    value: "text-rose-100",
  },
};

export function StatusChip({
  label,
  value,
  tone = "neutral",
}: StatusChipProps) {
  const classes = toneClasses[tone];

  return (
    <div
      className={`rounded-full border px-3 py-1 text-xs ${classes.container}`}
    >
      <span className={classes.label}>{label}</span>{" "}
      <span className={classes.value}>{value}</span>
    </div>
  );
}
