import type {
  WalletIdentityView,
  WalletSummaryMetricView,
  WalletTone,
} from "../lib/walletView";
import { MetricPill } from "./MetricPill";
import { StatusChip } from "./StatusChip";

interface HeroSummaryProps {
  identity: WalletIdentityView;
  summaryMetrics: WalletSummaryMetricView[];
}

const summaryToneClasses: Record<
  WalletTone,
  { shell: string; eyebrow: string; copy: string }
> = {
  neutral: {
    shell: "border-white/10 bg-slate-950/60",
    eyebrow: "text-slate-500",
    copy: "text-slate-400",
  },
  positive: {
    shell: "border-teal-300/20 bg-teal-400/[0.08]",
    eyebrow: "text-teal-200/75",
    copy: "text-slate-300",
  },
  warning: {
    shell: "border-amber-300/20 bg-amber-400/[0.08]",
    eyebrow: "text-amber-200/75",
    copy: "text-slate-300",
  },
  critical: {
    shell: "border-rose-300/20 bg-rose-400/[0.08]",
    eyebrow: "text-rose-200/75",
    copy: "text-slate-300",
  },
};

export function HeroSummary({
  identity,
  summaryMetrics,
}: HeroSummaryProps) {
  const tone = summaryToneClasses[identity.statusTone];

  return (
    <section
      className={`rounded-[2rem] border p-4 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur sm:p-5 ${tone.shell}`}
    >
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-3">
          <div className="space-y-1">
            <p className={`text-xs uppercase tracking-[0.22em] ${tone.eyebrow}`}>
              Wallet overview
            </p>
            <h2 className="text-xl font-semibold tracking-tight text-slate-50 sm:text-2xl">
              Available balance and queue status
            </h2>
            <p className={`text-sm leading-6 ${tone.copy}`}>
              Quick wallet totals, note availability, and pending work.
            </p>
          </div>

          <div className="flex flex-wrap gap-2">
            <StatusChip label="Network" value={identity.networkLabel} compact />
            <StatusChip
              label="Status"
              value={identity.statusLabel}
              tone={identity.statusTone}
              compact
            />
            <StatusChip label="Last sync" value={identity.lastSyncedRelative} compact />
          </div>
        </div>

        <div className="grid min-w-0 gap-3 sm:grid-cols-2 xl:min-w-[24rem] xl:grid-cols-2">
          {summaryMetrics.map((metric) => (
            <MetricPill
              key={metric.id}
              label={metric.label}
              value={metric.value}
              tone={metric.tone}
            />
          ))}
        </div>
      </div>
    </section>
  );
}
