\title{
CHURP: Dynamic-Committee Proactive Secret Sharing
}

\author{
Sai Krishna Deepak Maram ${ }^{\dagger}$ \\ Cornell Tech
}

\author{
Fan Zhang* ${ }^{\dagger}$ \\ Cornell Tech
}

\author{
Lun Wang \\ UC Berkeley
}

\author{
Andrew Low ${ }^{*}$ \\ UC Berkeley
}

\author{
Yupeng Zhang ${ }^{*}$ \\ Texas A\&M
}

\author{
Ari Juels ${ }^{*}$ \\ Cornell Tech
}

\author{
Dawn Song* \\ UC Berkeley
}

\begin{abstract}
We introduce CHURP (CHUrn-Robust Proactive secret sharing). CHURP enables secure secret-sharing in dynamic settings, where the committee of nodes storing a secret changes over time. Designed for blockchains, CHURP has lower communication complexity than previous schemes: $O(n)$ on-chain and $O\left(n^{2}\right)$ off-chain in the optimistic case of no node failures.

CHURP includes several technical innovations: An efficient new proactivization scheme of independent interest, a technique (using asymmetric bivariate polynomials) for efficiently changing secretsharing thresholds, and a hedge against setup failures in an efficient polynomial commitment scheme. We also introduce a general new technique for inexpensive off-chain communication across the peerto-peer networks of permissionless blockchains.

We formally prove the security of CHURP, report on an implementation, and present performance measurements.
\end{abstract}

\section{CCS CONCEPTS}
- Theory of computation $\rightarrow$ Cryptographic protocols; $\cdot \mathbf{S e}-$ curity and privacy $\rightarrow$ Key management.

\section{KEYWORDS}

secret sharing; dynamic committee; decentralization; blockchain

\section{ACM Reference Format}

Sai Krishna Deepak Maram, Fan Zhang, Lun Wang, Andrew Low, Yupeng Zhang, Ari Juels, and Dawn Song. 2019. CHURP: Dynamic-Committee Proac tive Secret Sharing. In 2019 ACMSIGSAC Conference on Computer and Commu nications Security (CCS '19), November 11-15, 2019, London, United Kingdom. ACM, New York, NY, USA, 20 pages. https://doi.org/10.1145/3319535.3363203

\section{INTRODUCTION}

Secure storage of private keys is a pervasive challenge in cryptographic systems. It is especially acute for blockchains and other decentralized systems. In these systems, private keys control the
\footnotetext{
*Also part of IC3, The Initiative for CryptoCurrencies \& Contract

$\dagger$ The first two authors contributed equally to this work.

CCS '19, November 11-15, 2019, London, United Kingdom

(c) 2019 Association for Computing Machinery

This is the author's version of the work. It is posted here for your personal use. Not for redistribution. The definitive Version of Record was published in 2019 ACM SIGSAC Conference on Computer and Communications Security (CCS '19), November 11-15, 2019, London, United Kingdom, https://doi.org/10.1145/3319535.3363203.
}

most important resources-money, identities [6], etc. Their loss has serious and often irreversible consequences.

An estimated four million Bitcoin (today worth \$14+ Billion) have vanished forever due to lost keys [69]. Many users thus store their cryptocurrency with exchanges such as Coinbase, which holds at least $10 \%$ of all circulating Bitcoin [9]. Such centralized key storage is also undesirable: It erodes the very decentralization that defines blockchain systems.

An attractive alternative is secret sharing. In $(t, n)$-secret sharing, a committee of $n$ nodes holds shares of a secret $s$-usually encoded as $P(0)$ of a polynomial $P(x)$ [73]. An adversary must compromise at least $t+1$ players to steal $s$, and at least $n-t$ shares must be lost to render $s$ unrecoverable.

Proactive secret sharing (PSS), introduced in the seminal work of Herzberg et al. [48], provides even stronger security. PSS periodically proactivizes the shares held by players, while keeping $s$ constant. Players obtain new shares of the secret $s$ that are independent of their old shares, which are then discarded. Provided an adversary never obtains more than $t$ shares between proactivizations, PSS protects the secret $s$ against ongoing compromise of players.

Secret sharing-particularly PSS-would seem to enable users to delegate private keys safely to committees and avoid reliance on a single entity or centralized system. Indeed, a number of commercial and research blockchain systems, e.g., [10, 21, 31, 53, 81], rely on secret sharing to protect users' keys and other sensitive data.

These systems, though, largely overlook a secret-sharing problem that is critical in blockchain systems: node churn.

In permissionless (open) blockchains, such as Bitcoin or Ethereum, nodes may freely join and leave the system at any time. In permissioned (closed) blockchains, only authorized nodes can join, but nodes can fail and membership change. Thus blockchain protocols for secret sharing must support committee membership changes, i.e., dynamic committees.

Today there are no adequate PSS schemes for dynamic committees. Existing protocols support static, but not dynamic committees $[18,48]$, assume weak, passive adversaries [30, 70], or incur prohibitive communication costs [12, 59, 72, 77, 80].

In this paper, we address this critical gap by introducing a new dynamic-committee proactive secret-sharing protocol called CHURP (CHUrn-Robust Proactivization).

\subsection{CHURP functionality}

CHURP allows a dynamic committee, i.e., one undergoing churn, to maintain a shared secret $s$ securely.

Like a standard PSS scheme, CHURP proactivizes shares in every fixed interval of time known as an epoch. It supports dynamic committees as follows. An old committee of size $n$ with a $(t, n)$-sharing of a secret $s$ can transition during a handoff to a possibly disjoint new committee of size $n$ with a new $(t, n)$-sharing of $s$. CHURP achieves security against an active adversary that compromises $t<n / 2$ nodes in each of the old and new committees. CHURP also allows changes to $t$ and $n$ between epochs. (Periodic changes to $s$ are specifically not a goal of PSS schemes, but are easy to add.)

Our main achievement in CHURP is its very low communication complexity: optimistic per-epoch communication complexity in a blockchain setting of $O(n)$ on-chain-which is optimal-and $O\left(n^{2}\right)$ off-chain, i.e., over point-to-point channels. While the on-chain complexity is lower than off-chain, it comes with the additional cost of placing transactions on the blockchain. Cheating nodes cause pessimistic $O\left(n^{2}\right)$ on-chain communication complexity (no off-chain cost). Both communication costs are substantially lower than in other schemes.

Despite somewhat complicated mechanics, CHURP realizes a very simple abstraction: It simulates a trusted third party that stores $s$ for secure use in a wide range of applications-threshold cryptography, secure multi-party computation, etc.

\subsection{Technical challenges and solutions}

CHURP is the first dynamic committee PSS scheme with an endto-end implementation that is practical even for large committees. To achieve its low communication complexity, CHURP overcomes several technical challenges in a different manner than the prior work aimed at dynamic committees, as explained below.

The first challenge is that previous PSS schemes, relying on techniques from Herzberg et al. [48], incur high communication complexity for proactivization ( $O\left(n^{3}\right)$ off-chain per epoch). CHURP uses a bivariate polynomial $B(x, y)$ to share secret $s$, and introduces a new proactivization protocol with cost $O\left(n^{2}\right)$. This protocol is based on efficient bivariate 0 -sharing, i.e., generation of a randomized, shared polynomial $B(x, y)$ with $B(0,0)=0$ to refresh shares. Alternative approaches to PSS that do not explicitly generate a shared polynomial exist [37,68], but CHURP's 0 -sharing technique is of independent in terest: It can also lower the communication complexity of Herzberg et al. [48] and related schemes.

The second challenge is that during a handoff, an adversary may control $t$ nodes in each of the old and new committees, and thus $2 t$ nodes in total. Compromise of $2 t$ shares in a $(t, n)$-sharing would leak the secret s. Previous schemes, e.g., [72], address this problem using "blinding" approaches with costly communication, while [12], address it via impractical virtualization techniques. Instead, CHURP uses a low communication-complexity technique called dimensionswitching, that is based on known share resharing techniques. It uses an asymmetric bivariate polynomial $B(x, y)$, with degree $t$ in one dimension and degree $2 t$ in the other. During a handoff, it switches temporarily to a ( $2 t, n$ )-sharing of $s$ to tolerate up to $2 t$ compromised shares; afterward, it switches back to a $(t, n)$-sharing. Each switching involves a round of share resharing. Although dimension-switching is based on known techniques, CHURP's novelty lies in applying them to the dynamic committee setting to tolerate $2 t$ compromises

Finally, most PSS schemes commit to secret degree-t polynomials using classical schemes (e.g., [34, 64]) with per-commitment size $O(t)$. CHURP uses an alternative due to Kate, Zaverucha, and Goldberg (KZG) [50] with size $O(1)$. Use of KZG for secret sharing isn't new [11], but CHURP introduces a novel KZG hedge. KZG assumes trusted setup and a non-standard hardness assumption. If these fail, CHURP still remains secure-but degrades to slightly weaker adversarial threshold $t<n / 3$. The detection mechanisms used to hedge are efficient- $O(n)$ on-chain-and are KZG-free-so, our techniques can easily be adapted to future secret-sharing schemes that rely similarly on KZG or related non-standard assumptions.

We compose these techniques to realize CHURP with provable security and give a rigorous security proof.

\subsection{Implementation and Experiments}

We present an implementation of CHURP. Our experiments show very practical communication and computation costs-at least 1000 x improvement over the existing state-of-the-art dynamic-committee PSS scheme [72] in the off-chain communication complexity for large committees (See Section 6).

Additionally, to achieve inexpensive off-chain communication among nodes in CHURP, we introduce a new technique for permissionless blockchains that is of independent interest. It leverages the peer-to-peer gossip network as a low-cost anonymous point-topoint channel. We experimentally demonstrate off-chain communication in Ethereum with monetary cost orders of magnitude less than on-chain communication.

\subsection{Outline and Contributions}

After introducing the functional, adversarial, and communication models in Section 2, we present our main contributions:

- CHUrn-Robust Proactive secret sharing (CHURP): In Section 3, we introduce CHURP, a dynamic-committee PSS scheme with lower communication complexity than previous schemes.

- Novel secret-sharing techniques: We introduce a new 0-sharing protocol for efficient proactivization in Section 4, dimension-switching technique to safeguard the secret in committee handoffs in Section 5.3, and hedging techniques for failures in the KZG commitment scheme in Section 5.5.

- New point-to-point blockchain communication technique: We introduce a novel point-to-point communication technique for permissionless blockchains in Section 7-usable in CHURP and elsewherewith orders of magnitude less cost than on-chain communication.

- Implementation and experiments: We report on an implementation of CHURP in Section 6 and present performance measurements demonstrating its practicality.

We give a security proof for CHURP in Appendix A. We discuss related work in Section 8 and CHURP's many potential applicationsthreshold cryptography, smart contracts with private keys, consensus simplification for light clients, etc.-in Appendix B. We have released the CHURP system as an open-source tool at https://www. churp.io.

\section{MODEL AND ASSUMPTIONS}

We now describe the functional, adversarial, and communication models used for CHURP.

In a secret-sharing scheme, a committee of nodes shares a fixed secret $s$. Let $C$ denote a committee and $\left\{C_{i}\right\}_{i=1}^{n}$ denote the $n$ nodes in

