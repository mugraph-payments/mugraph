# Wallet Integration Checklist

Exhaustive task checklist derived from [wallet-integration.md](./wallet-integration.md).

---

## Milestone A: Off-chain Wallet (connect + refresh + send)

### Phase 1: Tauri Backend — Node Client and Local Storage

#### 1.1 Add `mugraph-core` dependency to wallet crate

- [x] Add `mugraph-core = { workspace = true }` to `wallet/src-tauri/Cargo.toml`
- [x] Add `ed25519-dalek = { version = "2.1", features = ["rand_core"] }`
- [x] Add `reqwest = { version = "0.12", features = ["json"] }`
- [x] Add `redb = { workspace = true }`
- [x] Add `rand = { workspace = true }`
- [x] Add `serde = { version = "1", features = ["derive"] }`
- [x] Add `serde_json = { workspace = true }`
- [x] Add `tokio = { workspace = true }`

#### 1.2 Implement wallet-side node client (`wallet/src-tauri/src/node_client.rs`)

- [x] Create `NodeClient` struct with `reqwest::Client`, `rpc_url`, `health_url`
- [x] Implement `NodeClient::new(base: &Url)` constructor
- [x] Implement `health()` method (`GET /health`)
- [x] Implement `info()` method (`Request::Info` -> returns `PublicKey`, optional script address)
- [x] Implement `refresh()` method (`Request::Refresh` -> returns `Vec<BlindSignature>`)
- [x] Wire serialization using tagged union format `{"m": "...", "p": {...}}`
- [x] Wire deserialization using `{"m": "...", "r": {...}}` response format
- [x] Pattern-match `Response::Error` into proper error propagation

#### 1.3 Local note storage (`wallet/src-tauri/src/store.rs`)

- [x] Create redb database initialization
- [x] Create `config_global` table (wallet label, last network)
- [x] Create `provider_config` table (type, api_key, base_url_override)
- [x] Create `node_config` table (keyed by network -> node URL)
- [x] Create `keypair` table (secret_key bytes, ed25519_sk bytes)
- [x] Create `cardano_keypair` table (payment_sk, payment_vk)
- [x] Create `delegate_info` table (`<network>:pk`, `<network>:script_addr`)
- [x] Create `notes` table (`<network>:<nonce>` -> serialized Note + status + created_at)
- [x] Create `activity` table (`<network>:<id>` -> serialized activity record)
- [x] Create `blinding_factors` table (`<network>:<nonce>` -> Scalar bytes)
- [x] Create `offchain_requests` table (id -> serialized receive request metadata)
- [x] Create `cardano_utxos` table (`<network>:<tx_hash>#<index>` -> UTxO metadata)
- [x] Implement crash-recovery scan for orphaned blinding factors on startup
- [x] Surface orphaned factors to user with nonce + timestamp

#### 1.4 Expose Tauri commands (`wallet/src-tauri/src/commands.rs` + `lib.rs`)

- [x] Define `AppState` struct (redb Database, Mugraph Keypair, Ed25519 signing key, Cardano payment keypair, provider config, per-network NodeClients)
- [x] Implement `complete_guided_setup` command
- [x] Implement `get_wallet_state` command
- [x] Implement `switch_network` command
- [x] Implement `create_receive_request` command
- [x] Implement `import_notes` command
- [x] Implement `deposit` command
- [x] Implement `withdraw` command
- [x] Implement `send` command
- [x] Implement `refresh_notes` command
- [x] Implement `sync` command
- [x] Remove placeholder `greet` command from `lib.rs`
- [x] Wire up AppState and all commands in `lib.rs`

### Phase 2: Core Wallet Operations in Rust

#### 2.1 Connect / Bootstrap (guided setup)

