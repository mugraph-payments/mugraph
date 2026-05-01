import { Copy } from "@phosphor-icons/react";
import { useState } from "react";
import * as api from "../lib/api";

interface ReceiveAssetOption {
  id: string;
  label: string;
  balanceLabel: string;
}

interface ReceiveDetailsProps {
  network: string;
  label: string;
  delegatePkShort: string;
  scriptAddressShort: string;
  networkLabel: string;
  assetOptions: ReceiveAssetOption[];
  /** Refresh wallet state after a successful import. */
  onDone: () => Promise<void> | void;
}

type ReceiveTab = "request" | "import";

function parseAssetId(assetId: string): { policy_id: string; asset_name: string } {
  // Asset IDs are stored as `policyId:assetName` strings.
  const colon = assetId.indexOf(":");
  if (colon < 0) return { policy_id: assetId, asset_name: "" };
  return {
    policy_id: assetId.slice(0, colon),
    asset_name: assetId.slice(colon + 1),
  };
}

export function ReceiveDetails({
  network,
  label,
  delegatePkShort,
  scriptAddressShort,
  networkLabel,
  assetOptions,
  onDone,
}: ReceiveDetailsProps) {
  const [tab, setTab] = useState<ReceiveTab>("request");

  // Request state
  const [assetId, setAssetId] = useState<string>(assetOptions[0]?.id ?? "");
  const [requestAmount, setRequestAmount] = useState<string>("");
  const [requestLabel, setRequestLabel] = useState<string>("");
  const [requestEnvelope, setRequestEnvelope] = useState<string | null>(null);
  const [requestError, setRequestError] = useState<string | null>(null);
  const [requestSubmitting, setRequestSubmitting] = useState(false);
  const [requestCopied, setRequestCopied] = useState(false);

  // Import state
  const [importPayload, setImportPayload] = useState<string>("");
  const [importResult, setImportResult] = useState<api.ImportResult | null>(null);
  const [importError, setImportError] = useState<string | null>(null);
  const [importSubmitting, setImportSubmitting] = useState(false);

  const requestAmountNum = (() => {
    const trimmed = requestAmount.trim();
    if (!trimmed) return null;
    const num = Number(trimmed);
    if (!Number.isFinite(num) || num <= 0 || !Number.isInteger(num)) return null;
    return num;
  })();
  const requestReady =
    !!assetId && requestLabel.trim() !== "" && requestAmountNum !== null && !requestSubmitting;

  async function handleRequestSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!requestReady || requestAmountNum === null) return;
    setRequestSubmitting(true);
    setRequestError(null);
    setRequestEnvelope(null);
    try {
      const { policy_id, asset_name } = parseAssetId(assetId);
      const env = await api.createReceiveRequest({
        network,
        policy_id,
        asset_name,
        amount: requestAmountNum,
        label: requestLabel.trim() || undefined,
      });
      setRequestEnvelope(env);
    } catch (e) {
      setRequestError(e instanceof Error ? e.message : String(e));
    } finally {
      setRequestSubmitting(false);
    }
  }

  async function handleRequestCopy() {
    if (!requestEnvelope) return;
    try {
      await navigator.clipboard.writeText(requestEnvelope);
      setRequestCopied(true);
      setTimeout(() => setRequestCopied(false), 1500);
    } catch {
      // best-effort
    }
  }

  async function handleImportSubmit(event: React.FormEvent) {
    event.preventDefault();
    const trimmed = importPayload.trim();
    if (!trimmed || importSubmitting) return;
    setImportSubmitting(true);
    setImportError(null);
    setImportResult(null);
    try {
      const res = await api.importNotes(trimmed);
      setImportResult(res);
      setImportPayload("");
      await onDone();
    } catch (e) {
      setImportError(e instanceof Error ? e.message : String(e));
    } finally {
      setImportSubmitting(false);
    }
  }

  return (
    <div className="grid w-full max-w-2xl gap-5">
      <div className="flex gap-1 rounded-xl bg-white/[0.04] p-1">
        <button
          type="button"
          onClick={() => setTab("request")}
          className={`wallet-interactive flex-1 rounded-lg px-4 py-2 text-sm font-medium ${
            tab === "request"
              ? "bg-white/[0.08] text-slate-50"
              : "text-slate-400 hover:text-slate-200"
          }`}
        >
          Request notes
        </button>
        <button
          type="button"
          onClick={() => setTab("import")}
          className={`wallet-interactive flex-1 rounded-lg px-4 py-2 text-sm font-medium ${
            tab === "import"
              ? "bg-white/[0.08] text-slate-50"
              : "text-slate-400 hover:text-slate-200"
          }`}
        >
          Import notes
        </button>
      </div>

      {tab === "request" ? (
        <form className="grid gap-4" onSubmit={handleRequestSubmit}>
          <p className="wallet-meta-note text-slate-500">
            Sharing as <span className="text-slate-300">{label}</span> on {networkLabel}.
          </p>

          {assetOptions.length === 0 ? (
            <div className="wallet-subtle-card p-4 text-sm text-slate-300">
              No assets known yet. Deposit funds first to populate the asset list.
            </div>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2">
              <label className="grid gap-1.5 text-sm text-slate-200">
                <span className="wallet-kicker text-slate-500">Asset</span>
                <select
                  className="wallet-input"
                  value={assetId}
                  onChange={(e) => setAssetId(e.target.value)}
                >
                  {assetOptions.map((a) => (
                    <option key={a.id} value={a.id}>
                      {a.label}
                    </option>
                  ))}
                </select>
              </label>

              <label className="grid gap-1.5 text-sm text-slate-200">
                <span className="wallet-kicker text-slate-500">Amount</span>
                <input
                  type="text"
                  inputMode="numeric"
                  className="wallet-input wallet-data"
                  value={requestAmount}
                  onChange={(e) => setRequestAmount(e.target.value)}
                  placeholder="e.g. 1200"
                  aria-invalid={
                    requestAmount.trim() && requestAmountNum === null ? true : undefined
                  }
                />
              </label>

              <label className="grid gap-1.5 text-sm text-slate-200 sm:col-span-2">
                <span className="wallet-kicker text-slate-500">Label</span>
                <input
                  type="text"
                  className="wallet-input"
                  value={requestLabel}
                  onChange={(e) => setRequestLabel(e.target.value)}
                  placeholder="Invoice or note"
                />
              </label>
            </div>
          )}

          {requestError ? (
            <p className="wallet-hint text-rose-300" role="alert">
              {requestError}
            </p>
          ) : null}

          {requestEnvelope ? (
            <div className="wallet-subtle-card grid gap-3 p-4 sm:p-5">
              <p className="wallet-kicker text-slate-500">Receive request envelope</p>
              <textarea
                readOnly
                value={requestEnvelope}
                className="wallet-input wallet-code h-40 resize-none break-all text-xs text-slate-200"
              />
              <button
                type="button"
                onClick={handleRequestCopy}
                className="wallet-interactive wallet-cta-secondary inline-flex w-fit items-center gap-2 rounded-lg border px-3 py-1.5 text-sm font-medium text-slate-200"
              >
                <Copy className="h-3.5 w-3.5" weight="bold" />
                {requestCopied ? "Copied" : "Copy envelope"}
              </button>
            </div>
          ) : null}

          <button
            type="submit"
            disabled={!requestReady}
            className="wallet-interactive wallet-cta-primary w-full rounded-2xl border px-4 py-3 text-base font-medium text-slate-50 disabled:opacity-45 disabled:active:scale-100"
          >
            {requestSubmitting ? "Generating…" : "Generate request"}
          </button>

          <p className="wallet-meta-note text-slate-500">
            Delegate {delegatePkShort} · script {scriptAddressShort}
          </p>
        </form>
      ) : (
        <form className="grid gap-4" onSubmit={handleImportSubmit}>
          <p className="wallet-meta-note text-slate-500">
            Paste the JSON envelope you received from another wallet. Imported notes are
            auto-refreshed against the node, and any that fail validation land in the quarantined
            state.
          </p>

          <textarea
            value={importPayload}
            onChange={(e) => setImportPayload(e.target.value)}
            placeholder='{"network":"preprod","notes":[…]}'
            className="wallet-input wallet-code h-40 resize-none break-all text-xs text-slate-200"
          />

          {importError ? (
            <p className="wallet-hint text-rose-300" role="alert">
              {importError}
            </p>
          ) : null}

          {importResult ? (
            <p className="wallet-hint text-teal-300" role="status">
              Imported {importResult.imported} note{importResult.imported === 1 ? "" : "s"}
              {importResult.quarantined > 0 ? `, ${importResult.quarantined} quarantined` : ""}.
            </p>
          ) : null}

          <button
            type="submit"
            disabled={!importPayload.trim() || importSubmitting}
            className="wallet-interactive wallet-cta-primary w-full rounded-2xl border px-4 py-3 text-base font-medium text-slate-50 disabled:opacity-45 disabled:active:scale-100"
          >
            {importSubmitting ? "Importing…" : "Import notes"}
          </button>
        </form>
      )}
    </div>
  );
}
