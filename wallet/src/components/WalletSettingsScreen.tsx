import { LockKey, Pulse, ScanSmiley } from "@phosphor-icons/react";
import { useState, type ReactNode } from "react";
import type { WalletNoteView } from "../lib/walletView";
import type { WalletDepositDraft, WalletWithdrawDraft } from "../types/wallet";
import { DepositDetails } from "./DepositDetails";
import { NotesPanel } from "./NotesPanel";
import { WithdrawDetails } from "./WithdrawDetails";

interface AssetOption {
  id: string;
  label: string;
  balanceLabel: string;
}

interface WalletSettingsScreenProps {
  delegatePkShort: string;
  scriptAddressShort: string;
  syncPostureLabel: string;
  depositDraft: WalletDepositDraft;
  onDepositDraftChange: (draft: WalletDepositDraft) => void;
  withdrawDraft: WalletWithdrawDraft;
  onWithdrawDraftChange: (draft: WalletWithdrawDraft) => void;
  latestDepositReference: string;
  latestWithdrawReference: string;
  pendingActivityCount: number;
  topAssetLabel: string;
  assetOptions: AssetOption[];
  notes: WalletNoteView[];
}

function TechnicalMetaRow({
  label,
  value,
  icon,
}: {
  label: string;
  value: string;
  icon: ReactNode;
}) {
  return (
    <div className="wallet-subtle-card p-4">
      <div className="flex items-start gap-3">
        <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-white/[0.05] text-slate-100 ring-1 ring-white/10">
          {icon}
        </div>
        <div className="min-w-0">
          <p className="wallet-kicker text-slate-500">{label}</p>
          <p className="wallet-code mt-2 break-all text-sm text-slate-100">{value}</p>
        </div>
      </div>
    </div>
  );
}

export function WalletSettingsScreen({
  delegatePkShort,
  scriptAddressShort,
  syncPostureLabel,
  depositDraft,
  onDepositDraftChange,
  withdrawDraft,
  onWithdrawDraftChange,
  latestDepositReference,
  latestWithdrawReference,
  pendingActivityCount,
  topAssetLabel,
  assetOptions,
  notes,
}: WalletSettingsScreenProps) {
  const [activeAdvancedAction, setActiveAdvancedAction] = useState<
    "deposit" | "withdraw" | "notes"
  >("deposit");

  return (
    <section className="wallet-panel p-5 sm:p-6 lg:p-7">
      <div className="wallet-screen-grid">
        <div className="wallet-screen-rail">
          <div className="wallet-section-intro">
            <p className="wallet-kicker text-slate-500">Settings</p>
            <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
              Wallet settings
            </h2>
            <p className="wallet-copy max-w-[32ch] text-base leading-7 text-slate-400">
              Keep core wallet identity close at hand while switching between funding, settlement,
              and note review tools.
            </p>
          </div>

          <div className="grid gap-3">
            <TechnicalMetaRow
              label="Delegate key"
              value={delegatePkShort}
              icon={<LockKey className="h-[1.125rem] w-[1.125rem]" weight="duotone" />}
            />
            <TechnicalMetaRow
              label="Script address"
              value={scriptAddressShort}
              icon={<ScanSmiley className="h-[1.125rem] w-[1.125rem]" weight="duotone" />}
            />
            <TechnicalMetaRow
              label="Sync status"
              value={syncPostureLabel}
              icon={<Pulse className="h-[1.125rem] w-[1.125rem]" weight="duotone" />}
            />
          </div>
        </div>

        <div className="wallet-screen-body">
          <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_auto] xl:items-end">
            <div className="wallet-section-intro">
              <p className="wallet-kicker text-slate-500">Advanced tools</p>
              <p className="wallet-copy max-w-[42ch] text-base leading-7 text-slate-400">
                Choose the flow you want to work in, then keep the active draft and supporting
                wallet details visible together.
              </p>
            </div>

            <div className="grid grid-cols-1 gap-3 sm:grid-cols-3 xl:min-w-[23rem]">
              <button
                type="button"
                aria-pressed={activeAdvancedAction === "deposit"}
                onClick={() => setActiveAdvancedAction("deposit")}
                className={`wallet-interactive rounded-xl border px-4 py-3 text-base font-semibold ${
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
                className={`wallet-interactive rounded-xl border px-4 py-3 text-base font-semibold ${
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
                className={`wallet-interactive rounded-xl border px-4 py-3 text-base font-semibold ${
                  activeAdvancedAction === "notes"
                    ? "wallet-cta-primary text-slate-50"
                    : "wallet-cta-secondary text-slate-200"
                }`}
              >
                Notes
              </button>
            </div>
          </div>

          {activeAdvancedAction === "deposit" ? (
            <DepositDetails
              scriptAddressShort={scriptAddressShort}
              delegatePkShort={delegatePkShort}
              latestDepositReference={latestDepositReference}
              pendingActivityCount={pendingActivityCount}
              draft={depositDraft}
              assetOptions={assetOptions}
              onDraftChange={onDepositDraftChange}
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
      </div>
    </section>
  );
}
