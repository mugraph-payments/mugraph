import { BrandMark } from "./BrandMark";

interface WalletHeaderProps {
  label: string;
  networkLabel: string;
  lastSyncedRelative: string;
}

export function WalletHeader({ label, networkLabel, lastSyncedRelative }: WalletHeaderProps) {
  return (
    <header className="wallet-panel-soft overflow-hidden px-4 py-4 sm:px-5 sm:py-5">
      <div className="flex items-start gap-3">
        <BrandMark />
        <div className="min-w-0 flex-1">
          <h1 className="wallet-heading truncate text-[1.375rem] text-slate-50">{label}</h1>
          <p className="mt-1 text-[0.9375rem] text-slate-400">
            {networkLabel} · synced {lastSyncedRelative}
          </p>
        </div>
      </div>
    </header>
  );
}
