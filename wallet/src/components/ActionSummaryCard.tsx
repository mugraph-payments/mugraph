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
    <div className={`overflow-hidden rounded-[1.5rem] border p-5 sm:p-6 ${toneClasses[tone]}`}>
      <div className="grid gap-3">
        <p className="wallet-kicker text-slate-500">{eyebrow}</p>
        <div className="grid gap-2">
          <h3 className="wallet-heading text-lg font-medium text-slate-100">{title}</h3>
          <p className="wallet-copy break-words text-base leading-7 text-slate-300">
            {description}
          </p>
        </div>
      </div>
      {footer ? <div className="mt-5">{footer}</div> : null}
    </div>
  );
}
