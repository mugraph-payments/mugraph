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
    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div className="min-w-0 space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs font-medium uppercase tracking-[0.22em] text-slate-200">
              {asset.ticker}
            </span>
            <span className="text-sm text-slate-400">{asset.name}</span>
          </div>
          <p className="text-xl font-semibold tracking-tight text-slate-50">
            {asset.balanceLabel}
          </p>
        </div>

        <div
          className={`inline-flex items-center gap-2 rounded-full border px-3 py-1 text-xs ${trendClasses[asset.trendTone]}`}
        >
          <TrendIcon className="h-4 w-4" weight="bold" />
          <span>{trend.label}</span>
        </div>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Notes
          </p>
          <p className="mt-1 text-sm text-slate-200">{asset.noteCountLabel}</p>
        </div>
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Share
          </p>
          <p className="mt-1 text-sm text-slate-200">{asset.shareLabel}</p>
        </div>
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Trend
          </p>
          <p className="mt-1 text-sm text-slate-200">{trend.label}</p>
        </div>
      </div>
    </div>
  );
}
