import type { WalletNoteView } from "../lib/walletView";
import { NoteRow } from "./NoteRow";

interface NotesPanelProps {
  notes: WalletNoteView[];
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
      <h3 className="text-sm font-medium text-slate-100">{title}</h3>
      <p className="mt-2 max-w-xl text-sm leading-6 text-slate-400">{copy}</p>
    </div>
  );
}

export function NotesPanel({ notes }: NotesPanelProps) {
  return (
    <section className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-4 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur sm:p-5">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div className="space-y-1">
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Notes
          </p>
          <h2 className="text-xl font-semibold tracking-tight text-slate-50">
            Private note inventory
          </h2>
        </div>
        <p className="max-w-xl text-sm leading-6 text-slate-400">
          Review spendable notes, source state, and compact cryptographic
          references without expanding the inventory lane.
        </p>
      </div>

      {notes.length === 0 ? (
        <EmptyPanelBody
          title="No notes are available"
          copy="This wallet preview has no private notes loaded yet. The inventory lane stays visible so the empty state remains intentional."
        />
      ) : (
        <div className="mt-4 grid gap-3">
          {notes.map((note) => (
            <NoteRow key={note.id} note={note} />
          ))}
        </div>
      )}
    </section>
  );
}
