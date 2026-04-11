# Wallet Integration Plan

This document describes how to connect the wallet UI (Tauri + React) to the
Mugraph node backend, replacing the current stub data with real protocol
operations.

## Current State

The wallet is a Tauri v2 desktop app (430x932 window) with a React/TypeScript
frontend. All data comes from `wallet/src/data/stubWallet.ts` — hardcoded
assets, notes, and activity. The Tauri backend (`wallet/src-tauri/src/lib.rs`)
has only a placeholder `greet` command. No Rust crate dependencies on
`mugraph-core` or `mugraph-node` exist yet.

The node exposes a single JSON-RPC endpoint at `POST /rpc` plus `GET /health`.
All operations use a tagged union `{"m": "operation_name", "p": {...}}` request
format and `{"m": "operation_name", "r": {...}}` response format. The node
already handles: `public_key` (info), `refresh`, `emit` (dev-only), `deposit`,
and `withdraw`.

## Architecture Overview

```
+-------------------+      Tauri IPC       +--------------------+
|  React Frontend   | <------------------> |  Tauri Rust Backend |
|  (wallet/src/)    |   invoke/commands    |  (wallet/src-tauri/)|
+-------------------+                      +--------------------+
                                                    |
                                                    | HTTP JSON-RPC
                                                    v
                                           +--------------------+
                                           |   Mugraph Node     |
                                           |   POST /rpc        |
                                           +--------------------+
                                                    |
                                                    | On-chain
                                                    v
                                           +--------------------+
                                           |     Cardano        |
                                           +--------------------+
```

The wallet never talks to the node directly from the browser context. All
network calls go through Tauri commands in the Rust backend. This keeps
secret material (blinding factors, nonces, keypair) in Rust and avoids
CORS/CSP issues.

## Phase 1: Tauri Backend — Node Client and Local Storage

### 1.1 Add `mugraph-core` dependency to the wallet crate

`wallet/src-tauri` is already a workspace member (declared in root
`Cargo.toml`). Use the workspace dependency form:

```toml
[dependencies]
mugraph-core = { workspace = true }
reqwest = { version = "0.12", features = ["json"] }
redb = { workspace = true }
serde = { version = "1", features = ["derive"] }
serde_json = { workspace = true }
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
tokio = { workspace = true }
```

This gives the wallet access to `Note`, `Refresh`, `Request`, `Response`,
`RefreshBuilder`, the full `crypto` module (blind/sign/unblind/verify), and
the `Keypair` type — everything needed to construct blinded requests
client-side.

### 1.2 Implement a wallet-side node client

Create `wallet/src-tauri/src/node_client.rs` — a thin HTTP client mirroring
the pattern in `simulator/src/client.rs`:

```rust
pub struct NodeClient {
    http: reqwest::Client,
    rpc_url: String,
}

impl NodeClient {
    pub fn new(node_url: &str) -> Self { ... }
    pub async fn info(&self) -> Result<(PublicKey, Option<String>)> { ... }
    pub async fn refresh(&self, refresh: &Refresh) -> Result<Vec<BlindSignature>> { ... }
    pub async fn deposit(&self, req: &DepositRequest) -> Result<Response> { ... }
    pub async fn withdraw(&self, req: &WithdrawRequest) -> Result<Response> { ... }
}
```

Each method serializes a `Request` variant, POSTs to `/rpc`, and deserializes
the `Response`.

### 1.3 Local note storage

Create `wallet/src-tauri/src/store.rs` using `redb` (already a workspace
dependency). Store:

