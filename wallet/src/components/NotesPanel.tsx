import type { WalletNoteView } from "../lib/walletView";
import { NoteRow } from "./NoteRow";

interface NotesPanelProps {
  notes: WalletNoteView[];
}

function EmptyPanelBody({ title, copy }: { title: string; copy: string }) {
  return (
    <div className="wallet-card mt-5 p-5">
      <h3 className="wallet-heading text-sm font-medium text-slate-100">{title}</h3>
      <p className="wallet-copy mt-2 max-w-xl text-sm leading-6 text-slate-400">{copy}</p>
    </div>
  );
}

export function NotesPanel({ notes }: NotesPanelProps) {
  return (
    <section className="wallet-panel p-5 sm:p-6">
      <div className="flex flex-col gap-2 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <p className="wallet-kicker text-slate-500">Notes</p>
          <h2 className="wallet-heading mt-2 text-2xl font-semibold tracking-tight text-slate-50">
            Private note inventory
          </h2>
        </div>
        <p className="wallet-copy max-w-2xl text-sm leading-6 text-slate-400">
          Keep source, status, and cryptographic references readable without making the list feel
          like a debug console.
        </p>
      </div>

      {notes.length === 0 ? (
        <EmptyPanelBody
          title="No notes are available"
          copy="This wallet preview has no private notes loaded yet. The inventory lane stays visible so the empty state remains intentional."
        />
      ) : (
        <div className="mt-5 grid gap-3">
          {notes.map((note) => (
            <NoteRow key={note.id} note={note} />
          ))}
        </div>
      )}
    </section>
  );
}
