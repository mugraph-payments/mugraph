interface ActionFieldProps {
  label: string;
  value: string;
}

export function ActionField({ label, value }: ActionFieldProps) {
  return (
    <div className="rounded-[1.25rem] border border-white/10 bg-white/[0.03] p-3">
      <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
        {label}
      </p>
      <p className="mt-2 text-sm text-slate-100">{value}</p>
    </div>
  );
}
