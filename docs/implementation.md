# µgraph Implementation Guide

This guide is for developers and for the operators of a delegate node. It
describes the code, the build, the tests, and the operation of a node. The
protocol specification (`docs/spec/protocol.md`) gives the rules that the code
implements.

## 1. Repository layout

| Directory     | Crate or tool       | Purpose                                                                                                 |
| ------------- | ------------------- | ------------------------------------------------------------------------------------------------------- |
| `core/`       | `mugraph-core`      | Types, cryptography, keysets, the refresh builder, and the RPC types. The node and the wallet share it. |
| `node/`       | `mugraph-node`      | The delegate: the HTTP server, the operations, the Cardano provider clients, and the database.          |
| `wallet/`     | `mugraph-wallet`    | The wallet app: a Tauri 2 back end in Rust (`wallet/src-tauri`) and a React front end (`wallet/src`).   |
| `validator/`  | Aiken               | The vault validator and its blueprint `plutus.json`.                                                    |
| `mock-chain/` | `mock-chain`        | A local Cardano chain with a Blockfrost-compatible API, for tests and demonstrations.                   |
| `simulator/`  | `mugraph-simulator` | A terminal program that sends random payments to one or more nodes.                                     |
| `scripts/`    | Bash                | The demonstration and cluster scripts.                                                                  |
| `nix/`        | Nix                 | The packages and the development shell.                                                                 |
| `docs/`       | Markdown            | The specification, the whitepaper, and the guides.                                                      |

## 2. Architecture

### 2.1 Core

The `core` crate has no network or storage code. Its main modules are:

- `crypto`: hash to the group, blind signatures, DLEQ proofs, and the two verification functions.
- `keyset`: the denomination keys, the keysets, and the split of an amount into denominations.
- `types`: notes, refreshes, blinded outputs, requests, responses, and Cardano types.
- `builder`: `RefreshBuilder`, which makes a balanced refresh from notes and output amounts.

### 2.2 Node

The node is an Axum server with two routes, `GET /health` and `POST /rpc`. The
file `node/src/routes/mod.rs` sends each RPC method to its handler:

| Method                       | Handler                                                     |
| ---------------------------- | ----------------------------------------------------------- |
| `refresh`                    | `node/src/routes/refresh.rs`                                |
| `deposit`                    | `node/src/routes/deposit.rs` and `node/src/routes/deposit/` |
| `withdraw`                   | `node/src/routes/withdraw/`                                 |
| `keys`, `public_key`, `emit` | `node/src/routes/mod.rs`                                    |
| `cross_node_transfer_*`      | `node/src/routes/cross_node/`                               |

The node keeps its state in one redb database file. The main tables are:

| Table            | Content                                                             |
| ---------------- | ------------------------------------------------------------------- |
| `notes`          | The spent set: the signature of each spent note                     |
| `deposits`       | Each accepted deposit, by UTxO reference                            |
| `withdrawals`    | Each withdrawal, by transaction hash, with its state                |
| `cardano_wallet` | The node's Cardano payment key, the validator and the vault address |
| `cross_node_*`   | The state of cross-node transfers                                   |

Two tasks run in the background when the node is not in dev mode. The deposit
monitor marks old deposits. The reconciler keeps the state of cross-node
transfers.

The node talks to Cardano through a provider: Blockfrost or Maestro. The node
has the vault validator in its binary (`validator/plutus.json`), so it needs no
Aiken tools at run time.

### 2.3 Wallet

The wallet back end (`wallet/src-tauri/src/`) has these modules:

| Module           | Purpose                                                                          |
| ---------------- | -------------------------------------------------------------------------------- |
| `commands.rs`    | The Tauri commands: setup, deposit, send, import, refresh, withdraw, and sync    |
| `notes.rs`       | The refresh, blind and unblind helpers, and the checks of received notes         |
| `cardano_tx.rs`  | The deposit and withdrawal transactions                                          |
| `provider.rs`    | The Blockfrost and Maestro clients                                               |
| `node_client.rs` | The RPC client for the node                                                      |
| `store.rs`       | The local redb database: notes, blinding factors, pending deposits, and activity |

The wallet saves each blinding factor before it sends a request. If the
request fails after the node signed, the wallet can use the saved factors to
recover the notes. The wallet also saves each deposit claim, and it tries the
claim again at each sync, until the node accepts it.

## 3. Build

### 3.1 Development shell

The project uses Nix. The development shell has the Rust toolchain, Aiken,
Bun, and the Android tools.

```sh
nix develop
```

