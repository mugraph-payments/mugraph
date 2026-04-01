import { ArrowSquareIn, ArrowSquareOut } from "@phosphor-icons/react";
import type {
  WalletActivityView,
  WalletAssetView,
  WalletIdentityView,
  WalletSummaryMetricView,
} from "../lib/walletView";
import type { WalletActionKind } from "../types/wallet";
import { ActivityRow } from "./ActivityRow";
import { AssetRow } from "./AssetRow";
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
    <section className="grid gap-4 self-start lg:grid-cols-[minmax(0,1.15fr)_minmax(22rem,0.85fr)]">
      <section className="wallet-panel p-5 sm:p-6 lg:col-span-2">
        <div className="grid gap-5 lg:grid-cols-[minmax(0,1fr)_minmax(18rem,22rem)] lg:items-start">
          <div className="space-y-5">
            <div className="space-y-2">
              <p className="wallet-kicker text-slate-500">Available balance</p>
              <h2 className="wallet-heading text-3xl font-semibold tracking-tight text-slate-50 lg:text-5xl">
                {totalAda?.value ?? "0 ADA"}
              </h2>
              <p className="wallet-data text-lg text-slate-300 lg:text-2xl">
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

            <div className="grid grid-cols-3 gap-3 lg:max-w-3xl">
              <article className="wallet-subtle-card p-4">
                <p className="wallet-kicker text-slate-500">Assets</p>
                <p className="wallet-data mt-2 text-lg font-semibold text-slate-50 lg:text-xl">
                  {assets.length}
                </p>
              </article>
              <article className="wallet-subtle-card p-4">
                <p className="wallet-kicker text-slate-500">Notes</p>
                <p className="wallet-data mt-2 text-lg font-semibold text-slate-50 lg:text-xl">
                  {noteCount?.value ?? "0"}
                </p>
              </article>
              <article className="wallet-subtle-card p-4">
                <p className="wallet-kicker text-slate-500">Pending</p>
                <p className="wallet-data mt-2 text-lg font-semibold text-slate-50 lg:text-xl">
                  {pendingCount?.value ?? "0"}
                </p>
              </article>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3 lg:grid-cols-1 lg:self-center">
            <button
              type="button"
              onClick={() => onPrimaryActionSelect("send")}
              className="wallet-interactive wallet-cta-primary flex items-center justify-center gap-2 rounded-xl border px-4 py-3.5 text-base font-semibold text-slate-50 lg:justify-start lg:px-5 lg:py-3.5"
            >
              <ArrowSquareOut className="h-5 w-5" weight="duotone" />
              Send
            </button>
            <button
              type="button"
              onClick={() => onPrimaryActionSelect("receive")}
              className="wallet-interactive wallet-cta-secondary flex items-center justify-center gap-2 rounded-xl border px-4 py-3.5 text-base font-semibold text-slate-50 lg:justify-start lg:px-5 lg:py-3.5"
            >
              <ArrowSquareIn className="h-5 w-5" weight="duotone" />
              Receive
            </button>
          </div>
        </div>
      </section>

      <section className="wallet-panel p-5">
        <div className="flex items-end justify-between gap-3">
          <div>
            <p className="wallet-kicker text-slate-500">Recent activity</p>
            <h3 className="wallet-heading mt-1.5 text-lg font-semibold text-slate-50">
              Transactions
            </h3>
          </div>
          <span className="text-sm text-slate-400">{recentActivity.length} items</span>
        </div>

        <div className="mt-3 divide-y divide-white/[0.06]">
          {recentActivity.map((item) => (
            <ActivityRow key={item.id} activity={item} />
          ))}
        </div>
      </section>

      <section className="wallet-panel p-5">
        <div className="flex items-end justify-between gap-3">
          <div>
            <p className="wallet-kicker text-slate-500">Portfolio</p>
            <h3 className="wallet-heading mt-1.5 text-lg font-semibold text-slate-50">Assets</h3>
          </div>
          <span className="text-sm text-slate-400">{topAssets.length} tokens</span>
        </div>

        <div className="mt-3 divide-y divide-white/[0.06]">
          {topAssets.map((asset) => (
            <AssetRow key={asset.id} asset={asset} />
          ))}
        </div>
      </section>
    </section>
  );
}
