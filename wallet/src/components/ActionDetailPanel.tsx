import { walletState } from "../data/stubWallet";
import { createWalletView, type WalletActionView } from "../lib/walletView";
import { DepositDetails } from "./DepositDetails";
import { ReceiveDetails } from "./ReceiveDetails";
import { SendDetails } from "./SendDetails";
import { WithdrawDetails } from "./WithdrawDetails";

interface ActionDetailPanelProps {
  action: WalletActionView;
}

export function ActionDetailPanel({ action }: ActionDetailPanelProps) {
  const view = createWalletView(walletState);
  const latestDeposit =
    view.activity.find((item) => item.kindLabel === "Deposit") ?? null;
  const latestWithdraw =
    view.activity.find((item) => item.kindLabel === "Withdraw") ?? null;
  const topAsset = view.assets[0]?.balanceLabel ?? "No holdings";

  return (
    <section className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Detail region
          </p>
          <h2 className="mt-2 text-xl font-semibold tracking-tight text-slate-50">
            {action.label} is selected
          </h2>
        </div>
        <span className="self-start rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[11px] uppercase tracking-[0.22em] text-slate-300">
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
        />
      ) : null}

      {action.id === "deposit" ? (
        <DepositDetails
          scriptAddressShort={view.identity.scriptAddressShort}
          delegatePkShort={view.identity.delegatePkShort}
          latestDepositReference={
            latestDeposit?.referenceShort ?? "No deposit reference"
          }
          pendingActivityCount={walletState.summary.pendingActivityCount}
        />
      ) : null}

      {action.id === "send" ? (
        <SendDetails
          noteCount={walletState.summary.noteCount}
          topAssetLabel={topAsset}
          pendingActivityCount={walletState.summary.pendingActivityCount}
        />
      ) : null}

      {action.id === "withdraw" ? (
        <WithdrawDetails
          latestWithdrawReference={
            latestWithdraw?.referenceShort ?? "No withdraw reference"
          }
          pendingActivityCount={walletState.summary.pendingActivityCount}
          scriptAddressShort={view.identity.scriptAddressShort}
          topAssetLabel={topAsset}
        />
      ) : null}
    </section>
  );
}
