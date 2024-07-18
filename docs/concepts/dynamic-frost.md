# Dynamic-FROST: Schnorr Threshold Signatures with a Flexible Committee

## Abstract

This document details the Dynamic-FROST (D-FROST) protocol, which combines Flexible Round-Optimized Schnorr Threshold (FROST) signatures with CHUrn-Robust Proactive (CHURP) secret sharing. D-FROST enables changes in both the committee of participants and the threshold value without relying on a trusted third party. This technical specification targets cryptographers, blockchain developers, and security researchers interested in implementing or analyzing dynamic threshold signature schemes. We outline the protocol's structure, security properties, and potential applications.

## 1. Introduction

Threshold signatures allow a predefined subset of participants to generate a valid, aggregated signature. Traditional threshold signature schemes assume fixed parameters for the number of participants and the threshold value. However, practical applications, such as blockchain wallets and consensus algorithms, often require the flexibility to update these parameters.

D-FROST addresses this need by combining two existing protocols:

1. FROST (Flexible Round-Optimized Schnorr Threshold) signatures [1]
2. CHURP (CHUrn-Robust Proactive) secret sharing [2]

This combination supports dynamic committee and threshold changes while preserving essential security properties.

## 2. Background

### 2.1 FROST Overview

FROST is a Schnorr threshold signature scheme that enables a group of $t$ out of $n$ participants to collaboratively sign a message $m$ using a single group public key. The scheme operates in a group $G$ of prime order $q$ where the Decisional Diffie-Hellman problem is computationally hard.

Key features of FROST include:

- Use of Pedersen's Distributed Key Generation (DKG) algorithm with additional proof of knowledge for $a_{i0}$ to prevent rogue-key attacks
- Asynchronous signing operations
- Efficient communication with a two-round signing protocol
- Single-round signing with preprocessing
- Ability to abort in the presence of a misbehaving participant, improving efficiency in honest settings

### 2.2 CHURP Overview

CHURP is a dynamic proactive secret sharing scheme that allows changes in both the committee composition and the threshold without altering the group secret. It uses a bivariate polynomial $B(x, y)$ to distribute secret shares, ensuring the secret remains intact during transitions.

Key features of CHURP include:

- Support for dynamic committee changes
- Proactive security through periodic share refreshing
- Efficient handoff protocol for committee transitions
- Use of "dimension-switching" technique during handoffs to temporarily switch to a (2t,n)-sharing, tolerating up to 2t compromised shares

## 3. D-FROST Protocol

### 3.1 Setup Phase

1. Initial Committee Selection:
   - Select an initial committee $C = \{P_i\}_{i \in [n]}$ and a threshold $t$.
   - Distribute private/public key pairs to all participants.

2. Key Generation:
   - Execute FROST's key generation scheme to create $(t, n)$-shares of the secret $s$.
   - Use Pedersen's DKG with additional proof of knowledge for $a_{i0}$ to prevent rogue-key attacks.
   - Each participant $P_i$ generates a polynomial $f_i(x) = \sum_{j=0}^{t-1} a_{ij}x^j$ and broadcasts commitments $g^{a_{ij}}$.
   - Participants exchange encrypted shares $f_i(k)$ and verify them using the commitments.
   - The group public key is computed as $Y = \prod_{i=1}^n g^{a_{i0}}$.

### 3.2 Transition to Steady State

The Opt-SteadyState process from CHURP involves the following steps:

1. Generate a random bivariate polynomial $B(x, y)$ such that:
   $$\text{deg}B(x, y) = \langle t-1, 2t-2 \rangle \text{ and } B(0, 0) = s$$

