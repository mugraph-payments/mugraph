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

## Product Decisions Locked for v1

The following product decisions are treated as fixed requirements for the
implementation described below:

- **Live-only wallet**: remove stub mode from the shipped app. The current
  stub data remains a development artifact only until live integration is
  complete.
- **Guided setup is mandatory**: first run must collect configuration for
  **all three networks** (`mainnet`, `preprod`, `preview`) before the wallet
  is considered ready.
- **Network model**: the wallet uses one shared Mugraph identity and one
  shared in-app Cardano payment keypair across all networks, while notes,
  activity, node URLs, and chain state stay partitioned per network.
- **Node/provider model**: the wallet must not require node API changes.
  It talks to exactly one configured Mugraph node per network, and it talks
  directly to Cardano providers for chain queries/submission when needed.
- **Provider configuration**: use one provider account model globally
  (same provider type and credentials reused across networks), with the
  network-specific endpoint derived from the selected network.
- **Startup/default network**: after setup, open the **last used network**.
  If one network later has broken configuration, warn at startup but allow
  the user to continue using healthy networks.
- **Receive semantics**: `Receive` is **off-chain only**. Cardano L1 money-in
  belongs exclusively to the separate `Deposit` flow.
- **Deposit UX**: before a user can deposit, show the in-app Cardano funding
  address (and QR) so they can fund the wallet externally. No richer Cardano
  balance dashboard is required for v1.
- **Send transport**: support both copy/paste text and QR transfer. Use a
  JSON envelope with explicit hex-encoded note fields; do **not** add a
  schema version field in v1.
- **Receive request enforcement**: strict asset/amount/label matching applies
  only to off-chain wallet-to-wallet requests, where the request payload is
  carried by the wallet format itself. It does not apply to plain Cardano
  address transfers.
- **Imported note trust model**: imported notes must auto-refresh
  immediately. If refresh fails, keep them in the notes list as a special
  quarantined/untrusted status, exclude them from spendable balance, and put
  the wallet into an attention state.
- **Withdrawal failure UX**: if a withdrawal fails after notes are burned,
  surface a hard attention banner with recovery/support guidance.
- **Coin selection**: use a simple deterministic largest-first note/UTxO
  selection strategy in v1.

## Phase 1: Tauri Backend — Node Client and Local Storage

### 1.1 Add `mugraph-core` dependency to the wallet crate

`wallet/src-tauri` is already a workspace member (declared in root
`Cargo.toml`). Use the workspace dependency form:

```toml
[dependencies]
mugraph-core = { workspace = true }
ed25519-dalek = { version = "2.1", features = ["rand_core"] }
reqwest = { version = "0.12", features = ["json"] }
redb = { workspace = true }
rand = { workspace = true }
serde = { version = "1", features = ["derive"] }
serde_json = { workspace = true }
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
tokio = { workspace = true }
```

This gives the wallet access to `Note`, `Refresh`, `Request`, `Response`,
`RefreshBuilder`, the full `crypto` module (blind/sign/unblind/verify),
`Keypair`, and `ed25519_dalek::SigningKey` — everything needed to construct
blinded requests and authenticate deposits.

### 1.2 Implement a wallet-side node client

Create `wallet/src-tauri/src/node_client.rs` — a thin HTTP client mirroring
the pattern in `simulator/src/client.rs`:

```rust
pub struct NodeClient {
    http: reqwest::Client,
    rpc_url: Url,
    health_url: Url,
}

impl NodeClient {
    pub fn new(base: &Url) -> Result<Self> { ... }
    pub async fn health(&self) -> Result<()> { ... }
    pub async fn info(&self) -> Result<(PublicKey, Option<String>)> { ... }
    pub async fn refresh(&self, refresh: &Refresh) -> Result<Vec<BlindSignature>> { ... }
    pub async fn deposit(&self, req: &DepositRequest) -> Result<DepositResponse> { ... }
    pub async fn withdraw(&self, req: &WithdrawRequest) -> Result<WithdrawResponse> { ... }
}
```

Each method serializes a `Request` variant, POSTs to `/rpc`, deserializes
the `Response`, and pattern-matches to return the typed success variant or
propagate `Response::Error` as an error. See `simulator/src/client.rs` for
the reference pattern (it already implements `health`, `public_key`, `emit`,
`refresh`).

### 1.3 Local note storage

