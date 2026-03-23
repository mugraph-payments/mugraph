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
    <header className="wallet-panel px-4 py-4 sm:px-5">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div className="min-w-0">
          <div className="flex items-center gap-3">
            <BrandMark />
            <div className="hidden h-8 w-px bg-white/10 lg:block" />
            <div className="min-w-0">
              <p className="wallet-kicker text-slate-500">Operator wallet preview</p>
              <h1 className="wallet-heading truncate text-lg font-semibold tracking-tight text-slate-50 sm:text-xl">
                {label}
              </h1>
            </div>
          </div>
        </div>

        <div className="flex flex-wrap gap-2 lg:justify-end">
          <StatusChip label="Mode" value="Stub" compact />
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
