import { useEffect, useMemo, useState } from "react";
import { ActionDetailPanel } from "./components/ActionDetailPanel";
import { ActionGrid } from "./components/ActionGrid";
import { ActivityPanel } from "./components/ActivityPanel";
import { AppShell } from "./components/AppShell";
import { AssetPanel } from "./components/AssetPanel";
import { HeroSummary } from "./components/HeroSummary";
import { NotesPanel } from "./components/NotesPanel";
import { WalletHeader } from "./components/WalletHeader";
import { WalletWorkspace } from "./components/WalletWorkspace";
import { walletShellState, walletState } from "./data/stubWallet";
import { createWalletView } from "./lib/walletView";
import type { WalletActiveRegion } from "./types/wallet";

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
  const [isCompactLayout, setIsCompactLayout] = useState(() =>
    getIsCompactLayout(),
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
  const selectedAction =
    view.actions.find((action) => action.id === selectedActionId) ??
    view.actions[0];

  function handleActionSelect(actionId: typeof selectedActionId) {
    setSelectedActionId(actionId);

    if (isCompactLayout) {
      setActiveRegion("secondary");
    }
  }

  return (
    <AppShell
      header={
        <WalletHeader
          label={view.identity.label}
          networkLabel={view.identity.networkLabel}
          statusLabel={view.identity.statusLabel}
          lastSyncedRelative={view.identity.lastSyncedRelative}
        />
      }
      workspace={
        <WalletWorkspace
          isCompactLayout={isCompactLayout}
          activeRegion={activeRegion}
          onRegionChange={setActiveRegion}
          primary={
            <>
              <HeroSummary
                identity={view.identity}
                summaryMetrics={view.summaryMetrics}
              />

              <ActionGrid
                actions={view.actions}
                selectedActionId={selectedAction.id}
                onActionSelect={handleActionSelect}
                previewStateId="ready"
              />

              <AssetPanel assets={view.assets} />

              <NotesPanel notes={view.notes} />

              <ActivityPanel activity={view.activity} />
            </>
          }
          secondary={
            <ActionDetailPanel action={selectedAction} previewStateId="ready" />
          }
        />
      }
    />
  );
}

export default App;
