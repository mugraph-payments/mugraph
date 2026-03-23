import type { WalletPreviewStateId } from "../data/walletPreviewStates";

interface DepositDetailsProps {
  scriptAddressShort: string;
  delegatePkShort: string;
  latestDepositReference: string;
  pendingActivityCount: number;
  isEmpty?: boolean;
  previewStateId?: WalletPreviewStateId;
}

const surfaceToneClasses: Record<
  WalletPreviewStateId,
  { shell: string; copy: string; field: string }
> = {
  ready: {
    shell: "border-white/10 bg-white/[0.02]",
    copy: "text-slate-300",
    field: "border-white/10 bg-white/[0.03]",
  },
  empty: {
    shell: "border-white/10 bg-white/[0.02]",
    copy: "text-slate-300",
    field: "border-white/10 bg-white/[0.03]",
  },
  syncing: {
    shell: "border-amber-400/20 bg-amber-400/[0.05]",
    copy: "text-amber-50/90",
    field: "border-amber-400/15 bg-amber-400/[0.06]",
  },
  attention: {
    shell: "border-rose-400/20 bg-rose-400/[0.05]",
    copy: "text-rose-50/90",
    field: "border-rose-400/15 bg-rose-400/[0.06]",
  },
};

function DetailField({
  label,
  value,
  tone,
}: {
  label: string;
  value: string;
  tone: WalletPreviewStateId;
}) {
  const classes = surfaceToneClasses[tone];

  return (
    <div className={`rounded-[1.25rem] p-3 ${classes.field}`}>
      <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
        {label}
      </p>
      <p className="mt-2 text-sm text-slate-100">{value}</p>
    </div>
  );
}

export function DepositDetails({
  scriptAddressShort,
  delegatePkShort,
  latestDepositReference,
  pendingActivityCount,
  isEmpty = false,
  previewStateId = "ready",
}: DepositDetailsProps) {
  const classes = surfaceToneClasses[previewStateId];

  return (
    <div className={`mt-4 rounded-[1.5rem] p-4 ${classes.shell}`}>
      <p className={`text-sm leading-6 ${classes.copy}`}>
        {isEmpty
          ? "Deposit tracking is unavailable in the empty preview because there is no funding target or activity history loaded yet."
          : "Deposit tracking keeps the funding target, delegate context, and latest on-chain deposit reference in one place while new notes are moving into the wallet inventory."}
      </p>

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        <DetailField
          label="Funding target"
          value={isEmpty ? "Unavailable in empty preview" : scriptAddressShort}
          tone={previewStateId}
        />
        <DetailField
          label="Delegate"
          value={isEmpty ? "Unavailable in empty preview" : delegatePkShort}
          tone={previewStateId}
        />
        <DetailField
          label="Latest deposit ref"
          value={isEmpty ? "No deposit reference yet" : latestDepositReference}
          tone={previewStateId}
        />
        <DetailField
          label="Pending activity"
          value={isEmpty ? "0 items" : `${pendingActivityCount} item${pendingActivityCount === 1 ? "" : "s"}`}
          tone={previewStateId}
        />
      </div>
    </div>
  );
}
