import {
  ChartPieSlice,
  Coins,
  ListChecks,
  Pulse,
} from "@phosphor-icons/react";
import type { ComponentType } from "react";
import type { WalletShellSectionView } from "../lib/walletView";
import type { WalletActiveSection } from "../types/wallet";

interface WalletSectionTabsProps {
  sections: WalletShellSectionView[];
  activeSection: WalletActiveSection;
  onSectionChange: (section: WalletActiveSection) => void;
}

const sectionIcons: Record<
  WalletActiveSection,
  ComponentType<{ className?: string; weight?: "regular" | "duotone" }>
> = {
  overview: ChartPieSlice,
  holdings: Coins,
  notes: ListChecks,
  activity: Pulse,
};

export function WalletSectionTabs({
  sections,
  activeSection,
  onSectionChange,
}: WalletSectionTabsProps) {
  return (
    <section className="wallet-panel p-3">
      <div className="px-2 pb-3 pt-1">
        <p className="wallet-kicker text-slate-500">Sections</p>
        <p className="wallet-copy mt-2 text-sm leading-6 text-slate-400">
          Work one layer of the wallet at a time instead of scanning everything at once.
        </p>
      </div>

      <div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-1">
        {sections.map((section) => {
          const isActive = section.id === activeSection;
          const Icon = sectionIcons[section.id];

          return (
            <button
              key={section.id}
              type="button"
              aria-pressed={isActive}
              onClick={() => onSectionChange(section.id)}
              className={`wallet-interactive flex items-start gap-3 rounded-[1.25rem] border p-3 text-left ${
                isActive
                  ? "wallet-accent-ring border-teal-300/25 bg-teal-400/[0.08] text-teal-50"
                  : "border-white/10 bg-white/[0.025] text-slate-300"
              }`}
            >
              <div
                className={`mt-0.5 flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl ring-1 ${
                  isActive
                    ? "bg-teal-400/12 text-teal-100 ring-teal-300/20"
                    : "bg-white/[0.05] text-slate-100 ring-white/10"
                }`}
              >
                <Icon className="h-5 w-5" weight="duotone" />
              </div>
              <div className="min-w-0">
                <p className="wallet-kicker text-slate-500">Workspace</p>
                <p className="wallet-heading mt-1 text-sm font-semibold text-inherit">
                  {section.label}
                </p>
                <p className="wallet-copy mt-1 text-sm leading-6 text-inherit/75">
                  {section.description}
                </p>
              </div>
            </button>
          );
        })}
      </div>
    </section>
  );
}
