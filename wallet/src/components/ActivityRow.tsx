import { ArrowCircleDown, ArrowCircleUp, ArrowsClockwise } from "@phosphor-icons/react";
import type { ComponentType } from "react";
import type { WalletActivityView, WalletTone } from "../lib/walletView";

interface ActivityRowProps {
  activity: WalletActivityView;
  onConfirm?: (id: string) => void;
  onRevert?: (id: string) => void;
}

const kindIcons: Record<
  WalletActivityView["kindLabel"],
  ComponentType<{ className?: string; weight?: "regular" | "duotone" }>
> = {
  Deposit: ArrowCircleDown,
  Refresh: ArrowsClockwise,
  Withdraw: ArrowCircleUp,
};

const kindIconStyle: Record<string, string> = {
  Deposit: "bg-teal-400/10 text-teal-300",
  Withdraw: "bg-rose-400/10 text-rose-300",
  Refresh: "bg-white/[0.05] text-slate-300",
};

const statusTextStyle: Record<WalletTone, string> = {
  neutral: "text-slate-400",
  positive: "text-teal-300",
  warning: "text-amber-300",
  critical: "text-rose-300",
};

export function ActivityRow({ activity, onConfirm, onRevert }: ActivityRowProps) {
  const KindIcon = kindIcons[activity.kindLabel] ?? ArrowsClockwise;
  const iconStyle = kindIconStyle[activity.kindLabel] ?? kindIconStyle.Refresh;
  const isIncoming = activity.kindLabel === "Deposit";
  const isOutgoing = activity.kindLabel === "Withdraw";
  const amountPrefix = isIncoming ? "+" : isOutgoing ? "−" : "";
  const isPending = activity.statusLabel === "Pending";
  const showActions = isPending && (onConfirm || onRevert);

  return (
    <article className="flex items-center gap-3 py-3.5">
      <div
        className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-full ${iconStyle}`}
      >
        <KindIcon className="h-[1.125rem] w-[1.125rem]" weight="duotone" />
      </div>

      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium text-slate-100">{activity.kindLabel}</p>
        <p className="mt-0.5 text-xs text-slate-400">{activity.createdAtRelative}</p>
      </div>

      <div className="flex items-center gap-3">
        {showActions ? (
          <div className="flex gap-1.5">
            {onConfirm ? (
              <button
                type="button"
                onClick={() => onConfirm(activity.id)}
                className="wallet-interactive rounded-lg bg-teal-400/10 px-2.5 py-1 text-xs font-medium text-teal-300 hover:bg-teal-400/20"
              >
                Confirm
              </button>
            ) : null}
            {onRevert ? (
              <button
                type="button"
                onClick={() => onRevert(activity.id)}
                className="wallet-interactive rounded-lg bg-rose-400/10 px-2.5 py-1 text-xs font-medium text-rose-300 hover:bg-rose-400/20"
              >
                Revert
              </button>
            ) : null}
          </div>
        ) : null}

        <div className="text-right">
          <p
            className={`wallet-data text-sm font-semibold ${isIncoming ? "text-teal-300" : "text-slate-100"}`}
          >
            {amountPrefix}
            {activity.amountLabel}
          </p>
          <p className={`mt-0.5 text-xs ${statusTextStyle[activity.statusTone]}`}>
            {activity.statusLabel}
          </p>
        </div>
      </div>
    </article>
  );
}
