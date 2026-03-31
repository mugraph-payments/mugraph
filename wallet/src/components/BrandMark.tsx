interface BrandMarkProps {
  compact?: boolean;
}

export function BrandMark({ compact = false }: BrandMarkProps) {
  return (
    <div className="flex items-center gap-3">
      <div
        className={`flex items-center justify-center rounded-xl bg-white/95 ring-1 ring-white/15 ${
          compact ? "h-10 w-10" : "h-11 w-11"
        }`}
        style={{ boxShadow: "0 4px 12px -4px rgba(45,212,191,0.4)" }}
      >
        <img
          src="/mugraph-mark.svg"
          alt="Mugraph mark"
          className={compact ? "h-6 w-6" : "h-7 w-7"}
        />
      </div>

      {compact ? null : (
        <div className="min-w-0 space-y-0.5">
          <p className="wallet-kicker text-teal-200/80">Mugraph Wallet</p>
          <p className="truncate text-sm text-slate-300">Private Cardano settlement cockpit</p>
        </div>
      )}
    </div>
  );
}
