import { ArrowDownRight, ArrowUpRight, Minus } from "@phosphor-icons/react";
import type { WalletAssetView, WalletTone } from "../lib/walletView";

interface AssetRowProps {
  asset: WalletAssetView;
}

const trendStyle: Record<WalletTone, string> = {
  neutral: "text-slate-400",
  positive: "text-teal-300",
  warning: "text-amber-300",
  critical: "text-rose-300",
};

function trendMeta(tone: WalletTone) {
  switch (tone) {
    case "positive":
      return { label: "Up", icon: ArrowUpRight };
    case "warning":
      return { label: "Down", icon: ArrowDownRight };
    default:
      return { label: "Flat", icon: Minus };
  }
}

export function AssetRow({ asset }: AssetRowProps) {
  const trend = trendMeta(asset.trendTone);
  const TrendIcon = trend.icon;

  return (
    <article className="flex items-center gap-3 py-3.5">
      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-white/[0.05] ring-1 ring-white/10">
        <span className="text-xs font-bold tracking-wide text-slate-200">
          {asset.ticker.slice(0, 3)}
        </span>
      </div>

      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium text-slate-100">{asset.name}</p>
        <p className="mt-0.5 text-xs text-slate-400">
          {asset.shareLabel} · {asset.noteCountLabel}
        </p>
      </div>

      <div className="text-right">
        <p className="wallet-data text-sm font-semibold text-slate-100">{asset.balanceLabel}</p>
        <p
          className={`mt-0.5 inline-flex items-center gap-1 text-xs ${trendStyle[asset.trendTone]}`}
        >
          <TrendIcon className="h-3 w-3" weight="bold" />
          {trend.label}
        </p>
      </div>
    </article>
  );
}