| Table              | Key              | Value                                       |
|--------------------|------------------|---------------------------------------------|
| `config`           | `"node_url"`     | String (e.g. `http://localhost:9999`)        |
| `config`           | `"network"`      | String (`mainnet`, `preprod`, `preview`)     |
| `config`           | `"label"`        | String (wallet label)                        |
| `keypair`          | `"secret_key"`   | `SecretKey` bytes (32 bytes)                 |
| `delegate_info`    | `"pk"`           | `PublicKey` bytes                            |
| `delegate_info`    | `"script_addr"`  | String (Cardano script address)              |
| `notes`            | nonce (Hash)     | Serialized `Note` + status + created_at      |
| `activity`         | id (String)      | Serialized activity record                   |
| `blinding_factors` | nonce (Hash)     | Scalar bytes (32 bytes)                      |

The `keypair` table stores the wallet's long-lived `mugraph_core::Keypair`
identity. Generated once on first launch and persisted. The `PublicKey` from
this keypair is the wallet's identity for blinding operations.

The `blinding_factors` table is the safety net for in-flight operations:

- **Write `r` before sending any blinded request.** This applies to deposit
  outputs, refresh outputs, and withdraw `change_outputs`. The blinding factor
  is generated via `crypto::blind()` which returns `BlindedPoint { factor, point }`.
  The `factor` (a `curve25519_dalek::Scalar`) must hit disk before the HTTP
  request leaves. Loss of `r` between blind and unblind = lost funds.
- **Delete `r` only after the unblinded note is verified and stored.** The
  unblinded signature is computed via `crypto::unblind_signature(&sig, &r, &pk)`
  and then verified with `crypto::verify(&pk, commitment, unblinded_sig)`. Only
  after that verification passes and the note is committed to the `notes` table
  should the corresponding `r` row be removed.
- **On startup, scan for orphaned `r` values.** These represent in-flight
  operations that crashed. The wallet should attempt to recover by re-sending
  the blinded request (if the node supports idempotent deposit/refresh/withdraw)
  or alert the user.

The `r` value can also be stored permanently on the final note inside
`DleqProofWithBlinding.blinding_factor` (`core/src/types/dleq.rs:43-48`),
which already has a field for this purpose.

### 1.4 Expose Tauri commands

Replace the placeholder `greet` command in `wallet/src-tauri/src/lib.rs` with
real commands:

```rust
#[tauri::command]
async fn connect(node_url: String, state: State<'_, AppState>) -> Result<NodeInfo, String>;

#[tauri::command]
async fn get_wallet_state(state: State<'_, AppState>) -> Result<WalletSnapshot, String>;

#[tauri::command]
async fn deposit(utxo_ref: String, amount: u64, asset: String, state: State<'_, AppState>) -> Result<DepositResult, String>;

#[tauri::command]
async fn withdraw(destination: String, amount: u64, asset: String, state: State<'_, AppState>) -> Result<WithdrawResult, String>;

#[tauri::command]
async fn send(notes: Vec<String>, state: State<'_, AppState>) -> Result<SendResult, String>;

#[tauri::command]
async fn refresh_notes(note_ids: Vec<String>, target_denominations: Vec<u64>, state: State<'_, AppState>) -> Result<RefreshResult, String>;

#[tauri::command]
async fn sync(state: State<'_, AppState>) -> Result<SyncResult, String>;
```

`AppState` holds the `NodeClient`, `redb::Database`, and the wallet's
`Keypair`.

## Phase 2: Core Wallet Operations in Rust

### 2.1 Connect (bootstrap)

1. Call `Request::Info` on the node.
2. Receive `Response::Info { delegate_pk, cardano_script_address }`.
3. Store `delegate_pk` and `script_address` locally.
4. If this is the first launch, generate a `Keypair` via `Keypair::random(&mut rng)`
   and persist to the `keypair` table.
5. Return identity info to the frontend.

### 2.2 Deposit (Cardano L1 to Mugraph L2)

This is the flow that gets real funds into the wallet. It has two stages: an
on-chain Cardano transaction, then an off-chain claim against the node.

#### Stage A: On-chain deposit transaction

The deposit UTxO must carry a specific inline Plutus datum with three fields
(defined in `node/src/deposit_datum.rs`):

