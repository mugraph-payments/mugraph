import { ArrowDownRight, ArrowUpRight, Minus } from "@phosphor-icons/react";
import type { WalletAssetView, WalletTone } from "../lib/walletView";

interface AssetRowProps {
  asset: WalletAssetView;
}

const trendClasses: Record<WalletTone, string> = {
  neutral: "border-white/10 bg-white/5 text-slate-200",
  positive: "border-teal-400/20 bg-teal-400/10 text-teal-100",
  warning: "border-amber-400/20 bg-amber-400/10 text-amber-100",
  critical: "border-rose-400/20 bg-rose-400/10 text-rose-100",
};

function trendMeta(tone: WalletTone) {
  switch (tone) {
    case "positive":
      return {
        label: "Up",
        icon: ArrowUpRight,
      };
    case "warning":
      return {
        label: "Down",
        icon: ArrowDownRight,
      };
    default:
      return {
        label: "Flat",
        icon: Minus,
      };
  }
}

export function AssetRow({ asset }: AssetRowProps) {
  const trend = trendMeta(asset.trendTone);
  const TrendIcon = trend.icon;

  return (
    <article className="wallet-card h-full p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <span className="wallet-kicker rounded-full border border-white/10 bg-white/[0.04] px-2.5 py-1 text-slate-200">
              {asset.ticker}
            </span>
            <span className="text-base text-slate-400">{asset.name}</span>
          </div>

          <p className="wallet-data mt-3 text-xl font-semibold text-slate-50">
            {asset.balanceLabel}
          </p>
        </div>

        <div
          className={`inline-flex w-fit items-center gap-2 rounded-full border px-3 py-1.5 text-xs ${trendClasses[asset.trendTone]}`}
        >
          <TrendIcon className="h-4 w-4" weight="bold" />
          <span className="wallet-data">{trend.label}</span>
        </div>
      </div>

      <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-3">
        <div className="wallet-subtle-card p-3">
          <p className="wallet-kicker text-slate-500">Balance share</p>
          <p className="wallet-data mt-1 text-base text-slate-100">{asset.shareLabel}</p>
        </div>
        <div className="wallet-subtle-card p-3">
          <p className="wallet-kicker text-slate-500">Notes</p>
          <p className="wallet-data mt-1 text-base text-slate-100">{asset.noteCountLabel}</p>
        </div>
        <div className="wallet-subtle-card p-3 col-span-2 sm:col-span-1">
          <p className="wallet-kicker text-slate-500">Trend</p>
          <p className="wallet-data mt-1 text-base text-slate-100">{trend.label}</p>
        </div>
      </div>
    </article>
  );
}
