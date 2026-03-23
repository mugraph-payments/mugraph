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
      <h3 className="wallet-heading text-sm font-medium text-slate-100">{title}</h3>
      <p className="wallet-copy mt-2 max-w-xl text-sm leading-6 text-slate-400">{copy}</p>
    </div>
  );
}

export function NotesPanel({ notes }: NotesPanelProps) {
  return (
    <section className="wallet-panel p-4 sm:p-5">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div className="space-y-1">
          <p className="wallet-kicker text-slate-500">Notes</p>
          <h2 className="wallet-heading text-xl font-semibold tracking-tight text-slate-50">
            Private note inventory
          </h2>
        </div>
        <p className="wallet-copy max-w-xl text-sm leading-6 text-slate-400">
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
