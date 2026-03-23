import type { WalletTone } from "../lib/walletView";

interface MetricPillProps {
  label: string;
  value: string;
  tone?: WalletTone;
}

const toneClasses: Record<
  WalletTone,
  { shell: string; label: string; value: string }
> = {
  neutral: {
    shell: "border-white/10 bg-white/[0.03]",
    label: "text-slate-500",
    value: "text-slate-100",
  },
  positive: {
    shell: "border-teal-300/20 bg-teal-400/10",
    label: "text-teal-200/75",
    value: "text-teal-50",
  },
  warning: {
    shell: "border-amber-300/20 bg-amber-400/10",
    label: "text-amber-200/75",
    value: "text-amber-50",
  },
  critical: {
    shell: "border-rose-300/20 bg-rose-400/10",
    label: "text-rose-200/75",
    value: "text-rose-50",
  },
};

export function MetricPill({
  label,
  value,
  tone = "neutral",
}: MetricPillProps) {
  const classes = toneClasses[tone];

  return (
    <div className={`rounded-[1.25rem] border px-4 py-3 ${classes.shell}`}>
      <p className={`text-[11px] uppercase tracking-[0.22em] ${classes.label}`}>
        {label}
      </p>
      <p className={`mt-2 text-lg font-semibold tracking-tight ${classes.value}`}>
        {value}
      </p>
    </div>
  );
}
