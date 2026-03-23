import type { WalletPreviewStateId } from "../data/walletPreviewStates";
import type { WalletReceiveDraft } from "../types/wallet";
import { ActionField } from "./ActionField";
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
      <div className="mt-4 grid gap-4">
        <ActionSummaryCard
          eyebrow="Receive task"
          title={isEmpty ? "Receive is unavailable" : `Share ${label}`}
          description={
            isEmpty
              ? `${label} has no active wallet context loaded yet.`
              : `Share the active funding target and delegate context when receiving into ${label}.`
          }
          tone={isEmpty ? "warning" : "neutral"}
        />

        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <ActionField
            label="Funding target"
            value={isEmpty ? "Unavailable" : scriptAddressShort}
          />
          <ActionField
            label="Delegate"
            value={isEmpty ? "Unavailable" : delegatePkShort}
          />
          <ActionField label="Network" value={networkLabel} />
          <ActionField label="Last sync" value={lastSyncedRelative} />
        </div>
      </div>
    );
  }

  const selectedAsset =
    assetOptions.find((option) => option.id === draft.assetId) ?? null;
  const requestedAmount = parsePositiveAmount(draft.requestedAmountInput);
  const requestLabel = draft.requestLabel.trim();
  const isReady = Boolean(selectedAsset && requestLabel);
  const shareModeLabel = draft.shareMode === "qr" ? "QR request" : "Address request";
  const summaryTitle = isReady
    ? `Ready to generate ${shareModeLabel.toLowerCase()}`
    : "Finish the receive task";
  const summaryDescription = isReady
    ? `${label} can share ${selectedAsset?.label ?? "the selected asset"}${requestedAmount !== null ? ` for ${draft.requestedAmountInput.trim()}` : " with an open amount"} using ${shareModeLabel.toLowerCase()}.`
    : [
        !selectedAsset ? "Select an asset" : null,
        !requestLabel ? "Add a request label" : null,
        draft.requestedAmountInput.trim() && requestedAmount === null
          ? "Enter a valid requested amount"
          : null,
      ]
        .filter((item): item is string => item !== null)
        .join(" • ");

  return (
    <div className="mt-4 grid gap-4">
      <ActionSummaryCard
        eyebrow="Receive task"
        title={summaryTitle}
        description={summaryDescription}
        tone={isReady ? "positive" : "warning"}
        footer={
          <button
            type="button"
            disabled={!isReady}
            className="w-full rounded-[1rem] border border-teal-300/30 bg-teal-400/10 px-4 py-3 text-sm font-medium text-teal-50 disabled:cursor-not-allowed disabled:border-white/10 disabled:bg-white/[0.03] disabled:text-slate-500"
          >
            Generate request
          </button>
        }
      />

      <div className="grid gap-3 sm:grid-cols-2">
        <label className="grid gap-2 text-sm text-slate-200">
          <span className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
            Asset
          </span>
          <select
            value={draft.assetId}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                assetId: event.target.value,
              })
            }
            className="rounded-[1rem] border border-white/10 bg-white/[0.03] px-3 py-3 text-sm text-slate-100 outline-none transition focus:border-teal-300/30"
          >
            <option value="">Select an asset</option>
            {assetOptions.map((asset) => (
              <option key={asset.id} value={asset.id}>
                {asset.label}
              </option>
            ))}
          </select>
        </label>

        <label className="grid gap-2 text-sm text-slate-200">
          <span className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
            Requested amount
          </span>
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
            className="rounded-[1rem] border border-white/10 bg-white/[0.03] px-3 py-3 text-sm text-slate-100 outline-none transition placeholder:text-slate-500 focus:border-teal-300/30"
          />
        </label>

        <label className="grid gap-2 text-sm text-slate-200 sm:col-span-2">
          <span className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
            Request label
          </span>
          <input
            type="text"
            value={draft.requestLabel}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                requestLabel: event.target.value,
              })
            }
            placeholder="Invoice, desk top-up, or operator note"
            className="rounded-[1rem] border border-white/10 bg-white/[0.03] px-3 py-3 text-sm text-slate-100 outline-none transition placeholder:text-slate-500 focus:border-teal-300/30"
          />
        </label>

        <fieldset className="grid gap-2 sm:col-span-2">
          <legend className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
            Share mode
          </legend>
          <div className="grid gap-3 sm:grid-cols-2">
            {[
              { id: "address", label: "Address request", copy: "Share the funding target directly." },
              { id: "qr", label: "QR request", copy: "Present a scannable receive payload." },
            ].map((option) => {
              const isSelected = draft.shareMode === option.id;

              return (
                <button
                  key={option.id}
                  type="button"
                  onClick={() =>
                    onDraftChange({
                      ...draft,
                      shareMode: option.id as WalletReceiveDraft["shareMode"],
                    })
                  }
                  className={`rounded-[1.25rem] border p-4 text-left transition-colors ${
                    isSelected
                      ? "border-teal-300/30 bg-teal-400/10"
                      : "border-white/10 bg-white/[0.03]"
                  }`}
                >
                  <p className="text-sm font-medium text-slate-100">{option.label}</p>
                  <p className="mt-2 text-sm leading-6 text-slate-400">{option.copy}</p>
                </button>
              );
            })}
          </div>
        </fieldset>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <ActionField
          label="Funding target"
          value={scriptAddressShort}
        />
        <ActionField label="Delegate" value={delegatePkShort} />
        <ActionField label="Network" value={networkLabel} />
        <ActionField label="Last sync" value={lastSyncedRelative} />
      </div>
    </div>
  );
}
