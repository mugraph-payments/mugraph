import type { WalletAssetView } from "../lib/walletView";
import { AssetRow } from "./AssetRow";

interface AssetPanelProps {
  assets: WalletAssetView[];
}

function EmptyPanelBody({
  title,
  copy,
}: {
  title: string;
  copy: string;
}) {
  return (
    <div className="wallet-card mt-5 p-5">
      <h3 className="wallet-heading text-sm font-medium text-slate-100">{title}</h3>
      <p className="wallet-copy mt-2 max-w-xl text-sm leading-6 text-slate-400">{copy}</p>
    </div>
  );
}

export function AssetPanel({ assets }: AssetPanelProps) {
  return (
    <section className="wallet-panel p-5 sm:p-6">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="wallet-kicker text-slate-500">Holdings</p>
          <h2 className="wallet-heading mt-2 text-2xl font-semibold tracking-tight text-slate-50">
            Asset inventory
          </h2>
        </div>
        <p className="wallet-copy max-w-2xl text-sm leading-6 text-slate-400">
          View balances, note density, and portfolio share without burying the important numbers in oversized cards.
        </p>
      </div>

      {assets.length === 0 ? (
        <EmptyPanelBody
          title="No assets are loaded"
          copy="This wallet preview has no holdings available yet. The inventory lane stays visible so the empty state still feels intentional."
        />
      ) : (
        <div className="mt-5 grid gap-3">
          {assets.map((asset) => (
            <AssetRow key={asset.id} asset={asset} />
          ))}
        </div>
      )}
    </section>
  );
}
