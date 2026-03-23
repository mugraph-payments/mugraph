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
    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <div className="flex h-9 w-9 items-center justify-center rounded-2xl bg-teal-400/10 text-teal-100 ring-1 ring-teal-300/20">
              <KindIcon className="h-5 w-5" weight="duotone" />
            </div>
            <span className="text-sm font-medium text-slate-100">
              {activity.kindLabel}
            </span>
            <span className="text-xs uppercase tracking-[0.22em] text-slate-500">
              {activity.createdAtRelative}
            </span>
          </div>
          <p className="text-lg font-semibold tracking-tight text-slate-50">
            {activity.amountLabel}
          </p>
          <p className="text-sm leading-6 text-slate-400">{activity.summary}</p>
        </div>

        <div className="flex flex-wrap items-center gap-2 lg:justify-end">
          <ActivityStatusBadge
            label={activity.statusLabel}
            tone={activity.statusTone}
          />
          <span className="text-xs uppercase tracking-[0.22em] text-slate-500">
            {activity.referenceShort}
          </span>
        </div>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Kind
          </p>
          <p className="mt-1 text-sm text-slate-200">{activity.kindLabel}</p>
        </div>
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Status
          </p>
          <p className="mt-1 text-sm text-slate-200">{activity.statusLabel}</p>
        </div>
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Amount
          </p>
          <p className="mt-1 text-sm text-slate-200">{activity.amountLabel}</p>
        </div>
        <div className="min-w-0">
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Reference
          </p>
          <p className="mt-1 truncate text-sm text-slate-200">
            {activity.referenceShort}
          </p>
        </div>
      </div>
    </div>
  );
}
