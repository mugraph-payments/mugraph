import type { WalletActionDraftView, WalletActionView } from "../lib/walletView";
import type {
 WalletDepositDraft,
 WalletReceiveDraft,
 WalletSendDraft,
 WalletWithdrawDraft,
} from "../types/wallet";
import { ActionField } from "./ActionField";
import { ActionSummaryCard } from "./ActionSummaryCard";
import { DepositDetails } from "./DepositDetails";
import { ReceiveDetails } from "./ReceiveDetails";
import { SendDetails } from "./SendDetails";
import { WithdrawDetails } from "./WithdrawDetails";

interface AssetOption {
 id: string;
 label: string;
 balanceLabel: string;
}

interface ReceiveContext {
 label: string;
 delegatePkShort: string;
 scriptAddressShort: string;
 networkLabel: string;
 lastSyncedRelative: string;
}

interface DepositContext {
 scriptAddressShort: string;
 delegatePkShort: string;
 latestDepositReference: string;
}

interface WithdrawContext {
 latestWithdrawReference: string;
 scriptAddressShort: string;
 topAssetLabel: string;
}

interface WalletActionPanelProps {
 action: WalletActionView;
 draft: WalletActionDraftView;
 sendDraft: WalletSendDraft;
 onSendDraftChange: (draft: WalletSendDraft) => void;
 receiveDraft: WalletReceiveDraft;
 onReceiveDraftChange: (draft: WalletReceiveDraft) => void;
 depositDraft: WalletDepositDraft;
 onDepositDraftChange: (draft: WalletDepositDraft) => void;
 withdrawDraft: WalletWithdrawDraft;
 onWithdrawDraftChange: (draft: WalletWithdrawDraft) => void;
 assetOptions: AssetOption[];
 receiveContext: ReceiveContext;
 depositContext: DepositContext;
 withdrawContext: WithdrawContext;
 noteCount: number;
 pendingActivityCount: number;
}

export function WalletActionPanel({
 action,
 draft,
 sendDraft,
 onSendDraftChange,
 receiveDraft,
 onReceiveDraftChange,
 depositDraft,
 onDepositDraftChange,
 withdrawDraft,
 onWithdrawDraftChange,
 assetOptions,
 receiveContext,
 depositContext,
 withdrawContext,
 noteCount,
 pendingActivityCount,
}: WalletActionPanelProps) {
 const readinessTone = draft.isReady ? "positive" : "warning";
 const readinessTitle = draft.isReady
  ? `${draft.title} draft is ready`
  : `${draft.title} still needs input`;
 const readinessDescription = draft.isReady
  ? `${draft.primaryLabel} can continue with the current draft.`
  : draft.missingRequirements.join(" • ");

 return (
  <section className="wallet-panel p-5">
   <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
    <div>
     <p className="wallet-kicker text-slate-500">Current action</p>
     <h2 className="wallet-heading mt-2 text-2xl font-semibold tracking-tight text-slate-50">
      {action.label}
     </h2>
     <p className="wallet-copy mt-2 text-base leading-7 text-slate-400">
      {action.helper}
     </p>
    </div>
    <span className="wallet-kicker self-start rounded-full border border-white/10 bg-white/[0.03] px-3 py-1 text-slate-300">
     {action.id}
    </span>
   </div>

   {action.id === "send" ? (
    <SendDetails
     draft={sendDraft}
     assetOptions={assetOptions}
     noteCount={noteCount}
     pendingActivityCount={pendingActivityCount}
     onDraftChange={onSendDraftChange}
    />
   ) : action.id === "receive" ? (
    <ReceiveDetails
     label={receiveContext.label}
     delegatePkShort={receiveContext.delegatePkShort}
     scriptAddressShort={receiveContext.scriptAddressShort}
     networkLabel={receiveContext.networkLabel}
     lastSyncedRelative={receiveContext.lastSyncedRelative}
     draft={receiveDraft}
     assetOptions={assetOptions}
     onDraftChange={onReceiveDraftChange}
    />
   ) : action.id === "deposit" ? (
    <DepositDetails
     scriptAddressShort={depositContext.scriptAddressShort}
     delegatePkShort={depositContext.delegatePkShort}
     latestDepositReference={depositContext.latestDepositReference}
     pendingActivityCount={pendingActivityCount}
     draft={depositDraft}
     assetOptions={assetOptions}
     onDraftChange={onDepositDraftChange}
    />
   ) : action.id === "withdraw" ? (
    <WithdrawDetails
     latestWithdrawReference={withdrawContext.latestWithdrawReference}
     pendingActivityCount={pendingActivityCount}
     scriptAddressShort={withdrawContext.scriptAddressShort}
     topAssetLabel={withdrawContext.topAssetLabel}
     draft={withdrawDraft}
     assetOptions={assetOptions}
     onDraftChange={onWithdrawDraftChange}
    />
   ) : (
    <>
     <div className="mt-4 grid gap-4 xl:grid-cols-[minmax(0,1.1fr)_minmax(16rem,0.9fr)]">
      <ActionSummaryCard
       eyebrow="Flow"
       title={draft.title}
       description={draft.helper}
       footer={
        <div className="rounded-2xl border border-white/10 bg-black/20 px-3 py-2 text-sm text-slate-300">
         Primary action: <span className="text-slate-100">{draft.primaryLabel}</span>
        </div>
       }
      />

      <ActionSummaryCard
       eyebrow="Readiness"
       title={readinessTitle}
       description={readinessDescription}
       tone={readinessTone}
      />
     </div>

     <div className="mt-4 grid gap-3 sm:grid-cols-2">
      {draft.fields.map((field) => (
       <ActionField
        key={`${draft.id}-${field.label}`}
        label={field.label}
        value={field.value}
       />
      ))}
     </div>
    </>
   )}
  </section>
 );
}
