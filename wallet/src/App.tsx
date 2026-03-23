import { useMemo, useState } from "react";
import { ActionDetailPanel } from "./components/ActionDetailPanel";
import { ActionGrid } from "./components/ActionGrid";
import { ActivityPanel } from "./components/ActivityPanel";
import { AppShell } from "./components/AppShell";
import { AssetPanel } from "./components/AssetPanel";
import { HeroSummary } from "./components/HeroSummary";
import { NotesPanel } from "./components/NotesPanel";
import { PreviewStateSwitcher } from "./components/PreviewStateSwitcher";
import { WalletHeader } from "./components/WalletHeader";
import {
  getWalletPreviewState,
  walletPreviewStates,
  type WalletPreviewStateId,
} from "./data/walletPreviewStates";
import { createWalletView } from "./lib/walletView";

function App() {
  const [previewStateId, setPreviewStateId] =
    useState<WalletPreviewStateId>("ready");
  const [selectedActionId, setSelectedActionId] = useState<
    ReturnType<typeof createWalletView>["actions"][number]["id"]
  >("send");

  const activePreview = getWalletPreviewState(previewStateId);
  const view = useMemo(
    () => createWalletView(activePreview.state),
    [activePreview.state],
  );
  const selectedAction =
    view.actions.find((action) => action.id === selectedActionId) ??
    view.actions[0];

  return (
    <AppShell
      header={
        <>
          <PreviewStateSwitcher
            activePreviewId={previewStateId}
            previews={walletPreviewStates}
            onPreviewSelect={setPreviewStateId}
          />
          <WalletHeader
            label={view.identity.label}
            networkLabel={view.identity.networkLabel}
            statusLabel={view.identity.statusLabel}
            lastSyncedRelative={view.identity.lastSyncedRelative}
          />
        </>
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
      secondary={
        <ActionDetailPanel
          action={selectedAction}
          previewStateId={previewStateId}
        />
      }
    />
  );
}

export default App;
