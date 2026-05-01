import { useState } from "react";
import * as api from "../lib/api";

interface GuidedSetupScreenProps {
  /** Called once setup completes successfully so the parent can refresh state. */
  onComplete: () => void;
}

interface FormState {
  label: string;
  mainnetUrl: string;
  preprodUrl: string;
  previewUrl: string;
  providerType: "blockfrost" | "maestro";
  providerApiKey: string;
  providerUrlOverride: string;
}

const initialState: FormState = {
  label: "Mugraph Wallet",
  mainnetUrl: "http://127.0.0.1:9999",
  preprodUrl: "http://127.0.0.1:9999",
  previewUrl: "http://127.0.0.1:9999",
  providerType: "blockfrost",
  providerApiKey: "demo",
  providerUrlOverride: "http://127.0.0.1:8090",
};

export function GuidedSetupScreen({ onComplete }: GuidedSetupScreenProps) {
  const [form, setForm] = useState<FormState>(initialState);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isReady =
    form.label.trim() &&
    form.mainnetUrl.trim() &&
    form.preprodUrl.trim() &&
    form.previewUrl.trim() &&
    form.providerApiKey.trim();

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!isReady) return;
    setSubmitting(true);
    setError(null);
    try {
      await api.completeGuidedSetup({
        label: form.label.trim(),
        mainnet_node_url: form.mainnetUrl.trim(),
        preprod_node_url: form.preprodUrl.trim(),
        preview_node_url: form.previewUrl.trim(),
        provider_type: form.providerType,
        provider_api_key: form.providerApiKey.trim(),
        provider_base_url_override: form.providerUrlOverride.trim() || null,
      });
      onComplete();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSubmitting(false);
    }
  }

  function update<K extends keyof FormState>(key: K, value: FormState[K]) {
    setForm((prev) => ({ ...prev, [key]: value }));
  }

  return (
    <div className="min-h-dvh text-slate-50">
      <div className="wallet-phone-shell mx-auto flex min-h-dvh w-full max-w-3xl flex-col px-4 py-6 sm:px-5">
        <section className="wallet-panel p-5 sm:p-6 lg:p-7">
          <form className="grid gap-5" onSubmit={handleSubmit}>
            <header className="wallet-section-intro">
              <p className="wallet-kicker text-slate-500">Welcome</p>
              <h1 className="wallet-heading text-[2rem] text-slate-50">Set up your wallet</h1>
              <p className="mt-2 text-sm text-slate-400">
                Point the wallet at a Mugraph node for each network and the Cardano provider it
                should share. Same provider credentials are reused across networks.
              </p>
            </header>

            <label className="grid gap-2 text-base text-slate-200">
              <span className="wallet-kicker text-slate-500">Wallet label</span>
              <input
                type="text"
                className="wallet-input"
                value={form.label}
                onChange={(e) => update("label", e.target.value)}
                placeholder="Everyday Wallet"
              />
            </label>

            <fieldset className="grid gap-3">
              <legend className="wallet-kicker text-slate-500">Node URLs</legend>
              <label className="grid gap-1.5 text-sm text-slate-200">
                <span className="text-xs text-slate-400">Mainnet</span>
                <input
                  type="url"
                  className="wallet-input wallet-code"
                  value={form.mainnetUrl}
                  onChange={(e) => update("mainnetUrl", e.target.value)}
                  placeholder="http://..."
                />
              </label>
              <label className="grid gap-1.5 text-sm text-slate-200">
                <span className="text-xs text-slate-400">Preprod</span>
                <input
                  type="url"
                  className="wallet-input wallet-code"
                  value={form.preprodUrl}
                  onChange={(e) => update("preprodUrl", e.target.value)}
                  placeholder="http://..."
                />
              </label>
              <label className="grid gap-1.5 text-sm text-slate-200">
                <span className="text-xs text-slate-400">Preview</span>
                <input
                  type="url"
                  className="wallet-input wallet-code"
                  value={form.previewUrl}
                  onChange={(e) => update("previewUrl", e.target.value)}
                  placeholder="http://..."
                />
              </label>
            </fieldset>

            <fieldset className="grid gap-3">
              <legend className="wallet-kicker text-slate-500">Cardano provider</legend>
              <div className="grid gap-3 sm:grid-cols-2">
                <label className="grid gap-1.5 text-sm text-slate-200">
                  <span className="text-xs text-slate-400">Provider</span>
                  <select
                    className="wallet-input"
                    value={form.providerType}
                    onChange={(e) =>
                      update("providerType", e.target.value as FormState["providerType"])
                    }
                  >
                    <option value="blockfrost">Blockfrost</option>
                    <option value="maestro">Maestro</option>
                  </select>
                </label>
                <label className="grid gap-1.5 text-sm text-slate-200">
                  <span className="text-xs text-slate-400">API key</span>
                  <input
                    type="text"
                    className="wallet-input wallet-code"
                    value={form.providerApiKey}
                    onChange={(e) => update("providerApiKey", e.target.value)}
                    placeholder="project_id"
                  />
                </label>
              </div>
              <label className="grid gap-1.5 text-sm text-slate-200">
                <span className="text-xs text-slate-400">
                  Base URL override (optional — set for the demo's mock chain)
                </span>
                <input
                  type="url"
                  className="wallet-input wallet-code"
                  value={form.providerUrlOverride}
                  onChange={(e) => update("providerUrlOverride", e.target.value)}
                  placeholder="http://127.0.0.1:8090"
                />
              </label>
            </fieldset>

            {error ? (
              <p className="wallet-hint text-rose-300" role="alert">
                {error}
              </p>
            ) : null}

            <button
              type="submit"
              disabled={!isReady || submitting}
              className="wallet-interactive wallet-cta-primary w-full rounded-2xl border px-4 py-3 text-base font-medium text-slate-50 disabled:opacity-45 disabled:active:scale-100"
            >
              {submitting ? "Setting up…" : "Complete setup"}
            </button>
          </form>
        </section>
      </div>
    </div>
  );
}
