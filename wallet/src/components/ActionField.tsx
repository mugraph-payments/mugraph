interface ActionFieldProps {
  label: string;
  value: string;
}

export function ActionField({ label, value }: ActionFieldProps) {
  return (
    <div className="wallet-subtle-card p-3">
      <p className="wallet-kicker text-slate-500">{label}</p>
      <p className="wallet-data mt-2 break-words text-sm text-slate-100">{value}</p>
    </div>
  );
}
