import type { WalletPreviewStateId } from "../data/walletPreviewStates";
import { walletState } from "../data/stubWallet";
import { createWalletView, type WalletActionView } from "../lib/walletView";
import { DepositDetails } from "./DepositDetails";
import { ReceiveDetails } from "./ReceiveDetails";
import { SendDetails } from "./SendDetails";
import { WithdrawDetails } from "./WithdrawDetails";

interface ActionDetailPanelProps {
  action: WalletActionView;
  previewStateId: WalletPreviewStateId;
}

const panelToneClasses: Record<
  WalletPreviewStateId,
  { shell: string; eyebrow: string; badge: string; copy: string }
> = {
  ready: {
    shell: "border-white/10 bg-slate-950/60",
    eyebrow: "text-slate-500",
    badge: "border-white/10 bg-white/5 text-slate-300",
    copy: "text-slate-400",
  },
  empty: {
    shell: "border-white/10 bg-slate-950/60",
    eyebrow: "text-slate-500",
    badge: "border-white/10 bg-white/5 text-slate-300",
    copy: "text-slate-400",
  },
  syncing: {
    shell:
      "border-amber-400/20 bg-[linear-gradient(180deg,rgba(245,158,11,0.08),rgba(2,6,23,0.72))]",
    eyebrow: "text-amber-300/75",
    badge: "border-amber-300/25 bg-amber-400/10 text-amber-50",
    copy: "text-amber-100/80",
  },
  attention: {
    shell: "border-rose-400/20 bg-[linear-gradient(180deg,rgba(244,63,94,0.08),rgba(2,6,23,0.72))]",
    eyebrow: "text-rose-300/75",
    badge: "border-rose-300/25 bg-rose-400/10 text-rose-50",
    copy: "text-rose-100/80",
  },
};

export function ActionDetailPanel({ action, previewStateId }: ActionDetailPanelProps) {
  const view = createWalletView(walletState);
  const latestDeposit = view.activity.find((item) => item.kindLabel === "Deposit") ?? null;
  const latestWithdraw = view.activity.find((item) => item.kindLabel === "Withdraw") ?? null;
  const topAsset = view.assets[0]?.balanceLabel ?? "No holdings";
  const isEmptyPreview = previewStateId === "empty";
  const tone = panelToneClasses[previewStateId];

  return (
    <section
      className={`rounded-[2rem] border p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur ${tone.shell}`}
    >
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className={`text-xs uppercase tracking-[0.22em] ${tone.eyebrow}`}>Detail region</p>
          <h2 className="mt-2 text-xl font-semibold tracking-tight text-slate-50">
            {action.label} is selected
          </h2>
        </div>
        <span
          className={`self-start rounded-full border px-3 py-1 text-[11px] uppercase tracking-[0.22em] ${tone.badge}`}
        >
          {action.id}
        </span>
      </div>

      {action.id === "receive" ? (
        <ReceiveDetails
          label={view.identity.label}
          delegatePkShort={view.identity.delegatePkShort}
          scriptAddressShort={view.identity.scriptAddressShort}
          networkLabel={view.identity.networkLabel}
          lastSyncedRelative={view.identity.lastSyncedRelative}
          isEmpty={isEmptyPreview}
          previewStateId={previewStateId}
        />
      ) : null}

      {action.id === "deposit" ? (
        <DepositDetails
          scriptAddressShort={view.identity.scriptAddressShort}
          delegatePkShort={view.identity.delegatePkShort}
          latestDepositReference={latestDeposit?.referenceShort ?? "No deposit reference"}
          pendingActivityCount={walletState.summary.pendingActivityCount}
          isEmpty={isEmptyPreview}
          previewStateId={previewStateId}
        />
      ) : null}

      {action.id === "send" ? (
        <SendDetails
          noteCount={walletState.summary.noteCount}
          topAssetLabel={topAsset}
          pendingActivityCount={walletState.summary.pendingActivityCount}
          isEmpty={isEmptyPreview}
          previewStateId={previewStateId}
        />
      ) : null}

      {action.id === "withdraw" ? (
        <WithdrawDetails
          latestWithdrawReference={latestWithdraw?.referenceShort ?? "No withdraw reference"}
          pendingActivityCount={walletState.summary.pendingActivityCount}
          scriptAddressShort={view.identity.scriptAddressShort}
          topAssetLabel={topAsset}
          isEmpty={isEmptyPreview}
          previewStateId={previewStateId}
        />
      ) : null}

      {previewStateId !== "ready" ? (
        <p className={`mt-4 text-xs uppercase tracking-[0.22em] ${tone.eyebrow}`}>
          {previewStateId === "syncing"
            ? "Detail guidance is adapting to an in-flight wallet state."
            : previewStateId === "attention"
              ? "Detail guidance is highlighting that intervention may be needed."
              : "Empty preview keeps this region intentional without collapsing it."}
        </p>
      ) : null}
    </section>
  );
}
