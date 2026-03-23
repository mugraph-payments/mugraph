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
    <div className="mt-4 rounded-[1.5rem] border border-dashed border-white/10 bg-white/[0.02] p-5">
      <h3 className="wallet-heading text-sm font-medium text-slate-100">{title}</h3>
      <p className="wallet-copy mt-2 max-w-xl text-sm leading-6 text-slate-400">{copy}</p>
    </div>
  );
}

export function AssetPanel({ assets }: AssetPanelProps) {
  return (
    <section className="wallet-panel p-4 sm:p-5">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div className="space-y-1">
          <p className="wallet-kicker text-slate-500">Holdings</p>
          <h2 className="wallet-heading text-xl font-semibold tracking-tight text-slate-50">
            Asset inventory
          </h2>
        </div>
        <p className="wallet-copy max-w-xl text-sm leading-6 text-slate-400">
          Track wallet balances, note density, and balance share without leaving
          the inventory lane.
        </p>
      </div>

      {assets.length === 0 ? (
        <EmptyPanelBody
          title="No assets are loaded"
          copy="This wallet preview has no holdings available yet. The inventory lane stays visible so the empty state still feels intentional."
        />
      ) : (
        <div className="mt-4 grid gap-3">
          {assets.map((asset) => (
            <AssetRow key={asset.id} asset={asset} />
          ))}
        </div>
      )}
    </section>
  );
}
