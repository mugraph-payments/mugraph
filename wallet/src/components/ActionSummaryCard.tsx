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
      className={`rounded-3xl border p-4 ${toneClasses[tone]}`}
      style={{ boxShadow: "var(--wallet-card-shadow)" }}
    >
      <p className="wallet-kicker text-slate-500">{eyebrow}</p>
      <h3 className="wallet-heading mt-2 text-lg font-medium text-slate-100">{title}</h3>
      <p className="wallet-copy mt-2 text-base leading-7 text-slate-300">{description}</p>
      {footer ? <div className="mt-4">{footer}</div> : null}
    </div>
  );
}
