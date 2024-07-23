<p align="center">
  <picture>
    <source srcset="docs/assets/logo-white.svg" media="(prefers-color-scheme: dark)">
    <img src="docs/assets/logo-dark.svg" alt="Mugraph Logo" width="300">
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
  </p>
</p>
<hr />

Mugraph (pronounced *"mew-graph"*) is a Layer 2 Network for the Cardano blockchain for untraceable payments with instant finality. It is very simplified in both operations and architecture, meant to be easy to integrate everywhere.

Most Layer 2 networks are blockchains themselves, with unified liquidity pools and decentralized sequencers commiting deltas of state to the Layer 1 Blockchain. That is not the case with Mugraph.

Instead, consider Mugraph as a **Payment Network**, composed of **Delegates**.

A Delegate is a node, usually controlled by a single entity, that holds user balance in a escrow contract, called the **Vault**. Conceptually, they behave similarly to what Paypal, Venmo, CashApp and others do in the tradicional finance ecosystem.

They guarantee liquidity and route user payments just like those traditional systems, but with some critical differences:

- Delegates hold user funds in **Shared Custody**, and they can not spend it themselves.
- Delegates are **blind** and don't know who is transacting inside the system.
- Delegates provide **group anonimity** for their users, signing transactions on behalf of them.

# Table of Contents

## The Basics

1. [Motivation](./docs/motivation.md)
1. [Roadmap](./docs/roadmap.md)
1. [Licensing](./docs/licensing.md)

## The Protocol

1. [Delegates](./protocol/delegates.md)
1. [Wallets](./protocol/wallets.md)

## The Future

1. [Cross-node Transfers](./future/cross-node-transfers.md)
