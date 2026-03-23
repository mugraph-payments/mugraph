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
      className="wallet-panel p-4 sm:p-5"
    >
      <div className="flex flex-col gap-1">
        <p className="wallet-kicker text-slate-500">Wallet actions</p>
        <h2 className="wallet-heading text-lg font-semibold tracking-tight text-slate-50 sm:text-xl">
          Choose the active flow
        </h2>
        <p className="wallet-copy text-sm leading-6 text-slate-400">
          Send, receive, deposit, and withdraw stay grouped in one action lane.
        </p>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-1">
        {actions.map((action) => {
          const Icon = actionIcons[action.id];

          return (
            <motion.button
              key={action.id}
              type="button"
              aria-pressed={action.isActive}
              onClick={() => onActionSelect(action.id)}
              whileHover={prefersReducedMotion ? undefined : { y: -1 }}
              whileTap={prefersReducedMotion ? undefined : { scale: 0.985 }}
              transition={{ type: "spring", stiffness: 260, damping: 20 }}
              className={`wallet-interactive min-w-0 rounded-[1.5rem] border p-3.5 text-left ${
                action.isActive
                  ? "border-teal-300/30 bg-teal-400/10"
                  : "border-white/10 bg-white/[0.03]"
              }`}
            >
              <div className="flex items-start justify-between gap-3">
                <div
                  className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl ring-1 ${
                    action.isActive
                      ? "bg-teal-400/10 text-teal-100 ring-teal-300/20"
                      : "bg-white/[0.06] text-slate-100 ring-white/10"
                  }`}
                >
                  <Icon className="h-5 w-5" weight="duotone" />
                </div>
                <span className="wallet-kicker text-slate-500">{action.id}</span>
              </div>

              <div className="mt-3 space-y-1.5">
                <h3 className="wallet-heading text-base font-medium text-slate-100">
                  {action.label}
                </h3>
                <p className="wallet-copy text-sm leading-5 text-slate-400">
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
