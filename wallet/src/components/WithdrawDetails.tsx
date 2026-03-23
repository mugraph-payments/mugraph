interface WithdrawDetailsProps {
  latestWithdrawReference: string;
  pendingActivityCount: number;
  scriptAddressShort: string;
  topAssetLabel: string;
  isEmpty?: boolean;
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

export function WithdrawDetails({
  latestWithdrawReference,
  pendingActivityCount,
  scriptAddressShort,
  topAssetLabel,
  isEmpty = false,
}: WithdrawDetailsProps) {
  return (
    <div className="mt-4 rounded-[1.5rem] border border-white/10 bg-white/[0.02] p-4">
      <p className="text-sm leading-6 text-slate-300">
        {isEmpty
          ? "Withdraw is unavailable in the empty preview because there is no settlement history or active inventory loaded yet."
          : "Withdrawal prep keeps the most recent settlement reference, current queue pressure, and the active script context visible while notes are being assembled for on-chain settlement."}
      </p>

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        <DetailField
          label="Latest withdraw ref"
          value={isEmpty ? "No withdraw reference yet" : latestWithdrawReference}
        />
        <DetailField
          label="Pending queue"
          value={isEmpty ? "0 items" : `${pendingActivityCount} item${pendingActivityCount === 1 ? "" : "s"}`}
        />
        <DetailField
          label="Settlement path"
          value={isEmpty ? "Unavailable in empty preview" : scriptAddressShort}
        />
        <DetailField
          label="Primary asset in view"
          value={isEmpty ? "No holdings loaded" : topAssetLabel}
        />
      </div>
    </div>
  );
}
