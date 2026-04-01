import { ClockCounterClockwise, GearSix, House, Wallet } from "@phosphor-icons/react";
import type { ComponentType } from "react";
import type { WalletActiveDestination } from "../types/wallet";

interface WalletBottomNavProps {
  activeDestination: WalletActiveDestination;
  onDestinationSelect: (destination: WalletActiveDestination) => void;
}

const destinationMeta: Array<{
  id: WalletActiveDestination;
  label: string;
  icon: ComponentType<{ className?: string; weight?: "regular" | "duotone" | "fill" }>;
}> = [
  {
    id: "home",
    label: "Home",
    icon: House,
  },
  {
    id: "activity",
    label: "Activity",
    icon: ClockCounterClockwise,
  },
  {
    id: "assets",
    label: "Assets",
    icon: Wallet,
  },
  {
    id: "settings",
    label: "Settings",
    icon: GearSix,
  },
];

export function WalletBottomNav({ activeDestination, onDestinationSelect }: WalletBottomNavProps) {
  return (
    <nav
      aria-label="Main wallet navigation"
      className="wallet-panel sticky bottom-4 z-10 overflow-hidden p-2 lg:static lg:bottom-auto lg:p-3"
    >
      <div className="hidden lg:block px-2 pb-3">
        <p className="wallet-kicker text-slate-500">Navigate</p>
        <p className="mt-2 text-sm leading-6 text-slate-400">
          Switch between your wallet overview, activity, assets, and settings.
        </p>
      </div>

      <div className="grid grid-cols-4 gap-2 lg:grid-cols-1">
        {destinationMeta.map((destination) => {
          const isActive = destination.id === activeDestination;
          const Icon = destination.icon;

          return (
            <button
              key={destination.id}
              type="button"
              aria-pressed={isActive}
              onClick={() => onDestinationSelect(destination.id)}
              className={`wallet-interactive flex flex-col items-center gap-1 rounded-xl px-2 py-3 text-center lg:flex-row lg:justify-start lg:gap-3 lg:px-4 lg:py-3 lg:text-left ${
                isActive
                  ? "wallet-accent-ring border border-teal-300/25 bg-teal-400/[0.08] text-slate-50"
                  : "text-slate-400 hover:bg-white/[0.04] hover:text-slate-200"
              }`}
            >
              <Icon className="h-5 w-5" weight={isActive ? "fill" : "duotone"} />
              <span className="text-sm font-medium lg:text-base">{destination.label}</span>
            </button>
          );
        })}
      </div>
    </nav>
  );
}