- [x] Implement guided setup flow collecting config for all 3 networks (mainnet, preprod, preview)
- [x] Collect one node URL per network
- [x] Collect one provider type (blockfrost or maestro)
- [x] Collect one provider credential set (reused across networks)
- [x] Use one shared Mugraph identity and one shared in-app Cardano payment keypair across all networks
- [x] On first launch: generate `Keypair::random()` for BDHKE operations and persist
- [x] On first launch: generate `ed25519_dalek::SigningKey` for CIP-8/witness auth and persist
- [x] On first launch: generate one Cardano payment keypair and persist
- [x] For each network: call `Request::Info` on that network's node
- [x] Store `delegate_pk` + `cardano_script_address` per network namespace
- [x] Mark setup complete only after all 3 networks pass bootstrap
- [x] On subsequent launches: open last-used network
- [x] Handle broken network config at startup: warn but allow healthy networks

#### 2.4 Send (off-chain, user to user)

- [x] Implement coin selection (largest-first deterministic)
- [x] If exact denominations unavailable: trigger refresh first to split/merge
- [x] Serialize selected Notes into v1 JSON envelope (network, delegate_pk, sender_label, created_at, notes array with hex-encoded fields)
- [x] Do not add a schema/version field to the v1 off-chain send envelope
- [x] Support copy/paste text transport
- [x] Support QR transport (when payload fits single-code limit; otherwise require text)
- [x] Mark sent notes as `spent` locally
- [x] Implement import: validate envelope network + delegate match active wallet
- [x] Implement import: verify each note signature via `crypto::verify(&delegate_pk, commitment, signature)` and require the returned bool to be `true`
- [x] Implement auto-refresh of imported notes immediately after import
- [x] If auto-refresh fails: keep notes with quarantined/untrusted status
- [x] Exclude quarantined notes from spendable balance
- [x] Set wallet status to `attention` when quarantined notes exist
- [x] Provide retry/discard path for quarantined notes

#### 2.5 Refresh (split, merge, re-validate)

- [x] Build `Refresh` using `RefreshBuilder` (`.input()` / `.output()` / `.build()`)
- [x] For each output atom: compute commitment via `atom.commitment(&refresh.asset_ids)`
- [x] For each output atom: blind commitment via `crypto::blind(&mut rng, commitment.as_ref())`
- [x] Convert blinded points to `Signature` and attach to `refresh.blinded_points`
- [x] Ensure `refresh.blinded_points` is populated for every output before serialization
- [x] Persist each blinding factor to `blinding_factors` table BEFORE sending request
- [x] Send `Request::Refresh(refresh)` to node
- [x] Receive `Response::Transaction { outputs }`
- [x] For each output: recover blinded point for DLEQ verification
- [x] For each output: verify DLEQ proof via `crypto::verify_dleq_signature()`
- [x] For each output: unblind signature via `crypto::unblind_signature()`
- [x] For each output: verify final signature via `crypto::verify()` and require the returned bool to be `true`
- [x] For each output: construct full `Note` with unblinded signature + `DleqProofWithBlinding`
- [x] Store new notes with status `available`
- [x] Delete recovered `r` rows from `blinding_factors`
- [x] Mark input notes as `spent`

#### 2.6 Sync

- [x] Implement periodic `Request::Info` to verify node reachability
- [x] Detect if delegate key has changed
- [x] Check pending deposit status
- [x] Check pending withdrawal on-chain confirmation
- [x] Update `lastSyncedAt`

#### 2.7 Milestone A dev/test note seeding

- [x] Define how Milestone A gets initial notes for end-to-end testing before L1 deposit exists
- [x] Use the node's dev-only `emit` capability or another documented manual seeding path for local/dev testing

### Phase 3: Frontend Integration

#### 3.1 Replace stub data with Tauri invoke calls

- [x] Replace static imports from `stubWallet.ts` with `invoke()` calls via `@tauri-apps/api/core`
- [x] Route all node/provider access through Tauri commands; no direct browser-context RPC calls
- [x] Remove any user-facing stub/demo mode from the shipped app (live-only wallet)
- [x] Ensure no live/stub mode toggle appears in the production UI
- [x] Create `wallet/src/lib/api.ts` — TypeScript invoke wrappers for all commands

#### 3.2 State management (`wallet/src/lib/walletStore.ts`)

