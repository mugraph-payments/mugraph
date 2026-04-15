# Wallet Integration Checklist

Exhaustive task checklist derived from [wallet-integration.md](./wallet-integration.md).

---

## Milestone A: Off-chain Wallet (connect + refresh + send)

### Phase 1: Tauri Backend — Node Client and Local Storage

#### 1.1 Add `mugraph-core` dependency to wallet crate

- [ ] Add `mugraph-core = { workspace = true }` to `wallet/src-tauri/Cargo.toml`
- [ ] Add `ed25519-dalek = { version = "2.1", features = ["rand_core"] }`
- [ ] Add `reqwest = { version = "0.12", features = ["json"] }`
- [ ] Add `redb = { workspace = true }`
- [ ] Add `rand = { workspace = true }`
- [ ] Add `serde = { version = "1", features = ["derive"] }`
- [ ] Add `serde_json = { workspace = true }`
- [ ] Add `tokio = { workspace = true }`

#### 1.2 Implement wallet-side node client (`wallet/src-tauri/src/node_client.rs`)

- [ ] Create `NodeClient` struct with `reqwest::Client`, `rpc_url`, `health_url`
- [ ] Implement `NodeClient::new(base: &Url)` constructor
- [ ] Implement `health()` method (`GET /health`)
- [ ] Implement `info()` method (`Request::Info` -> returns `PublicKey`, optional script address)
- [ ] Implement `refresh()` method (`Request::Refresh` -> returns `Vec<BlindSignature>`)
- [ ] Wire serialization using tagged union format `{"m": "...", "p": {...}}`
- [ ] Wire deserialization using `{"m": "...", "r": {...}}` response format
- [ ] Pattern-match `Response::Error` into proper error propagation

#### 1.3 Local note storage (`wallet/src-tauri/src/store.rs`)

- [ ] Create redb database initialization
- [ ] Create `config_global` table (wallet label, last network)
- [ ] Create `provider_config` table (type, api_key, base_url_override)
- [ ] Create `node_config` table (keyed by network -> node URL)
- [ ] Create `keypair` table (secret_key bytes, ed25519_sk bytes)
- [ ] Create `cardano_keypair` table (payment_sk, payment_vk)
- [ ] Create `delegate_info` table (`<network>:pk`, `<network>:script_addr`)
- [ ] Create `notes` table (`<network>:<nonce>` -> serialized Note + status + created_at)
- [ ] Create `activity` table (`<network>:<id>` -> serialized activity record)
- [ ] Create `blinding_factors` table (`<network>:<nonce>` -> Scalar bytes)
- [ ] Create `offchain_requests` table (id -> serialized receive request metadata)
- [ ] Create `cardano_utxos` table (`<network>:<tx_hash>#<index>` -> UTxO metadata)
- [ ] Implement crash-recovery scan for orphaned blinding factors on startup
- [ ] Surface orphaned factors to user with nonce + timestamp

#### 1.4 Expose Tauri commands (`wallet/src-tauri/src/commands.rs` + `lib.rs`)

- [ ] Define `AppState` struct (redb Database, Mugraph Keypair, Ed25519 signing key, Cardano payment keypair, provider config, per-network NodeClients)
- [ ] Implement `complete_guided_setup` command
- [ ] Implement `get_wallet_state` command
- [ ] Implement `switch_network` command
- [ ] Implement `create_receive_request` command
- [ ] Implement `import_notes` command
- [ ] Implement `send` command
- [ ] Implement `refresh_notes` command
- [ ] Implement `sync` command
- [ ] Remove placeholder `greet` command from `lib.rs`
- [ ] Wire up AppState and all commands in `lib.rs`

### Phase 2: Core Wallet Operations in Rust

#### 2.1 Connect / Bootstrap (guided setup)

- [ ] Implement guided setup flow collecting config for all 3 networks (mainnet, preprod, preview)
- [ ] Collect one node URL per network
- [ ] Collect one provider type (blockfrost or maestro)
- [ ] Collect one provider credential set (reused across networks)
- [ ] On first launch: generate `Keypair::random()` for BDHKE operations and persist
- [ ] On first launch: generate `ed25519_dalek::SigningKey` for CIP-8/witness auth and persist
- [ ] On first launch: generate one Cardano payment keypair and persist
- [ ] For each network: call `Request::Info` on that network's node
- [ ] Store `delegate_pk` + `cardano_script_address` per network namespace
- [ ] Mark setup complete only after all 3 networks pass bootstrap
- [ ] On subsequent launches: open last-used network
- [ ] Handle broken network config at startup: warn but allow healthy networks

#### 2.5 Refresh (split, merge, re-validate)

