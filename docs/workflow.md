# Workflow

## Assumptions

- Mithril aggregators generate non-repudiable proofs that a transaction has been included in a valid block with enough stake validating it.
- 

## Participants

- Alice: User in the system
- Bob: User in the system
- Charles: User in the system
- Dave: Delegator in the network (acts as the signer or "mint" in the BDHKE protocol)

## Variables

- $G$: Generator point of an elliptic curve (set as the Ristretto curve basepoint)
- $H$: Another generator point of a different elliptic curve
- $d$: Dave's Delegator Node
- $d_v$: Smart Contract Vault held by Dave
- $m_a$: Mithril Aggregator for the chosen Network

## Cryptographic Functions

### Schnorr Signatures

1. `schnorr::sign(private_key: Scalar, message: [u8]) -> Signature`:
   - Generate random $k$
   - Compute $r = G \cdot k$
   - Compute $e = \text{hash}(r, \text{message})$
   - Compute $s = k + e \cdot \text{private\_key}$
   - Return Signature$(r, s)$

2. `schnorr::verify(public_key: RistrettoPoint, signature: Signature, message: [u8]) -> bool`:
   - Compute $e = \text{hash}(r, \text{message})$
   - Check if $G \cdot s = r + \text{public\_key} \cdot e$

### Blind Diffie-Hellman Key Exchange (BDHKE)

1. `diffie_hellman::blind(secret_message: [u8]) -> (RistrettoPoint, Scalar, RistrettoPoint)`:
   - Compute $y = f_\text{htc}(\text{secret\_message})$
   - Generate random $r$
   - Compute $B' = y + G \cdot r$
   - Return $(y, r, B')$

2. `diffie_hellman::sign_blinded(private_key: Scalar, blinded_point: RistrettoPoint) -> (RistrettoPoint, Signature)`:
   - Compute $C' = \text{blinded\_point} \cdot \text{private\_key}$
   - Generate Schnorr signature:
     - Call `schnorr::sign(private_key, &C'.compress().to_bytes())` to get `signature`
   - Return $(C', \text{signature})$

3. `diffie_hellman::unblind_and_verify_signature(signed_point: RistrettoPoint, blinding_factor: Scalar, public_key: RistrettoPoint, signature: Signature) -> Option<RistrettoPoint>`:
   - Verify Schnorr signature:
     - Call `schnorr::verify(public_key, signature, &signed_point.compress().to_bytes())`
   - If verification succeeds, return $C = \text{signed\_point} - \text{public\_key} \cdot \text{blinding\_factor}$
   - Otherwise, return None

4. `diffie_hellman::verify_unblinded_point(private_key: Scalar, message: [u8], unblinded_point: RistrettoPoint) -> bool`:
   - Compute $y = f_\text{htc}(\text{message})$
   - Check if $y \cdot \text{private\_key} = \text{unblinded\_point}$

### Pedersen Commitments

1. `pedersen::commit(value: Scalar, blinding_factor: Scalar, h: RistrettoPoint) -> RistrettoPoint`:
   - Compute $\text{commitment} = G \cdot \text{value} + h \cdot \text{blinding\_factor}$
   - Return $\text{commitment}$

2. `pedersen::verify(commitment: RistrettoPoint, value: Scalar, blinding_factor: Scalar, h: RistrettoPoint) -> bool`:
   - Compute $\text{computed\_commitment} = G \cdot \text{value} + h \cdot \text{blinding\_factor}$
   - Check if $\text{commitment} == \text{computed\_commitment}$

## Workflows

### Setting Up a Delegator

1. Dave generates a private key $a$ and computes the public key $A = G \cdot a$.
2. Dave sends a transaction to the network, including:
   - A list of active valid keys for the delegator set (currently ${A}$)
   - A list of expired keys for the delegator set (currently empty)
   - $G$: a generator point of the elliptic curve
   - $H$: another generator point of a different elliptic curve
3. The network creates the vault $d_v$.

### Minting Tokens

1. Alice deposits 100 ADA into $d_v$.

2. On confirmation, Alice generates a transaction snapshot for her transaction $t_0$ at $m_a$.

3. Alice prepares for the BDHKE protocol:
   - Choose a secret message $x$ (for example, transaction details).
   - Call `diffie_hellman::blind(x)` to get $(y, r, B')$.

4. Alice sends proof of deposit with amounts, transaction preimage, and $B'$ to $d$.

5. Dave (the delegator) responds:
   - Call `diffie_hellman::sign_blinded(a, B')` to get $(C', \text{signature})$.
   - Send the blind signature $(C', \text{signature})$ for each input in $t_0$.

6. Alice unblind-verifies the signature:
   - Call `diffie_hellman::unblind_and_verify_signature(C', r, A, signature)` to get $C$.
   - If $C$ is None, the verification failed.
   - Otherwise, Alice has the unblinded signature $C$.

7. Alice stores the unblinded signature $C$ in her database.

8. Alice provides a Schnorr signature as proof:
   - Generate a private key $k$ and compute the public key $K = G \cdot k$.
   - Call `schnorr::sign(k, x)` to get Signature$(R, s)$.
   - Send $K$, $R$, and $s$ to Dave for verification.

9. Dave verifies Alice's proof:
   - Call `diffie_hellman::verify_unblinded_point(a, x, C)`.
   - If the verification succeeds, Dave confirms the minting process.

### Swapping Tokens

[To be completed]

### Burning Tokens

[To be completed]