Without Nix, install a nightly Rust toolchain (see `rust-toolchain.toml`),
Aiken, and Bun.

### 3.2 Node and tools

```sh
cargo build --release -p mugraph-node
cargo build --release -p mock-chain -p mugraph-simulator
```

Nix can also build the node and the simulator:

```sh
nix build .#mugraph-node
nix build .#mugraph-simulator
```

### 3.3 Docker image

```sh
docker build -t mugraph-node .
```

The image runs `mugraph-node server --addr 0.0.0.0:9999` as the user
`mugraph`.

### 3.4 Validator

The repository has the compiled validator in `validator/plutus.json`. To
change the validator, do these steps:

1. Change the Aiken code in `validator/`.
2. Run `aiken check` and `aiken build` in `validator/`.
3. Build the node again.

A new validator has a new hash and a new vault address.

### 3.5 Wallet

For the desktop app in development:

```sh
cd wallet
bun install
cargo tauri dev
```

For an Android package (aarch64):

```sh
cd wallet
cargo tauri android build --target aarch64 --apk
```

The GitHub workflow `release.yml` builds an unsigned Android package and an
unsigned iOS archive when you start it by hand.

## 4. Tests

### 4.1 All tests

```sh
cargo test --workspace
(cd validator && aiken check)
```

### 4.2 Important tests

| Test                                               | What it shows                                                                                                                                                     |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `wallet/src-tauri/tests/protocol_e2e.rs`           | A deposit, a transfer, a second import of the same notes, and a withdrawal, on the mock chain with a real node. After each step, the notes equal the vault value. |
| `wallet/src-tauri/tests/withdraw_tx_evaluation.rs` | The Plutus evaluator accepts the wallet's withdrawal transaction with the real validator.                                                                         |
| `node/tests/validator_evaluation.rs`               | The validator accepts spends that the node signs, and refuses spends that only the depositor signs.                                                               |
| `node/tests/refresh_route_tests.rs`                | The refresh rules: blinded outputs, hidden nonces, denomination keys, and atomic spends.                                                                          |
| `node/src/routes/withdraw/mod.rs`                  | The withdrawal rules: note checks, the value rule, signers, and vault outputs.                                                                                    |
| `core/src/keyset.rs`, `core/src/crypto.rs`         | Property tests for the keys, the blind signatures, and the proofs.                                                                                                |

The mock chain checks witnesses, required signers, redeemers, balance and
fees, and it runs the validator. Thus, the end-to-end test also checks that
the transactions obey these ledger rules.

## 5. Operate a node

### 5.1 Before you start

You need these items:

- A Blockfrost project key for the network (for example, preprod).
- A master key for the node.
- A disk for the node database.

Make the master key:

```sh
mugraph-node generate-key
```

The command prints `secret_key=<hex>` and `public_key=<hex>`.

> **Warning:** Keep the secret key in a safe place, and do not show it to
> other persons. A person with the secret key can make notes and empty the
> vault.

> **Caution:** Start the node with the same secret key each time. A node with
> a new key refuses all notes that it issued before.

### 5.2 Start the node

```sh
export CARDANO_NETWORK=preprod
export CARDANO_PROVIDER=blockfrost
export CARDANO_API_KEY=<your Blockfrost key>
export MUGRAPH_DB_PATH=/var/lib/mugraph/db.redb
mugraph-node server --addr 0.0.0.0:9999 --secret-key <secret key>
```

With Docker:

```sh
docker run -d --name mugraph -p 9999:9999 \
  -e CARDANO_NETWORK=preprod \
  -e CARDANO_API_KEY=<your Blockfrost key> \
  -e MUGRAPH_DB_PATH=/app/data/db.redb \
  -v mugraph-data:/app/data \
  mugraph-node server --addr 0.0.0.0:9999 --secret-key <secret key>
```

At the first start, the node makes a Cardano payment key and keeps it in the
database. To use your own payment key, set `CARDANO_PAYMENT_SK`. The log shows
the vault address.

Make sure that the node works:

```sh
curl http://localhost:9999/health
curl -X POST http://localhost:9999/rpc -H 'content-type: application/json' \
  -d '{"m":"public_key"}'
```

### 5.3 Settings

