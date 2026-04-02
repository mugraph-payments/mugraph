import { ArrowSquareIn, ArrowSquareOut } from "@phosphor-icons/react";
import type {
  WalletActivityView,
  WalletAssetView,
  WalletSummaryMetricView,
} from "../lib/walletView";
import type { WalletActionKind } from "../types/wallet";
import { ActivityRow } from "./ActivityRow";
import { AssetRow } from "./AssetRow";

interface WalletHomeScreenProps {
  summaryMetrics: WalletSummaryMetricView[];
  assets: WalletAssetView[];
  activity: WalletActivityView[];
  onPrimaryActionSelect: (actionId: Extract<WalletActionKind, "send" | "receive">) => void;
}

function findMetric(metrics: WalletSummaryMetricView[], id: WalletSummaryMetricView["id"]) {
  return metrics.find((metric) => metric.id === id);
}

export function WalletHomeScreen({
  summaryMetrics,
  assets,
  activity,
  onPrimaryActionSelect,
}: WalletHomeScreenProps) {
  const totalAda = findMetric(summaryMetrics, "total-value-ada");
  const totalUsd = findMetric(summaryMetrics, "total-value-usd");
  const topAssets = assets.slice(0, 3);
  const recentActivity = activity.slice(0, 3);

  return (
    <section className="grid gap-4 self-start lg:grid-cols-[minmax(0,1.15fr)_minmax(20rem,0.85fr)]">
      {/* ── Balance hero ──────────────────────────────── */}
      <section className="wallet-panel p-5 sm:p-6 lg:col-span-2">
        <div className="flex flex-col gap-5 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="wallet-kicker text-slate-500">Total balance</p>
            <h2 className="wallet-heading mt-1 text-3xl font-semibold tracking-tight text-slate-50 sm:text-4xl">
              {totalAda?.value ?? "0 ADA"}
            </h2>
          </div>

          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => onPrimaryActionSelect("send")}
              className="wallet-interactive wallet-cta-primary flex flex-1 items-center justify-center gap-2 rounded-xl border px-5 py-2.5 text-sm font-semibold text-slate-50 sm:flex-none"
            >
              <ArrowSquareOut className="h-4 w-4" weight="duotone" />
              Send
            </button>
            <button
              type="button"
              onClick={() => onPrimaryActionSelect("receive")}
              className="wallet-interactive wallet-cta-secondary flex flex-1 items-center justify-center gap-2 rounded-xl border px-5 py-2.5 text-sm font-semibold text-slate-50 sm:flex-none"
            >
              <ArrowSquareIn className="h-4 w-4" weight="duotone" />
              Receive
            </button>
          </div>
        </div>

        <div className="mt-5 flex gap-4 text-sm">
          <span className="text-slate-400">
            <span className="wallet-data font-medium text-slate-200">{assets.length}</span> assets
          </span>
          <span className="text-slate-500">·</span>
          <span className="text-slate-400">
            <span className="wallet-data font-medium text-slate-200">
              {findMetric(summaryMetrics, "note-count")?.value ?? "0"}
            </span>{" "}
            notes
          </span>
          <span className="text-slate-500">·</span>
          <span className="text-slate-400">
            <span className="wallet-data font-medium text-slate-200">
              {findMetric(summaryMetrics, "pending-activity-count")?.value ?? "0"}
            </span>{" "}
            pending
          </span>
        </div>
      </section>

      {/* ── Recent transactions ───────────────────────── */}
      <section className="wallet-panel p-5">
        <div className="flex items-end justify-between gap-3">
          <h3 className="text-sm font-semibold text-slate-50">Transactions</h3>
          <span className="text-xs text-slate-400">{recentActivity.length} items</span>
        </div>
        <div className="mt-3 divide-y divide-white/[0.06]">
          {recentActivity.map((item) => (
            <ActivityRow key={item.id} activity={item} />
          ))}
        </div>
      </section>

      {/* ── Top assets ────────────────────────────────── */}
      <section className="wallet-panel p-5">
        <div className="flex items-end justify-between gap-3">
          <h3 className="text-sm font-semibold text-slate-50">Holdings</h3>
          <span className="text-xs text-slate-400">{topAssets.length} assets</span>
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
