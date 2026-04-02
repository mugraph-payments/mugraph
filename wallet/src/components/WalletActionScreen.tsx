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
  const screenTitle =
    activeAction === "send" ? "Compose a private transfer" : "Share a receive request";
  const screenDescription =
    activeAction === "send"
      ? "Build the transfer first, then hand off the final QR code only when the draft is ready."
      : "Set the asset, label, and handoff mode first so the request is easy to share and verify.";

  return (
    <section className="wallet-panel p-5 sm:p-6 lg:p-7">
      <div className="wallet-section-stack">
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end lg:gap-6">
          <div className="grid gap-4">
            <button
              type="button"
              onClick={onClose}
              className="wallet-interactive inline-flex w-fit items-center gap-1.5 rounded-lg px-2 py-1.5 text-sm text-slate-400 hover:text-slate-200"
            >
              <ArrowLeft className="h-4 w-4" weight="bold" />
              Back
            </button>

            <div className="wallet-section-intro">
              <p className="wallet-kicker text-slate-500">Payment flow</p>
              <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
                {screenTitle}
              </h2>
              <p className="wallet-copy max-w-[42ch] text-base leading-7 text-slate-400">
                {screenDescription}
              </p>
            </div>
          </div>

          <div className="flex gap-1 rounded-xl bg-white/[0.04] p-1">
            <button
              type="button"
              onClick={() => onActionSelect("send")}
              className={`wallet-interactive flex items-center gap-1.5 rounded-lg px-4 py-2 text-sm font-medium ${
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
              className={`wallet-interactive flex items-center gap-1.5 rounded-lg px-4 py-2 text-sm font-medium ${
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
      </div>
    </section>
  );
}
