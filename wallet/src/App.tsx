import { useMemo, useState } from "react";
import { ActivityPanel } from "./components/ActivityPanel";
import { AssetPanel } from "./components/AssetPanel";
import { HeroSummary } from "./components/HeroSummary";
import { NotesPanel } from "./components/NotesPanel";
import { WalletActionNav } from "./components/WalletActionNav";
import { WalletActionPanel } from "./components/WalletActionPanel";
import { WalletHeader } from "./components/WalletHeader";
import { WalletOverviewBoard } from "./components/WalletOverviewBoard";
import { WalletSectionTabs } from "./components/WalletSectionTabs";
import { WalletSidebar } from "./components/WalletSidebar";
import { walletActionDrafts, walletShellState, walletState } from "./data/stubWallet";
import {
 buildWalletActionDraftsView,
 buildWalletShellViewModel,
 createWalletView,
} from "./lib/walletView";
import type {
 WalletActiveSection,
 WalletDepositDraft,
 WalletReceiveDraft,
 WalletSendDraft,
 WalletWithdrawDraft,
} from "./types/wallet";

function App() {
 const [selectedActionId, setSelectedActionId] = useState<
  ReturnType<typeof createWalletView>["actions"][number]["id"]
 >(walletShellState.activeAction);
 const [activeSection, setActiveSection] = useState<WalletActiveSection>(
  walletShellState.activeSection,
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
    activeRegion: "primary",
    activeSection,
    activeAction: selectedActionId,
   }),
  [activeSection, selectedActionId],
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

 const activeSectionPanel = (() => {
  switch (activeSection) {
   case "overview":
    return (
     <WalletOverviewBoard
      assets={view.assets}
      notes={view.notes}
      activity={view.activity}
     />
    );
   case "holdings":
    return <AssetPanel assets={view.assets} />;
   case "notes":
    return <NotesPanel notes={view.notes} />;
   case "activity":
    return <ActivityPanel activity={view.activity} />;
  }
 })();

 return (
  <div className="min-h-dvh text-slate-50">
   <div className="mx-auto flex min-h-dvh w-full max-w-7xl flex-col gap-5 px-4 py-5 sm:px-5 lg:px-6 lg:py-6">
    <WalletHeader
     label={view.identity.label}
     networkLabel={view.identity.networkLabel}
     statusLabel={view.identity.statusLabel}
     statusTone={view.identity.statusTone}
     lastSyncedRelative={view.identity.lastSyncedRelative}
    />

    <div className="grid min-h-0 flex-1 gap-5 xl:grid-cols-[16rem_minmax(0,1fr)] 2xl:grid-cols-[16rem_minmax(0,1fr)_22rem]">
     <aside className="grid content-start gap-5 xl:sticky xl:top-6 xl:self-start">
      <WalletSidebar
       label={view.identity.label}
       networkLabel={view.identity.networkLabel}
       statusLabel={view.identity.statusLabel}
       statusTone={view.identity.statusTone}
       delegatePkShort={view.identity.delegatePkShort}
       scriptAddressShort={view.identity.scriptAddressShort}
       lastSyncedRelative={view.identity.lastSyncedRelative}
      />
      <WalletSectionTabs
       sections={shellView.sections}
       activeSection={activeSection}
       onSectionChange={setActiveSection}
      />
     </aside>

     <main className="grid min-w-0 content-start gap-5">
      <HeroSummary
       identity={view.identity}
       summaryMetrics={view.summaryMetrics}
      />
      {activeSectionPanel}
     </main>

     <aside className="grid content-start gap-5 xl:col-start-2 2xl:col-start-auto 2xl:sticky 2xl:top-6 2xl:self-start">
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
     </aside>
    </div>
   </div>
  </div>
 );
}

export default App;
