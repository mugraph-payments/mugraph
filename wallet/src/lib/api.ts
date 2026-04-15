import { invoke } from "@tauri-apps/api/core";

export interface SetupConfig {
  label: string;
  mainnet_node_url: string;
  preprod_node_url: string;
  preview_node_url: string;
  provider_type: string;
  provider_api_key: string;
  provider_base_url_override: string | null;
}

export interface NetworkBootstrap {
  network: string;
  delegate_pk: string;
  cardano_script_address: string | null;
}

export interface SetupResult {
  networks: NetworkBootstrap[];
}

export interface StoredNote {
  note: {
    amount: number;
    delegate: string;
    policy_id: string;
    asset_name: string;
    nonce: string;
    signature: string;
  };
  status: "available" | "spent" | "quarantined";
  created_at: number;
}

export interface WalletSnapshot {
  network: string;
  notes: StoredNote[];
  delegate_pk: string | null;
  cardano_script_address: string | null;
  has_orphaned_blinding_factors: boolean;
}

export interface ImportResult {
  imported: number;
  quarantined: number;
}

export interface SendResult {
  envelope: string;
  /** "qr" if the payload fits a single QR code, "text" otherwise. */
  transport_hint: "qr" | "text";
}

export interface RefreshResult {
  new_note_count: number;
}

export interface SyncResult {
  node_reachable: boolean;
  delegate_pk_changed: boolean;
}

export async function completeGuidedSetup(config: SetupConfig): Promise<SetupResult> {
  return invoke<SetupResult>("complete_guided_setup", { config });
}

export async function getWalletState(network: string): Promise<WalletSnapshot> {
  return invoke<WalletSnapshot>("get_wallet_state", { network });
}

export async function switchNetwork(network: string): Promise<WalletSnapshot> {
  return invoke<WalletSnapshot>("switch_network", { network });
}

export async function createReceiveRequest(input: {
  network: string;
  policy_id: string;
  asset_name: string;
  amount: number;
  label?: string;
}): Promise<string> {
  return invoke<string>("create_receive_request", { input });
}

export async function importNotes(payload: string): Promise<ImportResult> {
  return invoke<ImportResult>("import_notes", { payload });
}

export async function sendNotes(input: {
  network: string;
  note_nonces: string[];
}): Promise<SendResult> {
  return invoke<SendResult>("send", { input });
}

export async function refreshNotes(input: {
  network: string;
  note_nonces: string[];
  target_amounts: number[];
}): Promise<RefreshResult> {
  return invoke<RefreshResult>("refresh_notes", { input });
}

export async function syncNetwork(network: string): Promise<SyncResult> {
  return invoke<SyncResult>("sync", { network });
}

export interface DepositInput {
  network: string;
  utxo_tx_hash: string;
  utxo_index: number;
  output_amounts: number[];
}

export interface DepositResult {
  notes_created: number;
  deposit_ref: string;
}

export interface WithdrawInput {
  network: string;
  destination_address: string;
  amount: number;
}

export interface WithdrawResult {
  tx_hash: string;
  change_notes: number;
}

export async function deposit(input: DepositInput): Promise<DepositResult> {
  return invoke<DepositResult>("deposit", { input });
}

export async function withdraw(input: WithdrawInput): Promise<WithdrawResult> {
  return invoke<WithdrawResult>("withdraw", { input });
}
