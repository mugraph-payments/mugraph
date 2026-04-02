import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
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

  const prefersReducedMotion = useReducedMotion();

  const activeDestinationPanel = (() => {
    switch (activeDestination) {
      case "home":
        return activeConsumerAction ? (
          <WalletActionScreen
            activeAction={activeConsumerAction}
            onActionSelect={handlePrimaryActionSelect}
            onClose={() => setActiveConsumerAction(null)}
            sendDraft={sendDraft}
            onSendDraftChange={setSendDraft}
            receiveDraft={receiveDraft}
            onReceiveDraftChange={setReceiveDraft}
            assetOptions={assetOptions}
            identity={view.identity}
          />
        ) : (
          <WalletHomeScreen
            summaryMetrics={view.summaryMetrics}
            assets={view.assets}
            activity={view.activity}
            onPrimaryActionSelect={handlePrimaryActionSelect}
          />
        );
      case "assets":
        return <AssetPanel assets={view.assets} notes={view.notes} />;
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
      <div className="wallet-phone-shell mx-auto flex min-h-dvh w-full flex-col px-4 py-5 sm:px-5 sm:py-6 xl:px-7 xl:py-7 2xl:px-8">
        <div className="grid flex-1 content-start gap-6 lg:grid-cols-[18rem_minmax(0,1fr)] xl:grid-cols-[19rem_minmax(0,1fr)] xl:gap-8">
          <aside className="grid min-w-0 content-start gap-5 lg:sticky lg:top-7 lg:self-start xl:gap-6">
            <WalletHeader
              label={view.identity.label}
              networkLabel={view.identity.networkLabel}
              lastSyncedRelative={view.identity.lastSyncedRelative}
            />

            <WalletBottomNav
              activeDestination={activeDestination}
              onDestinationSelect={handleDestinationSelect}
            />
          </aside>

          <main className="grid min-h-0 items-start gap-6 pb-24 lg:pb-4 xl:gap-8">
            <AnimatePresence mode="wait" initial={false}>
              <motion.div
                key={`${activeDestination}-${activeConsumerAction ?? "none"}`}
                initial={prefersReducedMotion ? false : { opacity: 0, y: 6 }}
                animate={{ opacity: 1, y: 0 }}
                exit={prefersReducedMotion ? undefined : { opacity: 0, y: -6 }}
                transition={{ duration: 0.12, ease: [0.16, 1, 0.3, 1] }}
                className="grid items-start gap-6"
              >
                {activeDestinationPanel}
              </motion.div>
            </AnimatePresence>
          </main>
        </div>
      </div>
    </div>
  );
}

export default App;
