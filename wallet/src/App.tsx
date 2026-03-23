import { useState } from "react";
import { ActionDetailPanel } from "./components/ActionDetailPanel";
import { ActionGrid } from "./components/ActionGrid";
import { ActivityPanel } from "./components/ActivityPanel";
import { AppShell } from "./components/AppShell";
import { AssetPanel } from "./components/AssetPanel";
import { HeroSummary } from "./components/HeroSummary";
import { NotesPanel } from "./components/NotesPanel";
import { WalletHeader } from "./components/WalletHeader";
import { walletState } from "./data/stubWallet";
import { createWalletView } from "./lib/walletView";

function App() {
  const view = createWalletView(walletState);
  const [selectedActionId, setSelectedActionId] = useState(view.actions[0]?.id);
  const selectedAction =
    view.actions.find((action) => action.id === selectedActionId) ??
    view.actions[0];

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
      primary={
        <>
          <HeroSummary
            identity={view.identity}
            summaryMetrics={view.summaryMetrics}
          />

          <ActionGrid
            actions={view.actions}
            selectedActionId={selectedAction.id}
            onActionSelect={setSelectedActionId}
          />

          <AssetPanel assets={view.assets} />

          <NotesPanel notes={view.notes} />

          <ActivityPanel activity={view.activity} />
        </>
      }
      secondary={<ActionDetailPanel action={selectedAction} />}
    />
  );
}

export default App;
