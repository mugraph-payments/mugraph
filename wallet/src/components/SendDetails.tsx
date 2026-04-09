import { CheckCircle, Plus, QrCode, Sparkle, X } from "@phosphor-icons/react";
import { motion, useReducedMotion } from "framer-motion";
import { useState } from "react";
import type { WalletSendDraft, WalletSendEntry } from "../types/wallet";

type SendStep = "form" | "qr" | "confirmed";

interface SendAssetOption {
  id: string;
  label: string;
  balanceLabel: string;
}

interface SendDetailsProps {
  draft: WalletSendDraft;
  assetOptions: SendAssetOption[];
  onDraftChange: (draft: WalletSendDraft) => void;
}

const successMessages = [
  "Quiet handoff complete.",
  "Transfer delivered without fuss.",
  "That note is on its way.",
];

function isEntryValid(entry: WalletSendEntry): boolean {
  if (!entry.assetId) return false;
  const n = Number(entry.amountInput.trim());
  return Number.isFinite(n) && n > 0;
}

/** Extract the numeric portion from a formatted balance like "12,483.54 ADA" */
function maxFromLabel(label: string): string {
  const num = label.split(" ")[0]?.replace(/,/g, "") ?? "0";
  return num;
}

function QrPlaceholder({ lines }: { lines: string[] }) {
  return (
    <div className="flex flex-col items-center gap-4 py-2">
      <div
        className="wallet-soft-float flex h-48 w-48 items-center justify-center rounded-2xl border border-white/10 bg-white p-3"
        aria-label="Transaction QR code"
      >
        <svg viewBox="0 0 21 21" className="h-full w-full" shapeRendering="crispEdges">
          <rect width="21" height="21" fill="white" />
          <rect x="0" y="0" width="7" height="7" fill="black" />
          <rect x="1" y="1" width="5" height="5" fill="white" />
          <rect x="2" y="2" width="3" height="3" fill="black" />
          <rect x="14" y="0" width="7" height="7" fill="black" />
          <rect x="15" y="1" width="5" height="5" fill="white" />
          <rect x="16" y="2" width="3" height="3" fill="black" />
          <rect x="0" y="14" width="7" height="7" fill="black" />
          <rect x="1" y="15" width="5" height="5" fill="white" />
          <rect x="2" y="16" width="3" height="3" fill="black" />
          <rect x="8" y="0" width="1" height="1" fill="black" />
          <rect x="10" y="0" width="1" height="1" fill="black" />
          <rect x="8" y="2" width="1" height="1" fill="black" />
          <rect x="10" y="2" width="1" height="1" fill="black" />
          <rect x="12" y="2" width="1" height="1" fill="black" />
          <rect x="8" y="4" width="1" height="1" fill="black" />
          <rect x="11" y="4" width="1" height="1" fill="black" />
          <rect x="9" y="6" width="1" height="1" fill="black" />
          <rect x="11" y="6" width="1" height="1" fill="black" />
          <rect x="0" y="8" width="1" height="1" fill="black" />
          <rect x="2" y="8" width="1" height="1" fill="black" />
          <rect x="4" y="8" width="1" height="1" fill="black" />
          <rect x="6" y="8" width="1" height="1" fill="black" />
          <rect x="9" y="8" width="1" height="1" fill="black" />
          <rect x="11" y="8" width="1" height="1" fill="black" />
          <rect x="14" y="8" width="1" height="1" fill="black" />
          <rect x="16" y="8" width="1" height="1" fill="black" />
          <rect x="18" y="8" width="1" height="1" fill="black" />
          <rect x="20" y="8" width="1" height="1" fill="black" />
          <rect x="8" y="9" width="1" height="1" fill="black" />
          <rect x="10" y="9" width="1" height="1" fill="black" />
          <rect x="14" y="9" width="1" height="1" fill="black" />
          <rect x="17" y="9" width="1" height="1" fill="black" />
          <rect x="9" y="10" width="1" height="1" fill="black" />
          <rect x="12" y="10" width="1" height="1" fill="black" />
          <rect x="15" y="10" width="1" height="1" fill="black" />
          <rect x="18" y="10" width="1" height="1" fill="black" />
          <rect x="20" y="10" width="1" height="1" fill="black" />
          <rect x="8" y="11" width="1" height="1" fill="black" />
          <rect x="11" y="11" width="1" height="1" fill="black" />
          <rect x="14" y="11" width="1" height="1" fill="black" />
          <rect x="16" y="11" width="1" height="1" fill="black" />
          <rect x="19" y="11" width="1" height="1" fill="black" />
          <rect x="8" y="12" width="1" height="1" fill="black" />
          <rect x="10" y="12" width="1" height="1" fill="black" />
          <rect x="12" y="12" width="1" height="1" fill="black" />
          <rect x="15" y="12" width="1" height="1" fill="black" />
          <rect x="17" y="12" width="1" height="1" fill="black" />
          <rect x="20" y="12" width="1" height="1" fill="black" />
          <rect x="9" y="14" width="1" height="1" fill="black" />
          <rect x="11" y="14" width="1" height="1" fill="black" />
          <rect x="14" y="14" width="1" height="1" fill="black" />
          <rect x="17" y="14" width="1" height="1" fill="black" />
          <rect x="19" y="14" width="1" height="1" fill="black" />
          <rect x="8" y="16" width="1" height="1" fill="black" />
          <rect x="10" y="16" width="1" height="1" fill="black" />
          <rect x="15" y="16" width="1" height="1" fill="black" />
          <rect x="18" y="16" width="1" height="1" fill="black" />
          <rect x="9" y="18" width="1" height="1" fill="black" />
          <rect x="12" y="18" width="1" height="1" fill="black" />
          <rect x="14" y="18" width="1" height="1" fill="black" />
          <rect x="16" y="18" width="1" height="1" fill="black" />
          <rect x="20" y="18" width="1" height="1" fill="black" />
          <rect x="8" y="20" width="1" height="1" fill="black" />
          <rect x="10" y="20" width="1" height="1" fill="black" />
          <rect x="14" y="20" width="1" height="1" fill="black" />
          <rect x="17" y="20" width="1" height="1" fill="black" />
          <rect x="19" y="20" width="1" height="1" fill="black" />
        </svg>
      </div>
      <div className="text-center">
        {lines.map((line) => (
          <p key={line} className="wallet-data text-sm font-medium text-slate-200">
            {line}
          </p>
        ))}
      </div>
      <p className="text-center text-xs text-slate-400">
        A quick scan keeps the handoff private and tidy.
      </p>
    </div>
  );
}

