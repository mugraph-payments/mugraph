import { useState } from "react";
import * as api from "../lib/api";

interface WithdrawAssetOption {
  id: string;
  label: string;
  balanceLabel: string;
}

interface WithdrawDetailsProps {
  network: string;
  scriptAddressShort: string;
  topAssetLabel: string;
  latestWithdrawReference: string;
  pendingActivityCount: number;
  assetOptions: WithdrawAssetOption[];
  /** Refresh wallet state after a successful withdrawal. */
  onDone: () => Promise<void> | void;
}

function parsePositiveInteger(input: string): number | null {
  const trimmed = input.trim();
  if (!trimmed) return null;
  const num = Number(trimmed);
  if (!Number.isFinite(num) || num <= 0 || !Number.isInteger(num)) return null;
  return num;
}

function parseAssetId(assetId: string): { policy_id?: string; asset_name?: string } {
  const colon = assetId.indexOf(":");
  if (colon < 0) return { policy_id: assetId };
  return {
    policy_id: assetId.slice(0, colon),
    asset_name: assetId.slice(colon + 1),
  };
}

export function WithdrawDetails({
  network,
  scriptAddressShort,
  topAssetLabel,
  latestWithdrawReference,
  pendingActivityCount,
  assetOptions,
  onDone,
}: WithdrawDetailsProps) {
  const [assetId, setAssetId] = useState<string>(assetOptions[0]?.id ?? "");
  const [amountInput, setAmountInput] = useState<string>("");
  const [destinationAddress, setDestinationAddress] = useState<string>("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<api.WithdrawResult | null>(null);

  const amount = parsePositiveInteger(amountInput);
  const isReady = !!assetId && amount !== null && destinationAddress.trim() !== "" && !submitting;

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!isReady || amount === null) return;
    setSubmitting(true);
    setError(null);
    setResult(null);
    try {
      const { policy_id, asset_name } = parseAssetId(assetId);
      const res = await api.withdraw({
        network,
        destination_address: destinationAddress.trim(),
        amount,
        policy_id,
        asset_name,
      });
      setResult(res);
      setAmountInput("");
      setDestinationAddress("");
      await onDone();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <form className="grid gap-5" onSubmit={handleSubmit}>
      <p className="wallet-meta-note text-slate-500">
        Withdrawals burn notes, build a Cardano transaction spending the script UTxO that backs
        them, and submit it through the configured provider. Any leftover value comes back as change
        notes.
      </p>

      <div className="grid gap-3 sm:grid-cols-2">
        <label className="grid gap-2 text-base text-slate-200">
          <span className="wallet-kicker text-slate-500">Asset</span>
          <select
            value={assetId}
            onChange={(event) => setAssetId(event.target.value)}
            className="wallet-input"
          >
            <option value="">Select an asset</option>
            {assetOptions.map((asset) => (
              <option key={asset.id} value={asset.id}>
                {asset.label}
              </option>
            ))}
          </select>
        </label>

        <label className="grid gap-2 text-base text-slate-200">
          <span className="wallet-kicker text-slate-500">Amount (lovelace)</span>
          <input
            type="text"
            inputMode="numeric"
            value={amountInput}
            onChange={(event) => setAmountInput(event.target.value)}
            placeholder="e.g. 25000000"
            aria-invalid={amountInput.trim() && amount === null ? true : undefined}
            className="wallet-input wallet-data"
          />
          {amountInput.trim() && amount === null ? (
            <p className="wallet-hint text-rose-300">Enter a positive integer in lovelace.</p>
          ) : null}
        </label>

        <label className="grid gap-2 text-base text-slate-200 sm:col-span-2">
          <span className="wallet-kicker text-slate-500">Destination address</span>
          <input
            type="text"
            value={destinationAddress}
            onChange={(event) => setDestinationAddress(event.target.value)}
            placeholder="addr_test1..."
            className="wallet-input wallet-code"
          />
        </label>
      </div>

      {error ? (
        <p className="wallet-hint text-rose-300" role="alert">
          {error}
        </p>
      ) : null}

      {result ? (
        <div className="wallet-subtle-card grid gap-2 p-4 sm:p-5" role="status">
          <p className="wallet-kicker text-teal-300">Withdrawal submitted</p>
          <p className="wallet-code break-all text-xs text-slate-200">tx {result.tx_hash}</p>
          <p className="text-xs text-slate-400">
            {result.change_notes} change note{result.change_notes === 1 ? "" : "s"} created locally.
          </p>
        </div>
      ) : null}

      <button
        type="submit"
        disabled={!isReady}
        className="wallet-interactive wallet-cta-primary w-full rounded-2xl border px-4 py-3 text-base font-medium text-slate-50 disabled:opacity-45 disabled:active:scale-100"
      >
        {submitting ? "Submitting withdrawal…" : "Submit withdrawal"}
      </button>

      <p className="wallet-meta-note text-slate-500">
        From {scriptAddressShort} · primary asset {topAssetLabel} · {pendingActivityCount} pending ·
        ref {latestWithdrawReference}
      </p>
    </form>
  );
}
