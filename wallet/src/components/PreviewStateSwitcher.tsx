import { motion, useReducedMotion } from "framer-motion";
import type {
  WalletPreviewState,
  WalletPreviewStateId,
} from "../data/walletPreviewStates";

interface PreviewStateSwitcherProps {
  activePreviewId: WalletPreviewStateId;
  previews: WalletPreviewState[];
  onPreviewSelect: (id: WalletPreviewStateId) => void;
}

const previewToneClasses: Record<
  WalletPreviewStateId,
  { shell: string; button: string; label: string }
> = {
  ready: {
    shell: "border-white/10 bg-slate-950/60",
    button: "border-teal-300/30 bg-teal-400/10 text-teal-50",
    label: "text-teal-300/75",
  },
  empty: {
    shell: "border-white/10 bg-slate-950/60",
    button: "border-slate-300/20 bg-white/[0.06] text-slate-100",
    label: "text-slate-300/75",
  },
  syncing: {
    shell: "border-amber-400/20 bg-[linear-gradient(180deg,rgba(245,158,11,0.08),rgba(2,6,23,0.72))]",
    button: "border-amber-300/30 bg-amber-400/10 text-amber-50",
    label: "text-amber-300/75",
  },
  attention: {
    shell: "border-rose-400/20 bg-[linear-gradient(180deg,rgba(244,63,94,0.08),rgba(2,6,23,0.72))]",
    button: "border-rose-300/30 bg-rose-400/10 text-rose-50",
    label: "text-rose-300/75",
  },
};

export function PreviewStateSwitcher({
  activePreviewId,
  previews,
  onPreviewSelect,
}: PreviewStateSwitcherProps) {
  const activePreview =
    previews.find((preview) => preview.id === activePreviewId) ?? previews[0];
  const activeTone = previewToneClasses[activePreviewId];
  const prefersReducedMotion = useReducedMotion();

  return (
    <motion.section
      initial={prefersReducedMotion ? false : { opacity: 0.98, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.24, ease: [0.16, 1, 0.3, 1] }}
      className={`rounded-[1.5rem] border p-4 shadow-[0_24px_80px_-40px_rgba(15,23,42,0.95)] backdrop-blur ${activeTone.shell}`}
    >
      <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className={`text-xs uppercase tracking-[0.22em] ${activeTone.label}`}>
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
          const tone = previewToneClasses[preview.id];

          return (
            <motion.button
              key={preview.id}
              type="button"
              onClick={() => onPreviewSelect(preview.id)}
              whileHover={prefersReducedMotion ? undefined : { y: -1 }}
              whileTap={prefersReducedMotion ? undefined : { scale: 0.985 }}
              transition={{ type: "spring", stiffness: 260, damping: 20 }}
              className={`rounded-[1.25rem] border px-3 py-3 text-left will-change-transform ${
                isActive
                  ? tone.button
                  : "border-white/10 bg-white/[0.03] text-slate-200"
              }`}
            >
              <p className="text-xs uppercase tracking-[0.22em] text-slate-500">
                {preview.id}
              </p>
              <p className="mt-2 text-sm font-medium">{preview.label}</p>
            </motion.button>
          );
        })}
      </div>
    </motion.section>
  );
}
