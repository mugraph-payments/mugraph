import { emptyWalletState, walletState } from "./stubWallet";
import type { WalletState } from "../types/wallet";

export type WalletPreviewStateId =
  | "ready"
  | "empty"
  | "syncing"
  | "attention";

export interface WalletPreviewState {
  id: WalletPreviewStateId;
  label: string;
  description: string;
  state: WalletState;
}

const readyState: WalletState = walletState;

const emptyState: WalletState = {
  ...emptyWalletState,
  identity: {
    ...emptyWalletState.identity,
    lastSyncedAt: "2026-03-18T09:42:00Z",
  },
  summary: {
    ...emptyWalletState.summary,
    liquidAssetCount: 0,
    noteCount: 0,
    pendingActivityCount: 0,
  },
  assets: [],
  notes: [],
  activity: [],
};

const syncingState: WalletState = {
  ...walletState,
  identity: {
    ...walletState.identity,
    status: "syncing",
    lastSyncedAt: "2026-03-18T09:39:00Z",
  },
  summary: {
    ...walletState.summary,
    pendingActivityCount: 2,
  },
};

const attentionState: WalletState = {
  ...walletState,
  identity: {
    ...walletState.identity,
    status: "attention",
    lastSyncedAt: "2026-03-18T07:42:00Z",
  },
  summary: {
    ...walletState.summary,
    pendingActivityCount: 3,
  },
};

export const walletPreviewStates: WalletPreviewState[] = [
  {
    id: "ready",
    label: "Ready",
    description: "Baseline wallet preview with the full stub portfolio.",
    state: readyState,
  },
  {
    id: "empty",
    label: "Empty",
    description: "Inventory cleared to preview empty-state branches.",
    state: emptyState,
  },
  {
    id: "syncing",
    label: "Syncing",
    description: "Wallet is catching up and has active pending work.",
    state: syncingState,
  },
  {
    id: "attention",
    label: "Attention",
    description: "Wallet needs operator attention before more actions proceed.",
    state: attentionState,
  },
];

export function getWalletPreviewState(
  id: WalletPreviewStateId,
): WalletPreviewState {
  return (
    walletPreviewStates.find((preview) => preview.id === id) ??
    walletPreviewStates[0]
  );
}
