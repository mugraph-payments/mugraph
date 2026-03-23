interface BrandMarkProps {
  compact?: boolean;
}

export function BrandMark({ compact = false }: BrandMarkProps) {
  return (
    <div className="flex items-center gap-3">
      <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-white/95 shadow-[0_16px_40px_-24px_rgba(45,212,191,0.75)] ring-1 ring-white/20">
        <img
          src="/mugraph-mark.svg"
          alt="Mugraph mark"
          className="h-7 w-7"
        />
      </div>

      {!compact ? (
        <div className="space-y-1">
          <p className="text-xs uppercase tracking-[0.3em] text-teal-300/70">
            Mugraph wallet
          </p>
          <p className="text-sm font-medium text-slate-100">
            Private Cardano payments workspace
          </p>
        </div>
      ) : null}
    </div>
  );
}
