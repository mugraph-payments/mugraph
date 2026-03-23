import type {
  WalletIdentityView,
  WalletSummaryMetricView,
} from "../lib/walletView";
import { MetricPill } from "./MetricPill";
import { StatusChip } from "./StatusChip";

interface HeroSummaryProps {
  identity: WalletIdentityView;
  summaryMetrics: WalletSummaryMetricView[];
}

export function HeroSummary({
  identity,
  summaryMetrics,
}: HeroSummaryProps) {
  return (
    <section className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="space-y-3">
          <p className="text-xs uppercase tracking-[0.22em] text-teal-300/70">
            Portfolio summary
          </p>
          <div className="space-y-2">
            <h2 className="text-2xl font-semibold tracking-tight text-slate-50 sm:text-3xl">
              {identity.label}
            </h2>
            <p className="max-w-2xl text-sm leading-6 text-slate-400 sm:text-base">
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
            <StatusChip label="Last sync" value={identity.lastSyncedRelative} />
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
