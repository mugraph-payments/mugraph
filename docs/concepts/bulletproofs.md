# Bulletproofs

## Introduction

Bulletproofs are a non-interactive zero-knowledge proof protocol that enables very short proofs for confidential transactions and other applications. Introduced by Bünz et al. in 2018, Bulletproofs offer several key advantages:

- Logarithmic proof size in relation to the statement being proved
- No requirement for a trusted setup
- Reliance solely on the discrete logarithm assumption
- Efficient batch verification

Bulletproofs are particularly effective for range proofs on committed values. They can prove that a committed value lies in a given range using only $2\log_2(n) + 9$ group and field elements, where $n$ represents the bit-length of the range.

## Mathematical Background

This section outlines the key mathematical concepts underlying Bulletproofs.

### Notation

- $\mathbb{G}$ denotes a cyclic group of prime order $p$
- Bold lowercase letters represent vectors, for example, $\mathbf{a} \in \mathbb{F}^n$
- $\langle \mathbf{a}, \mathbf{b} \rangle = \sum_{i=1}^n a_i \cdot b_i$ represents the inner product
- $\mathbf{a} \circ \mathbf{b} = (a_1 \cdot b_1, \ldots, a_n \cdot b_n)$ represents the Hadamard product
- For $k \in \mathbb{Z}_p^*$, $\mathbf{k}^n = (1, k, k^2, \ldots, k^{n-1})$

### Pedersen Commitments

A Pedersen commitment to $x \in \mathbb{Z}_p$ using randomness $r \in \mathbb{Z}_p$ is defined as:

$\text{Com}(x; r) = g^x h^r$

where $g, h \in \mathbb{G}$ are independent generators. Pedersen commitments are perfectly hiding and computationally binding under the discrete logarithm assumption.

### Discrete Log Relation Assumption

For all probabilistic polynomial-time (PPT) adversaries $\mathcal{A}$ and for all $n \geq 2$, there exists a negligible function $\mu(\lambda)$ such that:

$P[\mathbb{G} = \text{Setup}(1^\lambda), g_1, \ldots, g_n \stackrel{\$}{\leftarrow} \mathbb{G}; a_1, \ldots, a_n \in \mathbb{Z}_p \leftarrow \mathcal{A}(\mathbb{G}, g_1, \ldots, g_n) : \exists a_i \neq 0 \wedge \prod_{i=1}^n g_i^{a_i} = 1] \leq \mu(\lambda)$

## Inner Product Argument

The core of Bulletproofs is an efficient inner product argument. For generators $\mathbf{g}, \mathbf{h} \in \mathbb{G}^n$, $u \in \mathbb{G}$, and $P \in \mathbb{G}$, the prover convinces the verifier that it knows vectors $\mathbf{a}, \mathbf{b} \in \mathbb{Z}_p^n$ such that:

$P = \mathbf{g}^{\mathbf{a}} \mathbf{h}^{\mathbf{b}} \cdot u^{\langle \mathbf{a}, \mathbf{b} \rangle}$

### Protocol

1. The prover computes:
   - $L = \mathbf{g}_{[n':]}^{\mathbf{a}_{[:n']}} \mathbf{h}_{[:n']}^{\mathbf{b}_{[n':]}} u^{\langle \mathbf{a}_{[:n']}, \mathbf{b}_{[n':]} \rangle}$
   - $R = \mathbf{g}_{[:n']}^{\mathbf{a}_{[n':]}} \mathbf{h}_{[n':]}^{\mathbf{b}_{[:n']}} u^{\langle \mathbf{a}_{[n':]}, \mathbf{b}_{[:n']} \rangle}$

   where $n' = n/2$

2. The prover sends $L, R$ to the verifier.

3. The verifier sends a random challenge $x \stackrel{\$}{\leftarrow} \mathbb{Z}_p^*$.

