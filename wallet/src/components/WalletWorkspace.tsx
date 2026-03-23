import type { ReactNode } from "react";
import type { WalletShellSectionView } from "../lib/walletView";
import type { WalletActiveRegion, WalletActiveSection } from "../types/wallet";
import { WalletSectionTabs } from "./WalletSectionTabs";

interface WalletWorkspaceProps {
  isCompactLayout: boolean;
  activeRegion: WalletActiveRegion;
  activeSection: WalletActiveSection;
  sections: WalletShellSectionView[];
  onRegionChange: (region: WalletActiveRegion) => void;
  onSectionChange: (section: WalletActiveSection) => void;
  overview: ReactNode;
  holdings: ReactNode;
  notes: ReactNode;
  activity: ReactNode;
  actionNav: ReactNode;
  actionPanel: ReactNode;
}

interface RegionToggleButtonProps {
  label: string;
  region: WalletActiveRegion;
  activeRegion: WalletActiveRegion;
  onRegionChange: (region: WalletActiveRegion) => void;
}

function RegionToggleButton({
  label,
  region,
  activeRegion,
  onRegionChange,
}: RegionToggleButtonProps) {
  const isActive = activeRegion === region;

  return (
    <button
      type="button"
      aria-pressed={isActive}
      onClick={() => onRegionChange(region)}
      className={`flex-1 rounded-full border px-3 py-2 text-sm transition-colors ${
        isActive
          ? "border-teal-300/30 bg-teal-400/10 text-teal-50"
          : "border-white/10 bg-white/[0.03] text-slate-300"
      }`}
    >
      {label}
    </button>
  );
}

export function WalletWorkspace({
  isCompactLayout,
  activeRegion,
  activeSection,
  sections,
  onRegionChange,
  onSectionChange,
  overview,
  holdings,
  notes,
  activity,
  actionNav,
  actionPanel,
}: WalletWorkspaceProps) {
  const showPrimary = !isCompactLayout || activeRegion === "primary";
  const showSecondary = !isCompactLayout || activeRegion === "secondary";

  const compactPrimarySection = (() => {
    switch (activeSection) {
      case "overview":
        return overview;
      case "holdings":
        return holdings;
      case "notes":
        return notes;
      case "activity":
        return activity;
    }
  })();

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-x-clip">
      {isCompactLayout ? (
        <div className="flex items-center gap-2 rounded-[1.5rem] border border-white/10 bg-slate-950/60 p-2 backdrop-blur">
          <RegionToggleButton
            label="Wallet"
            region="primary"
            activeRegion={activeRegion}
            onRegionChange={onRegionChange}
          />
          <RegionToggleButton
            label="Actions"
            region="secondary"
            activeRegion={activeRegion}
            onRegionChange={onRegionChange}
          />
        </div>
      ) : null}

      <div className="grid min-h-0 flex-1 items-start gap-4 overflow-x-clip lg:grid-cols-[minmax(0,1fr)_minmax(15rem,18rem)] xl:grid-cols-[minmax(0,1.3fr)_minmax(16rem,18rem)]">
        {showPrimary ? (
          <section className="grid min-w-0 content-start gap-4 overflow-x-clip">
            <div className="px-1">
              <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
                Wallet region
              </p>
            </div>

            {isCompactLayout ? (
              <>
                <WalletSectionTabs
                  sections={sections}
                  activeSection={activeSection}
                  onSectionChange={onSectionChange}
                />
                <div className="min-w-0 overflow-x-clip">{compactPrimarySection}</div>
              </>
            ) : (
              <>
                {overview}
                {holdings}
                {notes}
                {activity}
              </>
            )}
          </section>
        ) : null}

        {showSecondary ? (
          <aside className="grid min-w-0 content-start gap-4 overflow-x-clip lg:max-h-[calc(100dvh-9rem)] lg:overflow-y-auto lg:overscroll-contain lg:pr-1">
            <div className="px-1">
              <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
                Action region
              </p>
            </div>
            {actionNav}
            {actionPanel}
          </aside>
        ) : null}
      </div>
    </div>
  );
}