Create `wallet/src-tauri/src/store.rs` using `redb` (already a workspace
dependency). Because v1 uses a **shared identity with per-network chain
state**, the store should separate global identity/config from per-network
state. At minimum, store:

| Table               | Key                           | Value                                       |
| ------------------- | ----------------------------- | ------------------------------------------- |
| `config_global`     | `"label"`                     | String (wallet label)                       |
| `config_global`     | `"last_network"`              | String (`mainnet`, `preprod`, `preview`)    |
| `provider_config`   | `"type"`                      | String (`blockfrost` or `maestro`)          |
| `provider_config`   | `"api_key"`                   | String                                      |
| `provider_config`   | `"base_url_override"`         | Optional String                             |
| `node_config`       | `<network>`                   | String node URL for that network            |
| `keypair`           | `"secret_key"`                | `SecretKey` bytes (32 bytes, Ristretto)     |
| `keypair`           | `"ed25519_sk"`                | Ed25519 signing key (32 bytes)              |
| `cardano_keypair`   | `"payment_sk"`                | Cardano payment signing key bytes           |
| `cardano_keypair`   | `"payment_vk"`                | Cardano payment verification key bytes      |
| `delegate_info`     | `<network>:pk`                | `PublicKey` bytes                           |
| `delegate_info`     | `<network>:script_addr`       | String (Cardano script address)             |
| `notes`             | `<network>:<nonce>`           | Serialized `Note` + status + created_at     |
| `activity`          | `<network>:<id>`              | Serialized activity record                  |
| `blinding_factors`  | `<network>:<nonce>`           | Scalar bytes (32 bytes)                     |
| `offchain_requests` | id                            | Serialized receive request metadata         |
| `cardano_utxos`     | `<network>:<tx_hash>#<index>` | Cached in-app Cardano funding UTxO metadata |

This layout makes the shared-vs-network-specific split explicit:

- Global tables hold identity, provider credentials, and last-used UI state.
- Per-network tables/key prefixes isolate notes, activity, delegate info,
  script addresses, and in-flight blinded operations.

The wallet persists three long-lived keys, all generated on first launch:

- `secret_key`: the wallet's `mugraph_core::Keypair` (Ristretto group).
  The `PublicKey` from this keypair is the wallet's identity for BDHKE
  blinding operations.
- `ed25519_sk`: an `ed25519_dalek::SigningKey` for CIP-8 deposit
  authentication and withdrawal witness signatures. The `Blake2b-224` hash
  of the corresponding `VerifyingKey` is the `user_pubkey_hash` in deposit
  datums.
- `payment_sk` / `payment_vk`: one shared in-app Cardano payment keypair used
  to derive network-specific Cardano addresses for `mainnet`, `preprod`, and
  `preview`.

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
  operations that crashed between blinding and storing the final note. See
  Phase 4 "Blinding factor persistence" for the recovery strategy.

The `r` value can also be stored permanently on the final note inside
`DleqProofWithBlinding.blinding_factor` (`core/src/types/dleq.rs:43-48`),
which already has a field for this purpose.

### 1.4 Expose Tauri commands

Replace the placeholder `greet` command in `wallet/src-tauri/src/lib.rs` with
commands that match the final product model:

```rust
#[tauri::command]
async fn complete_guided_setup(config: SetupConfig, state: State<'_, AppState>) -> Result<SetupResult, String>;

#[tauri::command]
async fn get_wallet_state(network: String, state: State<'_, AppState>) -> Result<WalletSnapshot, String>;

#[tauri::command]
async fn switch_network(network: String, state: State<'_, AppState>) -> Result<WalletSnapshot, String>;

#[tauri::command]
async fn create_receive_request(input: ReceiveRequestInput, state: State<'_, AppState>) -> Result<ReceiveRequest, String>;

#[tauri::command]
async fn import_notes(payload: String, state: State<'_, AppState>) -> Result<ImportResult, String>;

#[tauri::command]
async fn deposit(input: DepositInput, state: State<'_, AppState>) -> Result<DepositResult, String>;

#[tauri::command]
async fn withdraw(input: WithdrawInput, state: State<'_, AppState>) -> Result<WithdrawResult, String>;

#[tauri::command]
async fn send(input: SendInput, state: State<'_, AppState>) -> Result<SendResult, String>;

#[tauri::command]
async fn refresh_notes(note_ids: Vec<String>, target_denominations: Vec<u64>, state: State<'_, AppState>) -> Result<RefreshResult, String>;

#[tauri::command]
async fn sync(network: String, state: State<'_, AppState>) -> Result<SyncResult, String>;
```

