import { CheckCircle, Copy, Sparkle } from "@phosphor-icons/react";
import { motion, useReducedMotion } from "framer-motion";
import { useMemo, useState } from "react";
import * as api from "../lib/api";
import type { WalletNote } from "../types/wallet";

interface SendDetailsProps {
  network: string;
  notes: WalletNote[];
  /** Refresh wallet state after a successful send. */
  onDone: () => Promise<void> | void;
}

const successMessages = [
  "Quiet handoff complete.",
  "Transfer delivered without fuss.",
  "That note is on its way.",
];

function formatLovelace(lovelace: number): string {
  return `${(lovelace / 1_000_000).toLocaleString(undefined, { maximumFractionDigits: 6 })} ADA`;
}

function shortNonce(nonce: string): string {
  return `${nonce.slice(0, 10)}…`;
}

export function SendDetails({ network, notes, onDone }: SendDetailsProps) {
  const available = useMemo(() => notes.filter((n) => n.status === "available"), [notes]);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [envelope, setEnvelope] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const successMessage = useMemo(
    () => successMessages[Math.floor(Math.random() * successMessages.length)],
    [envelope],
  );
  const prefersReducedMotion = useReducedMotion();

  const total = useMemo(
    () => available.filter((n) => selected.has(n.nonce)).reduce((a, n) => a + n.amount, 0),
    [available, selected],
  );

  function toggle(nonce: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(nonce)) next.delete(nonce);
      else next.add(nonce);
      return next;
    });
  }

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (selected.size === 0 || submitting) return;
    setSubmitting(true);
    setError(null);
    try {
      const res = await api.sendNotes({
        network,
        note_nonces: Array.from(selected),
      });
      setEnvelope(res.envelope);
      setSelected(new Set());
      await onDone();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleCopy() {
    if (!envelope) return;
    try {
      await navigator.clipboard.writeText(envelope);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // best-effort; ignore
    }
  }

  if (envelope) {
    return (
      <div className="mx-auto grid w-full max-w-2xl gap-5 py-2" role="status" aria-live="polite">
        <div className="flex flex-col items-center gap-3 text-center">
          <motion.div
            initial={prefersReducedMotion ? false : { opacity: 0, scale: 0.82, y: 6 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            transition={{ duration: 0.32, ease: [0.16, 1, 0.3, 1] }}
            className="wallet-success-glow"
          >
            <CheckCircle className="h-12 w-12 text-teal-300" weight="duotone" />
          </motion.div>
          <h3 className="wallet-heading text-[1.625rem] text-slate-50">Notes ready to share</h3>
          <p className="text-sm text-teal-300">{successMessage}</p>
          <p className="text-center text-xs text-slate-400">
            Paste this envelope into the recipient's Receive → Import tab.
          </p>
        </div>

        <div className="wallet-subtle-card grid gap-3 p-4 sm:p-5">
          <textarea
            readOnly
            value={envelope}
            className="wallet-input wallet-code h-48 resize-none break-all text-xs text-slate-200"
          />
          <button
            type="button"
            onClick={handleCopy}
            className="wallet-interactive wallet-cta-secondary inline-flex w-fit items-center gap-2 rounded-lg border px-3 py-1.5 text-sm font-medium text-slate-200"
          >
            <Copy className="h-3.5 w-3.5" weight="bold" />
            {copied ? "Copied" : "Copy envelope"}
          </button>
        </div>

        <button
          type="button"
          onClick={() => setEnvelope(null)}
          className="wallet-interactive wallet-cta-secondary mx-auto rounded-xl border px-6 py-2.5 text-sm font-medium text-slate-200"
        >
          New transfer
        </button>
      </div>
    );
  }

  return (
    <form className="grid w-full max-w-2xl gap-4" onSubmit={handleSubmit}>
      {available.length === 0 ? (
        <div className="wallet-subtle-card p-4 text-sm text-slate-300">
          No spendable notes. Deposit funds first or wait for a refresh.
        </div>
      ) : (
        <div className="grid gap-2">
          {available.map((note) => {
            const checked = selected.has(note.nonce);
            return (
              <button
                key={note.nonce}
                type="button"
                aria-pressed={checked}
                onClick={() => toggle(note.nonce)}
                className={`wallet-choice grid w-full grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 ${
                  checked ? "border-teal-300/30 bg-teal-400/10" : "border-white/10 bg-white/[0.03]"
                }`}
              >
                <span
                  className={`inline-flex h-4 w-4 items-center justify-center rounded border ${
                    checked ? "border-teal-300/60 bg-teal-300/20 text-teal-200" : "border-white/20"
                  }`}
                  aria-hidden="true"
                >
                  {checked ? "✓" : ""}
                </span>
                <span className="wallet-code break-all text-left text-xs text-slate-300">
                  {shortNonce(note.nonce)}
                </span>
                <span className="wallet-data text-right text-sm font-medium text-slate-100">
                  {formatLovelace(note.amount)}
                </span>
              </button>
            );
          })}
        </div>
      )}

      {error ? (
        <p className="wallet-hint text-rose-300" role="alert">
          {error}
        </p>
      ) : null}

      {selected.size > 0 ? (
        <p className="wallet-meta-note text-slate-500">
          Sending {selected.size} note{selected.size === 1 ? "" : "s"} totalling{" "}
          {formatLovelace(total)}
        </p>
      ) : null}

      <button
        type="submit"
        disabled={selected.size === 0 || submitting}
        className="wallet-interactive wallet-cta-primary w-full rounded-2xl border px-4 py-3 text-base font-medium text-slate-50 disabled:opacity-45 disabled:active:scale-100"
      >
        {submitting ? "Building envelope…" : "Send selected notes"}
      </button>

      <p className="wallet-meta-note flex items-center gap-2 text-slate-500">
        <Sparkle className="h-3.5 w-3.5 text-slate-400" weight="fill" />
        Selected notes are marked spent locally as soon as the envelope is built.
      </p>
    </form>
  );
}
