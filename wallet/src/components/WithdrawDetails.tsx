import type { WalletPreviewStateId } from "../data/walletPreviewStates";
import type { WalletWithdrawDraft } from "../types/wallet";
import { ActionSummaryCard } from "./ActionSummaryCard";

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
      <ActionSummaryCard
        eyebrow="Withdraw"
        title={isEmpty ? "Withdraw unavailable" : "Prepare a withdrawal"}
        description={
          isEmpty
            ? "Load wallet data to prepare settlement."
            : `Settle from ${scriptAddressShort}. Latest ref: ${latestWithdrawReference}.`
        }
        tone={isEmpty ? "warning" : "neutral"}
      />
    );
  }

  const selectedAsset = assetOptions.find((option) => option.id === draft.assetId) ?? null;
  const amount = parsePositiveAmount(draft.amountInput);
  const destinationAddress = draft.destinationAddress.trim();
  const isReady = Boolean(selectedAsset && amount !== null && destinationAddress);
  const summaryTitle = isReady ? "Ready to review" : "Complete the withdrawal";
  const summaryDescription = isReady
    ? `${draft.amountInput.trim()} ${selectedAsset?.label ?? ""} to ${destinationAddress}.`
    : [
        !selectedAsset ? "Select an asset" : null,
        amount === null ? "Enter an amount" : null,
        !destinationAddress ? "Add a destination address" : null,
      ]
        .filter((item): item is string => item !== null)
        .join(" • ");

  return (
    <div className="grid gap-5">
      <ActionSummaryCard
        eyebrow="Withdraw"
        title={summaryTitle}
        description={summaryDescription}
        tone={isReady ? "positive" : "warning"}
        footer={
          <button
            type="button"
            disabled={!isReady}
            className="wallet-interactive wallet-cta-primary w-full rounded-2xl border px-4 py-3 text-base font-medium text-slate-50 disabled:opacity-45 disabled:active:scale-100"
          >
            Review withdrawal
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
          {draft.amountInput.trim() && amount === null ? (
            <p className="wallet-hint text-rose-300">
              Enter a positive amount before reviewing settlement.
            </p>
          ) : null}
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
            placeholder="Optional"
            className="wallet-input"
          />
        </label>
      </div>

      <p className="wallet-meta-note text-slate-500">
        From {scriptAddressShort} · primary asset {topAssetLabel} · {pendingActivityCount} pending ·
        ref {latestWithdrawReference}
      </p>
    </div>
  );
}
