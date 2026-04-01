import {
  formatAda,
  formatNetworkLabel,
  formatNumber,
  formatPercent,
  formatRelativeTime,
  formatUsd,
  formatWalletStatus,
  truncateMiddle,
} from "./format";
import type {
  AssetHolding,
  WalletActionKind,
  WalletActionPreset,
  WalletActivity,
  WalletActivityStatus,
  WalletNote,
  WalletNoteStatus,
  WalletState,
  WalletStatus,
} from "../types/wallet";

export type WalletTone = "neutral" | "positive" | "warning" | "critical";

export interface WalletIdentityView {
  label: string;
  networkLabel: string;
  statusLabel: string;
  statusTone: WalletTone;
  delegatePkShort: string;
  scriptAddressShort: string;
  lastSyncedRelative: string;
}

export interface WalletSummaryMetricView {
  id: "total-value-ada" | "total-value-usd" | "note-count" | "pending-activity-count";
  label: string;
  value: string;
  tone: WalletTone;
}

export interface WalletActionView extends WalletActionPreset {
  isPrimary: boolean;
}

export interface WalletAssetView {
  id: string;
  ticker: string;
  name: string;
  balanceLabel: string;
  noteCountLabel: string;
  shareLabel: string;
  trendTone: WalletTone;
}

export interface WalletNoteView {
  id: string;
  assetTicker: string;
  amountLabel: string;
  sourceLabel: string;
  statusLabel: string;
  statusTone: WalletTone;
  nonceShort: string;
  signatureShort: string;
  createdAtRelative: string;
}

export interface WalletActivityView {
  id: string;
  kindLabel: string;
  statusLabel: string;
  statusTone: WalletTone;
  amountLabel: string;
  referenceShort: string;
  summary: string;
  createdAtRelative: string;
}

export interface WalletView {
  identity: WalletIdentityView;
  summaryMetrics: WalletSummaryMetricView[];
  actions: WalletActionView[];
  assets: WalletAssetView[];
  notes: WalletNoteView[];
  activity: WalletActivityView[];
}

const PRIMARY_ACTION_ORDER: WalletActionKind[] = ["send", "receive", "deposit", "withdraw"];

export function createWalletView(state: WalletState, now = new Date()): WalletView {
  return {
    identity: buildWalletIdentityView(state, now),
    summaryMetrics: buildWalletSummaryMetricViews(state),
    actions: buildWalletActionViews(state.actions),
    assets: state.assets.map(buildWalletAssetView),
    notes: state.notes.map((note) => buildWalletNoteView(note, now)),
    activity: state.activity.map((item) => buildWalletActivityView(item, now)),
  };
}

export function buildWalletIdentityView(state: WalletState, now = new Date()): WalletIdentityView {
  return {
    label: state.identity.label,
    networkLabel: formatNetworkLabel(state.identity.network),
    statusLabel: formatWalletStatus(state.identity.status),
    statusTone: getWalletStatusTone(state.identity.status),
    delegatePkShort: truncateMiddle(state.identity.delegatePk, 10, 8),
    scriptAddressShort: truncateMiddle(state.identity.scriptAddress, 14, 10),
    lastSyncedRelative: formatRelativeTime(state.identity.lastSyncedAt, now),
  };
}

export function buildWalletSummaryMetricViews(state: WalletState): WalletSummaryMetricView[] {
  return [
    {
      id: "total-value-ada",
      label: "Total value",
      value: formatAda(state.summary.totalValueAda),
      tone: "neutral",
    },
    {
      id: "total-value-usd",
      label: "USD reference",
      value: formatUsd(state.summary.totalValueUsd),
      tone: "neutral",
    },
    {
      id: "note-count",
      label: "Spendable notes",
      value: `${state.summary.noteCount}`,
      tone: state.summary.noteCount > 0 ? "positive" : "warning",
    },
    {
      id: "pending-activity-count",
      label: "Pending activity",
      value: `${state.summary.pendingActivityCount}`,
      tone: state.summary.pendingActivityCount > 0 ? "warning" : "positive",
    },
  ];
}

export function buildWalletActionViews(actions: WalletActionPreset[]): WalletActionView[] {
  return [...actions]
    .sort(
      (left, right) =>
        PRIMARY_ACTION_ORDER.indexOf(left.id) - PRIMARY_ACTION_ORDER.indexOf(right.id),
    )
    .map((action) => ({
      ...action,
      isPrimary: PRIMARY_ACTION_ORDER.includes(action.id),
    }));
}

export function buildWalletAssetView(asset: AssetHolding): WalletAssetView {
  return {
    id: asset.id,
    ticker: asset.ticker,
    name: asset.name,
    balanceLabel: formatAssetBalance(asset.balance, asset.ticker),
    noteCountLabel: `${asset.noteCount} notes`,
    shareLabel: formatPercent(asset.shareOfWalletPct),
    trendTone: getAssetTrendTone(asset.trend),
  };
}

export function buildWalletNoteView(note: WalletNote, now = new Date()): WalletNoteView {
  return {
    id: note.id,
    assetTicker: note.assetTicker,
    amountLabel: formatAssetBalance(note.amount, note.assetTicker),
    sourceLabel: toTitleCase(note.source),
    statusLabel: toTitleCase(note.status),
    statusTone: getWalletNoteStatusTone(note.status),
    nonceShort: truncateMiddle(note.nonce, 10, 8),
    signatureShort: truncateMiddle(note.signaturePreview, 8, 8),
    createdAtRelative: formatRelativeTime(note.createdAt, now),
  };
}

export function buildWalletActivityView(
  activity: WalletActivity,
  now = new Date(),
): WalletActivityView {
  return {
    id: activity.id,
    kindLabel: toTitleCase(activity.kind),
    statusLabel: toTitleCase(activity.status),
    statusTone: getWalletActivityStatusTone(activity.status),
    amountLabel: formatAssetBalance(activity.amount, activity.assetTicker),
    referenceShort: truncateMiddle(activity.reference, 10, 8),
    summary: activity.summary,
    createdAtRelative: formatRelativeTime(activity.createdAt, now),
  };
}

export function getWalletStatusTone(status: WalletStatus): WalletTone {
  switch (status) {
    case "ready":
      return "positive";
    case "syncing":
      return "warning";
    case "attention":
      return "critical";
  }
}

export function getWalletNoteStatusTone(status: WalletNoteStatus): WalletTone {
  switch (status) {
    case "available":
      return "positive";
    case "pending":
      return "warning";
    case "reserved":
      return "neutral";
    case "spent":
      return "critical";
  }
}

export function getWalletActivityStatusTone(status: WalletActivityStatus): WalletTone {
  switch (status) {
    case "completed":
      return "positive";
    case "pending":
      return "warning";
    case "failed":
      return "critical";
  }
}

function getAssetTrendTone(trend: AssetHolding["trend"]): WalletTone {
  switch (trend) {
    case "up":
      return "positive";
    case "flat":
      return "neutral";
    case "down":
      return "warning";
  }
}

function formatAssetBalance(value: number, ticker: string): string {
  if (ticker === "ADA") {
    return formatAda(value);
  }

  return `${formatNumber(value, 2)} ${ticker}`;
}

function toTitleCase(value: string): string {
  return value
    .split(/[_-]/g)
    .map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}
