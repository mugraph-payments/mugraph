import { createContext, useContext } from "react";
import type { WalletSnapshot, ImportResult, SendResult, RefreshResult, SyncResult } from "./api";
import * as api from "./api";
import type {
  AssetHolding,
  MugraphNetwork,
  WalletNote,
  WalletState,
  WalletStatus,
} from "../types/wallet";

// Known asset metadata for Milestone A
const KNOWN_ASSETS: Record<string, { ticker: string; name: string; decimals: number }> = {
  "00000000000000000000000000000000000000000000000000000000:lovelace": {
    ticker: "ADA",
    name: "Cardano",
    decimals: 6,
  },
  "00000000000000000000000000000000000000000000000000000000:": {
    ticker: "ADA",
    name: "Cardano",
    decimals: 6,
  },
};

function lookupAssetMeta(policyId: string, assetName: string) {
  const key = `${policyId}:${assetName}`;
  return (
    KNOWN_ASSETS[key] ?? {
      ticker: assetName || "UNKNOWN",
      name: assetName || "Unknown Asset",
      decimals: 0,
    }
  );
}

export function snapshotToWalletState(snapshot: WalletSnapshot): WalletState {
  const assetMap = new Map<
    string,
    { balance: number; noteCount: number; policyId: string; assetName: string }
  >();

  for (const stored of snapshot.notes) {
    if (stored.status !== "available") continue;
    const key = `${stored.note.policy_id}:${stored.note.asset_name}`;
    const existing = assetMap.get(key);
    if (existing) {
      existing.balance += stored.note.amount;
      existing.noteCount += 1;
    } else {
      assetMap.set(key, {
        balance: stored.note.amount,
        noteCount: 1,
        policyId: stored.note.policy_id,
        assetName: stored.note.asset_name,
      });
    }
  }

  const assets: AssetHolding[] = Array.from(assetMap.entries()).map(([id, data]) => {
    const meta = lookupAssetMeta(data.policyId, data.assetName);
    return {
      id,
      ticker: meta.ticker,
      name: meta.name,
      policyId: data.policyId,
      assetName: data.assetName,
      balance: data.balance,
      decimals: meta.decimals,
      noteCount: data.noteCount,
      shareOfWalletPct: 0,
      trend: "flat" as const,
    };
  });

  const notes: WalletNote[] = snapshot.notes.map((stored) => ({
    id: `note-${stored.note.nonce.slice(0, 8)}`,
    assetTicker: lookupAssetMeta(stored.note.policy_id, stored.note.asset_name).ticker,
    amount: stored.note.amount,
    nonce: stored.note.nonce,
    signaturePreview: stored.note.signature.slice(0, 32),
    source: "refresh" as const,
    status:
      stored.status === "quarantined"
        ? ("pending" as const)
        : (stored.status as "available" | "spent"),
    createdAt: new Date(stored.created_at * 1000).toISOString(),
  }));

  const hasQuarantined = snapshot.notes.some((n) => n.status === "quarantined");
  const status: WalletStatus =
    snapshot.has_orphaned_blinding_factors || hasQuarantined ? "attention" : "ready";

  const activity = snapshot.activity.map((a) => ({
    id: a.id,
    kind: a.kind as "deposit" | "refresh" | "withdraw",
    status: "completed" as const,
    assetTicker: "ADA",
    amount: 0,
    summary: a.details,
    reference: a.id,
    createdAt: new Date(a.timestamp * 1000).toISOString(),
  }));

  const pendingActivityCount = snapshot.activity.filter(
    (a) => a.kind === "withdraw" || a.kind === "deposit",
  ).length;

  return {
    identity: {
      label: snapshot.label,
      mode: "live",
      network: snapshot.network as MugraphNetwork,
      status,
      delegatePk: snapshot.delegate_pk ?? "",
      scriptAddress: snapshot.cardano_script_address ?? "",
      lastSyncedAt: new Date().toISOString(),
    },
    summary: {
      totalValueAda: 0,
      totalValueUsd: 0,
      liquidAssetCount: assets.length,
      noteCount: snapshot.notes.filter((n) => n.status === "available").length,
      pendingActivityCount,
    },
    assets,
    notes,
    activity,
    actions: [
      {
        id: "send",
        label: "Send",
        helper: "Send funds from your wallet using the notes you already hold.",
      },
      { id: "receive", label: "Receive", helper: "Share your wallet details to receive funds." },
      {
        id: "deposit",
        label: "Deposit",
        helper: "Track incoming Cardano funds before they become available.",
      },
      {
        id: "withdraw",
        label: "Withdraw",
        helper: "Move wallet funds out to a destination address.",
      },
    ],
  };
}

export interface WalletStore {
  state: WalletState;
  activeNetwork: MugraphNetwork;
  setupComplete: boolean;
  loading: boolean;
  refresh: () => Promise<void>;
  switchNetwork: (network: MugraphNetwork) => Promise<void>;
  importNotes: (payload: string) => Promise<ImportResult>;
  sendNotes: (noteNonces: string[]) => Promise<SendResult>;
  refreshNotes: (noteNonces: string[], targetAmounts: number[]) => Promise<RefreshResult>;
  sync: () => Promise<SyncResult>;
  completeSetup: (config: api.SetupConfig) => Promise<void>;
}

export const WalletStoreContext = createContext<WalletStore | null>(null);

export function useWalletStore(): WalletStore {
  const store = useContext(WalletStoreContext);
  if (!store) {
    throw new Error("useWalletStore must be used within a WalletStoreProvider");
  }
  return store;
}