function SendEntryCard({
  entry,
  index,
  assetOptions,
  usedAssetIds,
  canRemove,
  onUpdate,
  onRemove,
}: {
  entry: WalletSendEntry;
  index: number;
  assetOptions: SendAssetOption[];
  usedAssetIds: Set<string>;
  canRemove: boolean;
  onUpdate: (patch: Partial<WalletSendEntry>) => void;
  onRemove: () => void;
}) {
  const selectedAsset = assetOptions.find((a) => a.id === entry.assetId) ?? null;
  const available = assetOptions.filter((a) => !usedAssetIds.has(a.id) || a.id === entry.assetId);

  function handleMax() {
    if (!selectedAsset) return;
    onUpdate({ amountInput: maxFromLabel(selectedAsset.balanceLabel) });
  }

  return (
    <div className="wallet-subtle-card relative space-y-3 p-4 sm:p-5">
      {canRemove ? (
        <button
          type="button"
          onClick={onRemove}
          className="wallet-interactive absolute right-2 top-2 flex h-8 w-8 items-center justify-center rounded-lg text-slate-500 hover:text-rose-300"
          aria-label={`Remove asset ${index + 1}`}
        >
          <X className="h-3.5 w-3.5" weight="bold" />
        </button>
      ) : null}

      <div>
        <span className="text-xs font-medium text-slate-400">Asset</span>
        <select
          value={entry.assetId}
          onChange={(e) => onUpdate({ assetId: e.target.value })}
          className="wallet-input mt-2 w-full text-sm"
        >
          <option value="">Select asset</option>
          {available.map((a) => (
            <option key={a.id} value={a.id}>
              {a.label}
            </option>
          ))}
        </select>
      </div>

      <div>
        <span className="text-xs font-medium text-slate-400">Amount</span>
        <div className="relative mt-2">
          <input
            type="text"
            inputMode="decimal"
            value={entry.amountInput}
            onChange={(e) => onUpdate({ amountInput: e.target.value })}
            placeholder="0.00"
            className="wallet-input wallet-data w-full pr-16 text-sm"
          />
          <button
            type="button"
            onClick={handleMax}
            disabled={!selectedAsset}
            className="wallet-interactive absolute right-2 top-1/2 -translate-y-1/2 rounded-md bg-white/[0.08] px-2.5 py-1 text-xs font-semibold text-slate-200 hover:bg-white/[0.12] hover:text-slate-50 disabled:opacity-30"
          >
            Max
          </button>
        </div>
        {selectedAsset ? (
          <p className="mt-2 text-xs text-slate-400">
            Available{" "}
            <span className="wallet-data font-medium text-slate-300">
              {selectedAsset.balanceLabel}
            </span>
          </p>
        ) : null}
      </div>
    </div>
  );
}

