import type { WalletNoteView } from "../lib/walletView";
import { NoteRow } from "./NoteRow";

interface NotesPanelProps {
  notes: WalletNoteView[];
}

export function NotesPanel({ notes }: NotesPanelProps) {
  return (
    <section className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Notes inventory
          </p>
          <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-50">
            Private notes stay legible without flattening their state
          </h2>
        </div>
        <p className="max-w-xl text-sm leading-6 text-slate-400">
          Note rows keep amount, source, timing, and compact cryptographic
          references visible while making availability state easy to scan.
        </p>
      </div>

      <div className="mt-5 grid gap-3">
        {notes.map((note) => (
          <NoteRow key={note.id} note={note} />
        ))}
      </div>
    </section>
  );
}
