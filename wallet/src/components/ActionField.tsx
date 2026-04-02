interface ActionFieldProps {
  label: string;
  value: string;
}

export function ActionField({ label, value }: ActionFieldProps) {
  return (
    <div className="wallet-subtle-card min-w-0 overflow-hidden p-4">
      <p className="wallet-kicker text-slate-500">{label}</p>
      <p className="wallet-data mt-2 break-all text-base leading-6 text-slate-100">{value}</p>
    </div>
  );
}