4. Both parties compute:
   - $\mathbf{g}' = \mathbf{g}_{[:n']}^{x^{-1}} \circ \mathbf{g}_{[n':]}^x$
   - $\mathbf{h}' = \mathbf{h}_{[:n']}^x \circ \mathbf{h}_{[n':]}^{x^{-1}}$
   - $P' = L^{x^2} \cdot P \cdot R^{x^{-2}}$

5. The prover computes:
   - $\mathbf{a}' = \mathbf{a}_{[:n']} \cdot x + \mathbf{a}_{[n':]} \cdot x^{-1}$
   - $\mathbf{b}' = \mathbf{b}_{[:n']} \cdot x^{-1} + \mathbf{b}_{[n':]} \cdot x$

6. The protocol is recursively run on input $(\mathbf{g}', \mathbf{h}', u, P'; \mathbf{a}', \mathbf{b}')$.

The protocol terminates when $n = 1$, at which point the prover sends $a, b \in \mathbb{Z}_p$ and the verifier checks if $P = g^a h^b u^{ab}$.

## Range Proof Protocol

For a Pedersen commitment $V = g^v h^\gamma$, the range proof protocol proves that $v \in [0, 2^n - 1]$ without revealing $v$.

### Protocol

Prover's input: $(g, h, V, n; v, \gamma)$
Verifier's input: $(g, h, V, n)$

1. The prover computes:
   - $\mathbf{a}_L \in \{0,1\}^n$ such that $\langle \mathbf{a}_L, \mathbf{2}^n \rangle = v$
   - $\mathbf{a}_R = \mathbf{a}_L - \mathbf{1}^n$
   - $\alpha \stackrel{\$}{\leftarrow} \mathbb{Z}_p$
   - $A = h^\alpha \mathbf{g}^{\mathbf{a}_L} \mathbf{h}^{\mathbf{a}_R}$
   - $\mathbf{s}_L, \mathbf{s}_R \stackrel{\$}{\leftarrow} \mathbb{Z}_p^n$, $\rho \stackrel{\$}{\leftarrow} \mathbb{Z}_p$
   - $S = h^\rho \mathbf{g}^{\mathbf{s}_L} \mathbf{h}^{\mathbf{s}_R}$

2. The prover sends $A, S$ to the verifier.

3. The verifier sends random challenges $y, z \stackrel{\$}{\leftarrow} \mathbb{Z}_p^*$.

4. Both parties compute:
   $\delta(y,z) = (z - z^2)\langle \mathbf{1}^n, \mathbf{y}^n \rangle - z^3 \langle \mathbf{1}^n, \mathbf{2}^n \rangle$

5. The prover computes:
   - $l(X) = (\mathbf{a}_L - z\mathbf{1}^n) + \mathbf{s}_L X$
   - $r(X) = \mathbf{y}^n \circ (\mathbf{a}_R + z\mathbf{1}^n + \mathbf{s}_R X) + z^2 \mathbf{2}^n$
   - $t(X) = \langle l(X), r(X) \rangle = t_0 + t_1 X + t_2 X^2$
   - $\tau_1, \tau_2 \stackrel{\$}{\leftarrow} \mathbb{Z}_p$
   - $T_1 = g^{t_1} h^{\tau_1}, T_2 = g^{t_2} h^{\tau_2}$

6. The prover sends $T_1, T_2$ to the verifier.

7. The verifier sends a random challenge $x \stackrel{\$}{\leftarrow} \mathbb{Z}_p^*$.

8. The prover computes:
   - $\tau_x = \tau_2 x^2 + \tau_1 x + z^2 \gamma$
   - $\mu = \alpha + \rho x$
   - $\hat{t} = \langle l(x), r(x) \rangle$
   - $l = l(x), r = r(x)$

9. The prover sends $\tau_x, \mu, \hat{t}, l, r$ to the verifier.

10. The verifier checks:
    - $g^{\hat{t}} h^{\tau_x} \stackrel{?}{=} V^{z^2} g^{\delta(y,z)} T_1^x T_2^{x^2}$
    - $h^\mu g^l h^r \stackrel{?}{=} A S^x g^{-z} h^{z\mathbf{y}^n + z^2\mathbf{2}^n}$
    - $\hat{t} \stackrel{?}{=} \langle l, r \rangle$

### Logarithmic-Size Proof

To achieve logarithmic proof size, replace steps 9-10 with an inner product argument for $\langle l, r \rangle = \hat{t}$.

## Aggregating Multiple Range Proofs

For $m$ commitments $V_j = g^{v_j} h^{\gamma_j}$, $j \in [1,m]$, we can prove that all $v_j \in [0, 2^n - 1]$ simultaneously.

The protocol is similar to the single range proof, with the following modifications:

1. Use vectors of length $n \cdot m$: $\mathbf{a}_L, \mathbf{a}_R, \mathbf{s}_L, \mathbf{s}_R \in \mathbb{Z}_p^{n \cdot m}$

2. Modify $l(X)$ and $r(X)$:
   - $l(X) = (\mathbf{a}_L - z \cdot \mathbf{1}^{n \cdot m}) + \mathbf{s}_L \cdot X$
   - $r(X) = \mathbf{y}^{n \cdot m} \circ (\mathbf{a}_R + z \cdot \mathbf{1}^{n \cdot m} + \mathbf{s}_R \cdot X) + \sum_{j=1}^m z^{1+j} \cdot (\mathbf{0}^{(j-1) \cdot n} \| \mathbf{2}^n \| \mathbf{0}^{(m-j) \cdot n})$

3. Update $\tau_x$ and $\delta(y,z)$:
   - $\tau_x = \tau_1 \cdot x + \tau_2 \cdot x^2 + \sum_{j=1}^m z^{1+j} \cdot \gamma_j$
   - $\delta(y,z) = (z - z^2) \cdot \langle \mathbf{1}^{n \cdot m}, \mathbf{y}^{n \cdot m} \rangle - \sum_{j=1}^m z^{j+2} \cdot \langle \mathbf{1}^n, \mathbf{2}^n \rangle$

The rest of the protocol remains the same.

## Applications

Bulletproofs have several important applications:

1. Confidential Transactions: Prove that transaction amounts are positive without revealing the amounts.

2. Mimblewimble: Enable efficient range proofs in Mimblewimble-based cryptocurrencies.

3. Zero-Knowledge Proofs for Smart Contracts: Allow privacy-preserving smart contracts with efficient verification.

4. Provisions: Prove solvency of cryptocurrency exchanges without revealing sensitive information.

5. Verifiable Shuffles: Create efficient verifiable shuffles for voting systems and mixnets.

## Performance

Bulletproofs offer significant improvements in proof size compared to previous range proof techniques:

- A 64-bit range proof is only 672 bytes
- An aggregate proof for 16 64-bit ranges is 928 bytes

Verification time scales linearly with the number of proofs, but batch verification techniques can significantly reduce the amortized cost.

## Security Properties

Bulletproofs provide the following security properties:

1. Perfect Completeness: Honest provers always convince the verifier.

2. Perfect Special Honest-Verifier Zero-Knowledge: The proof reveals no information about the witness to an honest verifier.

3. Computational Soundness: Under the discrete logarithm assumption, it is computationally infeasible for a dishonest prover to create a valid proof for a false statement.

## Conclusion

Bulletproofs represent a significant advancement in zero-knowledge proof technology, enabling efficient range proofs and other applications with minimal trust assumptions. Their short proof sizes and efficient verification make them well-suited for blockchain and cryptocurrency applications where minimizing on-chain data and verification costs is crucial.

## References

[1] Bünz, B., Bootle, J., Boneh, D., Poelstra, A., Wuille, P., & Maxwell, G. (2018). Bulletproofs: Short proofs for confidential transactions and more. In 2018 IEEE Symposium on Security and Privacy (SP) (pp. 315-334). IEEE.