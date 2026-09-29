<p align="center">
  <picture>
    <source srcset="assets/logo-white.svg" media="(prefers-color-scheme: dark)">
    <img src="assets/logo-dark.svg" alt="Mugraph Logo" width="300">
  </picture>

<p align="center"><em>Instant, untraceable payments for Cardano.</em></p>

<p align="center">
    <img src="https://github.com/mugraph-payments/mugraph/actions/workflows/build.yml/badge.svg" alt="Build Status" />
    <a href="https://opensource.org/licenses/Apache-2.0">
      <img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="Apache 2.0 Licensed" />
    </a>
    <a href="https://opensource.org/licenses/MIT">
      <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="MIT Licensed" />
    </a>
    <a href="https://discord.gg/npSJU6Qk">
      <img src="https://dcbadge.limes.pink/api/server/npSJU6Qk?style=social" alt="Mugraph Discord Server" />
    </a>
  </p>
</p>

µgraph is a payment layer for Cardano. A server, the **delegate**, holds
Cardano funds in a script address, the **vault**. In exchange, it gives users
bearer tokens, the **notes**. Users pay each other with notes, and a payment
settles in one request to the delegate, not in a block.

The delegate signs notes with **blind signatures**. Thus, the delegate can
check that a note is valid. But it can not link a note that it receives to the
user who got it. The delegate sees only the asset and the amount.

> **Status:** Version 1.0 is for the Cardano test networks (preprod and
> preview). It did not have an external security audit. Do not use it with
> real funds.

## Why µgraph

Few people use cryptocurrencies to buy goods. We think that three problems
stop them:

1. **Speed.** A card payment confirms in about two seconds. A Cardano block
   takes about 20 seconds.
2. **Privacy.** A payment on a public chain shows the payer, the payee and the
   amount to all observers.
3. **Ease of use.** Users do not want to manage many keys, channels, or
   protocols to pay for a coffee.

µgraph addresses these problems with Chaumian e-cash on top of Cardano. The
whitepaper explains the design and its costs.

## How it works

1. **Deposit.** A user sends ADA to the vault. The delegate signs notes of the
   same value.
2. **Pay.** The user gives notes to another user, for example as a text
   message.
3. **Refresh.** The receiver swaps the notes for new notes at the delegate.
   Now only the receiver knows them.
4. **Withdraw.** A user burns notes, and the delegate signs a Cardano
   transaction that pays the user from the vault.

Each note has one key for its asset and its amount. Thus, a user can not get a
signature for more value than they pay. The vault validator lets only the
delegate spend the vault.

## Documentation

| Document                                        | Content                                                   |
| ----------------------------------------------- | --------------------------------------------------------- |
| [Protocol specification](docs/spec/protocol.md) | The exact rules of the protocol, version 1.0              |
| [Whitepaper](docs/whitepaper.md)                | The design, the reasons for it, and the security analysis |
| [Implementation guide](docs/implementation.md)  | The code, the build, the tests, and how to run a node     |
| [User guide](docs/user-guide.md)                | How to use the wallet                                     |
| [Cryptography](docs/crypto.md)                  | The blind signatures and the proofs, step by step         |
| [API reference](docs/openapi.yaml)              | The RPC methods of a node (OpenAPI)                       |
| [Cross-node specifications](docs/specs/)        | The extension for payments between delegates              |

## Quick start

Enter the development shell (Nix):

```sh
nix develop
```

Run all tests:

```sh
cargo test --workspace
(cd validator && aiken check)
```

Run a local demonstration with a mock chain, a node, and two wallets:

```sh
scripts/demo.sh up
```

Run a node on preprod with Docker:

```sh
docker build -t mugraph-node .
docker run -p 9999:9999 -e CARDANO_API_KEY=<Blockfrost key> \
  mugraph-node server --addr 0.0.0.0:9999 --secret-key <key>
```

Make the key with `mugraph-node generate-key`. The implementation guide gives
all the steps and the settings.

## Repository

| Directory     | Content                               |
| ------------- | ------------------------------------- |
| `core/`       | The shared types and the cryptography |
| `node/`       | The delegate node                     |
| `wallet/`     | The wallet app (Tauri and React)      |
| `validator/`  | The vault validator (Aiken)           |
| `mock-chain/` | A local Cardano chain for tests       |
| `simulator/`  | A load simulator                      |

## License

### Software

µgraph and all projects under `mugraph-payments` have two licenses: the
[MIT](./LICENSE) license and the [Apache 2.0](./LICENSE-APACHE) license. You
can use the software under either license. If you need an exception, contact
us.

### Logo

The project logo uses the [Berkeley Mono Typeface](https://berkeleygraphics.com/),
under a
[Developer License](https://cdn.berkeleygraphics.com/static/legal/licenses/developer-license.pdf).

All graphics that we make have the
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/?ref=chooser-v1)
license. It only requires attribution. If this license is a problem for your
use case, contact us.
