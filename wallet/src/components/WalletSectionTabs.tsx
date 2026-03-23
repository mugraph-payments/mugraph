import type { WalletShellSectionView } from "../lib/walletView";
import type { WalletActiveSection } from "../types/wallet";

interface WalletSectionTabsProps {
  sections: WalletShellSectionView[];
  activeSection: WalletActiveSection;
  onSectionChange: (section: WalletActiveSection) => void;
}

export function WalletSectionTabs({
  sections,
  activeSection,
  onSectionChange,
}: WalletSectionTabsProps) {
  return (
    <div className="rounded-[1.5rem] border border-white/10 bg-slate-950/60 p-2 backdrop-blur">
      <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
        {sections.map((section) => {
          const isActive = section.id === activeSection;

          return (
            <button
              key={section.id}
              type="button"
              aria-pressed={isActive}
              onClick={() => onSectionChange(section.id)}
              className={`min-w-0 rounded-[1rem] border px-3 py-3 text-left transition-colors ${
                isActive
                  ? "border-teal-300/30 bg-teal-400/10 text-teal-50"
                  : "border-white/10 bg-white/[0.03] text-slate-300"
              }`}
            >
              <p className="truncate text-xs uppercase tracking-[0.22em] text-slate-500">
                {section.label}
              </p>
              <p className="mt-1 hidden text-xs leading-5 text-inherit/80 sm:block">
                {section.description}
              </p>
            </button>
          );
        })}
      </div>
    </div>
  );
}
