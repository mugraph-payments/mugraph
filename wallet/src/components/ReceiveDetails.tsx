import type { WalletPreviewStateId } from "../data/walletPreviewStates";
import type { WalletReceiveDraft } from "../types/wallet";
import { ActionSummaryCard } from "./ActionSummaryCard";

interface ReceiveAssetOption {
  id: string;
  label: string;
  balanceLabel: string;
}

interface ReceiveDetailsProps {
  label: string;
  delegatePkShort: string;
  scriptAddressShort: string;
  networkLabel: string;
  lastSyncedRelative: string;
  draft?: WalletReceiveDraft;
  assetOptions?: ReceiveAssetOption[];
  onDraftChange?: (draft: WalletReceiveDraft) => void;
  isEmpty?: boolean;
  previewStateId?: WalletPreviewStateId;
}

function parsePositiveAmount(input: string): number | null {
  const trimmedValue = input.trim();

  if (!trimmedValue) {
    return null;
  }

  const parsedValue = Number(trimmedValue);

  if (!Number.isFinite(parsedValue) || parsedValue <= 0) {
    return null;
  }

  return parsedValue;
}

export function ReceiveDetails({
  label,
  delegatePkShort,
  scriptAddressShort,
  networkLabel,
  lastSyncedRelative,
  draft,
  assetOptions = [],
  onDraftChange,
  isEmpty = false,
}: ReceiveDetailsProps) {
  if (!draft || !onDraftChange) {
    return (
      <ActionSummaryCard
        eyebrow="Receive"
        title={isEmpty ? "Receive unavailable" : `Share ${label}`}
        description={
          isEmpty
            ? "Load a wallet to create a receive request."
            : `Use ${networkLabel} and ${delegatePkShort} to share a funding request.`
        }
        tone={isEmpty ? "warning" : "neutral"}
      />
    );
  }

  const selectedAsset = assetOptions.find((option) => option.id === draft.assetId) ?? null;
  const requestedAmount = parsePositiveAmount(draft.requestedAmountInput);
  const requestLabel = draft.requestLabel.trim();
  const requestedAmountIsSatisfied = !draft.requestedAmountInput.trim() || requestedAmount !== null;
  const isReady = Boolean(selectedAsset && requestLabel && requestedAmountIsSatisfied);
  const shareModeLabel = draft.shareMode === "qr" ? "QR" : "Address";
  const summaryTitle = isReady ? `Ready to share` : "Complete the request";
  const summaryDescription = isReady
    ? `${shareModeLabel} request for ${selectedAsset?.label ?? "the selected asset"}${requestedAmount !== null ? ` · ${draft.requestedAmountInput.trim()}` : ""}.`
    : [
        !selectedAsset ? "Select an asset" : null,
        !requestLabel ? "Add a label" : null,
        draft.requestedAmountInput.trim() && requestedAmount === null
          ? "Enter a valid amount"
          : null,
      ]
        .filter((item): item is string => item !== null)
        .join(" • ");

  return (
    <div className="grid gap-5">
      <ActionSummaryCard
        eyebrow="Receive"
        title={summaryTitle}
        description={summaryDescription}
        tone={isReady ? "positive" : "warning"}
        footer={
          <button
            type="button"
            disabled={!isReady}
            className="wallet-interactive wallet-cta-primary w-full rounded-2xl border px-4 py-3 text-base font-medium text-slate-50 disabled:opacity-45 disabled:active:scale-100"
          >
            Generate request
          </button>
        }
      />

      <div className="grid gap-3 sm:grid-cols-2">
        <label className="grid gap-2 text-base text-slate-200">
          <span className="wallet-kicker text-slate-500">Asset</span>
          <select
            value={draft.assetId}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                assetId: event.target.value,
              })
            }
            className="wallet-input"
          >
            <option value="">Select an asset</option>
            {assetOptions.map((asset) => (
              <option key={asset.id} value={asset.id}>
                {asset.label}
              </option>
            ))}
          </select>
        </label>

        <label className="grid gap-2 text-base text-slate-200">
          <span className="wallet-kicker text-slate-500">Requested amount</span>
          <input
            type="text"
            inputMode="decimal"
            value={draft.requestedAmountInput}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                requestedAmountInput: event.target.value,
              })
            }
            placeholder="Optional"
            aria-invalid={draft.requestedAmountInput.trim() ? requestedAmount === null : undefined}
            className="wallet-input wallet-data"
          />
          {draft.requestedAmountInput.trim() && requestedAmount === null ? (
            <p className="wallet-hint text-rose-300">
              Enter a positive amount or leave this blank.
            </p>
          ) : null}
        </label>

        <label className="grid gap-2 text-base text-slate-200 sm:col-span-2">
          <span className="wallet-kicker text-slate-500">Label</span>
          <input
            type="text"
            value={draft.requestLabel}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                requestLabel: event.target.value,
              })
            }
            placeholder="Invoice or note"
            className="wallet-input"
          />
        </label>

        <fieldset className="grid gap-2 sm:col-span-2">
          <legend className="wallet-kicker text-slate-500">Share mode</legend>
          <div className="grid gap-3 sm:grid-cols-2">
            {[
              { id: "address", label: "Address", copy: scriptAddressShort },
              { id: "qr", label: "QR", copy: `Synced ${lastSyncedRelative}` },
            ].map((option) => {
              const isSelected = draft.shareMode === option.id;

              return (
                <button
                  key={option.id}
                  type="button"
                  aria-pressed={isSelected}
                  onClick={() =>
                    onDraftChange({
                      ...draft,
                      shareMode: option.id as WalletReceiveDraft["shareMode"],
                    })
                  }
                  className={`wallet-choice ${
                    isSelected
                      ? "border-teal-300/30 bg-teal-400/10"
                      : "border-white/10 bg-white/[0.03]"
                  }`}
                >
                  <p className="wallet-section-title text-slate-100">{option.label}</p>
                  <p className="wallet-copy mt-2 break-all text-sm text-slate-400">{option.copy}</p>
                </button>
              );
            })}
          </div>
        </fieldset>
      </div>

      <p className="wallet-meta-note text-slate-500">
        {networkLabel} · delegate {delegatePkShort}
      </p>
    </div>
  );
}
