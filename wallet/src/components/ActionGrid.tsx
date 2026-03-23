import {
  ArrowCircleDown,
  ArrowCircleUp,
  ArrowSquareIn,
  ArrowSquareOut,
} from "@phosphor-icons/react";
import type { ComponentType } from "react";
import type { WalletActionView } from "../lib/walletView";
import type { WalletActionKind } from "../types/wallet";

interface ActionGridProps {
  actions: WalletActionView[];
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

export function ActionGrid({ actions }: ActionGridProps) {
  return (
    <section className="rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Primary actions
          </p>
          <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-50">
            Move through the wallet from the actions first
          </h2>
        </div>
        <p className="max-w-xl text-sm leading-6 text-slate-400">
          Send, receive, deposit, and withdraw stay near the top so the shell
          behaves like a real wallet instead of a passive dashboard.
        </p>
      </div>

      <div className="mt-5 grid gap-3 sm:grid-cols-2">
        {actions.map((action) => {
          const Icon = actionIcons[action.id];

          return (
            <div
              key={action.id}
              className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4"
            >
              <div className="flex items-start justify-between gap-3">
                <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-teal-400/10 text-teal-100 ring-1 ring-teal-300/20">
                  <Icon className="h-6 w-6" weight="duotone" />
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
            </div>
          );
        })}
      </div>
    </section>
  );
}
