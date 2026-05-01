# Demo — Two Wallets, One Mock Chain

Status: Draft

Goal: run the full Mugraph happy-path end-to-end on a single machine with no
real Cardano network. Two wallet instances exchange notes through one node;
all chain calls (UTxO queries, tx submission, tip, confirmations) are served
by an in-process mock chain.

## 1. Architecture

```
+-----------+     +-----------+
| wallet A  |     | wallet B  |   Tauri dev mode, two windows
+-----+-----+     +-----+-----+   each with its own data dir via
       \                /         MUGRAPH_WALLET_DATA_DIR override
        v              v
        +--------------+
        | mugraph node |  --cardano-provider blockfrost
        +------+-------+  --cardano-provider-url http://127.0.0.1:8090
               |          --cardano-api-key demo
               |          --deposit-confirm-depth 1
               v
        +--------------+
        |  mock-chain  |  :8090
        +--------------+
```

Both the node (`node/src/provider/`) and the wallet
(`wallet/src-tauri/src/provider.rs`) already abstract over Blockfrost/Maestro
and accept a custom base URL. The mock implements the Blockfrost subset both
of them call, so neither provider client changes — only the URL.

## 2. Mock chain interface

### 2.1 Blockfrost-compatible endpoints

| Endpoint                            | Caller | Purpose                                            |
| ----------------------------------- | ------ | -------------------------------------------------- |
| `GET /blocks/latest`                | both   | tip for confirmation depth + observation           |
| `GET /addresses/{addr}/utxos`       | wallet | list spendable UTxOs at script/funding address     |
| `GET /txs/{tx_hash}`                | both   | tx info → block_height (confirmation tracking)     |
| `GET /txs/{tx_hash}/utxos`          | node   | resolve a referenced deposit UTxO                  |
| `POST /tx/submit`                   | both   | accept a CBOR tx, mint outputs, consume inputs     |
| `GET /epochs/latest/parameters`     | node   | `ProtocolParams` for fee calculation               |
| `GET /scripts/datum/{hash}/cbor`    | node   | inline-datum lookup when datum is referenced by hash |

Authentication: ignore the `project_id` header. Any value is accepted.

### 2.2 Admin endpoints

Out of band, no Blockfrost analog. Used by the demo runner.

| Endpoint                   | Body                              | Purpose                                          |
| -------------------------- | --------------------------------- | ------------------------------------------------ |
| `POST /admin/faucet`       | `{address, lovelace}`             | mint a UTxO at an address                        |
| `POST /admin/mine`         | `{count}`                         | advance tip by N blocks                          |
| `POST /admin/auto_mine`    | `{on, interval_ms?}`              | toggle auto-mine on every submit / on a ticker  |
| `GET /admin/state`         | —                                 | dump tip, UTxO set, tx history (HUD)             |
| `POST /admin/reset`        | —                                 | wipe state                                       |

## 3. Mock chain internals

- New crate `mock-chain/` (workspace member). Binary + small lib. Axum HTTP
  server.
- In-memory state:
  - `tip: Block`
  - `blocks: Vec<Block>` (chronological)
  - `utxos: HashMap<(TxHash, u16), UtxoEntry>` (live)
  - `txs: HashMap<TxHash, TxRecord>`
  - `datums: HashMap<DatumHash, CborHex>`
- `submit_tx`: parse with `pallas-primitives` (already in workspace via the
  node crate). Validate inputs exist + unspent and basic value preservation.
  Skip Plutus script execution. Apply: consume inputs, append outputs (record
  inline datums against the output), record tx, include in the *current
  pending block*.
- Block model: pending block accumulates txs; auto-mined either after each
  submit or on a ticker (mode chosen at startup). Tip increments on mine.
  `tx.block_height` is set when the block is mined — exactly what
  `evaluate_tx_observation` needs.
- Persistence: in-memory by default. Optional `--snapshot path` flag dumps
  state on shutdown and reloads on startup for resumeable demos.

Default mode for the demo: **auto-mine on every successful submit**. Combined
with `--deposit-confirm-depth 1` on the node, this lets deposits land in a
single round-trip.

## 4. Prerequisites

These two changes must land before the demo can run end-to-end. Both are
small and product-justified, not just demo plumbing.

### 4.1 Wallet data-dir override

`wallet/src-tauri/src/lib.rs:15-19` hardcodes
`dirs::data_dir().join("mugraph-wallet")`. Honor a `MUGRAPH_WALLET_DATA_DIR`
env var so two wallet instances can run side-by-side on one machine.

### 4.2 Deposit command builds and submits the on-chain tx

Today `wallet/src-tauri/src/commands.rs::deposit` takes
`utxo_tx_hash`/`utxo_index` of a UTxO that is *already* at the script address
with the right datum, and only runs the off-chain claim. The product flow
described in `docs/wallet-integration.md` §2.2 Stage A says the wallet should
build the on-chain deposit tx itself. `cardano_tx::build_deposit_tx` is
implemented but unused.

