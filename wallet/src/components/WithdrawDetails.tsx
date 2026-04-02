import type { WalletPreviewStateId } from "../data/walletPreviewStates";
import type { WalletWithdrawDraft } from "../types/wallet";
import { ActionField } from "./ActionField";
import { ActionSummaryCard } from "./ActionSummaryCard";
import { DraftProgress } from "./DraftProgress";

interface WithdrawAssetOption {
  id: string;
  label: string;
  balanceLabel: string;
}

interface WithdrawDetailsProps {
  latestWithdrawReference: string;
  pendingActivityCount: number;
  scriptAddressShort: string;
  topAssetLabel: string;
  draft?: WalletWithdrawDraft;
  assetOptions?: WithdrawAssetOption[];
  onDraftChange?: (draft: WalletWithdrawDraft) => void;
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

export function WithdrawDetails({
  latestWithdrawReference,
  pendingActivityCount,
  scriptAddressShort,
  topAssetLabel,
  draft,
  assetOptions = [],
  onDraftChange,
  isEmpty = false,
}: WithdrawDetailsProps) {
  if (!draft || !onDraftChange) {
    return (
      <div className="wallet-action-layout">
        <div className="wallet-action-rail">
          <ActionSummaryCard
            eyebrow="Withdraw status"
            title={isEmpty ? "Withdraw is unavailable" : "Prepare settlement"}
            description={
              isEmpty
                ? "Withdraw is unavailable because there is no settlement history or active inventory loaded yet."
                : "Withdrawal prep keeps the latest settlement reference, queue pressure, and active script context visible while notes are assembled for on-chain settlement."
            }
            tone={isEmpty ? "warning" : "neutral"}
          />
        </div>

        <div className="wallet-action-body">
          <div className="wallet-meta-grid">
            <ActionField
              label="Latest withdraw ref"
              value={isEmpty ? "No withdraw reference yet" : latestWithdrawReference}
            />
            <ActionField
              label="Pending queue"
              value={
                isEmpty
                  ? "0 items"
                  : `${pendingActivityCount} item${pendingActivityCount === 1 ? "" : "s"}`
              }
            />
            <ActionField
              label="Settlement path"
              value={isEmpty ? "Unavailable" : scriptAddressShort}
            />
            <ActionField
              label="Primary asset in view"
              value={isEmpty ? "No holdings loaded" : topAssetLabel}
            />
          </div>
        </div>
      </div>
    );
  }

  const selectedAsset = assetOptions.find((option) => option.id === draft.assetId) ?? null;
  const amount = parsePositiveAmount(draft.amountInput);
  const destinationAddress = draft.destinationAddress.trim();
  const isReady = Boolean(selectedAsset && amount !== null && destinationAddress);
  const missingRequirements = [
    !selectedAsset ? "Select an asset" : null,
    amount === null ? "Enter a valid amount" : null,
    !destinationAddress ? "Add a destination address" : null,
  ].filter((item): item is string => item !== null);
  const completedCount = [
    Boolean(selectedAsset),
    amount !== null,
    Boolean(destinationAddress),
  ].filter(Boolean).length;

  const summaryTitle = isReady
    ? `Ready to review ${draft.amountInput.trim()} ${selectedAsset?.label ?? "withdrawal"}`
    : "Finish the withdrawal draft";
  const summaryDescription = isReady
    ? `Prepare settlement from ${scriptAddressShort} to ${destinationAddress}. Latest reference ${latestWithdrawReference} and ${pendingActivityCount} pending queue item${pendingActivityCount === 1 ? "" : "s"} remain visible before handoff.`
    : missingRequirements.join(" • ");

  return (
    <div className="wallet-action-layout">
      <div className="wallet-action-rail">
        <ActionSummaryCard
          eyebrow="Withdraw draft"
          title={summaryTitle}
          description={summaryDescription}
          tone={isReady ? "positive" : "warning"}
          footer={
            <button
              type="button"
              disabled={!isReady}
              className="wallet-interactive w-full rounded-2xl border border-teal-300/30 bg-teal-400/10 px-4 py-3 text-base font-medium text-teal-50 disabled:cursor-not-allowed disabled:border-white/10 disabled:bg-white/[0.03] disabled:text-slate-500 disabled:active:scale-100"
            >
              Review withdrawal
            </button>
          }
        />

        <DraftProgress
          label="Draft progress"
          completed={completedCount}
          total={3}
          summary="Withdrawal prep keeps the mandatory settlement details visible first so the last review step stays short."
          tone={isReady ? "positive" : "warning"}
        />
      </div>

      <div className="wallet-action-body">
        <div className="grid gap-3 sm:grid-cols-2">
          <label className="grid gap-2 text-base text-slate-200">
            <span className="wallet-kicker text-slate-500">Settlement asset</span>
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
            <span className="wallet-kicker text-slate-500">Destination address</span>
            <input
              type="text"
              value={draft.destinationAddress}
              onChange={(event) =>
                onDraftChange({
                  ...draft,
                  destinationAddress: event.target.value,
                })
              }
              placeholder="addr..."
              className="wallet-input wallet-code"
            />
          </label>

          <label className="grid gap-2 text-base text-slate-200 sm:col-span-2">
            <span className="wallet-kicker text-slate-500">Reference</span>
            <input
              type="text"
              value={draft.reference}
              onChange={(event) =>
                onDraftChange({
                  ...draft,
                  reference: event.target.value,
                })
              }
              placeholder="Settlement note or treasury tag"
              className="wallet-input"
            />
          </label>
        </div>

        <div className="wallet-meta-grid">
          <ActionField label="Latest withdraw ref" value={latestWithdrawReference} />
          <ActionField label="Settlement path" value={scriptAddressShort} />
          <ActionField
            label="Selected balance"
            value={selectedAsset?.balanceLabel ?? "Asset not selected"}
          />
          <ActionField label="Primary asset in view" value={topAssetLabel} />
        </div>
      </div>
    </div>
  );
}
