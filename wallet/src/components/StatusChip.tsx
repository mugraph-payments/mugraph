import type { WalletTone } from "../lib/walletView";

interface StatusChipProps {
  label: string;
  value: string;
  tone?: WalletTone;
}

const toneClasses: Record<WalletTone, string> = {
  neutral: "border-white/10 bg-white/5 text-slate-200",
  positive: "border-teal-400/20 bg-teal-400/10 text-teal-100",
  warning: "border-amber-400/20 bg-amber-400/10 text-amber-100",
  critical: "border-rose-400/20 bg-rose-400/10 text-rose-100",
};

export function StatusChip({
  label,
  value,
  tone = "neutral",
}: StatusChipProps) {
  return (
    <div
      className={`rounded-full border px-3 py-1 text-xs ${toneClasses[tone]}`}
    >
      <span className="text-slate-500">{label}</span>{" "}
      <span>{value}</span>
    </div>
  );
}
