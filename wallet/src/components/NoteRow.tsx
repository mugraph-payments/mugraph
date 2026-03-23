import type { WalletNoteView } from "../lib/walletView";
import { NoteStatusBadge } from "./NoteStatusBadge";

interface NoteRowProps {
  note: WalletNoteView;
}

export function NoteRow({ note }: NoteRowProps) {
  return (
    <article className="wallet-card p-4">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <span className="wallet-kicker rounded-full border border-white/10 bg-white/[0.04] px-3 py-1 text-slate-200">
              {note.assetTicker}
            </span>
            <span className="wallet-copy break-words text-sm text-slate-400">
              {note.sourceLabel}
            </span>
          </div>

          <div className="space-y-1">
            <p className="wallet-kicker text-slate-500">Note amount</p>
            <p className="wallet-data break-words text-xl font-semibold tracking-tight text-slate-50 sm:text-2xl">
              {note.amountLabel}
            </p>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2 lg:justify-end">
          <NoteStatusBadge label={note.statusLabel} tone={note.statusTone} />
          <span className="wallet-kicker text-slate-500">{note.createdAtRelative}</span>
        </div>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <div className="wallet-subtle-card p-3">
          <p className="wallet-kicker text-slate-500">Source</p>
          <p className="mt-2 text-sm text-slate-100">{note.sourceLabel}</p>
        </div>
        <div className="wallet-subtle-card p-3">
          <p className="wallet-kicker text-slate-500">Created</p>
          <p className="mt-2 text-sm text-slate-100">{note.createdAtRelative}</p>
        </div>
        <div className="wallet-subtle-card min-w-0 p-3">
          <p className="wallet-kicker text-slate-500">Nonce</p>
          <p className="wallet-code mt-2 truncate text-sm text-slate-100" title={note.nonceShort}>
            {note.nonceShort}
          </p>
        </div>
        <div className="wallet-subtle-card min-w-0 p-3">
          <p className="wallet-kicker text-slate-500">Signature</p>
          <p
            className="wallet-code mt-2 truncate text-sm text-slate-100"
            title={note.signatureShort}
          >
            {note.signatureShort}
          </p>
        </div>
      </div>
    </article>
  );
}
