import { ArrowSquareIn, ArrowSquareOut } from "@phosphor-icons/react";
import type {
  WalletActivityView,
  WalletAssetView,
  WalletSummaryMetricView,
} from "../lib/walletView";
import type { WalletActionKind } from "../types/wallet";
import { ActivityRow } from "./ActivityRow";

interface WalletHomeScreenProps {
  summaryMetrics: WalletSummaryMetricView[];
  assets: WalletAssetView[];
  activity: WalletActivityView[];
  onPrimaryActionSelect: (actionId: Extract<WalletActionKind, "send" | "receive">) => void;
}

function findMetric(metrics: WalletSummaryMetricView[], id: WalletSummaryMetricView["id"]) {
  return metrics.find((metric) => metric.id === id);
}

function HoldingsPreviewRow({ asset }: { asset: WalletAssetView }) {
  return (
    <div className="flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0">
      <div className="min-w-0">
        <p className="text-sm font-medium text-slate-100">{asset.ticker}</p>
        <p className="mt-0.5 text-xs text-slate-400">{asset.noteCountLabel}</p>
      </div>
      <p className="wallet-data shrink-0 text-sm font-semibold text-slate-100">
        {asset.balanceLabel}
      </p>
    </div>
  );
}

export function WalletHomeScreen({
  summaryMetrics,
  assets,
  activity,
  onPrimaryActionSelect,
}: WalletHomeScreenProps) {
  const totalAda = findMetric(summaryMetrics, "total-value-ada");
  const totalUsd = findMetric(summaryMetrics, "total-value-usd");
  const noteCount = findMetric(summaryMetrics, "note-count");
  const pendingActivityCount = findMetric(summaryMetrics, "pending-activity-count");
  const recentActivity = activity.slice(0, 3);
  const topHoldings = assets.slice(0, 4);

  return (
    <section className="grid gap-6 self-start xl:grid-cols-[minmax(0,1.15fr)_minmax(18rem,0.85fr)]">
      <section className="wallet-panel p-5 sm:p-6 lg:col-span-2">
        <div className="grid gap-5">
          <div className="wallet-section-intro">
            <p className="wallet-kicker text-slate-500">Wallet</p>
            <h2 className="wallet-heading text-3xl font-semibold tracking-tight text-slate-50 sm:text-4xl">
              {totalAda?.value ?? "0 ADA"}
            </h2>
            <p className="text-sm text-slate-400">Ready to send or receive.</p>
          </div>

          <div className="flex flex-wrap gap-3">
            <button
              type="button"
              onClick={() => onPrimaryActionSelect("send")}
              className="wallet-interactive wallet-cta-primary flex flex-1 items-center justify-center gap-2 rounded-xl border px-5 py-3 text-sm font-semibold text-slate-50 sm:flex-none"
            >
              <ArrowSquareOut className="h-4 w-4" weight="duotone" />
              Send
            </button>
            <button
              type="button"
              onClick={() => onPrimaryActionSelect("receive")}
              className="wallet-interactive wallet-cta-secondary flex flex-1 items-center justify-center gap-2 rounded-xl border px-5 py-3 text-sm font-semibold text-slate-50 sm:flex-none"
            >
              <ArrowSquareIn className="h-4 w-4" weight="duotone" />
              Receive
            </button>
          </div>

          <div className="wallet-inline-metrics text-sm text-slate-400">
            <span className="wallet-data text-slate-200">{totalUsd?.value ?? "$0.00"}</span>
            <span className="text-slate-500">•</span>
            <span>
              <span className="wallet-data text-slate-200">{assets.length}</span> assets
            </span>
            <span className="text-slate-500">•</span>
            <span>
              <span className="wallet-data text-slate-200">{noteCount?.value ?? "0"}</span> notes
            </span>
            <span className="text-slate-500">•</span>
            <span>
              <span className="wallet-data text-slate-200">
                {pendingActivityCount?.value ?? "0"}
              </span>{" "}
              pending
            </span>
          </div>
        </div>
      </section>

      <section className="wallet-panel p-5 sm:p-6">
        <div className="flex items-end justify-between gap-3">
          <h3 className="text-sm font-semibold text-slate-50">Recent activity</h3>
          <span className="text-xs text-slate-400">{recentActivity.length}</span>
        </div>

        <div className="wallet-list mt-4">
          {recentActivity.map((item) => (
            <ActivityRow key={item.id} activity={item} />
          ))}
        </div>
      </section>

      <section className="wallet-panel p-5 sm:p-6">
        <div className="flex items-end justify-between gap-3">
          <h3 className="text-sm font-semibold text-slate-50">Assets</h3>
          <span className="text-xs text-slate-400">{assets.length}</span>
        </div>

        <div className="wallet-list mt-4">
          {topHoldings.map((asset) => (
            <HoldingsPreviewRow key={asset.id} asset={asset} />
          ))}
        </div>
      </section>
    </section>
  );
}
