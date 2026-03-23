import type { WalletPreviewStateId } from "../data/walletPreviewStates";
import type { WalletDepositDraft } from "../types/wallet";
import { ActionField } from "./ActionField";
import { ActionSummaryCard } from "./ActionSummaryCard";
import { DraftProgress } from "./DraftProgress";

interface DepositAssetOption {
 id: string;
 label: string;
 balanceLabel: string;
}

interface DepositDetailsProps {
 scriptAddressShort: string;
 delegatePkShort: string;
 latestDepositReference: string;
 pendingActivityCount: number;
 draft?: WalletDepositDraft;
 assetOptions?: DepositAssetOption[];
 onDraftChange?: (draft: WalletDepositDraft) => void;
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

export function DepositDetails({
 scriptAddressShort,
 delegatePkShort,
 latestDepositReference,
 pendingActivityCount,
 draft,
 assetOptions = [],
 onDraftChange,
 isEmpty = false,
}: DepositDetailsProps) {
 if (!draft || !onDraftChange) {
  return (
   <div className="mt-4 grid gap-4">
    <ActionSummaryCard
     eyebrow="Deposit tracking"
     title={isEmpty ? "Deposit is unavailable" : "Track wallet funding"}
     description={
      isEmpty
       ? "Deposit tracking is unavailable because there is no funding target or activity history loaded yet."
       : "Deposit tracking keeps the funding target, delegate context, and latest on-chain deposit reference in one place while new notes move into the wallet inventory."
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
     <ActionField
      label="Latest deposit ref"
      value={isEmpty ? "No deposit reference yet" : latestDepositReference}
     />
     <ActionField
      label="Pending activity"
      value={isEmpty ? "0 items" : `${pendingActivityCount} item${pendingActivityCount === 1 ? "" : "s"}`}
     />
    </div>
   </div>
  );
 }

 const selectedAsset =
  assetOptions.find((option) => option.id === draft.assetId) ?? null;
 const amount = parsePositiveAmount(draft.amountInput);
 const sourceAddress = draft.sourceAddress.trim();
 const isReady = Boolean(selectedAsset && amount !== null && sourceAddress);
 const missingRequirements = [
  !selectedAsset ? "Select an asset" : null,
  amount === null ? "Enter a valid amount" : null,
  !sourceAddress ? "Add a source address" : null,
 ].filter((item): item is string => item !== null);
 const completedCount = [Boolean(selectedAsset), amount !== null, Boolean(sourceAddress)].filter(
  Boolean,
 ).length;

 const summaryTitle = isReady
  ? `Ready to track ${draft.amountInput.trim()} ${selectedAsset?.label ?? "deposit"}`
  : "Finish the deposit draft";
 const summaryDescription = isReady
  ? `Track a funding request from ${sourceAddress} into ${scriptAddressShort}. Latest reference ${latestDepositReference} stays visible while the queue has ${pendingActivityCount} pending item${pendingActivityCount === 1 ? "" : "s"}.`
  : missingRequirements.join(" • ");

 return (
  <div className="mt-4 grid gap-4">
   <ActionSummaryCard
    eyebrow="Deposit draft"
    title={summaryTitle}
    description={summaryDescription}
    tone={isReady ? "positive" : "warning"}
    footer={
     <button
      type="button"
      disabled={!isReady}
      className="wallet-interactive w-full rounded-2xl border border-teal-300/30 bg-teal-400/10 px-4 py-3 text-base font-medium text-teal-50 disabled:cursor-not-allowed disabled:border-white/10 disabled:bg-white/[0.03] disabled:text-slate-500 disabled:active:scale-100"
     >
      Track deposit
     </button>
    }
   />

   <DraftProgress
    label="Draft progress"
    completed={completedCount}
    total={3}
    summary="Deposit setup keeps the required chain details visible first, then leaves optional reference notes for later."
    tone={isReady ? "positive" : "warning"}
   />

   <div className="grid gap-3 sm:grid-cols-2">
    <label className="grid gap-2 text-base text-slate-200">
     <span className="wallet-kicker text-slate-500">Funding asset</span>
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
     <span className="wallet-kicker text-slate-500">Source address</span>
     <input
      type="text"
      value={draft.sourceAddress}
      onChange={(event) =>
       onDraftChange({
        ...draft,
        sourceAddress: event.target.value,
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
      placeholder="Operator note or settlement tag"
      className="wallet-input"
     />
    </label>
   </div>

   <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
    <ActionField label="Funding target" value={scriptAddressShort} />
    <ActionField label="Delegate" value={delegatePkShort} />
    <ActionField label="Latest deposit ref" value={latestDepositReference} />
    <ActionField
     label="Selected balance"
     value={selectedAsset?.balanceLabel ?? "Asset not selected"}
    />
   </div>
  </div>
 );
}