`AppState` should hold:

- the `redb::Database`
- the wallet's Mugraph `Keypair`
- the wallet's Ed25519 signing key
- the shared in-app Cardano payment keypair
- provider configuration
- one `NodeClient` per configured network

## Phase 2: Core Wallet Operations in Rust

### 2.1 Connect (bootstrap)

1. Guided setup runs on first launch and must collect configuration for all
   three networks:
   - one node URL for `mainnet`
   - one node URL for `preprod`
   - one node URL for `preview`
   - one provider type (`blockfrost` or `maestro`)
   - one provider credential set reused across all three networks
2. If this is the first launch, generate all wallet keys and persist them
   (this step is independent of node connectivity):
   - `Keypair::random(&mut rng)` for BDHKE operations.
   - `ed25519_dalek::SigningKey::generate(&mut rng)` for CIP-8/witness
     authentication.
   - one Cardano payment keypair for the in-app funding wallet.
3. For each configured network, call `Request::Info` on that network's node.
4. Receive `Response::Info { delegate_pk, cardano_script_address }` and store
   the delegate/script pair under that network namespace.
5. Mark setup complete only after all three networks have passed bootstrap.
6. Open the last-used network on subsequent launches.

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

1. Show the in-app Cardano funding address/QR to the user so they can fund
   the wallet externally before attempting a deposit.
2. Select source UTxOs from the in-app Cardano wallet using a simple
   deterministic largest-first strategy.
3. Decide on the output denominations and blinding ahead of time.
4. Compute the canonical payload and its Blake2b-256 hash.
5. Build the Cardano transaction that sends funds to `script_address` with
   the datum containing `user_pubkey_hash`, `node_pubkey_hash`, and the
   computed `intent_hash`.
6. Submit the transaction on-chain.

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
   - Pack `B'` into a `BlindSignature` for the request's `outputs` field.
     The wallet doesn't have a DLEQ proof at this point — use a default:
     ```rust
     BlindSignature {
         signature: Blinded(Signature::from(blinded.point)),
         proof: DleqProof::default(),
     }
     ```
     The node ignores the proof field on inputs; it produces its own DLEQ
     proof in the response.
2. Build a `DepositRequest` with:
   - `utxo`: the UTxO reference (`tx_hash`, `index`).
   - `outputs`: the `Vec<BlindSignature>` carrying blinded commitment points.
   - `message`: JSON `{"user_pubkey": "<hex-encoded-ed25519-pubkey>"}`.
   - `signature`: a CIP-8 COSE_Sign1 structure (tagged CBOR, `alg: EdDSA`)
     whose payload is the canonical JSON bytes and whose signature is Ed25519
     over the `tbs_data` (see `node/src/routes/deposit/signature.rs:42-125`).
     **This requires an `ed25519_dalek::SigningKey`, not the wallet's
     `mugraph_core::Keypair`.** The BDHKE keypair uses the Ristretto group
     (curve25519-dalek) and cannot produce Ed25519 signatures. The wallet
     must maintain a separate Ed25519 key for deposit/withdraw authentication.
   - `nonce`: replay-prevention timestamp.
   - `network`: `"mainnet"`, `"preprod"`, or `"preview"`.
3. Send `Request::Deposit(deposit_request)` to the node.
4. Receive `Response::Deposit { signatures, deposit_ref }`:
   - Each signature is a `BlindSignature` containing a blinded signature
     `C' = k * B'` and a DLEQ proof.
   - Recover the blinded point `B'` from the original request output
     (the wallet must keep the `BlindedPoint` values from step 1 in memory
     for the duration of the request):
     `let blinded_point = blinded_points[i].point;`
   - Verify each DLEQ proof (`sig` is the `BlindSignature` from the response):
     `assert!(crypto::verify_dleq_signature(&delegate_pk, &blinded_point, &sig.signature, &sig.proof)?);`
   - Unblind:
     `let unblinded = crypto::unblind_signature(&sig.signature, &r, &delegate_pk)?;`
     This computes `C' - r * K = k * H(commitment)`.
   - Verify the unblinded signature (note: `verify` returns `Result<bool>`,
     so check the bool — `?` alone only catches decompression errors):
     `assert!(crypto::verify(&delegate_pk, commitment.as_ref(), unblinded)?);`
   - Construct full `Note` objects with the unblinded signature.
   - Store notes locally with status `available`.
   - Store the blinding factor in `DleqProofWithBlinding.blinding_factor` on
     the note, then delete the row from the `blinding_factors` table.
