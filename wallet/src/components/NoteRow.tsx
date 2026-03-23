import type { WalletNoteView } from "../lib/walletView";
import { NoteStatusBadge } from "./NoteStatusBadge";

interface NoteRowProps {
  note: WalletNoteView;
}

export function NoteRow({ note }: NoteRowProps) {
  return (
    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs font-medium uppercase tracking-[0.22em] text-slate-200">
              {note.assetTicker}
            </span>
            <span className="text-sm text-slate-400">{note.sourceLabel}</span>
          </div>
          <p className="text-xl font-semibold tracking-tight text-slate-50">
            {note.amountLabel}
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2 lg:justify-end">
          <NoteStatusBadge label={note.statusLabel} tone={note.statusTone} />
          <span className="text-xs uppercase tracking-[0.22em] text-slate-500">
            {note.createdAtRelative}
          </span>
        </div>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Source
          </p>
          <p className="mt-1 text-sm text-slate-200">{note.sourceLabel}</p>
        </div>
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Created
          </p>
          <p className="mt-1 text-sm text-slate-200">
            {note.createdAtRelative}
          </p>
        </div>
        <div className="min-w-0">
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Nonce
          </p>
          <p className="mt-1 truncate text-sm text-slate-200">{note.nonceShort}</p>
        </div>
        <div className="min-w-0">
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Signature
          </p>
          <p className="mt-1 truncate text-sm text-slate-200">
            {note.signatureShort}
          </p>
        </div>
      </div>
    </div>
  );
}
