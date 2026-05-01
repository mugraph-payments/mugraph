import { useState } from "react";
import type { WalletNoteView } from "../lib/walletView";
import type { WalletWithdrawDraft } from "../types/wallet";
import { DepositDetails } from "./DepositDetails";
import { NotesPanel } from "./NotesPanel";
import { WithdrawDetails } from "./WithdrawDetails";

interface AssetOption {
  id: string;
  label: string;
  balanceLabel: string;
}

interface WalletSettingsScreenProps {
  network: string;
  fundingAddress: string | null;
  delegatePkShort: string;
  scriptAddressShort: string;
  syncPostureLabel: string;
  withdrawDraft: WalletWithdrawDraft;
  onWithdrawDraftChange: (draft: WalletWithdrawDraft) => void;
  latestDepositReference: string;
  latestWithdrawReference: string;
  pendingActivityCount: number;
  topAssetLabel: string;
  assetOptions: AssetOption[];
  notes: WalletNoteView[];
  /** Refresh the wallet snapshot after a successful mutation. */
  onMutationDone: () => Promise<void> | void;
}

export function WalletSettingsScreen({
  network,
  fundingAddress,
  delegatePkShort,
  scriptAddressShort,
  syncPostureLabel,
  withdrawDraft,
  onWithdrawDraftChange,
  latestDepositReference,
  latestWithdrawReference,
  pendingActivityCount,
  topAssetLabel,
  assetOptions,
  notes,
  onMutationDone,
}: WalletSettingsScreenProps) {
  const [activeAdvancedAction, setActiveAdvancedAction] = useState<
    "deposit" | "withdraw" | "notes"
  >("deposit");

  return (
    <section className="wallet-panel p-5 sm:p-6 lg:p-7">
      <div className="grid gap-6">
        <div className="wallet-section-intro">
          <p className="wallet-kicker text-slate-500">Settings</p>
          <h2 className="wallet-heading text-[2rem] text-slate-50">Wallet details</h2>
        </div>

        <section className="wallet-panel-soft p-4 sm:p-5">
          <div className="grid gap-4 lg:grid-cols-3">
            <div>
              <p className="wallet-kicker text-slate-500">Delegate</p>
              <p className="wallet-code mt-2 break-all text-sm text-slate-100">{delegatePkShort}</p>
            </div>
            <div>
              <p className="wallet-kicker text-slate-500">Script</p>
              <p className="wallet-code mt-2 break-all text-sm text-slate-100">
                {scriptAddressShort}
              </p>
            </div>
            <div>
              <p className="wallet-kicker text-slate-500">Sync</p>
              <p className="mt-2 text-sm text-slate-100">{syncPostureLabel}</p>
            </div>
          </div>
        </section>

        <div className="grid gap-3 sm:grid-cols-3">
          <button
            type="button"
            aria-pressed={activeAdvancedAction === "deposit"}
            onClick={() => setActiveAdvancedAction("deposit")}
            className={`wallet-interactive rounded-xl border px-4 py-3 text-sm font-semibold ${
              activeAdvancedAction === "deposit"
                ? "wallet-cta-primary text-slate-50"
                : "wallet-cta-secondary text-slate-200"
            }`}
          >
            Deposit
          </button>
          <button
            type="button"
            aria-pressed={activeAdvancedAction === "withdraw"}
            onClick={() => setActiveAdvancedAction("withdraw")}
            className={`wallet-interactive rounded-xl border px-4 py-3 text-sm font-semibold ${
              activeAdvancedAction === "withdraw"
                ? "wallet-cta-primary text-slate-50"
                : "wallet-cta-secondary text-slate-200"
            }`}
          >
            Withdraw
          </button>
          <button
            type="button"
            aria-pressed={activeAdvancedAction === "notes"}
            onClick={() => setActiveAdvancedAction("notes")}
            className={`wallet-interactive rounded-xl border px-4 py-3 text-sm font-semibold ${
              activeAdvancedAction === "notes"
                ? "wallet-cta-primary text-slate-50"
                : "wallet-cta-secondary text-slate-200"
            }`}
          >
            Notes
          </button>
        </div>

        {activeAdvancedAction === "deposit" ? (
          <DepositDetails
            network={network}
            fundingAddress={fundingAddress}
            scriptAddressShort={scriptAddressShort}
            delegatePkShort={delegatePkShort}
            latestDepositReference={latestDepositReference}
            pendingActivityCount={pendingActivityCount}
            onDone={onMutationDone}
          />
        ) : activeAdvancedAction === "withdraw" ? (
          <WithdrawDetails
            latestWithdrawReference={latestWithdrawReference}
            pendingActivityCount={pendingActivityCount}
            scriptAddressShort={scriptAddressShort}
            topAssetLabel={topAssetLabel}
            draft={withdrawDraft}
            assetOptions={assetOptions}
            onDraftChange={onWithdrawDraftChange}
          />
        ) : (
          <NotesPanel notes={notes} />
        )}
      </div>
    </section>
  );
}
