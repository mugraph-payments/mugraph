import type { ReactNode } from "react";

interface ActionSummaryCardProps {
  eyebrow: string;
  title: string;
  description: string;
  tone?: "neutral" | "positive" | "warning";
  footer?: ReactNode;
}

const toneClasses = {
  neutral: "border-white/10 bg-white/[0.03]",
  positive: "border-teal-300/20 bg-teal-400/10",
  warning: "border-amber-300/20 bg-amber-400/10",
};

export function ActionSummaryCard({
  eyebrow,
  title,
  description,
  tone = "neutral",
  footer,
}: ActionSummaryCardProps) {
  return (
    <div
      className={`rounded-[1.5rem] border p-4 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] ${toneClasses[tone]}`}
    >
      <p className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
        {eyebrow}
      </p>
      <h3 className="mt-2 text-base font-medium text-slate-100">{title}</h3>
      <p className="mt-2 text-sm leading-6 text-slate-300">{description}</p>
      {footer ? <div className="mt-4">{footer}</div> : null}
    </div>
  );
}
