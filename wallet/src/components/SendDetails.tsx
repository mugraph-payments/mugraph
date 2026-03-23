import type { WalletPreviewStateId } from "../data/walletPreviewStates";

interface SendDetailsProps {
  noteCount: number;
  topAssetLabel: string;
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

export function SendDetails({
  noteCount,
  topAssetLabel,
  pendingActivityCount,
  isEmpty = false,
  previewStateId = "ready",
}: SendDetailsProps) {
  const classes = surfaceToneClasses[previewStateId];

  return (
    <div className={`mt-4 rounded-[1.5rem] p-4 ${classes.shell}`}>
      <p className={`text-sm leading-6 ${classes.copy}`}>
        {isEmpty
          ? "Send is unavailable in the empty preview because there are no spendable notes loaded for transfer prep."
          : "Sending starts from the private note inventory. This stub view keeps the operator focused on how many notes are available, which asset dominates the wallet, and whether pending work should be cleared before preparing a transfer."}
      </p>

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        <DetailField
          label="Spendable notes"
          value={isEmpty ? "0" : `${noteCount}`}
          tone={previewStateId}
        />
        <DetailField
          label="Largest holding"
          value={isEmpty ? "No holdings loaded" : topAssetLabel}
          tone={previewStateId}
        />
        <DetailField
          label="Pending queue"
          value={isEmpty ? "0 items" : `${pendingActivityCount} item${pendingActivityCount === 1 ? "" : "s"}`}
          tone={previewStateId}
        />
        <DetailField
          label="Transfer mode"
          value={isEmpty ? "Unavailable in empty preview" : "Private note transfer"}
          tone={previewStateId}
        />
      </div>
    </div>
  );
}
