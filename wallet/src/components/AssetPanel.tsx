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
        <p className="wallet-code mt-0.5 text-[0.65rem] text-slate-500">{note.nonceShort}</p>
      </div>
    </article>
  );
}

export function AssetPanel({ assets, notes }: AssetPanelProps) {
  return (
    <section className="wallet-panel p-5 sm:p-6">
      {/* ── Balance summary ─────────────────────── */}
      <div className="space-y-1">
        <p className="wallet-kicker text-slate-500">Holdings</p>
        <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
          Notes
        </h2>
      </div>

      {assets.length > 0 ? (
        <div className="mt-4 flex flex-wrap gap-x-5 gap-y-1 text-sm">
          {assets.map((asset) => (
            <span key={asset.id} className="text-slate-400">
              <span className="wallet-data font-medium text-slate-200">{asset.balanceLabel}</span>
              {" · "}
              {asset.noteCountLabel}
            </span>
          ))}
        </div>
      ) : null}

      {/* ── Note list ───────────────────────────── */}
      {notes.length === 0 ? (
        <div className="mt-6 py-8 text-center">
          <p className="text-sm font-medium text-slate-300">No notes yet</p>
          <p className="mt-1 text-sm text-slate-400">
            Spendable notes from deposits and refreshes will appear here.
          </p>
        </div>
      ) : (
        <div className="mt-4 divide-y divide-white/[0.06]" role="list" aria-label="Note list">
          <div className="flex items-center justify-between pb-2 text-xs text-slate-500">
            <span>
              {notes.length} {notes.length === 1 ? "note" : "notes"}
            </span>
          </div>
          {notes.map((note) => (
            <div key={note.id} role="listitem">
              <NoteRow note={note} />
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
