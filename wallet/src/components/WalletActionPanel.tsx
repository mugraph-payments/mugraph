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
    : `${draft.title} draft needs attention`;
  const readinessDescription = draft.isReady
    ? `${draft.primaryLabel} can continue with the current stub-backed draft.`
    : draft.missingRequirements.join(" • ");

  return (
    <section className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Action panel
          </p>
          <h2 className="mt-2 text-xl font-semibold tracking-tight text-slate-50">
            {action.label} is selected
          </h2>
        </div>
        <span className="self-start rounded-full border border-white/10 bg-white/[0.03] px-3 py-1 text-[11px] uppercase tracking-[0.22em] text-slate-300">
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
              eyebrow="Current flow"
              title={draft.title}
              description={draft.helper}
              footer={
                <div className="rounded-[1.25rem] border border-white/10 bg-slate-950/50 px-3 py-2 text-sm text-slate-300">
                  Primary action: <span className="text-slate-100">{draft.primaryLabel}</span>
                </div>
              }
            />

            <ActionSummaryCard
              eyebrow="Draft status"
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
