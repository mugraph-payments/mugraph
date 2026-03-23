import type { WalletPreviewStateId } from "../data/walletPreviewStates";

interface ReceiveDetailsProps {
  label: string;
  delegatePkShort: string;
  scriptAddressShort: string;
  networkLabel: string;
  lastSyncedRelative: string;
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

export function ReceiveDetails({
  label,
  delegatePkShort,
  scriptAddressShort,
  networkLabel,
  lastSyncedRelative,
  isEmpty = false,
  previewStateId = "ready",
}: ReceiveDetailsProps) {
  const classes = surfaceToneClasses[previewStateId];

  return (
    <div className={`mt-4 rounded-[1.5rem] p-4 ${classes.shell}`}>
      <p className={`text-sm leading-6 ${classes.copy}`}>
        {isEmpty
          ? `Receive is unavailable in the empty preview because ${label} has no active wallet context loaded yet.`
          : `Share the active script address when funding ${label}. The delegate context stays visible so incoming notes can be tied back to the correct issuer without exposing long identifiers in full.`}
      </p>

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        <DetailField
          label="Script address"
          value={isEmpty ? "Unavailable in empty preview" : scriptAddressShort}
          tone={previewStateId}
        />
        <DetailField
          label="Delegate"
          value={isEmpty ? "Unavailable in empty preview" : delegatePkShort}
          tone={previewStateId}
        />
        <DetailField label="Network" value={networkLabel} tone={previewStateId} />
        <DetailField
          label="Last sync"
          value={lastSyncedRelative}
          tone={previewStateId}
        />
      </div>
    </div>
  );
}
