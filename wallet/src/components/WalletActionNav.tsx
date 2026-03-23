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
   className="wallet-panel p-5"
  >
   <div className="space-y-2">
    <p className="wallet-kicker text-slate-500">Actions</p>
    <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
     Composer
    </h2>
    <p className="wallet-copy text-base leading-7 text-slate-400">
     Pick the intent first, then keep the draft directly underneath it.
    </p>
   </div>

   <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-1">
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
       className={`wallet-interactive flex items-start gap-3 rounded-2xl border p-3.5 text-left ${
        action.isActive
         ? "wallet-accent-ring border-teal-300/25 bg-teal-400/[0.08]"
         : "border-white/10 bg-white/[0.025]"
       }`}
      >
       <div
        className={`flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl ring-1 ${
         action.isActive
          ? "bg-teal-400/12 text-teal-100 ring-teal-300/20"
          : "bg-white/[0.05] text-slate-100 ring-white/10"
        }`}
       >
        <Icon className="h-5 w-5" weight="duotone" />
       </div>

       <div className="min-w-0">
        <div className="flex items-center gap-2">
         <h3 className="wallet-heading text-base font-semibold text-slate-50">
          {action.label}
         </h3>
         <span className="wallet-kicker text-slate-500">{action.id}</span>
        </div>
        <p className="wallet-copy mt-1 text-base leading-7 text-slate-400">
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