```
Constr(0, [
    user_pubkey_hash:  Bytes(28),   -- Blake2b-224 of user's Ed25519 pubkey
    node_pubkey_hash:  Bytes(28),   -- Blake2b-224 of node's payment_vk
    intent_hash:       Bytes(32),   -- Blake2b-256 of canonical JSON payload
])
```

The `intent_hash` is a Blake2b-256 hash over the canonical payload (see
`node/src/routes/deposit/signature.rs:159-187`), which includes the UTxO
reference, blinded output encodings, delegate public key, script address,
nonce, and network. This means the wallet must:

1. Decide on the output denominations and blinding ahead of time.
2. Compute the canonical payload and its Blake2b-256 hash.
3. Build the Cardano transaction that sends funds to `script_address` with
   the datum containing `user_pubkey_hash`, `node_pubkey_hash`, and the
   computed `intent_hash`.
4. Submit the transaction on-chain.

This requires `whisky-csl` (or a Cardano transaction builder library) in the
wallet backend to construct and serialize the CBOR transaction with the inline
datum. An external Cardano wallet (Nami, Eternl, etc.) cannot produce this
datum without custom transaction-building support.

#### Stage B: Off-chain deposit claim

After the on-chain transaction is confirmed (the node enforces a confirmation
depth, default 15 blocks — see `node/src/routes/deposit/source_validation.rs:161`):

1. For each output note the wallet wants to mint:
   - Generate a random nonce.
   - Compute the commitment: `Hash(delegate_pk || asset_bytes || amount || nonce)`
     (this is `Note::commitment()` from `core/src/types/note.rs:30-43`).
   - Blind the commitment: call `crypto::blind(&mut rng, commitment.as_ref())`
     which returns `BlindedPoint { factor: r, point: B' }` where
     `B' = H(commitment) + r * G` (see `core/src/crypto.rs:30-42`).
   - **Persist `r` to the `blinding_factors` table immediately.**
   - Pack the compressed `B'` point bytes into a `BlindSignature` struct to
     carry in the request's `outputs` field.
2. Build a `DepositRequest` with:
   - `utxo`: the UTxO reference (`tx_hash`, `index`).
   - `outputs`: the `Vec<BlindSignature>` carrying blinded commitment points.
   - `message`: JSON `{"user_pubkey": "<hex-encoded-ed25519-pubkey>"}`.
   - `signature`: a CIP-8 COSE_Sign1 structure (tagged CBOR, `alg: EdDSA`)
     whose payload is the canonical JSON bytes and whose signature is Ed25519
     over the `tbs_data` (see `node/src/routes/deposit/signature.rs:42-125`).
   - `nonce`: replay-prevention timestamp.
   - `network`: `"mainnet"`, `"preprod"`, or `"preview"`.
3. Send `Request::Deposit(deposit_request)` to the node.
4. Receive `Response::Deposit { signatures, deposit_ref }`:
   - Each signature is a `BlindSignature` containing a blinded signature
     `C' = k * B'` and a DLEQ proof.
   - Verify each DLEQ proof: `crypto::verify_dleq_signature(&delegate_pk, &B', &C', &proof)`.
   - Unblind: `C = crypto::unblind_signature(&C', &r, &delegate_pk)` which
     computes `C' - r * K = k * H(commitment)`.
   - Verify the unblinded signature: `crypto::verify(&delegate_pk, commitment, C)`.
   - Construct full `Note` objects with the unblinded signature.
   - Store notes locally with status `available`.
   - Store the blinding factor in `DleqProofWithBlinding.blinding_factor` on
     the note, then delete the row from the `blinding_factors` table.
5. Record the deposit in the activity log.

**Note on CIP-8 signing**: the deposit request requires an Ed25519 signature
for the COSE_Sign1 envelope. This is a deposit-specific concern — the wallet
can derive or store a separate `ed25519_dalek::SigningKey` in the config table
specifically for deposit authentication. The wallet's primary identity remains
its `mugraph_core::Keypair`.