- [x] Require guided setup completion before entering main wallet shell
- [x] Restore last-used network on launch
- [x] Call `get_wallet_state(activeNetwork)` on mount and after every mutation
- [x] Trigger periodic/background `sync` for the active network
- [x] Provide active-network `WalletState` to all components via context
- [x] Expose mutation functions: `createReceiveRequest`, `importNotes`, `deposit`, `withdraw`, `send`, `refreshNotes`, `sync`
- [x] Surface startup warnings for broken network configs without blocking healthy networks
- [x] Hardcode known test asset metadata (ADA/lovelace, USDM) for Milestone A
- [x] Handle missing price data gracefully (zero/omit `totalValueUsd`, `shareOfWalletPct`)
- [x] Default `AssetHolding.trend` to `"flat"` for Milestone A

#### 3.3 Wire up action screens

- [x] `SendDetails` — invoke `send`, emit text/QR off-chain envelope
- [x] `ReceiveDetails` — create strict off-chain receive requests only (no L1 semantics)
- [x] `NotesPanel` — live notes from local store including quarantine states
- [x] `ActivityPanel` — live activity from local store
- [x] `AssetPanel` — computed from live note aggregation

#### 3.4 Error handling

- [x] Map node unreachable/timeout -> `attention` status + "Node offline" banner with retry
- [x] Map BDHKE verification failure -> `attention` status + "Signature mismatch" banner
- [x] Map unbalanced refresh -> `ready` status + inline form error
- [x] Map blinding factor persistence errors -> `attention` + block operation until resolved
- [x] Map orphaned blinding factors -> `attention` + startup recovery prompt

#### 3.5 Settings screen

- [x] Wire delegate PK and script address to real values from bootstrap
- [x] Add node URL inputs for all 3 networks
- [x] Add shared provider config block (provider type, API key, optional base URL override)
- [x] Add network selector (mainnet/preprod/preview)
- [x] Add manual sync trigger
- [x] Add startup warning surface for broken network configs

#### Off-chain receive request payload

- [x] Define strict receive request JSON payload (network, delegate_pk, recipient_label, asset, amount, label)
- [x] `import_notes` / send flow must reject envelopes not matching active strict request

---

## Milestone B: Cardano L1 Integration (deposit + withdraw)

### Additional dependencies

- [x] Add `whisky-csl` (Cardano tx building) to `wallet/src-tauri/Cargo.toml`
- [x] Add `coset` (COSE_Sign1) dependency
- [x] Add `blake2` (intent hash) dependency
- [x] Add `hex` dependency

### Node client extensions

- [x] Add `deposit()` method to `NodeClient`
- [x] Add `withdraw()` method to `NodeClient`

### CIP-8 signature construction

- [x] Implement COSE_Sign1 envelope builder
- [x] Set `alg: EdDSA` in protected header
- [x] Embed canonical payload bytes from `build_canonical_payload`
- [x] Sign `tbs_data` with Ed25519 key
- [x] Serialize with `to_tagged_vec()`

### 2.2 Deposit — Stage A: On-chain deposit transaction

