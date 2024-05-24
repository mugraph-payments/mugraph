# Introduction

µgraph is a network of nodes, called µtails, that form small groups, called Galaxies, to collectively control access to a Vault of user funds. They work similarly to Hydra Heads, but with a very simplified architecture, and a relaxed requirement on node availability (nodes can be offline for a long time).

Nodes are registered directly on-chain, by interacting with a smart contract, and their membership is semi-dynamic, so nodes can take a while to join the network.

Those nodes do three things:

1. They receive messages from the users, and propagate them to the other nodes.
1. They sign messages they haven't seen before.
1. They keep a "log" of everything they have seen before, and in what order.

They don't know at all what those messages contain, and who is sending them, and they don't need to. They are **Blind Signers**, and they only sign messages that are known to be valid.

Once a meaningful threshold of nodes have signed a message, it is commited. There's nothing else that needs to be done.

## Accounts and Deltas

Anyone can deposit funds into these vaults, and `mint` those assets inside the network. When minted, those assets are wrapped in an **Account**, a simple file that contains the assets, and a `spend_key`, which when revealed "burns" the account so it can not be used again.

Accounts can be `minted`, `burned`, and `spent`, meaning:

- `mint`: when a user deposits funds into a Vault, those funds are wrapped into one Account.
- `burn`: an account can be `burned` to get the nodes to sign a transaction to withdraw those funds on the blockchain.
- `spent`: an account can be `spent` to generate new accounts. The account is `closed`, can not be used again, so only their children are now unspent.

Each of those operations can be sent to tails inside the galaxy as a zero-knowledge proof, which guarantees both the "provenance" of the operation, and the authenticity of the operation (no value has been created or destroyed, only maintained).
