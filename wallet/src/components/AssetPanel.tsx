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
    <div className="mt-5 rounded-[1.5rem] border border-dashed border-white/10 bg-white/[0.02] p-5">
      <h3 className="text-sm font-medium text-slate-100">{title}</h3>
      <p className="mt-2 max-w-xl text-sm leading-6 text-slate-400">{copy}</p>
    </div>
  );
}

export function AssetPanel({ assets }: AssetPanelProps) {
  return (
    <section className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Asset holdings
          </p>
          <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-50">
            Balances stay dense, readable, and note-aware
          </h2>
        </div>
        <p className="max-w-xl text-sm leading-6 text-slate-400">
          Each row keeps the wallet’s current balance, note count, share of the
          wallet, and short-term trend readable without expanding into a full
          details page.
        </p>
      </div>

      {assets.length === 0 ? (
        <EmptyPanelBody
          title="No assets are loaded into this preview"
          copy="Use the ready preview to inspect holdings. The empty preview keeps the asset lane intentional instead of rendering a blank list."
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