- [x] Show in-app Cardano funding address + QR so user can fund wallet externally
- [x] Select source UTxOs from in-app Cardano wallet (largest-first strategy)
- [x] Decide output denominations and blinding ahead of time
- [x] Compute canonical payload and its Blake2b-256 hash (`intent_hash`)
- [x] Build Cardano transaction sending funds to `script_address` with inline Plutus datum:
  - [x] `user_pubkey_hash` (Blake2b-224 of Ed25519 verifying key)
  - [x] `node_pubkey_hash` (Blake2b-224 of node's payment_vk)
  - [x] `intent_hash` (Blake2b-256 of canonical JSON payload)
- [x] Submit transaction on-chain

### 2.2 Deposit — Stage B: Off-chain deposit claim

- [x] Wait until the deposit UTxO reaches the required confirmation depth before sending `Request::Deposit`
- [x] Surface pending/confirming deposit status in activity/UI before off-chain claim
- [x] For each output note: generate random nonce
- [x] For each output note: compute commitment via `Note::commitment()`
- [x] For each output note: blind commitment via `crypto::blind()`
- [x] Persist `r` to `blinding_factors` table immediately
- [x] Pack blinded points into `BlindSignature` with default `DleqProof`
- [x] Keep original blinded points in memory so response-side DLEQ proofs can be verified
- [x] Build `DepositRequest` with: utxo ref, outputs, message (user_pubkey JSON), CIP-8 signature, nonce, network
- [x] Send `Request::Deposit(deposit_request)` to node
- [x] Receive `Response::Deposit { signatures, deposit_ref }`
- [x] For each response signature: verify DLEQ proof
- [x] For each response signature: unblind via `crypto::unblind_signature()`
- [x] For each response signature: verify unblinded signature (check bool, not just `?`)
- [x] Construct full `Note` objects with unblinded signatures
- [x] Store notes with status `available`
- [x] Store blinding factor in `DleqProofWithBlinding.blinding_factor`
- [x] Delete `r` rows from `blinding_factors` table
- [x] Record deposit in activity log

### 2.3 Withdraw (Mugraph L2 to Cardano L1)

- [x] Accept destination Cardano address + amount from user
- [x] Select notes covering the amount (coin selection, largest-first)
- [x] Query spendable script UTxOs from Cardano provider at node's script address
- [x] Filter UTxOs by datum `user_pubkey_hash`
- [x] Build Cardano transaction:
  - [x] Inputs: script UTxOs with matching deposit datums
  - [x] Outputs: destination address + change outputs to script address
  - [x] Metadata: withdraw intent + network binding
  - [x] Fee: under `max_withdrawal_fee` (2M lovelace) within `fee_tolerance_pct` (5%)
  - [x] User witnesses: Ed25519 signatures over tx body hash
- [x] Compute transaction hash (Blake2b-256 of tx body bytes only)
- [x] Build `WithdrawRequest` with: notes as `Vec<BlindSignature>`, change_outputs (blinded), tx_cbor (hex body + user witness set), tx_hash (hex)
- [x] Persist each change output blinding factor BEFORE sending request
- [x] Send `Request::Withdraw(withdraw_request)` to node
- [x] Receive `Response::Withdraw { signed_tx_cbor, tx_hash, change_notes }`
- [x] Mark consumed notes as `spent`
- [x] Unblind each change note using persisted blinding factor
- [x] Verify each unblinded change note signature
- [x] Store change notes as `available`
- [x] Delete recovered `r` rows from `blinding_factors`
- [x] Record withdrawal in activity log
- [x] On withdrawal failure after notes burned: surface hard attention banner with recovery/support guidance

### Deposit/withdraw UI

- [x] Wire `DepositDetails` screen: funding address/QR first -> on-chain deposit -> off-chain claim
- [x] Wire `WithdrawDetails` screen: destination + amount -> invoke withdraw
- [x] Implement hard attention handling for failed withdrawals
- [x] Expose `deposit` and `withdraw` mutation functions in state management

---

## Phase 4: Security (applies across both milestones)

### Blinding factor persistence protocol

- [x] Enforce ordering: generate `r` -> write to disk (fsync) -> send request -> unblind -> write Note -> delete `r`
- [x] Steps 5+6 (write Note + delete `r`) must be a single redb write transaction
- [x] On startup: scan `blinding_factors` for orphaned entries
- [x] Surface orphaned entries to user with nonce + timestamp
- [x] Consider extending `blinding_factors` schema for future automatic retry (operation type, blinded point, request context)

### Note storage encryption

- [x] Evaluate encryption approach: OS-level disk encryption, Tauri secure storage plugin, or passphrase-derived key
- [x] Implement chosen encryption for note values in redb — v1 defers to OS-level disk encryption

### Double-spend protection

- [x] Auto-refresh imported notes immediately on receive
- [x] On refresh failure: quarantine notes (exclude from balance, set `attention` status)
- [x] Provide retry/discard UI for quarantined notes
