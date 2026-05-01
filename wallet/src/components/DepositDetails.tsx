import { useCallback, useEffect, useMemo, useState } from "react";
import * as api from "../lib/api";

interface DepositDetailsProps {
  network: string;
  fundingAddress: string | null;
  scriptAddressShort: string;
  delegatePkShort: string;
  latestDepositReference: string;
  pendingActivityCount: number;
  /** Called after a successful deposit so the parent can refresh state. */
  onDone: () => Promise<void> | void;
}

interface FundingUtxoOption {
  id: string;
  txHash: string;
  outputIndex: number;
  lovelace: number;
  shortLabel: string;
}

function formatLovelace(lovelace: number): string {
  return `${(lovelace / 1_000_000).toLocaleString(undefined, { maximumFractionDigits: 6 })} ADA`;
}

function shortRef(txHash: string, index: number): string {
  return `${txHash.slice(0, 10)}…#${index}`;
}

export function DepositDetails({
  network,
  fundingAddress,
  scriptAddressShort,
  delegatePkShort,
  latestDepositReference,
  pendingActivityCount,
  onDone,
}: DepositDetailsProps) {
  const [utxos, setUtxos] = useState<FundingUtxoOption[]>([]);
  const [selectedUtxoId, setSelectedUtxoId] = useState<string | null>(null);
  const [denominations, setDenominations] = useState<string[]>([""]);
  const [loadingUtxos, setLoadingUtxos] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<api.DepositResult | null>(null);
  const [copied, setCopied] = useState(false);

  const loadUtxos = useCallback(async () => {
    setLoadingUtxos(true);
    setError(null);
    try {
      const fresh = await api.listFundingUtxos(network);
      const mapped: FundingUtxoOption[] = fresh.map((u) => ({
        id: `${u.tx_hash}:${u.output_index}`,
        txHash: u.tx_hash,
        outputIndex: u.output_index,
        lovelace: u.lovelace,
        shortLabel: shortRef(u.tx_hash, u.output_index),
      }));
      setUtxos(mapped);
      // Drop the selection if the chosen UTxO no longer exists on chain.
      setSelectedUtxoId((current) =>
        current && mapped.some((u) => u.id === current) ? current : (mapped[0]?.id ?? null),
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoadingUtxos(false);
    }
  }, [network]);

  useEffect(() => {
    void loadUtxos();
  }, [loadUtxos]);

  const selectedUtxo = useMemo(
    () => utxos.find((u) => u.id === selectedUtxoId) ?? null,
    [utxos, selectedUtxoId],
  );

  const parsedDenominations = useMemo(() => {
    return denominations.map((value) => {
      const trimmed = value.trim();
      if (!trimmed) return null;
      const num = Number(trimmed);
      if (!Number.isFinite(num) || num <= 0 || !Number.isInteger(num)) return null;
      return num;
    });
  }, [denominations]);

  const allDenomsValid =
    parsedDenominations.length > 0 && parsedDenominations.every((v): v is number => v !== null);
  const totalDeposit = allDenomsValid ? parsedDenominations.reduce((a, b) => a + (b ?? 0), 0) : 0;
  const fee = 200_000;
  const fits =
    selectedUtxo !== null && allDenomsValid && selectedUtxo.lovelace >= totalDeposit + fee;
  const isReady = fits && !submitting;

  function updateDenom(index: number, value: string) {
    setDenominations((prev) => prev.map((d, i) => (i === index ? value : d)));
  }

  function addDenom() {
    setDenominations((prev) => [...prev, ""]);
  }

  function removeDenom(index: number) {
    setDenominations((prev) => (prev.length === 1 ? prev : prev.filter((_, i) => i !== index)));
  }

  async function handleCopyFunding() {
    if (!fundingAddress) return;
    try {
      await navigator.clipboard.writeText(fundingAddress);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // best-effort; ignore
    }
  }

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!selectedUtxo || !isReady) return;
    setSubmitting(true);
    setError(null);
    setResult(null);
    try {
      const res = await api.deposit({
        network,
        utxo_tx_hash: selectedUtxo.txHash,
        utxo_index: selectedUtxo.outputIndex,
        output_amounts: parsedDenominations.filter((v): v is number => v !== null),
      });
      setResult(res);
      setDenominations([""]);
      await onDone();
      await loadUtxos();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <form className="grid gap-5" onSubmit={handleSubmit}>
      <section className="wallet-panel-soft p-4 sm:p-5">
        <p className="wallet-kicker text-slate-500">Cardano funding address</p>
        {fundingAddress ? (
          <div className="mt-2 grid gap-2">
            <p className="wallet-code break-all text-sm text-slate-100">{fundingAddress}</p>
            <button
              type="button"
              onClick={handleCopyFunding}
              className="wallet-interactive wallet-cta-secondary w-fit rounded-lg border px-3 py-1.5 text-xs font-medium text-slate-200"
            >
              {copied ? "Copied" : "Copy address"}
            </button>
            <p className="wallet-meta-note text-slate-500">
              Send Cardano funds here first, then pick a UTxO below to deposit into Mugraph.
            </p>
          </div>
        ) : (
          <p className="mt-2 text-sm text-rose-300">
            Funding address not derived yet. Run guided setup, then return here.
          </p>
        )}
      </section>

      <section className="grid gap-3">
        <div className="flex items-center justify-between">
          <p className="wallet-kicker text-slate-500">Funding UTxO</p>
          <button
            type="button"
            onClick={loadUtxos}
            disabled={loadingUtxos}
            className="wallet-interactive rounded-md bg-white/[0.06] px-2.5 py-1 text-xs font-medium text-slate-200 hover:bg-white/[0.12] disabled:opacity-40"
          >
            {loadingUtxos ? "Refreshing…" : "Refresh"}
          </button>
        </div>
        {utxos.length === 0 ? (
          <div className="wallet-subtle-card p-4 text-sm text-slate-300">
            {loadingUtxos
              ? "Looking up funding UTxOs…"
              : "No UTxOs at the funding address. Send Cardano funds here, then refresh."}
          </div>
        ) : (
          <div className="grid gap-2">
            {utxos.map((u) => {
              const selected = u.id === selectedUtxoId;
              return (
                <button
                  key={u.id}
                  type="button"
                  aria-pressed={selected}
                  onClick={() => setSelectedUtxoId(u.id)}
                  className={`wallet-choice grid w-full grid-cols-[minmax(0,1fr)_auto] items-center gap-3 ${
                    selected
                      ? "border-teal-300/30 bg-teal-400/10"
                      : "border-white/10 bg-white/[0.03]"
                  }`}
                >
                  <span className="wallet-code break-all text-left text-xs text-slate-300">
                    {u.shortLabel}
                  </span>
                  <span className="wallet-data text-right text-sm font-medium text-slate-100">
                    {formatLovelace(u.lovelace)}
                  </span>
                </button>
              );
            })}
          </div>
        )}
      </section>

      <section className="grid gap-3">
        <p className="wallet-kicker text-slate-500">Note denominations (lovelace)</p>
        <div className="grid gap-2">
          {denominations.map((value, index) => {
            const parsed = parsedDenominations[index];
            const invalid = value.trim() && parsed === null;
            return (
              <div key={index} className="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
                <input
                  type="text"
                  inputMode="numeric"
                  className="wallet-input wallet-data"
                  value={value}
                  placeholder="e.g. 50000000"
                  aria-invalid={invalid ? true : undefined}
                  onChange={(e) => updateDenom(index, e.target.value)}
                />
                <button
                  type="button"
                  onClick={() => removeDenom(index)}
                  disabled={denominations.length === 1}
                  className="wallet-interactive rounded-lg border border-white/10 px-3 py-2 text-xs text-slate-300 disabled:opacity-30"
                >
                  Remove
                </button>
              </div>
            );
          })}
        </div>
        <button
          type="button"
          onClick={addDenom}
          className="wallet-interactive w-fit rounded-md bg-white/[0.06] px-2.5 py-1 text-xs font-medium text-slate-200 hover:bg-white/[0.12]"
        >
          + Add denomination
        </button>
        {selectedUtxo && allDenomsValid ? (
          <p className="wallet-meta-note text-slate-500">
            Total {formatLovelace(totalDeposit)} + {formatLovelace(fee)} fee ={" "}
            {formatLovelace(totalDeposit + fee)} of {formatLovelace(selectedUtxo.lovelace)}
            {fits ? "" : " — exceeds funding UTxO"}
          </p>
        ) : null}
      </section>

      {error ? (
        <p className="wallet-hint text-rose-300" role="alert">
          {error}
        </p>
      ) : null}

      {result ? (
        <p className="wallet-hint text-teal-300" role="status">
          Created {result.notes_created} notes (ref {result.deposit_ref}).
        </p>
      ) : null}

      <button
        type="submit"
        disabled={!isReady}
        className="wallet-interactive wallet-cta-primary w-full rounded-2xl border px-4 py-3 text-base font-medium text-slate-50 disabled:opacity-45 disabled:active:scale-100"
      >
        {submitting ? "Submitting deposit…" : "Submit deposit"}
      </button>

      <p className="wallet-meta-note text-slate-500">
        Target {scriptAddressShort} · delegate {delegatePkShort} · {pendingActivityCount} pending ·
        ref {latestDepositReference}
      </p>
    </form>
  );
}
