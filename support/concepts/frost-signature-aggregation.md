# FROST: Flexible Round-Optimized Schnorr Threshold Signatures

FROST (Flexible Round-Optimized Schnorr Threshold Signatures) is an advanced threshold signature scheme developed by Chelsea Komlo and Ian Goldberg. It enables a group of participants to collaboratively generate Schnorr signatures, significantly improving upon previous threshold Schnorr signature schemes by reducing network overhead and allowing unrestricted parallelism of signing operations.

## Mathematical Foundations

FROST operates in a group $G$ of prime order $q$ where the Decisional Diffie-Hellman problem is hard. Let $g$ be a generator of $G$, and $H$ be a cryptographic hash function mapping to $\\mathbb{Z}\_q^\*$. The scheme uses two hash functions:

1. $H_1: {0,1}^\* \\rightarrow \\mathbb{Z}\_q^\*$ for generating binding values
1. $H_2: G \\times G \\times {0,1}^\* \\rightarrow \\mathbb{Z}\_q^\*$ for generating the challenge

## Protocol Components

FROST consists of three main components:

1. Distributed Key Generation (DKG)
1. Preprocessing Stage (optional)
1. Signing Protocol

### Distributed Key Generation

FROST uses a variant of Pedersen's Distributed Key Generation (DKG) protocol to generate long-lived key shares:

1. Each participant $P_i$ generates a random polynomial $f_i(x)$ of degree $t-1$:

   $$f_i(x) = a\_{i0} + a\_{i1}x + \\cdots + a\_{i(t-1)}x^{t-1}$$

1. $P_i$ computes secret shares $s\_{ij} = f_i(j)$ for each other participant $P_j$.

1. $P_i$ broadcasts commitments to the coefficients of $f_i(x)$: $C\_{ik} = g^{a\_{ik}}$ for $k = 0$ to $t-1$.

1. $P_i$ sends $s\_{ij}$ securely to each $P_j$.

1. Participants verify the received shares against the broadcasted commitments.

1. Each participant computes their final secret share as $s_i = \\sum_j s\_{ji}$.

The group's public key $Y$ is computed as $Y = \\prod_i Y_i$, where $Y_i = g^{s_i}$.

Additionally, each participant provides a zero-knowledge proof of knowledge of their secret $a\_{i0}$ during the key generation stage to prevent rogue key attacks when $t \\geq n/2$.

### Preprocessing Stage

The optional preprocessing stage allows participants to generate and publish commitment shares in advance:

1. Each participant $P_i$ generates $\\pi$ pairs of single-use nonces $(d\_{ij}, e\_{ij})$ and corresponding commitments $(D\_{ij} = g^{d\_{ij}}, E\_{ij} = g^{e\_{ij}})$.
1. $P_i$ publishes the list of commitment pairs $L_i = \\langle(D\_{ij}, E\_{ij})\\rangle\_{j=1}^{\\pi}$.

### Signing Protocol

The signing protocol can be performed in a single round if preprocessing was completed, or in two rounds otherwise:

1. A signature aggregator (SA) selects $\\alpha$ participants ($t \\leq \\alpha \\leq n$) and available commitments.
1. SA sends the message $m$ and set of commitments $B$ to each signing participant.
1. Participants compute binding values $\\rho_i = H_1(i, m, B)$.
1. Participants calculate their partial commitment $R_i = D_i \\cdot (E_i)^{\\rho_i}$ and the group commitment $R = \\prod\_{i\\in S} R_i$.
1. Participants compute the challenge $c = H_2(R, Y, m)$.
1. Participants generate their signature share $z_i = d_i + (e_i \\cdot \\rho_i) + \\lambda_i \\cdot s_i \\cdot c$, where $\\lambda_i$ are Lagrange coefficients.
1. SA verifies and aggregates the signature shares to produce the final signature $\\sigma = (R, z)$, where $z = \\sum\_{i\\in S} z_i$.

## Security Analysis

FROST provides security against chosen-message attacks, assuming the discrete logarithm problem is hard in the underlying group and the adversary controls fewer than $t$ participants. The security proof uses a forking lemma technique to reduce the security of FROST to the discrete logarithm problem.

### Theorem (Informal)

If the discrete logarithm problem in $G$ is $(\\tau', \\epsilon')$-hard, then FROST with $n$ signing participants, a threshold of $t$, and a preprocess batch size of $\\pi$ is $(\\tau, n_h, n_p, n_s, \\epsilon)$-secure, where:

