import type { WalletAssetView } from "../lib/walletView";
import { AssetRow } from "./AssetRow";

interface AssetPanelProps {
  assets: WalletAssetView[];
}

export function AssetPanel({ assets }: AssetPanelProps) {
  return (
    <section className="wallet-panel p-5 sm:p-6">
      <div className="flex items-end justify-between gap-3">
        <div className="space-y-1">
          <p className="wallet-kicker text-slate-500">Portfolio</p>
          <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
            Assets
          </h2>
        </div>
        {assets.length > 0 ? (
          <span className="text-sm text-slate-400">
            {assets.length} {assets.length === 1 ? "token" : "tokens"}
          </span>
        ) : null}
      </div>

      {assets.length === 0 ? (
        <div className="mt-6 py-8 text-center">
          <p className="text-sm font-medium text-slate-300">No assets yet</p>
          <p className="mt-1 text-sm text-slate-400">Your token holdings will appear here.</p>
        </div>
      ) : (
        <div className="mt-4 divide-y divide-white/[0.06]" role="list" aria-label="Asset list">
          {assets.map((asset) => (
            <div key={asset.id} role="listitem">
              <AssetRow asset={asset} />
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
