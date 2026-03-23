import type { WalletPreviewStateId } from "../data/walletPreviewStates";

interface WithdrawDetailsProps {
  latestWithdrawReference: string;
  pendingActivityCount: number;
  scriptAddressShort: string;
  topAssetLabel: string;
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

export function WithdrawDetails({
  latestWithdrawReference,
  pendingActivityCount,
  scriptAddressShort,
  topAssetLabel,
  isEmpty = false,
  previewStateId = "ready",
}: WithdrawDetailsProps) {
  const classes = surfaceToneClasses[previewStateId];

  return (
    <div className={`mt-4 rounded-[1.5rem] p-4 ${classes.shell}`}>
      <p className={`text-sm leading-6 ${classes.copy}`}>
        {isEmpty
          ? "Withdraw is unavailable in the empty preview because there is no settlement history or active inventory loaded yet."
          : "Withdrawal prep keeps the most recent settlement reference, current queue pressure, and the active script context visible while notes are being assembled for on-chain settlement."}
      </p>

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        <DetailField
          label="Latest withdraw ref"
          value={isEmpty ? "No withdraw reference yet" : latestWithdrawReference}
          tone={previewStateId}
        />
        <DetailField
          label="Pending queue"
          value={isEmpty ? "0 items" : `${pendingActivityCount} item${pendingActivityCount === 1 ? "" : "s"}`}
          tone={previewStateId}
        />
        <DetailField
          label="Settlement path"
          value={isEmpty ? "Unavailable in empty preview" : scriptAddressShort}
          tone={previewStateId}
        />
        <DetailField
          label="Primary asset in view"
          value={isEmpty ? "No holdings loaded" : topAssetLabel}
          tone={previewStateId}
        />
      </div>
    </div>
  );
}
