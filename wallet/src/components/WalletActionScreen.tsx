import { ArrowLeft, ArrowSquareIn, ArrowSquareOut } from "@phosphor-icons/react";
import type { WalletIdentityView } from "../lib/walletView";
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
  onActionSelect: (actionId: "send" | "receive") => void;
  onClose: () => void;
  sendDraft: WalletSendDraft;
  onSendDraftChange: (draft: WalletSendDraft) => void;
  receiveDraft: WalletReceiveDraft;
  onReceiveDraftChange: (draft: WalletReceiveDraft) => void;
  assetOptions: AssetOption[];
  identity: WalletIdentityView;
}

export function WalletActionScreen({
  activeAction,
  onActionSelect,
  onClose,
  sendDraft,
  onSendDraftChange,
  receiveDraft,
  onReceiveDraftChange,
  assetOptions,
  identity,
}: WalletActionScreenProps) {
  return (
    <section className="wallet-panel p-5 sm:p-6">
      <div className="flex items-center justify-between gap-3">
        <button
          type="button"
          onClick={onClose}
          className="wallet-interactive inline-flex items-center gap-1.5 rounded-lg px-2 py-1.5 text-sm text-slate-400 hover:text-slate-200"
        >
          <ArrowLeft className="h-4 w-4" weight="bold" />
          Back
        </button>

        <div className="flex gap-1 rounded-lg bg-white/[0.04] p-1">
          <button
            type="button"
            onClick={() => onActionSelect("send")}
            className={`wallet-interactive flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium ${
              activeAction === "send"
                ? "bg-white/[0.08] text-slate-50"
                : "text-slate-400 hover:text-slate-200"
            }`}
          >
            <ArrowSquareOut
              className="h-4 w-4"
              weight={activeAction === "send" ? "fill" : "duotone"}
            />
            Send
          </button>
          <button
            type="button"
            onClick={() => onActionSelect("receive")}
            className={`wallet-interactive flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium ${
              activeAction === "receive"
                ? "bg-white/[0.08] text-slate-50"
                : "text-slate-400 hover:text-slate-200"
            }`}
          >
            <ArrowSquareIn
              className="h-4 w-4"
              weight={activeAction === "receive" ? "fill" : "duotone"}
            />
            Receive
          </button>
        </div>
      </div>

      {activeAction === "send" ? (
        <SendDetails
          draft={sendDraft}
          assetOptions={assetOptions}
          onDraftChange={onSendDraftChange}
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
