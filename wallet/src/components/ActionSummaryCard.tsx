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
  positive: "border-white/10 bg-white/[0.04]",
  warning: "border-white/10 bg-white/[0.04]",
};

export function ActionSummaryCard({
  eyebrow,
  title,
  description,
  tone = "neutral",
  footer,
}: ActionSummaryCardProps) {
  return (
    <div className={`overflow-hidden rounded-[1.25rem] border p-4 sm:p-5 ${toneClasses[tone]}`}>
      <div className="grid gap-2">
        <p className="wallet-kicker text-slate-500">{eyebrow}</p>
        <h3 className="wallet-heading text-[1.375rem] text-slate-100">{title}</h3>
        <p className="wallet-copy break-words text-slate-300">{description}</p>
      </div>
      {footer ? <div className="mt-4">{footer}</div> : null}
    </div>
  );
}
