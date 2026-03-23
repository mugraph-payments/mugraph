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
  { shell: string; eyebrow: string; accent: string }
> = {
  neutral: {
    shell: "border-white/10 bg-slate-950/60",
    eyebrow: "text-teal-300/70",
    accent: "text-slate-400",
  },
  positive: {
    shell: "border-teal-400/15 bg-[linear-gradient(180deg,rgba(20,184,166,0.12),rgba(2,6,23,0.72))]",
    eyebrow: "text-teal-200/80",
    accent: "text-teal-100/75",
  },
  warning: {
    shell: "border-amber-400/20 bg-[linear-gradient(180deg,rgba(245,158,11,0.12),rgba(2,6,23,0.72))]",
    eyebrow: "text-amber-200/80",
    accent: "text-amber-100/75",
  },
  critical: {
    shell: "border-rose-400/20 bg-[linear-gradient(180deg,rgba(244,63,94,0.12),rgba(2,6,23,0.72))]",
    eyebrow: "text-rose-200/80",
    accent: "text-rose-100/75",
  },
};

export function HeroSummary({
  identity,
  summaryMetrics,
}: HeroSummaryProps) {
  const tone = summaryToneClasses[identity.statusTone];

  return (
    <section
      className={`rounded-[2rem] border p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur ${tone.shell}`}
    >
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="space-y-3">
          <p className={`text-xs uppercase tracking-[0.22em] ${tone.eyebrow}`}>
            Portfolio summary
          </p>
          <div className="space-y-2">
            <h2 className="text-2xl font-semibold tracking-tight text-slate-50 sm:text-3xl">
              {identity.label}
            </h2>
            <p className={`max-w-2xl text-sm leading-6 sm:text-base ${tone.accent}`}>
              Track the wallet posture at a glance before dropping into actions,
              notes, and settlement details.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <StatusChip label="Network" value={identity.networkLabel} />
            <StatusChip
              label="Status"
              value={identity.statusLabel}
              tone={identity.statusTone}
            />
            <StatusChip
              label="Last sync"
              value={identity.lastSyncedRelative}
              tone={identity.statusTone === "positive" ? "neutral" : identity.statusTone}
            />
          </div>
        </div>
      </div>

      <div className="mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        {summaryMetrics.map((metric) => (
          <MetricPill
            key={metric.id}
            label={metric.label}
            value={metric.value}
            tone={metric.tone}
          />
        ))}
      </div>
    </section>
  );
}
