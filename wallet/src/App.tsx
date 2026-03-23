import { useEffect, useMemo, useState } from "react";
import { ActivityPanel } from "./components/ActivityPanel";
import { AppShell } from "./components/AppShell";
import { AssetPanel } from "./components/AssetPanel";
import { HeroSummary } from "./components/HeroSummary";
import { NotesPanel } from "./components/NotesPanel";
import { WalletActionNav } from "./components/WalletActionNav";
import { WalletActionPanel } from "./components/WalletActionPanel";
import { WalletHeader } from "./components/WalletHeader";
import { WalletSidebar } from "./components/WalletSidebar";
import { WalletWorkspace } from "./components/WalletWorkspace";
import { walletActionDrafts, walletShellState, walletState } from "./data/stubWallet";
import {
  buildWalletActionDraftsView,
  buildWalletShellViewModel,
  createWalletView,
} from "./lib/walletView";
import type {
  WalletActiveRegion,
  WalletActiveSection,
  WalletDepositDraft,
  WalletReceiveDraft,
  WalletSendDraft,
  WalletWithdrawDraft,
} from "./types/wallet";

function getIsCompactLayout() {
  return window.matchMedia("(max-width: 1023px)").matches;
}

function App() {
  const [selectedActionId, setSelectedActionId] = useState<
    ReturnType<typeof createWalletView>["actions"][number]["id"]
  >(walletShellState.activeAction);
  const [activeRegion, setActiveRegion] = useState<WalletActiveRegion>(
    walletShellState.activeRegion,
  );
  const [activeSection, setActiveSection] = useState<WalletActiveSection>(
    walletShellState.activeSection,
  );
  const [isCompactLayout, setIsCompactLayout] = useState(() =>
    getIsCompactLayout(),
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

  useEffect(() => {
    const mediaQuery = window.matchMedia("(max-width: 1023px)");
    const handleMediaQueryChange = (event: MediaQueryListEvent) => {
      setIsCompactLayout(event.matches);
    };

    setIsCompactLayout(mediaQuery.matches);

    mediaQuery.addEventListener("change", handleMediaQueryChange);

    return () => {
      mediaQuery.removeEventListener("change", handleMediaQueryChange);
    };
  }, []);

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
        activeRegion,
        activeSection,
        activeAction: selectedActionId,
      }),
    [activeRegion, activeSection, selectedActionId],
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

  function handleActionSelect(actionId: typeof selectedActionId) {
    setSelectedActionId(actionId);

    if (isCompactLayout) {
      setActiveRegion("secondary");
    }
  }

  function handleSectionChange(section: WalletActiveSection) {
    setActiveSection(section);
    setActiveRegion("primary");
  }

  return (
    <AppShell
      header={
        <WalletHeader
          label={view.identity.label}
          networkLabel={view.identity.networkLabel}
          statusLabel={view.identity.statusLabel}
          statusTone={view.identity.statusTone}
          lastSyncedRelative={view.identity.lastSyncedRelative}
        />
      }
      workspace={
        <WalletWorkspace
          isCompactLayout={isCompactLayout}
          activeRegion={activeRegion}
          activeSection={activeSection}
          sections={shellView.sections}
          onRegionChange={setActiveRegion}
          onSectionChange={handleSectionChange}
          overview={
            <div className="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_20rem] xl:items-start">
              <HeroSummary
                identity={view.identity}
                summaryMetrics={view.summaryMetrics}
              />

              <WalletSidebar
                label={view.identity.label}
                networkLabel={view.identity.networkLabel}
                statusLabel={view.identity.statusLabel}
                statusTone={view.identity.statusTone}
                delegatePkShort={view.identity.delegatePkShort}
                scriptAddressShort={view.identity.scriptAddressShort}
                lastSyncedRelative={view.identity.lastSyncedRelative}
              />
            </div>
          }
          holdings={<AssetPanel assets={view.assets} />}
          notes={<NotesPanel notes={view.notes} />}
          activity={<ActivityPanel activity={view.activity} />}
          actionNav={
            <WalletActionNav
              actions={shellView.actions}
              onActionSelect={handleActionSelect}
            />
          }
          actionPanel={
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
          }
        />
      }
    />
  );
}

export default App;
