import { LockKey, Pulse, ScanSmiley } from "@phosphor-icons/react";
import type { ReactNode } from "react";
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

function MetaRow({
  label,
  value,
  icon,
}: {
  label: string;
  value: string;
  icon: ReactNode;
}) {
  return (
    <div className="wallet-subtle-card p-3">
      <div className="flex items-center gap-3">
        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-white/[0.05] text-slate-100 ring-1 ring-white/10">
          {icon}
        </div>
        <div className="min-w-0">
          <p className="wallet-kicker text-slate-500">{label}</p>
          <p className="wallet-code mt-1 break-all text-sm text-slate-100">{value}</p>
        </div>
      </div>
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
    <aside className="wallet-panel p-5">
      <div className="space-y-3">
        <div>
          <p className="wallet-kicker text-slate-500">Wallet context</p>
          <h2 className="wallet-heading mt-2 text-xl font-semibold tracking-tight text-slate-50">
            {label}
          </h2>
          <p className="wallet-copy mt-2 text-sm leading-6 text-slate-400">
            Keep the funding target, delegate identity, and live network posture visible while moving through private transfers and settlement flows.
          </p>
        </div>

        <div className="flex flex-wrap gap-2">
          <StatusChip label="Network" value={networkLabel} compact />
          <StatusChip label="Status" value={statusLabel} tone={statusTone} compact />
          <StatusChip label="Updated" value={lastSyncedRelative} compact />
        </div>
      </div>

      <div className="mt-5 grid gap-3">
        <MetaRow
          label="Delegate key"
          value={delegatePkShort}
          icon={<LockKey className="h-4.5 w-4.5" weight="duotone" />}
        />
        <MetaRow
          label="Script address"
          value={scriptAddressShort}
          icon={<ScanSmiley className="h-4.5 w-4.5" weight="duotone" />}
        />
        <MetaRow
          label="Sync posture"
          value={`${statusLabel} on ${networkLabel}`}
          icon={<Pulse className="h-4.5 w-4.5" weight="duotone" />}
        />
      </div>
    </aside>
  );
}
