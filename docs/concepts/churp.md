# CHURP: Dynamic-Committee Proactive Secret Sharing

## Introduction

CHURP (CHUrn-Robust Proactive secret sharing) is a protocol that enables secure secret sharing in dynamic settings where the committee of nodes storing a secret changes over time. Designed for blockchain systems, CHURP offers significantly lower communication complexity compared to previous schemes while maintaining strong security guarantees against active adversaries.

## Key Features and Innovations

1. Dynamic Committee Support
   - Allows nodes to join and leave committees between epochs
   - Supports changing the secret sharing threshold ($t$) and committee size ($n$)

2. Proactive Security
   - Periodically refreshes shares to protect against ongoing compromises
   - Maintains security as long as $t$ or fewer nodes are compromised between refreshes

3. Efficient Communication
   - Achieves $O(n)$ on-chain and $O(n^2)$ off-chain complexity in the optimistic case
   - Represents significant improvement over previous $O(n^4)$ schemes

4. Strong Adversarial Model
   - Tolerates up to $t < n/2$ compromised nodes per committee
   - Handles up to $2t$ compromised nodes during handoffs

5. Flexible Execution Paths
   - Optimistic path for efficiency in the common case
   - Pessimistic paths for robustness against faults and cryptographic failures

## Technical Innovations

### Efficient Bivariate 0-Sharing

CHURP introduces a novel bivariate 0-sharing protocol that reduces the off-chain communication complexity of proactivization from $O(n^3)$ to $O(n^2)$. The protocol generates a random bivariate polynomial $Q(x,y)$ of degree $<t,2t>$ such that $Q(0,0) = 0$.

Key steps:
1. Choose a subset $U'$ of $2t+1$ nodes
2. Execute univariate 0-sharing among $U'$ to generate shares $\{s_j\}$
3. Each node in $U'$ generates a random degree-$t$ polynomial $R_j(x)$ with $R_j(0) = s_j$
4. The polynomials $\{R_j(x)\}$ define $Q(x,y)$ where $Q(x,j) = R_j(x)$

This technique achieves $O(tn)$ off-chain communication complexity.

### Dimension-Switching

CHURP uses asymmetric bivariate polynomials $B(x,y)$ of degree $<t,2t>$ to enable secure handoffs between committees. This allows:

- $(t,n)$-sharing of the secret $s = B(0,0)$ in steady state
- Temporary switch to $(2t,n)$-sharing during handoffs to tolerate up to $2t$ compromised nodes

The dimension-switching process:
1. Old committee holds full shares $B(i,y)$
2. Switch to reduced shares $B(x,j)$ of degree $2t$
3. Proactivize to generate $B'(x,y) = B(x,y) + Q(x,y)$
4. Distribute new full shares $B'(i,y)$ to new committee

### KZG Polynomial Commitments with Hedging

CHURP employs the Kate-Zaverucha-Goldberg (KZG) commitment scheme for efficient $O(1)$ size commitments to polynomials. To mitigate risks associated with the scheme's trusted setup and non-standard hardness assumption, CHURP introduces a novel hedging technique:

1. StateVerif: An $O(n)$ on-chain verification protocol that checks invariants without using KZG
2. Fallback to Exp-CHURP-B: If KZG fails, switch to a KZG-free protocol with $t < n/3$ threshold

### Transaction Ghosting

CHURP introduces a technique for inexpensive off-chain communication using blockchain peer-to-peer networks:

1. Embed messages in transactions that are broadcast but subsequently overwritten
2. Achieve ~$0.06/MB cost, orders of magnitude cheaper than on-chain communication
3. Provides anonymity benefits compared to direct peer-to-peer connections

## Protocol Overview

CHURP operates in epochs, with a handoff protocol to transfer shares between committees at epoch boundaries. The handoff consists of three main phases:

1. Share Reduction (Opt-ShareReduce)
   - Switches from full shares $B(i,y)$ to reduced shares $B(x,j)$
   - Uses dimension-switching to tolerate $2t$ compromised nodes

2. Proactivization (Opt-Proactivize)
   - Generates new independent shares using bivariate 0-sharing
   - Updates reduced shares: $B'(x,j) = B(x,j) + Q(x,j)$

3. Full Share Distribution (Opt-ShareDist)
   - Distributes new full shares $B'(i,y)$ to the new committee
   - Restores system to steady state with updated shares

## Execution Paths

CHURP includes multiple execution paths to handle different scenarios:

1. Opt-CHURP (Optimistic Path)
   - Assumes no misbehavior
   - $O(n)$ on-chain, $O(n^2)$ off-chain complexity
   - Requires $t < n/2$ corruption threshold

2. Exp-CHURP-A (Pessimistic Path A)
   - Handles detected faults and identifies malicious nodes
   - $O(n^2)$ on-chain complexity
   - Maintains $t < n/2$ corruption threshold

3. Exp-CHURP-B (Pessimistic Path B)
   - Activated if KZG commitment scheme fails
   - Uses Pedersen commitments instead of KZG
   - Degrades to $t < n/3$ corruption threshold

## State Verification (StateVerif)

CHURP's state verification mechanism:
- Verifies correctness of secret sharing without relying on KZG
- $O(n)$ on-chain communication complexity
- Checks two invariants:
  1. Inv-Secret: $s = B(0,0)$ remains unchanged
  2. Inv-State: $B'(x,y)$ is of degree $<t,2t>$

## Security Guarantees

CHURP provides formal security proofs for:

1. Secrecy: An adversary controlling $\leq t$ nodes per committee learns no information about the secret $s$.

2. Integrity: If $\leq t$ nodes are corrupted in each of the old and new committees, after the handoff:
   - Honest nodes can correctly compute their shares
   - The secret $s$ remains intact

## Performance and Implementation

The authors implemented CHURP and conducted extensive experiments:

- Achieved >1000x improvement in off-chain communication compared to previous schemes for large committees
- Demonstrated practical execution times:
  - ~3 minutes for committee size 1001 in LAN setting
  - Small constant increase (~1.6 seconds) for WAN setting
- Evaluated Transaction Ghosting:
  - Achieved ~32.3 KB/s bandwidth
  - ~$0.06/MB cost for off-chain communication
  - 92.2% message delivery rate

## Applications

CHURP enables several important applications in decentralized systems:

1. Usable cryptocurrency key management
2. Decentralized identity systems
3. Auditable access control for smart contracts
4. Simplified consensus for light clients
5. Secure multiparty computation for smart contracts
6. Threshold cryptography in dynamic settings

By addressing the critical challenge of efficient and secure dynamic-committee proactive secret sharing, CHURP opens up new possibilities for building robust and scalable decentralized systems.