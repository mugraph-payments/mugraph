interface DepositDetailsProps {
  scriptAddressShort: string;
  delegatePkShort: string;
  latestDepositReference: string;
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

export function DepositDetails({
  scriptAddressShort,
  delegatePkShort,
  latestDepositReference,
  pendingActivityCount,
}: DepositDetailsProps) {
  return (
    <div className="mt-4 rounded-[1.5rem] border border-white/10 bg-white/[0.02] p-4">
      <p className="text-sm leading-6 text-slate-300">
        Deposit tracking keeps the funding target, delegate context, and latest
        on-chain deposit reference in one place while new notes are moving into
        the wallet inventory.
      </p>

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        <DetailField label="Funding target" value={scriptAddressShort} />
        <DetailField label="Delegate" value={delegatePkShort} />
        <DetailField label="Latest deposit ref" value={latestDepositReference} />
        <DetailField
          label="Pending activity"
          value={`${pendingActivityCount} item${pendingActivityCount === 1 ? "" : "s"}`}
        />
      </div>
    </div>
  );
}