- [ ] Build `Refresh` using `RefreshBuilder` (`.input()` / `.output()` / `.build()`)
- [ ] For each output atom: compute commitment via `atom.commitment(&refresh.asset_ids)`
- [ ] For each output atom: blind commitment via `crypto::blind(&mut rng, commitment.as_ref())`
- [ ] Convert blinded points to `Signature` and attach to `refresh.blinded_points`
- [ ] Persist each blinding factor to `blinding_factors` table BEFORE sending request
- [ ] Send `Request::Refresh(refresh)` to node
- [ ] Receive `Response::Transaction { outputs }`
- [ ] For each output: recover blinded point for DLEQ verification
- [ ] For each output: verify DLEQ proof via `crypto::verify_dleq_signature()`
- [ ] For each output: unblind signature via `crypto::unblind_signature()`
- [ ] For each output: verify final signature via `crypto::verify()`
- [ ] For each output: construct full `Note` with unblinded signature + `DleqProofWithBlinding`
- [ ] Store new notes with status `available`
- [ ] Delete recovered `r` rows from `blinding_factors`
- [ ] Mark input notes as `spent`

#### 2.4 Send (off-chain, user to user)

- [ ] Implement coin selection (largest-first deterministic)
- [ ] If exact denominations unavailable: trigger refresh first to split/merge
- [ ] Serialize selected Notes into v1 JSON envelope (network, delegate_pk, sender_label, created_at, notes array with hex-encoded fields)
- [ ] Support copy/paste text transport
- [ ] Support QR transport (when payload fits single-code limit; otherwise require text)
- [ ] Mark sent notes as `spent` locally
- [ ] Implement import: validate envelope network + delegate match active wallet
- [ ] Implement import: verify each note signature via `crypto::verify(&delegate_pk, commitment, signature)`
- [ ] Implement auto-refresh of imported notes immediately after import
- [ ] If auto-refresh fails: keep notes with quarantined/untrusted status
- [ ] Exclude quarantined notes from spendable balance
- [ ] Set wallet status to `attention` when quarantined notes exist
- [ ] Provide retry/discard path for quarantined notes

#### 2.6 Sync

- [ ] Implement periodic `Request::Info` to verify node reachability
- [ ] Detect if delegate key has changed
- [ ] Check pending deposit status
- [ ] Check pending withdrawal on-chain confirmation
- [ ] Update `lastSyncedAt`

### Phase 3: Frontend Integration

#### 3.1 Replace stub data with Tauri invoke calls

- [ ] Replace static imports from `stubWallet.ts` with `invoke()` calls via `@tauri-apps/api/core`
- [ ] Create `wallet/src/lib/api.ts` — TypeScript invoke wrappers for all commands

#### 3.2 State management (`wallet/src/lib/walletStore.ts`)

- [ ] Require guided setup completion before entering main wallet shell
- [ ] Restore last-used network on launch
- [ ] Call `get_wallet_state(activeNetwork)` on mount and after every mutation
- [ ] Provide active-network `WalletState` to all components via context
- [ ] Expose mutation functions: `createReceiveRequest`, `importNotes`, `send`, `refreshNotes`
- [ ] Surface startup warnings for broken network configs without blocking healthy networks
- [ ] Hardcode known test asset metadata (ADA/lovelace, USDM) for Milestone A
- [ ] Handle missing price data gracefully (zero/omit `totalValueUsd`, `shareOfWalletPct`)
- [ ] Default `AssetHolding.trend` to `"flat"` for Milestone A

#### 3.3 Wire up action screens

- [ ] `SendDetails` — invoke `send`, emit text/QR off-chain envelope
- [ ] `ReceiveDetails` — create strict off-chain receive requests only (no L1 semantics)
- [ ] `NotesPanel` — live notes from local store including quarantine states
- [ ] `ActivityPanel` — live activity from local store
- [ ] `AssetPanel` — computed from live note aggregation

#### 3.4 Error handling

- [ ] Map node unreachable/timeout -> `attention` status + "Node offline" banner with retry
- [ ] Map BDHKE verification failure -> `attention` status + "Signature mismatch" banner
- [ ] Map unbalanced refresh -> `ready` status + inline form error
- [ ] Map blinding factor persistence errors -> `attention` + block operation until resolved
- [ ] Map orphaned blinding factors -> `attention` + startup recovery prompt

#### 3.5 Settings screen

- [ ] Wire delegate PK and script address to real values from bootstrap
- [ ] Add node URL inputs for all 3 networks
- [ ] Add shared provider config block (provider type, API key, optional base URL override)
- [ ] Add network selector (mainnet/preprod/preview)
- [ ] Add manual sync trigger
- [ ] Add startup warning surface for broken network configs

#### Off-chain receive request payload

- [ ] Define strict receive request JSON payload (network, delegate_pk, recipient_label, asset, amount, label)
- [ ] `import_notes` / send flow must reject envelopes not matching active strict request

---

## Milestone B: Cardano L1 Integration (deposit + withdraw)

### Additional dependencies

- [ ] Add `whisky-csl` (Cardano tx building) to `wallet/src-tauri/Cargo.toml`
- [ ] Add `coset` (COSE_Sign1) dependency
- [ ] Add `blake2` (intent hash) dependency
- [ ] Add `hex` dependency

### Node client extensions

- [ ] Add `deposit()` method to `NodeClient`
- [ ] Add `withdraw()` method to `NodeClient`

### CIP-8 signature construction