### 2.3 Withdraw (Mugraph L2 to Cardano L1)

Withdrawal requires the wallet to build a full Cardano transaction that spends
script UTxOs held by the node. This is a significant piece of work requiring
`whisky-csl` or an equivalent Cardano transaction library.

1. User specifies a destination Cardano address and amount.
2. Wallet selects notes that cover the amount (coin selection).
3. Wallet builds the unsigned Cardano transaction:
   - **Inputs**: script UTxOs at the node's script address. Each input must
     carry the deposit datum (`user_pubkey_hash`, `node_pubkey_hash`,
     `intent_hash`) matching the original deposit. The node re-validates
     these datums during withdrawal (`node/src/deposit_datum.rs`).
   - **Outputs**: destination address with the requested amount, plus any
     change outputs that pay back to the script address.
   - **Metadata**: auxiliary metadata label for withdraw intent + network
     binding (validated at `node/src/routes/withdraw/mod.rs:123`).
   - **User witnesses**: Ed25519 signatures from the user matching the
     `user_pubkey_hash` in each input datum.
   - **Fee**: must be under `max_withdrawal_fee` (node config, default
     2,000,000 lovelace) within `fee_tolerance_pct` (default 5%).
   - Compute the transaction hash from the CBOR body.
4. Create a `WithdrawRequest` with:
   - `notes`: `Vec<BlindSignature>` representing the notes to burn.
   - `change_outputs`: `Vec<BlindSignature>` carrying the blinded points for
     each transaction output that pays back to the script address, in the same
     transaction output order. **Persist each corresponding blinding factor to
     `blinding_factors` before sending the request.**
   - `tx_cbor`: hex-encoded unsigned transaction CBOR.
   - `tx_hash`: hex-encoded expected transaction hash (must match the node's
     recomputation from the CBOR).
5. Send `Request::Withdraw(withdraw_request)` to the node.
6. Receive `Response::Withdraw { signed_tx_cbor, tx_hash, change_notes }`:
   - The node has burned the input notes, attached its witness to the
     transaction, and submitted it to the Cardano provider.
   - Mark consumed notes as `spent` locally.
   - Unblind each returned change note using the persisted blinding factor for
     the matching `change_outputs` entry.
   - Verify each unblinded signature and store the resulting change notes as
     `available`, then delete the recovered `r` rows from `blinding_factors`.
7. Record the withdrawal in the activity log.

### 2.4 Send (off-chain, user to user)

Sending notes between users does NOT touch the node. Notes are bearer tokens.

1. User selects notes to send.
2. If exact denominations aren't available, perform a **refresh** first (see
   2.5) to split/merge notes into the desired amounts.
3. Serialize the selected `Note` objects.
4. Transfer the serialized notes to the recipient via any channel (QR code,
   NFC, file, network message — transport is out of scope for now).
5. Mark sent notes as `spent` locally.
6. Recipient imports the notes and verifies each signature against the
   delegate public key:
   `crypto::verify(&delegate_pk, commitment.as_ref(), note.signature)`

**Important**: Off-chain sends do NOT prevent double-spending. The recipient
trusts the sender not to have already used the notes. The recipient can
verify with the delegate via a refresh to "re-mint" the notes and guarantee
they haven't been spent.

### 2.5 Refresh (split, merge, or re-validate notes)

Refresh is the fundamental L2 transaction. It takes input notes and produces
output notes of the same total value, with new nonces and signatures. Uses:

- **Splitting** a large note into smaller denominations.
- **Merging** multiple small notes into a larger one.
- **Re-validating** received notes (proving to yourself they aren't
  double-spent).

The reference implementation for the full BDHKE roundtrip is `emit_note` at
`node/src/routes/refresh.rs:16-46`. The wallet replicates the client side of
that flow.

Flow:

