import { BrandMark } from "./BrandMark";

interface WalletHeaderProps {
  label: string;
  networkLabel: string;
  lastSyncedRelative: string;
}

export function WalletHeader({ label, networkLabel, lastSyncedRelative }: WalletHeaderProps) {
  return (
    <header className="overflow-hidden px-1 py-1">
      <div className="flex items-center gap-3">
        <BrandMark />
        <div className="min-w-0">
          <h1 className="truncate text-base font-semibold tracking-tight text-slate-50">{label}</h1>
          <p className="mt-0.5 text-xs text-slate-400">
            {networkLabel} · synced {lastSyncedRelative}
          </p>
        </div>
      </div>
    </header>
  );
}
