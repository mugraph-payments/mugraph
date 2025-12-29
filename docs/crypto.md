# Mugraph Cryptography

## Blind Diffie-Hellman Key Exchange (BDHKE)

Blind Diffie-Hellman Key Exchange (BDHKE) protocol is a cryptographic method that allows two parties to establish a shared secret key without revealing their individual private keys. BDHKE is used on Mugraph to guarantee group concealing, making the server oblivious to the identity of the note owner, severing the connection between inputs from one transaction to the next. The BDHKE protocol was described in a 1996 cypherpunk mailing list post by David Wagner. It was devised as an alternative to RSA blinding to circumvent the now-expired patent by David Chaum.

### Overview

BDHKE is an extension of the traditional Diffie-Hellman key exchange, incorporating a blinding factor to make it blind (meaning the message can be verified without knowning which original message generated it). The protocol involves two main parties:

1. Alice: The user who initiates the key exchange and wants to obtain a blindly signed value.
2. Bob: The signer (often referred to as the "mint" in digital cash systems) who performs the blind signing operation.

The goal is for Bob to sign Alice's message without knowing its content, while Alice can later prove that the signature came from Bob.

### Protocol Steps

#### 1. Initial Setup

Alice and Bob agree on a common elliptic curve group with a generator point $G$.

#### 2. Key Generation

Alice generates a private key $a$ and computes the corresponding public key $A$:

$$
\begin{aligned}
A &= a \cdot G
\end{aligned}
$$

Alice sends $A$ to Bob.

Bob generates a private key $k$ and computes the corresponding public key $K$:

$$
\begin{aligned}
K &= k \cdot G
\end{aligned}
$$

Bob makes $K$ publicly available.

#### 3. Blinding

Alice performs the following steps:

a. Choose a secret message $x$. In ecash protocols, this message is remembered to prevent double spending.
b. Compute $Y = H(x)$, where $H$ is a function that maps the secret to a point on the elliptic curve (hash to curve).
c. Generate a random blinding factor $r$.
d. Compute the blinded point $B'$:

$$
\begin{aligned}
B' &= Y + r \cdot G
\end{aligned}
$$

Alice sends $B'$ to Bob.

#### 4. Signing

Bob receives $B'$ and computes the blinded signature $C'$:

$$
\begin{aligned}
C' &= k \cdot B'
\end{aligned}
$$

Bob sends $C'$ back to Alice.

#### 5. Unblinding

Alice unblinds the signature by subtracting $r \cdot K$ from $C'$:

$$
\begin{aligned}
C &= C' - r \cdot K \\
  &= k \cdot B' - r \cdot K \\
  &= k \cdot (Y + r \cdot G) - r \cdot (k \cdot G) \\
  &= k \cdot Y + k \cdot r \cdot G - r \cdot k \cdot G \\
  &= k \cdot Y
\end{aligned}
$$

#### 6. Verification

To verify the signature, Alice (or any verifier) can check if:

$$C = K \cdot H(x)$$

If this equality holds, it proves that $C$ originated from Bob's private key $k$, without Bob knowing the original message $x$.

### Security Considerations

1. The security of BDHKE relies on the hardness of the Elliptic Curve Discrete Logarithm Problem (ECDLP).
2. The blinding factor $r$ must be kept secret by Alice to maintain the blindness property.
3. The $H$ function should be carefully chosen to ensure it maps uniformly to the elliptic curve and does not introduce vulnerabilities.

## Discrete Log Equality Proof (DLEQ Proof)

To prevent potential attacks where Bob might not correctly generate $C'$, an additional step is included:

Bob provides a Discrete Log Equality Proof (DLEQ) to demonstrate that the $k$ in $K = k \cdot G$ is the same $k$ used in $C' = k \cdot B'$. This proof can be implemented using a Schnorr signature as follows:

1. Bob generates a random nonce $r$.
2. Bob computes:

$$
\begin{aligned}
R_1 &= r \cdot G \\
R_2 &= r \cdot B' \\
e &= \text{hash}(R_1, R_2, K, C') \\
s &= r + e \cdot k
\end{aligned}
$$

3. Bob sends $e$ and $s$ to Alice.
4. Alice verifies the proof by checking:

$$
\begin{aligned}
R_1 &= s \cdot G - e \cdot K \\
R_2 &= s \cdot B' - e \cdot C' \\
e &= \text{hash}(R_1, R_2, K, C')
\end{aligned}
$$

If the verification passes, Alice can be confident that Bob correctly generated $C'$.
