# Workflow

## Assumptions

- Mithril aggregators generate non-repudiable proofs that a transaction has been included in a valid block with enough stake validating it.
- [Additional assumptions to be added as the system develops]

## Participants

- Alice: A user in the system
- Bob: A user in the system
- Charles: A user in the system
- Dave: A delegator in the network (acts as the signer or "mint" in the BDHKE protocol)

## Variables

- $G$: Generator point of an elliptic curve (set as the Ristretto curve basepoint)
- $d$: Dave's Delegator Node
- $d_v$: Smart Contract Vault held by Dave
- $m_a$: Mithril Aggregator for the chosen Network

## Cryptographic Functions

### Implementing Blind Diffie-Hellman Key Exchange (BDHKE)

The following functions implement the BDHKE protocol:

1. `diffie_hellman::blind(secret_message: [u8]) -> (RistrettoPoint, Scalar, RistrettoPoint)`

   This function blinds a secret message:
   
   - Compute $y = f_\text{htc}(\text{secret message})$, where $f_\text{htc}$ is a hash-to-curve function
   - Generate a random scalar $r$
   - Compute $B' = y + G \cdot r$
   - Return the tuple $(y, r, B')$

2. `diffie_hellman::sign_blinded(private_key: Scalar, blinded_point: RistrettoPoint) -> (RistrettoPoint, DLEQProof)`

   This function signs a blinded point and generates a Discrete Logarithm Equality (DLEQ) proof:
   
   - Compute $C' = \text{blinded point} \cdot \text{private key}$
   - Compute $\text{public key} = G \cdot \text{private key}$
   - Generate DLEQ proof:
     - Generate a random scalar $k$
     - Compute $R_1 = G \cdot k$
     - Compute $R_2 = \text{blinded point} \cdot k$
     - Compute $e = \text{hash}(R_1, R_2, \text{public key}, C')$
     - Compute $s = k + e \cdot \text{private key}$
   - Return the tuple $(C', \text{DLEQProof}(e, s))$

3. `diffie_hellman::verify_dleq_proof(public_key: RistrettoPoint, blinded_point: RistrettoPoint, signed_point: RistrettoPoint, proof: DLEQProof) -> bool`

   This function verifies a DLEQ proof:
   
   - Compute $R_1 = G \cdot s - \text{public key} \cdot e$
   - Compute $R_2 = \text{blinded point} \cdot s - \text{signed point} \cdot e$
   - Compute $e' = \text{hash}(R_1, R_2, \text{public key}, \text{signed point})$
   - Return true if $e == e'$, false otherwise

4. `diffie_hellman::unblind_and_verify_signature(signed_point: RistrettoPoint, blinding_factor: Scalar, public_key: RistrettoPoint, proof: DLEQProof, blinded_point: RistrettoPoint) -> Option<RistrettoPoint>`

   This function unblinds and verifies a signature:
   
   - If `verify_dleq_proof(public_key, blinded_point, signed_point, proof)` succeeds:
     - Compute and return $C = \text{signed point} - \text{public key} \cdot \text{blinding factor}$
   - Otherwise, return None

5. `diffie_hellman::verify_unblinded_point(private_key: Scalar, message: [u8], unblinded_point: RistrettoPoint) -> bool`

   This function verifies an unblinded point:
   
   - Compute $y = f_\text{htc}(\text{message})$
   - Return true if $y \cdot \text{private key} = \text{unblinded point}$, false otherwise

### Implementing Pedersen Commitments

The following functions implement Pedersen Commitments:

1. `pedersen::commit(value: Scalar, blinding_factor: Scalar, h: RistrettoPoint) -> RistrettoPoint`

   This function creates a Pedersen Commitment:
   
   - Compute $\text{commitment} = G \cdot \text{value} + h \cdot \text{blinding factor}$
   - Return $\text{commitment}$

2. `pedersen::verify(commitment: RistrettoPoint, value: Scalar, blinding_factor: Scalar, h: RistrettoPoint) -> bool`

   This function verifies a Pedersen Commitment:
   
   - Compute $\text{computed commitment} = G \cdot \text{value} + h \cdot \text{blinding factor}$
   - Return true if $\text{commitment} == \text{computed commitment}$, false otherwise

## Workflows

### Setting Up a Delegator

1. Dave generates a private key $a$ and computes the public key $A = G \cdot a$.

2. Dave generates another generator point $H$, which will be used for Pedersen commitments.

3. Dave sends a transaction to the network, including:
   - A list of active valid keys for the delegator set (currently ${A}$)
   - A list of expired keys for the delegator set (currently empty)
   - $G$: the generator point of the elliptic curve
   - $H$: the second generator point for Pedersen commitments

4. Dave creates the vault $d_v$ on the network.

### Minting Tokens

1. Alice deposits 100 ADA into $d_v$.

2. Upon confirmation, Alice generates a transaction snapshot for her transaction $t_0$ at $m_a$.

3. Alice prepares for the BDHKE protocol:
   - Choose a secret message $x$ (for example, transaction details).
   - Call `diffie_hellman::blind(x)` to obtain $(y, r, B')$.

4. Alice sends proof of deposit with amounts, transaction preimage, and $B'$ to $d$.

5. Dave (the delegator) responds:
   - Call `diffie_hellman::sign_blinded(a, B')` to obtain $(C', \text{DLEQProof})$.
   - Send the blind signature $(C', \text{DLEQProof})$ for each input in $t_0$.

6. Alice unblind-verifies the signature:
   - Call `diffie_hellman::unblind_and_verify_signature(C', r, A, DLEQProof, B')` to obtain $C$.
   - If $C$ is None, the verification failed.
   - Otherwise, Alice has the unblinded signature $C$.

7. Alice stores the unblinded signature $C$ in her database.

8. Alice can later prove ownership of the minted tokens by demonstrating knowledge of $x$ and $C$.

9. Dave can verify Alice's proof by calling `diffie_hellman::verify_unblinded_point(a, x, C)`.

### Swapping Tokens

The swap operation allows users to split, combine, or exchange tokens while maintaining privacy of the amounts. It involves multiple inputs (Proofs) and outputs (BlindedMessages), using Pedersen commitments to hide the actual amounts.

To swap tokens:

1. Alice prepares her existing tokens. She has proofs $P_1, P_2, ..., P_n$ with corresponding amounts $a_1, a_2, ..., a_n$.

2. Alice generates new secret messages $x_1, x_2, ..., x_m$ for the desired output tokens with corresponding amounts $b_1, b_2, ..., b_m$.

3. For each input amount $a_i$, Alice creates a Pedersen commitment:
   - Generate a random blinding factor $r_i$
   - Call `pedersen::commit(a_i, r_i, H)` to obtain $C_i^{in}$

4. For each output amount $b_i$, Alice creates a Pedersen commitment:
   - Generate a random blinding factor $s_i$
   - Call `pedersen::commit(b_i, s_i, H)` to obtain $C_i^{out}$

5. For each secret message $x_i$, Alice performs the blinding operation:
   - Call `diffie_hellman::blind(x_i)` to obtain $(y_i, t_i, B'_i)$

6. Alice prepares the swap request:
   - Inputs: $\{(P_1, C_1^{in}, r_1), (P_2, C_2^{in}, r_2), ..., (P_n, C_n^{in}, r_n)\}$
   - Outputs: $\{(B'_1, C_1^{out}, s_1), (B'_2, C_2^{out}, s_2), ..., (B'_m, C_m^{out}, s_m)\}$
   
   Alice keeps the actual amounts $a_i$ and $b_i$ private.

7. Alice sends the swap request to Dave's delegator node $d$.

8. Dave verifies the input proofs and checks that the sum of input commitments equals the sum of output commitments:
   - Compute $\sum_{i=1}^n C_i^{in} = \sum_{i=1}^m C_i^{out}$
   - This equality holds due to the homomorphic property of Pedersen commitments

9. For each output $(B'_i, C_i^{out}, s_i)$, Dave:
   - Calls `diffie_hellman::sign_blinded(a, B'_i)` to obtain $(C'_i, \text{DLEQProof}_i)$

10. Dave sends the blind signatures $\{(C'_1, \text{DLEQProof}_1), ..., (C'_m, \text{DLEQProof}_m)\}$ to Alice.

11. For each received blind signature, Alice:
    - Calls `diffie_hellman::unblind_and_verify_signature(C'_i, t_i, A, \text{DLEQProof}_i, B'_i)` to obtain $C_i$
    - If any $C_i$ is None, the verification fails

12. Alice now has new proofs $\{(x_1, C_1), (x_2, C_2), ..., (x_m, C_m)\}$ with corresponding amounts $\{b_1, b_2, ..., b_m\}$

13. Alice stores these new proofs and their associated amounts in her database.

14. The system invalidates the original input proofs $P_1, P_2, ..., P_n$, preventing their further use.

Note: To preserve privacy, Alice should randomize the order of both inputs and outputs in the swap request.

### Receiving Swapped Tokens

When Carol receives Proofs from Alice, she can perform a similar swap operation to obtain new Proofs:

1. Carol receives Proofs $\{(x_1, C_1), (x_2, C_2), ..., (x_k, C_k)\}$ from Alice.

2. Carol generates new secret messages $z_1, z_2, ..., z_l$ for her desired output tokens.

3. Carol performs steps 3-11 as described above, using the received Proofs as inputs.

4. After the swap, Carol has new Proofs that only she can spend, and the Proofs received from Alice are invalidated.

### Burning Tokens to Redeem ADA

[This section is yet to be completed.]