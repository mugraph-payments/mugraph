import {
  ArrowCircleDown,
  ArrowCircleUp,
  ArrowSquareIn,
  ArrowSquareOut,
} from "@phosphor-icons/react";
import { motion, useReducedMotion } from "framer-motion";
import type { ComponentType } from "react";
import type { WalletShellActionView } from "../lib/walletView";
import type { WalletActionKind } from "../types/wallet";

interface WalletActionNavProps {
  actions: WalletShellActionView[];
  onActionSelect: (actionId: WalletActionKind) => void;
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

export function WalletActionNav({
  actions,
  onActionSelect,
}: WalletActionNavProps) {
  const prefersReducedMotion = useReducedMotion();

  return (
    <motion.section
      initial={prefersReducedMotion ? false : { opacity: 0.98, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.22, ease: [0.16, 1, 0.3, 1] }}
      className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur"
    >
      <div className="flex flex-col gap-2">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
          Wallet actions
        </p>
        <h2 className="text-xl font-semibold tracking-tight text-slate-50">
          Choose the active flow
        </h2>
        <p className="text-sm leading-6 text-slate-400">
          Send, receive, deposit, and withdraw stay grouped in the action region
          so the shell behaves like a wallet workspace instead of a dashboard.
        </p>
      </div>

      <div className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-1">
        {actions.map((action) => {
          const Icon = actionIcons[action.id];

          return (
            <motion.button
              key={action.id}
              type="button"
              onClick={() => onActionSelect(action.id)}
              whileHover={prefersReducedMotion ? undefined : { y: -1 }}
              whileTap={prefersReducedMotion ? undefined : { scale: 0.985 }}
              transition={{ type: "spring", stiffness: 260, damping: 20 }}
              className={`rounded-[1.5rem] border p-4 text-left transition-colors ${
                action.isActive
                  ? "border-teal-300/30 bg-teal-400/10"
                  : "border-white/10 bg-white/[0.03]"
              }`}
            >
              <div className="flex items-start justify-between gap-3">
                <div
                  className={`flex h-10 w-10 items-center justify-center rounded-2xl ring-1 ${
                    action.isActive
                      ? "bg-teal-400/10 text-teal-100 ring-teal-300/20"
                      : "bg-white/[0.06] text-slate-100 ring-white/10"
                  }`}
                >
                  <Icon className="h-5 w-5" weight="duotone" />
                </div>
                <span className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
                  {action.id}
                </span>
              </div>

              <div className="mt-4 space-y-2">
                <h3 className="text-base font-medium text-slate-100">
                  {action.label}
                </h3>
                <p className="text-sm leading-6 text-slate-400">
                  {action.helper}
                </p>
              </div>
            </motion.button>
          );
        })}
      </div>
    </motion.section>
  );
}
