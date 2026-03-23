import { walletState } from "./data/stubWallet";
import { AppShell } from "./components/AppShell";
import { WalletHeader } from "./components/WalletHeader";
import { createWalletView } from "./lib/walletView";

const panelClassName =
  "rounded-[2rem] border border-white/10 bg-slate-950/60 p-5 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur";

function ShellMetric({
  label,
  value,
}: {
  label: string;
  value: string;
}) {
  return (
    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
      <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
        {label}
      </p>
      <p className="mt-2 text-xl font-semibold tracking-tight text-slate-100">
        {value}
      </p>
    </div>
  );
}

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
          <section className={panelClassName}>
            <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
              <div>
                <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
                  Primary workspace
                </p>
                <h2 className="mt-2 text-2xl font-semibold tracking-tight text-slate-50">
                  Responsive shell regions are in place
                </h2>
                <p className="mt-2 max-w-2xl text-sm leading-6 text-slate-400">
                  The next steps can drop summary surfaces, action clusters, and
                  inventory panels into this structure without revisiting the app
                  shell.
                </p>
              </div>
            </div>

            <div className="mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
              <ShellMetric label="Assets" value={`${view.assets.length}`} />
              <ShellMetric label="Notes" value={`${walletState.notes.length}`} />
              <ShellMetric
                label="Activity"
                value={`${walletState.activity.length}`}
              />
              <ShellMetric
                label="Primary actions"
                value={`${view.actions.length}`}
              />
            </div>
          </section>

          <section className={`${panelClassName} grid gap-4 xl:grid-cols-[1.1fr_0.9fr]`}>
            <ShellRegion
              title="Portfolio surfaces"
              copy="Hero metrics, holdings, and note inventory will stack here in a desktop-first but mobile-safe layout."
            />
            <ShellRegion
              title="Timeline lane"
              copy="Recent activity and status context will live here with room for compact cards and denser desktop views."
            />
          </section>
        </>
      }
      secondary={
        <>
          <section className={panelClassName}>
            <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
              Action workspace
            </p>
            <h2 className="mt-2 text-xl font-semibold tracking-tight text-slate-50">
              The action-first rail anchors the shell
            </h2>
            <div className="mt-4 grid gap-3">
              {view.actions.map((action) => (
                <div
                  key={action.id}
                  className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4"
                >
                  <div className="flex items-center justify-between gap-3">
                    <h3 className="text-sm font-medium text-slate-100">
                      {action.label}
                    </h3>
                    <span className="text-[11px] uppercase tracking-[0.22em] text-slate-500">
                      {action.id}
                    </span>
                  </div>
                  <p className="mt-2 text-sm leading-6 text-slate-400">
                    {action.helper}
                  </p>
                </div>
              ))}
            </div>
          </section>

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
        </>
      }
    />
  );
}

export default App;
