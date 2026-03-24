import {
  CaretRight,
  LockKey,
  Pulse,
  ScanSmiley,
} from "@phosphor-icons/react";
import { useState, type ReactNode } from "react";
import { DepositDetails } from "./DepositDetails";
import { NotesPanel } from "./NotesPanel";
import { WithdrawDetails } from "./WithdrawDetails";
import type { WalletNoteView } from "../lib/walletView";
import type { WalletDepositDraft, WalletWithdrawDraft } from "../types/wallet";

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
    <div className="wallet-subtle-card p-3">
      <div className="flex items-center gap-3">
        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-white/[0.05] text-slate-100 ring-1 ring-white/10">
          {icon}
        </div>
        <div className="min-w-0">
          <p className="wallet-kicker text-slate-500">{label}</p>
          <p className="wallet-code mt-1 break-all text-sm text-slate-100">{value}</p>
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
    <section className="wallet-panel p-5 sm:p-6">
      <div className="space-y-2">
        <p className="wallet-kicker text-slate-500">Settings</p>
        <h2 className="wallet-heading text-2xl font-semibold tracking-tight text-slate-50">
          Wallet settings
        </h2>
        <p className="wallet-copy max-w-2xl text-base leading-7 text-slate-400">
          Manage your wallet details, transfers, and note history from one place.
        </p>
      </div>

      <section className="wallet-panel-soft mt-5 p-4 xl:grid xl:grid-cols-[18rem_minmax(0,1fr)] xl:gap-5 xl:p-5">
        <div>
          <div className="flex items-center justify-between gap-3 xl:items-start">
            <div>
              <p className="wallet-kicker text-slate-500">Advanced</p>
              <p className="mt-2 text-base text-slate-300">
                Extra wallet tools and note history live here.
              </p>
            </div>
            <CaretRight className="h-5 w-5 text-slate-500" weight="bold" />
          </div>

          <div className="mt-4 grid gap-3">
            <TechnicalMetaRow
              label="Delegate key"
              value={delegatePkShort}
              icon={<LockKey className="h-4.5 w-4.5" weight="duotone" />}
            />
            <TechnicalMetaRow
              label="Script address"
              value={scriptAddressShort}
              icon={<ScanSmiley className="h-4.5 w-4.5" weight="duotone" />}
            />
            <TechnicalMetaRow
              label="Sync status"
              value={syncPostureLabel}
              icon={<Pulse className="h-4.5 w-4.5" weight="duotone" />}
            />
          </div>
        </div>

        <div className="mt-5 xl:mt-0">
          <div className="grid grid-cols-3 gap-3">
            <button
              type="button"
              aria-pressed={activeAdvancedAction === "deposit"}
              onClick={() => setActiveAdvancedAction("deposit")}
              className={`wallet-interactive rounded-2xl border px-4 py-3 text-base font-semibold ${
                activeAdvancedAction === "deposit"
                  ? "wallet-accent-ring border-teal-300/25 bg-teal-400/[0.08] text-teal-50"
                  : "border-white/10 bg-white/[0.04] text-slate-200"
              }`}
            >
              Deposit
            </button>
            <button
              type="button"
              aria-pressed={activeAdvancedAction === "withdraw"}
              onClick={() => setActiveAdvancedAction("withdraw")}
              className={`wallet-interactive rounded-2xl border px-4 py-3 text-base font-semibold ${
                activeAdvancedAction === "withdraw"
                  ? "wallet-accent-ring border-teal-300/25 bg-teal-400/[0.08] text-teal-50"
                  : "border-white/10 bg-white/[0.04] text-slate-200"
              }`}
            >
              Withdraw
            </button>
            <button
              type="button"
              aria-pressed={activeAdvancedAction === "notes"}
              onClick={() => setActiveAdvancedAction("notes")}
              className={`wallet-interactive rounded-2xl border px-4 py-3 text-base font-semibold ${
                activeAdvancedAction === "notes"
                  ? "wallet-accent-ring border-teal-300/25 bg-teal-400/[0.08] text-teal-50"
                  : "border-white/10 bg-white/[0.04] text-slate-200"
              }`}
            >
              Notes
            </button>
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
      </section>
    </section>
  );
}
