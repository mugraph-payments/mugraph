# Workflow

## Assumptions

- Mithril aggregators generate non-repudiable proofs that a transaction has been included in a valid block with sufficient stake validation.
- [Additional assumptions will be added as the system develops]

## Participants

- Alice: A user in the system
- Bob: A user in the system
- Charles: A user in the system
- Dave: A delegator in the network (acts as the signer or "mint" in the BDHKE protocol)

## Variables and Functions

1. $G$: Generator point of an elliptic curve (set as the Ristretto curve basepoint)
1. $d$: Dave's Delegator Node
1. $d_v$: Smart Contract Vault held by Dave
1. $m_a$: Mithril Aggregator for the chosen Network
1. $\text{Blind}(m) \rightarrow (y, r, B')$

   This function blinds a secret message $m$:
   
   - Compute $y = f_\text{htc}(m)$, where $f_\text{htc}$ is a hash-to-curve function
   - Generate a random scalar $r$
   - Compute $B' = y + G \cdot r$
   - Return the tuple $(y, r, B')$

1. $\text{SignBlinded}(sk, B') \rightarrow (C', \pi_\text{DLEQ})$

   This function signs a blinded point and generates a Discrete Logarithm Equality (DLEQ) proof:
   
   - Compute $C' = B' \cdot sk$
   - Compute $pk = G \cdot sk$
   - Generate DLEQ proof $\pi_\text{DLEQ}$:
     - Generate a random scalar $k$
     - Compute $R_1 = G \cdot k$
     - Compute $R_2 = B' \cdot k$
     - Compute $e = \mathcal{H}(R_1, R_2, pk, C')$
     - Compute $s = k + e \cdot sk$
   - Return the tuple $(C', \pi_\text{DLEQ}(e, s))$

1. $\text{VerifyDLEQProof}(pk, B', C', \pi_\text{DLEQ}) \rightarrow \{0, 1\}$

   This function verifies a DLEQ proof:
   
   - Compute $R_1 = G \cdot s - pk \cdot e$
   - Compute $R_2 = B' \cdot s - C' \cdot e$
   - Compute $e' = \mathcal{H}(R_1, R_2, pk, C')$
   - Return 1 if $e = e'$, 0 otherwise

1. $\text{UnblindAndVerifySignature}(C', r, pk, \pi_\text{DLEQ}, B') \rightarrow C \text{ or } \perp$

   This function unblinds and verifies a signature:
   
   - If $\text{VerifyDLEQProof}(pk, B', C', \pi_\text{DLEQ}) = 1$:
     - Compute and return $C = C' - pk \cdot r$
   - Otherwise, return $\perp$

1. $\text{VerifyUnblindedPoint}(sk, m, C) \rightarrow \{0, 1\}$

   This function verifies an unblinded point:
   
   - Compute $y = f_\text{htc}(m)$
   - Return 1 if $y \cdot sk = C$, 0 otherwise

1. $\text{Commit}(v, r, H) \rightarrow C$

   This function creates a Pedersen Commitment:
   
   - Compute $C = G \cdot v + H \cdot r$
   - Return $C$

1. $\text{VerifyCommitment}(C, v, r, H) \rightarrow \{0, 1\}$

   This function verifies a Pedersen Commitment:
   
   - Compute $C' = G \cdot v + H \cdot r$
   - Return 1 if $C = C'$, 0 otherwise

1. $\text{SchnorrSign}(sk, m) \rightarrow (R, s)$

   This function signs a message $m$ using private key $sk$:
   
   - Generate a random scalar $k$
   - Compute $R = G \cdot k$, where $G$ is the Ristretto basepoint
   - Compute $e = \mathcal{H}(R, m)$, where $\mathcal{H}$ is a hash-to-scalar function
   - Compute $s = k + e \cdot sk$
   - Return the signature $(R, s)$

1. $\text{SchnorrVerify}(pk, (R, s), m) \rightarrow \{0, 1\}$

   This function verifies a Schnorr signature $(R, s)$ on message $m$ using public key $pk$:
   
   - Compute $e = \mathcal{H}(R, m)$
   - Compute $LHS = G \cdot s$
   - Compute $RHS = R + pk \cdot e$
   - Return 1 if $LHS = RHS$, 0 otherwise

Note: In the actual implementation, $R$ is a Ristretto point, $s$ is a scalar, and the public key $pk$ is also a Ristretto point. The basepoint $G$ is the Ristretto basepoint.

## Workflows

### Setting Up a Delegator

1. Dave generates a private key $a$ and computes the public key $A = G \cdot a$.

1. Dave generates another generator point $H$, which will be used for Pedersen commitments.

1. Dave sends a transaction to the network, including:
   - A list of active valid keys for the delegator set (currently ${A}$)
   - A list of expired keys for the delegator set (currently empty)
   - $G$: the generator point of the elliptic curve
   - $H$: the second generator point for Pedersen commitments

1. Dave creates the vault $d_v$ on the network.

### Minting Tokens

1. Alice deposits 100 ADA into $d_v$.

1. Upon confirmation, Alice generates a transaction snapshot for her transaction $t_0$ at $m_a$.

1. Alice prepares for the BDHKE protocol:
   - Choose a secret message $x$ (for example, transaction details).
   - Call $\text{Blind}(x)$ to obtain $(y, r, B')$.

1. Alice sends proof of deposit with amounts, transaction preimage, and $B'$ to $d$.

1. Dave (the delegator) responds:
   - Call $\text{SignBlinded}(a, B')$ to obtain $(C', \text{DLEQProof})$.
   - Send the blind signature $(C', \text{DLEQProof})$ for each input in $t_0$.

1. Alice unblind-verifies the signature:
   - Call $\text{UnblindAndVerifySignature}(C', r, A, \text{DLEQProof}, B')$ to obtain $C$.
   - If $C$ is None, the verification failed.
   - Otherwise, Alice has the unblinded signature $C$.

1. Alice stores the unblinded signature $C$ in her database.

1. Alice can later prove ownership of the minted tokens by demonstrating knowledge of $x$ and $C$.

1. Dave can verify Alice's proof by calling $\text{VerifyUnblindedPoint}(a, x, C)$.

### Swapping Tokens

The swap operation allows users to split, combine, or exchange tokens while maintaining privacy of the amounts. It involves multiple inputs (Proofs) and outputs (BlindedMessages), using Pedersen commitments to hide the actual amounts.

To swap tokens:

1. Alice prepares her existing tokens. She has proofs $P_1, P_2, ..., P_n$ with corresponding amounts $a_1, a_2, ..., a_n$.

1. Alice generates new secret messages $x_1, x_2, ..., x_m$ for the desired output tokens with corresponding amounts $b_1, b_2, ..., b_m$.

1. For each input amount $a_i$, Alice creates a Pedersen commitment:
   - Generate a random blinding factor $r_i$
   - Call $\text{Commit}(a_i, r_i, H)$ to obtain $C_i^{in}$

1. For each output amount $b_i$, Alice creates a Pedersen commitment:
   - Generate a random blinding factor $s_i$
   - Call $\text{Commit}(b_i, s_i, H)$ to obtain $C_i^{out}$

1. For each secret message $x_i$, Alice performs the blinding operation:
   - Call $\text{Blind}(x_i)$ to obtain $(y_i, t_i, B'_i)$

1. Alice prepares the swap request:
   - Inputs: $\{(P_1, C_1^{in}, r_1), (P_2, C_2^{in}, r_2), ..., (P_n, C_n^{in}, r_n)\}$
   - Outputs: $\{(B'_1, C_1^{out}, s_1), (B'_2, C_2^{out}, s_2), ..., (B'_m, C_m^{out}, s_m)\}$
   
   Alice keeps the actual amounts $a_i$ and $b_i$ private.

1. Alice sends the swap request to Dave's delegator node $d$.

1. Dave verifies the input proofs and checks that the sum of input commitments equals the sum of output commitments:
   - Compute $\sum_{i=1}^n C_i^{in} = \sum_{i=1}^m C_i^{out}$
   - This equality holds due to the homomorphic property of Pedersen commitments

1. For each output $(B'_i, C_i^{out}, s_i)$, Dave:
   - Calls $\text{SignBlinded}(a, B'_i)$ to obtain $(C'_i, \text{DLEQProof}_i)$

10. Dave sends the blind signatures $\{(C'_1, \text{DLEQProof}_1), ..., (C'_m, \text{DLEQProof}_m)\}$ to Alice.

11. For each received blind signature, Alice:
    - Calls $\text{UnblindAndVerifySignature}(C'_i, t_i, A, \text{DLEQProof}_i, B'_i)$ to obtain $C_i$
    - If any $C_i$ is None, the verification fails

12. Alice now has new proofs $\{(x_1, C_1), (x_2, C_2), ..., (x_m, C_m)\}$ with corresponding amounts $\{b_1, b_2, ..., b_m\}$

13. Alice stores these new proofs and their associated amounts in her database.

14. The system invalidates the original input proofs $P_1, P_2, ..., P_n$, preventing their further use.

Note: To preserve privacy, Alice should randomize the order of both inputs and outputs in the swap request.

### Receiving Swapped Tokens

When Carol receives Proofs from Alice, she can perform a similar swap operation to obtain new Proofs:

1. Carol receives Proofs $\{(x_1, C_1), (x_2, C_2), ..., (x_k, C_k)\}$ from Alice.

1. Carol generates new secret messages $z_1, z_2, ..., z_l$ for her desired output tokens.

1. Carol performs steps 3-11 as described above, using the received Proofs as inputs.

1. After the swap, Carol has new Proofs that only she can spend, and the Proofs received from Alice are invalidated.

### Burning Tokens to Redeem ADA

[This section is yet to be completed.]