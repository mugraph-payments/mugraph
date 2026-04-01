import type { WalletTone } from "../lib/walletView";
import type { WalletActiveDestination } from "../types/wallet";
import { BrandMark } from "./BrandMark";
import { StatusChip } from "./StatusChip";

interface WalletHeaderProps {
  label: string;
  networkLabel: string;
  statusLabel: string;
  statusTone: WalletTone;
  lastSyncedRelative: string;
  activeDestination: WalletActiveDestination;
}

export function WalletHeader({
  label,
  networkLabel,
  statusLabel,
  statusTone,
  lastSyncedRelative,
  activeDestination,
}: WalletHeaderProps) {
  const isHome = activeDestination === "home";

  return (
    <header className="wallet-panel overflow-hidden px-4 py-4 sm:px-5">
      <div className="flex flex-col gap-4">
        <div className="min-w-0">
          <div className="flex items-center gap-3">
            <BrandMark compact={isHome} />
            <div className="min-w-0">
              <p className="wallet-kicker text-slate-500">{isHome ? "Wallet" : "Wallet app"}</p>
              <h1 className="wallet-heading truncate text-xl font-semibold tracking-tight text-slate-50 sm:text-2xl">
                {label}
              </h1>
              {isHome ? (
                <p className="mt-2 hidden text-sm leading-6 text-slate-400 lg:block">
                  {networkLabel} network · synced {lastSyncedRelative}
                </p>
              ) : null}
            </div>
          </div>
        </div>

        {isHome ? null : (
          <div className="flex flex-wrap gap-2">
            <StatusChip label="Mode" value="Stub" compact />
            <StatusChip label="Network" value={networkLabel} compact />
            <StatusChip label="Status" value={statusLabel} tone={statusTone} compact />
            <StatusChip label="Last sync" value={lastSyncedRelative} compact />
          </div>
        )}
      </div>
    </header>
  );
}