5. Record the deposit in the activity log.

**Key management note**: the wallet maintains two separate keys:

- **BDHKE keypair** (`mugraph_core::Keypair`): Ristretto-group key for
  blinding/unblinding operations. Stored in the `keypair` table. This is
  the wallet's primary identity for L2 operations.
- **Ed25519 signing key** (`ed25519_dalek::SigningKey`): for CIP-8 deposit
  authentication and withdrawal witness signatures. Stored in the `keypair`
  table under `"ed25519_sk"`. The corresponding `Blake2b-224(verifying_key)`
  is the `user_pubkey_hash` embedded in deposit datums.

### 2.3 Withdraw (Mugraph L2 to Cardano L1)

Withdrawal requires the wallet to build a full Cardano transaction that spends
script UTxOs held by the node. This is a significant piece of work requiring
`whisky-csl` or an equivalent Cardano transaction library.

1. User specifies a destination Cardano address and amount.
2. Wallet selects notes that cover the amount (coin selection).
3. Wallet queries spendable script UTxOs directly from the configured
   Cardano provider (Blockfrost or Maestro) at the node's script address,
   filtering by datum `user_pubkey_hash` because v1 does not change the node
   API surface.
4. Wallet builds the Cardano transaction:
   - **Inputs**: script UTxOs at the node's script address. Each input must
     carry the deposit datum (`user_pubkey_hash`, `node_pubkey_hash`,
     `intent_hash`) matching the original deposit. The node re-validates
     these datums during withdrawal (`node/src/deposit_datum.rs`).
   - **Outputs**: destination address with the requested amount, plus any
     change outputs that pay back to the script address.
   - **Metadata**: auxiliary metadata label for withdraw intent + network
     binding (validated at `node/src/routes/withdraw/mod.rs:132`).
   - **Fee**: must be under `max_withdrawal_fee` (node config, default
     2,000,000 lovelace) within `fee_tolerance_pct` (default 5%).
   - **User witnesses**: Ed25519 signatures over the transaction body hash,
     from the user key matching the `user_pubkey_hash` in each input datum.
     These are added to the transaction's witness set before serializing.
   - Compute the transaction hash: Blake2b-256 of the **transaction body
     bytes only** (not the full transaction CBOR). See
     `node/src/tx_signer.rs:93-103` — the node recomputes this from the
     submitted CBOR and rejects mismatches.
5. Create a `WithdrawRequest` with:
   - `notes`: `Vec<BlindSignature>` representing the notes to burn. Each
     entry carries the note's real (unblinded) signature so the node can
     match it against the spent-notes table:
     ```rust
     BlindSignature {
         signature: Blinded(note.signature),
         proof: DleqProof::default(),
     }
     ```
   - `change_outputs`: `Vec<BlindSignature>` carrying the blinded points for
     each transaction output that pays back to the script address, in the same
     transaction output order. Constructed the same way as deposit outputs:
     ```rust
     BlindSignature {
         signature: Blinded(Signature::from(blinded.point)),
         proof: DleqProof::default(),
     }
     ```
     **Persist each corresponding blinding factor to `blinding_factors`
     before sending the request.**
   - `tx_cbor`: hex-encoded transaction CBOR (body + user witness set).
     The node parses this as a full `csl::Transaction`, validates the user
     witnesses, then adds its own node witness before submitting.
   - `tx_hash`: hex-encoded expected transaction hash (must match the node's
     recomputation from the CBOR).
6. Send `Request::Withdraw(withdraw_request)` to the node.
7. Receive `Response::Withdraw { signed_tx_cbor, tx_hash, change_notes }`:
   - The node has burned the input notes, attached its witness to the
     transaction, and submitted it to the Cardano provider.
   - Mark consumed notes as `spent` locally.
   - Unblind each returned change note using the persisted blinding factor for
     the matching `change_outputs` entry.
   - Verify each unblinded signature and store the resulting change notes as
     `available`, then delete the recovered `r` rows from `blinding_factors`.
8. Record the withdrawal in the activity log.

### 2.4 Send (off-chain, user to user)

