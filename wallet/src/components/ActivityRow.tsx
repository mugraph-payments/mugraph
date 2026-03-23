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
      <div className="grid gap-4 lg:grid-cols-[auto_minmax(0,1fr)_auto] lg:items-start">
        <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-white/[0.05] text-slate-100 ring-1 ring-white/10">
          <KindIcon className="h-5 w-5" weight="duotone" />
        </div>

        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <span className="wallet-kicker text-slate-500">{activity.kindLabel}</span>
            <span className="text-base text-slate-400">{activity.createdAtRelative}</span>
          </div>

          <div className="mt-3 grid gap-3 sm:grid-cols-3">
            <div>
              <p className="wallet-kicker text-slate-500">Amount</p>
              <p className="wallet-data mt-1 text-xl font-semibold text-slate-50">
                {activity.amountLabel}
              </p>
            </div>
            <div className="sm:col-span-2">
              <p className="wallet-kicker text-slate-500">Summary</p>
              <p className="wallet-copy mt-1 text-base leading-7 text-slate-400">
                {activity.summary}
              </p>
            </div>
          </div>
        </div>

        <div className="flex flex-col items-start gap-2 lg:items-end">
          <ActivityStatusBadge
            label={activity.statusLabel}
            tone={activity.statusTone}
          />
          <span className="wallet-code text-base text-slate-500">{activity.referenceShort}</span>
        </div>
      </div>
    </article>
  );
}
