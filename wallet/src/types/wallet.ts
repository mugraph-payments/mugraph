export type MugraphNetwork = "mainnet" | "preprod" | "preview";

export type WalletMode = "stub";
export type WalletStatus = "ready" | "syncing" | "attention";
export type WalletNoteSource = "deposit" | "refresh" | "withdraw" | "change";
export type WalletNoteStatus = "available" | "pending" | "reserved" | "spent";
export type WalletActivityKind = "deposit" | "refresh" | "withdraw";
export type WalletActivityStatus = "completed" | "pending" | "failed";
export type WalletActionKind = "send" | "receive" | "deposit" | "withdraw";
export type WalletShellRegion = "primary" | "secondary";
export type WalletShellSection =
  | "overview"
  | "holdings"
  | "notes"
  | "activity";
export type WalletReceiveShareMode = "address" | "qr";
export type WalletActiveRegion = WalletShellRegion;
export type WalletActiveSection = WalletShellSection;
export type WalletActiveAction = WalletActionKind;

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

export interface WalletShellState {
  activeRegion: WalletActiveRegion;
  activeSection: WalletActiveSection;
  activeAction: WalletActiveAction;
}

export interface WalletSendDraft {
  assetId: string;
  amountInput: string;
  recipient: string;
  memo: string;
}

export interface WalletReceiveDraft {
  assetId: string;
  requestedAmountInput: string;
  requestLabel: string;
  shareMode: WalletReceiveShareMode;
}

export interface WalletDepositDraft {
  assetId: string;
  amountInput: string;
  sourceAddress: string;
  reference: string;
}

export interface WalletWithdrawDraft {
  assetId: string;
  amountInput: string;
  destinationAddress: string;
  reference: string;
}

export interface WalletActionDrafts {
  send: WalletSendDraft;
  receive: WalletReceiveDraft;
  deposit: WalletDepositDraft;
  withdraw: WalletWithdrawDraft;
}

export interface WalletState {
  identity: WalletIdentity;
  summary: PortfolioSummary;
  assets: AssetHolding[];
  notes: WalletNote[];
  activity: WalletActivity[];
  actions: WalletActionPreset[];
}
