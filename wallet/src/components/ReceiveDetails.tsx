interface ReceiveDetailsProps {
  label: string;
  delegatePkShort: string;
  scriptAddressShort: string;
  networkLabel: string;
  lastSyncedRelative: string;
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

export function ReceiveDetails({
  label,
  delegatePkShort,
  scriptAddressShort,
  networkLabel,
  lastSyncedRelative,
  isEmpty = false,
}: ReceiveDetailsProps) {
  return (
    <div className="mt-4 rounded-[1.5rem] border border-white/10 bg-white/[0.02] p-4">
      <p className="text-sm leading-6 text-slate-300">
        {isEmpty
          ? `Receive is unavailable in the empty preview because ${label} has no active wallet context loaded yet.`
          : `Share the active script address when funding ${label}. The delegate context stays visible so incoming notes can be tied back to the correct issuer without exposing long identifiers in full.`}
      </p>

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        <DetailField
          label="Script address"
          value={isEmpty ? "Unavailable in empty preview" : scriptAddressShort}
        />
        <DetailField
          label="Delegate"
          value={isEmpty ? "Unavailable in empty preview" : delegatePkShort}
        />
        <DetailField label="Network" value={networkLabel} />
        <DetailField label="Last sync" value={lastSyncedRelative} />
      </div>
    </div>
  );
}