| Flag                          | Variable                    | Default                          | Meaning                                                       |
| ----------------------------- | --------------------------- | -------------------------------- | ------------------------------------------------------------- |
| `--addr`                      |                             | `0.0.0.0:9999`                   | The address for HTTP                                          |
| `--secret-key`                |                             | none                             | The master key, in hex                                        |
| `--seed`                      |                             | none                             | A number that makes the key. Only for tests.                  |
| `--cardano-network`           | `CARDANO_NETWORK`           | `preprod`                        | `mainnet`, `preprod`, `preview` or `testnet`                  |
| `--cardano-provider`          | `CARDANO_PROVIDER`          | `blockfrost`                     | `blockfrost` or `maestro`                                     |
| `--cardano-api-key`           | `CARDANO_API_KEY`           | none                             | The provider key                                              |
| `--cardano-provider-url`      | `CARDANO_PROVIDER_URL`      | none                             | A different provider URL, for example the mock chain          |
| `--cardano-payment-sk`        | `CARDANO_PAYMENT_SK`        | none                             | A payment key to import, in hex                               |
| `--deposit-confirm-depth`     | `DEPOSIT_CONFIRM_DEPTH`     | 15                               | The confirmations that a deposit needs                        |
| `--deposit-expiration-blocks` | `DEPOSIT_EXPIRATION_BLOCKS` | 1440                             | The age in blocks at which the monitor marks a deposit record |
| `--min-deposit-value`         | `MIN_DEPOSIT_VALUE`         | 1,000,000                        | The smallest deposit, in lovelace                             |
| `--max-tx-size`               | `MAX_TX_SIZE`               | 16384                            | The largest withdrawal transaction, in bytes                  |
| `--max-withdrawal-fee`        | `MAX_WITHDRAWAL_FEE`        | 2,000,000                        | The largest withdrawal fee, in lovelace                       |
| `--fee-tolerance-pct`         | `FEE_TOLERANCE_PCT`         | 5                                | The fee margin above the largest fee, in percent              |
| `--xnode-peer-registry-file`  | `XNODE_PEER_REGISTRY_FILE`  | none                             | The trusted peers for cross-node messages                     |
| `--xnode-node-id`             | `XNODE_NODE_ID`             | `node://local`                   | The node ID for cross-node messages                           |
| `--dev-mode`                  | `DEV_MODE`                  | off                              | Test mode: no Cardano, and `emit` works                       |
|                               | `MUGRAPH_DB_PATH`           | `~/.local/share/mugraph/db.redb` | The database file                                             |
|                               | `RUST_LOG`                  | none                             | The log level, for example `info`                             |

> **Warning:** Do not use `--dev-mode` on a public network. In dev mode, any
> person can make notes with `emit`.

### 5.4 Network access

The node serves plain HTTP. Put it behind a reverse proxy with TLS. The
`refresh` method has no rate limit, so the proxy should limit the requests
from each client.

### 5.5 Backups

The database has the spent set. If you lose it, users can spend their notes a
second time. Make regular backups of the database file. Keep the secret key
and the payment key in a different place from the backups.

### 5.6 Upgrade from version 0

Version 1.0 changes the signatures, the validator, and the vault address.
Notes from version 0 are not valid. To upgrade, do these steps:

1. Stop the version 0 node.
2. Start version 1.0 with a new database file.
3. Tell the users to configure their wallets again.

Funds at the version 0 vault address stay under the version 0 validator.

## 6. Demonstration

The script `scripts/demo.sh` runs a mock chain and a node on your computer,
with two wallets.

1. Start the chain and the node with `scripts/demo.sh up`.
2. Open wallet A with the command that `scripts/demo.sh wallet a` prints.
3. Open wallet B with the command that `scripts/demo.sh wallet b` prints.
4. In each wallet, enter the node URL `http://127.0.0.1:9999` for each network.
5. In each wallet, select Blockfrost, enter the key `demo`, and enter the URL `http://127.0.0.1:8090`.
6. In wallet A, open Settings, then Deposit, and copy the funding address.
7. Send test funds with `scripts/demo.sh faucet <address> 200000000`.
8. Make a deposit in wallet A.
9. Mine one more block with `scripts/demo.sh mine 1`.
10. Wait for the next sync of wallet A. The wallet then gets its notes.
11. Send notes from wallet A, and import them in wallet B.
12. Make a withdrawal in wallet A.

The mock chain makes a block for each transaction. The demo node needs one
confirmation, so step 9 is necessary before the node accepts the deposit.

Other commands of the script are `state`, `utxos`, `logs`, `reset` and
`down`.

## 7. Simulator

The simulator sends random payments to nodes in dev mode:

```sh
scripts/dev-cluster.sh
```

The script starts two nodes and the simulator in a tmux session. The
simulator shows the payments, the errors, and a check that the value of each
asset does not change.
