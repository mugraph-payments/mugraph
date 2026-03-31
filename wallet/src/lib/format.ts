import type { MugraphNetwork, WalletStatus } from "../types/wallet";

const usdFormatter = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

export function formatNumber(value: number, maximumFractionDigits = 2): string {
  const minimumFractionDigits = Number.isInteger(value) ? 0 : 2;

  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: Math.min(minimumFractionDigits, maximumFractionDigits),
    maximumFractionDigits,
  }).format(value);
}

export function formatAda(value: number): string {
  return `${formatNumber(value, 2)} ADA`;
}

export function formatUsd(value: number): string {
  return usdFormatter.format(value);
}

export function formatPercent(value: number): string {
  return `${formatNumber(value, 1)}%`;
}

export function truncateMiddle(value: string, leading = 10, trailing = 8): string {
  if (value.length <= leading + trailing + 1) {
    return value;
  }

  return `${value.slice(0, leading)}…${value.slice(-trailing)}`;
}

export function formatRelativeTime(input: string | Date, now = new Date()): string {
  const target = typeof input === "string" ? new Date(input) : input;
  const diffMs = now.getTime() - target.getTime();

  if (Number.isNaN(diffMs)) {
    return "unknown";
  }

  const minute = 60_000;
  const hour = 60 * minute;
  const day = 24 * hour;

  if (diffMs < minute) {
    return "just now";
  }

  if (diffMs < hour) {
    return `${Math.floor(diffMs / minute)}m ago`;
  }

  if (diffMs < day) {
    return `${Math.floor(diffMs / hour)}h ago`;
  }

  return `${Math.floor(diffMs / day)}d ago`;
}

export function formatNetworkLabel(network: MugraphNetwork): string {
  switch (network) {
    case "mainnet":
      return "Mainnet";
    case "preprod":
      return "Preprod";
    case "preview":
      return "Preview";
  }
}

export function formatWalletStatus(status: WalletStatus): string {
  switch (status) {
    case "ready":
      return "Ready";
    case "syncing":
      return "Syncing";
    case "attention":
      return "Needs attention";
  }
}