1. Build a `Refresh` using `RefreshBuilder`:
   ```rust
   let refresh = RefreshBuilder::new()
       .input(note_a)     // 1000 USDM
       .input(note_b)     // 500 USDM
       .output(policy_id, asset_name, 750)  // split 1
       .output(policy_id, asset_name, 750)  // split 2
       .build()?;
   ```
   Conservation is enforced: `build()` calls `verify()` which checks that
   per-asset input totals equal output totals (`core/src/types/refresh.rs:89-122`).

2. For each output atom in the built `Refresh`, blind the commitment:
   ```rust
   let commitment = atom.commitment(&refresh.asset_ids);
   let blinded = crypto::blind(&mut rng, commitment.as_ref());
   // blinded.factor = r (Scalar)
   // blinded.point  = H(commitment) + r * G (RistrettoPoint)
   ```
   **Persist each `blinded.factor` to the `blinding_factors` table keyed by
   the atom's nonce BEFORE sending the request.**

3. Send `Request::Refresh(refresh)` to the node.

4. Receive `Response::Transaction { outputs }` — a `Vec<BlindSignature>`,
   one per output atom:
   - For each `(atom, signature, r)` triple:
     - Verify the DLEQ proof:
       `crypto::verify_dleq_signature(&delegate_pk, &blinded_point, &signature.signature, &signature.proof)`.
     - Unblind the signature:
       `let unblinded = crypto::unblind_signature(&signature.signature, &r, &delegate_pk)?;`
     - Verify the final signature:
       `crypto::verify(&delegate_pk, commitment.as_ref(), unblinded)?;`
     - Construct the `Note`:
       ```rust
       Note {
           amount: atom.amount,
           delegate: atom.delegate,
           policy_id: asset.policy_id,
           asset_name: asset.asset_name,
           nonce: atom.nonce,
           signature: unblinded,
           dleq: Some(DleqProofWithBlinding {
               proof: signature.proof,
               blinding_factor: Hash(r.to_bytes()),
           }),
       }
       ```
     - Store the note with status `available`.
     - Delete the `r` row from `blinding_factors`.

5. Mark input notes as `spent`.

### 2.6 Sync

Periodic background operation:

1. Call `Request::Info` to verify the node is reachable and the delegate key
   hasn't changed.