![](https://cdn.mathpix.com/cropped/2024_07_17_ad78533e3470f9e4f2abg-03.jpg?height=461&width=678&top_left_y=276&top_left_x=260)

Figure 1: Handoff between two committees at the end of a dynamic proactive secret-sharing epoch. The secret $s$ remains fixed. Committees may intersect, e.g., $B_{2}=A_{2}$ and $B_{3}=A_{3}$.

the committee. Each node $C_{i}$ holds a distinct share $s_{i}$. CHURP proactivizes shares, i.e., changes them periodically to prevent leakage of $s$ to an adversary that gradually compromises nodes. Again, we emphasize that CHURP does so for a dynamic committee [12, 72], i.e., nodes may periodically leave / join the committee.

Shares change in a proactive secret-sharing protocol such as CHURP during what is called a handoff protocol. Handoff proactivizes $s$, i.e., changes its associated shares, while transferring $s$ from an old committee to a new, possibly intersecting one. Fig. 1 depicts the handoff process. The adversarial model for proactive secret sharing in general limits adversarial control to a threshold $t$ of nodes per committee. During a handoff, CHURP allows nodes to agree out of band on a change to $t$, as explained below.

\subsection{Functional model}

Epoch:Time in CHURP, as in any proactive secret-sharing scheme [48], is divided into fixed intervals of predetermined length called epochs. In each epoch, a specific committee of nodes assumes control of and then holds $s$. Concretely, in an epoch $e$, a committee $C^{(e)}$ of size $N^{(e)}$ shares $s$ using a $\left(t, N^{(e)}\right)$-threshold scheme.

\begin{tabular}{|c|c|c|c|}
\hline Handoff & Committee $C^{(e-1)}$ & Handoff & Committee $C^{(e)}$ \\
\hline
\end{tabular}

Figure 2: Each epoch begins with a handoff phase where the old committee hands off the secret $s$ to the new committee. It is followed by a period of committee operation.

Handoff: Fig. 2 depicts the handoff at the beginning of an epoch. It involves a transfer of $s$ from an old committee, which we denote $C^{(e-1)}$, to a new committee, denoted $C^{(e)}$. Prior to completion of the handoff, $C^{(e-1)}$ is able to perform operations using $s$.

Churn: In the dynamic-committee setting of CHURP, nodes can leave a committee at any time, but can only be added during a handoff. Let $C_{\text {left }}^{(e-1)}$ denote the set of nodes that have left the committee before the handoff in epoch $e$. Let $C_{\text {alive }}^{(e-1)}=C^{(e-1)} \backslash C_{\text {left }}^{(e-1)}$ denote the set of nodes that participate in the handoff. We let churn rate $\alpha$ denote a bound such that $\left|C_{\text {alive }}^{(e-1)}\right| \geq\left|C^{(e-1)}\right|(1-\alpha)$. Later, we provide a lower bound on the committee size using the rate $\alpha$.
Keys: We assume that every node in CHURP has private / public key pair and that public keys are known to all nodes in the system. Such a setup is common in secret-sharing systems [48, 72].

\subsection{Adversarial model}

We consider a powerful active adversary $\mathcal{A}$. It may decide to corrupt nodes at any time. Once a node is corrupted by the adversary, it is assumed to be corrupted until the end of the current epoch. (A node may thus be "released" by an adversary in a new epoch so that it is no longer corrupted.) Corrupted nodes are allowed to deviate from the protocol arbitrarily. The proofs of correctness used by nodes in CHURP requires that we assume a computationally bounded (polynomial-time) adversary.

As noted above, we limit the adversary $\mathcal{A}$ to corruption of no more than a threshold of nodes in a given committee. This threshold, as noted above, may change in CHURP through out-of-band agreement by committees. In this case, letting $t$ and $t^{\prime}$ denote corruption thresholds for old and new committees respectively, $\mathcal{A}$ may control at most $t$ nodes in $C^{(e-1)}$ and $t^{\prime}$ nodes in $C^{(e)}$. We present the protocol in CHURP for threshold changes in Section 5.4. For simplicity of exposition, however, we assume in what follows that $t=t^{\prime}$, i.e., the corruption threshold $t$ remains fixed.

Observe that during the handoff between epochs $e-1$ and $e$, members of both committees, $C^{(e-1)}$ and $C^{(e)}$, are active. Thus $\mathcal{A}$ may control up to $2 t$ nodes at this time. As committees may intersect, i.e., an adversary may control a given node $i$ in both the old and new committees. Alternatively, $\mathcal{A}$ may control node $i$ in one committee, but not the other, reflecting either a fresh corruption or node recovery.

Definition 1. A protocolfor dynamic-committee proactive secret sharing satisfies the following properties in the functional model above for any probabilistic polynomial time adversary $\mathcal{A}$ with threshold $t$ :

Secrecy: If $\mathcal{A}$ corrupts no more than $t$ nodes in a committee of any epoch, $\mathcal{A}$ learns no information about the secret $s$.

Integrity: If $\mathcal{A}$ corrupts no more than t nodes in each of the committees $C^{(e-1)}$ and $C^{(e)}$, after the handoff, the shares for honest nodes can be correctly computed and the secrets remains intact.

\subsection{Communication model}

We aim to minimize communication complexity in CHURP.Specifically, we optimize for on-chain complexity and off-chain complexity in that order. We also consider the round complexity of our protocol designs, but prioritize communication complexity because blockchains-particularly permissionless ones-incur high costs for on-chain operations. We measure the communication complexity of our protocol (and related ones) in terms of on-chain and off-chain communication cost, as follows:

On-chain: Existing approaches such as MPSS [72] use PBFT [19] for consensus. Instead, we assume the availability of a blockchain (or other bulletin-board abstraction) to all nodes in the committee. We do this for two reasons. First, abstracting away the consensus layer results in simpler, and more modular secret-sharing protocols. Second, it makes sense to capitalize on the availability of blockchains today, rather than re-engineer their functionality.

In our model, nodes can either post a message (or) retrieve any number of messages from the blockchain. After a node posts a message to the blockchain, within a finite time period $T$, it gets published,
i.e., blockchain access is synchronous and the message is now retrievable by any node. This channel is assumed to be reliable: messages posted are not lost. This model is widely adopted in the literature (e.g., See $[54,63,79]$ ).

Permissionless blockchains. While our techniques apply also to permissioned blockchains, we focus on permissionless blockchainse.g., Ethereum. On such chains, users pay (heavily) for writes, but reads are free. Thus we measure on-chain communication complexity only in terms of writes, e.g., $O(n)$ on-chain cost means $O(n)$ bits written to the blockchain.

Off-chain: Nodes may alternatively communicate point-to-point (P2P) without direct use of the blockchain. We assume that every node has such a channel with every other node. P 2 P channels are also assumed to be reliable: all messages arrive without getting lost. We work in a synchronous model, i.e., any message sent via this channel will be received within a known bounded time period, $T^{\prime}$.

We emphasize that synchronicity of the P 2 P network is required only for performance, not for liveness, secrecy or integrity. Looking ahead, without enough synchronicity, the off-chain protocol halts and the execution switches to the on-chain channel. In other words, an adversary may slow down the protocol execution temporarily by delaying messages, but she cannot learn or corrupt the secret. Moreover, CHURP only requires a short period of synchronicity (e.g., a few minutes) at the end of every epoch (a relatively long epoch, e.g., a day, would be the norm for CHURP). We discuss synchronicity assumptions in Section 5.3.3.

Off-chain P2P channels can be implemented in different ways depending on the deployment environment. In a decentralized setting, though, nodes are often assumed not to have P 2 P communication, to protect them from targeted attacks and anonymity compromise In such cases, one can use anonymous channels, such as Tor [75], to preserve anonymity with additional setup cost and engineering complexity. Alternatively, off-chain channels can be implemented by an overlay on top of the existing blockchain infrastructure. We show how to leverage the gossip network of a blockchain system [32] for inexpensive off-chain communication in Section 7.

We measure off-chain communication complexity as the total number of bits transmitted in P2P channels. In general, where we refer informally to proactivization protocols' cost in this work, we mean their communication complexity, on-chain or off-chain, as the case may be.

\section{OVERVIEW OF CHURP}

Now we provide an overview of CHURP, with intuition behind our core techniques. First, we briefly review two key new techniques used in CHURP: bivariate 0-sharing and dimension-switching. (We defer details until later in the paper.) Then we give an overview and example of optimistic execution of CHURP. Finally, we briefly dis cuss pessimistic execution paths in CHURP, i.e., what happens when nodes are faulty, and our third key technique of hedging against failures in KZG.

\subsection{Key secret-sharing techniques}

Recall that in an ordinary $(t, n)$-threshold Shamir secret sharing (see [73]), shares of secret $s$ are points on a univariate polynomial $P(x)$ such that $P(0)=s$. Instead, to enable its two key techniques,
CHURP employs a bivariate polynomial $B(x, y)$ such that $B(0,0)=s$. A share of $B(x, y)$ is itself a univariate polynomial: Either $B(x, i)$ or $B(i, y)$ where $i$ is the node index.

Bivariate 0 -sharing: Proactivization in nearly all secret-sharing schemes involves generating a fresh, random polynomial that shares a 0 -valued secret, e.g., $Q(x, y)$ such that $Q(0,0)=0$. This is added to the current polynomial that encodes the secret $s$. We call such a polynomial $Q(x, y)$ a 0 -hole polynomial and generation of this polynomial 0 -sharing. Previous approaches' main communication bottleneck is naïve 0 -sharing that incurs high ( $O\left(n^{3}\right)$ off-chain) communication complexity. Our 0 -sharing protocol achieves lower ( $O\left(n^{2}\right)$ off-chain) complexity. (Details in Section 4).

Dimension-switching: CHURP uses a bivariate polynomial $B(x, y)$ asymmetric and of non-uniform degree. Specifically, it uses a polynomial $B(x, y)$ of degree $\langle t, 2 t\rangle$. By this, we mean that it is degree- $t$ in $x$ (highest term $x^{t}$ ) and degree- $2 t$ in $y$ (highest term $y^{2 t}$ ).

This structure enables our novel dimension-switching technique in CHURP. Nodes can switch between a sharing in the degree- $t$ dimension of $B(x, y)$ and the degree- $2 t$ dimension. The result is a change from a $(t, n)$-sharing of $s$ to a $(2 t, n)$-sharing-and vice versa. We apply known resharing techniques $[28,36]$ via bivariate polynomials to switch between different sharings. As we show, dimension switching provides an efficient way to address a key challenge mentioned above. During a handover, the adversary can control up to $2 t$ nodes, but between handovers, we instead want a $(t, n)$-threshold sharing of $s$. (Details in Section 5.3.)

\subsection{CHURP: Overview}

We now give an overview of CHURP execution. We first consider the optimistic case, and discuss pessimistic cases below in Section 3.5.

At the end of a given epoch $e-1$, before a handoff occurs, the current committee $C^{(e-1)}$ is in what we call a steady state.

The committee $C^{(e-1)}$ holds a $(t, n)$-sharing of $s=B(0,0)$. This sharing uses the degree- $t$ dimension of $B(x, y)$, as noted above. Node $C_{i}^{(e-1)}$ holds share $s_{i}=B(i, y)$, and can compute $B(x, 0)$ for $x=i$. So it is easy to see that $s_{i}$ is actually a share in a $(t, n)$-sharing of $B(0,0)$. We refer to the shares in steady state as full shares.

During the handoff in epoch $e$, nodes in the old and new committees $C^{(e-1)}$ and $C^{(e)}$ switch their sharing of $s$ to the degree-2t dimension of $B(x, y)$, resulting in what we call reduced shares.

Specifically, node $C_{j}^{(e)}$ holds share $s_{j}=B(x, j)$. Node $C_{j}^{(e)}$ can compute $B(0, y)$ for $y=j$, and consequently $s_{j}$ is a share in a $(2 t, n)$-sharing of $B(0,0)$. The share $s_{j}$ here has "reduced" power in the sense that $2 t+1$ of these shares (as opposed to $t+1$ full shares in steady state) are needed to reconstruct $s$. Thus the adversary cannot recover $s$ despite potentially compromising $2 t$ nodes across the old and new committees $C^{(e-1)}$ and $C^{(e)}$.

After share reduction, the polynomial $B(x, y)$ is proactivized. A 0 -hole bivariate polynomial $Q(x, y)$, i.e., such that $Q(0,0)=0$, is generated (using the new protocol given in Section 4). $Q(x, y)$ is then added to $B(x, y)$, yielding a fresh polynomial $B^{\prime}(x, y)=B(x, y)+Q(x, y)$. Nodes update their reduced shares accordingly. Because $Q(x, y)$ is 0 -hole, the secret $s$ remains unchanged, i.e., $s=B^{\prime}(0,0)$.

Shares in $B^{\prime}(x, y)$, i.e., for the new committee, are now independent of those for $B(x, y)$, i.e., for the old committee. So it is now safe
to perform full-share distribution, i.e., to switch to the degree-t dimension of $B^{\prime}(x, y)$. This involves distributing full shares to the new committee $C^{(e)}$. At this point, the steady state is achieved for epoch $e$. Committee $C^{(e)}$ holds a $(t, n)$-sharing of $s$ using $B^{\prime}(x, y)$.

To summarize, the three phases in the CHURP handoff are:

- Share reduction: Nodes switch from the degree- $t$ dimension of $B(x, y)$ to the degree- $2 t$ dimension. As a result, each node $C_{j}^{(e)}$ in the new committee obtains a reduced share $B(x, j)$.

- Proactivization: The new committee generates $Q(x, y)$ such that $Q(0,0)=0$, and each node $C_{j}^{(e)}$ obtains a reduced share: $B^{\prime}(x, j)=$ $B(x, j)+Q(x, j)$. Proactivization ensures that shares in the new committee are independent of those in the old.

- Full-share distribution: New shares $B^{\prime}(i, y)$ are generated from reduced shares $\left\{B^{\prime}(x, j)\right\}_{j}$, by switching back to the degree- $t$ dimension of $B^{\prime}(x, y)$.

The protocol thus returns to its steady state. Note that during the handoff, remaining nodes in old committee can still perform operations using $s$. So there is no operational discontinuity in CHURP.

\subsection{An example}

In Fig. 3, we show a simple example of the handoff protocol in CHURP assuming all nodes are honest. The old committee consists of three nodes $C^{(e-1)}=\left\{A_{1}, A_{2}, A_{3}\right\}$. $A_{3}$ leaves at the end of the epoch, and a new node $A_{3}^{\prime}$ joins. The new committee is thus $C^{(e)}=\left\{A_{1}, A_{2}, A_{3}^{\prime}\right\}$. The underlying polynomial $B(x, y)$ is thus of degree $\langle 1,2\rangle$. Node $A_{i}$ 's share is $B(i, y)$ or 3 points: $B(i, 1), B(i, 2)$ and $B(i, 3)$. The figure depicts the three phases of the handoff, as follows.

Share reduction: To start the handoff, each node $j$ in the new committee constructs its reduced share $B(x, j)$ from points received from $C^{(e-1)}$. As shown in the figure, node $A_{3}^{\prime}$ receives points $B(1,3)$ and $B(2,3)$ from $A_{1}$ and $A_{2}$ respectively, from which $B(x, 3)$ can be constructed. Similarly, $A_{1}$ and $A_{2}$ construct $B(x, 1)$ and $B(x, 2)$.

Proactivization: Having reconstructed reduced shares $\{B(x, j)\}_{j}$, nodes in the new committee collectively generate a 0 -hole bivariate polynomial $Q(x, y)$ of degree $\langle t, 2 t\rangle$, with the constraint that each $j$ only learns $Q(x, j)$. Reduced shares are updated as $B^{\prime}(x, j)=B(x, j)+$ $Q(x, j)$. In the example above, node $j$ ends up with $Q(x, j)$ of a random 0 -hole polynomial $Q(x, y)$.

Full-share distribution: Nodes in the new committee get their full shares from the updated reduced shares. Take $A_{1}$ as an example. By this point, $A_{1}$ has $B^{\prime}(x, 1)$ and sends $B^{\prime}(i, 1)$ to $A_{i}$ for $i \in\{2,3\}$. Other nodes do the same. Hence, $A_{1}$ receives $B^{\prime}(1,2)$ and $B^{\prime}(1,3)$ from $A_{2}$ and $A_{3}^{\prime}$ respectively. It now has the necessary three points $\left\{B^{\prime}(1, j)\right\}_{j \in[3]}$ in order to interpolate its full share $B^{\prime}(1, y)$.

\subsection{Active security}

As noted before, the above example assumes an honest-but-curious adversary. Additional machinery in the form of cryptographic proofs of correctness for node communications-detailed in Section 5.3-are required against an active adversary. These proofs do not alter the overall structure of the protocol.

\subsection{Pessimistic CHURP execution paths}

What we have described thus far is an optimistic execution of CHURP. This corresponds to a subprotocol Opt-CHURP that is highly efficient and optimistic: it only completes when all nodes are honest and the assumptions of the KZG scheme hold.

When things go wrong, CHURP can detect the violation and resort to pessimistic paths. Specifically, Exp-CHURP-A can hold malicious nodes accountable. Moreover, CHURP introduces a novel hedge against any soundness failure of the KZG scheme, due to either a compromised trusted setup or a falsified hardness assumption $(t$-SDH). The hedging technique is efficient and incurs only $O(n)$ onchain cost to detect such failures. When detected, CHURP switches to Exp-CHURP-B that only relies on DL and no trusted setup.

As noted above, the on-chain / off-chain communication complexity of CHURP is $O(n) / O\left(n^{2}\right)$ in the optimistic case. Unlike the optimistic path, the two pessimistic paths do not use the off-chain channel and incur $O\left(n^{2}\right)$ on-chain cost. Opt-CHURP and Exp-CHURP-A requires $t<n / 2$, while Exp-CHURP-B requires $t<n / 3$. We give more details on all the paths in CHURP in Section 5.

\section{EFFICIENT BIVARIATE 0-SHARING}

In this section, we introduce our technique for efficient 0 -sharing of bivariate polynomials. It is a key new building block in CHURP, used in the proactivization phase. The bivariate 0 -sharing protocol uses resharing techniques $[28,36]$ as a building block.

Recall that in the context of bivariate polynomials, 0 -sharing means having a committee $C$ generate a $\langle t, 2 t\rangle$-bivariate polynomial $Q(x, y)$ such that $Q(0,0)=0$. Each node $C_{i}$ holds a share $Q(i, y)$.

Previous works have naïvely extended 0 -sharing techniques for univariate polynomials to the bivariate case: Each node generates its own 0 -hole bivariate polynomial $Q_{i}$ i.e., $Q_{i}(0,0)=0$, and distributes points on it. Thus each node transmits $O(n)$ univariate polynomials, resulting in $O\left(n^{2}\right)$ off-chain communication complexity per node, and $O\left(n^{3}\right)$ in total.

Our new technique, specified as protocol BivariateZeroShare, brings the total off-chain communication complexity down to just $O(t n)$ in the optimistic case. In the pessimistic case, i.e., if a node is caught cheating, different protocols (see Section 5) must then be invoked. Even in the pessimistic case, though, our techniques incur no more cost than in previous schemes: $O\left(n^{3}\right)$ in the dynamic setting and $O\left(n^{2}\right)$ in the static Herzberg et al. setting.

BivariateZeroShare comprises two steps. In the first step, a 0 sharing subprotocol UnivariateZeroShare is executed among a subset $\mathcal{U}$ of $2 t+1$ nodes. At the end of this step, each node $\mathcal{U}_{j}$ holds a share $s_{j}$ of a univariate polynomial $P(x)$. In the second step, each node in $\mathcal{U}$ reshares its share $s_{j}$ among all nodes, i.e., the full committee. Each node $C_{i}$ thereby obtains share $Q(i, y)$ of bivariate polynomial $Q(x, y)$, as desired.

BivariateZeroShare is formally specified in Fig. 10. (For the interest of space, we present all protocols formally in the appendix. Nonetheless, the text description here is sufficient to understand the paper.) For ease of presentation, we describe an honest-but-curious protocol version in this section. Our full protocol, which is secure against active adversaries, is detailed in Section 5.3.

First step-Sharing $P(x)$ :As noted, BivariateZeroShare first chooses a subset $\mathcal{U} \subseteq C$ of $2 t+1$ nodes, i.e., $|\mathcal{U}|=2 t+1$. This can be done

![](https://cdn.mathpix.com/cropped/2024_07_17_ad78533e3470f9e4f2abg-06.jpg?height=434&width=1261&top_left_y=279&top_left_x=432)

Figure 3: An example of the handoff protocol: Curves denote univariate polynomials (reduced shares) while squares denote points on these polynomials. See Section 3.3 for a description.

as follows: Order nodes lexicographically by their public keys and choose the first $2 t+1$. Without loss of generality, $\mathcal{U}=\left\{C_{j}\right\}_{j=1}^{2 t+1}$.

The nodes of $\mathcal{U}$ then execute the univariate 0 -sharing subprotocol UnivariateZeroShare presented in Fig. 9. This subprotocol is not new-it was previously used for proactivization in [48]. Each node $\mathcal{U}_{j}$ generates a degree- $2 t$ univariate 0 -hole polynomial $P_{j}(x)$. ${ }^{1}$ The sum $P(x)=\sum_{j=1}^{2 t+1} P_{j}(x)$ is itself a degree- $2 t$ univariate 0 -hole polynomial $P(x)$. Then, $\mathcal{U}_{j}$ redistributes points on its local polynomial $P_{j}(x)$, enabling every $\mathcal{U}_{i}$ at the end of the step to compute its share $s_{i}=P(i)$

Second step-Resharing $P(x)$ : Nodes in $\mathcal{U}$ now reshare $P(x)$ among all of $C$, resulting in a sharing of the desired bivariate polynomial $Q(x, y)$.

Each node $\mathcal{U}_{j}$ generates a degree-t univariate polynomial $R_{j}(x)$ uniformly at random under the constraint $R_{j}(0)=s_{j}$, i.e., $R_{j}(x)$ encodes the node's share $s_{j}$. Together, the $2 t+1$ degree-t polynomials $\left\{R_{j}(x)\right\}$ uniquely define a degree- $\langle t, 2 t\rangle$ bivariate polynomial $Q(x, y)$ such that $Q(x, j)=R_{j}(x)$ for $j=1,2, \ldots, 2 t+1$ and $Q(0,0)=0$.

Node $\mathcal{U}_{j}$ sends $R_{j}(i)=Q(i, j)$ to every other node $C_{i}$ in the full committee. Using the received points, each committee member $C_{i}$ interpolates to compute its share-a $2 t$-degree polynomial $Q(i, y)$. The constraint $Q(0,0)=0$ is satisfied because the zero coefficients of $R_{j}(x)$ are composed of shares generated from the 0 -sharing step before, i.e., UnivariateZeroShare. Since each node in $\mathcal{U}$ transmits $n$ points, the overall cost incurred is just $O(t n)$ off-chain.

We use $(t, n)$-BivariateZeroShare as a subroutine in CHURP with some modifications. As explained before, it can also reduce the off-chain communication complexity of Herzberg et al.'s PSS scheme [48], i.e., the static-committee setting, by a factor of $O(n)$. Due to lack of space, we present this application in Appendix D.

\section{CHURP PROTOCOL DETAILS}

CHURP consists of a suite of tiered protocols with different trust assumptions and communication complexity.

The execution starts at the top tier-a highly efficient optimistic protocol. Only upon detection of adversarial misbehavior, does the execution fall back to lower tiers. The three tiers of CHURP and their relationship are shown in Fig. 4, detailed as below.
\footnotetext{
${ }^{1}$ An attack is outlined in [56] that breaks the UnivariateZeroShare protocol in [48] It does so in an adversarial model similar to ours, i.e., the adversary controls $t$ nodes in old and new committees and thus $2 t$ in total, rather than $t$ in total as in [48]. CHURP defeats this attack via dimension-switching, using reduced shares during the handoff.
}

The top tier, Opt-CHURP, is the default protocol of CHURP. It is optimistic and highly efficient: if no node misbehaves, the execution completes incurring only $O(n)$ on-chain and $O\left(n^{2}\right)$ off-chain cost. As a design choice, Opt-CHURP does not identify faulty nodes but rather just detects faulty behavior, upon which the execution switches to a lower tier protocol, also referred to as a pessimistic path.

The second tier is Exp-CHURP-A, the main pessimistic path of CHURP. Unlike the optimistic path, Exp-CHURP-A exclusively uses on-chain communication channel, which allows to identify and expel faulty nodes using proofs of correctness. Exp-CHURP-A trades performance for robustness: the execution is guaranteed to complete as long as the adversarial threshold $t<n / 2$, but incurs $O\left(n^{2}\right)$ on-chain communication in the worst case.

Both Opt-CHURP and Exp-CHURP-A use KZG commitments to achieve $t<n / 2$. As noted before, this commitment scheme requires a trusted setup phase to generate public keys with a trapdoor. The trapdoor must be "destroyed" after the setup; otherwise soundness is lost, i.e., binding property of KZG is broken. KZG introduces the only trusted setup in CHURP, and thus represents its main protocol-level vulnerability. KZG also relies on a non-standard hardness assumption, the $t$-Strong Diffie-Hellman assumption ( $t$-SDH).

To hedge against soundness failure in KZG (either due to a falsified trust assumption or a compromised trusted setup), we introduce an additional verification step (StateVerif), which can be executed at the end of Opt-CHURP or Exp-CHURP-A. StateVerif is highly efficient-incurs only $O(n)$ on-chain complexity. Any fault detected by StateVerif indicates that KZG is unusable, and triggers a KZG-free pessimistic path named Exp-CHURP-B. Exp-CHURP-B has the same cost as Exp-CHURP-A, but one drawback: It tolerates a lower adversarial threshold, $t<n / 3$. More details on StateVerif in Section 5.5.

In summary, the three tiers (subprotocols) of CHURP are:

(1) Opt-CHURP: The default protocol of CHURP. It incurs $O(n)$ on-chain and $O\left(n^{2}\right)$ off-chain communication complexity under the optimal resilience bound $t<n / 2$.

(2) Exp-CHURP-A: Invoked if Opt-CHURP fails. It incurs $O\left(n^{2}\right)$ on-chain communication complexity under the optimal bound $t<n / 2$.

(3) Exp-CHURP-B: Invoked if a soundness breach of KZG is detected by StateVerif. It incurs the same cost as Exp-CHURP-A, but requires $t<n / 3$.

Table 2 summarizes the three tiers. Due to space constraints, we present only Opt-CHURP in the body of the paper and present ExpCHURP-A and Exp-CHURP-B in Appendix C.

![](https://cdn.mathpix.com/cropped/2024_07_17_ad78533e3470f9e4f2abg-07.jpg?height=453&width=672&top_left_y=478&top_left_x=260)

Figure 4: CHURP protocol tiers. Opt-CHURP is the default protocol of CHURP. Exp-CHURP-A and Exp-CHURP-B are run only if a fault occurs in Opt-CHURP.

\subsection{Notation and Invariants}

We now introduce the notation and invariants that will be used to explain the protocols of CHURP. Notation is summarized in Table 1.

KZG polynomial commitments: KZG commitment allows a prover to commit to a polynomial $P(x)$ and later prove the correct evaluation $P(i)$ to a verifier. (Further details in Fig. 8 and [50].)

CHURP invariants: We say the system arrives at a steady state after it completes a successful handoff. The following invariants stipulate the desired properties of a steady state. We use invariants to explain the protocol and reason about its security.

Let $C$ be a committee of $n$ nodes $\left\{C_{i}\right\}_{i=1}^{n}$. Let $B(x, y)$ denote the asymmetric bivariate polynomial of degree $\langle t, 2 t\rangle$ used to share the secret $s$, i.e., $s=B(0,0)$. In a steady state, the three invariants below must hold:

- Inv-Secret: The secret $s$ is the same across handoffs.

- Inv-State: Each node $C_{i}$ holds a full share $B(i, y)$ and a proof to the correctness thereof. Specifically, the full share $B(i, y)$ is a degree$2 t$ polynomial, and hence can be uniquely represented by $2 t+1$ points $\{B(i, j)\}_{j=1}^{2 t+1}$. The proof is a set of witnesses $\left\{W_{B(i, j)}\right\}_{j=1}^{2 t+1}$.

- Inv-Comm: KZG commitments to reduced shares $\left(\{B(x, j)\}_{j=1}^{2 t+1}\right)$ are available to all nodes.

The first invariant Inv-Secret ensures the secret remains unchanged, a core functionality of CHURP.

Inv-State and Inv-Comm ensures the correctness of the protocol. For example, recall from Section 3 that during the handoff (the Share Reduction phase), nodes in the old and the new committee switch their dimension of sharing, from full shares to reduced shares. Using the commitments (specified by Inv-Comm) and the witnesses (specified by Inv-State), new committee nodes can verify the correctness of reduced shares, thus the correctness of dimension-switching.

Note that to realize Inv-Comm, hashes of KZG commitments are put on-chain for consensus while the commitments are transmitted off-chain between nodes.

\begin{tabular}{cl}
\hline Notation & Description \\
\hline$C^{(e-1)}, C^{(e)}$ & Old, New committee \\
$B(x, y)$ & Bivariate polynomial used to share the secret \\
$\langle t, k\rangle$ & Degree of $\langle x, y\rangle$ terms in $B$ \\
$R S_{i}(x)=B(x, i)$ & Reduced share held by $C_{i}$ \\
$F S_{i}(y)=B(i, y)$ & Full share held by $C_{i}$ 's \\
$C_{B(x, j)}$ & KZG commitment to $B(x, j)$ \\
$W_{B(i, j)}$ & Witness to evaluation of $B(x, j)$ at $i$ \\
$Q(x, y)$ & Bivariate proactivization polynomial \\
$\mathcal{U}^{\prime}$ & Subset of nodes chosen to participate in handoff \\
$\lambda_{i}$ & Lagrange coefficients \\
\hline
\end{tabular}

Table 1: Notation

\subsection{CHURP Setup}

The setup phase of CHURP sets the system to a proper initial steady state. To start, an initial committee $C^{(0)}$ is selected. The setup of KZG is performed and the secret is shared among $C^{(0)}$. Using their shares, members of $C^{(0)}$ can generate commitments to install the three invariants.

The setup of KZG can be performed by a trusted party or a committee assuming at least one of them is honest. The secret to be managed by CHURP can be generated by a trusted party or in a distributed fashion, e.g., [41]. We leave committee selection out-of-scope for this paper. Readers can refer to, e.g., [44], for a discussion.

\subsection{CHURP Optimistic Path (Opt-CHURP)}

Recall that Opt-CHURP transfers shares of some secret $s$ from an old committee, denoted $C=C^{(e-1)}$, to a new committee $C^{\prime}=C^{(e)}$. CHURP can support both committee-size and threshold changes, i.e., a transition from $(n, t)$ to some $\left(n^{\prime}, t^{\prime}\right)$ in any epoch. For ease of exposition here, though, we allow $n$ to change across epochs assuming a constant threshold $t$. Changing the threshold is discussed in Section 5.4.

Opt-CHURP proceeds in three phases. The first phase, Opt-ShareReduce, performs dimension-switching to tolerate an adversary capable of compromising $2 t$ nodes across the old and new committees. By the end of this phase, reduced shares are constructed by members of the new committee. The second phase, OptProactivize, proactivizes these reduced shares so that new shares are independent of the old ones. The third and the final phase, OptShareDist, restores full shares from reduced shares, and thus returns to the steady state.

At the beginning of Opt-CHURP, each node in $C^{\prime}$ requests the set of KZG commitments from any node in $C$, say $C_{1}$. Recall that by the invariant Inv-Comm, each node in $C$ holds the KZG commitments to the current reduced shares, $\left\{C_{B(x, j)}\right\}_{j=1}^{2 t+1}$, while the corresponding hashes are on-chain. The received commitments are verified using the on-chain hashes. Optimistically, each node in $C^{\prime}$ receives the correct set of commitments. If a node receives corrupt ones, we switch to a pessimistic path where the KZG commitments are published on-chain. The above check enabled by the on-chain hashes ensures that new committee nodes receive the correct set of commitments. The phases of Opt-CHURP are as follows:

5.3.1 Share Reduction (Opt-ShareReduce). The protocol starts by choosing a subset $\mathcal{U}^{\prime} \subseteq C^{\prime}$ of $2 t+1$ members (possible because $\left.\left|C^{\prime}\right|>2 t\right)$. The nodes in $\mathcal{U}^{\prime}$ are denoted $\left\{\mathcal{U}_{j}^{\prime}\right\}_{j=1}^{2 t+1}$.

Some members in the old committee $C$ may have left the protocol by this point. Let $C_{\text {alive }} \subseteq C$ denote the subset of nodes that are present, w.l.o.g., let this subset be $\left\{C_{i}\right\}_{i=1}^{\left|C_{\text {alive }}\right|}$.

Recall that by the invariant Inv-State, each node $C_{i}$ holds a full share $B(i, y)$. Now, $C_{i}$ distributes points on its full share allowing computation of reduced shares $B(x, j)$ by all members of $\mathcal{U}^{\prime}$-making a dimension-switch from the degree- $t$ dimension of $B(x, y)$ to the degree- $2 t$ dimension. Specifically, $C_{i}$ sends $B(i, j)$ to $\mathcal{U}_{j}^{\prime}$, which interpolates the received points to get its reduced share $B(x, j){ }^{2}$ Note that in the optimistic path we require all $2 t+1$ nodes in $\mathcal{U}^{\prime}$ to participate If any adversarial nodes fail to do so, we switch to a pessimistic path as detailed above.

The received points are accompanied by witnesses allowing for verification using the KZG commitments received previously. Since $t+1$ correct points are sufficient to reconstruct the reduced share, we need at least $2 t+1$ points $\left(\left|C_{\text {alive }}\right|>2 t\right)$ to guarantee liveness.

The size of $\mathcal{C}_{\text {alive }}$ is governed by the bounded churn rate $\alpha$, i.e., $\left|C_{\text {alive }}\right| \geq|C|(1-\alpha)$. Thus, the condition for liveness, $\left|C_{\text {alive }}\right|>2 t$, places a lower bound on the committee size, $|C|(1-\alpha)>2 t$ or $|C|>\lfloor 2 t / 1-\alpha\rfloor$.

The protocol Opt-ShareReduce is formally specified in Fig. 11. At the end of Opt-ShareReduce, dimension-switching is complete and each node $\mathcal{U}_{j}^{\prime}$ has a reduced share $B(x, j)$.

Communication complexity: Each node in $\mathcal{U}^{\prime}$ receives $O(n)$ points, so Opt-ShareReduce incurs $O(n t)$ off-chain cost.

5.3.2 Proactivization (Opt-Proactivize). In this phase, $\mathcal{U}^{\prime}$ proactivizes the bivariate polynomial $B(x, y)$-a key step in generating new shares independent of the old ones held by members of $C$. The polynomial $B(x, y)$ is updated using a random bivariate polynomial $Q(x, y)$ generated such that $Q(0,0)=0$. The result is a new polynomial $B^{\prime}(x, y)=B(x, y)+Q(x, y)$. The fact that $Q(0,0)=0$ ensures preservation of our first invariant Inv-Secret.

We achieve this by adapting the bivariate 0 -sharing technique (BivariateZeroShare) presented in Section 4 to handle active adversaries. Recall that BivariateZeroShare comprises two steps. First, a univariate 0 -sharing subroutine generates shares of the number 0 . These shares are then re-shared in a second step resulting in a sharing of $Q(x, y)$ among $C^{\prime}$.

By the end of the previous, i.e., Share Reduction phase, every node $\mathcal{U}_{j}^{\prime}$ in the set of $2 t+1$ nodes $\mathcal{U}^{\prime}$ holds a reduced share $B(x, j)$. Now, by the end of the current, i.e., Proactivization phase, we update these reduced shares by adding $Q(x, j)$ from the generated bivariate polynomial $Q(x, y)$.

The protocol starts by invoking the 0 -sharing subroutine UnivariateZeroShare introduced previously, which is the first step of BivariateZeroShare. Specifically, $(2 t, 2 t+1)$-UnivariateZeroShare is run among $\mathcal{U}^{\prime}$ to generate shares $s_{j}$ at each $\mathcal{U}_{j}^{\prime}$. To handle active adversaries, $\mathcal{U}_{j}^{\prime}$ sends a commitment to the share, $g^{s_{j}}$, to all other nodes in $\mathcal{U}^{\prime}$ (where $g$ is a publicly known generator). Lagrange coefficients $\left\{\lambda_{j}^{2 t}\right\}_{j}$ can be precomputed to interpolate and verify if the shares form a 0 -sharing, $\sum_{j=1}^{2 t+1} \lambda_{j}^{2 t} s_{j}=0$. Translating it to the commitments, all nodes check the following:
\footnotetext{
${ }^{2}$ Dimension-switch can be thought as a resharing of the shares. The zero points on full shares $B(i, 0)$ i.e., shares of the secret $s$, are reshared.
}

$$
\begin{equation*}
\prod_{j=1}^{2 t+1}\left(g^{s_{j}}\right)^{\lambda_{j}^{2 t}}=1 \tag{1}
\end{equation*}
$$

Then, $\mathcal{U}_{j}^{\prime}$ generates a random degree- $t$ univariate polynomial $R_{j}(x)$ that encodes the node's share $s_{j}$, i.e., $R_{j}(0)=s_{j}$. Together, the $2 t+1$ polynomials uniquely define a 0 -hole bivariate polynomial $Q(x, y)$ such that $\left\{Q(x, j)=R_{j}(x)\right\}_{j=1}^{2 t+1} . \mathcal{U}_{j}^{\prime}$ also updates the reduced share, $B^{\prime}(x, j)=B(x, j)+R_{j}(x)$. Points on $B^{\prime}(x, j)$ will be distributed to the entire committee $C^{\prime}$ in the next phase of Opt-CHURP. (We make a modification to BivariateZeroShare: In the re-sharing step of BivariateZeroShare, points on $Q(x, j)$ were distributed directly.)

Each $\mathcal{U}_{j}^{\prime}$ sends constant-size information to other nodes off-chain enabling verification of the above step. Let $Z_{j}(x)=R_{j}(x)-s_{j}$ denote a 0 -hole polynomial, the commitment to $Z_{j}(x), C_{Z_{j}}$, and a witness to the evaluation at zero are distributed enabling verification of the statement: $Z_{j}(0)=0$; equivalent to $R_{j}(0)=s_{j}$. The commitment to the updated reduced share $B^{\prime}(x, j)$ is also distributed. Since $B^{\prime}(x, j)=$ $B(x, j)+Z_{j}+s_{j}$, the homomorphic property of the commitment scheme allows other nodes to verify if $C_{B^{\prime}(x, j)}=C_{B(x, j)} \times C_{Z_{j}} \times C_{s_{j}}$ where $C_{s_{j}}=g^{s_{j}}$ and the other two were received previously.

In total, each node $\mathcal{U}_{j}^{\prime}$ generates the following set of commitment and witness information during Opt-Proactivize, $\left\{g^{s_{j}}, C_{Z_{j}}, W_{Z_{j}(0)}, C_{B^{\prime}(x, j)}\right\}$. While this set is transmitted off-chain to all nodes in the full committee $C^{\prime}$, a hash of it is published onchain. The received commitments can then be verified using the published hash, thereby ensuring that everyone receives the same commitments. Note that the set of commitments is sent to $C^{\prime}$ instead of just the subset $\mathcal{U}^{\prime}$ to preserve the invariant Inv-Comm, i.e., ensure that all nodes hold KZG commitments to the updated reduced shares.

The verification mechanisms used in this protocol are sufficient to detect any faulty behavior, although they do not identify which nodes are faulty. Thus, the adversary can disrupt the protocol without revealing his / her nodes. For example, it could send corrupt commitments to nodes selectively. Although the published hash reveals this, a verifiable accusation cannot be made since the commitments were sent off-chain. Another example would be a corrupt node sending points from a non-0-hole polynomial in the UnivariateZeroShare protocol. Again, we detect such a fault but cannot identify which nodes are faulty. So detection of a fault simply leads to a switch to the pessimistic path, Exp-CHURP-A. While Exp-CHURP-A is capable of identifying misbehaving nodes, note that we do not retroactively identify the faulty nodes from Opt-CHURP.

The protocol Opt-Proactivize is formally specified in Fig. 12. By the end of this, if no faults are detected, each $\mathcal{U}_{j}^{\prime}$ holds $B^{\prime}(x, j)$. The invariants Inv-Secret and Inv-Comm hold as $s=B^{\prime}(0,0)$ and all of $C^{\prime}$ hold the KZG commitments respectively. In the next phase, we preserve the other invariant Inv-State.

Communication complexity: Each node in $\mathcal{U}^{\prime}$ publishes a hash onchain and transmits $O(t)$ data off-chain. Hence, Opt-Proactivize incurs $O(t)$ on-chain and $O\left(t^{2}\right)$ off-chain cost.

5.3.3 Full Share Distribution (Opt-ShareDist). In the final phase, full shares are distributed to all members of the new committee, thus preserving the Inv-State invariant. A successful completion of this phase marks the end of handoff.

By the end of the previous phase, each $\mathcal{U}_{j}^{\prime}$ in the chosen subset of nodes $\mathcal{U}^{\prime} \subseteq C^{\prime}$ holds a new reduced share $B^{\prime}(x, j)$.

\begin{tabular}{cccc}
\hline Protocol & On-chain, Off-chain & Threshold & Optimistic \\
\hline Opt-CHURP & $O(n), O\left(n^{2}\right)$ & $t<n / 2$ & Yes \\
Exp-CHURP-A & $O\left(n^{2}\right), \mathrm{n} / \mathrm{a}$ & $t<n / 2$ & No \\
Exp-CHURP-B & $O\left(n^{2}\right), \mathrm{n} / \mathrm{a}$ & $t<n / 3$ & No \\
\hline Opt-Schultz-MPSS & $O(n), O\left(n^{4}\right)$ & $t<n / 3$ & Yes \\
Schultz-MPSS & $O\left(n^{2}\right), O\left(n^{4}\right)$ & $t<n / 3$ & No \\
\hline
\end{tabular}

Table 2: On-chain costs and Off-chain costs for the dynamic setting. An optimistic protocol ends successfully only if no faulty behavior is detected. $\mathrm{n} / \mathrm{a}$ indicates Not Applicable.

Now, $\mathcal{U}_{j}^{\prime}$ distributes points on $B^{\prime}(x, j)$, allowing computation of full shares $B^{\prime}(i, y)$ by all members of $C^{\prime}$-we make a dimension-switch from the degree- $2 t$ dimension of $B^{\prime}(x, y)$ to the degree- $t$ dimension Specifically, each $C_{i}^{\prime}$ receives $2 t+1$ points $\left\{B^{\prime}(i, j)\right\}_{j=1}^{2 t+1}$, which can be interpolated to compute $B^{\prime}(i, y)$, its full share. This is made verifiable by sending witness along with the points.

Since the point distribution is off-chain, a faulty node can send corrupt points without getting identified similar to the previous phase. In this event, we switch to the pessimistic path Exp-CHURP-A without identifying which nodes are faulty.

The protocol Opt-ShareDist is formally specified in Fig. 13. If all nodes receive correct points, this phase ends successfully and the optimistic path ends. The remaining invariant Inv-State is fulfilled as each node in $C^{\prime}$ receives a full share, and hence the system returns to the steady state. After a successful completion of CHURP, we require that members of the old committee $C$ delete their old full shares and members of $\mathcal{U}^{\prime}$ delete their new reduced shares.

Communication complexity: Each node in $C^{\prime}$ receives $2 t+1$ points, thus Opt-ShareDist incurs $O(n t)$ off-chain cost.

Each of the three phases in Opt-CHURP (and thus Opt-CHURP itself) incur no more than $O(n)$ on-chain and $O\left(n^{2}\right)$ off-chain cost. In terms of round complexity, it completes in three rounds (one for each phase) that does not depend on the committee size. Due to lack of space, we reiterate that the pessimistic paths of CHURP are discussed in Appendix C. Table 2 compares on-chain and off-chain costs of the three paths of CHURP and Schultz-MPSS [72], the latter will be explained in more detail in Section 6.3.1.

Theorem 1. Protocol Opt-CHURP is a dynamic-committee proactive secret sharing scheme by Definition 1.

We present the security proof in Appendix A.

Notes on the synchronicity assumptions. As discussed in Section 2, CHURP works in the synchronous model and assumes a latency bound for both on-chain and off-chain communication. While the for mer is a well-accepted assumption (e.g., see [54, 63, 79]), the latter is assumed by the blockchain consensus protocol itself, as the required difficulty of proof-of-work is dependent on the maximum network delay [62]. However, we emphasize that synchronicity for off-chain communication is needed only for performance, not for liveness or safety of the full protocol. In the optimistic path, if messages take longer to deliver, a fault is detected and the protocol switches to the pessimistic path. After that, nodes communicate via the on-chain channel only.

\subsection{Change of threshold}

Thus far we have focused on schemes that allow the committee size to change while the threshold $t$ remains constant. We now briefly describe how to enable an old committee with threshold $t_{e-1}$ (i.e. the adversary can corrupt up to $t_{e-1}$ nodes) to hand off shares to a new committee with a different threshold $t_{e}$.

Generally, we follow the same methodology as that of $[58,72]$. To increase the threshold (i.e., $t_{e}>t_{e-1}$ ), the new committee generates a ( $t_{e}, 2 t_{e}$ )-degree zero-hole polynomial $Q(x, y)$ so that the proactivized sharing has threshold $t_{e}$. To reduce the threshold (i.e., $t_{e}<t_{e-1}$ ), the old committee creates $2 \times\left(t_{e-1}-t_{e}\right)$ virtual servers that participate in the handoff as honest players, but expose their shares publicly. At the end of the handoff, the new commitment incorporates the virtual servers' shares to form a sharing of threshold $t_{e}$ in a similar process as the public evaluation scheme in [58].

To make changes of the threshold verifiable, we also need to extend the KZG commitment scheme with the degree verification functionality such that given a commitment $C_{\phi, d}$ to a polynomial $\phi$, it can be publicly verified that $\phi$ is at most $d$-degree. Our extension relies on the $q$-power knowledge of exponent ( $q$-PKE [45]) assumption. Due to lack of space, we refer readers to Appendix E for more details.

\subsection{State Verification (StateVerif)}

Both Opt-CHURP and Exp-CHURP-A make use of the KZG commitment scheme, which requires a trusted setup phase and its security (binding property) relies on the $t$-SDH assumption. Now, we devise a hedge against these-a verification phase that relies only on discrete log assumptions. At a high level, StateVerif includes checks to ensure that the two important invariants, Inv-Secret and Inv-State, hold, without using the KZG commitments on-chain.

Checking Inv-Secret: Assume that the commitment to the secret $g^{s}$ is on-chain from the beginning (done as part of the setup phase). Recall that at the end of Opt-CHURP or Exp-CHURP-A, each new committee node $C_{i}^{\prime}$ holds a full share $B^{\prime}(i, y)$. The secret can also be computed from the zero points of the full shares, $s=\sum_{i=1}^{n} \lambda_{i} B^{\prime}(i, 0)$, where $n=\left|C^{\prime}\right|$ and $\lambda_{i}=\lambda_{i}^{n-1}$ as defined in Eq. (1). Each $C_{i}^{\prime}$ computes $s_{i}=B^{\prime}(i, 0)$ and publishes $g^{s_{i}}$. All nodes verify that $\operatorname{Inv}$-Secret remains intact by checking $g^{s}=\prod_{i=1}^{n}\left(g^{s_{i}}\right)^{\lambda_{i}}$.

Checking Inv-State: In this check, we ensure that the bivariate polynomial $B^{\prime}(x, y)$ is of degree $\langle t, 2 t\rangle$. We achieve this by checking that the $2 t+1$ reduced shares $\left\{B^{\prime}(x, j)\right\}_{j \in[2 t+1]}$ are of degree $t$. We build an efficient procedure that reduces the checks to a single check through a random linear combination. If the degree of $P_{r}(x) \stackrel{\text { def }}{=} \sum_{j=1}^{2 t+1} r_{j} B^{\prime}(x, j)$ is $t$, where $r_{j}$ s are chosen randomly, then with high probability, the degree of all $B^{\prime}(x, j)$ is $t$. It is important that the adversary does not know the randomness a priori, as adversarial nodes can then choose reduced shares of degree $>t$ (in the proactivization phase) in such a way that the higher degree coefficients cancel in the linear combination. In practice, $r_{j} \mathrm{~s}$ can be obtained from a public source of randomness [15].

Each $C_{i}^{\prime}$ computes $s_{i}^{\prime}=P_{r}(i)=\sum_{j=1}^{2 t+1} r_{j} B^{\prime}(i, j)$ and publishes $g^{s_{i}^{\prime}}$ on-chain. All nodes now compute powers of the coefficients of $P_{r}$. Let $P_{r}(x)=\sum_{j=1}^{n} a_{j} x^{j}$, then $a_{j}=\sum_{i=1}^{n} \lambda_{i j} P_{r}(i)$, where $\lambda_{i j}$ are Lagrange coefficients (an extension of Eq. (1)). Therefore, $g^{a_{j}}=\prod_{i=1}^{n}\left(g^{s_{i}^{\prime}}\right)^{\lambda_{i j}}$. All nodes check $\forall j>t, g^{a_{j}}=1$, thus $P_{r}(x)$ is $t$-degree.

The two checks above incur $O(n)$ on-chain cost in total, thus StateVerif is highly efficient. StateVerif can fail due to two possible reasons: either the commitments are computed incorrectly by adversarial nodes, or the assumptions in the KZG scheme fail. Additional tests need to be performed to determine the cause of failure, these incur $O\left(n^{2}\right)$ on-chain cost and are discussed in Appendix C.2. If adversarial nodes are detected, the protocol expels these nodes and switches to Exp-CHURP-A. On the other hand, if KZG assumptions fail, the protocol switches to Exp-CHURP-B.

\section{CHURP IMPLEMENTATION \& EVALUATION}

We now report on an implementation and evaluation of CHURP, including a comparison with the state-of-the-art alternative, SchultzMPSS [72]

\subsection{Implementation}

We implemented Opt-CHURP in about 2,100 lines of Go and the code is available at https://www.churp.io. Our implementation uses the GNU Multiprecision Library [3] and the Pairing-Based Cryptography Library [5] for cryptographic primitives, and gRPC [4] for network infrastructure.

For polynomial arithmetic, we used the polynomial ring $\mathbb{F}_{p}[x]$ for a 256-bit prime $p$. For the KZG commitment scheme, we used a type A pairing on an elliptic curve $y^{2}=x^{3}+x$ over $\mathbb{F}_{q}$ for a 512-bit $q$. The order of the EC group is also $p$. We use SHA256 for hashing.

Blockchain Simulation: CHURP can be deployed on both permissioned and permissionless blockchains. We abstract away the specific choice and simulate one using a trusted node. Note that when deployed in the wild, writing to the blockchain would incur an additional constant latency.

\subsection{Evaluation}

In our evaluation, experiments are run in a distributed network of up to 1000 EC 2 c 5 . large instances, each with 2 vCPU and 4GB of memory. Each instance acts as a node in the committee and the handoff protocol is executed assuming a static committee. All experiments are averaged over 1000 epochs, i.e., 1000 invocations of Opt-CHURP. We measure three metrics for each protocol epoch: the latency (the total execution time), the on-chain complexity (the total bytes written to the blockchain (i.e. the trusted node)), and the off-chain complexity (the total bytes transmitted between all nodes). The evaluation results are presented below.

Latency: In the first set of experiments, all EC2 instances belong to the same region, also referred to as the LAN setting. This setting is useful to understand the computation time of Opt-CHURP, results are presented in Fig. 5. The experimental results show a quadratic increase consistent with the $O\left(n^{2}\right)$ asymptotic computational complexity of Opt-CHURP and suggests a low constant, e.g., for a committee of size 1001 the total protocol execution time is only about 3 minutes (Fig. 5b). As noted before, this does not include the additional latency for on-chain writes. Note that Opt-CHURP involves only 1 on-chain write per node which happens at the end of Opt-Proactivize, and in Ethereum currently each write takes about 15 seconds. Fig. 5b also shows that among the three phases, Opt-ShareDist dominates the execution time due to the relatively expensive $O(n)$ calls to KZG's

![](https://cdn.mathpix.com/cropped/2024_07_17_ad78533e3470f9e4f2abg-10.jpg?height=401&width=767&top_left_y=285&top_left_x=1102)

(a) Latency for the LAN (left bar) and WAN (right bar) setting with committee sizes 11-101.

![](https://cdn.mathpix.com/cropped/2024_07_17_ad78533e3470f9e4f2abg-10.jpg?height=407&width=786&top_left_y=775&top_left_x=1103)

(b) Latency for the LAN setting with committee size 101-1001.

Figure 5: Latency

CreateWitness per node. (CreateWitness involves $O(n)$ group element exponentiation, thus total $O\left(n^{2}\right)$ computation.)

In the second set of experiments, we select EC2 instances across multiple regions in US, Canada, Asia and Europe, also referred to as the WAN setting. In this setting the network latency is relatively unstable, although even in the worst-case it is still sub-second. Hence, during a handoff of Opt-CHURP in the WAN setting, we expect a constant increase in the latency over the LAN setting. Moreover, we expect this constant to be relatively small compared to the time spent in computation. We validate our hypothesis-for a committee size of 100 , the WAN latency is 4.54 seconds while the LAN latency is 2.92 seconds (Fig. 5a), i.e., the additional time spent in network latency is around 1.6 sec and constant across different committee sizes as expected. Note that we were unable to execute experiments in the WAN setting for committee sizes beyond 100 due to scaling limitations in AWS. (We plan to get around this soon.)

On-chain communication complexity: Opt-CHURP incurs a linear on-chain communication complexity- $n$ hashes, i.e. $32 n$ bytes, are written to the blockchain in each handoff.

Off-chain communication complexity: Fig. 6 compares the off-chain complexity for different committee sizes for Opt-CHURP and [72], a discussion about the comparison is in Section 6.3.1. Now, we discuss the off-chain costs of Opt-CHURP. The concrete performance numbers are consistent with the expected $O\left(n^{2}\right)$ complexity.

The off-chain data transmitted per node includes: $2 n$ (polynomial point, witness) pairs in the share reduction and the share distribution phase, and $n$ elements of $\mathbb{F}_{p}$ in the proactivization phase; each node also sends 1 commitment to share, 3 commitments to polynomials, and 1 witness. With aforementioned parameters, a commitment to a $t$-degree polynomial is of size 65 B (with compression) and points on

![](https://cdn.mathpix.com/cropped/2024_07_17_ad78533e3470f9e4f2abg-11.jpg?height=401&width=811&top_left_y=282&top_left_x=191)

Figure 6: Concrete off-chain communication complexity for Opt-CHURP and Schultz-MPSS, with log-scale y-axis. Points show experimental results; expected polynomial curves (respectively quadratic and quartic) are also shown.

polynomial are of size 32B. For example, for $t=50$ and $n=101$, the off-chain complexity of Opt-CHURP is about $226 n^{2}+325 n \approx 2.3 \mathrm{MB}$ In Fig. 6, the expected curve is slightly below the observed data points due to trivial header messages unaccounted in the above calculations.

As we'll show now, the above is about 2300 x lower than the communication complexity of the state of the art.

\subsection{Comparison with other schemes}

6.3.1 Schultz's MPSS. The Mobile Proactive Secret Sharing (MPSS) protocol of Schultz et al. [72], referred to as Schultz-MPSS hereafter, achieves the similar goal as CHURP in asynchronous settings, assuming $t<n / 3$. Compared to [72], Opt-CHURP achieves an $O\left(n^{2}\right)$ improvement for off-chain communication complexity. To evaluate the concrete performance, we also implemented the optimistic path of Schultz-MPSS (Section 5 of [72]) and evaluated the communication complexity empirically.

Asymptotic improvement: Schultz-MPSS extends the usage of expensive blinding polynomials introduced by Herzberg et al. [48] to enable a dynamic committee membership. We recall briefly the asymptotic complexity of Schultz-MPSS and refer readers to [72] for details. Each node in the old committee generates a proposal of size $O\left(n^{2}\right)$ and send it to other nodes, resulting in an $O\left(n^{4}\right)$ off-chain communication complexity in total. Each node then validates the proposals and reaches consensus on the set of proposals to use by sending $O(n)$ accusations to the primary, incurring a $O\left(n^{2}\right)$ on-chain communication complexity. In the optimistic case where no accusation is sent-labelled Opt-Schultz-MPSS-the consensus publishes $O(n)$ hashes of proposals on chain and thus only incurs $O(n)$ on-chain communication complexity.

Table 2 compares the asymptotic communication complexity of Schultz-MPSS and CHURP. Schultz-MPSS has the same on-chain complexity as CHURP, but is $O\left(n^{2}\right)$ more expensive for off-chain.

Performance evaluation: We implemented the optimistic path of Schultz-MPSS in about 3,100 lines of Go code. To adapt SchultzMPSS to the blockchain setting, we replace the BFT component of Schultz-MPSS with a trusted node. Fig. 6 compares the off-chain communication complexity of Opt-Schultz-MPSS and Opt-CHURP.

For practical parameterizations, our experiments show that OptCHURP can incur orders of magnitude less (off-chain) communication complexity than Opt-Schultz-MPSS. For example, for a committee of size 100, the off-chain complexity of Schultz-MPSS is

\begin{tabular}{lll}
\hline & On-chain & Transaction Ghosting \\
\hline Bandwidth (KB/sec) & $\leq 6.4$ & $32.3(9.31)$ \\
Latency (sec) & varies (Fig. 7) & $1.09(0.82)$ \\
Message transmission cost (USD/MB) & varies (Fig. 7) & $\$ 0.06(\$ 0.02)$ \\
Transaction delivery rate & $100 \%$ & $92.2 \%(14.2 \%)$ \\
\hline
\end{tabular}

Table 3: Comparison between communication via the Ethereum blockchain and via Transaction Ghosting. Numbers in parentheses are standard deviations. The cost for Transaction Ghosting is based on an initial gas price of 1 GWei. See Section 7.3 for details.

$53.667 n^{4} \approx 5.3 \mathrm{~GB}$, whereas that for Opt-CHURP is only 2.3 MB , a 2300x improvement! (If $n \geq 65$, the improvement is at least three orders of magnitude.) Since Schultz-MPSS incurs excessive (GB) off-chain cost, we do not run it for committee sizes beyond 100 .

6.3.2 Baron et al. [12]. Baron et al. devise a batched secret-sharing scheme that incurs $O\left(n^{3}\right)$ cost to transfer $O\left(n^{3}\right)$ secrets from an old to a new committee. In the single secret setting of CHURP, [12] achieves worse asymptotic cost than CHURP's optimistic path ( $O\left(n^{3}\right)$ vs $O\left(n^{2}\right)$ ) and equivalent in the pessimistic case. The asymptotic cost, though, masks the much worse practical performance caused by the use of impractical techniques to boost corruption tolerance. The implications are twofold. First, their protocol only works when the committee size is large (hundreds to thousands as we explain below), whereas CHURP works for arbitrary committee sizes. Second, even with a large committee, their protocol requires large subgroups of nodes (hundreds to thousands) to run maliciously-secure MPC, making their protocol significantly more expensive in practice.

The bottleneck in [12] lies in the use of virtualization techniques to achieve corruption threshold close to $t<n / 2$. Virtualization involves two steps: first, the committee of size $n$ is divided into $n$ virtual groups of size $s<n$; then each group is treated as a node in the committee to execute the protocol using MPC. [12] uses the group construction techniques of [26] that only work for large committees: for a fixed $\epsilon>0$, to achieve a corruption threshold $t<(1 / 2-\epsilon) n$, the size of the constructed group is $16 / \epsilon^{2}$ (See Appendix B. 2 of [26]). We want $\epsilon$ to be small, e.g., $\epsilon=0.01$-yielding $t$ only slightly worse than CHURP. This, however, causes the group size to explode to $s=160,000$. Even choosing a moderate $\epsilon$, say $\epsilon=1 / 6$-yielding $t<n / 3$ which is worse than CHURP, still requires a group of size $s=576$, meaning [12] needs to be run using maliciously-secure MPC among $n>576$ groups of 576 nodes each, making it extremely impractical.

\section{POINT-TO-POINT TECHNIQUE DETAILS}

CHURP takes advantage of a hybrid on-chain / off-chain communication model to minimize communication costs. A blockchain is used to reach consensus on a total ordering of messages, while much cheaper and faster off-chain P2P communication transmits messages with no ordering requirement.

Off-chain P2P channels can be implemented in different ways depending on the deployment environment. However, in a decentralized setting, establishing direct off-chain connection between nodes is undesirable, as it would compromise nodes' anonymity. Revealing network-layer identities (e.g., IP addresses) would also be dangerous, as it could lead to targeted attacks. One can instead use anonymizing overlay networks, such as Tor-but at the cost of considerable additional setup cost and engineering complexity.

Alternatively, off-chain channels can be implemented as an overlay on existing blockchain infrastructure. In this section, we present Transaction Ghosting, a technique for cheap P2P messaging on a blockchain. The key trick to reduce cost is to overwrite transactions so that they are broadcast, but subsequently dropped by the network. Most of these transactions-and their embedded messages-are then essentially broadcast for free. We focus on Ethereum, but similar techniques can apply to other blockchains, e.g., Bitcoin.

\subsection{Transaction Ghosting}

A (simplified) Ethereum transaction $\mathrm{tx}=(n, m, g)$ includes a nonce $n$, payload $m$, and a per-byte gas price $g$ paid to the miner of $t x$. For a basic ("send") transaction, Alice pays a miner $f_{0}+|m| \times g$, where $f_{0}$ is a base transaction cost and $|m|$ is the payload size. (We make this more precise below.)

Alice sends tx to network peers, who add tx to their pool of unconfirmed transactions, known as the mempool [61]. They propagate tx so that it can be included ultimately in all peers' view of the mempool. tx remains in the mempool until a miner includes it in a block, at which point it is removed and $f_{0}+|m| \times g$ units of currency is transferred from Alice to the miner

The key observation is, until tx is mined, Alice can overwrite it with another transaction $t x^{\prime}$. When this happens, $t x$ is dropped from the mempool. Thus, both $t x$ and $t x^{\prime}$ are propagated to all nodes, but Alice only pays for $t x^{\prime}$, i.e., $t x$ is broadcast for free.

Two additional techniques can further reduce costs. Alice can embed $m$ in tx only, putting no message data in $\mathrm{tx}^{\prime}$. She then only pays nothing for the data containing $m$, only the cost associated with $\mathrm{tx}^{\prime}$. Additionally, this technique generalizes to multiple overwrites i.e., Alice can embed a large message $m$ in multiple transactions $\left\{\operatorname{tx}_{i}\right\}_{i \in[k-1]}$, which is useful given bounds (e.g., 32 kB in Ethereum) on transaction sizes. Alice will still pay only the cost of the final transaction $\mathrm{tx}_{k}$.

\subsection{Choosing overwrite rate $k$}

An optimal strategy is to make $k$ as high as possible, i.e., overwrite many times. Ethereum, though, imposes a constraint on overwriting: the sender must raise the transaction fee in a fresh transaction by at least a minimum fraction $\rho$. (In Ethereum clients, $\rho$ ranges from $10 \%$ to $12.5 \%$.

Here we determine the optimal value of $k$. Recall that the fee for a transaction with $|m|$ bytes of data is $f=f_{0}+g \times|m|$, for constants $f_{0}$ and $g$. Overwriting transactions with a fractional fee increase of $\rho$ results in an average per-byte fee of $\frac{f \times \rho^{k}}{(1+k) \times|m|}$ for $k$ overwritings, as suming the $k$ th transaction gets mined. In the worst case, where $\rho=$ $12.5 \%$, the optimal strategy is to overwrite $k=7$ times, yielding average cost $0.29 \times \frac{f}{|m|}$ per byte, about $70 \%$ less than without overwriting

Moreover, if we send the first $k-1$ transactions with $|m|$ bytes of data and the last one empty, the average cost is driven down to $\frac{f_{0} \times \rho^{k}}{|m| \times k}$ per byte (because one only pays for the last empty transaction). As a concrete example, for $k=7,|m|=31 K$, and $f=(21,000+68 \times m) \times 1$ $\mathrm{GWei}^{3}$, sending 1 MB data costs about $\$ 0.06$.
\footnotetext{
${ }^{3} \mathrm{GWei}$ is an unit in Ethereum. 1 GWei is $10^{-9}$ Ether.
}

The above analysis assumes the $k$ th transaction can always successfully overwrite previous ones, which happens in our experiments for two reasons. First, the $k$ th transaction is smaller and higher-priced, thus preferred by miners; second, previous transactions usually remain pending for a long time (tens of minutes or longer), always allowing enough time for the $k$ th to fully propagate.

\subsection{Experiments}

We validate our ideas experimentally on the Ethereum blockchain (mainnet). The sender and receiver are full nodes connected to the Ethereum P2P network-with no out-of-band channel. The goal is for the sender to transmit messages to the receiver by embedding them in pending transactions. To overwrite a pending transaction in Ethereum, the sender reuses the same nonce and raises the gas price.

In our experiments, we rewrite $k=7$ times. Each of the first 7 transactions contains 31 KB of data and the 8 th is empty. A total of approximately 100 MB data is successfully transmitted in 4,200 transactions, in about 1 hour. Table 3 summarizes the results of our experiments, which we now discuss.

Bandwidth:DoS prevention measures and network latency in Ethereum cause overly frequent overwritten transactions to drop. Experimentally, we can propagate overwritten transactions at a rate of just under once a second, yielding approximate bandwidth $32.3 \mathrm{~KB} / \mathrm{s}$, as the maximum permitted per-transaction data is 32 KB [43]. While this suffices for CHURP, we belived more engineering would yield higher bandwidth. Studies of blockchain arbitrage [38] show that arbitrageurs can overwrite transactions in hundreds of milliseconds.

We emphasize that the shown bandwidth is per channel. One can establish $N$ concurrent channels by overwriting $N$ transactions simultaneously.

Message-transmission cost: Transaction costs for message delivery in Transaction Ghosting are extremely low: $\$ 0.06$ per megabyte on average, with gas price 1 GWei . The gas price should be chosen minimum required to get transactions relayed by peer nodes. Empirically of late, a gas price between 1 to 2 GWei offers good delivery rate, which we now explain.

Transaction delivery rate: Although a sender can make sure overwriting succeeds in her mempool, overwritten transactions are not guaranteed to arrive on the receiver's side. Possible reasons are an overloaded mempool [61], network congestion and/or out-of-order delivery. Generally transactions with a higher transaction fee are relayed preferentially by peer nodes, and less frequently dropped. The 8th transaction in our rewriting sequence has the highest fee and the smallest payload, and is always delivered in our experiments.

Overall, we observe an average transaction delivery rate of $91.9 \%$ in our experiments, or $\mathrm{a} \approx 9 \%$ loss rate. Our Transaction Ghosting is thus an erasure channel. A sender can either erasure-code $m$ to ensure full delivery without interaction with the receiver, or use a standard network retransmission protocol so the receiver can signal a delivery failure. These techniques are out of scope for our exploration here.

\subsection{Comparison to on-chain communication}

For comparison, we estimate the same metrics for on-chain communication, i.e. using the Ethereum blockchain as a message carrier. The results are summarized in Table 3.

![](https://cdn.mathpix.com/cropped/2024_07_17_ad78533e3470f9e4f2abg-13.jpg?height=377&width=791&top_left_y=278&top_left_x=209)

Figure 7: Tradeoff in latency vs. message transmission cost. The blue curve shows the observed on-chain tradeoff. The red dot at ( $0.06 \mathrm{~s}, 1.09 \mathrm{USD} / \mathrm{MB}$ ) corresponds to Transaction Ghosting.

An upper bound on the on-chain bandwidth is estimated assuming a 8 million block gas limit. Each block can hold at most three 32KB transactions, thus a total of 96 KB data every 15 seconds, or $6.4 \mathrm{~KB} / \mathrm{s}$.

The message transmission cost per megabyte is estimated as that of sending 32 transactions with 32 KB data in each, assuming an exchange rate of $1 \mathrm{ETH}=\$ 200$. The latency, i.e., the time between a transaction first appears in the mempool and the time it is mined, depends on the gas price and the network condition. A lower latency requires a higher gas price and thus a higher transmission cost. Several services such as $[1,2]$ collect metrics for gas price vs. latency tradeoff. We used [1] for our estimation. The tradeoff between latency and message transmission cost is shown in Fig. 7.

At the time of writing, gas prices in Ethereum have been consistently low for a period of approximately two months [33], preventing experimentation in a high-gas-price regime. We believe, however, that the same techniques would still work in such settings-with higher overall cost.

\section{RELATED WORK}

Verifiable Secret Sharing (VSS): Polynomial-based secret sharing was introduced by Shamir [73]. Feldman [34] and Pedersen [64] proposed an extension called verifiable secret sharing (VSS), in which dealt shares' correctness can be verified against a commitment of the underlying polynomial. In these schemes, a commitment to a degree- $t$ polynomial has size $O(t)$. The polynomial-commitment scheme of Kate et al. [50] (KZG) reduces this to $O(1)$, and is adopted for secret sharing in, e.g., [11], and in CHURP.

KZG hedge: Prior works [46] hedge against the failure of a commitment scheme (or a cryptosystem [13]) by creating hybrid schemes that combine multiple schemes, in contrast to CHURP's approach of using protocol tiers with different schemes in each tier. This approach coupled with novel, efficient detection techniques to switch between tiers (StateVerif), allows CHURP to include an efficient top tier (optimistic path). The notion of graceful degradation in the event of a failure appears in several works [13, 39, 71]-loosely similar to how CHURP degrades to a lower corruption threshold when the KZG scheme fails (exact notion hasn't appeared before).

Proactive Secret Sharing (PSS): Proactive security, the idea of refreshing secrets to withstand compromise, was first proposed by Ostrovsky and Yung [60] for multi-party computation (MPC). It was first adapted for secret sharing by Herzberg et al. [48], whose techniques continue to be used in subsequent works, e.g., $[16,24,40,47$,

\begin{tabular}{cccccc}
\hline Protocol & Dynamic & Adversary & Network & Threshold & Cost \\
\hline Herzberg et al. [48] & No & active & synch. & $t<n / 2$ & $O\left(n^{2}\right)$ \\
Cachin et al. [18] & No & active & asynch. & $t<n / 3$ & $O\left(n^{4}\right)$ \\
\hline Desmedt et al. [28] & Yes & passive & synch. & $t<n / 2$ & $O\left(n^{2}\right)$ \\
Wong et al. [77] & Yes & active & synch. & $t<n / 2$ & $\exp (n)$ \\
Zhou et al. [80] & Yes & active & asynch. & $t<n / 3$ & $\exp (n)$ \\
Schultz-MPSS [72] & Yes & active & asynch. & $t<n / 3$ & $O\left(n^{4}\right)$ \\
Baron et al. [12] & Yes & active & synch. & $t<n(1 / 2-\epsilon)$ & $O\left(n^{3}\right)$ \\
CHURP (this work) & Yes & active & synch. & $t<n / 2$ & $O\left(n^{2}\right)$ (optimistic) \\
& & & & & $O\left(n^{3}\right)$ (pessimistic) \\
\hline
\end{tabular}

Table 4: Comparison of Proactive Secret Sharing (PSS) schemesthose above the line do not handle dynamic committees while the ones below do so. Cost indicates the off-chain commn. complexity.

55, 58, 72], and in CHURP (in UnivariateZeroShare). As noted, a result of independent interest in our work is an $O(n)$ reduction in the off-chain communication complexity of [48]. (See Appendix D.)

All the above schemes assume a synchronous network model and computationally bounded adversary; CHURP does too, given its blockchain setting. PSS schemes have also been proposed in asynchronous settings $[18,72,80]$ and unconditional settings $[59,74]$. Nikov and Nikova [56] provide a survey of the different techniques used in PSS schemes along with some attacks (which CHURP addresses via its novel dimension-switching techniques).

Dynamic committee membership: Desmedt and Jajodia [28] propose a scheme that can change the committee and threshold in a secret-sharing system, but is unfortunately not verifiable. Wong et al. [77] build a verifiable scheme assuming that the nodes in the new committee are non-faulty. Subsequent works [12, 29, 80] build schemes that do not make such assumptions, but are impractical for our use-[80] incurs exponential communication cost, [29] incurs exponential computation cost, and [12] uses impractical virtualization techniques (See Section 6.3.2). Schultz et al. [72] were the first to build a practical scheme under an adversarial model similar to ours. While [72] incurs $O\left(n^{4}\right)$ off-chain communication cost, as Table 4 shows, CHURP improves it to worst-case $O\left(n^{3}\right)$ off-chain cost $\left(O\left(n^{2}\right)\right.$ in the optimistic case). We convert the on-chain cost incurred by CHURP to its equivalent off-chain cost in order to facilitate a comparison with prior work in the following manner: Instead of using a blockchain, use PBFT [19] to post messages on the bulletin board which incurs an extra $O(n)$ off-chain cost per bit.

Bivariate polynomials: Bivariate polynomials have been explored extensively in the secret-sharing literature, to build VSS protocols [18, 35], for multipartite secret-sharing [76], to achieve unconditional security [59], and to build MPC protocols [14, 42]. Prior to CHURP, few works $[30,70]$ have considered application of bivariate polynomials to dynamic committees, but these have been limited to passive adversaries. CHURP's novel use of dimension-switching provides security against a strong active adversary controlling $2 t$ nodes during the handoff. The dimension-switching technique applies well known resharing techniques [28, 42] via bivariate polynomials to switch between full and reduced shares.

0 -sharing, the technique of generating a 0 -hole polynomial has been widely used for proactive security since the work of [48]. As we explain before, prior works $[30,59,70]$ have naively extended these to the bivariate case leading to expensive 0 -sharing protocols. Instead, CHURP applies resharing techniques [28] to build an efficient bivariate 0 -sharing protocol.

CHURP's use of two sharings appears in some prior works $[66,68]$ (with largely differing goals and detail) where each node stores an additive share of the secret and a backup share of every other node's additive share. Proactivization is achieved by resharing the additive shares, in contrast to CHURP's approach of generating a shared polynomial explicitly which is then used to update the reduced shares. We note that adapting these techniques for use in CHURP is non-trivial, moreover, CHURP's bivariate 0 -sharing protocol has other uses as well, e.g., can reduce the off-chain cost of [48].

\section{ACKNOWLEDGEMENT}

This work was funded by NSF grants CNS-1514163, CNS-1564102, and CNS-1704615, as well as ARO grant W911NF16-1-0145 and support from IC3 industry partners.

\section{REFERENCES}

[1] [n.d.]. ETH Gas Station. https://ethgasstation.info/. (Accessed on 11/13/2018).

[2] [n.d.]. Ethereum Gas Price Tracker. https://etherscan.io/gastracker.

[3] [n.d.]. Go language interface to GMP - GNU Multiprecision Library.

[4] [n.d.]. gRPC: A high performance, open-source universal RPC framework

[5] [n.d.]. The PBC Go Wrapper. https://github.com/Nik-U/pbc.

[6] 2018. Decentralized Identity Foundation (DIF) homepage.

[7] 2018. uPort. https://www.uport.me/.

[8] Yazin Akkawi. 21 Dec. 2017. Bitcoin's Most Pressing Issue Summarized in Two Letters: UX. Inc. (21 Dec. 2017).

[9] Brian Armstrong. Feb. 25, 2018. Coinbase is not a wallet. https //blog.coinbase.com/coinbase-is-not-a-wallet-b5b9293ca0e7.

[10] Avi Asayag, Gad Cohen, Ido Grayevsky, Maya Leshkowitz, Ori Rottenstreich, Ronen Tamari, and David Yakira. 2018. Helix: a scalable and fair consensus algorithm. Technical Report. Technical report, Orbs Research.

[11] Michael Backes, Amit Datta, and Aniket Kate. 2013. Asynchronous computational VSS with reduced communication complexity. In CT-RSA. Springer

[12] Joshua Baron, Karim El Defrawy, Joshua Lampkins, and Rafail Ostrovsky. 2015. Communication-optimal proactive secret sharing for dynamic groups. In ACNS.

[13] Mihir Bellare, Zvika Brakerski, Moni Naor, Thomas Ristenpart, Gil Segev, Hovav Shacham, and Scott Yilek. 2009. Hedged public-key encryption: How to protect against bad randomness. In ASIACRYPT. Springer, 232-249.

[14] Michael Ben-Or, Shafi Goldwasser, and Avi Wigderson. 1988. Completeness theo rems for non-cryptographic fault-tolerant distributed computation. In ACM TOCS.

[15] Joseph Bonneau, Jeremy Clark, and Steven Goldfeder. 2015. On Bitcoin as a public randomness source. IACR ePrint Archive 2015 (2015), 1015

[16] Kevin D Bowers, Ari Juels, and Alina Oprea. 2009. HAIL: A high-availability and integrity layer for cloud storage. In 16th ACM CCS.

[17] Vitalik Buterin. 2014. Slasher: A punitive proof-of-stake algorithm. (2014).

[18] Christian Cachin, Klaus Kursawe, Anna Lysyanskaya, and Reto Strobl. 2002 Asynchronous verifiable secret sharing and proactive cryptosystems. In ACMCCS.

[19] Miguel Castro and Barbara Liskov. 2002. Practical Byzantine fault tolerance and proactive recovery. ACM TOCS (2002)

[20] David Chaum, Claude Crépeau, and Ivan Damgard. 1988. Multiparty uncondi tionally secure protocols. In ACM TOCS

[21] Raymond Cheng, Fan Zhang, Jernej Kos, Warren He, Nicholas Hynes, Noah Johnson, Ari Juels, Andrew Miller, and Dawn Song. 2019. Ekiden: A Platform for Confidentiality-Preserving, Trustworthy, and Performant Smart Contracts. In 2019 IEEE EuroS\&P.

[22] Konstantinos Christidis and Michael Devetsikiotis. 2016. Blockchains and smart contracts for the internet of things. Ieee Access (2016).

[23] Ronald Cramer, Ivan Damgård, and Ueli Maurer. 2000. General secure multi-party computation from any linear secret-sharing scheme. In EUROCRYPT

[24] Ronald Cramer, Rosario Gennaro, and Berry Schoenmakers. 1997. A secure and optimally efficient multi-authority election scheme. ETT (1997).

[25] Phil Daian, Rafael Pass, and Elaine Shi. 2016. Snow White: Provably Secure Proofs of Stake. Cryptology ePrint Archive, Report 2016/919

[26] Ivan Damgård, Yuval Ishai, Mikkel Krøigaard, Jesper Buus Nielsen, and Adam Smith. 2008. Scalable multiparty computation with nearly optimal work and resilience. In CRYPTO. Springer, 241-261.

[27] Yvo Desmedt and Yair Frankel. 1991. Shared generation of authenticators and signatures. In CRYPTO.

[28] Yvo Desmedt and Sushil Jajodia. 1997. Redistributing secret shares to new acces structures and its applications. Technical Report.

[29] Yvo Desmedt and Kirill Morozov. 2015. Parity check based redistribution of secret shares. In ISIT.
[30] Shlomi Dolev, Juan Garay, Niv Gilboa, and Vladimir Kolesnikov. 2009. Swarming secrets.

[31] Michael Egorov, MacLane Wilkison, and David Nuñez. 2017. Nucypher KMS: decentralized key management system. arXiv preprint arXiv:1707.06140 (2017).

[32] Ethereum. [n.d.]. Devp2p. https://github.com/ethereum/devp2p

[33] Etherscan. [n.d.]. Ethereum Average GasPrice Chart. https://etherscan.io/chart/ gasprice. [Online; accessed 2018].

[34] Paul Feldman. 1987. A practical scheme for non-interactive verifiable secret sharing. In FOCS

[35] Pesech Feldman and Silvio Micali. 1997. An optimal probabilistic protocol for synchronous Byzantine agreement. SIAM 7. Comput. (1997)

[36] Yair Frankel, Peter Gemmell, Philip D MacKenzie, and Moti Yung. 1997. Optimal-resilience proactive public-key cryptosystems. In FOCS.

[37] Yair Frankel, Peter Gemmell, Philip D MacKenzie, and Moti Yung. 1997. Proactive rsa. In CRYPTO.

[38] frontrun.me. [n.d.]. Visualizing Ethereum gas auctions. http://frontrun.me/.

[39] Georg Fuchsbauer. 2018. Subversion-zero-knowledge SNARKs. In IACR International Workshop on Public Key Cryptography. Springer, 315-347.

[40] Rosario Gennaro, Stanisław Jarecki, Hugo Krawczyk, and Tal Rabin. 1996. Robust threshold DSS signatures. In EUROCRYPT.

[41] Rosario Gennaro, Stanisław Jarecki, Hugo Krawczyk, and Tal Rabin. 1999. Secure distributed key generation for discrete-log based cryptosystems. In EUROCRYPT

[42] Rosario Gennaro, Michael O Rabin, and Tal Rabin. [n.d.]. Simplified VSS and fast-track multiparty computations with applications to threshold cryptography

[43] geth. [n.d.]. The maximum data size in a transaction is 32 KB . https://github.com ethereum/go-ethereum/blob/6a33954731658667056466bf7573ed1c397f4750, core/tx_pool.go\#L570.

[44] Yossi Gilad, Rotem Hemo, Silvio Micali, Georgios Vlachos, and Nickolai Zeldovich. 2017. Algorand: Scaling Byzantine Agreements for Cryptocurrencies. In SOSP.

[45] Jens Groth. 2010. Short pairing-based non-interactive zero-knowledge arguments. In ASIACRYPT

[46] Amir Herzberg. 2009. Folklore, practice and theory of robust combiners. Journal of Computer Security 17, 2 (2009), 159-189.

[47] Amir Herzberg, Markus Jakobsson, Stanislław Jarecki, Hugo Krawczyk, and Moti Yung. 1997. Proactive public key and signature systems. In $A C M C C S$.

[48] Amir Herzberg, Stanisław Jarecki, Hugo Krawczyk, and Moti Yung. 1995. Proactive secret sharing or: How to cope with perpetual leakage. In CRYPTO.

[49] Kames. 26 June 2018. The Basics of Decentralized Identity How Blockchain Technology \& Cryptographic Primitives Embolden the Future of Digital Identity.

[50] Aniket Kate, Gregory M Zaverucha, and Ian Goldberg. 2010. Constant-size commitments to polynomials and their applications. In ASIACRYPT.

[51] Aggelos Kiayias, Alexander Russell, Bernardo David, and Roman Oliynykov. 2017. Ouroboros: A provably secure proof-of-stake blockchain protocol. In CRYPTO.

[52] Sunny King and Scott Nadal. 2012. Ppcoin: Peer-to-peer crypto-currency with proof-of-stake. self-published paper, August (2012).

[53] Eleftherios Kokoris-Kogias, Enis Ceyhun Alp, Sandra Deepthy Siby, Nicolas Gailly, Linus Gasser, Philipp Jovanovic, Ewa Syta, and Bryan Ford. 2018. CALYPSO: Auditable Sharing of Private Data over Blockchains. Cryptology ePrint Archive

[54] A. Kosba, A. Miller, E. Shi, Z. Wen, and C. Papamanthou. 2016. Hawk: The Blockchain Model of Cryptography and Privacy-Preserving Smart Contracts. In IEEE $S \& P$.

[55] Haiyun Luo, Petros Zerfos, Jiejun Kong, Songwu Lu, and Lixia Zhang. 2002. Self-Securing Ad Hoc Wireless Networks. In ISCC.

[56] Ventzislav Nikov and Svetla Nikova. 2004. On proactive secret sharing schemes. In International Workshop on Selected Areas in Cryptography

[57] John P. Njui. [n.d.]. Coinbase Custody Service Secures Major Institutional Investor Worth $\$ 20$ Billion. Ethereum World News ([n. d.]).

[58] Mehrdad Nojoumian and Douglas R Stinson. 2013. On dealer-free dynamic threshold schemes. Adv. in Math. of Comm. (2013).

[59] Mehrdad Nojoumian, Douglas R Stinson, and Morgan Grainger. 2010. Unconditionally secure social secret sharing scheme. IET information security (2010).

[60] Rafail Ostrovsky and Moti Yung. 1991. How to withstand mobile virus attacks. In $A C M$ PODC.

[61] Parity. [n.d.]. Transaction Queue. https://wiki.parity.io/Transactions-Queue.

[62] Rafael Pass, Lior Seeman, and Abhi Shelat. 2017. Analysis of the blockchain protocol in asynchronous networks. In EUROCRYPT. Springer, 643-673.

[63] Rafael Pass and Elaine Shi. 2018. Thunderella: Blockchains with Optimistic Instant Confirmation. In EUROCRYPT.

[64] Torben Pryds Pedersen. 1991. Non-interactive and information-theoretic secure verifiable secret sharing. In CRYPTO.

[65] Giulio Prisco. 2015. Slock.it to Introduce Smart Locks Linked to Smart Ethereum Contracts, Decentralize the Sharing Economy. Bitcoin Magazine (2015).

[66] Bartosz Przydatek and Reto Strobl. 2004. Asynchronous proactive cryptosystems without agreement. In ASIACRYPT. Springer, 152-169.

[67] Michael O Rabin. 1983. Randomized byzantine generals. In FOCS.

[68] Tal Rabin. 1998. A simplified approach to threshold and proactive RSA. In CRYPTO

[69] Jeff John Roberts and Nicolas Rapp. 2017. Exclusive: Nearly 4 Million Bitcoins Lost Forever, New Study Says. http://fortune.com/2017/11/25/lost-bitcoins/

[70] Nitesh Saxena, Gene Tsudik, and Jeong Hyun Yi. 2005. Efficient node admission for short-lived mobile ad hoc networks. In 13th ICNP.

[71] Berry Schoenmakers, Meilof Veeningen, and Niels de Vreede. 2016. Trinocchio privacy-preserving outsourcing by distributed verifiable computation. In ACNS

[72] David A Schultz, Barbara Liskov, and Moses Liskov. 2008. Mobile proactive secret sharing. In $A C M$ PODC.

[73] Adi Shamir. 1979. How to share a secret. Commun. ACM (1979).

[74] Douglas R Stinson and Ruizhong Wei. 1999. Unconditionally secure proactive secret sharing scheme with combinatorial structures. In $S A C$

[75] Paul Syverson, R Dingledine, and N Mathewson. 2004. Tor: The secondgeneration onion router. In Usenix Security

[76] Tamir Tassa and Nira Dyn. 2009. Multipartite secret sharing by bivariate interpolation. Journal of Cryptology (2009).

77] Theodore M Wong, Chenxi Wang, and Jeannette M Wing. 2002. Verifiable secret redistribution for archive systems. In the 1st Security in Storage Workshop.

[78] Jay J Wylie, Michael W Bigrigg, John D Strunk, Gregory R Ganger, Han Kiliccote, and Pradeep K Khosla. 2000. Survivable information storage systems. In Computer (2000).

[79] Fan Zhang, Philip Daian, Gabriel Kaptchuk, Iddo Bentov, Ian Miers, and Ari Juels 2018. Paralysis Proofs: Secure Access-Structure Updates for Cryptocurrencies and More. Cryptology ePrint Archive, Report 2018/096.

[80] Lidong Zhou, Fred B Schneider, and Robbert Van Renesse. 2005. APSS: Proactive secret sharing in asynchronous systems. ACM TISSEC (2005).

[81] Guy Zyskind, Oz Nathan, et al. 2015. Decentralizing privacy: Using blockchain to protect personal data. In Security and Privacy Workshops.

\section{A SECURITY PROOF FOR Opt-CHURP}

Recall that a protocol for dynamic-committee proactive secret sharing satisfies secrecy and integrity. We prove secrecy first.

Secrecy. We consider the handoff protocol of one epoch first. As described in Section 5.3, Opt-CHURP consists of three phases: Opt ShareReduce, Opt-Proactivize and Opt-ShareDist. Other than the public inputs, the information obtained by the adversary $\mathcal{A}$ is:

\section{Opt-ShareReduce:}

- For all corrupt $\mathcal{U}_{j}$ in the previous handoff, reduced share $B(x, j)$.

- For all corrupt nodes $C_{i}$ in the old committee $\left\{B(i, j), W_{B(i, j)}\right\}_{j \in[2 t+1]}($ full share $B(i, y))$.

- For all corrupt $\mathcal{U}_{j}^{\prime}$ in the new committee selected to participate in the handoff, $\left\{B(i, j), W_{B(i, j)}\right\}_{i \in[2 t+1]}($ reduced share $B(x, j))$.

Opt-Proactivize:

- For all corrupt nodes $\mathcal{U}_{j}^{\prime}, s_{j}$ and $Q(x, j)=R_{j}(x)$.

- For all corrupt nodes $C_{i}^{\prime}$ in the new committee, $H_{j}$ and

$$
\left\{g^{s_{j}}, C_{Z_{j}}, W_{Z_{j}(0)}, C_{B^{\prime}(x, j)}\right\}
$$

Opt-ShareDist:

- For all corrupt $C_{i}^{\prime}$ in the new committee, $\left\{B^{\prime}(i, j), W_{B(i, j)}^{\prime}\right\}_{j \in[2 t+1]}$ The information above assumes the secrecy of our bivariate 0 -sharing protocol, which we explained in the main body. In addition, note that the public information posted on chain are all commitments of the polynomials. By the hiding property of the commitment scheme based on the discrete log assumption, the PPT $\mathcal{A}$ learns no extra information from these commitments. To prove secrecy, we have the following lemmas.

LEMMA 2. If $\mathcal{A}$ corrupts no more than $t$ nodes in the old committe node, and no more than $t$ nodes in $\mathcal{U}^{\prime}$, the information received by $\mathcal{A}$ in Opt-ShareReduce is random and independent of the secrets.

Proof. This is implied by the degree of the bivariate polynomial $B(x, y)$. In the worst case when all $t$ corrupted nodes are in $\mathcal{U}$ and $\mathcal{U}^{\prime}, \mathcal{A}$ learns $2 t$ reduced shares $B(x, j)$ and $t$ full shares $B(i, y)$. For a $\langle t, 2 t\rangle$-bivariate polynomial, any $t$ shares of $B(i, y)$ and $2 t$ shares of $B(x, j)$ are random and independent of $s=B(0,0)$.

Moreover, based on the discrete-log assumption, the proofs $W_{B(i, j)}$ are computationally zero-knowledge by the KZG scheme, and the PPT adversary cannot learn additional information from them.

LemmA 3. Given a bivariate 0 -sharing scheme with secrecy and integrity, if at least one node is honest in Opt-Proactivize, $Q(x, y)$ is randomly generated.

Proof. Any $2 t+1$ degree $t$ univariate polynomials $Q(x, j)$ uniquely define a $\langle t, 2 t\rangle$-bivariate polynomial. Therefore, as long as one node is honest and generates a random degree $t$ polynomial, $Q(x, y)$ is randomly generated to mask $B(x, y)$.

Similar to the proof above, the hashes and commitments do not leak additional information to a PPT adversary $\mathcal{A}$.

LemmA 4. If $\mathcal{A}$ corrupts no more than $t$ nodes in the new committee $C^{\prime}$, the information received by $\mathcal{A}$ in Opt-ShareDist is random and independent of the secret $s$.

Proof. ByLemma 2, $Q(x, y)$ is randomly generated, thus $B^{\prime}(x, y)=$ $B(x, y)+Q(x, y)$ is independent of $B(x, y)$. Regardless of the number of nodes corrupted by $\mathcal{A}$ in $\mathcal{U}^{\prime}, \mathcal{A}$ receives no more than $t$ out of $n^{\prime}$ shares of $B^{\prime}(i, y)$ in Opt-ShareDist. As the degree of $B^{\prime}(x, y)$ is $\langle t, 2 t\rangle$ and is independent of $B(x, y)$, these shares are random and independent of $s$. Again, the proofs in the second part do not leak additional information.

By Lemma 2, 3 and $4, \mathcal{A}$ does not learn any information about $s$ in consecutive epochs. The secrecy of the scheme follows by induction.

Integrity. For integrity, we have the following lemmas.

LemmA 5. After Opt-ShareReduce, at least $t+1$ honest nodes $\mathcal{U}_{j}^{\prime}$ can successfully reconstruct $B(x, j)$

Proof. As the number of nodes in the old committee $n \geq 2 t+1$, each node $\mathcal{U}_{j}^{\prime}$ receives at least $t+1$ correct shares of $B(i, j)$. As the degree on the first variable of $B(x, y)$ is $t, \mathcal{U}_{j}^{\prime}$ can reconstruct $B(x, j)$ successfully. Finally, as the number of nodes in $\mathcal{U}^{\prime}$ is $2 t+1$, there are at least $t+1$ honest nodes.

LemmA 6. Assuming the correctness of the bivariate 0-sharing scheme, after Opt-Proactivize, either honest nodes $\mathcal{U}_{j}^{\prime}$ hold the correct shares of $B^{\prime}(x, j)$ such that $B^{\prime}(0,0)=B(0,0)=s$ and their commitments $C_{B^{\prime}(x, j)}$ are on-chain, or at least $t+1$ honest nodes in $C^{\prime}$ output fail.

Proof. By line 15 in Figure $12,\left\{g^{s_{j}}, C_{Z_{j}}, W_{Z_{j}(0)}, C_{B^{\prime}(x, j)}\right\}$ is consistent with the hash $H_{j}$ posted on chain by $\mathcal{U}_{j}^{\prime}$. If $C_{Z_{j}}$ is not a univariate polynomial with constant term 0 , by line 16 , VerifyEval outputs false and $C_{i}^{\prime}$ outputs fail by the soundness of KZG. Otherwise, by the second check of line $16, C_{B^{\prime}(x, j)}$ is the commitment of a polynomial $B^{\prime}(x, j)$ with constant term $B(x, j)+s_{j}$. Finally, by the check of line 17, by the discrete-log assumption, $\sum_{j=1}^{2 t+1} s_{j} \lambda_{j}^{2 t}=0$. Therefore, $B^{\prime}(0,0)=B(0,0)$ because of the property of Lagrange coefficients.

By Lemma 5 and 6, if Opt-ShareReduce and Opt-Proactivize do not fail, all nodes $\mathcal{U}_{j}^{\prime}$ hold the correct shares of $B^{\prime}(x, j)$ such that $B^{\prime}(0,0)=B(0,0)=s$ and their commitments $C_{B^{\prime}(x, j)}$ are on the chain. In Opt-ShareDist, each node $C_{i}^{\prime}$ receives $2 t+1$ shares of $B^{\prime}(i, j)$ from all $\mathcal{U}_{j}^{\prime}$ s. By the soundness of the KZG scheme, if any of these shares

```
KZG Commitment
    (sk, pk)\leftarrowKeygen (1 \},q):\mathrm{ Select a bilinear group ( p, 爬,}\mp@subsup{\mathbb{G}}{T}{},e,g)\leftarrow\operatorname{BilGen(1\lambda) and s
    randomly in }\mp@subsup{\mathbb{Z}}{p}{*}.\mathrm{ Set sk =s and pk = g
    C
    ( }\phi(i),\mp@subsup{W}{i}{})\leftarrow\mathrm{ CreateWitness }(\phi(x),i,\textrm{pk}):\mathrm{ Compute }\phi(x)-\phi(i)=(x-i)w(x)\mathrm{ , set
    Wi=g
    \{ \text { True, False \}} \leftarrow \operatorname { V e r i f y E v a l ( } ( C _ { \phi } , i , \phi ( i ) , W _ { i } , \mathrm { pk } ) : \text { : output True if } e ( C _ { \phi } / g \mp@code { g ( i ) } , g ) =
    e ( g ^ { s - i } , W _ { i } ) \text { . Otherwise, output False}
```

Figure 8: Protocols of KZG commitment scheme.

```
(2t,2t+1)-UnivariateZeroShare
Input: t, set of }2t+1\mathrm{ nodes {}{\mp@subsup{\mathcal{U}}{j}{}\mp@subsup{}}{j\in[2t+1]}{
Output: Each node }\mp@subsup{\mathcal{U}}{j}{}\mathrm{ outputs a share }\mp@subsup{s}{j}{}=P(j)\mathrm{ for randomly
generated degree-2t polynomial }P(y)\mathrm{ with }P(0)=
node }\mp@subsup{\mathcal{U}}{j}{
    Generate a random 2t
    Send a point }\mp@subsup{P}{j}{}(i)\mathrm{ to node }\mp@subsup{\mathcal{U}}{i}{}\mathrm{ for each }i\in[2t+1
    Wait to receive points {}{\mp@subsup{P}{i}{}(j)\mp@subsup{}}{i\in[2t+1]}{}\mathrm{ from all other nodes

```
![](https://cdn.mathpix.com/cropped/2024_07_17_ad78533e3470f9e4f2abg-16.jpg?height=44&width=526&top_left_y=997&top_left_x=249)

Figure 9: $(2 t, 2 t+1)$-UnivariateZeroShare between $2 t+1$ nodes. A 0-hole univariate polynomial $P$ of degree $-2 t$ is generated.

$(t, n)$-BivariateZeroShare

Input: $t, n$, set of nodes $\left\{C_{i}\right\}_{i \in[n]}(2 t<n)$

Output: Each node $C_{i}$ outputs a share $Q(i, y)$ for randomly generated degree- $\langle t, 2 t\rangle$

bivariate polynomial $Q(x, y)$ with $Q(0,0)=0$

Order $\left\{C_{i}\right\}_{i \in[n]}$ based on lexicographic order of their public keys

Choose first $2 t+1$ nodes, w.l.o.g., $\mathcal{U}=\left\{C_{j}\right\}_{j \in[2 t+1]}$

Invoke $(2 t, 2 t+1)$-UnivariateZeroShare among $\left\{\mathcal{U}_{j}\right\}_{j \in[2 t+1]}$ to generate shares

$\left\{s_{j}\right\}_{j \in[2 t+1]}$

$\underline{\text { node } \mathcal{U}_{j}}$

Generate a random $t$-degree polynomial $R_{j}$ s.t $R_{j}(0)=s_{j}$

Send a point $R_{j}(i)$ to node $C_{i}$ for each $i \in[n]$

Denote the bivariate polynomial $Q(x, y)$ where $\left\{Q(x, j)=R_{j}(x)\right\}_{j \in[2 t+1]}$

$\underline{\text { node } C_{i} \text { : }}$

Wait to receive points $\left\{R_{j}(i)\right\}_{j \in[2 t+1]}=\{Q(i, j)\}_{j \in[2 t+1]}$

Interpolate to reconstruct a $2 t$-degree polynomial $Q(i, y)$

Output share $Q(i, y)$

Figure 10: $(t, n)$-BivariateZeroShare between $n$ nodes. A 0-hole bivariate polynomial $Q$ of degree $-\langle t, 2 t\rangle$ is generated.

is corrupt, VerifyEval rejects, and honest nodes in $C^{\prime}$ output fail. Otherwise, with $2 t+1$ correct shares of $B^{\prime}(i, j), C_{i}^{\prime}$ can successfully reconstruct $B^{\prime}(i, y)$, which completes the proof of integrity.

\section{B APPLICATIONS \\ IN DECENTRALIZED SYSTEMS}

Secret sharing finds use in innumerable applications involving cryptographic secrets, including secure multi-party computation (MPC) [14, 20, 23], threshold cryptography [27], Byzantine agreement [67], survivable storage systems [78], and cryptocurrency custody $[9,57]$, to name just a few.

Decentralized systems, however, are an especially attractive application domain, though, for two reasons.

\section{Opt-ShareReduce}

Public Input: $\left\{C_{B(x, j)}\right\}_{j \in[2 t+1]}$

Input: Set of nodes $\left\{C_{i}\right\}_{i \in[n]}$ where each node $C_{i}$ is given $\left\{B(i, j), W_{B(i, j)}\right\}_{j \in[2 t+1]}$.

Set of nodes $\left\{C_{j}^{\prime}\right\}_{j \in\left[n^{\prime}\right]}$ s.t. $n^{\prime} \geq 2 t+1$

Output: $\forall j \in[2 t+1]$, node $C_{j}^{\prime}$ output $B(x, j)$

Order $\left\{C_{j}^{\prime}\right\}$ based on lexicographic order of their public keys

Choose the first $2 t+1$ nodes, denoted as $\mathcal{U}^{\prime}$, w.l.o.g., $\mathcal{U}^{\prime}=\left\{C_{j}^{\prime}\right\}_{j \in[2 t+1]}$

$\underline{\text { node } C_{i}}$

$\forall j \in[2 t+1]$, send a point and witness $\left\{B(i, j), W_{B(i, j)}\right\}$ to $\mathcal{U}_{j}^{\prime}$ off-chain

node $\mathcal{U}_{j}^{\prime}$ :

Wait and receive $n$ points and witnesses, $\left\{B(i, j), W_{B(i, j)}\right\}_{i \in[n]}$

$\forall i \in[n]$, invoke $\operatorname{VerifyEval}\left(C_{B(x, j)}, i, B(i, j), W_{B(i, j)}\right)$

Interpolate any $t+1$ verified points to construct $B(x, j)$

Figure 11: Opt-ShareReduce between the committees $C$ and $C^{\prime}$.

```
Opt-Proactivize
    Public Input: $\left\{C_{B(x, j)}\right\}_{j \in[2 t+1]}$
    Input: Set of nodes $\left\{C_{i}^{\prime}\right\}_{i \in\left[n^{\prime}\right]}$. Let $\mathcal{U}^{\prime}=\left\{C_{j}^{\prime}\right\}_{j \in[2 t+1]}$, each node $\mathcal{U}_{j}^{\prime}$ is given $B(x, j)$
    Output: $\mathcal{U}_{j}^{\prime}$ outputs success and $B^{\prime}(x, j)$ for a degree- $\langle t, 2 t\rangle$ bivariate polynomia
    $B^{\prime}(x, y)$ with $B^{\prime}(0,0)=B(0,0)$ (or) fail
    Public Output: $\left\{C_{B^{\prime}(x, j)}\right\}_{j \in[2 t+1]}$
    Invoke $(2 t, 2 t+1)$-UnivariateZeroShare among the nodes $\left\{\mathcal{U}_{j}^{\prime}\right\}_{j \in[2 t+1]}$ to generate
    shares $\left\{s_{j}\right\}_{j \in[2 t+1]}$
    node $\mathcal{U}_{j}^{\prime}$
    Generate random $t$-degree polynomial $R_{j}(x)$ such that $R_{j}(0)=s_{j}$
    Denote the bivariate polynomial $Q(x, y)$ where $\left\{Q(x, j)=R_{j}(x)\right\}_{j \in[2 t+1]}$
    Denote the bivariate polynomial $B^{\prime}(x, y)=B(x, y)+Q(x, y)$
    node $\mathcal{U}_{j}^{\prime}$
        Compute $B^{\prime}(x, j)=B(x, j)+Q(x, j)$ and $Z_{j}(x)=R_{j}(x)-s_{j}$
        $\operatorname{Se}\left\{g^{s}{ }^{s}, C_{Z_{j}}, W_{Z_{j}(0)}, C_{B^{\prime}(x, j)}\right\} \quad$ off-chain to all nodes in $C^{\prime}$, where
    $C_{Z_{j}}=\operatorname{Commit}\left(Z_{j}\right) ; W_{Z_{j}(0)}=$ CreateWitness $\left(Z_{j}, 0\right) ; C_{B^{\prime}(x, j)}=\operatorname{Commit}\left(B^{\prime}(x, j)\right)$
        Publish hash of the commitments on-chain $H_{j}=H\left(g^{s_{j}}\left\|C_{Z_{j}}\right\| W_{Z_{j}(0)} \| C_{B^{\prime}(x, j)}\right)$
    $\underline{\text { node } C_{i}^{\prime}}$
15: $\forall j \in[2 t+1]$, retrieve on-chain hash $H_{j}$, also receive $\left\{g^{s_{j}}, C_{Z_{j}}, W_{Z_{j}(0)}, C_{B^{\prime}(x, j)}\right\}$
    off-chain
        $\forall j \in[2 t+1]$, if $H_{j} \quad \neq \quad H\left(g^{s}\left\|C_{Z_{j}}\right\| W_{Z_{j}(0)} \| C_{B^{\prime}(x, j)}\right)$ or
    $\operatorname{Verify} \operatorname{Eval}\left(C_{Z_{j}}, 0,0, W_{Z_{j}(0)}\right) \neq$ True or $C_{B^{\prime}(x, j)} \neq C_{B(x, j)} \times C_{Z_{j}} \times g^{s j}$, output fail
17: Using Lagrange coefficients in Eq. (1), if $\prod_{j=1}^{2 t+1}\left(g^{s_{j}}\right)^{\lambda_{j}^{2 t}} \neq 1$ output fail
    node $\mathcal{U}_{j}^{\prime}$ :
19: Output success and $B^{\prime}(x, j)$
```

Figure 12: Opt-Proactivize updates the reduced shares.

First, blockchain systems task individual users with management of their own private keys, an unworkable approach for most users. A frequent result, as noted above, is key loss [69] or centralized key management $[9,57]$ that defeats the main purpose of blockchain systems.

Second, blockchain objects cannot keep private state. This fact notably limits the useful applications of smart contracts, as they cannot compute digital signatures or manage encrypted data.

We briefly enumerate a few of the most important potential applications in decentralized systems of the (dynamic-committee proactive) secret-sharing enabled by CHURP:

Opt-ShareDist

```
Public Input: $\left\{C_{B^{\prime}(x, j)}\right\}_{j \in[2 t+1]}$
    Input: Set of nodes $\left\{C_{i}^{\prime}\right\}_{i \in\left[n^{\prime}\right]}$. Let $\mathcal{U}^{\prime}=\left\{C_{j}^{\prime}\right\}_{j \in[2 t+1]}$, each node $\mathcal{U}_{j}^{\prime}$ is given $B^{\prime}(x, j)$
    Output: $\forall i \in\left[n^{\prime}\right], C_{i}^{\prime}$ outputs success and $B^{\prime}(i, y)$ (or) fail
    node $\mathcal{U}_{j}^{\prime}$
    $\forall i \in\left[n^{\prime}\right]$, send a point and witness off-chain $\left\{B^{\prime}(i, j), W_{B(i, j)}^{\prime}\right\}$ to $C_{i}^{\prime}$ where
    $W_{B(i, j)}^{\prime}=$ CreateWitness $\left(B^{\prime}(x, j), i\right)$
    node $C_{i}^{\prime}$ :
    Wait and receive points and witnesses $\left\{B^{\prime}(i, j), W_{B(i, j)}^{\prime}\right\}_{j \in[2 t+1]}$
    $\forall j \in[2 t+1]$, invoke $\operatorname{VerifyEval}\left(C_{B^{\prime}(x, j)}, i, B^{\prime}(i, j), W_{B(i, j)}^{\prime}\right)$
    If all $2 t+1$ points are correct, interpolate to construct $B^{\prime}(i, y)$
    Output success and the full share $B^{\prime}(i, y)$
    In all other cases, output fail
```

Figure 13: Opt-ShareDist uses the updated reduced shares to distribute full shares in $C^{\prime}$.

Usable cryptocurrency management. Rather than relying on centralized parties (e.g., exchanges) to custody private keys for cryptocurrency, or using hardware or software wallets, which are notoriously difficult to manage [8], users could instead store their private keys with committees. These committees could authenticate users and enforce access-control, resulting in the decentralized equivalent of today's exchanges.

Decentralized identity. Initiatives such as the Decentralized Identity Foundation [6], which is backed by a number of major IT and services firms, as well as smaller efforts, such as uPort [7], envision an ecosystem in which users control their identities and data by means of private keys. Who will store these keys and how is left an open question [49]. The same techniques used in the cryptocurrency case for private-key management would similarly apply to assets such as identities. Additionally, a committee could manage encrypted identity documents on users' behalf.

Auditable access control. As proposed in [53], a committee could manage encrypted documents and decrypt them for recipients under a given access-control policy while logging their accesses on-chain. The result would be a strongly auditable access-control system. This application could be managed by a smart contract.

Smart-contract attestations. Committee management of smartcontract private keys could also enable digital signing by smart contracts. The idea would be that committee members execute threshold signatures using a shared private key, emitting a signature for a particular smart contract in response to a request issued by the contract on chain.

Such signing would be of particular benefit in creating a simple smart-contract interface with off-chain systems. For example, control of Internet-of-Things (IoT) devices is commonly proposed application of smart contracts [22] (smart locks being a notable early example [65]). If smart contracts cannot generate digital signatures, then the devices they control must monitor a blockchain, an ongoing resource-intensive operation. A smart contract that can generate a digital signature, however, can simply issue authenticable commands to target devices.
Simplified Committee-based consensus for light clients. A number of consensus schemes, e.g., proof-of-stake protocols [17, 25, 51, 52], aim to achieve good scalability by delegating consensus to committees. These committees change over time. Therefore verifying the blocks they sign requires awareness of their identities. By instead maintaining or only periodically rotating its public / private key pair, a committee could instead make it easier for light clients to verify signed blockchains.

Secure multiparty computation (MPC) for smart contracts. More generally, dynamic-committee secret sharing would enable decentralized secure multiparty computation (MPC) by smart contracts, effectively endowing them with confidential storage and computation functionalities, as envisioned in, e.g., [21, 81].

\section{CHURP PESSIMISTIC PATHS}

In this section, we present protocols for the two pessimistic paths of CHURP: Exp-CHURP-A and Exp-CHURP-B.

\section{1 Exp-CHURP-A}

This path is invoked when a failure occurs in Opt-CHURP. It also consists of three phases: Exp-ShareReduce, Exp-Proactivize, Exp-ShareDist.

Before the first phase starts, commitments to reduced shares $\{B(x, j)\}_{j=1}^{2 t+1}$ are published on-chain by $t+1$ nodes in the old committee. The on-chain hashes can be used to verify the posted commitments. As at least one of the $t+1$ nodes is honest, and thus each member of the new committee has the commitments.

C.1.1 Share Reduction (Exp-ShareReduce). This phase is the same as Opt-ShareReduce, and is not re-executed if Opt-ShareReduce ends successfully. This is because the degree of $B(x, j)$ is $t$ and an honest member $\mathcal{U}_{j}^{\prime}$ can successfully reconstruct the polynomial given $t+1$ honest values from $C_{\text {alive }}$ assuming $\left|C_{\text {alive }}\right| \geq 2 t+1$.

C.1.2 Proactivization (Exp-Proactivize). The goal of this phase is to perform a bivariate 0 -sharing, and identify and expel adversaries if malicious behavior is detected.

We first use a different zero-sharing protocol. Each node $\mathcal{U}_{i}^{\prime}$ generates $2 t+1$ sub-shares $\left\{s_{i j}\right\}_{j \in[2 t+1]}$ that form a 0 -sharing i.e., $\sum_{j=1}^{2 t+1} \lambda_{j}^{2 t} s_{i j}=0$ where the Lagrange coefficients $\lambda_{j}^{2 t}$ are introduced before. $\mathcal{U}_{i}^{\prime}$ then publishes $\left\{g^{s_{i j}}\right\}_{j \in[2 t+1]}$ and $\left\{\operatorname{Enc}_{\mathrm{pk}_{j}}\left[s_{i j}\right]\right\}_{j \in[2 t+1]}$ on-chain. $\mathcal{U}_{i}^{\prime}$ also publishes a zk proof of correctness of the encrypted ciphertext. A receiving node $\mathcal{U}_{j}^{\prime}$ verifies the set $\left\{g^{s_{i j}}\right\}_{j}$ using Eq. (1). Then, it decrypts the ciphertext to receive $s_{i j}$.

The advantage of this univariate zero-sharing protocol is that honest parties do not need to re-execute the protocol when an adversary is detected. They can simply discard the shares generated by the adversarial nodes. This is depicted pictorially in Fig. 14. One can see that by setting $s_{j}=\sum_{i \in \mathcal{U}^{\prime} \backslash \mathcal{U}_{\text {corrupt }}^{\prime}} s_{i j}$ for $j \in \mathcal{U}^{\prime}$, the shares form a valid univariate zero-sharing among the honest parties.

After the univariate zero-sharing, the same protocol as that in Opt-Proactivize (step 6-16 in Figure 12) is executed with commitments and witnesses in step 12 posted on-chain. Finally, another major difference to the optimistic path is that if any adversary in the $\mathcal{U}^{\prime}$ is expelled in this phase, we do not have enough nodes to recover the full shares in the next phase, as the degree of $B^{\prime}(i, y)$ is $2 t$ and

\begin{tabular}{|c|c|c|c|c|}
\hline$s_{11}$ & $s_{12}$ & $s_{13}$ & $s_{14}$ & $s_{15}$ \\
\hline$s_{21}$ & $s_{22}$ & $s_{23}$ & $s_{24}$ & $s_{25}$ \\
\hline$s_{31}$ & $s_{32}$ & $s_{33}$ & $s_{34}$ & $s_{35}$ \\
\hline$s_{41}$ & $s_{42}$ & $s_{43}$ & $s_{44}$ & $s_{45}$ \\
\hline$s_{51}$ & $S_{52}$ & $s_{53}$ & $S_{54}$ & $s_{55}$ \\
\hline
\end{tabular}

Figure 14: Matrix of sub-shares. The sub-share $s_{i j}$ is generated by node $i$ and sent to $j$. Each node generates a row while it's share is the sum of sub-shares in a column. If nodes 4 and 5 are adversarial, sub-shares generated by them are discarded.

a member $C_{i}^{\prime}$ needs $2 t+1$ points to reconstruct the polynomial. To address this problem, we further ask members in the old committee to publish the shares and witnesses sent to the adversarial nodes during Opt-ShareReduce on the chain. In this way, all honest parties have access to those reduced shares that belong to adversarial nodes, which allows them to reconstruct the full shares in the next phase. The security of the new protocol still holds, as these shares were accessed by the adversary anyway.

The full protocol of Exp-Proactivize is presented in Figure 15. The on-chain cost of this phase is $O\left(t^{2}\right)$.

C.1.3 Full Share Distribution (Exp-ShareDist). Finally, full shares are distributed to all members of the new committee in this phase To allow identification and expulsion of malicious nodes, members post all messages on the chain in contrast to the optimistic path.

If adversarial nodes are detected in this phase, similar to the proactivization phase, we ask members of the old committee to publish the reduced shares sent to them in Opt-ShareReduce. In addition, honest members need to exclude the proactivization polynomials generated by the adversarial nodes in the second phase. In particular, they discard the sub-shares related to the adversaries in the new univariate zero-sharing protocol, as explained in the previous section, and post their sub-shares for the the adversaries publicly on-chain Fortunately, this only incurs a small extra on-chain cost.

The full protocol of Exp-ShareDist is presented in Figure 16. The on-chain cost of this phase is also $O(t n)$. Therefore, the overall onchain complexity of the Exp-CHURP-A is $O\left(n^{2}\right)$ on-chain (no offchain).

\section{2 State Verification Details}

Failure. There are two possible reasons that may cause StateVerif to fail: either the commitments are computed incorrectly by adversarial nodes, or the assumptions in the KZG scheme fails. We further perform the following test to determine the reason.

We make use of the on-chain KZG commitments (published in CHURP) to verify the commitments $Z_{i}=g^{s_{i}}$ and $Z_{i}^{r n d}=g^{s_{i}^{\prime}}$. Each node $i$ posts exponents of their state $\left\{g^{B^{\prime}(i, j)}\right\} \forall j \in[2 t+$ 1], and their witness $w_{j, i}^{\prime}$ to the KZG polynomial commitments $C_{B^{\prime}(x, j)}$ on-chain (each node already has these witnesses at the end of Opt-CHURP or Exp-CHURP-A). Then all members verify the message published by node $i$ : VerifyEvalExp( $\left(C_{B^{\prime}(x, j)}, i, g^{B^{\prime}(i, j)}, W_{j, i}\right)$
Exp-Proactivize

Public Input: $\left\{C_{B(x, j)}\right\}_{j \in[2 t+1]}$

Input: Set of $2 t+1$ nodes $\left\{\mathcal{U}_{j}^{\prime}\right\}_{j \in[2 t+1]}$. Each node $\mathcal{U}_{j}^{\prime}$ is given $B(x, j)$

Output: $\mathcal{U}_{j}^{\prime}$ outputs $B^{\prime}(x, j)$ for a degree- $\langle t, 2 t\rangle$ bivariate polynomial $B^{\prime}(x, y)$ with $B^{\prime}(0,0)=B(0,0)$

Public Output: $\left\{C_{B^{\prime}}(x, j)\right\}_{j \in[2 t+1]}$

node $\mathcal{U}_{i}^{\prime}$

Generate $\left\{s_{i j}\right\}_{j \in[2 t+1]}$ that form a 0 -sharing i.e., $\sum_{j=1}^{2 t+1} \lambda_{j}^{2 t} s_{i j}=0$.

Publish $\left\{g^{s_{i j}}\right\}_{j \in[2 t+1]},\left\{\mathrm{Enc}_{\mathrm{pk}_{j}}\left[s_{i j}\right]\right\}_{j \in[2 t+1]}$ and zk proofs of correctness of the encryptions on-chain

node $\mathcal{U}_{j}^{\prime}$

Decrypt $\left\{\operatorname{Enc}_{\mathrm{pk}_{j}}\left[s_{i j}\right]\right\}$ from node $i$ and verify $s_{i j}$ using $g^{s_{i j}}$ on-chain

$\underline{\text { node } \mathcal{U}_{j}^{\prime}:}$

If any adversarial node $i$ is detected in step 9 , add it to $\mathcal{U}_{\text {corrupted }}^{\prime}$, and publish $s_{j i}$

Set $s_{j}=\sum_{i \in \mathcal{U}^{\prime}} \backslash \mathcal{U}_{\text {corrupted }}^{\prime} s_{i j}$.

Execute step 7-9, 11-12 of Opt-Proactivize in Figure 12, with messages posted on the chain in step 12 .

$\underline{\text { node } C_{i}^{\prime}}$

Execute step 16 of Opt-Proactivize in Figure 12. If it outputs fail, add $j$ to $\mathcal{U}_{\text {corrupted }}^{\prime}$

Nodes in $\mathcal{U}^{\prime}$ discard shares by executing step 12 again.

node $C_{i}$

For all malicious nodes $j$ detected in step 9 and 15 , publish point and witness $\left\{B(i, j), w_{i, j}\right\}$ on-chain.

Figure 15: Exp-Proactivize protocol.

for $j \in[2 t+1]$. (We make use of the following additional functionality in KZG scheme that allows us to verify the exponent of the

```
Exp-ShareDist
    Public Input: $\left\{C_{B^{\prime}(x, j)}\right\}_{j \in[2 t+1]}$
    Input: Set of nodes $\left\{C_{i}^{\prime}\right\}_{i \in\left[n^{\prime}\right]}$. Let $\mathcal{U}^{\prime}=\left\{C_{j}^{\prime}\right\}_{j \in[2 t+1]}$, each node $\mathcal{U}_{j}^{\prime}$ is given $B^{\prime}(x, j)$
    Output: $\forall i \in\left[n^{\prime}\right], C_{i}^{\prime}$ outputs $B^{\prime}(i, y)$
    $\underline{\text { node } \mathcal{U}_{j}^{\prime} \text { : }}$
    $\forall i \in\left[n^{\prime}\right]$, publish $\operatorname{Enc}_{p} k_{i}\left(B^{\prime}(i, j)\right), g^{B^{\prime}(i, j)}, w_{i, j}^{\prime}$ on-chain, where $w_{i, j}^{\prime}=$
    CreateWitness $\left(B^{\prime}(x, j), i\right)$. Also publish zk proofs of correctness of the encryption.
    node $C_{i}^{\prime}$
        Decrypt the message on-chain to get $\left\{B^{\prime}(i, j), w_{i, j}^{\prime}\right\}_{j \in[2 t+1]}$
        $\forall j \in \mathcal{U}^{\prime} \backslash \mathcal{U}_{\text {corrupted }}^{\prime}$, invoke $\operatorname{Verify} \operatorname{Eval}\left(C_{B^{\prime}(x, j)}, i, B^{\prime}(i, j), w_{i, j}^{\prime}\right)$. If any of the
    checks fail, add $j$ to $\mathcal{U}_{\text {corrupted }}^{\prime}$
    node $C_{i}$ :
        Publish $B(i, j), w_{i, j}$ for any new adversarial node $j$ detected above.
    $\underline{\text { node } \mathcal{U}_{i}^{\prime}}$
        Publish $s_{i j}$ for any new adversarial node $j$ detected above and discard shares by executing
    step 12 in Fig. 15
    node $C_{i}^{\prime}$
        $\forall j \in \mathcal{U}_{\text {corrupted }}^{\prime}$, validate their reduced shares posted by the old committee by
    $\forall i \in[n]$, VerifyEval( $\left.C_{B(x, j)}, i, B(i, j), w_{i, j}\right)$
        $\forall j \in \mathcal{U}_{\text {corrupted }}^{\prime}$ Interpolate any $t+1$ verified points to construct $B(x, j)$. Set
    $B^{\prime}(i, j)=B(i, j)+\sum_{i \in \text { honest }} s_{i j}$
        Interpolate all $B^{\prime}(i, j)$ for $j \in[2 t+1]$ to construct $B^{\prime}(i, y)$
        Output the full share $B^{\prime}(i, y)$
```

Figure 16: Exp-ShareDist protocol.
evaluation without any changes to the scheme: $\{$ True, False $\} \leftarrow$ VerifyEvalExp( $\left.\left.C_{\phi}, i, g^{\phi(i)}, W_{i}\right).\right)$

If the checks above pass, all members validate $Z_{i}, Z_{i}^{r n d}$ as: $Z_{i}=$ $\prod_{j=1}^{2 t+1}\left(g^{B^{\prime}(i, j)}\right)^{\lambda_{j}^{2 t}}, Z_{i}^{r n d}=\prod_{j=1}^{2 t+1}\left(g^{B^{\prime}(i, j)}\right)^{r_{j} \lambda_{j}^{2 t}}$.

If any of the checks above fail, it means the commitments are not correctly computed. The members can perform a verifiable accusations since all information is on-chain, and then switch to pessimistic path Exp-CHURP-A. Otherwise, it implies a failure of the assumptions in the KZG scheme. In this case, we switch to a different pessimistic path Exp-CHURP-B. In this test, each node publishes $O(n)$ data on-chain, incurring $O\left(n^{2}\right)$ on-chain cost overall.

\section{3 Exp-CHURP-B}

This pessimistic path is taken only after a detection of breach in the underlying assumptions of the KZG scheme.

In this path, we use relatively expensive polynomial commitments (Pedersen commitments) instead of KZG and supports a lower thresh old on the number of adversarial nodes $n>3 t$. In the share reduction phase, as $n>3 t$, we rely on the error correcting mechanisms of ReedSolomon codes to construct reduced shares, instead of the verification of KZG scheme. In the proactivization phase and full share distribution phase, we replace the KZG commitments and verification with the Pedersen commitments (step 13 in Figure 15 and step 5,8,12 in Figure 16). Exp-CHURP-B incurs $O\left(n^{2}\right)$ on-chain cost, assuming $n>3 t$. Due to the space limit, we omit the full protocol of Exp-CHURP-B.

\section{THE STATIC SETTING: IMPROVED PSS}

We also consider a different and narrower setting, one with a static committee i.e., the old and new committees are identical. The adversarial model is also weaker i.e., corruptions during the handoff phase are counted towards the threshold in both the adjacent epochs. The handoff in such a setting is simply an update since the committee is static. Hence, the update protocol consists of a recovery phase, enabling recovery of lost shares and a refresh phase, updating shares of all nodes.

In this section, we look at different techniques seen in literature for the static setting. Herzberg et al. [48] introduce this setting and present a protocol, Herzberg's PSS. A second technique seen in the literature makes use of bivariate polynomials. We then present an improved PSS protocol which achieves better overall performance than any known scheme.

Herzberg's PSS: This protocol incurs $O\left(n^{2}\right)$ off-chain communication complexity for refresh and an expensive $O\left(n^{2}\right)$ per node recovery (See [48])

Bivariate Polynomials: One way to avoid the expensive recovery cost is to perform secret sharing with a bivariate polynomial. This allows for efficient recovery, i.e., $O(n)$ off-chain communication complexity As discussed previously in Section 4, existing techniques for refresh are expensive costing $O\left(n^{3}\right)$.

Improved PSS: Much like the dynamic setting, we build an improved PSS protocol using the efficient bivariate 0 -sharing technique. This technique brings down the total communication complexity to just $O\left(n^{2}\right)$ off-chain. A comparison of communication costs incurred by different PSS schemes is in Table 5.

\begin{tabular}{c|c|ccc}
\hline & & Univariate [48] & Bivariate & Improved PSS \\
\hline \multirow{2}{*}{ Off-chain } & Recovery & $O\left(n^{2}\right)$ & $O(n)$ & $O(n)$ \\
& Refresh & $O\left(n^{2}\right)$ & $O\left(n^{3}\right)$ & $O\left(n^{2}\right)$ \\
\hline State & & $O(1)$ & $O(n)$ & $O(n)$ \\
\hline
\end{tabular}

Table 5: Comparison of protocols in the static setting with a honestbut-curious adversary. The original protocol of Herzberg et al. is presented in the univariate column. Recovery costs are per node. Note that recovery costs of our protocol are amortized over the total number of nodes being replaced.

```
Improved-PSS
    Input: Set of $n$ nodes $C$. Each node $C_{i}^{\prime}$ is given a degree- $t$ polynomial $B(i, y)$
    Output: $C_{i}^{\prime}$ outputs $B^{\prime}(i, y)$ for a degree- $\langle t, t\rangle$ bivariate polynomial $B^{\prime}(x, y)$ with
    $B^{\prime}(0,0)=B(0,0)$
    Order nodes in $C$ based on the lexicographic ordering determined by public keys
    Choose first $t+1$ nodes, $\mathcal{U} \subset C,|\mathcal{U}|=t+1$
    node $C_{i}$
        send $B(i, j)$ to node $\mathcal{U}_{j}, \forall j \in[t+1]$
    $\underline{\text { node } \mathcal{U}_{j}}$
    Reconstruct degree- $t$ polynomial $B(x, j)$
    Invoke $(t, t+1)$-UnivariateZeroShare among $\mathcal{U}$ generating shares $\left\{s_{j}\right\}_{j}, \forall j \in[t+1]$
    $\underline{\text { node } \mathcal{U}_{j}}$
        Generate a degree- $t$ polynomial $Q(x, j)$ s.t. $Q(0, j)=s j$
        Update the reconstructed polynomial $B^{\prime}(x, j)=B(x, j)+Q(x, j)$
        send $B^{\prime}(i, j)$ to each node $i \in C$
    $\underline{\text { node } C_{i}}$
        Construct degree- $t$ polynomial $B^{\prime}(i, y)$ using $t+1$ received points
```

Figure 17: Improved PSS for static setting, honest-but-curious adversary.

Let $C$ denote the committee, $C=C^{(e-1)}=C^{(e)}$, comprising $n$ nodes $\left\{C_{i}\right\}_{i=1}^{n}$. The secret is shared using an asymmetric bivariate polynomial $B(x, y), s=B(0,0)$. Unlike before, the degree of bivariate polynomial is only $\langle t, t\rangle$ as we have a weaker adversary.

Recall that node's share is a single polynomial $B(i, y)$. In Fig. 17, we present the improved PSS assuming a honest-but-curious adversary. Throughout the protocol, each node sends out atmost $O(n)$ points. Thus, our improved PSS scheme completes in $O\left(n^{2}\right)$ off-chain cost.

Active adversaries: In face of adversarial behaviour, multiple reruns of the protocol might be needed. This is crucial since all the $t+1$ received points need to be correct in order to compute the new share. Adversaries are detectable with the use of $K Z G$ commitments similar to the dynamic setting. We replace the detected adversarial nodes with uncorrupted nodes from $C$ (guaranteed to find such a node, $|C| \geq 2 t+1)$. We stress that this protocol incurs $O\left(n^{2}\right)$ off-chain cost even after adapting to handle active adversaries. This is achieved due to the following key property: Honest nodes never rerun any phase of the protocol. This is possible by making a slight modification to the univariate 0 -sharing (step 9): invoke $(t, n)$-UnivariateZeroShare among all nodes in $C$ instead of executing it in a subset of nodes only. Observe that the set of univariate polynomials held by any $t+1$-sized subset in $C$ defines a 0 -hole bivariate polynomial. Thus, reruns are executed only by the new uncorrupt nodes that replace the detected faulty nodes.

\section{KZG extended with degree verification}
1) $($ sk, pk $) \leftarrow \operatorname{Keygen}\left(1^{\lambda}, q\right):$ Select a bilinear group $\left(p, \mathbb{G}, \mathbb{G}_{T}, e, g\right) \leftarrow \operatorname{BilGen}\left(1^{\lambda}\right)$, $q+1$ group elements $\left\{\alpha_{i}\right\}_{i \in[q]}$ and $s$ randomly in $\mathbb{Z}_{p}^{*}$. Set $\mathrm{sk}=s, \mathrm{pk}_{0}=\left\{g^{s}, \ldots, g^{s}{ }^{d}\right\}$, $\mathrm{pk}_{d}=\left\{g^{\alpha} d^{s}, \ldots g^{\alpha} d^{s}\right\}$ for $d \in[q]$ and $\mathrm{pk}=\left\{\mathrm{pk}_{0}, \mathrm{pk}_{1}, \ldots, \mathrm{pk}_{q}\right\}$.
2) $C_{\phi} \leftarrow \operatorname{Commit}(\phi(x), \mathrm{pk})$ : Let $d=\operatorname{deg}(\phi)$. Compute $C_{\phi}=\left(d, g^{\phi(s)}, g^{\alpha_{d}}{ }^{\phi(s)}\right)$ using $\mathrm{pk}_{0}$ and $\mathrm{pk}_{d}$
3) $\left(\phi(i), W_{i}\right) \leftarrow$ CreateWitness $(\phi(x), i, \mathrm{pk})$ : Compute $\phi(x)-\phi(i)=(x-i) w(x)$, set $W_{i}=g^{w(s)}$
4) $\{$ True, False $\} \leftarrow \operatorname{Verify} \operatorname{Eval}\left(C_{\phi}, i, \phi(i), W_{i}, \mathrm{pk}\right)$ : Parse $C_{\phi}$ as $\left(d, \mathrm{C}, \mathrm{C}_{d}\right)$. Output True if $e\left(\mathrm{C} / g^{\phi(i)}, g\right)=e\left(g^{s-i}, W_{i}\right)$. Otherwise, output False.
5) $\{$ True, False $\} \leftarrow$ VerifyDegree $\left(C_{\phi}\right)$ : Parse $C_{\phi}$ as $\left(d, \mathrm{C}, \mathrm{C}_{d}\right)$. Output True if $e\left(\mathrm{C}_{d}, g\right)=e\left(\mathrm{C}, g^{\alpha} d\right)$. Otherwise, output False.

Figure 18: KZG [50] extended with degree verification.

\section{E CHANGING THE THRESHOLD}

\section{E. 1 Increasing the threshold: $t_{e}>t_{e-1}$}

Note that a change of the threshold reflects that of the adversary's power, i.e., the number of nodes it can corrupt in the committee $C^{(e-1)}$ and $C^{(e)}$, respectively. Therefore extra care is needed if we were to increase the power of the adversary (i.e. $t_{e}>t_{e-1}$ ). Similar to [72], increasing the threshold takes two steps: first, a handoff is executed between $C^{(e-1)}$ and $C^{(e)}$ assuming the threshold doesn't change; then we increase the threshold to $t_{e}$ after the handoff. As illustrated below, the new threshold takes effect after the handoff.

![](https://cdn.mathpix.com/cropped/2024_07_17_ad78533e3470f9e4f2abg-20.jpg?height=219&width=810&top_left_y=1235&top_left_x=213)

Specifically, to increase the threshold, $\left(t_{e-1}, t_{e}\right)$-handoff runs the proactivization phase with parameters $t=t_{e}$. That is, during the proactivization protocol, a bivariate polynomial $Q(x, y)$ of degree $\left(t_{e}, 2 t_{e}\right)$ is generated such that $Q(0,0)=0$. Each node $i$ holds a $t_{e^{-}}$ degree polynomial $Q(x, i)$ and commitments to $\{Q(x, i)\}_{i}$ are publicly available. The rest of the proactivization follows without modification, besides now each node $i$ holds two polynomials with different degrees: $B^{\prime}(x, i)$ that is $t_{e-1}$-degree while $Q(x, i)$ is $t_{e}$-degree. Thus the proactivized global polynomial $B^{\prime}(x, y)$ is of degree $\left(t_{e}, 2 t_{e}\right)$, concluding the threshold upgrade.

We also need to extend KZG to support dynamic thresholds, i.e., given a commitment $C_{\phi}$, it can be publicly verified that $\phi$ is at most $d$-degree. Essentially, the setup phase of the KZG fixes the highest degree (say, $D$ ) of polynomials it can work with. In the setting of a static threshold $t$, we set $D=t$ and a KZG commitment can guarantee that hidden polynomials are of degree $\leq t$, which is critical to the correctness of shares. To support dynamic thresholds up to $t_{\text {max }}$, we extend KZG as specified in Fig. 18 and run the trusted setup with $D=t_{\text {max }}$. Our extension relies on the $q$-PKE [45] assumption.

\section{E. 2 Decreasing the threshold}

The intuition of decreasing the threshold is to create $2 \times\left(t_{e-1}-t_{e}\right)$ virtual nodes, denoted as $\mathcal{V}$, and execute the handoff protocol between $C=C^{(e-1)}$ and $C^{\prime}=C^{(e)} \cup \mathcal{V}$, assuming the threshold remains $t_{e-1}$. A virtual node participates in the protocol as if an honest player, but exposes its state publicly. At the end of the handoff protocol, nodes in $C^{\prime}$ incorporate $\mathcal{V}$ 's state and restore the invariants. The handoff protocol is outlined as follows.

```
Decreasing the threshold
```

1) Choose a subset $\mathcal{U} \subseteq C^{\prime}$ of $2 t_{e}+1$ nodes. For notational simplicity, suppose $\mathcal{U}=\left\{1, \ldots, 2 t_{e}+1\right\}$ and $\mathcal{V}=\left\{2 t_{e}+2, \ldots, 2 t_{e-1}+1\right\}$. Each node $i \in \mathcal{U}$ recovers a reduced share $R S_{i}^{(e-1)}(x)=B(x, i)$. In addition, $C$ publishes reduced shares for virtual nodes: $R S_{j}^{(e-1)}(x)=B(x, j)$ for $j \in \mathcal{V}$.

2) $\mathcal{U}$ executes the proactivization phase and collectively generate a $\left(t_{e}, 2 t_{e}\right)$-degree bivariate zero-hole polynomial $Q(x, y)$. At the end of this phase, each node $i \in \mathcal{U}$ has $Q(x, i)$.

3) Let $V=\sum_{j \in \mathcal{V}} \lambda_{j}^{2 t} \boldsymbol{e - 1} R S_{j}^{(e-1)}(0)$. Each node $i \in \mathcal{U}$ incorprates virtual nodes' state and updates its state as $R S_{i}^{(e)}(x)=\frac{\lambda_{i}^{2 t_{e-1}}}{\lambda_{i}^{2 t_{e}}}\left(R S_{i}^{(e-1)}(x)+\frac{V}{\lambda_{i}^{2 t} e-1\left(2 t_{e}+1\right)}\right)+Q(x, i)$ where $\lambda^{2 t_{e-1}}$ and $\lambda^{2 t_{e}}$ are Lagrange coefficients for corresponding thresholds. One can verify that $R S_{i}^{(e)}(x)$ are $2 t_{e}$-sharing of the secret, i.e., $B(0,0)$ can be calculated from any $2 t_{e}+1$ of $R S_{i}^{(e)}(x)$

4) Each node $i \in \mathcal{U}$ sends to every node $j \in C^{\prime}$ a point $R S_{i}^{(e)}(j)$. The full share of each node $j \in C^{\prime}$ consists of $2 t_{e}+1$ points $\left\{R S_{i}^{(e)}(j)=B^{\prime}(i, j)\right\}_{i \in \mathcal{U}}$, from which $j$ can compute $F S_{j}(y)=B^{\prime}(j, y)$.

The updated reduced shares $R S_{i}^{(e)}(x)$ can be verified using the published value $V$, and the commitment to $R S_{i}^{(e-1)}(x)$ and $Q(x, i)$. At the end, each node $i$ has $2 t_{e}+1$ points on $B^{\prime}(i, y)$. It remains to show that $\left\{F S_{j}(y)=B^{\prime}(j, y)\right\}_{j}$ form a $t_{e}$-sharing of $B^{(e)}(0,0)$, which can be checked by $\sum_{i=1}^{t_{e}+1} \lambda_{i}^{t_{e}} F S_{i}(0)=\sum_{j=1}^{2 t_{e-1}+1} \lambda_{j}^{2 t_{e-1}} R S_{j}^{(e-1)}(0)=B(0,0)$.

Several optimizations are possible. For example, one can reduce the degree of $R S_{i}^{(e)}(x)$ to $t_{e}$ (as opposed to $t_{e-1}$ currently) by building new polynomials and proving equivalence to $R S_{i}^{(e-1)}(x)$. We leave further optimization for future work.