# µgraph: Fast, Untraceable Synthetic Payments in Cardano

µgraph is a network where you can deposit ADA into a vault owned by a µ (Mu), a node in the network, working similarly to a bank, minting **Synthetic Assets**, a kind of token with a special property: they mimic the price actions of other assets, by working on top of Collateralized Debt Positions (CDP), or Auctions/Stop-Losses.

In this network:

- You can send and receive money by just sending a text message
- You can transfer money from a cellphone to another, via NFC (Like Google Pay/Apple Pay)
- You can receive payments with any Point-of-Sale (POS) Terminal, as long as it runs Android
- You can have ANY asset, including currencies, stocks, and many other assets and financial instruments
- You still keep custody of your money, always

Our goal is to create a platform where non-technical users can use for real-world payments in a very similar way as current legacy payment systems. We believe, generally, that improving the user experience and a general market for synthetic assets are the next steps into the evolution of self-sovereign money.

And, we want to do it in a way that safeguards the basic human of rights: the right for financial privacy and self-determination.

## Introduction

In a nutshell, µgraph is a network where users leave ADA into a Vault, a Smart Contract inside the Cardano blockchain, together with a **∆ (Delta)**, a collection of **θ (Thetas)**.

Deltas and thetas are direct equivalents to transactions and UTXOs in the Cardano Blockchain, meaning:

- A **Theta** is a piece of data, containing a amount $N$ of a single asset, very much like UTXOs.
- A **Delta** is a transaction, or "bundle" of thetas, which can be either `Spent` or `Returned`.

Unlike a normal UTXO blockchain, however, those assets are not actually what gets transferred. Instead, it behaves much more like "claims" to the equivalent amount of collateral, that can be shared and transacted, as long as value is conserved.

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
