import { ArrowCircleDown, ArrowCircleUp, ArrowsClockwise } from "@phosphor-icons/react";
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
    <article className="wallet-panel h-full p-4">
      <div className="flex items-start gap-3">
        <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-white/[0.05] text-slate-100 ring-1 ring-white/10">
          <KindIcon className="h-5 w-5" weight="duotone" />
        </div>

        <div className="min-w-0 flex-1">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0">
              <p className="wallet-kicker text-slate-500">{activity.kindLabel}</p>
              <p className="wallet-data mt-1 text-lg font-semibold text-slate-50">
                {activity.amountLabel}
              </p>
            </div>
            <ActivityStatusBadge label={activity.statusLabel} tone={activity.statusTone} />
          </div>

          <p className="wallet-copy mt-3 text-base leading-7 text-slate-400">{activity.summary}</p>

          <div className="mt-3 flex items-center justify-between gap-3 text-sm text-slate-400">
            <span>{activity.createdAtRelative}</span>
            <span className="wallet-code text-sm text-slate-500">{activity.referenceShort}</span>
          </div>
        </div>
      </div>
    </article>
  );
}
