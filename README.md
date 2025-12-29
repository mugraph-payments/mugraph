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

Mugraph (pronounced *"mew-graph"*) is a Layer 2 Network for the Cardano
blockchain for untraceable payments with instant finality.

By **untraceable**, we mean that, inside a given group $A$ of users:

- All transactions between users inside $A$ are untraceable, meaning senders
and receivers are not bound in any way.
- All transactions to users outside $A$ come from a single, shared identity for
all group participants.

This shared identity comes from **Delegates**, similar to Payment Networks in
the traditional banking system, like Paypal, Venmo or CashApp, but with some
crucial distinctions:

- Delegates hold funds in a fully auditable Smart Contract Vault, held in the
  Layer 1.
- Delegates can not spend user funds without their authorization.
- Delegates are **blind**, meaning they don't know who is transacting.
- Delegates provide **group concealing** for their users, signing transactions
  on behalf of them.

An user can, and usually will, hold balance in multiple Delegates at once, and
they do not need to have balance in a Delegate to receive payments there.

## Motivation

> Why are people not using cryptocurrencies for payments?

This is the question I wanted to address when Mugraph started. Bitcoin was
created to address this specific problem, yet [The Lightning
Network](https://lightning.network) is not widely supported even in countries
that adopted Bitcoin as legal tender.

Even if Cardano has much more advanced technology than Bitcoin does (like eUTXO
or Smart Contracts), buying groceries with [USDM](https://mehen.io) remains
nighly impossible.

ZeroHedge explains this phenomenon in their article ["What Happened to
Bitcoin?"](https://www.zerohedge.com/crypto/what-happened-bitcoin):

> At the same time, new technologies were becoming available that vastly
> improved the efficiency and availability of exchange in fiat dollars. They
> included Venmo, Zelle, CashApp, FB payments, and many others besides, in
> addition to smartphone attachments and iPads that enabled any merchant of any
> size to process credit cards. These technologies were completely different
> from Bitcoin because they were permission-based and mediated by financial
> companies. But to users, they seemed great and their presence in the
> marketplace crowded out the use case of Bitcoin at the very time that my
> beloved technology had become an unrecognizable version of itself.

Excluding volatility (already being taken seriously by Stablecoins), we
identified three main problems that prevent the average person from considering
cryptocurrencies as a payment option:

1. **Scalability**: Cryptocurrency transactions are slow compared to
   centralized solutions. For example, credit cards have a practical
confirmation limit of 2 seconds.

1. **Privacy**: Users do not want to reveal their identity for every purchase.
   Financial privacy is a human right, and in the age of big data analysis and
AI, pseudonymity does not provide sufficient privacy.

1. **Ease of Use**: Users prefer not to deal with complex protocols, multiple
   wallets, or extensive security considerations.

I think that Mugraph has a real shot of solving those problems, and that's why
I'm building it.

## Running

### With Docker

TODO.

## Developing

All of the core development team uses [Nix](https://nixos.org) to set up the development environment, so changes in the environment setup are more likely to appear there first. With that being said, Mugraph uses stable Rust, which you can install with [rustup](https://rustup.rs):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then, you can build the application using `cargo build`, as expected.

### With Nix

First, you will need [Nix](https://nixos.org) installed, which you can do with the [Determinate Systems Nix Installer](https://github.com/DeterminateSystems/nix-installer), like so:

```sh
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Then, you can run this command to spawn a development shell:

```sh
nix develop
```

You can also install [direnv](https://direnv.net/) do do this automatically when you `cd` to the folder. You can now build the application using Cargo:

```sh
cargo build
```

## Licensing

### Software

Mugraph, as well as all projects under the `mugraph-payments` is dual-licensed under the [MIT](./LICENSE) and [Apache 2.0](./LICENSE-APACHE) licenses.

This should cover most possible uses for this software, but if you need an exception for any reason, please do get in touch.

### Logo

The project logo uses the [Berkeley Mono Typeface](https://berkeleygraphics.com/), under a [Developer License](https://cdn.berkeleygraphics.com/static/legal/licenses/developer-license.pdf).

All graphics we create are also licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/?ref=chooser-v1). It only requires attribution, but if this license is a problem for your use-case, get in touch.
