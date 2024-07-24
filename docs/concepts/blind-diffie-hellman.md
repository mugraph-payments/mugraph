# Blind Diffie-Hellman Key Exchange (BDHKE)

## Introduction

This document explains the Blind Diffie-Hellman Key Exchange (BDHKE) protocol, a cryptographic method that allows two parties to establish a shared secret key without revealing their individual private keys. BDHKE is particularly useful in scenarios where privacy and anonymity are crucial, such as in digital cash systems or anonymous authentication protocols.

## Historical Context

The BDHKE protocol was unearthed from a 1996 cypherpunk mailing list post by David Wagner. It was devised as an alternative to RSA blinding to circumvent the now-expired patent by David Chaum. An implementation of this protocol, called Lucre, demonstrates its practical application.

## Overview

BDHKE is an extension of the traditional Diffie-Hellman key exchange, incorporating a blinding factor to enhance privacy. The protocol involves two main parties:

1. Alice: The user who initiates the key exchange and wants to obtain a blindly signed value.
2. Bob: The signer (often referred to as the "mint" in digital cash systems) who performs the blind signing operation.

The goal is for Bob to sign Alice's message without knowing its content, while Alice can later prove that the signature came from Bob.

## Protocol Steps

### 1. Initial Setup

Alice and Bob agree on a common elliptic curve group with a generator point $G$.

### 2. Key Generation

Alice generates a private key $a$ and computes the corresponding public key $A$:

$$
\begin{align}
A &= a \cdot G
\end{align}
$$

Alice sends $A$ to Bob.

Bob generates a private key $k$ and computes the corresponding public key $K$:

$$
\begin{align}
K &= k \cdot G
\end{align}
$$

Bob makes $K$ publicly available.

### 3. Blinding

Alice performs the following steps:

a. Choose a secret message $x$. In ecash protocols, this message is remembered to prevent double spending.
b. Compute $Y = \text{hash\_to\_curve}(x)$, where $\text{hash\_to\_curve}$ is a function that maps the secret to a point on the elliptic curve.
c. Generate a random blinding factor $r$.
d. Compute the blinded point $B'$:

$$
\begin{align}
B' &= Y + r \cdot G
\end{align}
$$

Alice sends $B'$ to Bob.

### 4. Signing

Bob receives $B'$ and computes the blinded signature $C'$:

$$
\begin{align}
C' &= k \cdot B'
\end{align}
$$

Bob sends $C'$ back to Alice.

### 5. Unblinding

Alice unblinds the signature by subtracting $r \cdot K$ from $C'$:

$$
\begin{align}
C &= C' - r \cdot K \\
  &= k \cdot B' - r \cdot K \\
  &= k \cdot (Y + r \cdot G) - r \cdot (k \cdot G) \\
  &= k \cdot Y + k \cdot r \cdot G - r \cdot k \cdot G \\
  &= k \cdot Y
\end{align}
$$

### 6. Verification

To verify the signature, Alice (or any verifier) can check if:

$$C = k \cdot \text{hash\_to\_curve}(x)$$

If this equality holds, it proves that $C$ originated from Bob's private key $k$, without Bob knowing the original message $x$.

## Security Considerations

1. The security of BDHKE relies on the hardness of the Elliptic Curve Discrete Logarithm Problem (ECDLP).
2. The blinding factor $r$ must be kept secret by Alice to maintain the blindness property.
3. The $\text{hash\_to\_curve}$ function should be carefully chosen to ensure it maps uniformly to the elliptic curve and does not introduce vulnerabilities.

## Additional Security Measure: Discrete Log Equality Proof

To prevent potential attacks where Alice might not correctly generate $C'$, an additional step can be included:

Alice provides a Discrete Log Equality Proof (DLEQ) to demonstrate that the $a$ in $A = a \cdot G$ is equal to the $a$ in $C' = a \cdot B'$. This proof can be implemented using a Schnorr signature as follows:

1. Alice generates a random nonce $r$.
2. Alice computes:
   $$
   \begin{align}
   R_1 &= r \cdot G \\
   R_2 &= r \cdot B' \\
   e &= \text{hash}(R_1, R_2, A, C') \\
   s &= r + e \cdot a
   \end{align}
   $$
3. Alice sends $e$ and $s$ to Bob.
4. Bob verifies the proof by checking:
   $$
   \begin{align}
   R_1 &= s \cdot G - e \cdot A \\
   R_2 &= s \cdot B' - e \cdot C' \\
   e &= \text{hash}(R_1, R_2, A, C')
   \end{align}
   $$

If the verification passes, Bob can be confident that Alice correctly generated $C'$.

## Advantages and Disadvantages

### Advantages
1. Threshold setting: This scheme is relatively straightforward to perform in a threshold setting, as it only requires curve multiplication.
2. Patent-free: It was designed to avoid the now-expired RSA blinding patent.

### Disadvantages
1. Complex validation: Validation is more involved than simply checking a signature, as it requires repeating the Diffie-Hellman Key Exchange.

## Applications

BDHKE finds applications in various cryptographic protocols, including:

1. Digital cash systems
2. Anonymous credential systems
3. Privacy-preserving authentication protocols
4. Secure voting systems

## Conclusion

Blind Diffie-Hellman Key Exchange provides a powerful tool for creating cryptographic protocols that require both the establishment of shared secrets and the preservation of privacy. By combining the properties of Diffie-Hellman key exchange with blinding techniques, BDHKE enables a wide range of applications in the field of privacy-enhancing technologies.

## Acknowledgments

Thanks to Eric Sirion, Andrew Poelstra, and Adam Gibson for their helpful comments on the original protocol description.