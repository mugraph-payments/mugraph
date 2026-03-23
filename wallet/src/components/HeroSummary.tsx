import { ArrowSquareOut, StackSimple, TrendUp } from "@phosphor-icons/react";
import type {
  WalletIdentityView,
  WalletSummaryMetricView,
  WalletTone,
} from "../lib/walletView";
import { StatusChip } from "./StatusChip";

interface HeroSummaryProps {
  identity: WalletIdentityView;
  summaryMetrics: WalletSummaryMetricView[];
}

const summaryToneClasses: Record<
  WalletTone,
  { shell: string; eyebrow: string; copy: string; accent: string }
> = {
  neutral: {
    shell: "border-white/10",
    eyebrow: "text-slate-500",
    copy: "text-slate-400",
    accent: "text-slate-100",
  },
  positive: {
    shell: "border-teal-300/20",
    eyebrow: "text-teal-200/75",
    copy: "text-slate-300",
    accent: "text-teal-50",
  },
  warning: {
    shell: "border-amber-300/20",
    eyebrow: "text-amber-200/75",
    copy: "text-slate-300",
    accent: "text-amber-50",
  },
  critical: {
    shell: "border-rose-300/20",
    eyebrow: "text-rose-200/75",
    copy: "text-slate-300",
    accent: "text-rose-50",
  },
};

export function HeroSummary({
  identity,
  summaryMetrics,
}: HeroSummaryProps) {
  const tone = summaryToneClasses[identity.statusTone];
  const [totalAda, totalUsd, noteCount, pendingCount] = summaryMetrics;

  return (
    <section className={`wallet-panel p-5 sm:p-6 ${tone.shell}`}>
      <div className="grid gap-5 lg:grid-cols-[minmax(0,1.4fr)_minmax(18rem,0.9fr)]">
        <div className="min-w-0 space-y-5">
          <div className="space-y-2">
            <p className={`wallet-kicker ${tone.eyebrow}`}>Wallet posture</p>
            <h2 className="wallet-heading text-3xl font-semibold tracking-tight text-slate-50 sm:text-[2.6rem] sm:leading-[1.05]">
              Private balances and settlement flow in one focused workspace
            </h2>
            <p className={`wallet-copy max-w-2xl text-sm leading-7 ${tone.copy}`}>
              This redesign treats the wallet like an operator cockpit instead of a dashboard: the current value, inventory health, and next action all stay visible without crowding the screen.
            </p>
          </div>

          <div className="wallet-panel-soft p-5">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
              <div>
                <p className="wallet-kicker text-slate-500">Total value</p>
                <p className="wallet-data mt-3 text-4xl font-semibold tracking-tight text-slate-50 sm:text-5xl">
                  {totalAda?.value}
                </p>
                <p className="wallet-data mt-2 text-base text-slate-300">{totalUsd?.value}</p>
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
          </div>
        </div>

        <div className="grid gap-3 sm:grid-cols-3 lg:grid-cols-1">
          <div className="wallet-card p-4">
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="wallet-kicker text-slate-500">Inventory</p>
                <p className={`wallet-data mt-2 text-2xl font-semibold ${tone.accent}`}>
                  {noteCount?.value}
                </p>
              </div>
              <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-teal-400/10 text-teal-100 ring-1 ring-teal-300/20">
                <StackSimple className="h-5 w-5" weight="duotone" />
              </div>
            </div>
            <p className="wallet-copy mt-3 text-sm leading-6 text-slate-400">
              Spendable notes ready for private transfers and refreshes.
            </p>
          </div>

          <div className="wallet-card p-4">
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="wallet-kicker text-slate-500">Queue</p>
                <p className="wallet-data mt-2 text-2xl font-semibold text-amber-50">
                  {pendingCount?.value}
                </p>
              </div>
              <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-amber-400/10 text-amber-100 ring-1 ring-amber-300/20">
                <ArrowSquareOut className="h-5 w-5" weight="duotone" />
              </div>
            </div>
            <p className="wallet-copy mt-3 text-sm leading-6 text-slate-400">
              In-flight work that still needs confirmation, submission, or operator review.
            </p>
          </div>

          <div className="wallet-card p-4">
            <div className="flex items-start justify-between gap-3">
              <div>
                <p className="wallet-kicker text-slate-500">Momentum</p>
                <p className="wallet-data mt-2 text-2xl font-semibold text-slate-50">Healthy</p>
              </div>
              <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-blue-400/10 text-blue-100 ring-1 ring-blue-300/20">
                <TrendUp className="h-5 w-5" weight="duotone" />
              </div>
            </div>
            <p className="wallet-copy mt-3 text-sm leading-6 text-slate-400">
              The current layout emphasizes action readiness instead of raw data density.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
