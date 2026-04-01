import type { WalletPreviewStateId } from "../data/walletPreviewStates";
import type { WalletSendDraft } from "../types/wallet";
import { ActionField } from "./ActionField";
import { ActionSummaryCard } from "./ActionSummaryCard";
import { DraftProgress } from "./DraftProgress";

interface SendAssetOption {
  id: string;
  label: string;
  balanceLabel: string;
}

interface SendDetailsProps {
  draft?: WalletSendDraft;
  assetOptions?: SendAssetOption[];
  noteCount: number;
  pendingActivityCount: number;
  onDraftChange?: (draft: WalletSendDraft) => void;
  topAssetLabel?: string;
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

export function SendDetails({
  draft,
  assetOptions = [],
  noteCount,
  pendingActivityCount,
  onDraftChange,
  topAssetLabel,
  isEmpty = false,
}: SendDetailsProps) {
  if (!draft || !onDraftChange) {
    return (
      <div className="mt-4 grid gap-4">
        <ActionSummaryCard
          eyebrow="Send draft"
          title={isEmpty ? "Send is unavailable" : "Prepare a private transfer"}
          description={
            isEmpty
              ? "Send is unavailable because there are no spendable notes loaded for transfer prep."
              : "Sending starts from the private note inventory. This stub view keeps the operator focused on note availability, the largest holding, and queue pressure before preparing a transfer."
          }
          tone={isEmpty ? "warning" : "neutral"}
        />

        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          <ActionField label="Spendable notes" value={isEmpty ? "0" : `${noteCount}`} />
          <ActionField
            label="Largest holding"
            value={isEmpty ? "No holdings loaded" : (topAssetLabel ?? "No holdings loaded")}
          />
          <ActionField
            label="Queue pressure"
            value={
              isEmpty
                ? "0 pending items"
                : `${pendingActivityCount} pending item${pendingActivityCount === 1 ? "" : "s"}`
            }
          />
          <ActionField
            label="Transfer mode"
            value={isEmpty ? "Unavailable" : "Private note transfer"}
          />
        </div>
      </div>
    );
  }

  const selectedAsset = assetOptions.find((option) => option.id === draft.assetId) ?? null;
  const amount = parsePositiveAmount(draft.amountInput);
  const recipient = draft.recipient.trim();
  const isReady = Boolean(selectedAsset && amount !== null && recipient);
  const missingRequirements = [
    !selectedAsset ? "Select an asset" : null,
    amount === null ? "Enter a valid amount" : null,
    !recipient ? "Add a recipient" : null,
  ].filter((item): item is string => item !== null);
  const completedCount = [Boolean(selectedAsset), amount !== null, Boolean(recipient)].filter(
    Boolean,
  ).length;

  const summaryTitle = isReady
    ? `Ready to review ${draft.amountInput.trim()} ${selectedAsset?.label ?? "transfer"}`
    : "Finish the transfer draft";
  const summaryDescription = isReady
    ? `Prepare a private note transfer to ${recipient}. ${pendingActivityCount} pending queue item${pendingActivityCount === 1 ? "" : "s"} remain visible before handoff.`
    : missingRequirements.join(" • ");

  return (
    <div className="mt-4 grid gap-4">
      <ActionSummaryCard
        eyebrow="Send draft"
        title={summaryTitle}
        description={summaryDescription}
        tone={isReady ? "positive" : "warning"}
        footer={
          <button
            type="button"
            disabled={!isReady}
            className="wallet-interactive w-full rounded-2xl border border-teal-300/30 bg-teal-400/10 px-4 py-3 text-base font-medium text-teal-50 disabled:cursor-not-allowed disabled:border-white/10 disabled:bg-white/[0.03] disabled:text-slate-500 disabled:active:scale-100"
          >
            Review transfer
          </button>
        }
      />

      <DraftProgress
        label="Draft progress"
        completed={completedCount}
        total={3}
        summary="Only the required send fields stay in this first step so the handoff stays easy to review."
        tone={isReady ? "positive" : "warning"}
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
          <span className="wallet-kicker text-slate-500">Amount</span>
          <input
            type="text"
            inputMode="decimal"
            value={draft.amountInput}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                amountInput: event.target.value,
              })
            }
            placeholder="0.00"
            aria-invalid={draft.amountInput.trim() ? amount === null : undefined}
            className="wallet-input wallet-data"
          />
        </label>

        <label className="grid gap-2 text-base text-slate-200 sm:col-span-2">
          <span className="wallet-kicker text-slate-500">Recipient</span>
          <input
            type="text"
            value={draft.recipient}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                recipient: event.target.value,
              })
            }
            placeholder="addr..."
            className="wallet-input wallet-code"
          />
        </label>

        <label className="grid gap-2 text-base text-slate-200 sm:col-span-2">
          <span className="wallet-kicker text-slate-500">Memo</span>
          <textarea
            value={draft.memo}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                memo: event.target.value,
              })
            }
            rows={3}
            placeholder="Optional note for operators"
            className="wallet-input min-h-28 resize-y"
          />
        </label>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <ActionField label="Spendable notes" value={`${noteCount}`} />
        <ActionField
          label="Selected balance"
          value={selectedAsset?.balanceLabel ?? "Asset not selected"}
        />
        <ActionField
          label="Queue pressure"
          value={`${pendingActivityCount} pending item${pendingActivityCount === 1 ? "" : "s"}`}
        />
        <ActionField label="Readiness" value={isReady ? "Ready to review" : "Draft incomplete"} />
      </div>
    </div>
  );
}