Sending notes between users does NOT touch the node. Notes are bearer tokens.

1. User selects notes to send.
2. If exact denominations aren't available, perform a **refresh** first (see
   2.5) to split/merge notes into the desired amounts.
3. Serialize the selected `Note` objects into a v1 JSON envelope for text or
   QR transfer. The envelope is intentionally unversioned in v1 and should
   contain:

   ```json
   {
     "network": "preprod",
     "delegate_pk": "<hex>",
     "sender_label": "Everyday Wallet",
     "created_at": "2026-04-15T12:00:00Z",
     "notes": [
       {
         "amount": 1200,
         "policy_id": "<hex>",
         "asset_name": "<hex>",
         "delegate": "<hex>",
         "nonce": "<hex>",
         "signature": "<hex>"
       }
     ]
   }
   ```

   Use explicit hex-encoded fields rather than opaque note blobs.

4. Offer both transport modes:
   - copy/paste text for any payload size
   - QR only when the payload fits a practical single-code limit; otherwise
     require text sharing in v1
5. Mark sent notes as `spent` locally.
6. Recipient imports the envelope, validates that the envelope network and
   delegate match the active wallet/network, and verifies each signature
   against the delegate public key:
   `crypto::verify(&delegate_pk, note.commitment().as_ref(), note.signature)?`
   (returns `Result<bool>` — check the bool is `true`)

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
`node/src/routes/refresh.rs:23-53`. The wallet replicates the client side of
that flow.

Flow:

1. Build a `Refresh` using `RefreshBuilder`:

   ```rust
   let mut refresh = RefreshBuilder::new()
       .input(note_a)     // 1000 USDM
       .input(note_b)     // 500 USDM
       .output(policy_id, asset_name, 750)  // split 1
       .output(policy_id, asset_name, 750)  // split 2
       .build()?;
   ```

   Conservation is enforced: `build()` calls `verify()` which checks that
   per-asset input totals equal output totals (`core/src/types/refresh.rs:89-122`).

2. For each output atom in the built `Refresh`, blind the commitment and
   attach the blinded point to the `Refresh` before sending:

   ```rust
   let mut blinding_factors = Vec::new();
   let mut blinded_points = Vec::new();

   for (i, atom) in refresh.atoms.iter().enumerate() {
       if refresh.is_output(i) {
           let commitment = atom.commitment(&refresh.asset_ids);
           let blinded = crypto::blind(&mut rng, commitment.as_ref());
           // blinded.factor = r (Scalar)
           // blinded.point  = H(commitment) + r * G (RistrettoPoint)

           blinding_factors.push(blinded.factor);
           // Convert RistrettoPoint -> Signature (compressed 32 bytes)
           blinded_points.push(Signature::from(blinded.point));
       }
   }

   // Attach blinded points to the Refresh.
   // CRITICAL: if this field is left empty, the node silently falls back
   // to server-side hash_to_curve and the wallet cannot unblind the
   // response. The field has #[serde(skip_serializing_if = "Vec::is_empty")]
   // so an empty vec disappears from JSON with no error.
   refresh.blinded_points = blinded_points;
   ```

   **Persist each `blinding_factor` to the `blinding_factors` table keyed by
   the atom's nonce BEFORE sending the request.**

3. Send `Request::Refresh(refresh)` to the node.

4. Receive `Response::Transaction { outputs }` — a `Vec<BlindSignature>`,
   one per output atom:
   - For each `(atom, sig, r)` triple (where `sig: BlindSignature`):
     - Recover the blinded point for DLEQ verification:
       `let blinded_point = refresh.blinded_points[output_idx].to_point()?;`
     - Verify the DLEQ proof (`verify_dleq_signature` returns `Result<bool>`):
       `assert!(crypto::verify_dleq_signature(&delegate_pk, &blinded_point, &sig.signature, &sig.proof)?);`
     - Unblind the signature:
       `let unblinded = crypto::unblind_signature(&sig.signature, &r, &delegate_pk)?;`
     - Verify the final signature:
       `assert!(crypto::verify(&delegate_pk, commitment.as_ref(), unblinded)?);`
     - Look up the asset: `let asset = &refresh.asset_ids[atom.asset_id as usize];`
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
               proof: sig.proof,
               blinding_factor: r.into(),
           }),
       }
       ```
     - Store the note with status `available`.
     - Delete the `r` row from `blinding_factors`.

5. Mark input notes as `spent`.

The reference test for this full client-side flow is
`refresh_with_blinded_points_produces_unblindable_signatures` in
`node/src/routes/refresh.rs:197-267`.

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

const walletState = await invoke<WalletState>("get_wallet_state", {
  network: activeNetwork,
});
```

