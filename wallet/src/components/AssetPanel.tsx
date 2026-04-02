import type { WalletAssetView, WalletNoteView, WalletTone } from "../lib/walletView";

interface AssetPanelProps {
  assets: WalletAssetView[];
  notes: WalletNoteView[];
}

const statusStyle: Record<WalletTone, string> = {
  neutral: "text-slate-400",
  positive: "text-teal-300",
  warning: "text-amber-300",
  critical: "text-rose-300",
};

function BalanceCard({ asset }: { asset: WalletAssetView }) {
  return (
    <div className="wallet-subtle-card flex flex-col gap-2 p-4">
      <div className="flex items-center gap-2">
        <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-white/[0.06] ring-1 ring-white/10">
          <span className="text-[0.625rem] font-bold tracking-wide text-slate-300">
            {asset.ticker.slice(0, 3)}
          </span>
        </div>
        <span className="text-xs font-medium text-slate-300">{asset.ticker}</span>
      </div>
      <p className="wallet-data text-base font-semibold text-slate-100">{asset.balanceLabel}</p>
      <p className="text-xs text-slate-400">{asset.noteCountLabel}</p>
    </div>
  );
}

function NoteRow({ note }: { note: WalletNoteView }) {
  return (
    <article className="flex items-center gap-3 py-3.5">
      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-white/[0.05] ring-1 ring-white/10">
        <span className="text-xs font-bold tracking-wide text-slate-200">
          {note.assetTicker.slice(0, 3)}
        </span>
      </div>

      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <p className="text-sm font-medium text-slate-100">{note.assetTicker}</p>
          <span className={`text-xs ${statusStyle[note.statusTone]}`}>{note.statusLabel}</span>
        </div>
        <p className="mt-0.5 text-xs text-slate-400">
          {note.sourceLabel} · {note.createdAtRelative}
        </p>
      </div>

      <div className="text-right">
        <p className="wallet-data text-sm font-semibold text-slate-100">{note.amountLabel}</p>
        <p className="wallet-code mt-0.5 text-xs text-slate-500">{note.nonceShort}</p>
      </div>
    </article>
  );
}

export function AssetPanel({ assets, notes }: AssetPanelProps) {
  return (
    <section className="wallet-panel p-5 sm:p-6 lg:p-7">
      <div className="wallet-section-stack">
        <div className="wallet-section-intro">
          <p className="wallet-kicker text-slate-500">Holdings</p>
          <div className="grid gap-2 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end lg:gap-6">
            <div>
              <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
                Notes inventory
              </h2>
              <p className="wallet-copy mt-2 max-w-[42ch] text-base leading-7 text-slate-400">
                Review the assets currently inside the wallet, then inspect the individual notes
                that make each balance spendable.
              </p>
            </div>
            <div className="text-sm text-slate-400 lg:text-right">
              <p>
                <span className="wallet-data font-semibold text-slate-100">{assets.length}</span>{" "}
                assets in view
              </p>
              <p className="mt-1">
                <span className="wallet-data font-semibold text-slate-100">{notes.length}</span>{" "}
                spendable notes
              </p>
            </div>
          </div>
        </div>

        {assets.length > 0 ? (
          <section className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
            {assets.map((asset) => (
              <BalanceCard key={asset.id} asset={asset} />
            ))}
          </section>
        ) : null}

        {notes.length === 0 ? (
          <div className="py-8 text-center">
            <p className="text-sm font-medium text-slate-300">No notes yet</p>
            <p className="mt-1 text-sm text-slate-400">
              Spendable notes from deposits and refreshes will appear here.
            </p>
          </div>
        ) : (
          <section className="grid gap-4">
            <div className="flex items-center justify-between gap-3 text-xs text-slate-500">
              <span>
                {notes.length} {notes.length === 1 ? "note" : "notes"}
              </span>
            </div>
            <div className="wallet-list" role="list" aria-label="Note list">
              {notes.map((note) => (
                <div key={note.id} role="listitem">
                  <NoteRow note={note} />
                </div>
              ))}
            </div>
          </section>
        )}
      </div>
    </section>
  );
}
