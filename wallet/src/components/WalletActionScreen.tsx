import { ArrowLeft, ArrowSquareIn, ArrowSquareOut } from "@phosphor-icons/react";
import type {
  WalletActionView,
  WalletIdentityView,
} from "../lib/walletView";
import type { WalletReceiveDraft, WalletSendDraft } from "../types/wallet";
import { ReceiveDetails } from "./ReceiveDetails";
import { SendDetails } from "./SendDetails";

interface AssetOption {
  id: string;
  label: string;
  balanceLabel: string;
}

interface WalletActionScreenProps {
  activeAction: "send" | "receive";
  actions: WalletActionView[];
  onActionSelect: (actionId: "send" | "receive") => void;
  onClose: () => void;
  sendDraft: WalletSendDraft;
  onSendDraftChange: (draft: WalletSendDraft) => void;
  receiveDraft: WalletReceiveDraft;
  onReceiveDraftChange: (draft: WalletReceiveDraft) => void;
  assetOptions: AssetOption[];
  identity: WalletIdentityView;
  noteCount: number;
  pendingActivityCount: number;
}

const consumerActions: Array<{
  id: "send" | "receive";
  label: string;
  icon: typeof ArrowSquareOut;
}> = [
  {
    id: "send",
    label: "Send",
    icon: ArrowSquareOut,
  },
  {
    id: "receive",
    label: "Receive",
    icon: ArrowSquareIn,
  },
];

export function WalletActionScreen({
  activeAction,
  actions,
  onActionSelect,
  onClose,
  sendDraft,
  onSendDraftChange,
  receiveDraft,
  onReceiveDraftChange,
  assetOptions,
  identity,
  noteCount,
  pendingActivityCount,
}: WalletActionScreenProps) {
  const selectedAction =
    actions.find((action) => action.id === activeAction) ?? actions[0];
  const topAssetLabel = assetOptions[0]?.balanceLabel ?? "No holdings";

  return (
    <section className="wallet-panel p-5 sm:p-6">
      <div className="flex items-center justify-between gap-3">
        <button
          type="button"
          onClick={onClose}
          className="wallet-interactive inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] px-3 py-2 text-sm font-medium text-slate-200"
        >
          <ArrowLeft className="h-4 w-4" weight="bold" />
          Home
        </button>
        <span className="wallet-kicker text-slate-500">Action</span>
      </div>

      <div className="mt-4 space-y-2">
        <p className="wallet-kicker text-slate-500">Choose an action</p>
        <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
          {selectedAction?.label ?? "Action"}
        </h2>
      </div>

      <div className="mt-4 grid grid-cols-2 gap-3">
        {consumerActions.map((action) => {
          const Icon = action.icon;
          const isActive = action.id === activeAction;

          return (
            <button
              key={action.id}
              type="button"
              aria-pressed={isActive}
              onClick={() => onActionSelect(action.id)}
              className={`wallet-interactive flex items-center justify-center gap-2 rounded-2xl border px-4 py-4 text-base font-semibold ${
                isActive
                  ? "border-teal-300/25 bg-teal-400/[0.08] text-teal-50"
                  : "border-white/10 bg-white/[0.04] text-slate-200"
              }`}
            >
              <Icon className="h-5 w-5" weight={isActive ? "fill" : "duotone"} />
              {action.label}
            </button>
          );
        })}
      </div>

      {activeAction === "send" ? (
        <SendDetails
          draft={sendDraft}
          assetOptions={assetOptions}
          noteCount={noteCount}
          pendingActivityCount={pendingActivityCount}
          onDraftChange={onSendDraftChange}
          topAssetLabel={topAssetLabel}
        />
      ) : (
        <ReceiveDetails
          label={identity.label}
          delegatePkShort={identity.delegatePkShort}
          scriptAddressShort={identity.scriptAddressShort}
          networkLabel={identity.networkLabel}
          lastSyncedRelative={identity.lastSyncedRelative}
          draft={receiveDraft}
          assetOptions={assetOptions}
          onDraftChange={onReceiveDraftChange}
        />
      )}
    </section>
  );
}