### 3.2 State management

Add a lightweight state layer (React context or a small store) that:

- requires guided setup before entering the main wallet shell
- restores the last-used network on launch
- calls `get_wallet_state(activeNetwork)` on mount and after every mutation
- provides the active-network `WalletState` to all components via context
- exposes mutation functions (`createReceiveRequest`, `importNotes`,
  `deposit`, `withdraw`, `send`, `refreshNotes`) that invoke the
  corresponding Tauri commands and trigger a re-fetch
- surfaces startup warnings for broken network configs without blocking
  healthy networks

The existing `WalletState` TypeScript type is already well-structured for
this, but several fields require data that notes alone do not carry:

- `AssetHolding.ticker`, `.name`, `.decimals`: notes only store raw
  `policy_id` + `asset_name` bytes. The wallet needs a local asset
  registry (a static or user-maintained JSON map of `policy_id` to
  display metadata). For Milestone A, hardcode the known test assets
  (ADA/lovelace, USDM, etc). For Milestone B, consider fetching metadata
  from a Cardano token registry or the node.
- `PortfolioSummary.totalValueUsd`, `AssetHolding.shareOfWalletPct`:
  require price feeds. Omit or zero these fields in Milestone A. The
  frontend should gracefully handle missing price data.
- `AssetHolding.trend`: requires historical balance tracking. Omit in
  Milestone A (default to `"flat"`).

### 3.3 Wire up action screens

| Screen            | Current behavior      | Integrated behavior                                                                                |
| ----------------- | --------------------- | -------------------------------------------------------------------------------------------------- |
| `SendDetails`     | Static draft display  | Invoke `send`, emit text/QR off-chain envelope                                                     |
| `ReceiveDetails`  | Static QR placeholder | Create strict off-chain receive requests only; no Cardano L1 semantics                             |
| `DepositDetails`  | Static form display   | Show funding address/QR first, then build/sign the on-chain deposit and submit the off-chain claim |
| `WithdrawDetails` | Static form display   | Invoke `withdraw` with destination + amount and escalate failed burns via hard attention state     |
| `NotesPanel`      | Static note list      | Live notes from local store, including quarantine states                                           |
| `ActivityPanel`   | Static activity list  | Live activity from local store                                                                     |
| `AssetPanel`      | Static asset list     | Computed from live note aggregation                                                                |

### 3.4 Error handling

Tauri commands return `Result<T, String>`. The frontend should map errors
to wallet status and UI state:

| Error class                 | `WalletStatus` | UI behavior                        |
| --------------------------- | -------------- | ---------------------------------- |
| Node unreachable / timeout  | `attention`    | Banner: "Node offline", retry sync |
| BDHKE verification failure  | `attention`    | Banner: "Signature mismatch"       |
| Unbalanced refresh          | `ready`        | Inline form error                  |
| Blinding factor persistence | `attention`    | Block operation until resolved     |
| Orphaned blinding factors   | `attention`    | Startup recovery prompt            |

The shipped wallet should run in **live-only** mode. Stub data may remain in
source control temporarily for UI development, but it is not part of the
runtime product model and should not appear as a user-facing mode toggle.

Off-chain receive requests should use a strict wallet-defined payload so the
receiver can enforce the requested asset/amount/label exactly when the sender
uses the same wallet flow. At minimum, the request payload should carry:

```json
{
  "network": "preprod",
  "delegate_pk": "<hex>",
  "recipient_label": "Everyday Wallet",
  "asset": {
    "policy_id": "<hex>",
    "asset_name": "<hex>"
  },
  "amount": 1200,
  "label": "Invoice #482"
}
```

`ReceiveDetails` creates this payload for text/QR sharing. `import_notes` or a
paired send flow must reject note envelopes that do not match the active
request when the user is fulfilling a strict off-chain request.

### 3.5 Settings screen

The settings screen already shows delegate PK and script address. Wire these
to the real values from live bootstrap/setup. Add:

- Node URL inputs for all three networks.
- One shared provider configuration block (provider type, API key, optional
  base URL override) reused across networks.
