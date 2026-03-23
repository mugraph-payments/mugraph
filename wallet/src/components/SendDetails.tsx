interface SendDetailsProps {
  noteCount: number;
  topAssetLabel: string;
  pendingActivityCount: number;
}

function DetailField({
  label,
  value,
}: {
  label: string;
  value: string;
}) {
  return (
    <div className="rounded-[1.25rem] border border-white/10 bg-white/[0.03] p-3">
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
}: SendDetailsProps) {
  return (
    <div className="mt-4 rounded-[1.5rem] border border-white/10 bg-white/[0.02] p-4">
      <p className="text-sm leading-6 text-slate-300">
        Sending starts from the private note inventory. This stub view keeps the
        operator focused on how many notes are available, which asset dominates
        the wallet, and whether pending work should be cleared before preparing
        a transfer.
      </p>

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        <DetailField label="Spendable notes" value={`${noteCount}`} />
        <DetailField label="Largest holding" value={topAssetLabel} />
        <DetailField
          label="Pending queue"
          value={`${pendingActivityCount} item${pendingActivityCount === 1 ? "" : "s"}`}
        />
        <DetailField label="Transfer mode" value="Private note transfer" />
      </div>
    </div>
  );
}
