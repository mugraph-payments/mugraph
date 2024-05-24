# Specification

## Terminology

- **µtail:** a node inside the network, signs messages from the users.
- **Galaxy:** made of µtails, collectively controls access to the assets in a Vault.
- **Vault:** a smart contract that holds the funds of a Galaxy.
- **Account:** A "redeemable voucher" for a bundle of assets in a Vault.
- **Asset:** a triplet (`policy_id`, `asset_id`, `amount`), representing the equivalent asset in the vault.
- **Delta:** a "function" that maps a set of accounts into another set of accounts.

## Workflow

The µgraph network is made of Galaxies, "consensus groups" that transact the funds in behalf of the users. They are very small, so transactions inside those galaxies are completely instant and free. They are also **blind**, so they don't know who's transacting what with whom.

Each Galaxy is responsible for a Vault, which contains the user funds deposited in them. Nodes in the galaxy can not spend any funds, but they need a threshold confirmation from the nodes (2/3 of the µtails).

The consensus inside these galaxies is massively simplified to increase speed, at a reduced protection against non-byzantine actors, but also reduced strictness requirements.

More nodes increase reliability, but decreases transaction speed slightly. Those nodes are not required to stay online all the time, but they are expect to not do byzantine actions (though the consensus still protects against it).

Unlike Hydra Heads, Galaxies never publish their checkpoints on-chain, only interacting with the main chain if money needs to be deposited or withdrawn from the Vault.

Their responsibility, instead, is to track the causality between events, propagate those messages between the other µtails in the galaxy, and verify zero-knowledge proofs.

### System Messages

Those are the messages that can be sent by users:

### Deposits

Deposits to the vault are permissionless, and can be done by anyone, and works in a "Two-Phase Commit", that works like this:

1. The user deposits the funds to the vault, and wait for the transaction to be confirmed.
1. The user generates a proof based on **Mithril Certified Transactions**, proving the transaction has been commited on chain and that the user owns the funds that have been deposited.
1. The user "mints" an Account on-chain with the funds they deposited.

### Withdrawals

Withdrawals follows a very similar process:

1. The user sends a "burn" message to the galaxy, together with a Cardano transaction and a list of accounts.
1. Nodes sign the transaction as they propagate the event.
1. User submits transaction to the blockchain.

### Transactions

And for transactions:

1. The user sends a proof of the transaction, that proving that no value has been created or destroyed ($tx\_\\text{in} - tx\_\\text{out}$ must equal $0$).
1. Nodes sign the proof as they propagate the event.
1. Once the aggregated signature has reached the threshold, the transaction is commited.

### Gossip

µgraph uses a very, very simple gossip model:

First, here's how we track membership:

1. Nodes register to the galaxy by interacting with a smart contract.
1. They receive a NFT, which contains their IP in the metadata.
1. Nodes query the chain every few blocks to find new nodes.

And here's what happens when we receive a new message:

1. Have we seen the message before?

- If yes, respond with all signatures collected for this message.
- If not, sign message, then propagate.

1. To propagate the message:

- Choose random Node in Galaxy
- Merge the node Merkle Tree with the other node Merkle Tree
- Accumulate unseen signatures
- Propagate again if to another random node if any new message is seen.

1. Start at the latest round when node joined the network
1. Increase round number when new Mithril (deposit/withdrawal) proof is seen

This graph shows in a very simplified way how the messages are propagated in the system. The yellow line represents the chain itself, and merges in and out of the yellow lines are withdrawals and deposits, respectively.

```mermaid
%%{init: {'logLevel': 'debug', 'theme': 'base', 'gitGraph': {'showBranches': true,'mainBranchName': 'Cardano'}} }%%
gitGraph TB:
    branch NodeA
    branch NodeB
    checkout NodeA
    commit id: "... chain"
    checkout NodeB
    commit id: "... nodea"
    checkout Cardano
    commit id: "... nodeb"
    checkout Cardano
    commit id: "Deposit"
    checkout NodeA
    merge Cardano id: "Mint" tag: "Mithril Proof of Deposit"
    commit id: "Acknowledge Deposit"
    checkout NodeB
    merge NodeA
    commit
    checkout NodeA
    merge NodeB
    commit
    commit
    checkout NodeB
    commit
    merge NodeA
    commit
    commit
    checkout NodeA
    merge NodeB
    commit id: "Withdrawal"
    checkout NodeA
    merge NodeB
    commit id: "Sign Withdrawal"
    checkout NodeB
    merge NodeA
    checkout NodeA
    merge NodeB
    checkout Cardano
    merge NodeA id: "Submit to chain"
```

## Data Structures

### Zero Tree

This is what a Merkle Tree usually looks like:

