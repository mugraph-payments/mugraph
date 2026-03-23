import { CaretRight } from "@phosphor-icons/react";

export function WalletSettingsScreen() {
  return (
    <section className="wallet-panel p-5 sm:p-6">
      <div className="space-y-2">
        <p className="wallet-kicker text-slate-500">Settings</p>
        <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
          Wallet settings
        </h2>
        <p className="wallet-copy max-w-2xl text-base leading-7 text-slate-400">
          Manage wallet preferences, advanced tools, and technical details from one place.
        </p>
      </div>

      <section className="wallet-panel-soft mt-5 p-4">
        <div className="flex items-center justify-between gap-3">
          <div>
            <p className="wallet-kicker text-slate-500">Advanced</p>
            <p className="mt-2 text-base text-slate-300">
              Technical tools and private wallet internals live here.
            </p>
          </div>
          <CaretRight className="h-5 w-5 text-slate-500" weight="bold" />
        </div>
      </section>
    </section>
  );
}
