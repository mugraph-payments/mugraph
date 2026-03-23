import { ActionGrid } from "./components/ActionGrid";
import { AppShell } from "./components/AppShell";
import { AssetPanel } from "./components/AssetPanel";
import { HeroSummary } from "./components/HeroSummary";
import { WalletHeader } from "./components/WalletHeader";
import { walletState } from "./data/stubWallet";
import { createWalletView } from "./lib/walletView";

const panelClassName =
  "rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur";

function ShellRegion({
  title,
  copy,
}: {
  title: string;
  copy: string;
}) {
  return (
    <div className="rounded-[1.5rem] border border-dashed border-white/10 bg-white/[0.02] p-4">
      <h3 className="text-sm font-medium text-slate-100">{title}</h3>
      <p className="mt-2 text-sm leading-6 text-slate-400">{copy}</p>
    </div>
  );
}

function App() {
  const view = createWalletView(walletState);

  return (
    <AppShell
      header={
        <WalletHeader
          label={view.identity.label}
          networkLabel={view.identity.networkLabel}
          statusLabel={view.identity.statusLabel}
          lastSyncedRelative={view.identity.lastSyncedRelative}
        />
      }
      primary={
        <>
          <HeroSummary
            identity={view.identity}
            summaryMetrics={view.summaryMetrics}
          />

          <ActionGrid actions={view.actions} />

          <AssetPanel assets={view.assets} />

          <section className={`${panelClassName} grid gap-4 xl:grid-cols-[1.1fr_0.9fr]`}>
            <ShellRegion
              title="Note inventory"
              copy="Private notes will stack here with room for status badges, amounts, and compact cryptographic references."
            />
            <ShellRegion
              title="Timeline lane"
              copy="Recent activity and status context will live here with room for compact cards and denser desktop views."
            />
          </section>
        </>
      }
      secondary={
        <section className={panelClassName}>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Detail region
          </p>
          <h2 className="mt-2 text-xl font-semibold tracking-tight text-slate-50">
            Selected action details land here next
          </h2>
          <p className="mt-3 text-sm leading-6 text-slate-400">
            This region stays visible on compact screens and becomes a distinct
            secondary column at larger breakpoints.
          </p>
        </section>
      }
    />
  );
}

export default App;
