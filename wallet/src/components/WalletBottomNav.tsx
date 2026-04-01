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
  { id: "home", label: "Home", icon: House },
  { id: "activity", label: "Activity", icon: ClockCounterClockwise },
  { id: "assets", label: "Assets", icon: Wallet },
  { id: "settings", label: "Settings", icon: GearSix },
];

export function WalletBottomNav({ activeDestination, onDestinationSelect }: WalletBottomNavProps) {
  return (
    <nav
      aria-label="Main wallet navigation"
      className="wallet-panel sticky bottom-4 z-10 overflow-hidden p-2 lg:static lg:bottom-auto lg:p-2"
    >
      <div className="grid grid-cols-4 gap-1.5 lg:grid-cols-1">
        {destinationMeta.map((destination) => {
          const isActive = destination.id === activeDestination;
          const Icon = destination.icon;

          return (
            <button
              key={destination.id}
              type="button"
              aria-pressed={isActive}
              onClick={() => onDestinationSelect(destination.id)}
              className={`wallet-interactive flex flex-col items-center gap-1 rounded-lg px-2 py-2.5 text-center lg:flex-row lg:justify-start lg:gap-3 lg:rounded-xl lg:px-3.5 lg:py-2.5 lg:text-left ${
                isActive
                  ? "wallet-accent-ring border border-teal-300/25 bg-teal-400/[0.08] text-slate-50"
                  : "text-slate-400 hover:bg-white/[0.04] hover:text-slate-200"
              }`}
            >
              <Icon className="h-5 w-5" weight={isActive ? "fill" : "duotone"} />
              <span className="text-xs font-medium lg:text-sm">{destination.label}</span>
            </button>
          );
        })}
      </div>
    </nav>
  );
}
