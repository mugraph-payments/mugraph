interface WalletHeaderProps {
  label: string;
  networkLabel: string;
  statusLabel: string;
  lastSyncedRelative: string;
}

function HeaderChip({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-slate-300">
      <span className="text-slate-500">{label}</span>{" "}
      <span className="text-slate-100">{value}</span>
    </div>
  );
}

export function WalletHeader({
  label,
  networkLabel,
  statusLabel,
  lastSyncedRelative,
}: WalletHeaderProps) {
  return (
    <header className="rounded-[2rem] border border-white/10 bg-slate-950/70 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur">
      <div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
        <div className="space-y-2">
          <p className="text-xs uppercase tracking-[0.3em] text-teal-300/70">
            Wallet shell preview
          </p>
          <div className="space-y-1">
            <h1 className="text-3xl font-semibold tracking-tight text-slate-50 sm:text-4xl">
              {label}
            </h1>
            <p className="max-w-2xl text-sm leading-6 text-slate-400 sm:text-base">
              A responsive shell for the action-first wallet. Summary surfaces,
              inventories, and action details slot into the regions below next.
            </p>
          </div>
        </div>

        <div className="flex flex-wrap gap-2 md:justify-end">
          <HeaderChip label="Network" value={networkLabel} />
          <HeaderChip label="Status" value={statusLabel} />
          <HeaderChip label="Last sync" value={lastSyncedRelative} />
        </div>
      </div>
    </header>
  );
}