```mermaid
graph TB
    Root((Root)) --> L1((L1))
    Root((Root)) --> R1((R1))
    
    L1((L1)) --> L2((L2))
    L1((L1)) --> R2((R2))
    
    R1((R1)) --> L3((L3))
    R1((R1)) --> R3((R3))
    
    L2((L2)) --> L4((L4))
    L2((L2)) --> R4((R4))
    
    R2((R2)) --> L5((L5))
    R2((R2)) --> R5((R5))
    
    L3((L3)) --> L6((L6))
    L3((L3)) --> R6((R6))
    
    R3((R3)) --> L7((L7))
    R3((R3)) --> R7((R7))
    
    L4((L4)) --> D1((D1))
    L4((L4)) --> D2((D2))
    
    R4((R4)) --> D3((D3))
    R4((R4)) --> D4((D4))
    
    L5((L5)) --> D5((D5))
    L5((L5)) --> D6((D6))
    
    R5((R5)) --> D7((D7))
    R5((R5)) --> D8((D8))
    
    L6((L6)) --> D9((D9))
    L6((L6)) --> D10((D10))
    
    R6((R6)) --> D11((D11))
    R6((R6)) --> D12((D12))
    
    L7((L7)) --> D13((D13))
    L7((L7)) --> D14((D14))
    
    R7((R7)) --> D15((D15))
    R7((R7)) --> D16((D16))
```

For a Sparse Merkle Tree, instead, the right leaf can be null, and nulls have a single, known hash:

```mermaid
graph TB
    Root((Hash)) --> L1((L1))
    Root((Hash)) --> R1((R1))
    
    L1((L1)) --> L2((L2))
    L1((L1)) --> R2((R2))
    
    R1((R1)) --> L3((L3))
    R1((R1)) --> R3((∅))
    
    L2((L2)) --> L4((L4))
    L2((L2)) --> R4((∅))
    
    R2((R2)) --> R5((R5))
    R2((R2)) --> L5((∅))
    
    L3((L3)) --> L6((L6))
    L3((L3)) --> R6((∅))
    
    L4((L4)) --> D2((D1))
    L4((L4)) --> D1((∅))
    
    R5((R5)) --> D3((D2))
    R5((R5)) --> D4((∅))
    
    L6((L6)) --> D6((D3))
    L6((L6)) --> D5((∅))
```

What this means for us is that we can keep "partial snapshots" of the tree, and only needing to share the first level of the tree publically, massively reducing the size of what is shared between users.

### Merkle Sum Trees

A **Merkle Sum Tree** is a variant of a Merkle tree in which each node contains a value, and this value needs to be taken into consideration when generating the hash.

The benefit of this is that the tree itself guarantees that no value is created and no value is destroyed as new items are appended to it.

```mermaid
graph TB
    Root((36)) --> L1((10))
    Root((36)) --> R1((26))
    
    L1((10)) --> L2((3))
    L1((10)) --> R2((7))
    
    R1((26)) --> L3((12))
    R1((26)) --> R3((14))
    
    L2((3)) --> D1((1))
    L2((3)) --> D2((2))
    
    R2((7)) --> D3((3))
    R2((7)) --> D4((4))
    
    L3((12)) --> D5((5))
    L3((12)) --> D6((7))
    
    R3((14)) --> D7((6))
    R3((14)) --> D8((8))
```

### Zero Sum Trees, Finally

A **Zero Sum Tree**, then, is a Sparse Merkle Sum Tree that, when complete, has a total sum of 0.

```mermaid
graph TB
    Root(("0")) --> L1(("5"))
    Root(("0")) --> R1(("-5"))
    
    L1(("5")) --> L2(("3"))
    L1(("5")) --> R2(("-3"))
    
    R1(("-5")) --> L3(("2"))
    R1(("-5")) --> R3(("-2"))
    
    L2(("3")) --> D1(("3"))
    L2(("3")) --> D2(("-3"))
    
    R2(("-3")) --> D3(("2"))
    R2(("-3")) --> D4(("-2"))
    
    L3(("2")) --> D5(("2"))
    L3(("2")) --> D6(("-2"))
    
    R3(("-2")) --> D7(("1"))
    R3(("-2")) --> D8(("-1"))
```

What is important here is that the Hidden Layer is not seen by the nodes themselves, because what is published to them are **Delta Proofs**.

Those proofs are generated by the users, and verified by the nodes before propagation.

### Delta Proofs

In a nutshell, Delta Proofs are a form of "$\\delta$-mutators" (from the $\\delta$-state CRDT family), that is, a "delta" between the previous state and the current state.

But instead of publishing all the state changes, users in the Constellation instead provide a new root for the Zero Sum Tree, together with a zero-knowledge proof that this root is the result of the previous root and the delta.

## µtails

µtails, or the nodes inside a galaxy, receive transactions, deposits and withdraw from users, propagate those to other nodes inside the same galaxy, and maintain a **Causality Graph** of all the events in the system.

This causality graph encodes the binary relationship $\\leq$ between each event in the system, or in other words, what event happened before the other.
