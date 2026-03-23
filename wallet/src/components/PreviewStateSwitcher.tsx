import type { WalletPreviewState, WalletPreviewStateId } from "../data/walletPreviewStates";

interface PreviewStateSwitcherProps {
  activePreviewId: WalletPreviewStateId;
  previews: WalletPreviewState[];
  onPreviewSelect: (id: WalletPreviewStateId) => void;
}

export function PreviewStateSwitcher({
  activePreviewId,
  previews,
  onPreviewSelect,
}: PreviewStateSwitcherProps) {
  const activePreview =
    previews.find((preview) => preview.id === activePreviewId) ?? previews[0];

  return (
    <section className="rounded-[1.5rem] border border-white/10 bg-slate-950/60 p-4 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
            Preview state
          </p>
          <p className="mt-2 text-sm leading-6 text-slate-300">
            Switch the wallet shell among the four planned preview fixtures.
          </p>
        </div>
        <p className="max-w-lg text-sm leading-6 text-slate-400">
          {activePreview.description}
        </p>
      </div>

      <div className="mt-4 grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
        {previews.map((preview) => {
          const isActive = preview.id === activePreviewId;

          return (
            <button
              key={preview.id}
              type="button"
              onClick={() => onPreviewSelect(preview.id)}
              className={`rounded-[1.25rem] border px-3 py-3 text-left transition-colors ${
                isActive
                  ? "border-teal-300/30 bg-teal-400/10 text-teal-50"
                  : "border-white/10 bg-white/[0.03] text-slate-200"
              }`}
            >
              <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
                {preview.id}
              </p>
              <p className="mt-2 text-sm font-medium">{preview.label}</p>
            </button>
          );
        })}
      </div>
    </section>
  );
}
