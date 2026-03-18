export type MugraphNetwork = "mainnet" | "preprod" | "preview";

export type WalletMode = "stub";
export type WalletStatus = "ready" | "syncing" | "attention";
export type WalletNoteSource = "deposit" | "refresh" | "withdraw" | "change";
export type WalletNoteStatus = "available" | "pending" | "reserved" | "spent";
export type WalletActivityKind = "deposit" | "refresh" | "withdraw";
export type WalletActivityStatus = "completed" | "pending" | "failed";
export type WalletActionKind = "send" | "receive" | "deposit" | "withdraw";

export interface WalletIdentity {
  label: string;
  mode: WalletMode;
  network: MugraphNetwork;
  status: WalletStatus;
  delegatePk: string;
  scriptAddress: string;
  lastSyncedAt: string;
}

export interface PortfolioSummary {
  totalValueAda: number;
  totalValueUsd: number;
  liquidAssetCount: number;
  noteCount: number;
  pendingActivityCount: number;
}

export interface AssetHolding {
  id: string;
  ticker: string;
  name: string;
  policyId: string;
  assetName: string;
  balance: number;
  decimals: number;
  noteCount: number;
  shareOfWalletPct: number;
  trend: "up" | "flat" | "down";
}

export interface WalletNote {
  id: string;
  assetTicker: string;
  amount: number;
  nonce: string;
  signaturePreview: string;
  source: WalletNoteSource;
  status: WalletNoteStatus;
  createdAt: string;
}

export interface WalletActivity {
  id: string;
  kind: WalletActivityKind;
  status: WalletActivityStatus;
  assetTicker: string;
  amount: number;
  summary: string;
  reference: string;
  createdAt: string;
}

export interface WalletActionPreset {
  id: WalletActionKind;
  label: string;
  helper: string;
}

export interface WalletState {
  identity: WalletIdentity;
  summary: PortfolioSummary;
  assets: AssetHolding[];
  notes: WalletNote[];
  activity: WalletActivity[];
  actions: WalletActionPreset[];
}
