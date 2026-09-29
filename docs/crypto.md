# µgraph Cryptography

This document explains the cryptography of µgraph version 1.0 and shows why
it works. The protocol specification (`docs/spec/protocol.md`, sections 4 to 6)
gives the exact encodings.

## 1. Parts

| Part              | Choice                                               | Code                 |
| ----------------- | ---------------------------------------------------- | -------------------- |
| Group             | Ristretto255, base point `G`                         | `curve25519-dalek`   |
| Hash              | BLAKE3, with a domain tag for each use               | `blake3`             |
| Hash to the group | Ristretto map of 64 bytes of BLAKE3 output           | `core/src/crypto.rs` |
| Blind signature   | Blind Diffie-Hellman key exchange (BDHKE)            | `core/src/crypto.rs` |
| Proof             | Chaum-Pedersen proof of discrete log equality (DLEQ) | `core/src/crypto.rs` |
| Keys              | One key for each asset and denomination              | `core/src/keyset.rs` |

## 2. Blind signature

Alice is a wallet. Bob is the delegate, with the secret key `k` and the public
key `K = k·G`. Alice wants Bob to sign a message `x` without Bob to see `x`.

**Blind.** Alice calculates the point `Y = hash_to_curve(x)`. She selects a
random scalar `r` and sends `B' = Y + r·G` to Bob.

**Sign.** Bob sends back `C' = k·B'`.

**Unblind.** Alice calculates `C = C' − r·K`. This is the signature:

$$
\begin{aligned}
C &= C' - r \cdot K \\
  &= k \cdot (Y + r \cdot G) - r \cdot (k \cdot G) \\
  &= k \cdot Y
\end{aligned}
$$

**Verify.** Bob accepts `C` for `x` if `C = k·hash_to_curve(x)`. Only Bob
can do this check, because only Bob knows `k`.

Bob sees only `B'`. For each message `x`, there is a value of `r` that gives
the same `B'`. Thus, `B'` gives Bob no information about `x`. When Alice spends
the note later, Bob can not link `C` to `B'`.

## 3. Hash to the group

`hash_to_curve` must give a point with an unknown discrete log. Assume that a
person knows `y` with `Y = y·G`. Then that person can calculate the signature
without Bob:

$$
k \cdot Y = k \cdot y \cdot G = y \cdot K
$$

Version 0 used `Y = H(x)·G`, so any person could make signatures. Version 1.0
uses the Ristretto `from_uniform_bytes` map on 64 bytes of BLAKE3 output. No
person knows the discrete log of the result.

## 4. DLEQ proof

Bob proves that he used the same `k` in `K = k·G` and in `C' = k·B'`. The proof
stops a delegate that signs with a different key for each user, to mark the
users.

**Prove.** Bob selects a random scalar `n` and calculates:

$$
\begin{aligned}
R_1 &= n \cdot G \\
R_2 &= n \cdot B' \\
e &= \text{hash}(G, B', K, C', R_1, R_2) \\
z &= n + e \cdot k
\end{aligned}
$$

Bob sends `e` and `z` with `C'`.

**Verify.** Alice calculates:

$$
\begin{aligned}
R_1 &= z \cdot G - e \cdot K \\
R_2 &= z \cdot B' - e \cdot C'
\end{aligned}
$$

Alice accepts if `hash(G, B', K, C', R_1, R_2) = e`. For an honest proof,
`z·G − e·K = n·G` and `z·B' − e·C' = n·B'`, so the hash gives `e` again.

## 5. Verification by a holder

A holder of a note does not know `k`, so it can not check `C = k·Y`. Thus, a
note carries `r`, `e` and `z`. The holder calculates:

$$
\begin{aligned}
Y &= \text{hash\_to\_curve}(x) \\
B' &= Y + r \cdot G \\
C' &= C + r \cdot K
\end{aligned}
$$

Then the holder checks the DLEQ proof for `(K, B', C')`. If the proof is
valid, Bob signed `B'` with `k`, and thus `C = k·Y`.

## 6. Denomination keys

The delegate uses a different key for each asset `A` and each amount `2^d`:

$$
k_{A,d} = \text{hash}(k_M, A, d)
$$

Here `k_M` is the master key. A blinded point hides the amount of the note.
The key fixes the amount: a signature from the key for 1 lovelace is not valid
on a note that shows 1,000 lovelace. The public keys of an asset form its
keyset, and a wallet gets them with the `keys` request.

## 7. Security notes

1. The security of the signatures depends on the discrete log problem in
   Ristretto255.
2. With `r` and the note, a person can calculate `B'` and link the note to its
   signature request. A note carries `r` for holder checks. Thus, a wallet
   removes `r` from each note that it sends to the node.
3. A wallet must select each `r` and each note nonce with a secure random
   number generator.