export function SendDetails({ draft, assetOptions, onDraftChange }: SendDetailsProps) {
  const [step, setStep] = useState<SendStep>("form");
  const [confirmedLines, setConfirmedLines] = useState<string[]>([]);
  const [successMessage, setSuccessMessage] = useState(successMessages[0]);
  const prefersReducedMotion = useReducedMotion();
  const { entries } = draft;
  const validEntries = entries.filter(isEntryValid);
  const isReady = validEntries.length > 0;

  function updateEntry(index: number, patch: Partial<WalletSendEntry>) {
    const next = entries.map((e, i) => (i === index ? { ...e, ...patch } : e));
    onDraftChange({ entries: next });
  }

  function removeEntry(index: number) {
    onDraftChange({ entries: entries.filter((_, i) => i !== index) });
  }

  function addEntry() {
    onDraftChange({ entries: [...entries, { assetId: "", amountInput: "" }] });
  }

  function usedAssetIds(exceptIndex: number): Set<string> {
    return new Set(entries.filter((_, i) => i !== exceptIndex).map((e) => e.assetId));
  }

  function buildLines() {
    return validEntries.map((e) => {
      const opt = assetOptions.find((a) => a.id === e.assetId);
      return `${e.amountInput.trim()} ${opt?.label ?? ""}`;
    });
  }

  if (step === "confirmed") {
    return (
      <div className="mx-auto grid w-full max-w-lg gap-5 py-2">
        <div className="flex flex-col items-center gap-3 text-center">
          <motion.div
            initial={prefersReducedMotion ? false : { opacity: 0, scale: 0.82, y: 6 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            transition={{ duration: 0.32, ease: [0.16, 1, 0.3, 1] }}
            className="wallet-success-glow"
          >
            <CheckCircle className="h-12 w-12 text-teal-300" weight="duotone" />
          </motion.div>

          <motion.div
            initial={prefersReducedMotion ? false : { opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.28, delay: 0.05, ease: [0.16, 1, 0.3, 1] }}
            className="grid gap-2"
          >
            <h3 className="text-lg font-semibold text-slate-50">Transfer confirmed</h3>
            <p className="text-sm text-teal-300">{successMessage}</p>
          </motion.div>

          <motion.div
            initial={prefersReducedMotion ? false : { opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.28, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
            className="text-center"
          >
            {confirmedLines.map((line) => (
              <p key={line} className="wallet-data text-sm text-slate-300">
                {line}
              </p>
            ))}
          </motion.div>

          <motion.p
            initial={prefersReducedMotion ? false : { opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.28, delay: 0.14, ease: [0.16, 1, 0.3, 1] }}
            className="text-center text-xs text-slate-400"
          >
            The recipient scanned the code and the transfer has been recorded.
          </motion.p>
        </div>

        <button
          type="button"
          onClick={() => {
            onDraftChange({ entries: [{ assetId: "", amountInput: "" }] });
            setStep("form");
          }}
          className="wallet-interactive wallet-cta-secondary mx-auto rounded-xl border px-6 py-2.5 text-sm font-medium text-slate-200"
        >
          New transfer
        </button>
      </div>
    );
  }

  if (step === "qr" && isReady) {
    const lines = buildLines();
    return (
      <div className="mx-auto grid w-full max-w-lg gap-4 py-2">
        <QrPlaceholder lines={lines} />
        <p className="text-center text-xs text-slate-400">
          Show this code to the recipient, then confirm once they have scanned it.
        </p>
        <button
          type="button"
          onClick={() => {
            setConfirmedLines(lines);
            setSuccessMessage(successMessages[Math.floor(Math.random() * successMessages.length)]);
            setStep("confirmed");
          }}
          className="wallet-interactive wallet-cta-primary flex w-full items-center justify-center gap-2 rounded-xl border px-4 py-3 text-sm font-semibold text-slate-50"
        >
          <CheckCircle className="h-4 w-4" weight="duotone" />
          Confirm transfer
        </button>
        <button
          type="button"
          onClick={() => setStep("form")}
          className="wallet-interactive mx-auto rounded-lg px-4 py-2 text-sm text-slate-400 hover:text-slate-200"
        >
          Edit transaction
        </button>
      </div>
    );
  }

  return (
    <div className="grid w-full max-w-2xl gap-4">
      {entries.map((entry, index) => (
        <SendEntryCard
          key={index}
          entry={entry}
          index={index}
          assetOptions={assetOptions}
          usedAssetIds={usedAssetIds(index)}
          canRemove={entries.length > 1}
          onUpdate={(patch) => updateEntry(index, patch)}
          onRemove={() => removeEntry(index)}
        />
      ))}

      <div className="flex flex-wrap items-center justify-between gap-3 pt-1">
        {entries.length < assetOptions.length ? (
          <button
            type="button"
            onClick={addEntry}
            className="wallet-interactive flex w-fit items-center gap-1.5 rounded-lg px-2 py-1.5 text-xs font-medium text-slate-400 hover:text-slate-200"
          >
            <Plus className="h-3.5 w-3.5" weight="bold" />
            Add asset
          </button>
        ) : (
          <span />
        )}

        <button
          type="button"
          disabled={!isReady}
          onClick={() => setStep("qr")}
          className="wallet-interactive wallet-cta-primary flex min-w-[13rem] items-center justify-center gap-2 rounded-xl border px-4 py-3 text-sm font-semibold text-slate-50 disabled:cursor-not-allowed disabled:opacity-40 disabled:active:scale-100"
        >
          <QrCode className="h-4 w-4" weight="duotone" />
          Generate QR code
        </button>
      </div>

      <p className="flex items-center gap-2 text-xs text-slate-500">
        <Sparkle className="h-3.5 w-3.5 text-slate-400" weight="fill" />
        We keep the handoff short so the transfer feels instant in person.
      </p>
    </div>
  );
}
