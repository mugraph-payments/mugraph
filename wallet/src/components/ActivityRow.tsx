import {
  ArrowCircleDown,
  ArrowCircleUp,
  ArrowsClockwise,
} from "@phosphor-icons/react";
import type { ComponentType } from "react";
import type { WalletActivityView } from "../lib/walletView";
import { ActivityStatusBadge } from "./ActivityStatusBadge";

interface ActivityRowProps {
  activity: WalletActivityView;
}

const kindIcons: Record<
  WalletActivityView["kindLabel"],
  ComponentType<{ className?: string; weight?: "regular" | "duotone" }>
> = {
  Deposit: ArrowCircleDown,
  Refresh: ArrowsClockwise,
  Withdraw: ArrowCircleUp,
};

export function ActivityRow({ activity }: ActivityRowProps) {
  const KindIcon = kindIcons[activity.kindLabel] ?? ArrowsClockwise;

  return (
    <article className="wallet-card p-4">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-teal-400/10 text-teal-100 ring-1 ring-teal-300/20">
              <KindIcon className="h-5 w-5" weight="duotone" />
            </div>
            <span className="text-sm font-medium text-slate-100">{activity.kindLabel}</span>
            <span className="wallet-kicker text-slate-500">{activity.createdAtRelative}</span>
          </div>

          <div className="space-y-1">
            <p className="wallet-kicker text-slate-500">Amount</p>
            <p className="wallet-data break-words text-xl font-semibold tracking-tight text-slate-50 sm:text-2xl">
              {activity.amountLabel}
            </p>
          </div>

          <p className="wallet-copy text-sm leading-6 text-slate-400">{activity.summary}</p>
        </div>

        <div className="flex flex-wrap items-center gap-2 lg:justify-end">
          <ActivityStatusBadge
            label={activity.statusLabel}
            tone={activity.statusTone}
          />
          <span className="wallet-code text-[11px] text-slate-500">{activity.referenceShort}</span>
        </div>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <div className="wallet-subtle-card p-3">
          <p className="wallet-kicker text-slate-500">Kind</p>
          <p className="mt-2 text-sm text-slate-100">{activity.kindLabel}</p>
        </div>
        <div className="wallet-subtle-card p-3">
          <p className="wallet-kicker text-slate-500">Status</p>
          <p className="mt-2 text-sm text-slate-100">{activity.statusLabel}</p>
        </div>
        <div className="wallet-subtle-card p-3">
          <p className="wallet-kicker text-slate-500">Amount</p>
          <p className="wallet-data mt-2 text-sm text-slate-100">{activity.amountLabel}</p>
        </div>
        <div className="wallet-subtle-card min-w-0 p-3">
          <p className="wallet-kicker text-slate-500">Reference</p>
          <p className="wallet-code mt-2 truncate text-sm text-slate-100" title={activity.referenceShort}>
            {activity.referenceShort}
          </p>
        </div>
      </div>
    </article>
  );
}
