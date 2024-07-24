# Workflow

## Assumptions

- Mithril aggregators generate non-repudiable proofs that a transaction has been included in a valid block with sufficient stake validation.
- All Vaults initialize with a default balance of Zero.
- All hash functions utilize Blake2B-256.
- All cryptographic operations are performed using the Ristretto25519 curve.

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

12. $\text{CreateAssetCommitment}(asset\_id, amount, blinding) \rightarrow AssetCommitment$

    This function creates an asset commitment:
    
    - Compute $h_a = \text{hash\_to\_curve}(asset\_id)$
    - Compute $commitment = G \cdot amount + H \cdot blinding + h_a$
    - Create a range proof for $amount$
    - Generate random $r$
    - Compute $asset\_id\_commitment = G \cdot asset\_id + H \cdot r$
    - Compute $t = h_a \cdot blinding$
    - Compute $challenge = \mathcal{H}(commitment, asset\_id\_commitment, t)$
    - Compute $s = r + challenge \cdot blinding$
    - Return $AssetCommitment(commitment, AssetProof(t, s, asset\_id\_commitment), range\_proof)$

13. $\text{VerifyAssetCommitment}(commitment, asset\_id) \rightarrow \{0, 1\}$

    This function verifies an asset commitment:
    
    - Verify the range proof
    - Compute $h_a = \text{hash\_to\_curve}(asset\_id)$
    - Compute $challenge = \mathcal{H}(commitment.commitment, commitment.asset\_proof.asset\_id\_commitment, commitment.asset\_proof.t)$
    - Compute $LHS = G \cdot commitment.asset\_proof.s + h_a \cdot challenge$
    - Compute $RHS = commitment.asset\_proof.asset\_id\_commitment + commitment.asset\_proof.t \cdot challenge$
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

### Swapping Tokens (Multi-Asset Version with Visible Asset Types and Asset Commitments)

1. Define asset types $\{A_1, A_2, ..., A_k\}$.

2. Alice prepares input proofs $P_1, P_2, ..., P_n$ with amounts $\{a_{1,j}, a_{2,j}, ..., a_{n,j}\}$ for each asset type $j$.

3. Alice generates output secret messages $x_1, x_2, ..., x_m$ with amounts $\{b_{1,j}, b_{2,j}, ..., b_{m,j}\}$ for each asset type $j$.

4. For each input and asset type, Alice creates an asset commitment:
   - Generate random blinding factor $r_{i,j}$
   - Call $\text{CreateAssetCommitment}(A_j, a_{i,j}, r_{i,j})$ to obtain $C_{i,j}^{in}$

5. For each output and asset type, Alice creates an asset commitment:
   - Generate random blinding factor $s_{i,j}$
   - Call $\text{CreateAssetCommitment}(A_j, b_{i,j}, s_{i,j})$ to obtain $C_{i,j}^{out}$

6. Alice computes the difference commitment for each asset type:
   $C_j = \sum_{i=1}^n C_{i,j}^{in}.commitment - \sum_{i=1}^m C_{i,j}^{out}.commitment$

7. Alice blinds each output secret message:
   - Call $\text{Blind}(x_i)$ to obtain $(y_i, t_i, B'_i)$

8. Alice sends to Dave:
   - Input proofs: $\{P_1, P_2, ..., P_n\}$
   - Input asset commitments: $\{C_{i,j}^{in}\}$ for all $i$ and $j$
   - Output asset commitments: $\{C_{i,j}^{out}\}$ for all $i$ and $j$
   - Difference commitments: $\{C_1, C_2, ..., C_k\}$
   - Blinded output points: $\{B'_1, B'_2, ..., B'_m\}$

9. Dave verifies:
   - Validity of input proofs $P_1, ..., P_n$
   - All asset commitments using $\text{VerifyAssetCommitment}$
   - For each asset type $j$: $C_j = \mathcal{O}$ (the identity element)

10. If verified, Dave blind signs each $B'_i$.

11. Alice unblinds and verifies signatures, obtaining new proofs $\{(x_1, C_1), ..., (x_m, C_m)\}$.

12. Alice stores new proofs; system invalidates original input proofs.

### Creating and Verifying Asset Commitments

1. To create an asset commitment for asset type $A$ with amount $v$:
   - Generate a random blinding factor $r$
   - Call $\text{CreateAssetCommitment}(A, v, r)$ to obtain $C$
   - Store $(A, v, r, C)$ securely

2. To prove ownership of an asset commitment $C$:
   - Retrieve the stored $(A, v, r, C)$
   - Send $(A, v, r, C)$ to the verifier

3. To verify an asset commitment:
   - Receive $(A, v, r, C)$ from the prover
   - Call $\text{VerifyAssetCommitment}(C, A)$
   - If verification succeeds, the commitment is valid for asset type $A$
   - To verify the amount, check that $C.commitment = G \cdot v + H \cdot r + \text{hash\_to\_curve}(A)$

### Receiving Swapped Tokens

When Carol receives Proofs from Alice, she can perform a similar swap operation to obtain new Proofs:

1. Carol receives Proofs $\{(x_1, C_1), (x_2, C_2), ..., (x_k, C_k)\}$ from Alice.

1. Carol generates new secret messages $z_1, z_2, ..., z_l$ for her desired output tokens.

1. Carol performs steps 3-11 as described above, using the received Proofs as inputs.

1. After the swap, Carol has new Proofs that only she can spend, and the Proofs received from Alice are invalidated.

### Burning Tokens to Redeem ADA

[This section is yet to be completed.]