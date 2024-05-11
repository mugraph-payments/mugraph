# µgraph: Fast, Untraceable Synthetic Payments in Cardano

µgraph is a payment network, very similar in spirit to [Hydra](https://hydra.family) and [The Lightning Network](https://lightning.network/).

It is not meant to be an extension to a blockchain, or to be isomorphic to their internals. Instead, it is a fast, secure and anonymous payment platform for **synthetic assets**, a kind of blockchain Token that mirrors the price action of real-world assets and financial instruments (like currencies, stocks, ETFs, CDPs).

Users join the network by sending ADA to a Vault, co-owned between the protocol itself and a µ (Mu), a [Chaumian Mint](https://fedimint.org/docs/FAQs/WhatIsChaumianMint) that validates and signs transactions. A Mu knows nothing about what's being transacted, the amounts, the senders or the receivers.

In exchange for safeguarding funds and signing transactions, they control the staking rights for the ADA in their vault, funding their operations.

After joining the network, you can:

- send and receive money completely peer-to-peer
- send and receive payments via NFC (same workflow as Google Pay and Apple Pay)
- send and receive payments with 50$ POS Terminals
- hold value with ANY asset
- keep the custody of your funds
- safeguard your financial privacy

Our goal is to create a platform where non-technical users can use for real-world payments in a very similar way as current legacy payment systems. We believe, generally, that improving the user experience and a general market for synthetic assets are the next steps into the evolution of self-sovereign money. And, we want to do it in a way that safeguards the basic human of rights: the right for financial privacy and self-sovereignty. Most people should not even know what holds their value, as long as it verifiably does.

## Introduction

In a nutshell, µgraph is a network where users deposit ADA into a Vault (a Smart Contract, inside the Cardano blockchain), "controlled" by a Mu. This deposit can be redeemed at a Mu, as a collection of **θ (Thetas)**, direct equivalents to eUTXOS.

Thetas look like this, when unencrypted:

```json
{
    "asset_id": "a blake3 hash",
    "amount": 10,
    "spend_key": "ed25519 private key",
    "mint": "zk snark proof for the provenance of this theta"
}
```

Users can transfer assets between themselves by creating **∆ (Deltas)**, "bundles" of Thetas with the following schema:

```json
{
    "inputs": [
        "88c9b31ba03f23e10b93d0085a3b17d3bda9e34f880f1c27ad4b8c5bf3b0a62d",
        "4f82c135b2d607df560d8f19a76b0d038d02cf76d163d89328316c7e93a1c1e9"
    ],
    "outputs": [
        "5a2c9d8c3ff5ecb2d5cb9ff6c55963c0c4b286b2c7bbae2c64b68a2d244c2f99",
        "a2f8c2845c4b3f4d9e76b89c2f8d5ba3cbcc7cf0e391cc764a9dc29b76fb6ed7",
        "9cbbf103a4adcc9c0bb81231035b3b45a8cd9f02e60cfe2e8f9b699c718f0601",
        "a663f5ad34f1f8b078c39e915b5438e428997dcc7e2c15297c3e3ef34f1ec9de",
        "391c4fd415eb8f96b2b6fe5700db2aa60ef08b398c2f9b3d62f1cef4d73d216e"
    ],
    "proofs": {
        "cf6eb735cd9bc4c2291cddc7d3a6f5a8352d8d41f3d648979d98e7a0fc3ec99a": "Zk-snark proof for program 1 generated from function1.aleo",
        "e91827e9dc204f44a4782b9aa59e737f0a76f7b9ed406f256995c8a70376b589": "Zk-snark proof for program 2 generated from function2.aleo"
    }
}
```

Each of those Thetas has a different `spend key`, which can be used to redeem an asset on-chain.

- A **Theta** is a piece of data, containing a amount $N$ of a single asset, very much like UTXOs.
- A **Delta** is a transaction, or "bundle" of thetas, which can be either `Spent` or `Returned`.

Unlike a normal UTXO blockchain, however, those assets are not actually what gets transferred. Instead, it behaves much more like "claims" to the equivalent amount of collateral, that can be shared and transacted, as long as value is conserved.

______________________________________________________________________

TODO:

### Conservation of Value on Deltas

Deltas are defined as a mapping between $inputs$ and $outputs$, meaning it is a state transition between the two fields. A given Delta (or blockchain transaction, for that matter) is always valid, as long their inputs are not spent.

Let's define it a little bit more formally, then: for each input $\\Theta\_{\\text{input}, i}$ , where $i$ is the asset being transacted:

$$
\\begin{equation}
\\Theta\_{\\text{new}, i} = \\Theta\_{\\text{input}, i} - \\text{amount}_{\\text{spent}, i} + \\text{amount}_{\\text{returned}, i}
\\end{equation}
$$

And given that:

- $\\Theta\_{\\text{input}, i}$ represents the total unspent outputs available for asset $i$ to the sender.
- $\\text{amount}\_{\\text{spent}, i}$ is the amount of asset $i$ that the sender wishes to transfer.
- $\\text{amount}\_{\\text{returned}, i}$ is the change returned to the sender for asset $i$, if any, after the transaction.

The rule for Conservation of Value becomes clear: **deltas are only valid if their inputs and outputs contain the same amount of value. No value must be created nor destroyed on transactions**.

### Parallelism

The second "rule" we have mentioned before: \*\*

### Synthetic Assets

Mainly what this means is that Synthetic Assets (at least the kind that we are interested in) are minted by depositing a certain amount of collateral, usually on the main asset for the chain (BTC for Bitcoin, ADA for Cardano and so on).

Those assets are minted on the base chain, then made available for use inside
the µgraph network. At any point, synthetics can be redeemed for their current
price directly on the Blockchain.

### Chaumian E-Cash

### Rulechains
