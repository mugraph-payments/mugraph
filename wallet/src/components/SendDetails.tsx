import { Plus, QrCode, X } from "@phosphor-icons/react";
import { useState } from "react";
import type { WalletSendDraft, WalletSendEntry } from "../types/wallet";

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
        className="flex h-48 w-48 items-center justify-center rounded-2xl border border-white/10 bg-white p-3"
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
        Scan this code with another wallet to complete the transfer.
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
    <div className="wallet-subtle-card relative space-y-3 p-4">
      {canRemove ? (
        <button
          type="button"
          onClick={onRemove}
          className="wallet-interactive absolute right-2 top-2 flex h-7 w-7 items-center justify-center rounded-lg text-slate-500 hover:text-rose-300"
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
          className="wallet-input mt-1 w-full text-sm"
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
        <div className="relative mt-1">
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
            className="absolute right-2 top-1/2 -translate-y-1/2 rounded-md bg-white/[0.08] px-2.5 py-1 text-xs font-semibold text-slate-200 transition-colors hover:bg-white/[0.12] hover:text-slate-50 disabled:opacity-30"
          >
            Max
          </button>
        </div>
        {selectedAsset ? (
          <p className="mt-1.5 text-xs text-slate-400">
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
  const [showQr, setShowQr] = useState(false);
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

  if (showQr && isReady) {
    const lines = validEntries.map((e) => {
      const opt = assetOptions.find((a) => a.id === e.assetId);
      return `${e.amountInput.trim()} ${opt?.label ?? ""}`;
    });
    return (
      <div className="mx-auto mt-5 grid w-full max-w-md gap-5">
        <QrPlaceholder lines={lines} />
        <button
          type="button"
          onClick={() => setShowQr(false)}
          className="wallet-interactive wallet-cta-secondary mx-auto rounded-xl border px-6 py-2.5 text-sm font-medium text-slate-200"
        >
          Edit transaction
        </button>
      </div>
    );
  }

  return (
    <div className="mx-auto mt-5 grid w-full max-w-md gap-3">
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

      {entries.length < assetOptions.length ? (
        <button
          type="button"
          onClick={addEntry}
          className="wallet-interactive flex w-fit items-center gap-1.5 rounded-lg px-2 py-1.5 text-xs font-medium text-slate-400 hover:text-slate-200"
        >
          <Plus className="h-3.5 w-3.5" weight="bold" />
          Add asset
        </button>
      ) : null}

      <button
        type="button"
        disabled={!isReady}
        onClick={() => setShowQr(true)}
        className="wallet-interactive wallet-cta-primary mt-1 flex w-full items-center justify-center gap-2 rounded-xl border px-4 py-3 text-sm font-semibold text-slate-50 disabled:cursor-not-allowed disabled:opacity-40 disabled:active:scale-100"
      >
        <QrCode className="h-4 w-4" weight="duotone" />
        Generate QR code
      </button>
    </div>
  );
}
