import type { WalletNoteView, WalletTone } from "../lib/walletView";

interface NotesPanelProps {
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
    <article className="flex items-center gap-3 py-3">
      <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-white/[0.05] ring-1 ring-white/10">
        <span className="text-[0.6rem] font-bold tracking-wide text-slate-300">
          {note.assetTicker.slice(0, 3)}
        </span>
      </div>

      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-slate-100">{note.amountLabel}</span>
          <span className={`text-xs ${statusStyle[note.statusTone]}`}>{note.statusLabel}</span>
        </div>
        <p className="mt-0.5 truncate text-xs text-slate-500">
          {note.sourceLabel} · {note.createdAtRelative}
        </p>
      </div>

      <p className="wallet-code hidden truncate text-xs text-slate-500 sm:block sm:max-w-[8rem]">
        {note.nonceShort}
      </p>
    </article>
  );
}

export function NotesPanel({ notes }: NotesPanelProps) {
  return (
    <div className="mt-4">
      <div className="flex items-center justify-between text-xs text-slate-500">
        <span>
          {notes.length} {notes.length === 1 ? "note" : "notes"}
        </span>
      </div>

      {notes.length === 0 ? (
        <div className="py-6 text-center">
          <p className="text-sm text-slate-400">No notes loaded yet.</p>
        </div>
      ) : (
        <div className="divide-y divide-white/[0.06]">
          {notes.map((note) => (
            <NoteRow key={note.id} note={note} />
          ))}
        </div>
      )}
    </div>
  );
}