- Network selector (mainnet/preprod/preview).
- Manual sync trigger.
- Startup warning surface for broken network configs that does not block
  healthy networks.

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

- Scan `blinding_factors` for orphaned entries (entries whose nonce does
  not appear in the `notes` table with status `available`).
- Orphaned entries indicate an in-flight operation that crashed between
  blinding and storing the final note. The wallet cannot automatically
  retry because the `blinding_factors` table only stores the scalar —
  it does not store enough context to reconstruct the original request.
- Surface orphaned factors to the user with the nonce and timestamp so
  they can decide whether to attempt manual recovery or write off the
  funds.
- To enable automatic retry in the future, consider extending the
  `blinding_factors` table to also store the operation type, blinded
  point, and enough request context to replay.

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

If auto-refresh fails:

- keep the imported notes in the notes list with a dedicated quarantined /
  untrusted status
- exclude quarantined notes from spendable balances and send/withdraw flows
- surface wallet status as `attention`
- provide a retry/discard path from the note detail or notes list

## Phase 5: Future Work

These items are not part of the initial integration but should be considered:

- **CIP-30 integration**: Connect to browser-based Cardano wallets (Nami,
  Eternl, Lace) for deposit transactions. Would require a WebView bridge or
  a separate flow.
- **Multipart QR transport**: v1 only supports single-code QR sends. A later
  version may add multipart QR transfer for larger note bundles.
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

## Implementation Order

The work is split into two milestones. The first delivers a functional L2
wallet that can be tested end-to-end against a dev-mode node (which can
`emit` notes). The second adds Cardano L1 integration.

### Milestone A: Off-chain wallet (connect + refresh + send)

1. **`wallet/src-tauri/Cargo.toml`** — add dependencies per section 1.1
   (`mugraph-core`, `reqwest`, `redb`, `rand`, `ed25519-dalek`, `tokio`, etc).
2. **`wallet/src-tauri/src/node_client.rs`** — HTTP client for the node
   (mirror `simulator/src/client.rs`). Methods: `info`, `refresh`.
3. **`wallet/src-tauri/src/store.rs`** — local redb storage with `config`,
   `keypair`, `delegate_info`, `notes`, `activity`, `blinding_factors` tables.
   Crash-recovery scan for orphaned blinding factors on startup.
4. **`wallet/src-tauri/src/commands.rs`** — Tauri commands: `connect`,
   `get_wallet_state`, `refresh_notes`, `send`, `sync`.
5. **`wallet/src-tauri/src/lib.rs`** — wire up `AppState` and commands.
6. **`wallet/src/lib/api.ts`** — TypeScript invoke wrappers.
7. **`wallet/src/lib/walletStore.ts`** — reactive state from Tauri backend.
8. **`wallet/src/App.tsx`** — swap stub imports for live state.
9. **Action screen components** — wire send/receive forms to invoke calls.
10. **Settings screen** — guided setup support, three node URL inputs,
    shared provider config, network selector, sync, startup warnings.

At this point the wallet can connect to a dev-mode node, receive emitted
notes, refresh (split/merge) them, and send bearer tokens to other users.

### Milestone B: Cardano L1 integration (deposit + withdraw)

Additional dependencies for this milestone: `whisky-csl` (Cardano tx
building), `coset` (COSE_Sign1), `blake2` (intent hash), `hex`.

11. **`wallet/src-tauri/src/node_client.rs`** — add `deposit`, `withdraw`
    methods.
12. **CIP-8 signature construction** — add `coset` and `blake2` as
    dependencies. Implement the COSE_Sign1 envelope builder: set
    `alg: EdDSA` in the protected header, embed the canonical payload
    bytes (from `build_canonical_payload`), sign the `tbs_data` with the
    Ed25519 key, and serialize with `to_tagged_vec()`. The reference
    implementation is `build_cip8_signature` in the deposit test suite
    (`node/src/routes/deposit.rs:128-170`).
13. **Deposit flow** — Cardano transaction building with inline Plutus datum
    (`whisky-csl`), CIP-8 signature construction, BDHKE output blinding.
14. **Withdraw flow** — Cardano transaction building for script UTxO spending,
    change output blinding, witness attachment.
15. **Deposit/withdraw UI** — wire `DepositDetails` and `WithdrawDetails`
    screens to invoke calls, including the funding-address-first deposit UX
    and hard attention handling for failed withdrawals.
