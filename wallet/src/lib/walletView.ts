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
  WalletActionDrafts,
  WalletActionKind,
  WalletActionPreset,
  WalletActivity,
  WalletActivityStatus,
  WalletDepositDraft,
  WalletNote,
  WalletNoteStatus,
  WalletReceiveDraft,
  WalletRootDestination,
  WalletSendDraft,
  WalletShellSection,
  WalletShellState,
  WalletState,
  WalletStatus,
  WalletWithdrawDraft,
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
  id:
    | "total-value-ada"
    | "total-value-usd"
    | "note-count"
    | "pending-activity-count";
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

export interface WalletShellSectionView {
  id: WalletShellSection;
  label: string;
  description: string;
  isActive: boolean;
}

export interface WalletShellActionView extends WalletActionView {
  isActive: boolean;
}

export interface WalletShellViewModel {
  activeDestination: WalletRootDestination;
  activeAction: WalletActionKind;
  sections: WalletShellSectionView[];
  actions: WalletShellActionView[];
}

export interface WalletDraftFieldView {
  label: string;
  value: string;
}

export interface WalletActionDraftView {
  id: WalletActionKind;
  title: string;
  helper: string;
  primaryLabel: string;
  isReady: boolean;
  missingRequirements: string[];
  fields: WalletDraftFieldView[];
}

export interface WalletActionDraftsView {
  send: WalletActionDraftView;
  receive: WalletActionDraftView;
  deposit: WalletActionDraftView;
  withdraw: WalletActionDraftView;
}

const PRIMARY_ACTION_ORDER: WalletActionKind[] = [
  "send",
  "receive",
  "deposit",
  "withdraw",
];

const DESTINATION_SECTION_ORDER: Array<{
  destination: WalletRootDestination;
  section: WalletShellSection;
}> = [
  { destination: "home", section: "overview" },
  { destination: "assets", section: "holdings" },
  { destination: "settings", section: "notes" },
  { destination: "activity", section: "activity" },
];

export function createWalletView(
  state: WalletState,
  now = new Date(),
): WalletView {
  return {
    identity: buildWalletIdentityView(state, now),
    summaryMetrics: buildWalletSummaryMetricViews(state),
    actions: buildWalletActionViews(state.actions),
    assets: state.assets.map(buildWalletAssetView),
    notes: state.notes.map((note) => buildWalletNoteView(note, now)),
    activity: state.activity.map((item) => buildWalletActivityView(item, now)),
  };
}

