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
    <div className="wallet-subtle-card p-3">
      <p className="wallet-kicker text-slate-500">{label}</p>
      <p className="wallet-code mt-2 break-all text-sm text-slate-100">{value}</p>
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
    <aside className="wallet-panel hidden self-start p-4 lg:grid lg:max-h-[calc(100dvh-9rem)] lg:content-start lg:gap-4 lg:overflow-y-auto lg:overscroll-contain xl:sticky xl:top-4">
      <div className="space-y-2">
        <p className="wallet-kicker text-slate-500">Wallet identity</p>
        <h2 className="wallet-heading text-lg font-semibold tracking-tight text-slate-50 xl:text-xl">
          {label}
        </h2>
        <p className="wallet-copy text-sm leading-6 text-slate-400">
          Keep delegate and funding context visible while moving through wallet actions.
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
