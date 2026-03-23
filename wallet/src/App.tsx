import { useMemo, useState } from "react";
import { ActivityPanel } from "./components/ActivityPanel";
import { AssetPanel } from "./components/AssetPanel";
import { WalletActionNav } from "./components/WalletActionNav";
import { WalletActionPanel } from "./components/WalletActionPanel";
import { WalletActionScreen } from "./components/WalletActionScreen";
import { WalletBottomNav } from "./components/WalletBottomNav";
import { WalletHeader } from "./components/WalletHeader";
import { WalletHomeScreen } from "./components/WalletHomeScreen";
import { WalletSettingsScreen } from "./components/WalletSettingsScreen";
import { walletActionDrafts, walletShellState, walletState } from "./data/stubWallet";
import {
  buildWalletActionDraftsView,
  buildWalletShellViewModel,
  createWalletView,
} from "./lib/walletView";
import type {
  WalletActiveDestination,
  WalletDepositDraft,
  WalletReceiveDraft,
  WalletSendDraft,
  WalletWithdrawDraft,
} from "./types/wallet";

function App() {
  const [selectedActionId, setSelectedActionId] = useState<
    ReturnType<typeof createWalletView>["actions"][number]["id"]
  >(walletShellState.activeAction);
  const [activeDestination, setActiveDestination] = useState<WalletActiveDestination>(
    walletShellState.activeDestination,
  );
  const [activeConsumerAction, setActiveConsumerAction] = useState<"send" | "receive" | null>(
    null,
  );
  const [sendDraft, setSendDraft] = useState<WalletSendDraft>(
    walletActionDrafts.send,
  );
  const [receiveDraft, setReceiveDraft] = useState<WalletReceiveDraft>(
    walletActionDrafts.receive,
  );
  const [depositDraft, setDepositDraft] = useState<WalletDepositDraft>(
    walletActionDrafts.deposit,
  );
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
  const draftViews = useMemo(
    () =>
      buildWalletActionDraftsView(walletState, {
        ...walletActionDrafts,
        send: sendDraft,
        receive: receiveDraft,
        deposit: depositDraft,
        withdraw: withdrawDraft,
      }),
    [depositDraft, receiveDraft, sendDraft, withdrawDraft],
  );
  const shellView = useMemo(
    () =>
      buildWalletShellViewModel(walletState, {
        activeDestination,
        activeAction: selectedActionId,
      }),
    [activeDestination, selectedActionId],
  );
  const selectedAction =
    view.actions.find((action) => action.id === selectedActionId) ??
    view.actions[0];
  const selectedActionDraft = draftViews[selectedAction.id];
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
    setSelectedActionId(actionId);
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
          />
        );
      case "activity":
        return <ActivityPanel activity={view.activity} />;
    }
  })();

  return (
    <div className="min-h-dvh text-slate-50">
      <div className="wallet-phone-shell mx-auto flex min-h-dvh w-full flex-col gap-5 px-4 py-5 sm:px-5 sm:py-6">
        <WalletHeader
          label={view.identity.label}
          networkLabel={view.identity.networkLabel}
          statusLabel={view.identity.statusLabel}
          statusTone={view.identity.statusTone}
          lastSyncedRelative={view.identity.lastSyncedRelative}
          activeDestination={activeDestination}
        />

        <main className="grid min-h-0 flex-1 content-start gap-5 pb-24">
          {activeDestinationPanel}

          <WalletActionNav
            actions={shellView.actions}
            onActionSelect={setSelectedActionId}
          />

          <WalletActionPanel
            action={selectedAction}
            draft={selectedActionDraft}
            sendDraft={sendDraft}
            onSendDraftChange={setSendDraft}
            receiveDraft={receiveDraft}
            onReceiveDraftChange={setReceiveDraft}
            depositDraft={depositDraft}
            onDepositDraftChange={setDepositDraft}
            withdrawDraft={withdrawDraft}
            onWithdrawDraftChange={setWithdrawDraft}
            assetOptions={assetOptions}
            receiveContext={{
              label: view.identity.label,
              delegatePkShort: view.identity.delegatePkShort,
              scriptAddressShort: view.identity.scriptAddressShort,
              networkLabel: view.identity.networkLabel,
              lastSyncedRelative: view.identity.lastSyncedRelative,
            }}
            depositContext={{
              scriptAddressShort: view.identity.scriptAddressShort,
              delegatePkShort: view.identity.delegatePkShort,
              latestDepositReference:
                latestDeposit?.referenceShort ?? "No deposit reference",
            }}
            withdrawContext={{
              latestWithdrawReference:
                latestWithdraw?.referenceShort ?? "No withdraw reference",
              scriptAddressShort: view.identity.scriptAddressShort,
              topAssetLabel,
            }}
            noteCount={walletState.summary.noteCount}
            pendingActivityCount={walletState.summary.pendingActivityCount}
          />
        </main>

        <WalletBottomNav
          activeDestination={activeDestination}
          onDestinationSelect={handleDestinationSelect}
        />
      </div>
    </div>
  );
}

export default App;
