import type { WalletTone } from "../lib/walletView";

interface MetricPillProps {
  label: string;
  value: string;
  tone?: WalletTone;
}

const accentClasses: Record<WalletTone, string> = {
  neutral: "from-white/5 to-white/[0.02]",
  positive: "from-teal-400/12 to-white/[0.02]",
  warning: "from-amber-400/12 to-white/[0.02]",
  critical: "from-rose-400/12 to-white/[0.02]",
};

export function MetricPill({
  label,
  value,
  tone = "neutral",
}: MetricPillProps) {
  return (
    <div
      className={`rounded-[1.5rem] border border-white/10 bg-gradient-to-br ${accentClasses[tone]} p-4`}
    >
      <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
        {label}
      </p>
      <p className="mt-2 text-xl font-semibold tracking-tight text-slate-100">
        {value}
      </p>
    </div>
  );
}