- $\\epsilon' \\leq \\frac{\\epsilon^2}{2n_h + (\\pi + 1)n_p + 1}$
- $\\tau' = 4\\tau + (30\\pi n_p + (4t - 2)n_s + (n + t - 1)t + 6) \\cdot t\_{exp} + O(\\pi n_p + n_s + n_h + 1)$

Here, $n_h$ is the number of queries to the random oracle, $n_p$ is the number of allowed preprocess queries, $n_s$ is the number of allowed signing queries, and $t\_{exp}$ is the time of an exponentiation in $G$.

Key security properties include:

1. Threshold security: The scheme remains secure as long as fewer than $t$ participants are compromised.
1. Binding technique: Signature shares are bound to the specific message, set of signers, and commitments, preventing attacks that combine shares across different signing operations.
1. Non-interactive nonce generation: The use of additive secret sharing and share conversion allows efficient, non-interactive generation of nonces for each signature.
1. Protection against Wagner's algorithm attack: FROST's binding technique prevents attacks that exploit parallel signing sessions.

## Performance and Efficiency

FROST significantly improves upon previous threshold Schnorr signature schemes:

1. Reduced network rounds: FROST requires only one or two rounds of communication, compared to three or more rounds in schemes like those by Stinson and Strobl or Gennaro et al.
1. Unrestricted parallelism: Unlike some previous schemes, FROST allows unlimited concurrent signing operations without compromising security.
1. Flexible threshold: FROST allows any $t$ out of $n$ participants to sign, rather than requiring all $n$ participants to be involved in every signature.
1. Efficient preprocessing: The preprocessing stage in FROST is more efficient and flexible than in some previous schemes, allowing preprocessed values to be used with different signing coalitions.
1. Asynchronous signing: The separation of preprocessing and signing stages allows for asynchronous signing operations.

## Comparison with Other Threshold Signature Schemes

FROST differs from many previous threshold signature schemes in its approach to robustness:

1. Non-robust design: FROST trades off robustness for efficiency by aborting the protocol if misbehavior is detected, rather than attempting to complete the signature with honest participants only. This design choice allows for more efficient operations in practice.
1. Security threshold: Unlike robust designs which can at best provide security for $t \\leq n/2$, FROST is secure as long as the adversary controls fewer than $t$ participants.
1. Efficiency vs. robustness: FROST's design allows for more efficient signing operations compared to robust schemes, making it suitable for many real-world applications where network efficiency is crucial.

## Practical Considerations

When implementing FROST, consider the following:

1. Signature aggregator role: While not strictly necessary, using a signature aggregator can reduce communication overhead. The aggregator can be any participant or an external party with knowledge of public key shares.
1. Commitment storage: Implementations must securely store and manage unused commitments generated during preprocessing. Participants should delete used nonces to prevent accidental reuse. Proper storage and management of these commitments is crucial for maintaining the security of the system.
1. Participant authentication: The protocol assumes that participants can be reliably identified within the signing group.
1. Network reliability: FROST assumes a reliable network channel for message delivery between participants.
1. Aborting on misbehavior: FROST trades off robustness for efficiency by aborting the protocol if misbehavior is detected, rather than attempting to complete the signature with honest participants only.
1. Consistent view of commitments: Implementations must ensure all participants have a consistent view of commitment values during key generation. This is critical for maintaining the security and correctness of the protocol. Techniques such as using a centralized server or additional protocol rounds to compare received commitment values can be employed to achieve this consistency.

## Applications

FROST is suitable for a wide range of practical applications, including:

1. Cryptocurrency wallets: Securing high-value transactions with distributed trust.
1. Certificate authorities: Distributing trust in PKI systems.
1. Secure multiparty computation: As a building block for more complex protocols.
1. Distributed systems: For consensus mechanisms or authentication in decentralized networks.

## Conclusion

FROST represents a significant advancement in threshold Schnorr signature schemes, offering improved efficiency and flexibility while maintaining strong security properties. Its design makes it suitable for a wide range of practical applications, particularly in scenarios where network efficiency is crucial or where only a subset of participants need to be available for signing operations. As distributed systems and cryptographic protocols continue to evolve, FROST provides a powerful tool for building secure and efficient threshold signing mechanisms.