Wire it:

1. Caller passes the funding UTxO ref (from the in-app Cardano wallet, not
   the script address) plus the desired denominations.
2. Wallet computes the canonical payload and `intent_hash`.
3. Wallet calls `build_deposit_tx` to produce the deposit Cardano tx.
4. Wallet attaches the user Ed25519 witness (`attach_user_witness`).
5. Wallet submits via `CardanoProvider::submit_tx`.
6. Wallet calls the node's `Request::Deposit` with the *new* `tx_hash` /
   `index = 0` (the deposit output is always index 0 of the new tx).
7. Existing claim logic runs unchanged.

Withdraw is already wired this way, so this brings deposit to parity.

## 5. Demo storyline

A `scripts/demo.sh` orchestrator drives the mock via admin endpoints. The
wallets are clicked manually for the user-facing parts.

### Act 0 — Boot

1. Start mock chain on `:8090`, fresh state, auto-mine on every submit.
2. Start one mugraph node on `:9999`:
   ```
   --cardano-provider blockfrost
   --cardano-provider-url http://127.0.0.1:8090
   --cardano-api-key demo
   --cardano-network preprod
   --deposit-confirm-depth 1
   ```
3. Start wallet A (`MUGRAPH_WALLET_DATA_DIR=/tmp/wallet-a`) and wallet B
   (`/tmp/wallet-b`), each in Tauri dev mode.

### Act 1 — Guided setup, both wallets

Each wallet's guided setup points at the node + the same mock provider
(`blockfrost`, key `demo`, base URL override `http://127.0.0.1:8090`). Each
wallet generates its own keys, derives its own funding address, fetches
`delegate_pk` and `script_address` from the node. Settings panel shows both
healthy.

### Act 2 — Fund wallet A on-chain

Demo script:

```
POST /admin/faucet  {address: <A.funding_addr>, lovelace: 100_000_000}
POST /admin/mine    {count: 2}
```

Wallet A's Cardano funding panel now shows a 100 ADA UTxO.

### Act 3 — Deposit (A: 100 ADA → notes of 50, 30, 20)

Wallet A clicks Deposit, picks the funding UTxO, enters denominations
`[50_000_000, 30_000_000, 20_000_000]`. Wallet:

1. Builds the deposit Cardano tx with inline datum
   (`user_pubkey_hash`, `node_pubkey_hash`, `intent_hash`).
2. Submits via the mock (auto-mines into next block).
3. Calls the node's `Request::Deposit`.
4. Node's `source_validation` queries the mock for the UTxO at the script
   address, sees ≥ 1 confirmation, signs the blinded outputs.
5. Wallet unblinds and persists three notes.

A's notes panel: `50, 30, 20 ADA`. Activity: deposit recorded.

### Act 4 — Off-chain send (A → B: 30 ADA, no chain involvement)

A picks the 30-ADA note, hits Send → QR. B opens Receive → Import, scans or
pastes. B's Tauri auto-refreshes through the node, which re-mints the note
onto B. B now holds a fresh 30-ADA note; A's note is marked spent. The mock
chain is unchanged — this is the "money moves with no chain trace" beat.

### Act 5 — Refresh / merge (A: 50 + 20 → 70)

A selects the two remaining notes, clicks Refresh with target `[70_000_000]`.
Pure L2 round-trip through the node; A now holds a single 70 ADA note. Pure
L2, no mock activity.

### Act 6 — Withdraw (B: 25 ADA to a fresh address, with 5 ADA change)

Demo script generates a destination address (or re-uses A's funding address
to visually close the loop). B clicks Withdraw, enters destination + 25 ADA.
Wallet:

1. Queries mock for B's spendable script UTxOs (filtered by
   `user_pubkey_hash`).
2. Builds withdraw tx (script input → destination 25 ADA + change-back-to-
   script 5 ADA).
3. Attaches user Ed25519 witness.
4. Sends `Request::Withdraw` to node.
5. Node attaches its own witness, submits via the mock.
6. Mock auto-mines, returns `tx_hash`.

B's 30-ADA note is marked spent; a new 5-ADA change note appears. The
recipient address shows `+25_000_000` lovelace on `GET /admin/state`.
Activity: withdraw with the on-chain tx_hash.

### Act 7 — Cleanup / replay

`POST /admin/reset` and restart wallets. Run the demo again.

## 6. Out of scope for v1 of the demo

- Reorg simulation (would need a `/admin/reorg` endpoint and reconciler
  exercise).
- Multipart QR sends.
- Cross-node transfers (one node only).
- Plutus script execution (we trust the wallet/node not to send invalid
  scripts; the mock only tracks UTxO state).
- Maestro endpoint surface (only Blockfrost is implemented).

## 7. Implementation order

1. `docs/specs/demo.md` (this file).
2. `mock-chain` crate.
3. Wallet data-dir override (§4.1).
4. Deposit command tx-build wiring (§4.2).
5. `scripts/demo.sh` orchestrator.

Each step is committed independently.
