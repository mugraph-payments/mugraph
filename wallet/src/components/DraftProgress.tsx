interface DraftProgressProps {
  label: string;
  completed: number;
  total: number;
  summary: string;
  tone?: "neutral" | "positive" | "warning";
}

const toneClasses = {
  neutral: {
    shell: "border-white/10 bg-white/[0.03]",
    fill: "bg-slate-200/75",
    copy: "text-slate-300",
    stat: "text-slate-100",
  },
  positive: {
    shell: "border-teal-300/20 bg-teal-400/10",
    fill: "bg-teal-300",
    copy: "text-slate-200",
    stat: "text-teal-50",
  },
  warning: {
    shell: "border-amber-300/20 bg-amber-400/10",
    fill: "bg-amber-300",
    copy: "text-slate-200",
    stat: "text-amber-50",
  },
};

export function DraftProgress({
  label,
  completed,
  total,
  summary,
  tone = "neutral",
}: DraftProgressProps) {
  const safeTotal = Math.max(total, 1);
  const clampedCompleted = Math.min(Math.max(completed, 0), safeTotal);
  const percent = Math.round((clampedCompleted / safeTotal) * 100);
  const classes = toneClasses[tone];

  return (
    <section className={`rounded-[1.5rem] border p-5 sm:p-6 ${classes.shell}`}>
      <div className="grid gap-4 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-start">
        <div className="space-y-2">
          <p className="wallet-kicker text-slate-500">{label}</p>
          <p className={`text-base leading-7 ${classes.copy}`}>{summary}</p>
        </div>

        <div className="sm:text-right">
          <p className={`wallet-data text-xl font-semibold ${classes.stat}`}>
            {clampedCompleted}/{safeTotal}
          </p>
          <p className="wallet-kicker text-slate-500">complete</p>
        </div>
      </div>

      <div className="mt-5 h-2 overflow-hidden rounded-full bg-slate-950/60" aria-hidden="true">
        <div
          className={`h-full rounded-full ${classes.fill}`}
          style={{ width: `${percent}%`, transition: "width 0.28s cubic-bezier(0.16, 1, 0.3, 1)" }}
        />
      </div>

      <div className="sr-only" aria-live="polite">
        {clampedCompleted} of {safeTotal} steps complete.
      </div>
    </section>
  );
}
