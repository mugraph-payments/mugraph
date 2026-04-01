import { useMemo, useState } from "react";
import { ActivityPanel } from "./components/ActivityPanel";
import { AssetPanel } from "./components/AssetPanel";
import { WalletActionScreen } from "./components/WalletActionScreen";
import { WalletBottomNav } from "./components/WalletBottomNav";
import { WalletHeader } from "./components/WalletHeader";
import { WalletHomeScreen } from "./components/WalletHomeScreen";
import { WalletSettingsScreen } from "./components/WalletSettingsScreen";
import { walletActionDrafts, walletShellState, walletState } from "./data/stubWallet";
import { createWalletView } from "./lib/walletView";
import type {
  WalletActiveDestination,
  WalletDepositDraft,
  WalletReceiveDraft,
  WalletSendDraft,
  WalletWithdrawDraft,
} from "./types/wallet";

function App() {
  const [activeDestination, setActiveDestination] = useState<WalletActiveDestination>(
    walletShellState.activeDestination,
  );
  const [activeConsumerAction, setActiveConsumerAction] = useState<"send" | "receive" | null>(null);
  const [sendDraft, setSendDraft] = useState<WalletSendDraft>(walletActionDrafts.send);
  const [receiveDraft, setReceiveDraft] = useState<WalletReceiveDraft>(walletActionDrafts.receive);
  const [depositDraft, setDepositDraft] = useState<WalletDepositDraft>(walletActionDrafts.deposit);
  const [withdrawDraft, setWithdrawDraft] = useState<WalletWithdrawDraft>(
    walletActionDrafts.withdraw,
  );

  const view = useMemo(() => createWalletView(walletState), []);
  const latestDeposit = useMemo(
    () => view.activity.find((item) => item.kindLabel === "Deposit") ?? null,
    [view.activity],
  );
  const latestWithdraw = useMemo(
    () => view.activity.find((item) => item.kindLabel === "Withdraw") ?? null,
    [view.activity],
  );
  const assetOptions = view.assets.map((asset) => ({
    id: asset.id,
    label: asset.ticker,
    balanceLabel: asset.balanceLabel,
  }));
  const topAssetLabel = view.assets[0]?.balanceLabel ?? "No holdings";

  function handleDestinationSelect(destination: WalletActiveDestination) {
    setActiveDestination(destination);
    if (destination !== "home") {
      setActiveConsumerAction(null);
    }
  }

  function handlePrimaryActionSelect(actionId: "send" | "receive") {
    setActiveConsumerAction(actionId);
  }

  const activeDestinationPanel = (() => {
    switch (activeDestination) {
      case "home":
        return activeConsumerAction ? (
          <WalletActionScreen
            activeAction={activeConsumerAction}
            actions={view.actions}
            onActionSelect={handlePrimaryActionSelect}
            onClose={() => setActiveConsumerAction(null)}
            sendDraft={sendDraft}
            onSendDraftChange={setSendDraft}
            receiveDraft={receiveDraft}
            onReceiveDraftChange={setReceiveDraft}
            assetOptions={assetOptions}
            identity={view.identity}
            noteCount={walletState.summary.noteCount}
            pendingActivityCount={walletState.summary.pendingActivityCount}
          />
        ) : (
          <WalletHomeScreen
            identity={view.identity}
            summaryMetrics={view.summaryMetrics}
            assets={view.assets}
            activity={view.activity}
            onPrimaryActionSelect={handlePrimaryActionSelect}
          />
        );
      case "assets":
        return <AssetPanel assets={view.assets} />;
      case "settings":
        return (
          <WalletSettingsScreen
            delegatePkShort={view.identity.delegatePkShort}
            scriptAddressShort={view.identity.scriptAddressShort}
            syncPostureLabel={`${view.identity.statusLabel} on ${view.identity.networkLabel}`}
            depositDraft={depositDraft}
            onDepositDraftChange={setDepositDraft}
            withdrawDraft={withdrawDraft}
            onWithdrawDraftChange={setWithdrawDraft}
            latestDepositReference={latestDeposit?.referenceShort ?? "No deposit reference"}
            latestWithdrawReference={latestWithdraw?.referenceShort ?? "No withdraw reference"}
            pendingActivityCount={walletState.summary.pendingActivityCount}
            topAssetLabel={topAssetLabel}
            assetOptions={assetOptions}
            notes={view.notes}
          />
        );
      case "activity":
        return <ActivityPanel activity={view.activity} />;
    }
  })();

  return (
    <div className="min-h-dvh text-slate-50">
      <div className="wallet-phone-shell mx-auto flex min-h-dvh w-full flex-col px-4 py-5 sm:px-5 sm:py-6 lg:px-6 2xl:px-8">
        <div className="grid flex-1 gap-4 lg:grid-cols-[17rem_minmax(0,1fr)] xl:grid-cols-[18rem_minmax(0,1fr)] 2xl:grid-cols-[19rem_minmax(0,1fr)]">
          <aside className="grid min-w-0 overflow-hidden content-start gap-5 lg:sticky lg:top-6 lg:self-start lg:gap-6">
            <WalletHeader
              label={view.identity.label}
              networkLabel={view.identity.networkLabel}
              statusLabel={view.identity.statusLabel}
              statusTone={view.identity.statusTone}
              lastSyncedRelative={view.identity.lastSyncedRelative}
              activeDestination={activeDestination}
            />

            <WalletBottomNav
              activeDestination={activeDestination}
              onDestinationSelect={handleDestinationSelect}
            />
          </aside>

          <main className="grid min-h-0 items-start gap-4 pb-24 lg:pb-0">
            {activeDestinationPanel}
          </main>
        </div>
      </div>
    </div>
  );
}

export default App;
