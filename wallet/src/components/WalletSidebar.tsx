import type { WalletTone } from "../lib/walletView";
import { StatusChip } from "./StatusChip";

interface WalletSidebarProps {
  label: string;
  networkLabel: string;
  statusLabel: string;
  statusTone: WalletTone;
  delegatePkShort: string;
  scriptAddressShort: string;
  lastSyncedRelative: string;
}

function IdentityRow({
  label,
  value,
}: {
  label: string;
  value: string;
}) {
  return (
    <div className="rounded-[1.25rem] border border-white/10 bg-white/[0.03] p-3">
      <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
        {label}
      </p>
      <p className="mt-2 break-all text-sm text-slate-100">{value}</p>
    </div>
  );
}

export function WalletSidebar({
  label,
  networkLabel,
  statusLabel,
  statusTone,
  delegatePkShort,
  scriptAddressShort,
  lastSyncedRelative,
}: WalletSidebarProps) {
  return (
    <aside className="hidden rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur lg:grid lg:content-start lg:gap-4">
      <div className="space-y-2">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
          Wallet identity
        </p>
        <h2 className="text-xl font-semibold tracking-tight text-slate-50">
          {label}
        </h2>
        <p className="text-sm leading-6 text-slate-400">
          Keep the delegate context, network posture, and funding target visible
          while moving through wallet actions.
        </p>
      </div>

      <div className="flex flex-wrap gap-2">
        <StatusChip label="Network" value={networkLabel} />
        <StatusChip label="Status" value={statusLabel} tone={statusTone} />
        <StatusChip label="Last sync" value={lastSyncedRelative} />
      </div>

      <div className="grid gap-3">
        <IdentityRow label="Delegate" value={delegatePkShort} />
        <IdentityRow label="Funding target" value={scriptAddressShort} />
      </div>
    </aside>
  );
}
