import {
  ArrowCircleDown,
  ArrowCircleUp,
  ArrowSquareIn,
  ArrowSquareOut,
} from "@phosphor-icons/react";
import { motion, useReducedMotion } from "framer-motion";
import type { ComponentType } from "react";
import type { WalletPreviewStateId } from "../data/walletPreviewStates";
import type { WalletActionView } from "../lib/walletView";
import type { WalletActionKind } from "../types/wallet";

interface ActionGridProps {
  actions: WalletActionView[];
  selectedActionId: WalletActionKind;
  onActionSelect: (actionId: WalletActionKind) => void;
  previewStateId: WalletPreviewStateId;
}

const actionIcons: Record<
  WalletActionKind,
  ComponentType<{ className?: string; weight?: "regular" | "duotone" }>
> = {
  send: ArrowSquareOut,
  receive: ArrowSquareIn,
  deposit: ArrowCircleDown,
  withdraw: ArrowCircleUp,
};

const previewToneClasses: Record<
  WalletPreviewStateId,
  { shell: string; eyebrow: string; accent: string; selected: string; icon: string }
> = {
  ready: {
    shell: "border-white/10 bg-slate-950/60",
    eyebrow: "text-slate-500",
    accent: "text-slate-400",
    selected: "border-teal-300/30 bg-teal-400/10",
    icon: "bg-teal-400/10 text-teal-100 ring-teal-300/20",
  },
  empty: {
    shell: "border-white/10 bg-slate-950/60",
    eyebrow: "text-slate-500",
    accent: "text-slate-400",
    selected: "border-slate-300/20 bg-white/[0.06]",
    icon: "bg-white/[0.08] text-slate-100 ring-white/10",
  },
  syncing: {
    shell:
      "border-amber-400/20 bg-[linear-gradient(180deg,rgba(245,158,11,0.08),rgba(2,6,23,0.72))]",
    eyebrow: "text-amber-300/75",
    accent: "text-amber-100/80",
    selected: "border-amber-300/30 bg-amber-400/10",
    icon: "bg-amber-400/10 text-amber-100 ring-amber-300/20",
  },
  attention: {
    shell: "border-rose-400/20 bg-[linear-gradient(180deg,rgba(244,63,94,0.08),rgba(2,6,23,0.72))]",
    eyebrow: "text-rose-300/75",
    accent: "text-rose-100/80",
    selected: "border-rose-300/30 bg-rose-400/10",
    icon: "bg-rose-400/10 text-rose-100 ring-rose-300/20",
  },
};

export function ActionGrid({
  actions,
  selectedActionId,
  onActionSelect,
  previewStateId,
}: ActionGridProps) {
  const tone = previewToneClasses[previewStateId];
  const prefersReducedMotion = useReducedMotion();

  return (
    <motion.section
      initial={prefersReducedMotion ? false : { opacity: 0.98, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.22, ease: [0.16, 1, 0.3, 1] }}
      className={`rounded-[2rem] border p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur ${tone.shell}`}
    >
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className={`text-xs uppercase tracking-[0.22em] ${tone.eyebrow}`}>Primary actions</p>
          <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-50">
            Move through the wallet from the actions first
          </h2>
        </div>
        <p className={`max-w-xl text-sm leading-6 ${tone.accent}`}>
          Send, receive, deposit, and withdraw stay near the top so the shell behaves like a real
          wallet instead of a passive dashboard.
        </p>
      </div>

      <div className="mt-5 grid gap-3 sm:grid-cols-2">
        {actions.map((action) => {
          const Icon = actionIcons[action.id];
          const isSelected = action.id === selectedActionId;

          return (
            <motion.button
              key={action.id}
              type="button"
              onClick={() => onActionSelect(action.id)}
              whileHover={prefersReducedMotion ? undefined : { y: -2 }}
              whileTap={prefersReducedMotion ? undefined : { scale: 0.985 }}
              transition={{ type: "spring", stiffness: 280, damping: 22 }}
              className={`rounded-[1.5rem] border p-4 text-left will-change-transform ${
                isSelected ? tone.selected : "border-white/10 bg-white/[0.03]"
              }`}
            >
              <div className="flex items-start justify-between gap-3">
                <div
                  className={`flex h-11 w-11 items-center justify-center rounded-2xl ring-1 ${tone.icon}`}
                >
                  <Icon className="h-6 w-6" weight="duotone" />
                </div>
                <span className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
                  {action.id}
                </span>
              </div>

              <div className="mt-4 space-y-2">
                <h3 className="text-base font-medium text-slate-100">{action.label}</h3>
                <p className={`text-sm leading-6 ${tone.accent}`}>{action.helper}</p>
              </div>
            </motion.button>
          );
        })}
      </div>
    </motion.section>
  );
}
