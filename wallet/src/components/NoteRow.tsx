import type { WalletNoteView } from "../lib/walletView";
import { NoteStatusBadge } from "./NoteStatusBadge";

interface NoteRowProps {
  note: WalletNoteView;
}

export function NoteRow({ note }: NoteRowProps) {
  return (
    <article className="wallet-card p-4">
      <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-start">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <span className="wallet-kicker rounded-full border border-white/10 bg-white/[0.04] px-2.5 py-1 text-slate-200">
              {note.assetTicker}
            </span>
            <span className="text-base text-slate-400">{note.sourceLabel}</span>
          </div>

          <div className="mt-3 grid gap-3 sm:grid-cols-4">
            <div>
              <p className="wallet-kicker text-slate-500">Amount</p>
              <p className="wallet-data mt-1 text-xl font-semibold text-slate-50">
                {note.amountLabel}
              </p>
            </div>
            <div>
              <p className="wallet-kicker text-slate-500">Created</p>
              <p className="mt-1 text-base text-slate-100">{note.createdAtRelative}</p>
            </div>
            <div className="min-w-0">
              <p className="wallet-kicker text-slate-500">Nonce</p>
              <p
                className="wallet-code mt-1 truncate text-base text-slate-100"
                title={note.nonceShort}
              >
                {note.nonceShort}
              </p>
            </div>
            <div className="min-w-0">
              <p className="wallet-kicker text-slate-500">Signature</p>
              <p
                className="wallet-code mt-1 truncate text-base text-slate-100"
                title={note.signatureShort}
              >
                {note.signatureShort}
              </p>
            </div>
          </div>
        </div>

        <div className="flex justify-start lg:justify-end">
          <NoteStatusBadge label={note.statusLabel} tone={note.statusTone} />
        </div>
      </div>
    </article>
  );
}
