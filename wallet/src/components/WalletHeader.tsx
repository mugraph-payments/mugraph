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
    <header className="wallet-panel p-4 sm:p-5">
      <div className="flex flex-col gap-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex min-w-0 items-center gap-3">
            <BrandMark compact />

            <div className="min-w-0 space-y-1">
              <p className="wallet-kicker text-slate-400">Primary wallet</p>
              <h1 className="wallet-heading truncate text-xl font-semibold tracking-tight text-slate-50 sm:text-2xl">
                {label}
              </h1>
            </div>
          </div>

          <button
            type="button"
            className="wallet-interactive hidden items-center rounded-full border border-white/10 bg-white/[0.04] px-4 text-xs font-medium text-slate-200 shadow-[inset_0_1px_0_rgba(255,255,255,0.06)] sm:inline-flex"
          >
            Wallet home
          </button>
        </div>

        <div className="flex flex-wrap gap-2">
          <StatusChip label="Network" value={networkLabel} compact />
          <StatusChip
            label="Status"
            value={statusLabel}
            tone={statusTone}
            compact
          />
          <StatusChip label="Updated" value={lastSyncedRelative} compact />
        </div>
      </div>
    </header>
  );
}
