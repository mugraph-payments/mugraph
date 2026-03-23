import type { WalletNoteView } from "../lib/walletView";
import { NoteStatusBadge } from "./NoteStatusBadge";

interface NoteRowProps {
  note: WalletNoteView;
}

export function NoteRow({ note }: NoteRowProps) {
  return (
    <article className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-full border border-white/10 bg-white/[0.04] px-3 py-1 text-[11px] font-medium uppercase tracking-[0.22em] text-slate-200">
              {note.assetTicker}
            </span>
            <span className="break-words text-sm text-slate-400">
              {note.sourceLabel}
            </span>
          </div>

          <div className="space-y-1">
            <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
              Note amount
            </p>
            <p className="break-words text-xl font-semibold tracking-tight text-slate-50 sm:text-2xl">
              {note.amountLabel}
            </p>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2 lg:justify-end">
          <NoteStatusBadge label={note.statusLabel} tone={note.statusTone} />
          <span className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
            {note.createdAtRelative}
          </span>
        </div>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <div className="rounded-[1rem] border border-white/10 bg-slate-950/50 p-3">
          <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
            Source
          </p>
          <p className="mt-2 text-sm text-slate-100">{note.sourceLabel}</p>
        </div>
        <div className="rounded-[1rem] border border-white/10 bg-slate-950/50 p-3">
          <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
            Created
          </p>
          <p className="mt-2 text-sm text-slate-100">
            {note.createdAtRelative}
          </p>
        </div>
        <div className="min-w-0 rounded-[1rem] border border-white/10 bg-slate-950/50 p-3">
          <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
            Nonce
          </p>
          <p className="mt-2 truncate text-sm text-slate-100" title={note.nonceShort}>
            {note.nonceShort}
          </p>
        </div>
        <div className="min-w-0 rounded-[1rem] border border-white/10 bg-slate-950/50 p-3">
          <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
            Signature
          </p>
          <p
            className="mt-2 truncate text-sm text-slate-100"
            title={note.signatureShort}
          >
            {note.signatureShort}
          </p>
        </div>
      </div>
    </article>
  );
}
