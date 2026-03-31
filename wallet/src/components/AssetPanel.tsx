import type { WalletAssetView } from "../lib/walletView";
import { AssetRow } from "./AssetRow";

interface AssetPanelProps {
  assets: WalletAssetView[];
}

export function AssetPanel({ assets }: AssetPanelProps) {
  return (
    <section className="grid content-start gap-4">
      <div className="space-y-1">
        <p className="wallet-kicker text-slate-500">Assets</p>
        <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
          Your assets
        </h2>
        <p className="wallet-copy max-w-2xl text-sm leading-6 text-slate-400">
          Review each asset in a simple wallet list with balance, share, and note count.
        </p>
      </div>

      {assets.length === 0 ? (
        <div className="wallet-panel p-5">
          <h3 className="wallet-heading text-sm font-medium text-slate-100">
            No assets are loaded
          </h3>
          <p className="wallet-copy mt-2 max-w-xl text-sm leading-6 text-slate-400">
            This wallet preview has no holdings available yet.
          </p>
        </div>
      ) : (
        <div className="grid gap-3 xl:grid-cols-2 2xl:grid-cols-3" aria-label="Asset list">
          {assets.map((asset) => (
            <AssetRow key={asset.id} asset={asset} />
          ))}
        </div>
      )}
    </section>
  );
}
