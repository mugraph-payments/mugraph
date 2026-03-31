import { ArrowSquareIn, ArrowSquareOut } from "@phosphor-icons/react";
import type {
  WalletActivityView,
  WalletAssetView,
  WalletIdentityView,
  WalletSummaryMetricView,
} from "../lib/walletView";
import type { WalletActionKind } from "../types/wallet";
import { ActivityStatusBadge } from "./ActivityStatusBadge";
import { StatusChip } from "./StatusChip";

interface WalletHomeScreenProps {
  identity: WalletIdentityView;
  summaryMetrics: WalletSummaryMetricView[];
  assets: WalletAssetView[];
  activity: WalletActivityView[];
  onPrimaryActionSelect: (actionId: Extract<WalletActionKind, "send" | "receive">) => void;
}

function findMetric(metrics: WalletSummaryMetricView[], id: WalletSummaryMetricView["id"]) {
  return metrics.find((metric) => metric.id === id);
}

export function WalletHomeScreen({
  identity,
  summaryMetrics,
  assets,
  activity,
  onPrimaryActionSelect,
}: WalletHomeScreenProps) {
  const totalAda = findMetric(summaryMetrics, "total-value-ada");
  const totalUsd = findMetric(summaryMetrics, "total-value-usd");
  const noteCount = findMetric(summaryMetrics, "note-count");
  const pendingCount = findMetric(summaryMetrics, "pending-activity-count");
  const topAssets = assets.slice(0, 3);
  const recentActivity = activity.slice(0, 3);

  return (
    <section className="grid gap-5 xl:grid-cols-[minmax(0,1.15fr)_minmax(22rem,0.85fr)]">
      <section className="wallet-panel p-5 sm:p-6 xl:col-span-2 xl:p-7">
        <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_minmax(18rem,22rem)] xl:items-start">
          <div className="space-y-5">
            <div className="space-y-2">
              <p className="wallet-kicker text-slate-500">Available balance</p>
              <h2 className="wallet-heading text-3xl font-semibold tracking-tight text-slate-50 xl:text-5xl">
                {totalAda?.value ?? "0 ADA"}
              </h2>
              <p className="wallet-data text-lg text-slate-300 xl:text-2xl">
                {totalUsd?.value ?? "$0.00"}
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
            </div>

            <div className="grid grid-cols-3 gap-3 xl:max-w-3xl">
              <article className="wallet-subtle-card p-4">
                <p className="wallet-kicker text-slate-500">Assets</p>
                <p className="wallet-data mt-2 text-lg font-semibold text-slate-50 xl:text-xl">
                  {assets.length}
                </p>
              </article>
              <article className="wallet-subtle-card p-4">
                <p className="wallet-kicker text-slate-500">Notes</p>
                <p className="wallet-data mt-2 text-lg font-semibold text-slate-50 xl:text-xl">
                  {noteCount?.value ?? "0"}
                </p>
              </article>
              <article className="wallet-subtle-card p-4">
                <p className="wallet-kicker text-slate-500">Pending</p>
                <p className="wallet-data mt-2 text-lg font-semibold text-slate-50 xl:text-xl">
                  {pendingCount?.value ?? "0"}
                </p>
              </article>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3 xl:grid-cols-1 xl:self-center">
            <button
              type="button"
              onClick={() => onPrimaryActionSelect("send")}
              className="wallet-interactive flex items-center justify-center gap-2 rounded-2xl border border-teal-300/25 bg-teal-400/[0.08] px-4 py-4 text-base font-semibold text-teal-50 xl:justify-start xl:px-5 xl:py-4"
            >
              <ArrowSquareOut className="h-5 w-5" weight="duotone" />
              Send
            </button>
            <button
              type="button"
              onClick={() => onPrimaryActionSelect("receive")}
              className="wallet-interactive flex items-center justify-center gap-2 rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-4 text-base font-semibold text-slate-50 xl:justify-start xl:px-5 xl:py-4"
            >
              <ArrowSquareIn className="h-5 w-5" weight="duotone" />
              Receive
            </button>
          </div>
        </div>
      </section>

      <section className="wallet-panel p-5 xl:p-6">
        <div className="flex items-end justify-between gap-3">
          <div>
            <p className="wallet-kicker text-slate-500">Recent activity</p>
            <h3 className="wallet-heading mt-2 text-xl font-semibold text-slate-50">
              Recent activity
            </h3>
          </div>
          <span className="text-sm text-slate-400">{recentActivity.length} items</span>
        </div>

        <div className="mt-4 grid gap-3 2xl:grid-cols-2">
          {recentActivity.map((item) => (
            <article key={item.id} className="wallet-subtle-card p-4">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <p className="wallet-kicker text-slate-500">{item.kindLabel}</p>
                  <p className="wallet-data mt-1 text-lg font-semibold text-slate-50">
                    {item.amountLabel}
                  </p>
                </div>
                <ActivityStatusBadge label={item.statusLabel} tone={item.statusTone} />
              </div>
              <p className="wallet-copy mt-3 text-base leading-7 text-slate-400">{item.summary}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="wallet-panel p-5 xl:p-6">
        <div className="flex items-end justify-between gap-3">
          <div>
            <p className="wallet-kicker text-slate-500">Assets</p>
            <h3 className="wallet-heading mt-2 text-xl font-semibold text-slate-50">Your assets</h3>
          </div>
          <span className="text-sm text-slate-400">{topAssets.length} assets</span>
        </div>

        <div className="mt-4 grid gap-3 2xl:grid-cols-2">
          {topAssets.map((asset) => (
            <article key={asset.id} className="wallet-subtle-card p-4">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="wallet-kicker rounded-full border border-white/10 bg-white/[0.04] px-2.5 py-1 text-slate-200">
                      {asset.ticker}
                    </span>
                    <span className="truncate text-base text-slate-400">{asset.name}</span>
                  </div>
                  <p className="wallet-data mt-3 text-lg font-semibold text-slate-50">
                    {asset.balanceLabel}
                  </p>
                </div>
                <div className="text-right">
                  <p className="wallet-kicker text-slate-500">Share</p>
                  <p className="wallet-data mt-1 text-base text-slate-100">{asset.shareLabel}</p>
                </div>
              </div>
            </article>
          ))}
        </div>
      </section>
    </section>
  );
}
