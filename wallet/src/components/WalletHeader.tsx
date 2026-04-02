import { BrandMark } from "./BrandMark";

interface WalletHeaderProps {
  label: string;
  networkLabel: string;
  lastSyncedRelative: string;
}

export function WalletHeader({ label, networkLabel, lastSyncedRelative }: WalletHeaderProps) {
  return (
    <header className="wallet-panel-soft overflow-hidden px-4 py-4 sm:px-5 sm:py-5">
      <div className="grid gap-4">
        <div className="flex items-start gap-3">
          <BrandMark />
          <div className="min-w-0 flex-1">
            <p className="wallet-kicker text-slate-500">Mugraph wallet</p>
            <h1 className="wallet-heading truncate text-lg font-semibold tracking-tight text-slate-50">
              {label}
            </h1>
          </div>
        </div>

        <div className="wallet-inline-metrics text-xs text-slate-400">
          <span className="wallet-chip-neutral rounded-full border px-3 py-1.5">
            {networkLabel}
          </span>
          <span className="wallet-chip-neutral rounded-full border px-3 py-1.5">
            Synced {lastSyncedRelative}
          </span>
        </div>
      </div>
    </header>
  );
}