- [ ] Implement COSE_Sign1 envelope builder
- [ ] Set `alg: EdDSA` in protected header
- [ ] Embed canonical payload bytes from `build_canonical_payload`
- [ ] Sign `tbs_data` with Ed25519 key
- [ ] Serialize with `to_tagged_vec()`

### 2.2 Deposit — Stage A: On-chain deposit transaction

- [ ] Show in-app Cardano funding address + QR so user can fund wallet externally
- [ ] Select source UTxOs from in-app Cardano wallet (largest-first strategy)
- [ ] Decide output denominations and blinding ahead of time
- [ ] Compute canonical payload and its Blake2b-256 hash (`intent_hash`)
- [ ] Build Cardano transaction sending funds to `script_address` with inline Plutus datum:
  - [ ] `user_pubkey_hash` (Blake2b-224 of Ed25519 verifying key)
  - [ ] `node_pubkey_hash` (Blake2b-224 of node's payment_vk)
  - [ ] `intent_hash` (Blake2b-256 of canonical JSON payload)
- [ ] Submit transaction on-chain

### 2.2 Deposit — Stage B: Off-chain deposit claim

- [ ] For each output note: generate random nonce
- [ ] For each output note: compute commitment via `Note::commitment()`
- [ ] For each output note: blind commitment via `crypto::blind()`
- [ ] Persist `r` to `blinding_factors` table immediately
- [ ] Pack blinded points into `BlindSignature` with default `DleqProof`
- [ ] Build `DepositRequest` with: utxo ref, outputs, message (user_pubkey JSON), CIP-8 signature, nonce, network
- [ ] Send `Request::Deposit(deposit_request)` to node
- [ ] Receive `Response::Deposit { signatures, deposit_ref }`
- [ ] For each response signature: verify DLEQ proof
- [ ] For each response signature: unblind via `crypto::unblind_signature()`
- [ ] For each response signature: verify unblinded signature (check bool, not just `?`)
- [ ] Construct full `Note` objects with unblinded signatures
- [ ] Store notes with status `available`
- [ ] Store blinding factor in `DleqProofWithBlinding.blinding_factor`
- [ ] Delete `r` rows from `blinding_factors` table
- [ ] Record deposit in activity log

### 2.3 Withdraw (Mugraph L2 to Cardano L1)

- [ ] Accept destination Cardano address + amount from user
- [ ] Select notes covering the amount (coin selection, largest-first)
- [ ] Query spendable script UTxOs from Cardano provider at node's script address
- [ ] Filter UTxOs by datum `user_pubkey_hash`
- [ ] Build Cardano transaction:
  - [ ] Inputs: script UTxOs with matching deposit datums
  - [ ] Outputs: destination address + change outputs to script address
  - [ ] Metadata: withdraw intent + network binding
  - [ ] Fee: under `max_withdrawal_fee` (2M lovelace) within `fee_tolerance_pct` (5%)
  - [ ] User witnesses: Ed25519 signatures over tx body hash
- [ ] Compute transaction hash (Blake2b-256 of tx body bytes only)
- [ ] Build `WithdrawRequest` with: notes as `Vec<BlindSignature>`, change_outputs (blinded), tx_cbor (hex), tx_hash (hex)
- [ ] Persist each change output blinding factor BEFORE sending request
- [ ] Send `Request::Withdraw(withdraw_request)` to node
- [ ] Receive `Response::Withdraw { signed_tx_cbor, tx_hash, change_notes }`
- [ ] Mark consumed notes as `spent`
- [ ] Unblind each change note using persisted blinding factor
- [ ] Verify each unblinded change note signature
- [ ] Store change notes as `available`
- [ ] Delete recovered `r` rows from `blinding_factors`
- [ ] Record withdrawal in activity log
- [ ] On withdrawal failure after notes burned: surface hard attention banner with recovery/support guidance

### Deposit/withdraw UI

- [ ] Wire `DepositDetails` screen: funding address/QR first -> on-chain deposit -> off-chain claim
- [ ] Wire `WithdrawDetails` screen: destination + amount -> invoke withdraw
- [ ] Implement hard attention handling for failed withdrawals
- [ ] Expose `deposit` and `withdraw` mutation functions in state management

---

## Phase 4: Security (applies across both milestones)

### Blinding factor persistence protocol

- [ ] Enforce ordering: generate `r` -> write to disk (fsync) -> send request -> unblind -> write Note -> delete `r`
- [ ] Steps 5+6 (write Note + delete `r`) must be a single redb write transaction
- [ ] On startup: scan `blinding_factors` for orphaned entries
- [ ] Surface orphaned entries to user with nonce + timestamp
- [ ] Consider extending `blinding_factors` schema for future automatic retry (operation type, blinded point, request context)

### Note storage encryption

- [ ] Evaluate encryption approach: OS-level disk encryption, Tauri secure storage plugin, or passphrase-derived key
- [ ] Implement chosen encryption for note values in redb

### Double-spend protection

- [ ] Auto-refresh imported notes immediately on receive
- [ ] On refresh failure: quarantine notes (exclude from balance, set `attention` status)
- [ ] Provide retry/discard UI for quarantined notes