2. Compute KZG commitments $C_{B(x, y)}$ and share $(B(i, j), W'_{B(i, j)})$ with designated nodes $U'$.

3. Interpolate to construct $B(x, j)$, publish $C_{B(x, j)}$ and $W_{B(i, j)}$.

4. Verify correctness and ensure $B(0, 0) = s$.

### 3.3 Epoch Phases

1. CHURP Handoff:
   - Perform the CHURP handoff to update committee members and/or threshold while maintaining the secret $s$.
   - Use dimension-switching technique to temporarily switch to a (2t,n)-sharing during handoff:
     a. Old committee shares $B(i, y)$ with new committee.
     b. New committee reconstructs $B(x, j)$, switching to (2t,n)-sharing.
     c. Generate new bivariate polynomial $Q(x, y)$ with $Q(0, 0) = 0$.
     d. Update shares: $B'(x, y) = B(x, y) + Q(x, y)$.
     e. Switch back to (t,n)-sharing by distributing $B'(i, y)$ to new committee.

2. FROST Preprocessing:
   - Execute FROST's preprocessing stage to generate commitments for signing.
   - Each participant $P_i$ generates and publishes $\pi$ commitment pairs $(D_{ij}, E_{ij})$ for future use in signing.
   - $D_{ij} = g^{d_{ij}}$ and $E_{ij} = g^{e_{ij}}$ where $d_{ij}, e_{ij}$ are random nonces.

3. FROST Signing:
   - Perform the signing operation using FROST's protocol:
     a. Select a subset of $t$ participants and unused commitments $(D_i, E_i)$.
     b. Compute binding values $\rho_i = H_1(i, m, B)$ where $B$ is the set of participants and commitments.
     c. Calculate $R_i = D_i \cdot (E_i)^{\rho_i}$ and group commitment $R = \prod_{i} R_i$.
     d. Compute challenge $c = H_2(R, Y, m)$.
     e. Each participant computes response $z_i = d_i + e_i\rho_i + \lambda_is_ic$.
     f. Aggregate responses to get group signature $(R, z)$.

   The signing operation in FROST is designed to be asynchronous, allowing participants to compute their partial signatures independently. This asynchronous nature improves efficiency and reduces coordination overhead.

## 4. Security Analysis

### 4.1 Adversarial Model

D-FROST tolerates up to $t-1$ corrupted nodes in each committee (old and new) during handoffs. The adversary is computationally bounded and can adaptively corrupt nodes, but cannot exceed the threshold in any single committee.

### 4.2 Secrecy and Integrity

- Secrecy: We prove that an adversary corrupting at most $t-1$ nodes learns no information about the secret $s$.
- Integrity: We ensure that the polynomial $B(x, y)$ constructed during the steady state transition is correct and consistent.

### 4.3 Proactive Security

D-FROST inherits CHURP's proactive security properties:
- Periodic share refreshing prevents the accumulation of compromised shares over time.
- The dimension-switching technique during handoffs provides security against an adversary controlling up to 2t nodes across old and new committees.

### 4.4 EUF-CMA Security

D-FROST signatures are proven to be existentially unforgeable under chosen-message attack (EUF-CMA). This security property is inherited from both FROST and CHURP, ensuring that the combined protocol maintains a high level of security.

The security proof demonstrates that if the discrete logarithm problem in $G$ is $(\tau', \epsilon')$-hard, then D-FROST is $(\tau, n_h, n_p, n_s, \epsilon)$-secure, where:

- $n_h$ is the number of queries made to the random oracle
- $n_p$ is the number of allowed preprocess queries
- $n_s$ is the number of allowed signing queries

The specific security bounds are:

$$\epsilon' \leq \frac{\epsilon^2}{2n_h + (\pi + 1)n_p + 1}$$

$$\tau' = 4\tau + (30\pi n_p + (4t - 2)n_s + (n + t - 1)t + 6) \cdot t_{\exp} + O(\pi n_p + n_s + n_h + 1)$$

where $t_{\exp}$ is the time of an exponentiation in $G$, and $\pi$ is the preprocess batch size.

## 5. Implementation and Performance

D-FROST has been implemented using the Ristretto group over curve25519 for elliptic curve operations. The implementation achieves efficient performance, with signing operations completing in constant time regardless of the number of participants.

Performance measurements show that D-FROST outperforms previous dynamic threshold signature schemes:

- Communication Complexity:
  - Optimistic case (no node failures): $O(n)$ on-chain and $O(n^2)$ off-chain
  - Pessimistic case (node failures detected): $O(n^2)$ on-chain
- Computation Complexity:
  - Key Generation: $O(n^2)$ exponentiations
  - Signing: $O(t)$ exponentiations per participant

Compared to previous schemes like MPSS [4], D-FROST achieves significantly lower communication complexity, especially in the off-chain case (O(n^2) vs O(n^4)).

## 6. Practical Considerations

- Network Assumptions: D-FROST combines synchronous and asynchronous elements:
  - The CHURP handoff process assumes a synchronous network model for optimal performance. This ensures that all participants have a consistent view of the committee state during transitions.
  - The FROST signing process is designed to be asynchronous, allowing for more flexible and efficient signature generation.
  - In cases where the network cannot guarantee synchronicity, D-FROST can fall back to on-chain communication, using the blockchain as a synchronization mechanism at the cost of increased latency and transaction fees.

- Synchronicity Trade-offs:
  - Synchronous operations provide stronger security guarantees but may be less efficient in real-world networks with varying latencies.
  - Asynchronous operations offer better performance and flexibility but may be more vulnerable to certain types of attacks, such as race conditions.
  - D-FROST's hybrid approach aims to balance security and efficiency by using synchronous operations for critical state transitions (handoffs) and asynchronous operations for frequent tasks (signing).

- Fallback Mechanisms:
  - In case of detected network asynchrony during a handoff, D-FROST includes fallback mechanisms to ensure protocol completion:
    1. Participants can switch to using the blockchain as a broadcast channel.
    2. The protocol can temporarily increase timeouts to accommodate network delays.
    3. In extreme cases, the handoff can be aborted and reattempted when network conditions improve.

- Blockchain Integration: 
  - The protocol leverages the blockchain not only for dispute resolution but also as a reliable synchronization mechanism when needed.
  - On-chain operations provide a consistent view to all participants, which is crucial during committee transitions and for resolving any inconsistencies that may arise due to network asynchrony.

- Key Management: Participants must securely store their long-term secret shares and delete ephemeral values after use to maintain security.

## 7. Conclusion

D-FROST extends FROST's capabilities by incorporating dynamic updates through CHURP. This protocol maintains security while allowing flexible committee and threshold changes, making it a robust solution for modern cryptographic applications.

The combination of FROST's efficient Schnorr-based signatures with CHURP's dynamic proactive secret sharing creates a versatile tool for scenarios where the set of participants or security requirements may change over time.

By providing a secure and flexible threshold signature scheme, D-FROST opens new possibilities for decentralized systems and cryptographic protocols in an ever-evolving digital landscape.

## References

[1] Komlo, C., & Goldberg, I. (2020). FROST: Flexible Round-Optimized Schnorr Threshold Signatures.

[2] Maram, S. K. D., Zhang, F., Wang, L., Low, A., Zhang, Y., Juels, A., & Song, D. (2019). CHURP: Dynamic-Committee Proactive Secret Sharing. In Proceedings of the 2019 ACM SIGSAC Conference on Computer and Communications Security (pp. 2369-2386).

[3] Cimatti, A., De Sclavis, F., Galano, G., Giammusso, S., Iezzi, M., Muci, A., Nardelli, M., & Pedicini, M. (2023). Dynamic-FROST: Schnorr Threshold Signatures with a Flexible Committee.

[4] Schultz, D. A., Liskov, B., & Liskov, M. (2008). Mobile proactive secret sharing. In Proceedings of the twenty-seventh ACM symposium on Principles of distributed computing (pp. 458-458).