2. Check pending deposits (poll UTxO confirmation status if we add an endpoint
   for this, or rely on the node's deposit monitor).
3. Check pending withdrawals (verify on-chain confirmation).
4. Update `lastSyncedAt`.

## Phase 3: Frontend Integration

### 3.1 Replace stub data with Tauri invoke calls

In the React frontend, replace the static imports from `stubWallet.ts` with
Tauri IPC calls using `@tauri-apps/api/core`:

```typescript
import { invoke } from "@tauri-apps/api/core";

// Instead of importing walletState from stubWallet:
const walletState = await invoke<WalletState>("get_wallet_state");
```

### 3.2 State management

Add a lightweight state layer (React context or a small store) that:

- Calls `get_wallet_state` on mount and after every mutation.
- Provides the `WalletState` to all components via context.
- Exposes mutation functions (`deposit`, `withdraw`, `send`, `refreshNotes`)
  that invoke the corresponding Tauri commands and trigger a re-fetch.

The existing `WalletState` TypeScript type is already well-structured for this.
The `WalletMode` type should be extended from `"stub"` to `"stub" | "live"`.

### 3.3 Wire up action screens

| Screen           | Current behavior       | Integrated behavior                            |
|------------------|------------------------|------------------------------------------------|
| `SendDetails`    | Static draft display   | Invoke `send`, serialize notes, show QR/share  |
| `ReceiveDetails` | Static QR placeholder  | Display script address + QR for deposits       |
| `DepositDetails` | Static form display    | Invoke `deposit` with UTxO ref                 |
| `WithdrawDetails`| Static form display    | Invoke `withdraw` with destination + amount    |
| `NotesPanel`     | Static note list       | Live notes from local store                    |
| `ActivityPanel`  | Static activity list   | Live activity from local store                 |
| `AssetPanel`     | Static asset list      | Computed from live note aggregation             |

### 3.4 Settings screen

The settings screen already shows delegate PK and script address. Wire these
to the real values from `connect`. Add:

- Node URL input (stored in local config).
- Network selector (mainnet/preprod/preview).
- Manual sync trigger.

## Phase 4: Security Considerations

### Blinding factor persistence

The blinding factor `r` is the most security-critical piece of in-flight
wallet state. If lost between blinding and unblinding, the deposit/refresh
funds are **permanently lost**. The wallet must follow this ordering:

1. Generate `r` via `crypto::blind()`.
2. **Write `r` to the `blinding_factors` table (fsync).** This is the point
   of no return — from here, recovery is possible.
3. Send the blinded request over HTTP.
4. Receive the response, unblind with `r`, verify the signature.
5. Write the final `Note` (with `r` stored in `DleqProofWithBlinding`) to the
   `notes` table.
6. Delete the `r` row from `blinding_factors`.

Steps 5 and 6 should be a single redb write transaction for atomicity.

On crash recovery (startup):
- Scan `blinding_factors` for orphaned entries.
- For each, attempt to re-send the original blinded request. If the node
  supports idempotent handling, the same blinded point will produce the same
  signed response.
- If retry fails, surface the orphaned factor to the user for manual recovery.

### Note storage encryption

Notes are bearer tokens. Anyone with access to the note data can spend them.
The local `redb` database should be encrypted at rest. Options:

- OS-level disk encryption (simplest, defer to user).
- Tauri's secure storage plugin for the encryption key.
- Encrypt note values before writing to redb using a key derived from a
  user-provided passphrase.

### Double-spend protection for received notes

Notes received off-chain (via send) should be refreshed immediately. This is
the only way to guarantee they haven't been double-spent. The UI should make
this the default flow: receive -> auto-refresh -> confirmed.

## Phase 5: Future Work

These items are not part of the initial integration but should be considered:

- **CIP-30 integration**: Connect to browser-based Cardano wallets (Nami,
  Eternl, Lace) for deposit transactions. Would require a WebView bridge or
  a separate flow.
- **QR code transport for sends**: Encode serialized notes as QR codes for
  in-person payments. The note JSON is small enough (~300 bytes) to fit.
- **NFC transport**: For mobile targets (if Tauri mobile support is added).
- **Multi-delegate support**: Track notes across multiple delegates. The
  `Note.delegate` field already carries the delegate public key, so the data
  model supports this.
- **Offline queue**: Queue operations when the node is unreachable and submit
  when connectivity is restored. Only applies to refresh/deposit/withdraw —
  sends are already offline-capable.
- **Cross-node transfers**: The node already supports cross-node payments via
  the `cross_node_transfer_*` RPC variants. The wallet would need to know
  about multiple delegates and route payments accordingly.
- **Withdrawal change notes**: The node's `calculate_change_notes`
  (`node/src/routes/withdraw/mod.rs:324`) currently returns an empty vector.
  Once implemented, the wallet should unblind and store change notes using
  the same BDHKE flow as deposit/refresh outputs.

## Implementation Order

1. **`wallet/src-tauri/Cargo.toml`** — add workspace dependencies.
2. **`wallet/src-tauri/src/node_client.rs`** — HTTP client for the node.
3. **`wallet/src-tauri/src/store.rs`** — local redb storage with blinding
   factor table and crash-recovery scan.
4. **`wallet/src-tauri/src/commands.rs`** — Tauri command handlers.
5. **`wallet/src-tauri/src/lib.rs`** — wire up state and commands.
6. **`wallet/src/lib/api.ts`** — TypeScript invoke wrappers.
7. **`wallet/src/lib/walletStore.ts`** — reactive state from Tauri backend.
8. **`wallet/src/App.tsx`** — swap stub imports for live state.
9. **Action screen components** — wire forms to invoke calls.
10. **Settings screen** — node URL configuration.
