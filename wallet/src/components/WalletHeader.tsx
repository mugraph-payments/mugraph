import type { WalletTone } from "../lib/walletView";
import { BrandMark } from "./BrandMark";
import { StatusChip } from "./StatusChip";

interface WalletHeaderProps {
  label: string;
  networkLabel: string;
  statusLabel: string;
  statusTone: WalletTone;
  lastSyncedRelative: string;
}

export function WalletHeader({
  label,
  networkLabel,
  statusLabel,
  statusTone,
  lastSyncedRelative,
}: WalletHeaderProps) {
  return (
    <header className="rounded-[2rem] border border-white/10 bg-slate-950/70 p-4 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur sm:p-5">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div className="flex min-w-0 items-center gap-4">
          <BrandMark compact />

          <div className="min-w-0 space-y-1">
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
              Active wallet
            </p>
            <h1 className="truncate text-2xl font-semibold tracking-tight text-slate-50 sm:text-3xl">
              {label}
            </h1>
            <p className="max-w-2xl text-sm leading-6 text-slate-400">
              Manage balances, notes, and transfers from one compact wallet
              workspace.
            </p>
          </div>
        </div>

        <div className="flex flex-wrap gap-2 lg:justify-end">
          <StatusChip label="Network" value={networkLabel} compact />
          <StatusChip
            label="Status"
            value={statusLabel}
            tone={statusTone}
            compact
          />
          <StatusChip label="Last sync" value={lastSyncedRelative} compact />
        </div>
      </div>
    </header>
  );
}
