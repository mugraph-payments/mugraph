import type { WalletAssetView } from "../lib/walletView";

interface AssetRowProps {
  asset: WalletAssetView;
}

export function AssetRow({ asset }: AssetRowProps) {
  return (
    <article className="flex items-center gap-3 py-3.5">
      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-white/[0.05] ring-1 ring-white/10">
        <span className="text-xs font-bold tracking-wide text-slate-200">
          {asset.ticker.slice(0, 3)}
        </span>
      </div>

      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium text-slate-100">{asset.name}</p>
        <p className="mt-0.5 text-xs text-slate-400">{asset.noteCountLabel}</p>
      </div>

      <p className="wallet-data text-sm font-semibold text-slate-100">{asset.balanceLabel}</p>
    </article>
  );
}