export function buildWalletIdentityView(
  state: WalletState,
  now = new Date(),
): WalletIdentityView {
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

export function buildWalletSummaryMetricViews(
  state: WalletState,
): WalletSummaryMetricView[] {
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

export function buildWalletActionViews(
  actions: WalletActionPreset[],
): WalletActionView[] {
  return [...actions]
    .sort(
      (left, right) =>
        PRIMARY_ACTION_ORDER.indexOf(left.id) -
        PRIMARY_ACTION_ORDER.indexOf(right.id),
    )
    .map((action) => ({
      ...action,
      isPrimary: PRIMARY_ACTION_ORDER.includes(action.id),
    }));
}

export function buildWalletShellViewModel(
  state: WalletState,
  shellState: WalletShellState,
): WalletShellViewModel {
  const activeSection = getLegacySectionForDestination(shellState.activeDestination);

  return {
    activeDestination: shellState.activeDestination,
    activeAction: shellState.activeAction,
    sections: DESTINATION_SECTION_ORDER.map(({ section }) => ({
      id: section,
      label: getWalletShellSectionLabel(section),
      description: getWalletShellSectionDescription(section),
      isActive: activeSection === section,
    })),
    actions: buildWalletActionViews(state.actions).map((action) => ({
      ...action,
      isActive: shellState.activeAction === action.id,
    })),
  };
}

export function buildWalletActionDraftsView(
  state: WalletState,
  drafts: WalletActionDrafts,
): WalletActionDraftsView {
  return {
    send: buildWalletSendDraftView(state, drafts.send),
    receive: buildWalletReceiveDraftView(state, drafts.receive),
    deposit: buildWalletDepositDraftView(state, drafts.deposit),
    withdraw: buildWalletWithdrawDraftView(state, drafts.withdraw),
  };
}

export function buildWalletSendDraftView(
  state: WalletState,
  draft: WalletSendDraft,
): WalletActionDraftView {
  const missingRequirements = getSendDraftMissingRequirements(draft);
  const actionMeta = getActionMeta(state.actions, "send");

  return {
    id: "send",
    title: actionMeta.label,
    helper: actionMeta.helper,
    primaryLabel: "Review transfer",
    isReady: missingRequirements.length === 0,
    missingRequirements,
    fields: [
      {
        label: "Asset",
        value: getDraftAssetLabel(state, draft.assetId),
      },
      {
        label: "Amount",
        value: formatDraftAmount(state, draft.assetId, draft.amountInput),
      },
      {
        label: "Recipient",
        value: formatDraftText(draft.recipient, "Recipient not set"),
      },
      {
        label: "Memo",
        value: formatDraftText(draft.memo, "No memo"),
      },
    ],
  };
}

export function buildWalletReceiveDraftView(
  state: WalletState,
  draft: WalletReceiveDraft,
): WalletActionDraftView {
  const missingRequirements = getReceiveDraftMissingRequirements(draft);
  const actionMeta = getActionMeta(state.actions, "receive");

  return {
    id: "receive",
    title: actionMeta.label,
    helper: actionMeta.helper,
    primaryLabel: "Generate request",
    isReady: missingRequirements.length === 0,
    missingRequirements,
    fields: [
      {
        label: "Requested asset",
        value: getDraftAssetLabel(state, draft.assetId),
      },
      {
        label: "Requested amount",
        value:
          formatDraftAmount(state, draft.assetId, draft.requestedAmountInput) ||
          "Open amount",
      },
      {
        label: "Request label",
        value: formatDraftText(draft.requestLabel, "Untitled request"),
      },
      {
        label: "Share mode",
        value: draft.shareMode === "qr" ? "QR code" : "Address",
      },
    ],
  };
}

export function buildWalletDepositDraftView(
  state: WalletState,
  draft: WalletDepositDraft,
): WalletActionDraftView {
  const missingRequirements = getDepositDraftMissingRequirements(draft);
  const actionMeta = getActionMeta(state.actions, "deposit");

  return {
    id: "deposit",
    title: actionMeta.label,
    helper: actionMeta.helper,
    primaryLabel: "Track deposit",
    isReady: missingRequirements.length === 0,
    missingRequirements,
    fields: [
      {
        label: "Funding asset",
        value: getDraftAssetLabel(state, draft.assetId),
      },
      {
        label: "Amount",
        value: formatDraftAmount(state, draft.assetId, draft.amountInput),
      },
      {
        label: "Source address",
        value: formatDraftText(draft.sourceAddress, "Source not set"),
      },
      {
        label: "Reference",
        value: formatDraftText(draft.reference, "No reference"),
      },
    ],
  };
}

export function buildWalletWithdrawDraftView(
  state: WalletState,
  draft: WalletWithdrawDraft,
): WalletActionDraftView {
  const missingRequirements = getWithdrawDraftMissingRequirements(draft);
  const actionMeta = getActionMeta(state.actions, "withdraw");

  return {
    id: "withdraw",
    title: actionMeta.label,
    helper: actionMeta.helper,
    primaryLabel: "Review withdrawal",
    isReady: missingRequirements.length === 0,
    missingRequirements,
    fields: [
      {
        label: "Settlement asset",
        value: getDraftAssetLabel(state, draft.assetId),
      },
      {
        label: "Amount",
        value: formatDraftAmount(state, draft.assetId, draft.amountInput),
      },
      {
        label: "Destination",
        value: formatDraftText(
          draft.destinationAddress,
          "Destination not set",
        ),
      },
      {
        label: "Reference",
        value: formatDraftText(draft.reference, "No reference"),
      },
    ],
  };
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

export function buildWalletNoteView(
  note: WalletNote,
  now = new Date(),
): WalletNoteView {
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

function getLegacySectionForDestination(
  destination: WalletRootDestination,
): WalletShellSection {
  return (
    DESTINATION_SECTION_ORDER.find((entry) => entry.destination === destination)
      ?.section ?? "overview"
  );
}

export function getWalletShellSectionLabel(section: WalletShellSection): string {
  switch (section) {
    case "overview":
      return "Overview";
    case "holdings":
      return "Holdings";
    case "notes":
      return "Notes";
    case "activity":
      return "Activity";
  }
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

export function getWalletNoteStatusTone(
  status: WalletNoteStatus,
): WalletTone {
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

export function getWalletActivityStatusTone(
  status: WalletActivityStatus,
): WalletTone {
  switch (status) {
    case "completed":
      return "positive";
    case "pending":
      return "warning";
    case "failed":
      return "critical";
  }
}

function getWalletShellSectionDescription(
  section: WalletShellSection,
): string {
  switch (section) {
    case "overview":
      return "Wallet posture and summary metrics.";
    case "holdings":
      return "Asset balances and note density.";
    case "notes":
      return "Spendable private note inventory.";
    case "activity":
      return "Recent deposits, refreshes, and withdrawals.";
  }
}

function getActionMeta(
  actions: WalletActionPreset[],
  actionId: WalletActionKind,
): WalletActionPreset {
  return (
    actions.find((action) => action.id === actionId) ?? {
      id: actionId,
      label: toTitleCase(actionId),
      helper: "Action helper unavailable.",
    }
  );
}

function getDraftAssetLabel(state: WalletState, assetId: string): string {
  const asset = state.assets.find((item) => item.id === assetId);

  if (!asset) {
    return "Asset not selected";
  }

  return `${asset.name} (${asset.ticker})`;
}

function formatDraftAmount(
  state: WalletState,
  assetId: string,
  amountInput: string,
): string {
  const amount = parsePositiveAmount(amountInput);
  const asset = state.assets.find((item) => item.id === assetId);

  if (amount === null || !asset) {
    return amountInput.trim() || "Amount not set";
  }

  return formatAssetBalance(amount, asset.ticker);
}

function formatDraftText(value: string, fallback: string): string {
  const trimmedValue = value.trim();

  return trimmedValue.length > 0 ? trimmedValue : fallback;
}

function getSendDraftMissingRequirements(draft: WalletSendDraft): string[] {
  const missingRequirements: string[] = [];

  if (!draft.assetId.trim()) {
    missingRequirements.push("Select an asset");
  }

  if (parsePositiveAmount(draft.amountInput) === null) {
    missingRequirements.push("Enter a valid amount");
  }

  if (!draft.recipient.trim()) {
    missingRequirements.push("Add a recipient");
  }

  return missingRequirements;
}

function getReceiveDraftMissingRequirements(
  draft: WalletReceiveDraft,
): string[] {
  const missingRequirements: string[] = [];

  if (!draft.assetId.trim()) {
    missingRequirements.push("Select an asset");
  }

  if (
    draft.requestedAmountInput.trim() &&
    parsePositiveAmount(draft.requestedAmountInput) === null
  ) {
    missingRequirements.push("Enter a valid requested amount");
  }

  if (!draft.requestLabel.trim()) {
    missingRequirements.push("Add a request label");
  }

  return missingRequirements;
}

function getDepositDraftMissingRequirements(
  draft: WalletDepositDraft,
): string[] {
  const missingRequirements: string[] = [];

  if (!draft.assetId.trim()) {
    missingRequirements.push("Select an asset");
  }

  if (parsePositiveAmount(draft.amountInput) === null) {
    missingRequirements.push("Enter a valid amount");
  }

  if (!draft.sourceAddress.trim()) {
    missingRequirements.push("Add a source address");
  }

  return missingRequirements;
}

function getWithdrawDraftMissingRequirements(
  draft: WalletWithdrawDraft,
): string[] {
  const missingRequirements: string[] = [];

  if (!draft.assetId.trim()) {
    missingRequirements.push("Select an asset");
  }

  if (parsePositiveAmount(draft.amountInput) === null) {
    missingRequirements.push("Enter a valid amount");
  }

  if (!draft.destinationAddress.trim()) {
    missingRequirements.push("Add a destination address");
  }

  return missingRequirements;
}

function parsePositiveAmount(input: string): number | null {
  const trimmedValue = input.trim();

  if (!trimmedValue) {
    return null;
  }

  const parsedValue = Number(trimmedValue);

  if (!Number.isFinite(parsedValue) || parsedValue <= 0) {
    return null;
  }

  return parsedValue;
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
