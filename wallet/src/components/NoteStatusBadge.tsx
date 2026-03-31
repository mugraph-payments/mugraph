import type { WalletTone } from "../lib/walletView";

interface NoteStatusBadgeProps {
  label: string;
  tone: WalletTone;
}

const toneClasses: Record<WalletTone, string> = {
  neutral: "border-white/10 bg-white/5 text-slate-200",
  positive: "border-teal-400/20 bg-teal-400/10 text-teal-100",
  warning: "border-amber-400/20 bg-amber-400/10 text-amber-100",
  critical: "border-rose-400/20 bg-rose-400/10 text-rose-100",
};

export function NoteStatusBadge({ label, tone }: NoteStatusBadgeProps) {
  return (
    <span
      className={`inline-flex items-center rounded-full border px-3.5 py-1.5 text-sm font-medium ${toneClasses[tone]}`}
    >
      {label}
    </span>
  );
}
