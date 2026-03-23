import {
 ArrowClockwise,
 ArrowSquareOut,
 Coins,
 Stack,
} from "@phosphor-icons/react";
import type { ReactNode } from "react";
import type {
 WalletActivityView,
 WalletAssetView,
 WalletNoteView,
} from "../lib/walletView";
import { ActivityStatusBadge } from "./ActivityStatusBadge";
import { NoteStatusBadge } from "./NoteStatusBadge";

interface WalletOverviewBoardProps {
 assets: WalletAssetView[];
 notes: WalletNoteView[];
 activity: WalletActivityView[];
}

function PanelCard({
 icon,
 eyebrow,
 title,
 children,
}: {
 icon: ReactNode;
 eyebrow: string;
 title: string;
 children: ReactNode;
}) {
 return (
  <section className="wallet-card p-5">
   <div className="flex items-start gap-3">
    <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl bg-white/[0.06] text-slate-100 ring-1 ring-white/10">
     {icon}
    </div>
    <div className="min-w-0">
     <p className="wallet-kicker text-slate-500">{eyebrow}</p>
     <h3 className="wallet-heading mt-2 text-lg font-semibold text-slate-50">
      {title}
     </h3>
    </div>
   </div>
   <div className="mt-4">{children}</div>
  </section>
 );
}

export function WalletOverviewBoard({
 assets,
 notes,
 activity,
}: WalletOverviewBoardProps) {
 const topAssets = assets.slice(0, 3);
 const latestNotes = notes.slice(0, 3);
 const queueItems = activity.slice(0, 3);

 return (
  <section className="wallet-panel p-5 sm:p-6">
   <div className="flex flex-col gap-2 xl:flex-row xl:items-end xl:justify-between">
    <div>
     <p className="wallet-kicker text-slate-500">Control center</p>
     <h2 className="wallet-heading mt-2 text-3xl font-semibold tracking-tight text-slate-50">
      What matters right now
     </h2>
    </div>
    <p className="wallet-copy max-w-2xl text-base leading-8 text-slate-400">
     The overview keeps the portfolio, spendable note inventory, and live queue in one place so you can see the wallet posture before taking an action.
    </p>
   </div>

   <div className="mt-5 grid gap-4 xl:grid-cols-2 2xl:grid-cols-3">
    <PanelCard
     icon={<Coins className="h-5 w-5" weight="duotone" />}
     eyebrow="Portfolio"
     title="Top positions"
    >
     <div className="grid gap-3">
      {topAssets.map((asset) => (
       <div
        key={asset.id}
        className="wallet-subtle-card flex items-center justify-between gap-3 p-3"
       >
        <div className="min-w-0">
         <div className="flex items-center gap-2">
          <span className="wallet-kicker rounded-full border border-white/10 bg-white/[0.04] px-2.5 py-1 text-slate-200">
           {asset.ticker}
          </span>
          <span className="truncate text-base text-slate-400">{asset.name}</span>
         </div>
         <p className="wallet-data mt-2 text-base font-semibold text-slate-100">
          {asset.balanceLabel}
         </p>
        </div>
        <div className="text-right">
         <p className="wallet-kicker text-slate-500">Share</p>
         <p className="wallet-data mt-1 text-base text-slate-100">{asset.shareLabel}</p>
        </div>
       </div>
      ))}
     </div>
    </PanelCard>

    <PanelCard
     icon={<Stack className="h-5 w-5" weight="duotone" />}
     eyebrow="Inventory"
     title="Latest spendable notes"
    >
     <div className="grid gap-3">
      {latestNotes.map((note) => (
       <div
        key={note.id}
        className="wallet-subtle-card flex flex-col gap-3 p-3"
       >
        <div className="flex items-center justify-between gap-3">
         <div>
          <p className="wallet-kicker text-slate-500">{note.assetTicker}</p>
          <p className="wallet-data mt-1 text-base font-semibold text-slate-100">
           {note.amountLabel}
          </p>
         </div>
         <NoteStatusBadge label={note.statusLabel} tone={note.statusTone} />
        </div>
        <div className="flex items-center justify-between gap-3 text-sm text-slate-400">
         <span>{note.sourceLabel}</span>
         <span>{note.createdAtRelative}</span>
        </div>
       </div>
      ))}
     </div>
    </PanelCard>

    <PanelCard
     icon={<ArrowClockwise className="h-5 w-5" weight="duotone" />}
     eyebrow="Queue"
     title="Live settlement flow"
    >
     <div className="grid gap-3">
      {queueItems.map((item) => (
       <div
        key={item.id}
        className="wallet-subtle-card flex flex-col gap-3 p-3"
       >
        <div className="flex items-center justify-between gap-3">
         <div>
          <p className="wallet-kicker text-slate-500">{item.kindLabel}</p>
          <p className="wallet-data mt-1 text-sm font-semibold text-slate-100">
           {item.amountLabel}
          </p>
         </div>
         <ActivityStatusBadge
          label={item.statusLabel}
          tone={item.statusTone}
         />
        </div>
        <p className="wallet-copy text-base leading-7 text-slate-400">{item.summary}</p>
        <div className="flex items-center justify-between gap-3 text-base text-slate-400">
         <span>{item.createdAtRelative}</span>
         <span className="wallet-code inline-flex items-center gap-1 text-slate-500">
          <ArrowSquareOut className="h-3.5 w-3.5" weight="bold" />
          {item.referenceShort}
         </span>
        </div>
       </div>
      ))}
     </div>
    </PanelCard>
   </div>
  </section>
 );
}
