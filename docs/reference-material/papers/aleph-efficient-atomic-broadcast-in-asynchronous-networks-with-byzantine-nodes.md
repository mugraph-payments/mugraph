\title{
Aleph: Efficient Atomic Broadcast in Asynchronous Networks with Byzantine Nodes
}

\author{
Adam Gągol ${ }^{1,2}$, Damian Leśniak ${ }^{1,2}$, Damian Straszak ${ }^{1}$, Michał Świętek ${ }^{1,2}$ \\ ${ }^{1}$ Aleph Zero Foundation \\ ${ }^{2}$ Jagiellonian University
}

\begin{abstract}
The spectacular success of Bitcoin and Blockchain Technology in recent years has provided enough evidence that a widespread adoption of a common cryptocurrency system is not merely a distant vision, but a scenario that might come true in the near future. However, the presence of Bitcoin's obvious shortcomings such as excessive electricity consumption, unsatisfying transaction throughput, and large validation time (latency) makes it clear that a new, more efficient system is needed.

We propose a protocol in which a set of nodes maintains and updates a linear ordering of transactions that are being submitted by users. Virtually every cryptocurrency system has such a protocol at its core, and it is the efficiency of this protocol that determines the overall throughput and latency of the system. We develop our protocol on the grounds of the well-established field of Asynchronous Byzantine Fault Tolerant (ABFT) systems. This allows us to formally reason about correctness, efficiency, and security in the strictest possible model, and thus convincingly prove the overall robustness of our solution.

Our protocol improves upon the state-of-the-art HoneyBadgerBFT by Miller et al. by reducing the asymptotic latency while matching the optimal communication complexity. Furthermore, in contrast to the above, our protocol does not require a trusted dealer thanks to a novel implementation of a trustless ABFT Randomness Beacon.
\end{abstract}

\section{KEYWORDS}

Byzantine Fault Tolerance, Asynchrony, DAG, Atomic Broadcast, Randomness Beacon, Consensus, Cryptography

\section{INTRODUCTION}

The introduction of Bitcoin and the Blockchain in the seminal paper of Satoshi Nakamoto [39] is already considered a pivotal point in the history of Finanancial Technologies. While the rise of Bitcoin's popularity clearly shows that there is significant interest in a globally distributed currency system, the scalability issues have become a significant hurdle to achieve it. Indeed, Bitcoin's latency of 30 to 60 minutes, the throughput of 7 transactions per second, and the excessive power usage of the proof of work consensus protocol have motivated the search for alternatives.

At the core of virtually every cryptocurrency system lies a mechanism that collects transactions from users and constructs a total ordering of them, i.e., either explicitly or implicitly forming a blockchain of transactions. This total ordering is then used to determine which transaction came first in case of double-spending attempts and thus to decide which transactions should be validated. The protocol that guides the maintenance and growth of this total ordering is the heart of the whole system. In Bitcoin, the protocol is Proof of Work, but there are also systems based on Proof of Stake [12, 30] and modifications of these two basic paradigms [32, 44]. Aside from efficiency, the primary concern when designing such protocols is their security. While Bitcoin's security certainly has passed the test of time, numerous newly proposed designs claim security but fall short of providing convincing arguments. In many such cases, serious vulnerabilities have been discovered, see $[2,18]$.

Given these examples, one may agree that for a new system to be trusted, strong mathematical foundations should guarantee its security. What becomes important then are the assumptions under which the security claim is pursued - in order to best imitate the highly adversarial execution environment of a typical permissionless blockchain system, one should work in the strictest possible model. Such a model - the Asynchronous Network model with Byzantine Faults - has spawned a large volume of research within the field of Distributed Systems for the past four decades. Protocols that are designed to work in this model are called Asynchronous Byzantine Fault Tolerant (ABFT) - and are resistant to harsh network conditions: arbitrarily long delays on messages, node crashes, or even multiple nodes colluding in order to break the system. Interestingly, even though these protocols seem to perfectly meet the robustness requirements for these kinds of applications, they have still not gained much recognition in the crypto-community. This is perhaps because the ABFT model is often considered purely theoretical, and in fact, the literature might be hard to penetrate by an inexperienced reader due to heavy mathematical formalism. Indeed, several of the most important results in this area [9, 10, 19, 24] have been developed in the ' 80 s and ' 90 s and were likely not meant for deployment at that time but rather to obtain best asymptotic guarantees. Now, 30 years in the future, perhaps surprisingly, the ABFT model has become more practically relevant than ever, since the presence of bad actors in modern distributed ledger systems is inevitable, and their power ranges from blocking or taking over several nodes to even entirely shutting down large parts of the network.

In recent important work [35], Miller et al. presented the HoneyBadgerBFT (HBBFT) protocol, taking the first step towards practical ABFT systems. HBBFT achieves optimal, constant communication overhead, and its validation time scales logarithmically with the number of nodes. Moreover, importantly, HBBFT is rather simple to understand, and its efficiency has been also confirmed by running large-scale experiments. Still, an unpleasant drawback of HBBFT, especially in the context of trustless applications, is that it requires a trusted dealer to initialize.

In this paper, we present a completely new ABFT protocol that keeps all of the good properties of HBBFT and improves upon it
in two important aspects: tightening the complexity bounds on latency from logarithmic to constant and eliminating the need for a trusted dealer. Furthermore even though being developed for the asynchronous setting, it matches the optimistic-case performance of 3-round validation time of state-of-the-art synchronous protocols [21]. We believe that our protocol is simple to understand, due to its transparent structure that clearly separates the network layer from the protocol logic. We also present a contribution that might be of independent interest: an efficient, completely trustless ABFT Randomness Beacon that generates common, unpredictable randomness. Since such a mechanism is necessary in many blockchain-based systems, we believe it might see future applications. Finally, we believe that this paper, while offering a valuable theoretical improvement, also contributes to bridging the gap between the theory and practice of ABFT systems.

\section{OUR RESULTS}

The main goal of this paper ${ }^{1}$ is to design a distributed system that runs in a trustless network environment and whose purpose is to build a collective total ordering of messages submitted by users. Apart from blockchain, such systems have several other applications, for example implementing state machine replication, where messages can be arbitrary operations to be executed by a state machine, and the purpose of the system is to keep the states of several copies of the machine consistent by executing the commands in the same order.

A major decision to make when designing systems of this kind is how to realistically model a network environment where such a system would run. In the subsequent paragraphs, we introduce the model we work in and argue why we find it the most suitable for applications in distributed financial systems.

Nodes and Messages. The system consists of $N$ parties $\mathcal{P}=$ $\left\{\mathcal{P}_{1}, \mathcal{P}_{2}, \ldots, \mathcal{P}_{N}\right\}$ that are called nodes. Each node $\mathcal{P}_{i}$ identifies itself through its public key $p k_{i}$ for which it holds a private key $s k_{i}$ that allows it to sign messages if necessary. Messages in the system are point-to-point, i.e., a node $\mathcal{P}_{i}$ can send a message $m$ to another node $\mathcal{P}_{j}$; the node $\mathcal{P}_{j}$ is then convinced that the message came from $\mathcal{P}_{i}$ because of the signature. We assume signatures are unforgeable so a node also cannot deny sending a particular message, since it was signed with its private key.

Network Model. The crucial part of the model are the assumptions about the message delivery and delays. These assumptions are typically formulated by defining the abilities of an adversary, i.e., a powerful entity that watches the system and performs actions to slow it down or cause its malfunction. The first assumption that is somewhat necessary is that the adversary cannot destroy messages that were sent, i.e., when a node sends a message, then it eventually reaches the recipient. In practice, this assumption can be enforced by sending the same message multiple times if necessary. Note also that an adversary with the ability to destroy messages would easily block every possible system, by just destroying all messages. Given that, the most powerful ability an adversary could possibly have is
\footnotetext{
${ }^{1}$ In order to make this text accessible also for readers with no background in Secure Distributed Systems, the narration of the paper focuses on providing intuitions and explaining the core ideas, at the cost of occasionally being slightly informal. At the same time, we stay mathematically rigorous when it comes to theorems and proofs.
}

to delay messages for an arbitrary amount of time. This is what we assume and what is known in the literature as the Asynchronous Network Model. That means the adversary can watch messages being sent and schedule their delivery in an arbitrary order.

In contrast, another popular model is the Synchronous Network $M o d e l^{2}$, where a global-bound $\Delta$ exists such that whenever a message is sent, it is delivered after time at most $\Delta$. As one can imagine, this assumption certainly makes protocol design easier; however, the crucial question to address is: which of these models - asynchronous or synchronous - better fits the typical execution environment of a cryptocurrency system, i.e., the Internet.

While the asynchronous model may seem overly conservative, since no real-life adversary has full control over the network delays, there are mechanisms that may grant him partial control, such as timed DoS attacks. Additionally, synchrony assumptions may be violated due to factors such as transient network partitions or a massive CPU load on several nodes preventing them from sending messages timely.

Finally, the key quality of any protocol meant for finance-related applications is its overall robustness, i.e., a very general notion of resilience against changing network conditions and against other unforeseeable factors. The archetypical partially synchronous algorithm PBFT [21] (and its numerous variants [1, 11, 31]) works in two modes: optimistic and pessimistic. The often-claimed simplicity of PBFT indeed manifests itself in the optimistic mode, but the pessimistic mode (that could as well become the default one under unfavorable network conditions) is in fact a completely separate algorithm that is objectively complex and thus prone to implementation errors. Notably, Miller et al. in [35] demonstrate an attack scenario on protocols from the PBFT family that completely blocks their operation. In the same paper [35], it is also reasoned that asynchronous protocols are substantially more robust, as the model somewhat forces a single, homogeneous operation mode of the algorithm. As such, we believe that the asynchronous model, even though it enforces stricter conditions, is the best way to describe the environment in which our system is going to operate.

Node Faults. For the kind of system we are trying to design, one cannot assume that all the nodes always proceed according to the protocol. A node could simply crash, or go offline, and thus stop sending messages. Alternatively, a node or a set of nodes could act maliciously (be controlled by the adversary) and send arbitrary messages in order to confuse the remaining nodes and simply break the protocol. These latter kinds of nodes are typically referred to in the literature as dishonest, malicious, faulty, or Byzantine nodes, and a protocol that solves a given problem in the presence of dishonest nodes is called Byzantine Fault Tolerant (BFT). In cryptocurrency systems the presence of dishonest nodes is more than guaranteed as there will always be parties trying to take advantage of design flaws in order to gain financial benefits. It is known that no asynchronous system can function properly (reach consensus) in the presence of $N / 3$ or more dishonest nodes [10]; thus, we make the standard assumption that the total number of nodes is $N=3 f+1$, and $f$ of them are dishonest.
\footnotetext{
${ }^{2}$ The Synchronous Model comes in several variants depending on whether the global bound $\Delta$ is known to the algorithm or not and whether there is an initial, finite period of asynchrony.
}

\subsection{Our Contribution}

Before presenting our contributions, let us formalize the problem of building a total ordering of transactions, which in the literature is known under the name of Atomic Broadcast.

Definition 2.1 (Atomic Broadcast). Atomic Broadcast is a problem in which a set of nodes commonly constructs a total ordering of a set of transactions, where the transactions arrive at nodes in an on-line fashion, i.e., might not be given all at once. In a protocol that is meant to solve Atomic Broadcast, the following primitives are defined for every node:

(1) $\operatorname{Input}(t x)$ is called whenever a new transaction $t x$ is received by the node,

(2) Output(pos, $t x$ ) is called when the node decides to place the transaction $t x$ at the position pos $\in \mathbb{N}$.

We say that such a protocol implements Atomic Broadcast if it meets all the requirements listed below:

(1) Total Order. Every node outputs a sequence of transactions in an incremental manner, i.e., before outputting a transaction at position pos $\in \mathbb{N}$ it must have before output transactions at all positions $<$ pos, and only one transaction can be output at a given position.

(2) Agreement. Whenever an honest node outputs a transaction tx at position pos, every other honest node eventually outputs the same transaction at this position.

(3) Censorship Resilience. Every transaction tx that is input at some honest node is eventually output by all honest nodes.

The above definition formally describes the setting in which all nodes listen for transactions and commonly construct an ordering of them. While the Total Order and Agreement properties ensure that all the honest nodes always produce the same ordering, the Censorship Resilience property is to guarantee that no transaction is lost due to censorship (especially important in financial applications) but also guarantees that the system makes progress and does not become stuck as long as new transactions are being received.

Let us briefly discuss how the performance of such an Atomic Broadcast protocol is measured. The notion of time is not so straightforward when talking about asynchronous protocols, as the adversary has the power to arbitrarily delay messages between nodes. For this reason, the running time of such a protocol is usually measured in the number of asynchronous rounds it takes to achieve a specific goal [19]. Roughly speaking, the protocol advances to round number $r$ whenever all messages sent in round $r-2$ have been delivered; for a detailed description of asynchronous rounds, we refer the reader to Section G.

The second performance measure that is typically used when evaluating such protocols is communication complexity, i.e., how much data is being sent between the nodes (on average, by a single honest node). To reduce the number of parameters required to state this result, we assume that a transaction, a digital signature, an index of a node, etc., all incur a constant communication overhead when sent; in other words, the communication complexity is measured in machine words, which are assumed to fit all the above objects. Our first contribution is the Aleph protocol for solving Atomic Brodcast whose properties are described in the following
Theorem 2.1 (Atomic Broadcast). The Aleph protocol implements Atomic Broadcast over $N=3 f+1$ nodes in asynchronous network, of which $f$ are dishonest and has the following properties:

(1) Latency. For every transaction $t x$ that is input to at least $k$ honest nodes, the expected number of asynchronous rounds until it is output by every honest node is $O(N / k)$.

(2) Communication Complexity. The total communication complexity of the protocol in $R$ rounds ${ }^{3}$ is $O\left(T+R \cdot N^{2} \log N\right)$ per node, where $T$ is the total number of transactions input to honest nodes during $R$ rounds.

We believe that the most important parameter of such a protocol and also the one that is hardest to optimize in practice ${ }^{4}$ is the transaction latency. This is why we mainly focus on achieving the optimal $O(1)$ latency ${ }^{5}$. In the Honey Badger BFT [35] the latency is $\Omega(\log N)$ in the optimistic case, while it becomes $\Omega(\beta \log N)$ when there are roughly $\beta N^{2} \log N$ unordered transactions in the system at the time when $t x$ is input. In contrast, the latency of our protocol is $O(1)$ independently from the load.

Similarly to [35] we need to make somewhat troublesome assumptions about the rate at which transactions arrive in the system to reason about the communication complexity, see Section 2.2 for comparison. Still, in the regime of [35] where a steady inflow of transactions is being assumed (i.e., roughly $N^{2}$ per round), we match the optimal $O(1)$ communication complexity per transaction of Honey Badger BFT [35]. As a practical note, we also mention that a variant of our protocol (Section A) achieves the optimal 3-round validation latency in the "optimistic case" akin to partially synchronous protocols from the PBFT family [11, 21] (see Section C. 3 for a proof). On top of that, our protocol satisfies the so-called Responsiveness property [40] which intuitively means that it makes progress at speed proportional to the instantaneous network throughput and is not slowed down by predefined timeouts.

Importantly, we believe that aside from achieving optimal latency and communication complexity, Aleph is simple, clean, and easy to understand, which makes it well fit for a practical implementation. This is a consequence of its modular structure, which separates entirely the network communication layer from the protocol logic. More specifically, the only role of the network layer is to maintain a common (among nodes) data structure called a Communication History DAG, and the protocol logic is then stated entirely through combinatorial properties of this structure. We introduce our protocol in Section 3 and formally prove its properties in Sections C, D and E .

Another important property of our protocol is that unlike [16, 35], it does not require a trusted dealer. The role of a trusted dealer is typically to distribute certain cryptographic keys among nodes before the protocol starts. Clearly, in blockchain systems, no trusted entities can be assumed to exist, and thus a trusted setup is tricky if not impossible to achieve in real-world applications.

Our second contribution is an important, stand-alone component of our protocol that allows us to remove the trusted dealer assumption. More specifically, it can be seen as a protocol for generating
\footnotetext{
${ }^{3}$ Here by a round we formally mean a DAG-rounds, as formally defined in Section 3.1 ${ }^{4}$ The bandwidth is potentially unbounded and can be improved in various ways, while the speed of light puts a hard limit on the network delay.

${ }^{5}$ The $O(1)$ latency is achieved for transactions that are input to at least $k=\Omega(N)$ honest nodes. This is the same regime as in [35] where it is assumed that $k \geqslant 2 / 3 N$.
}
unpredictable, common randomness, or in other words, it implements an ABFT Randomness Beacon. Such a source of randomness is indispensable in any ABFT protocol for Atomic Broadcast, since by the FLP-impossibility result [24], it is not possible to reach consensus in this model using a deterministic protocol. Below we give a formalization of what it means for a protocol to implement such a randomness source. The number $\lambda$ that appears below is the so-called security parameter (i.e., the length of a hash, digital signature, etc.).

Definition 2.2 (Randomness Beacon). We say that a pair of protocols (Setup, Toss $(m)$ ) implements a Randomness Beacon if after running Setup once, each execution of $\operatorname{Toss}(m)$ (for any nonce $m \in$ $\left.\{0,1\}^{\star}\right)$ results in $\lambda$ fresh random bits. More formally, we require
- Termination. All honest nodes correctly terminate after running either Setup or $\operatorname{Toss}(m)$,
- Correctness. For a nonce $m \in\{0,1\}^{\star}$, $\operatorname{Toss}(m)$ results in all honest nodes outputting a common bitstring $\sigma_{m} \in$ $\{0,1\}^{\lambda}$,
- Unpredictability. No computationally bounded adversary can predict the outcome of Toss $(m)$ with non-negligible probability.

In the literature such a source of randomness (especially the variant that outputs just a single bit) is often called a Common Coin [15, 16, 35] or a Global Coin [19]. We note that once the Toss $(m)$ protocol terminates, the value of $\sigma_{m}$ is known, and after revealing it, another execution of $\operatorname{Toss}(m)$ will not provide new random bits; thus, the Unpredictability property is meant to be satisfied only before $\operatorname{Toss}(m)$ is initiated. As our second main contribution, in Section 4 we introduce the ABFT-Beacon protocol, and in Section E we prove the following

Theorem 2.2 (ABFT-Beacon). The ABFT-Beacon protocol implements a Randomness Beacon (Setup, Toss $(m)$ ) such that:
- the Setup phase takes $O$ (1) asynchronous rounds to complete and has $O\left(N^{2} \log N\right)$ communication complexity per node,
- each subsequent call to Toss $(m)$ takes 1 asynchronous round and has $O(N)$ communication complexity per node.

We also remark that the ABFT-Beacon is relatively light when it comes to computational complexity, as the setup requires roughly $O\left(N^{3}\right)$ time per node, and each subsequent toss takes typically $O(N)$ time (under a slightly relaxed adversarial setting); see Section 4.

As an addition to our positive results, in Section H we introduce the Fork Bomb - a spam attack scenario that affects most known DAG-based protocols. In this attack, malicious nodes force honest nodes to download exponential amounts of data and thus likely crash their machines. This attack when attempted to prevent at the implementation layer by banning "suspect nodes" is likely to harm honest nodes as well. Thus, we strongly believe that without a mechanism preventing this kind of attacks already at the protocol layer, liveness is not guaranteed. The basic version of Aleph is resistant against this attack through the use of reliable broadcast to disseminate information among nodes. In Section A we also show a mechanism to defend against this attack for a gossip-based variant of Aleph.

\subsection{Related Work}

Atomic Broadcast. For an excellent introduction to the field of Distributed Computing and overview of Atomic Broadcast and Consensus protocols we refer the reader to the book [13]. A more recent work of [18] surveys existing consensus protocols in the context of cryptocurrency systems.

The line of work on synchronous BFT protocols was initiated in [21] with the introduction of PBFT. PBFT and its numerous variants $[1,11,31]$ tolerate byzantine faults, yet their efficiency relies on the good behavior of the network, and drops significantly when entering the (in some cases implicit) "pessimistic mode". As thoroughly reasoned in [35], synchronous algorithms might not be well suited for blockchain-related applications, because of their lack of robustness and vulnerability to certain types of attacks.

In the realm of asynchronous BFT protocols, a large part of the literature focuses on the more classical Consensus problem, i.e., reaching binary agreement by all the honest nodes. As one can imagine, ordering transactions can be reduced to a sequence of binary decisions, and indeed there are known generic reductions that solve Atomic Broadcast by running Consensus multiple times [5, 15, 22, 36]. However, all these reductions either increase the number of rounds by a super-constant factor or introduce a significant communication overhead. Thus, even though Consensus protocols with optimal number of $O(1)$ rounds [19] and optimal communication complexity [16,38] were known early on, only in the recent work of [35] the Honey Badger BFT (HBBFT) protocol with optimal $O(1)$ communication complexity per transaction was proposed.

The comparison of our protocol to HBBFT is not straightforward since the models of transaction arrivals differ. Roughly, HBBFT assumes that at every round, every node has $\Omega\left(N^{2}\right)$ transactions in its buffer. Under this assumption the communication complexity of HBBFT per epoch, per node is roughly $O\left(N^{2}\right)$, and also $\Omega\left(N^{2}\right)$ transactions are ordered in one epoch, hence the optimal $O(1)$ per transaction is achieved. However, the design of HBBFT that is optimized towards low communication complexity has the negative effect that the latency might be large under high load. More precisely, if $\beta N^{2}$ transactions are pending in the system ${ }^{6}$ when $t x$ is being input, the latency of $t x$ is $\approx \beta$ epochs, thus $\approx \beta \log (N)$ rounds. Our algorithm, when adjusted to this model would achieve $\approx \beta$ rounds of latency (thus $\log (N)$-factor improvement), while retaining the same, optimal communication complexity.

In this paper we propose a different assumption on the transaction buffers that allows us to better demonstrate the capabilities of our protocol when it comes to latency. We assume that at every round the ratio between lengths of transaction buffers of any two honest nodes is at most a fixed constant. In this model, our protocol achieves $O(1)$ latency, while a natural adaptation of HBBFT would achieve latency $O(\log (N))$, thus again a factor-log$(N)$ improvement. A qualitative improvement over HBBFT that we achieve in this paper is that we completely get rid of the trusted dealer assumption. We also note that the definitions of Atomic Broadcast slightly differ between this paper and [35]: we achieve Censorship Resilience
\footnotetext{
${ }^{6}$ For the sake of this comparison we only consider transactions that have been input to $\Omega(N)$ honest nodes.
}
assuming that it was input to even a single honest node, while in [35] it has to be input to $\Omega(N)$ nodes.

The recent work ${ }^{7}$ of Abraham et al. [3] studies a closely related Validated Asynchronous Byzantine Agreement (VABA) problem, which is, roughly speaking, the problem of picking one value out of $N$ proposed by the nodes. The protocol in [3] achieves $O(1)$ latency and has optimal communication complexity of $O(N)$ per node. We believe that combining it with the ideas present in HoneyBadgerBFT can yield an algorithm with the same communication complexity as HoneyBadgerBFT but with latency improved by a factor of $\log N$ However, when compared to ours, such a protocol still requires a trusted dealer and achieves weaker censorship resilience.

Finally, we remark that our algorithm is based on maintaining a DAG data structure representing the communication history of the protocol. This can be seen as a realization of Lamport's "happenedbefore" relation [33] or the so-called Causal Order [27]. To the best of our knowledge, the first instance of using DAGs to design asynchronous protocols is the work of [37]. More recently DAGs gained more attention in the blockchain space [4, 23, 44].

Common Randomness. For a thorough overview of previous work on generating randomness in distributed systems and a discussion on the novelty of our solution we refer to Section 4.1.

\section{ATOMIC BROADCAST}

This section is devoted to an outline and discussion of the Aleph protocol. We start by sketching the overall idea of the algorithm and explaining how it works from a high-level perspective. In this process we present the most basic variant of the Aleph protocol, which already contains all the crucial ideas and gives a provably correct implementation of Atomic Broadcast. On the theoretical side, however, this basic variant might suffer from a slightly suboptimal communication complexity. In Section D we describe a simple tweak to the protocol which allows us to finally achieve the communication complexity as claimed in Theorem 2.1. We refer to Sections $C$ and $D$ for proofs of correctness and efficiency of Aleph.

\subsection{Asynchronous communication as a DAG}

The concept of a "communication round" as explained in the preliminaries, is rather natural in the synchronous model, but might be hard to grasp when talking about asynchronous settings. This is one of the reasons why asynchronous models are, generally speaking, harder to work with than their (partially) synchronous counterparts, especially when it comes to proving properties of such protocols.

Units and DAGs. To overcome the above issue, we present a general framework for constructing and analyzing asynchronous protocols that is based on maintaining (by all the nodes) the common "communication history" in the form of an append-only structure: a DAG (Directed Acyclic Graph).

The DAGs that we consider in this paper originate from the following idea: we would like to divide the execution of the algorithm into virtual rounds so that in every round $r$ every node emits exactly one Unit that should be thought of as a message broadcast to all the other nodes. Moreover, every such unit should have "pointers" to a
\footnotetext{
${ }^{7}$ We would like to thank the anonymous reviewer for bringing this work to ou attention.
}

large enough number of units from the previous round, emitted by other nodes. Such pointers can be realized by including hashes of the corresponding units, which, assuming that our hash function is collision-free, allows to uniquely determine the "parent units". Formally, every unit has the following fields:
- Creator. Index and a signature of unit's creator.
- Parents. A list of units' hashes.
- Data. Additional data to be included in the unit.

The fact that a unit $U$ has another unit $V$ included as its parent signifies that the information piece carried by $V$ was known to $U$ 's creator at the time of constructing $U$, i.e., $V$ causally preceeds $U$. All the nodes are expected to generate such units in accordance to some initially agreed upon rules (defined by the protocol) and maintain their local copies of the common DAG, to which new units are continuously being added.

Communication History DAGs. To define the basic rules of creating units note first that this DAG structure induces a partial ordering on the set of units. To emphasize this fact, we often write $V \leqslant U$ if either $V$ is a parent of $U$, or more generally (transitive closure) that $V$ can be reached from $U$ by taking the "parent pointer" several times. This also gives rise to the notion of DAG-round of a unit $U$ that is defined to be the maximum length of a downward chain starting at $U$. In other words, recursively, a unit with no parents has DAG-round 0 and otherwise a unit has DAG-round equal to the maximum of DAG-rounds of its parents plus one. We denote the DAG-round of a unit $U$ by $\mathrm{R}(U)$. Usually we just write "round" instead of DAG-round except for parts where the distinction between DAG-round and async-round (as defined in Section G) is relevant (i.e., mostly Section D). We now proceed to define the notion of a ch-DAG (communication history DAG) that serves as a backbone of the Aleph protocol.

DEFINITION 3.1 (ch-DAG). We say that a set of units $\mathcal{D}$ created by $N=3 f+1$ nodes forms a ch-DAG if the parents of every unit in $\mathcal{D}$ also belong to $\mathcal{D}$ and additionally the following conditions hold true

(1) Chains. For each honest node $\mathcal{P}_{i} \in \mathcal{P}$, the set of units in $\mathcal{D}$ created by $\mathcal{P}_{i}$ forms a chain.

(2) Dissemination. Every round-r unit in $\mathcal{D}$ has at least $2 f+1$ parents of round $r-1$.

(3) Diversity Every unit in $\mathcal{D}$ has parents created by pairwise distinct nodes.

What the above definition tries to achieve is that, roughly, every node should create one unit in every round and it should do so after learning a large enough portion (i.e. at least $2 f+1$ ) of units created in the previous round. The purpose of the Chains rule is to forbid forking, i.e., a situation where a node creates more than one unit in a single round. The second rule, Dissemination, guarantees that a node creating a unit in round $r$ learned as much as possible from the previous round - note that as there are only $N-f=2 f+1$ honest nodes in the system, we cannot require that a node receives more than $2 f+1$ units, as byzantine nodes might not have created them. The unit may have additional parents, but the Diversity rule ensures that they are created by different nodes - otherwise units could become progressively bigger as the ch-DAG grows, by linking
to all the units in existence, hence increasing the communication complexity of the protocol.

Building DAGs. The pseudocode DAG $-\operatorname{Grow}(\mathcal{D})$ provides a basic implementation of a node that takes part in maintaining a common DAG. Such a node initializes first $\mathcal{D}$ to an empty DAG and then runs two procedures CreateUnit and ReceiveUnits in parallel To create a unit at round $r$, we simply wait until $2 f+1$ units of round $r-1$ are available and then, for every node, we pick its unit of highest round (i.e., of round at most $r-1$ ) and include all these $N$ (unless some nodes created no units at all, in which case $<N$ ) units as parents of the unit. In other words, we wait just enough until we can advance to the next round, and attach to our round- $r$ unit everything we knew at that point in time.

```
DAG-Grow(D):
    CreateUnit(data):
        for r = 0, 1,2,\ldots. do
            if r>0 then
                wait until |{U\in\mathcal{D}:\textrm{R}(U)=r-1}|\geqslant2f+1
            P \leftarrow \{ \text { maximal } \mathcal { P } _ { i } \text { 's unit of round <r in } \mathcal { D } : \mathcal { P } _ { i } \in \mathcal { P } \}
            create a new unit U with }P\mathrm{ as parents
            include data in }
            add U to }\mathcal{D
            RBC(U)
ReceiveUnits:
    loop forever
        upon receiving a unit U via RBC do
            add U to }\mathcal{D
```

Both CreateUnit and ReceiveUnits make use of a primitive called RBC that stands for Reliable Broadcast. This is an asynchronous protocol that guarantees that every unit broadcast by an honest node is eventually received by all honest nodes. More specifically we use the validated RBC protocol which also ensures that incorrect units (with incorrect signatures or incorrect number of parents, etc.) are never broadcast successfully. Furthermore, our version of RBC forces every node to broadcast exactly one unit per round, thus effectively banning forks. We refer to Section F for a thorough discussion on Reliable Broadcast.

The validated RBC algorithm internally checks whether a certain Valid $(U)$ predicate is satisfied when receiving a unit $U$. This predicate makes sure that the requirements in Definition 3.1 are satisfied, as well as verifies that certain data, required by the protocol is included in $U$. Consequently, only valid units are added to the local ch-DAGs maintained by nodes. Furthermore, as alluded above there can be only a single copy of a unit created by a particular node in a particular round. This guarantees that the local copies of ch-DAGs maintained by different nodes always stay consistent.

Let us now describe more formally the desired properties of a protocol used to grow and maintain a common ch-DAG. For this it is useful to introduce the following convention: we denote the local copy of the ch-DAG maintaned by the $i$ th node by $\mathcal{D}_{i}$.

Definition 3.2. We distinguish the following properties of a protocol for constructing a common ch-DAG
(1) Reliable: for every unit $U$ added to a local copy $\mathcal{D}_{i}$ of an honest node $\mathcal{P}_{i}, U$ is eventually added to the local copy $\mathcal{D}_{j}$ of every honest node $\mathcal{P}_{j}$.

(2) Ever-expanding: for every honest node $\mathcal{P}_{i}$ the local copy $\mathcal{D}_{i}$ grows indefinitely, i.e., $\mathrm{R}\left(\mathcal{D}_{i}\right):=\max \left\{r(U) \mid U \in \mathcal{D}_{i}\right\}$ is unbounded.

(3) Fork-free: whenever two honest nodes $i_{1}, i_{2}$ hold units $U_{1} \in$ $\mathcal{D}_{i_{1}}$ and $U_{2} \in \mathcal{D}_{i_{2}}$ such that both $U_{1}, U_{2}$ have the same creator and the same round number, then $U_{1}=U_{2}$.

Having these properties defined, we are ready to state the main theorem describing the process of constructing ch-DAG by the DAG-Grow protocol.

ThEorem 3.1. The DAG-Grow protocol is reliable, ever-expanding, and fork-free Additionally, during asynchronous round $r$ each honest node holds a local copy of $\mathcal{D}$ of round at least $\Omega(r)$.

For a proof we refer the reader to Section B.1.

Benefits of Using DAGs. After formally introducing the idea of a ch-DAG and explaining how it is constructed we are finally ready to discuss the significance of this concept. First of all, ch-DAGs allow for a clean and conceptually simple separation between the communication layer (sending and receiving messages between nodes) and the protocol logic (mainly deciding on relative order of transactions). Specifically, the network layer is simply realized by running Reliable Broadcast in the background, and the protocol logic is implemented as running off-line computations on the local copy of the ch-DAG. One can think of the local copy of the ch-DAG as the state of the corresponding node; all decisions of a node are based solely on its state. One important consequence of this separation is that the network layer, being independent from the logic, can be as well implemented differently, for instance using ordinary broadcast or random gossip (see Section A).

In the protocol based on ch-DAGs the concept of an adversary and his capabilities are arguably easier to understand. The ability of the adversary to delay a message now translates into a unit being added to some node's local copy of the ch-DAG with a delay. Nonetheless, every unit that has ever been created will still be eventually added to all the ch-DAGs maintained by honest nodes. A consequence of the above is that the adversary can (almost arbitrarily) manipulate the structure of the ch-DAG, or, in other words, he is able to force a given set of round- $(r-1)$ parents for a given round- $r$ unit. But even then, it needs to pick at least $2 f+1$ round$(r-1)$ units, which enforces that more than a half of every unit's parents are created by honest nodes.

\subsection{Atomic broadcast via ch-DAG}

In this section we show how to build an Atomic Broadcast protocol based on the ch-DAG maintained locally by all the nodes. Recall that nodes receive transactions in an on-line fashion and their goal is to construct a common linear ordering of these transactions. Every node thus gathers transactions in its local buffer and whenever it creates a new unit, all transactions from its buffer are included in the data field of the new unit and removed from the buffer. ${ }^{8}$
\footnotetext{
${ }^{8}$ This is the simplest possible strategy for including transactions in the ch-DAG and while it is provably correct it may not be optimal in terms of communication complexity. We show how to fix this in Section D.
}

Thus, to construct a common linear ordering on transactions it suffices to construct a linear ordering of units in the ch-DAG (the transactions within units can be ordered in an arbitrary yet fixed manner, for instance alphabetically). The ordering that we are going to construct also has the nice property that it extends the ordering of units induced by the ch-DAG (i.e. the causal order).

Let us remark at this point that all primitives that we describe in this section take a local copy $\mathcal{D}$ of the ch-DAG as one of their parameters and return either
- a result (which might be a single bit, a unit, etc.), or
- $\perp$, signifying that the result is not yet available in $\mathcal{D}$.

The latter means that in order to read off the result, the local copy $\mathcal{D}$ needs to grow further. We occasionally omit the $\mathcal{D}$ argument, when it is clear from the context which local copy should be used.

Ordering Units. The main primitive that is used to order units in the ch-DAG, OrderUnits, takes a local copy $\mathcal{D}$ of the ch-DAG and outputs a list linord that contains a subset of units in $\mathcal{D}$. This list is a prefix of the global linear ordering that is commonly generated by all the nodes. We note that linord will normally not contain all the units in $\mathcal{D}$ but a certain subset of them. More precisely, linord contains all units in $\mathcal{D}$ except those created in the most recent (typically around 3) rounds. While these top units cannot be ordered yet, the structural information about the ch-DAG they carry is used to order the units below them. Intuitively, the algorithm that is run in the ch-DAG at round $r$ makes decisions regarding units that are several rounds deeper, thus the delay.

Note that different nodes might hold different versions of the ch-DAG at any specific point in time, but what we guarantee in the ch-DAG growing protocol is that all copies of ch-DAG are consistent, i.e., all the honest nodes always see exactly the same version of every unit ever created, and that every unit is eventually received by all honest nodes. The function OrderUnits is designed in such a way that even when called on different versions of the ch-DAG $\mathcal{D}_{1}$, $\mathcal{D}_{2}$, as long as they are consistent, the respective outputs linorder ${ }_{1}$, linorder $_{2}$ also agree, i.e., one of them is a prefix of the other.

The OrderUnits primitive is rather straightforward. At every round $r$, one unit $V_{r}$ from among units of round $r$ is chosen to be a "head" of this round, as implemented in ChooseHead. Next, all the units in $\mathcal{D}$ that are less than $V_{r}$, but are not less than any of $V_{0}, V_{1}, \ldots, V_{r-1}$ form the $r$ th batch of units. The batches are sorted by their round numbers and units within batches are sorted topologically breaking ties using the units' hashes.

Choosing Heads. Perhaps surprisingly, the only nontrivial part of the protocol is choosing a head unit for each round. It is not hard to see that simple strategies for choosing a head fail in an asynchronous network. For instance, one could try picking always the unit created by the first node to be the head: this does not work because the first node might be byzantine and never create any unit. To get around this issue, one could try another tempting strategy: to choose a head for round $r$, every node waits till round $r+10$, and declares as the head the unit of round $r$ in its copy of the ch-DAG that has the smallest hash. This strategy is also doomed to fail, as it might cause inconsistent choices of heads between nodes: this can happen when some of the nodes see a unit with a very small hash in their ch-DAG while the remaining ones did not receive it yet, which might have either happened just by accident or was

```
Aleph:
    OrderUnits( $\mathcal{D}):$
        linord $\leftarrow[]$
        for $r=0,1, \ldots, \mathrm{R}(\mathcal{D})$ do
            $V_{r} \leftarrow$ ChooseHead $(r, \mathcal{D})$
            if $V_{r}=\perp$ then break
            batch $\leftarrow\left\{U \in \mathcal{D}: U \leqslant V_{r}\right.$ and $U \notin$ linord $\}$
            order batch deterministically
            append batch to linord
        output linord
    ChooseHead $(r, \mathcal{D})$
        $\pi_{r} \leftarrow$ GeneratePermutation $(r, \mathcal{D})$
        if $\pi_{r}=\perp$ then output $\perp$
        else
            $\left(U_{1}, U_{2}, \ldots, U_{k}\right) \leftarrow \pi_{r}$
            for $i=1,2, \ldots, k$ do
                if $\operatorname{Decide}\left(U_{i}, \mathcal{D}\right)=1$ then
                    output $U_{i}$
                    else if $\operatorname{Decide}\left(U_{i}, \mathcal{D}\right)=\perp$ then
                    output $\perp$
        output $\perp$
```

forced by actions of the adversary. Note that under asynchrony, one can never be sure whether missing a unit from some rounds back means that there is a huge delay in receiving it or it was never created (the creator is byzantine). More generally, this also justifies that in any asynchronous BFT protocol it is never correct to wait for one fixed node to send a particular message.

Our strategy for choosing a head in round $r$ is quite simple: pick the first unit (i.e., with lowest creator id) that is visible by every node. The obvious gap in the above is how do we decide that a particular unit is visible? As observed in the example above, waiting a fixed number of rounds is not sufficient, as seeing a unit locally does not imply that all other nodes see it as well. Instead, we need to solve an instance of Binary Consensus (also called Binary Agreement). In the pseudocode this is represented by a $\operatorname{Decide}(U)$ function that outputs 0 or 1 ; we discuss it in the subsequent paragraph.

There is another minor adjustment to the above scheme that aims at decreasing the worst case latency, which in the just introduced version is $O(\log N)$ rounds ${ }^{9}$. When using a random permutation (unpredictable by the adversary) instead of the order given by the units creator indices, the latency provably goes down to $O(1)$. Such an unpredictable random permutation is realized by the GeneratePermutation function.

Consensus. For a round- $(r+1)$ unit $U$, by $\downarrow(U)$ we denote the set of all round- $r$ parents of $U$. Consider now a unit $U_{0}$ in round $r$; we would like the result of $\operatorname{Decide}\left(U_{0}, \mathcal{D}\right)$ to "answer" the question whether all nodes can see the unit $U_{0}$. This is done through voting: starting from round $r+1$ every unit casts a "virtual" vote ${ }^{10}$ on $U_{0}$. These votes are called virtual because they are never really
\footnotetext{
${ }^{9}$ The reason is that the adversary could cause the first $\Omega(N)$ units to be decided 0 . Since the delay of each such decision is a geometric random variable with expectation $\theta(1)$, the maximum of $\Omega(N)$ of them is $\Omega(\log N)$.

${ }^{10}$ The idea of virtual voting was used for the first time in [37].
}
broadcast to other nodes, but they are computed from the ch-DAG For instance, at round $r+1$, a unit $U$ is said to vote 1 if $U_{0}<U$ and 0 otherwise, which directly corresponds to the intuition that the nodes are trying to figure out whether $U_{0}$ is visible or not.

```
Aleph-Consensus( $\mathcal{D})$ :
    $\operatorname{Vote}\left(U_{0}, U, \mathcal{D}\right)$ :
        if $\mathrm{R}(U) \leqslant \mathrm{R}\left(U_{0}\right)+1$ then output ${ }^{11}\left[U_{0}<U\right]$
        else
            $A \leftarrow\left\{\operatorname{Vote}\left(U_{0}, V, \mathcal{D}\right): V \in \downarrow(U)\right\}$
            if $A=\{\sigma\}$ then output $\sigma$
            else output CommonVote $\left(U_{0}, \mathrm{R}(U), \mathcal{D}\right)$
    UnitDecide $\left(U_{0}, U, \mathcal{D}\right)$ :
        if $\mathrm{R}(U)<\mathrm{R}\left(U_{0}\right)+2$ then output $\perp$
        $\mathrm{v} \leftarrow \operatorname{CommonVote}\left(U_{0}, U\right)$
        if $\mid\left\{V \in \downarrow(U)\right.$ : $\left.\operatorname{Vote}\left(U_{0}, V\right)=v\right\} \mid \geqslant 2 f+1$ then output $v$
        else output $\perp$
    Decide $\left(U_{0}, \mathcal{D}\right)$ :
        if $\exists_{U \in \mathcal{D}}$ UnitDecide $\left(U_{0}, U, \mathcal{D}\right)=\sigma \in\{0,1\}$ then
            output $\sigma$
        else output $\perp$
```

Starting from round $r+2$ every unit can either make a final decision on a unit or simply vote again. This process is guided by the function CommonVote $\left(U_{0}, r^{\prime}, \mathcal{D}\right)$ that provides a common bit $\in\{0,1\}$ for every round $r^{\prime} \geqslant r+2$. Suppose now that $U$ is of round $r^{\prime} \geqslant r+2$ and at least $2 f+1$ of its round- $\left(r^{\prime}-1\right)$ parents in the ch-DAG voted 1 for $U_{0}$, then if CommonVote $\left(U_{0}, r^{\prime}, \mathcal{D}\right)=1$, then unit $U$ is declared to decide $U_{0}$ as 1 . Otherwise, if either there is no supermajority vote among parents (i.e., at least $2 f+1$ matching votes) or the supermajority vote does not agree with the CommonVote for this round, the decision is not made yet. In this case, the unit $U$ revotes using either the vote suggested by its parents (in case it was unanimous) or using the default CommonVote. Whenever any of the units $U \in \mathcal{D}$ decides $U_{0}$ then it is considered decided with that particular decision bit.

Crucially, the process is designed in such a way that when some unit $U$ decides $\sigma \in\{0,1\}$ on some unit $U_{0}$ then we prove that no unit ever decides $\bar{\sigma}$ (the negation of $\sigma$ ) on $U_{0}$ and also that every unit of high enough round decides $\sigma$ on $U_{0}$ as well. At this point it is already not hard to see that if a unit $U$ makes decision $\sigma$ on $U_{0}$ then it follows that all the units of round $\mathrm{R}(U)$ (and any round higher than that) vote $\sigma$ on $U_{0}$. To prove that observe that if a unit $V$ of round $r$ votes $\sigma^{\prime}$ then either:
- all its parents voted $\sigma^{\prime}$ and hence $\sigma^{\prime}=\sigma$, because $U$ and $V$ have at least $f+1$ parents in common, or
- $\sigma^{\prime}=$ CommonVote $\left(U_{0}, \mathrm{R}(V), \mathcal{D}\right)$, but since $U$ decided $\sigma$ for $U_{0}$ then CommonVote $\left(U_{0}, \mathrm{R}(U), \mathcal{D}\right)=\sigma$ and thus $\sigma=\sigma^{\prime}$ because $\mathrm{R}(U)=\mathrm{R}(V)$.

The above gives a sketch of "safety" proof of the protocol, i.e., that there will never be inconsistent decisions regarding a unit $U_{0}$.
\footnotetext{
${ }^{11}$ In the expression $\left[U_{0}<U\right]$ we use the Iverson bracket notation, i.e., $\left[U_{0}<U\right]=1$ if $U_{0}<U$ and it is 0 otherwise.
}

Another property that is desirable for a consensus protocol is that it always terminates, i.e., eventually outputs a consistent decision. In the view of the FLP Theorem [24] this cannot be achieved in the absence of randomness in the protocol. This is the reason why we need to inject random bits to the protocol and this is done in CommonVote. We show that, roughly, if CommonVote provides a random bit (that cannot be predicted by the adversary well in advance) then at every round the decision process terminates with probability at least $1 / 2$. Thus, the expected number of rounds until termination is $O(1)$.

We provide formal proofs of properties of the above protocol in Section D. In the next section, we show how the randomness in the protocol is generated to implement CommonVote.

\subsection{Common Randomness}

As already alluded to in Subsection 3.2, due to FLP-impossibility [24] there is no way to achieve binary agreement in finite number of rounds when no randomness is present in the protocol. We also note here that not any kind of randomness will do and the mere fact that a protocol is randomized does not "protect" it from FLPimpossibility. It is crucial to keep the random bits hidden from the adversary till a certain point, intuitively until the adversary has committed to what decision to pursue for a given unit at a certain round. When the random bit is revealed after this commitment, then there is a probability of $1 / 2$ that the adversary "is caught" and cannot delay the consensus decision further.

That means, in particular, that a source of randomness where the nodes are initialized with a common random seed and use a pseudorandom number generator to extract fresh bits is not sufficient, since such a strategy actually makes the algorithm deterministic.

Another issue when generating randomness for the protocol is that it should be common, thus in other words, the random bit output by every honest node in a particular round $r$ should agree between nodes. While this is strictly required for safety of the algorithm that is presented in the paragraph on Consensus, one can also consider simple variants thereof for which a relaxed version of commonness is sufficient. More specifically, if the bit output at any round is common with probability $p \in(0,1]$ then one can achieve consensus in $O(1 / p)$ rounds. Indeed, already in [9] Bracha constructed such a protocol and observed that if every node locally tosses a coin independently from the others than this becomes a randomness source that is common with probability $p \approx 2^{-N}$ and thus gives a protocol that takes $O\left(2^{N}\right)$ rounds to terminate. We refer to Subsection 2.2 for an overview of previous work on common randomness.

In our protocol, randomness is injected via a single procedure SecretBits whose properties we formalize in the definition below

Definition 3.3 (Secret Bits). The SecretBits(i,revealRound) primitive takes an index $i \in\{1,2, \ldots, N\}$ and revealRound $\in \mathbb{N}$ as parameters, outputs a $\lambda$-bits secret $s$, and has the following properties whenever initiated by the ChooseHead protocol

(1) no computationally bounded adversary can guess $s$ with non-negligible probability, as long as no honest node has yet created a unit of round revealRound,

(2) the secret s can be extracted by every node that holds any unit of round revealRound +1 .

As one might observe, the first parameter $i$ of the SecretBits function seems redundant. Indeed, given an implementation of SecretBits, we could as well use $\operatorname{SecretBits}(1, \cdot)$ in place of $\operatorname{SecretBits}(i, \cdot)$ for any $i \in[N]$ (here and in the remaining part of the paper $[N]$ stands for $\{1, \ldots, N\})$ and it would seemingly still satisfy the definition above. The issue here is very subtle and will become clear only in Section 4, where we construct a SecretBits function whose computability is somewhat sensitive to what $i$ it is run with ${ }^{12}$. In fact, we recommend the reader to ignore this auxiliary parameter, as our main implementation of SecretBits( $i$, revealRound) is anyway oblivious to the value of $i$ and thus essentially there is exactly one secret per round in the protocol.

The simplest attempt to implement SecretBits would be perhaps to use an external trusted party that observes the growth of the ch-DAG and emits one secret per round, whenever the time comes. Clearly, the correctness of the above relies crucially on the dealer being honest, which is an assumption we cannot make: indeed if there was such an honest entity, why do not we just let him order transactions instead of designing such a complicated protocol?

Instead, our approach is to utilize a threshold secret scheme (see [45]). In short, at round $r$ every node is instructed to include its share of the secret (that is hidden from every node) in the unit it creates. Then, in round $r+1$, every node collects $f+1$ different such shares included in the previous round and reconstructs the secret from them. Crucially, any set of $f$ or less shares is not enough to derive the secret, thus the adversary controlling $f$ nodes still needs at least one share from an honest node. While this allows the adversary to extract the secret one round earlier then the honest nodes, this advantage turns out to be irrelevant, as, intuitively, he needed to commit to certain actions several turns in advance (see Section $C$ for a detailed analysis).

Given the SecretBits primitive, we are ready to implement GeneratePermutation and CommonVote (the pseudocode is given in the table Aleph-CommonRandomness).

In the pseudocode, by hash we denote a hash function ${ }^{13}$ that takes an input and outputs a bistring of length $\lambda$. We remark that the CommonVote at round $R\left(U_{0}\right)+4$ being 0 is to enforce that units that are invisible at this round will be quickly decided negatively (see Lemma C.4). To explain the intuition behind GeneratePermutation, suppose for a second that $\operatorname{SecretBits}(i, r+4)$ outputs the same secret $x$ independently of $i$ (as it is the case for our main algorithm). Then the above pseudocode assigns a pseudorandom priority hash $(x \| \mid U)$ (where, for brevity $U$ denotes some serialization of $U$ ) to every unit $U$ or round $r$ which results in a random ordering of these units, as required.

At this point, the only remaining piece of the protocol is the SecretBits procedure. We provide two implementations in the subsequent Section (see Lemma 4.1): one simple, but requiring a trusted dealer, and the second, more involved but completely trustless.

\section{RANDOMNESS BEACON}

The goal of this section is to construct an ABFT Randomness Beacon, in order to provide an efficient implementation of SecretBits that is
\footnotetext{
${ }^{12}$ More precisely, intuitively SecretBits $(i, \cdot)$ is not expected to work properly if for instance node $\mathcal{P}_{i}$ has produced no unit at all. Also, importantly, the ChooseHead algorithm will never call SecretBits $(i, \cdot)$ in such a case

${ }^{13}$ We work in the standard Random Oracle Model.
}

```
Aleph-CommonRandomness:
    CommonVote $\left(U_{0}, r, \mathcal{D}\right)$ :
        if $r \leqslant \mathrm{R}\left(U_{0}\right)+3$ then output 1
        if $r=\mathrm{R}\left(U_{0}\right)+4$ then output 0
        else
            $i \leftarrow$ the creator of $U_{0}$
            $x \leftarrow \operatorname{SecretBits}(i, r, \mathcal{D})$
            if $x=\perp$ then output $\perp$
            output the first bit of hash $(x)$
    GeneratePermutation $(r, \mathcal{D})$ :
        for each unit $U$ of round $r$ in $\mathcal{D}$ do
            $i \leftarrow$ the creator of $U$
            $x \leftarrow \operatorname{SecretBits}(i, r+4, \mathcal{D})$
            if $x=\perp$ then output $\perp$
            assign priority $(U) \leftarrow \operatorname{hash}(x \| U) \in\{0,1\}^{\lambda}$
        let $\left(U_{1}, U_{2}, \ldots, U_{k}\right)$ be the units in $\mathcal{D}$ of round $r$ sorted by
        priority $(\cdot)$
        output $\left(U_{1}, U_{2}, \ldots, U_{k}\right)$
```

required by our Atomic Broadcast protocol. We start by a detailed review of previous works and how do they compare to the result of this paper. Subsequently we describe how to extract randomness from threshold signatures, which is a basic building block in our approach, and after that we proceed with the description of our randomness beacon.

\subsection{Comparison to Previous Work on Distributed Randomness Beacons}

We distinguish two main approaches for solving the problem of generating randomness in distributed systems in the presence of byzantine nodes: using Verifiable Secret Sharing (VSS) and via Threshold Signatures. We proceed to reviewing these two approaches and subsequently explain what does our work bring to the field. This discussion is succinctly summarized in Table 1.

Verifiable Secret Sharing. The rough idea of VSS can be explained as follows: a dealer first initializes a protocol to distribute shares $\left(x_{1}, x_{2}, \ldots, x_{N}\right)$ of a secret $x$ to the nodes $1,2, \ldots, N$ so that afterwards every set of $(f+1)$ nodes can extract $x$ by combining their shares, but any subset of $f$ nodes cannot do that ${ }^{14}$. An honest dealer is requested to pick $x$ uniformly at random (and erase $x$ from memory); yet as one can imagine, this alone is not enough to build a reliable, distributed randomness source, with the main issue being: how to elect a dealer with the guarantee that he is honest? In the seminal paper [19] Canetti and Rabin introduce the following trick: let all nodes in the network act as dealers and perform VSS with the intention to combine all these secrets into one (say, by xoring them) that would be random and unpredictable. Turning this idea into an actual protocol that works under full asynchrony is especially tricky, yet [19] manage to do that and end up with a protocol that has $O\left(N^{7}\right)$ communication complexity. This $O\left(N^{7}\right)$ essentially comes from executing an $O\left(N^{5}\right)$ communication VSS protocol $N^{2}$ times. Later on this has been improved by Cachin et
\footnotetext{
${ }^{14}$ The thresholds $f$ and $(f+1)$ here are not the only possible, but are most relevant for our setting, see [14] for a more detailed treatment.
}
al. [14] to $O\left(N^{4}\right)$ by making the VSS protocol more efficient. The issue with the AVSS approach is that the communication cost of generating one portion of randomness is rather high - it requires to start everything from scratch when new random bits are requested. This issue is not present in the approach based on threshold signatures, where a one-time setup is run and then multiple portions of random bits can be generated cheaply.

Threshold Signatures. For a technical introduction to this approach we refer to Section 4.2. The main idea is that if the nodes hold keys for threshold signatures with threshold $f+1$, then the threshold signature $\sigma_{m} \in\{0,1\}^{\lambda}$ of a message $m$ is unpredictable for the adversary and provides us with roughly $\lambda$ bits of entropy.

The idea of using threshold signatures as a source of randomness is not new and has been studied in the literature [16], especially in the context of blockchain [28,35]. While this technique indeed produces unbiased randomness assuming that the tossing keys have been correctly dealt to all nodes, the true challenge becomes How to deal secret keys without involving a trusted dealer? This problem is known in the literature under the name of Distributed Key Generation (DKG). There has been a lot of prior work on DKG [25, 26, 29, 41], however none of the so far proposed protocols has been designed to run under full asynchrony.

Historically, the first implemenation of DKG was proposed in the work of Pedersen [41]. The network model is not formally specified in [41], yet one can implement it in the synchronous BFT model with $f$ out of $N=3 f+1$ nodes being malicious. Pedersen's protocol, as later shown by Gennaro et al. in [26], does not guarantee a uniform distribution over private keys; in the same work [26] a fix is proposed that closes this gap, but also in a later work [25] by the same set of authors it is shown that Pedersen's protocol is still secure ${ }^{15}$. The DFINITY randomness beacon [28] runs DKG as setup and subsequently uses threshold signatures to generate randomness.

We note that the AVSS scheme by Cachin et al. [14] can be also used as a basis of DKG since it is polynomial-based and has the additional property of every node learning a certain "commitment" to the secret shares obtained by all the nodes, which in the setting of threshold signatures corresponds to public keys of the threshold scheme (see Section 4.2). This allows to construct a DKG protocol as follows: each node runs its own instance of AVSS and subsequently the nodes agree on a subset of $\geqslant f+1$ such dealt secrets to build the keys for threshold signatures. This scheme looks fairly simple, yet it is extremely tricky to implement correctly. One issue is security, i.e., making sure that the adversary is not "front-running" and cannot reveal any secret too early, but more fundamentally: this algorithm requires consensus to chooses a subset of secrets. ABFT consensus requires randomness (by FLP theorem [24]), which we are trying to obtain, and thus we fall into an infinite loop. To obtain consensus without the help of a trusted dealer one is forced to use variants of the Canetti-Rabin protocol [19] (recall that we are working here under the hard constraint of having no trusted dealer, which is not satisfied by most protocols in the literature) that unfortunately has very high communication complexity. Nevertheless using this idea, one can obtain an ABFT DKG protocol with $O\left(N^{4}\right)$ communication
\footnotetext{
${ }^{15}$ We note that out protocol can be seen as an adaptation of Pedersen's and thus also requires an ad-hoc proof of security (provided in Lemma E.4), as the distribution of secret keys might be skewed.
}

\begin{tabular}{|c|c|c|c|}
\hline \multirow{2}{*}{ Protocol } & \multirow{2}{*}{ Model } & \multicolumn{2}{|c|}{ Communication } \\
\cline { 3 - 4 } & Setup & Query \\
\hline \hline This Work & async. BFT & $O\left(N^{2} \log N\right)$ & $O(N)$ \\
\hline Canetti, Rabin [19] & async. BFT & - & $O\left(N^{7}\right)$ \\
\hline Cachin et al. [14] & async. BFT & - & $O\left(N^{4}\right)$ \\
\hline Kate et al. DKG [29] & w. sync. BFT & $O\left(N^{3}\right)$ & - \\
\hline Pedersen DKG [25, 41] & sync. BFT & $O\left(N^{2}\right)$ & - \\
\hline Gennaro et al. DKG [26] & sync. BFT & $O\left(N^{2}\right)$ & - \\
\hline DFINITY [28] & sync. BFT & $O\left(N^{2}\right)$ & $O(N)$ \\
\hline RandShare [46] & sync. BFT & - & $O\left(N^{2}\right)$ \\
\hline SCRAPE [20] & sync. BFT & - & $O\left(N^{2}\right)$ \\
\hline
\end{tabular}

Table 1: Comparison of randomness beacons. The Model column specifies what are the necessary assumptions for the protocol to work. The Communication columns specify communication complexities (per node) of a one-time setup phase (run at the very beginning), and of one query for fresh random bits. Some of them were designed for DKG and thus we do not specify the complexity of query (yet it can be made $O(N)$ in all cases). Some protocols also do not run any setup phase, in which case the complexity is omitted.

complexity. The approach of [29] uses the ideas of [14] and roughly follows the above outlined idea, but avoids the problem of consensus by working in a relaxed model: weak synchrony (which lies in between partial synchrony and full asynchrony).

Our Approach. We base our randomness beacon on threshold signatures, yet make a crucial observation that full DKG is not necessary for this purpose. Indeed what we run instead as setup (key generation phase) of our randomness beacon is a weaker variant of DKG. Very roughly: the end result of this variant is that the keys are dealt to a subset of at least $2 f+1$ nodes (thus possibly up to $f$ nodes end up without keys). This allows us to construct a protocol with setup that incurs only $\widetilde{O}\left(N^{2}\right)$ communication, whereas a variant with full $\mathrm{DKG}^{16}$ would require $\widetilde{O}\left(N^{3}\right)$.

To obtain such a setup in an asynchronous setting we need to deal with the problem (that was sketched in the previous paragraph) of reaching consensus without having a common source of randomness in place. Intuitively a "disentanglement" of these two problems seems hard: in one direction this can be made formal via FLP Theorem [24] (consensus requires randomness); in the opposite direction, intuitively, generating a common coin toss requires all the nodes to agree on a particular random bit.

Canetti and Rabin present an interesting way to circumvent this problem in [19]: their consensus algorithm requires only a "weak coin", i.e., a source of randomness that is common (among nodes) with constant, non-zero probability (and is allowed to give different results for different nodes otherwise). Then, via a beautiful construction they obtain such a randomness source that (crucially) does not require any consensus.

Our way to deal with this problem is different. Roughly speaking: in the setup each node "constructs" its unbiased source of randomness and reliably broadcasts it to the remaining nodes. Thus after
\footnotetext{
${ }^{16}$ We do not describe this variant in this paper; it can be obtained by replacing univariate by bivariate polynomials and adjusting the protocol accordingly (see [14]).
}
this step, the problem becomes to pick one out of $N$ randomness sources (consensus). To this end we make a binary decision on each of these sources and pick as our randomness beacon the first that obtained a decision of " 1 "; however each of these binary consensus instances uses its own particular source of randomness (the one it is making a decision about). At the same time we make sure that the nodes that were late with showing their randomness sources to others will be always decided 0 . While this is the basic idea behind our approach, there are several nontrivial gaps to be filled in order to obtain $\widetilde{O}\left(N^{2}\right)$ communication complexity and $O(1)$ latency.

Other Designs. More recently, in the work of [46] a randomness source RandShare has been introduced, which runs a certain variant of Pedersen's protocol to extract random bits. The protocol is claimed by the authors to work in the asynchronous model, yet fails to achieve that goal, as any protocol that waits for messages from all the nodes to proceed, fails to achieve liveness under asynchrony (or even under partial synchrony). In the Table 1 we list it as synchronous BFT, as after some adjustments it can be made to run in this model. In the same work [46] two other randomness beacons are also proposed: RandHound and RandHerd, yet both of them rely on strong, non-standard assumptions about the network and the adversary and thus we do not include them in Table 1. The method used by SCRAPE [20] at a high level resembles Pedersen's protocol but requires access to a blockchain in order to have a total order on messages sent out during the protocol executation.

Finally, we mention an interesting line of work on generating randomness based on VDFs (Verifiable Delay Functions [7, 34, 42, 47]). Very roughly, the idea is to turn a biasable randomness (such as the hash of a Bitcoin block) into unbiasable randomness via a VDF, i.e., a function $f:\{0,1\}^{\lambda} \rightarrow\{0,1\}^{\lambda}$ that cannot be computed quickly and whose computation cannot be parallelized, yet it is possible to prove that $f(x)=y$ for a given $y$ much faster than actually computing $f(x)$. The security of this approach relies on the assumption that the adversary cannot evaluate $f$ at random inputs much faster than honest participants.

\subsection{Randomness from Threshold Signatures}

In this subsection we present the main cryptographic component of our construction: generating randomness using Threshold Signatures. When using a trusted dealer, this component is already enough to implement SecretBits, but the bulk of this section is devoted to proving that we can eliminate the need for a trusted dealer.

Randomness through Signatures. The main idea for generating randomness is as follows: suppose that there is a key pair $(t k, v k)$ of private key and public key, such that $t k$ is unknown, while $v k$ is public, and $t k$ allows to sign messages for some public-key cryptosystem that is deterministic ${ }^{17}$. (We refer to $t k$ as to the "tossing key" while $v k$ stands for "verification key"; we use these names to distinguish from the regular private-public key pairs held by the nodes.) Then, for any message $m$, its digital signature $\sigma_{m}$ generated with respect to $t k$ cannot be guessed, but can be verified using $v k$ thus hash $\left(\sigma_{m}\right) \in\{0,1\}^{\lambda}$ provides us with $\lambda$ random bits! There seems to be a contradiction here though: how can the tossing key
\footnotetext{
${ }^{17} \mathrm{~A}$ system that for a given message $m$ there exists only one correct signature for key pair $(t k, v k)$.
}

be secret and at the same time we are able to sign messages with it? Surprisingly, this is possible using Threshold Cryptography: the tossing key $t k$ is "cut into pieces" and distributed among $N$ nodes so that they can jointly sign messages using this key but no node (or a group of dishonest nodes) can learn $t k$.

More specifically, we use a threshold signature scheme built upon BLS signatures [8]. Such a scheme works over a GDH group $G$, i.e., a group in which the computational Diffie-Hellman problem is hard (i.e. computing $g^{x y}$ given $g^{x}, g^{y} \in G$ ) but the decisional Diffie-Hellman problem is easy (i.e. verifying that $z=x y$ given $\left.g^{x}, g^{y}, g^{z} \in G\right)$. For more details and constructions of such groups we refer the reader to [8]. We assume from now on that $G$ is a fixed, cyclic, GDH group generated by $g \in G$ and that the order of $G$ is a large prime $q$. A tossing key $t k$ in BLS is generated as a random element in $\mathbb{Z}_{q}$ and the public key is $y=g^{t k}$. A signature of a message $m$ is simply $\sigma_{m}=\widetilde{m}^{t k}$, where $\widetilde{m} \in G$ is the hash (see [8] for a construction of such hash functions) of the message being a random element of $G$.

Distributing the Tossing Key. To distribute the secret tossing key among all nodes, the Shamir's Secret Sharing Scheme [45] is employed. A trusted dealer generates a random tossing key $t k$ along with a random polynomial $A$ of degree $f$ over $\mathbb{Z}_{q}$ such that $A(0)=t k$ and privately sends $t k_{i}=A(i)$ to every node $i=$ $1,2, \ldots, N$. The dealer also publishes verification keys, i.e., $V K=$ $\left(g^{t k_{1}}, \ldots, g^{t k_{N}}\right)$. Now, whenever a signature of a message $m$ needs to be generated, the nodes generate shares of the signature by signing $m$ with their tossing keys, i.e., each node $i$ multicasts $\widetilde{m}^{A(i)}$. The main observation to make is that now $\widetilde{m}^{A(0)}$ can be computed given only $\widetilde{m}^{A(j)}$ for $f+1$ different $j$ 's (by interpolation) and thus it is enough for a node $\mathcal{P}_{j}$ to multicast $\widetilde{m}^{A(j)}$ as its share and collect $f$ such shares from different nodes to recover $\sigma_{m}$. On the other hand, any collection of at most $f$ shares is not enough to do that, therefore the adversary cannot sign $m$ all by himself. For details, we refer to the pseudocode of ThresholdSignatures.

SecretBits Through Threshold Signatures. Given the just introduce machinery of threshold signatures, the SecretBits $(i, r)$ primitive is straightforward to implement. Moreover, as promised in Section 3, we give here an implementation that is oblivious to its first argument, i.e., it does only depend on $r$, but not on $i$.

First of all, there is a setup phase whose purpose is to deal keys for generating secrets to all nodes. We start by giving a simple version in which an honest dealer is required for the setup. Subsequently, we explain how can this be replaced by a trustless setup, to yield the final version of SecretBits.

In the simpler case, the trusted dealer generates tossing keys and verification keys ( $T K, V K$ ) for all the nodes using the GenerateKeys procedure, and then openly broadcasts $V K$ to all nodes and to every node $i$ he secretly sends the tossing key $t k_{i}$.

Given such a setup, when $\operatorname{SecretBits}(j, r)$ is executed by the protocol, every node $i$ can simply ignore the $j$ argument and generate

$$
s_{i}(m) \leftarrow \text { CreateShare }\left(m, t k_{i}\right)
$$

where $m$ is a nonce determined from the round number, say $m=$ " $r$ ". Next, after creating its round- $(r+1)$ unit $U$, the $P_{i}$ collects all the

```
ThresholdSignatures:
    GenerateKeys():
        Let $G=\langle g\rangle$ be a GDH group of prime order $q$
        generate a random polynomial $A$ of degree $f$ over $\mathbb{Z}_{q}$
        let $T K=\left(t k_{1}, \ldots, t k_{N}\right)$ with $t k_{i}=A(i)$ for $i \in[N]$
        let $V K=\left(v k_{1}, \ldots, v k_{N}\right)$ with $v k_{i}=g^{t k_{i}}$ for $i \in[N]$
        output $(T K, V K)$
    CreateShare $\left(m, t k_{i}\right)$ :
        $\widetilde{m} \leftarrow \operatorname{hash}(m)$
        output $\widetilde{m}^{t k_{i}}$
    VerifyShare $(m, s, i, V K)$ :
        $\widetilde{m} \leftarrow \operatorname{hash}(m)$
        /* can do the check below since $G$ is GDH */
        if $\log _{g}\left(v k_{i}\right)=\log _{\tilde{m}}(s)$ then
            output True
        else output False
    GenerateSignature $(m, S, V K)$ :
        /* Let $S=\left\{\left(s_{i}, i\right)\right\}_{i \in P}$ where $|P|=f+1 \quad * /$
        /* Assume $\forall_{j} \operatorname{VerifyShare}\left(m, s_{j}, i_{j}\right)=$ True */
        interpolate $A(0)$, i.e., find $l_{1}, l_{2}, \ldots, l_{f+1} \in \mathbb{Z}_{q}$ s.t.
                    $A(0)=\sum_{j=1}^{f+1} l_{j} A\left(i_{j}\right)$
        $\sigma_{m} \leftarrow \prod_{j=1}^{f+1} s_{j}^{l_{j}}$
        output $\sigma_{m}$
```

shares $S$ included in $U$ 's parents at round $r$ and computes:

$$
\begin{aligned}
\sigma_{m} & \leftarrow \operatorname{GenerateSignature}(m, S, V K) \\
s(m) & \leftarrow \operatorname{hash}\left(\sigma_{m}\right)
\end{aligned}
$$

and $s(m) \in\{0,1\}^{\lambda}$ is meant as the output of SecretBits $(\cdot, r)$.

Finally, to get rid of the trusted dealer, in Subsection 4.3 we describe a trustless protocol that performs the setup instead of a trusted dealer. The crucial difference is that the keys are not generated by one entity, but jointly by all the nodes. Moreover, in the asynchronous version of the setup, every honest node $i$ learns the verification key $V K$, some key $t k_{i}$ and a set of "share dealers" $T \subseteq[N]$ of size $2 f+1$, such that every node $P_{i}$ with $i \in T$ has a correct tossing key $t k_{i}$. This, while being slightly weaker than the outcome of the setup of trusted dealer, still allows to implement SecretBits as demostrated in the below lemma.

LemmA 4.1 (Secret Bits). The above scheme in both versions (with and without trusted dealer) correctly implements SecretBits.

We refer the reader to Subsection E for a proof.

\subsection{Randomness Beacon with Trustless Setup}

In Section 4.2 we have discussed how to implement a Randomness Beacon based on a trusted dealer. Here, we devise a version of this protocol that has all the benefits of the previous one and at the same time is completely trustless. For the sake of clarity, we first provide a high level perspective of the new ideas and how do they combine to give a trustless Randomness Beacon, next we provide a short summary of the protocol in the form of a very informal pseudocode, and finally we fill in the gaps by giving details on how the specific parts are implemented and glued together.

Key Boxes. Since no trusted dealer is available, perhaps the most natural idea is to let all the nodes serve as dealers simultaneously. More precisely we let every node emit (via RBC, i.e., by placing it as data in a unit) a tossing Key Box that is constructed by a node $k$ (acting as a dealer) as follows:
- as in GenerateKeys(), sample a random polynomial

$$
A_{k}(x)=\sum_{j=0}^{f} a_{k, j} x^{j} \in \mathbb{Z}_{q}[x]
$$

of degree $f$,
- compute a commitment to $A_{k}$ as

$$
C_{k}=\left(g^{a_{k, 0}}, g^{a_{k, 1}}, \ldots, g^{a_{k, f}}\right)
$$
- define the tossing keys $T K_{k}$ and verification keys $V K_{k}$

$$
\begin{array}{ll}
t k_{k, i}:=A_{k}(i) & \text { for } i=1,2, \ldots, N \\
v k_{k, i}:=g^{t k_{i}} & \text { for } i=1,2, \ldots, N
\end{array}
$$

Note that in particular each verification key $v k_{k, i}$ for $i \in$ $[N]$ can be computed from the commitment $C_{k}$ as $v k_{k, i}=$ $\prod_{j=0}^{f} C_{k, j}^{i^{j}}$.
- encrypt the tossing keys for every node $i$ using the dedicated public key ${ }^{18} p k_{k \rightarrow i}$ as

$$
e_{k, i}:=\operatorname{Enc}_{k \rightarrow i}\left(t k_{k, i}\right)
$$

and let $E_{k}:=\left(e_{k, 1}, e_{k, 2}, \ldots, e_{k, N}\right)$.
- the $k$ th key box is defined as $K B_{k}=\left(C_{k}, E_{k}\right)$

In our protocol, every node $\mathcal{P}_{k}$ generates its own key box $K B_{k}$ and places $\left(C_{k}, E_{k}\right)$ in his unit of round 0 . We define the $k$ th key set to be $K S_{k}=\left(V K_{k}, T K_{k}\right)$ and note that given the key box $K B_{k}=\left(C_{k}, E_{k}\right)$, every node can reconstruct $V K_{k}$ and moreover, every node $i$ can decrypt his tossing key $t k_{k, i}$ from the encrypted part $E_{k}$, but is not able to extract the remaining keys. The encypted tossing keys $E_{k}$ can be seen as a certain way of emulating "authenticated channels".

Verifying Key Sets. Since at least $2 / 3$ of nodes are honest, we also know that $2 / 3$ of all the key sets are safe to use, because they were produced by honest nodes who properly generated the key sets and the corresponding key boxes an erased the underlying polynomial (and thus all the tossing keys). Unfortunately, it is not possible to figure out which nodes cheated in this process (and kept the tossing keys that were supposed to be erased).

What is even harder to check, is whether a certain publicly known key box $K B_{k}$ was generated according to the instructions above. Indeed, as a node $\mathcal{P}_{i}$ we have access only to our tossing key $t k_{k, i}$ and while we can verify that this key agrees with the verification key (check if $g^{t k_{k, i}}=v k_{k, i}$ ), we cannot do that for the remaining keys that are held by other nodes. The only way to perform verification of the key sets is to do that in collaboration with other nodes. Thus, in the protocol, there is a round at which
\footnotetext{
${ }^{18}$ We assume that as part of PKI setup each node $i$ is given exactly $N$ different key pairs for encryption: $\left(s k_{k \rightarrow i}, p k_{k \rightarrow i}\right)$ for $k \in[N]$. The key $p k_{k \rightarrow i}$ is meant to be used by the $k$ th node to encrypt a message whose recipient is $i$ (denoted as Enc ${ }_{k \rightarrow i}(\cdot)$ ). This setup is merely for simplicity of arguments - one could instead have one key per node if using verifiable encryption or double encryption with labels.
}
every node "votes" for correctness of all the key sets it has seen so far. As will be explained in detail later, these votes cannot be "faked" in the following sense: if a node $\mathcal{P}_{i}$ votes on a key set $K S_{k}$ being incorrect, it needs to provide a proof that its key $t k_{k, i}$ (decrypted from $e_{k, i}$ ) is invalid, which cannot be done if $\mathcal{P}_{k}$ is an honest dealer Thus consequently, dishonest nodes cannot deceive the others that a valid key set is incorrect.

Choosing Trusted Key Sets. At a later round these votes are collected and summarized locally by every node $\mathcal{P}_{i}$ and a trusted set of key sets $T_{i} \subseteq[N]$ is determined. Intuitively, $T_{i}$ is the set of indices $k$ of key sets such that:
- the node $\mathcal{P}_{i}$ is familiar with $K B_{k}$,
- the node $\mathcal{P}_{i}$ has seen enough votes on $K S_{k}$ correctness that it is certain that generating secrets from $K S_{k}$ will be successful,

The second condition is determined based solely on the ch-DAG structure below the $i$ th node unit at a particular round. What will be soon important is that, even though the sets $T_{i}$ do not necessarily coincide for different $i$, it is guaranteed that $\left|T_{i}\right| \geqslant f+1$ for every $i$, and thus crucially at least one honest key set is included in each of them.

Combining Tosses. We note that once the set $T_{i}$ is determined for some index $i \in[N]$, this set essentially defines a global common source of randomness that cannot be biased by an adversary. Indeed, suppose we would like to extract the random bits corresponding to nonce $m$. First, in a given round, say $r$, every node should include its share for nonce $m$ corresponding to every key set that it voted as being correct. In the next round, it is guaranteed that the shares included in round $r$ are enough to recover the random secret $\sigma_{m, k}=m^{A_{k}(0)} \in G$ (the threshold signature of $m$ generated using key set $K S_{k}$ ) for every $k \in T_{i}$. Since up to $f$ out of these secrets might be generated by the adversary, we simply take

$$
\tau_{m, i}:=\prod_{k \in T_{i}} \sigma_{m, k}=m^{\sum_{k \in T_{i}} A_{k}(0)} \in G
$$

to obtain a uniformly random element of $G$ and thus (by hashing it) a random bitstring of length $\lambda$ corresponding to node $\mathcal{P}_{i}$, resulting from nonce $m$. From now on we refer to the $i$ th such source of randomness (i.e., corresponding to $\mathcal{P}_{i}$ ) as MultiCoin ${ }_{i}$.

Agreeing on Common MultiCoin. So far we have said that every node $\mathcal{P}_{i}$ defines locally its strong source of randomness MultiCoin ${ }_{i}$ Note however that paradoxically, this abundance of randomness sources is actually problematic: which one shall be used by all the nodes to have a "truly common" source of randomness? This is nothing other than a problem of "selecting a head", i.e., choosing one unit from a round - a problem that we already solved in Section 3! At this point however, an attentive reader is likely to object, as the algorithm from Section 3 only works provided a common source of randomness. Therefore, the argument seems to be cyclic as we are trying to construct such a source of randomness from the algorithm from Section 3. Indeed, great care is required here: as explained in Section 3, all what is required for the ChooseHead protocol to work is a primitive SecretBits $(i, r)$ that is supposed to inject a secret of $\lambda$ bits at the $r$ th round of the protocol. Not going too deep into details, we can appropriately implement such a method by employing the MultiCoins we have at our disposal. This part, along with the construction of MultiCoins, constitutes the technical core of the whole protocol.

Combining Key Sets. Finally, once the head is chosen to be $l \in[N]$, from now on one could use MultiCoin ${ }_{l}$ as the common source of randomness. If one does not care about savings in communication and computational complexity, then the construction is over. Otherwise, observe that tossing the MultiCoin ${ }_{l}$ at a nonce $m$ requires in the worst case $N$ shares from every single node. This is sub-optimal, and here is a simple way to reduce it to just 1 share per node. Recall that every Key Set $K S_{k}$ that contributes to MultiCoin ${ }_{l}$ is generated from a polynomial $A_{k} \in \mathbb{Z}_{q}[x]$ of degree $f$. It is not hard to see that by simple algebraic manipulations one can combine the tossing keys and all the verification keys of all key sets $K S_{k}$ for $k \in T_{l}$ so that the resulting Key Set corresponds to the sum of these polynomials

$$
A(x):=\left(\sum_{k \in T_{l}} A_{k}(x)\right) \in \mathbb{Z}_{q}[x]
$$

This gives a source of randomness that requires one share per nonce from every node; note that since the set $T_{l}$ contains at least one honest node, the polynomial $A$ can be considered random.

Protocol Sketch. We are now ready to provide an informal sketch of the Setup protocol and Toss protocol, see the ABFT - Beacon box below. The content of the box is mostly a succinct summary of Section 4.3. What might be unclear at this point is the condition of the if statement in the Toss function. It states that the tossing key $t k_{i}$ is supposed to be correct in order to produce a share: indeed it can happen that one of the key boxes that is part of the MultiCoin ${ }_{l}$ does not provide a correct tossing key for the $i$ th node, in which case the combined tossing key $t k_{i}$ cannot be correct either. This however is not a problem, as the protocol still guarantees that at least $2 f+1$ nodes hold correct combined tossing keys, and thus there will be always enough shares at our disposal to run ExtractBits.

\subsection{Details of the Setup}

We provide some more details regarding several components of the Setup phase of ABFT - Beacon that were treated informally in the previous subsection.

Voting on Key Sets. Just before creating the unit at round 3 the $i$ th node is supposed to inspect all the key boxes that are present in its current copy of the ch-DAG. Suppose $K B_{k}$ for some $k \in[N]$ is one of such key sets. The only piece of information about $K B_{k}$ that is known to $\mathcal{P}_{i}$ but hidden from the remaining nodes is its tossing key. Node $\mathcal{P}_{i}$ recovers this key by decrypting it using its secret key (dedicated for $k$ ) $s k_{k \rightarrow i}$

$$
t k_{k, i} \leftarrow \operatorname{Dec}_{k \rightarrow i}\left(e_{k, i}\right)
$$

Now, if node $\mathcal{P}_{k}$ is dishonest, it might have included an incorrect tossing key, to check correctness, the $i$ th node verifies whether

$$
g^{t k_{k, i}} \stackrel{?}{=} v k_{k, i}
$$

where $g$ is the fixed generator of the group $G$ we are working over. The $i$ th node includes the following piece of information in its round-3 unit

$$
\operatorname{VerKey}\left(K B_{k}, i\right)= \begin{cases}1 & \text { if } t k_{k, i} \text { correct } \\ \operatorname{Dec}_{k \rightarrow i}\left(e_{k, i}\right) & \text { oth. }\end{cases}
$$

```
ABFT-Beacon:
    1 Setup():
        /* Instructions for node $\mathcal{P}_{i}$. */
        Initialize growing the ch-DAG $\mathcal{D}$
        In the data field of your units include (as specified in
        Section 4.4):
            At round 0: a key box $K B_{i}$,
            At round 3: votes regarding correctness of key boxes
            present in $\mathcal{D}$,
            At rounds $\geqslant 6$ : shares necessary to extract
            randomness from SecretBits.
        Run ChooseHead to determine the head unit at round 6
        and let $l$ be its creator.
        Combine the key sets $\left\{K S_{j}: j \in T_{l}\right\}$ and let $\left(t k_{i}, V K\right)$ be
        the corresponding tossing key and verification keys.
    $9 \operatorname{Toss}(m):$
        /* Code for node $\mathcal{P}_{i}$. */
        if $t k_{i}$ is correct then
            $s_{i} \leftarrow$ CreateShare $\left(m, t k_{i}\right)$
            multicast $\left(s_{i}, i\right)$
        wait until receiving a set of $f+1$ valid shares $S$
        /* validity is checked using VerifyShare() */
        output $\sigma_{m}:=$ ExtractBits $(m, S, V K)$
```

Note that if a node $\mathcal{P}_{i}$ votes that $K B_{k}$ is incorrect (by including the bottom option of VerKey in its unit) it cannot lie, since other nodes can verify whether the plaintext it provided agrees with the ciphertext in $K B_{k}$ (as the encryption scheme is assumed to be deterministic) and if that is not the case, they treat a unit with such a vote as invalid (and thus not include it in their ch-DAG) Thus, consequently, the only way dishonest nodes could cheat here is by providing positive votes for incorrect key boxes. This can not harm honest nodes, because by positively verifying some key set a node declares that from now on it will be providing shares for this particular key set whenever requested. If in reality the corresponding tossing key is not available to this node, it will not be able to create such shares and hence all its units will be henceforth considered invalid.

One important property this scheme has is that it is safe for an honest recipient $\mathcal{P}_{i}$ of $e_{k, i}$ to reveal the plaintext decryption of $e_{k, i}$ in case it is not (as expected) the tossing key $t k_{k, i}$ - indeed if $\mathcal{P}_{k}$ is dishonest then either he actually encrypted some data $d$ in $e_{k, i}$ in which case he learns nothing new (because $d$ is revealed) or otherwise he obtained $e_{k, i}$ by some other means, in which case $\operatorname{Dec}_{k \rightarrow i}\left(e_{k, i}\right)$ is a random string, because no honest node ever creates a ciphertext encrypted with $p_{k \rightarrow i}$ and we assume that the encryption scheme is semantically secure. Note also that if instead of having $N$ key pairs per node we used a similar scheme with every node having just one key pair, then the adversary could reveal some tossing keys of honest nodes through the following attack: the adversary copies an honest node's (say $j$ th) ciphertext $e_{j, i}$ and includes it as $e_{k, i}$ in which case $\mathcal{P}_{i}$ is forced to reveal $\operatorname{Dec}_{i}\left(e_{j, i}\right)=t k_{j, i}$ which should have remained secret! This is the reason why we need dedicated key pairs for every pair of nodes.
Forming MultiCoins. The unit $V:=U[i ; 6]$ created by the $i$ th node in round 6 defines a set $T_{i} \subseteq[N]$ as follows: $k \in N$ is considered an element of $T_{i}$ if and only if all the three conditions below are met

(1) $U[k ; 0] \leqslant V$,

(2) For every $j \in[N]$ such that $U[j ; 3] \leqslant V$ it holds that

$$
\operatorname{VerKey}\left(K B_{k}, j\right)=1
$$

At this point it is important to note that every node that has $U[i ; 6]$ in its copy of the ch-DAG can compute $T_{i}$ as all the conditions above can be checked deterministically given only the ch-DAG.

SecretBits via MultiCoins. Recall that to implement CommonVote and GeneratePermutation it suffices to implement a more general primitive SecretBits $(i, r)$ whose purpose is to inject a secret at round $r$ that can be recovered by every node in round $r+1$ but cannot be recovered by the adversary till at least one honest node has created a unit of round $r$.

The technical subtlety that becomes crucial here is that the SecretBits $(i, r)$ is only called for $r \geqslant 9$ and only by nodes that have $U[i ; 6]$ in their local ch-DAG (see Lemma E.3). More specifically, this allows us to implement $\operatorname{SecretBits}(i, \cdot)$ through MultiCoin ${ }_{i}$. The rationale behind doing so is that every node that sees $U[i ; 6]$ can also see all the Key Sets that comprise the $i$ th MultiCoin and thus consequently it "knows" what MultiCoin ${ }_{i}$ is $^{19}$.

Suppose now that we would like to inject a secret at round $r$ for index $i \in[N]$. Define a nonce $m:=$ " $\mathrm{i} \| \mathrm{r}$ " and request all the nodes $\mathcal{P}_{k}$ such that $U[i ; 6] \leqslant U[k, r]$ to include in $U[k, r]$ a share for the nonce $m$ for every Key Set $K S^{j}$ such that $j \in T_{i}$. In addition, if $\mathcal{P}_{k}$ voted that $K S_{j}$ is incorrect in round 3 , or $\mathcal{P}_{k}$ did not vote for $K B_{j}$ at all (since $K B_{j}$ was not yet available to him at round 3) then $\mathcal{P}_{k}$ is not obligated to include a share (note that its unit $U[k, 3]$ that is below $U[k, r]$ contains evidence that its key was incorrect).

As we then show in Section 4.4, given any unit $U \in \mathcal{D}$ of round $r+1$ one can then extract the value of MultiCoin ${ }_{i}$ from the shares present in round- $r$ units in $\mathcal{D}$. Thus, consequently, the value of SecretBits $(i, r)$ in a ch-DAG $\mathcal{D}$ is available whenever any unit in $\mathcal{D}$ has round $\geqslant r+1$, hence we arrive at

LemmA 4.2 (Secret Bits from Multicoins). The above defined scheme based on MultiCoins correctly implements SecretBits.

The proof of the above lemma is provided in Section E.

\section{ACKNOWLEDGEMENTS}

First of all, authors would like to thank Matthew Niemerg for introducing us to the topic, constant support, and countless hours spent on valuable discussions. Additionally, we would like to show our gratitude to Michał Handzlik, Tomasz Kisielewki, Maciej Gawron, and Łukasz Lachowski, for reading the paper, proposing changes that improved its consistency and readability, and discussions that helped us to make the paper easier to understand.

This research was funded by the Aleph Zero Foundation.
\footnotetext{
${ }^{19}$ We epmhasize that using a fixed MultiCoin, for instance MultiCoin ${ }_{1}$ would not be correct here, as there is no guarantee the 1st node has delivered its unit at round 6. More generally, it is crucial that for different units $U_{0}$ we allow to use different MultiCoins, otherwise we would have solved Byzantine Consensus without randomness, which is impossible by the FLP Theorem [24].
}

\section{REFERENCES}

[1] Michael Abd-El-Malek, Gregory R. Ganger, Garth R. Goodson, Michael K. Reiter and Jay J. Wylie. 2005. Fault-scalable Byzantine fault-tolerant services. In Proceed ings of the 20th ACM Symposium on Operating Systems Principles 2005, SOSP 2005, Brighton, UK, October 23-26, 2005. 59-74. https://doi.org/10.1145/1095810.1095817

[2] Ittai Abraham, Guy Gueta, Dahlia Malkhi, Lorenzo Alvisi, Ramakrishna Kotla, and Jean-Philippe Martin. 2017. Revisiting Fast Practical Byzantine Fault Toler ance. CoRR abs/1712.01367 (2017). arXiv:1712.01367 http://arxiv.org/abs/1712 01367

[3] Ittai Abraham, Dahlia Malkhi, and Alexander Spiegelman. 2019. Asymptotically Optimal Validated Asynchronous Byzantine Agreement. In Proceedings of the 2019 ACM Symposium on Principles of Distributed Computing, PODC 2019, Toronto ON, Canada, July 29 - August 2, 2019. 337-346. https://doi.org/10.1145/3293611. 3331612

[4] Leemon Baird. 2016. The swirlds hashgraph consensus algorithm: Fair, fast, byzantine fault tolerance. Swirlds Tech Reports SWIRLDS-TR-2016-01, Tech. Rep (2016)

[5] Michael Ben-Or and Ran El-Yaniv. 2003. Resilient-optimal interactive consistency in constant time. Distributed Computing 16, 4 (2003), 249-262. https://doi.org $10.1007 /$ s00446-002-0083-3

[6] Alexandra Boldyreva. 2003. Threshold Signatures, Multisignatures and Blind Signatures Based on the Gap-Diffie-Hellman-Group Signature Scheme. In Public Key Cryptography - PKC 2003, 6th International Workshop on Theory and Practice in Public Key Cryptography, Miami, FL, USA, January 6-8, 2003, Proceedings. 31-46. https://doi.org/10.1007/3-540-36288-6 3

[7] Dan Boneh, Joseph Bonneau, Benedikt Bünz, and Ben Fisch. 2018. Verifiable Delay Functions. In Advances in Cryptology - CRYPTO 2018 - 38th Annual International Cryptology Conference, Santa Barbara, CA, USA, August 19-23, 2018, Proceedings, Part I. 757-788. https://doi.org/10.1007/978-3-319-96884-1_25

[8] Dan Boneh, Ben Lynn, and Hovav Shacham. 2004. Short Signatures from the Weil Pairing. F. Cryptology 17, 4 (2004), 297-319. https://doi.org/10.1007 s00145-004-0314-9

[9] Gabriel Bracha. 1987. Asynchronous Byzantine Agreement Protocols. Inf. Comput. 75, 2 (1987), 130-143. https://doi.org/10.1016/0890-5401(87)90054-X

[10] Gabriel Bracha and Sam Toueg. 1983. Resilient Consensus Protocols. In Proceedings of the Second Annual ACM SIGACT-SIGOPS Symposium on Principles of Distributed Computing, Montreal, Quebec, Canada, August 17-19, 1983. 12-26 https://doi.org/10.1145/800221.806706

[11] Ethan Buchman, Jae Kwon, and Zarko Milosevic. 2018. The latest gossip on BFT consensus. CoRR abs/1807.04938 (2018). arXiv:1807.04938 http://arxiv.org/abs 1807.04938

[12] Vitalik Buterin and Virgil Griffith. 2017. Casper the Friendly Finality Gadget. CoRR abs/1710.09437 (2017). arXiv:1710.09437 http://arxiv.org/abs/1710.09437

[13] Christian Cachin, Rachid Guerraoui, and Luís E. T. Rodrigues. 2011. Introduction to Reliable and Secure Distributed Programming (2. ed.). Springer. https://doi org/10.1007/978-3-642-15260-3

[14] Christian Cachin, Klaus Kursawe, Anna Lysyanskaya, and Reto Strobl. 2002 Asynchronous verifiable secret sharing and proactive cryptosystems. In Proceedings of the 9th ACM Conference on Computer and Communications Secu rity, CCS 2002, Washington, DC, USA, November 18-22, 2002. 88-97. https $/ /$ doi.org/10.1145/586110.586124

[15] Christian Cachin, Klaus Kursawe, Frank Petzold, and Victor Shoup. 2001. Secure and Efficient Asynchronous Broadcast Protocols. In Advances in Cryptology CRYPTO 2001, 21st Annual International Cryptology Conference, Santa Barbara, California, USA, August 19-23, 2001, Proceedings. 524-541. https://doi.org/10 1007/3-540-44647-8_31

[16] Christian Cachin, Klaus Kursawe, and Victor Shoup. 2005. Random Ora cles in Constantinople: Practical Asynchronous Byzantine Agreement Using Cryptography. 7. Cryptology 18, 3 (2005), 219-246. https://doi.org/10.1007 s00145-005-0318-0

[17] Christian Cachin and Stefano Tessaro. 2005. Asynchronous verifiable information dispersal. In 24th IEEE Symposium on Reliable Distributed Systems (SRDS'05). IEEE, 191-201

[18] Christian Cachin and Marko Vukolic. 2017. Blockchain Consensus Protocols in the Wild. CoRR abs/1707.01873 (2017). arXiv:1707.01873 http://arxiv.org/abs 1707.01873

[19] Ran Canetti and Tal Rabin. 1993. Fast asynchronous Byzantine agreement with optimal resilience. In Proceedings of the Twenty-Fifth Annual ACM Symposium on Theory of Computing, May 16-18, 1993, San Diego, CA, USA. 42-51. https //doi.org/10.1145/167088.167105

[20] Ignacio Cascudo and Bernardo David. 2017. SCRAPE: Scalable Randomnes Attested by Public Entities. In Applied Cryptography and Network Security 15th International Conference, ACNS 2017, Kanazawa, Japan, July 10-12, 2017 Proceedings. 537-556. https://doi.org/10.1007/978-3-319-61204-1_27

[21] Miguel Castro and Barbara Liskov. 1999. Practical Byzantine Fault Tolerance. In Proceedings of the Third USENIX Symposium on Operating Systems Design and Implementation (OSDI), New Orleans, Louisiana, USA, February 22-25, 1999.
173-186. https://dl.acm.org/citation.cfm?id=296824

[22] Miguel Correia, Nuno Ferreira Neves, and Paulo Veríssimo. 2006. From Consensus to Atomic Broadcast: Time-Free Byzantine-Resistant Protocols without Signatures. Comput. 7. 49,1 (2006), 82-96. https://doi.org/10.1093/comjnl/bxh145

[23] George Danezis and David Hrycyszyn. 2018. Blockmania: from Block DAGs to Consensus. arXiv preprint arXiv:1809.01620 (2018)

[24] Michael J. Fischer, Nancy A. Lynch, and Mike Paterson. 1985. Impossibility of Distributed Consensus with One Faulty Process. 7. ACM 32, 2 (1985), 374-382. https://doi.org/10.1145/3149.214121

[25] Rosario Gennaro, Stanislaw Jarecki, Hugo Krawczyk, and Tal Rabin. 2003. Secure Applications of Pedersen's Distributed Key Generation Protocol. In Topics in Cryptology - CT-RSA 2003, The Cryptographers' Track at the RSA Conference 2003, San Francisco, CA, USA, April 13-17, 2003, Proceedings. 373-390. https: //doi.org/10.1007/3-540-36563-X_26

[26] Rosario Gennaro, Stanislaw Jarecki, Hugo Krawczyk, and Tal Rabin. 2007. Secure Distributed Key Generation for Discrete-Log Based Cryptosystems. 7. Cryptology 20, 1 (2007), 51-83. https://doi.org/10.1007/s00145-006-0347-3

[27] Vassos Hadzilacos and Sam Toueg. 1994. A modular approach to fault-tolerant broadcasts and related problems. Technical Report. Cornell University.

[28] Timo Hanke, Mahnush Movahedi, and Dominic Williams. 2018. Dfinity technology overview series, consensus system. arXiv preprint arXiv:1805.04548 (2018).

[29] Aniket Kate, Yizhou Huang, and Ian Goldberg. 2012. Distributed Key Generation in the Wild. IACR Cryptology ePrint Archive 2012 (2012), 377. http://eprint.iacr. org $/ 2012 / 377$

[30] Aggelos Kiayias, Alexander Russell, Bernardo David, and Roman Oliynykov. 2017. Ouroboros: A Provably Secure Proof-of-Stake Blockchain Protocol. In Advances in Cryptology - CRYPTO 2017 - 37th Annual International Cryptology Conference, Santa Barbara, CA, USA, August 20-24, 2017, Proceedings, Part I. 357388. https://doi.org/10.1007/978-3-319-63688-7_12

[31] Ramakrishna Kotla, Lorenzo Alvisi, Michael Dahlin, Allen Clement, and Edmund L. Wong. 2009. Zyzzyva: Speculative Byzantine fault tolerance. ACM Trans. Comput. Syst. 27, 4 (2009), 7:1-7:39. https://doi.org/10.1145/1658357.1658358

[32] Jae Kwon and Ethan Buchman. [n.d.]. A Network of Distributed Ledgers. ([n.d.]). https://cosmos.network/cosmos-whitepaper.pdf

[33] Leslie Lamport. 1978. Time, Clocks, and the Ordering of Events in a Distributed System. Commun. ACM 21, 7 (1978), 558-565. https://doi.org/10.1145/359545. 359563

[34] Arjen K. Lenstra and Benjamin Wesolowski. 2017. Trustworthy public randomness with sloth, unicorn, and trx. IFACT 3, 4 (2017), 330-343. https: //doi.org/10.1504/IJACT.2017.10010315

[35] Andrew Miller, Yu Xia, Kyle Croman, Elaine Shi, and Dawn Song. 2016. The Honey Badger of BFT Protocols. In Proceedings of the 2016 ACM SIGSAC Conference on Computer and Communications Security, Vienna, Austria, October 24-28, 2016. 31-42. https://doi.org/10.1145/2976749.2978399

[36] Zarko Milosevic, Martin Hutle, and André Schiper. 2011. On the Reduction of Atomic Broadcast to Consensus with Byzantine Faults. In 30th IEEE Symposium on Reliable Distributed Systems (SRDS 2011), Madrid, Spain, October 4-7, 2011. 235-244. https://doi.org/10.1109/SRDS.2011.36

[37] Louise E. Moser and P. M. Melliar-Smith. 1999. Byzantine-Resistant Total Ordering Algorithms. Inf. Comput. 150, 1 (1999), 75-111. https://doi.org/10.1006/inco. 1998.2770

[38] Achour Mostéfaoui, Hamouma Moumen, and Michel Raynal. 2015. SignatureFree Asynchronous Binary Byzantine Consensus with $\mathrm{t}<\mathrm{n} / 3, \mathrm{O}(\mathrm{n} 2)$ Messages, and $\mathrm{O}(1)$ Expected Time. 7. ACM 62, 4 (2015), 31:1-31:21. https://doi.org/10. $1145 / 2785953$

[39] Satoshi Nakamoto. 2008. Bitcoin: A peer-to-peer electronic cash system. (2008).

[40] Rafael Pass and Elaine Shi. 2017. Hybrid Consensus: Efficient Consensus in the Permissionless Model. In 31st International Symposium on Distributed Computing, DISC 2017, October 16-20, 2017, Vienna, Austria. 39:1-39:16. https://doi.org/10. 4230/LIPIcs.DISC.2017.39

[41] Torben P. Pedersen. 1991. A Threshold Cryptosystem without a Trusted Party (Extended Abstract). In Advances in Cryptology-EUROCRYPT '91, Workshop on the Theory and Application of of Cryptographic Techniques, Brighton, UK, April 8-11, 1991, Proceedings. 522-526. https://doi.org/10.1007/3-540-46416-6_47

[42] Krzysztof Pietrzak. 2019. Simple Verifiable Delay Functions. In 10th Innovations in Theoretical Computer Science Conference, ITCS 2019, January 10-12, 2019, San Diego, California, USA. 60:1-60:15. https://doi.org/10.4230/LIPIcs.ITCS. 2019.60

[43] David Pointcheval and Jacques Stern. 1996. Security Proofs for Signature Schemes. In Advances in Cryptology - EUROCRYPT '96, International Conference on the Theory and Application of Cryptographic Techniques, Saragossa, Spain, May 12-16, 1996, Proceeding. 387-398. https://doi.org/10.1007/3-540-68339-9_33

[44] Serguei Popov. 2016. The tangle. cit. on (2016), 131

[45] Adi Shamir. 1979. How to Share a Secret. Commun. ACM 22, 11 (1979), 612-613. https://doi.org/10.1145/359168.359176

[46] Ewa Syta, Philipp Jovanovic, Eleftherios Kokoris-Kogias, Nicolas Gailly, Linus Gasser, Ismail Khoffi, Michael J. Fischer, and Bryan Ford. 2017. Scalable BiasResistant Distributed Randomness. In 2017 IEEE Symposium on Security and Privacy, SP 2017, San Jose, CA, USA, May 22-26, 2017. 444-460. https://doi.org/10.

\section{9/SP. 2017.45}

[47] Benjamin Wesolowski. 2019. Efficient Verifiable Delay Functions. In Advances in Cryptology - EUROCRYPT 2019 - 38th Annual International Conference on the Theory and Applications of Cryptographic Techniques, Darmstadt, Germany, May 19-23, 2019, Proceedings, Part III. 379-407. https://doi.org/10.1007/978-3-030-17659-4 13

\section{A PRACTICAL CONSIDERATIONS}

The protocol that we described in the main body of the paper has excellent theoretical properties and achieves optimal asymptotic guarantees, however in the original form might not be viable for practical implementation. The high level reason for that is that it was designed to operate in the harshest possible adversarial setting (i.e. the adversary controlling $f$ out of $3 f+1$ nodes and being able to arbitrarily delay messages) and it was not optimized for the "optimistic case". This means intuitively that as of now, the protocol can withstand arbitrarily oppressive "attacks" of the adversary, but does not get any faster when no attacks take place.

In this section we present several adjustments to the protocol that allow us to make it significantly faster in the optimistic case without compromising its strong security guarantees. As a conse quence, we obtain a version of the protocol that is well suited for practical implementation - we show in particular (see Section C.3) that under partial synchrony it matches the optimistic 3-round validation delay of PBFT and Tendermint [11, 21].

Below, we list the main sources of practical inefficiency of the protocol:

(1) The reliance on RBC: each DAG-round effectively takes 3 async-rounds, so in practice the protocol requires more rounds than simple synchronous or partially synchronous protocols such as PBFT. Furthermore, performing $\Omega(N)$ instances of RBC simultaneusly by a node, forces it to send $\Omega\left(N^{2}\right)$ distinct messages per round, which might not be feasible.

(2) The worst-case assumption that the adversary can pretty much arbitrarily manipulate the structure of the ch-DAG forces the randomness to be revealed with large delay and causes inefficiency.

(3) The total size of metadata (total size of units, ignoring transactions) produced in one round by all the nodes is quite large: $\Omega\left(N^{2} \lambda\right)$ bits, because each round- $r$ unit contains hashes of $\Omega(N)$ units from round $r-1$.

We show how to solve these three issues in Sections A.1, A. 2 and A. 3 respectively.

\section{A. 1 From RBC to Multicast and Random Gossip}

As a practical solution, we propose to use a combination of multicast and random gossip in place of RBC to share the ch-DAG between nodes. More specifically, the following Quick-DAG-Grow algorithm is meant to replace the DAG-Grow from the main body (based on RBC).

At this point we emphasize that while we believe that our proposal yields a truly efficient algorithm, it is rather cumbersome to formally reason about the communication complexity of such a solution (specifically about the gossip part) and we do not attempt

```
Quick-DAG-Grow $(\mathcal{D})$ :
    CreateUnit(data):
        for $r=0,1,2, \ldots$ do
            if $r>0$ then
                wait until $|\{U \in \mathcal{D}: \mathrm{R}(U)=r-1\}| \geqslant 2 f+1$
            $P \leftarrow\left\{\right.$ maximal $\mathcal{P}_{i}$ 's unit of round $<r$ in $\left.\mathcal{D}: \mathcal{P}_{i} \in \mathcal{P}\right\}$
            create a new unit $U$ with $P$ as parents
            include data in $U$
            add $U$ to $\mathcal{D}$
            multicast $U$
    ReceiveUnits:
        loop forever
            upon receiving a unit $U$ do
                if $U$ is correct then add $U$ to buffer $\mathcal{B}$
                while exists $V \in \mathcal{B}$ whose all parents are in $\mathcal{D}$ do
                    move $V$ from $\mathcal{B}$ to $\mathcal{D}$
    Gossip:
        /* Run by node $i \quad$ */
        loop forever
            $j \leftarrow$ randomly sampled node
            send $j$ concise info about $\mathcal{D}_{i}$
            receive all units in $\mathcal{D}_{j} \backslash \mathcal{D}_{i}$
            send all units in $\mathcal{D}_{i} \backslash \mathcal{D}_{j}$
```

to provide a formal treatment. Instead, in a practical implementation one needs to include a set of rules that make it impossible for malicious nodes to request the same data over and over again (in order to slow down honest nodes). To analyze further effects of switching from RBC to multicast + gossip, recall that RBC in the original protocol allows us to solve the following two problems

(1) the data availability problem, as a given piece of data is locally output by an honest node in RBC only when it is guaranteed to be eventually output by every other honest node,

(2) the problem of forks, since only one version of each unit may be locally output by an honest node.

The data availability problem, we discuss in Section A.1.1. Regarding forks: their existence in the ch-DAG does not pose a threat to the theoretical properties of our consensus protocol. Indeed, in Section C. 2 we show that the protocol is guaranteed to reach consensus even in the presence of forks in ch-DAG. The only (yet serious) problem with forks is that if malicious nodes produce a lot of them and all these forks need to be processed by honest nodes, this can cause crashes due to lack of resources, i.e., RAM or disc space. To counter this issue we add an auxiliary mechanism to the protocol whose purpose is bounding the number of possible forks (see Section A.1.2). Additionally, we show that without a very specific set of precautions, such an attack is rather simple to conduct and, to best of our knowledge, affects most of the currently proposed DAG-based protocols. We formalize it as a Fork Bomb attack in section $H$.

A.1.1 Ensuring data availability via gossip. Suppose for a moment that we have multicast as the only mechanism for sending
units to other nodes (i.e., the creator multicasts his newly created unit to all the nodes). While this is clearly the fastest way to disseminate information through a network, it is certainly prone to adversarial attacks on data availability. This is a consequence of the fact that only the creator of this unit would then share it with other nodes. Hence, if the creator is malicious, it may introduce intentional inconsistencies in local copies of ch-DAGs stored by honest nodes by sending newly created units only to some part of the network or even sending different variants to different nodes. To see that such an attack can not be trivially countered, let us explore possible scenarios, depending on the particular rules of unit creation:
- If node $i$ is allowed to choose a unit $U$ as a parent for its newly created unit $V$ despite the fact that it does not know the whole "context" of $U$ (i.e., it does not have all of $U$ 's parents in its local copy of ch-DAG), then we risk that the functions $\operatorname{Vote}(V, \cdot)$ and UnitDecide $(V, \cdot)$ will never succeed to terminate with a non $-\perp$ verdict. Hence, node $i$ may never be able to make a decision about $V$.
- If on the other hand $i$ is not allowed to choose $U$ as a parent in such a case, then the growth of ch-DAG may stall. Indeed, another honest node might have created $U$ with a parent $W$ that was available to him but not $i$ (because $W$ was created by a malicious node).

Hence it seems that to counter such intentional or unintentional (resulting for example from temporary network failures) inconsistencies, there has to be a mechanism in place that allows nodes to exchange information about units they did not produce. To this end in the Quick-DAG-Grow protocol each node regularly reconciliates its local version of the ch-DAG with randomly chosen peers, in a gossip-like fashion. Since each two honest nodes sync with each other infinitely often (with probability 1 ), this guarantees data availability.

A.1.2 Bounding the number of forks. We introduce a mechanism that bounds the number of possible forks to $N$ variants (or rather "branches") per node.

Note that if a (dishonest) node starts broadcasting a forked unit, this is quickly noticed by all honest nodes, and the forker can be banned after such an incident. Note however that we cannot reliably "rollback" the ch-DAG to a version before the fork has happened, as two different honest nodes might have already build their units on two different fork branches. Still, since units are signed by their creators, the forker can be proven to be malicious and punished accordingly, for example by slashing its stake. This means that the situation when a malicious node just creates a small number of variants of a unit is not really dangerous - it will be detected quite soon and this node will be banned forever.

What might be potentially dangerous though is when a group of malicious nodes collaborate to build a multi-level "Fork Bomb" (Section H) composed of a huge number of units and the honest nodes are forced to download them all, which could lead to crashes This is the main motivation behind introducing the alert protocol.

We note that sending a unit via RBC can be thought of as "committing" to a single variant of a unit. In the absence of RBC the simplest attack of an adversary would be to send a different variant of a unit to every node. Since just after receiving this unit, honest nodes cannot be aware of all the different variants, each of them might legally use a different variant as its parent. This means that there is no way to bound the number of different variants of one forked unit in the ch-DAG by less than $N$ and we would like to propose a mechanism that does not allow to create more (while without any additional rules there might be an exponential number of variants - see Section H).

In the solution we propose every node must "commit" to at most one variant of every received unit, by analogy to what happens in RBC. The general rule is that honest nodes can accept only those units that someone committed to. By default (when no forks are yet detected), the commitment will be simply realized by creating a next round unit that has one particular variant as its parent. On the other hand, if a fork is observed, then every node will have to send its commitment using RBC, to prevent attacks akin to the fork bomb (Section H).

A node can work in one of two states: normal and alert. In the normal state, the node multicasts its own units, gossips with other nodes, and builds its local copy of the ch-DAG. Now assume that a node $k$ detects a fork, i.e., obtains two distinct variants $U_{1}$ and $U_{2}$ of the round- $r$ unit created by $i$. Then node $k$ enters the alert mode. It stops building the ch-DAG and broadcasts a special message Alert $(k, i)$ using RBC. The message consists of three parts:

(1) Proof of node $i$ being malicious, i.e., the pair $\left(U_{1}, U_{2}\right)$.

(2) The hash and the round number of the unit of highest round created by node $i$ that is currently present in $\mathcal{D}_{k}$ (possibly null).

(3) The alert id (every node assigns numeric ids to its alerts, ie., $i d=0,1,2, \ldots$ and never starts a new alert before finishing the previous one).

By broadcasting the hash, node $k$ commits to a single variant of a whole chain of units created by node $i$, up to the one of highest round, currently known by node $k$. Since up to this point node $k$ did not know that node $i$ was malicious, then indeed units created by node $i$ in $\mathcal{D}_{k}$ form a chain.

Implementing this part requires some additional care, since now units cannot be validated and added one by one. Also, the sender should start with proving that the highest unit actually exists (by showing the unit and the set of its parents), so a malicious node cannot send arbitrarily large chunks of units, that cannot be parsed individually.

At this point we also note that since every node performs its alerts sequentially (by assigning numbers to them), in the worst case there can be at most $O(N)$ alerts run in parallel.

We now provide details on how a node should behave after a fork is discovered. Assume that node $k$ is honest, and $i$ is malicious (forking).

(1) Node $k$ stops communicating with node $i$ immediately after obtaining a proof that $i$ is malicious.

(2) Immediately after obtaining a proof that $i$ is malicious, node $k$ enters the alert mode, and broadcasts $\operatorname{Alert}(k, i)$.

(3) Node $k$ can issue at most one Alert $(k, \cdot)$ at once, and alerts must be numbered consecutively.

(4) Node $k$ cannot participate in Alert $(j, \cdot)$ if it has not locally terminated all previous alerts issued by node $j$.

(5) Node $k$ exits alert mode immediately after $\operatorname{Alert}(k, i)$ ends locally for $k$, and stores locally the proof of finishing the alert, along with the proof that node $i$ is malicious.

(6) After exiting the alert mode, node $k$ can still accept units created by $i$, but only if they are below at least one unit that some other node committed to.

The Quick-DAG-Grow protocol enriched by the alert mechanism implemented through the above rules can serve as a reliable way to grow a ch-DAG that provably contains at most $N$ forks per node. More specifically, in Section B. 2 we prove the following

Theorem A.1. Protocol Quick-DAG-Grow with alert system is reliable and growing (as in definition 3.2). Additionally, every unit can be forked at most $N$ times.

\section{A. 2 Adjustments to the Consensus Mechanism}

In Aleph protocol the mechanism of choosing round heads is designed in a way such that no matter whether the adversary is involved or not, the head is chosen in an expected constant number of rounds. While this is optimal from the theoretical perspective in practice we care a lot about constants, and for this reason we describe a slight change to the consensus mechanism that allows us to achieve latency of 3 rounds under favorable network conditions (which we expect to be the default) and still have worst-case constant latency.

The key changes are to make the permutation (along which the head is chosen) "a little" deterministic and to change the pattern of initial deterministic common votes.

Permutation. Recall that in order to choose the head for round $r$ in Aleph first a random permutation over units at this round $\pi_{r}$ is commonly generated and then in the order determined by $\pi_{r}$ the first unit decided 1 is picked as the head. The issue is that in order for the randomness to actually "trick" the adversary, and not allow him to make the latency high, the permutation can be revealed at earliest at round $r+4$ in case of Aleph, and $r+5$ in case of QuickAleph ${ }^{20}$. In particular the decision cannot be made earlier than a round after the permutation is revealed. To improve upon that in the optimistic case, we make the permutation $\pi_{r}$ partially deterministic and allow to recover the first entry of the permutation already in round $r$. More specifically, the mechanism of determining $\pi_{r}$ is as follows:
- Generate pseudo-randomly an index $i_{0} \in[N]$ based on just the round number $r$ or on the head of round $r-1$.
- Let $\tau_{r}$ be a random permutation constructed as previously using SecretBits $(\cdot, r+5)$.
- In $\pi_{r}$ as first come all units created by node $i_{0}$ in round $r$ (there might be multiple of them because of forking), sorted by hashes, and then come all the remaining units of round $r$ in the order as determined by $\tau_{r}$.

Such a design allows the nodes to learn $i_{0}$ and hence the first candidate for the head right away in round $r$ and thus (in case of positive decision) choose it already in round $r+3$. The remainder of the permutation is still random and cannot be manipulated by the adversary, thus the theoretical guarantees on latency are preserved.
\footnotetext{
${ }^{20}$ The difference stems from the potential existence of forks in ch-DAG constructed in QuickAleph and will become more clear after reading lemma C. 11
}

For completeness we provide pseudocode for the new version of GeneratePermutation. In the pseudocode DefaultIndex can be any deterministic function that is computed from $\mathcal{D}$ after the decision on round $r-1$ has been made. Perhaps the simplest option is DefaultIndex $(r, \mathcal{D}):=1+(r \bmod N)$. In practice we might want to use some more complex strategy, which promotes nodes creating units that spread fast; for instance, it could depend on the head chosen for the $(r-1)$-round.

```
GeneratePermutation $(r, \mathcal{D})$ :
    if $\mathrm{R}(\mathcal{D})<r+3$ then output $\perp$
    $2 i_{0} \leftarrow$ DefaultIndex $(r, \mathcal{D})$
    $3\left(V_{1}, V_{2}, \ldots, V_{l}\right) \leftarrow$ list of units created by $\mathcal{P}_{i_{0}}$ in round $r$ sorted
    by hashes
    /* Typically $l=1, l>1$ can only happen if $\mathcal{P}_{i_{0}}$
        forked in round $r$.
            $\star /$
    4 for each unit $U$ of round $r$ in $\mathcal{D}$ do
        $i \leftarrow$ the creator of $U$
        $x \leftarrow \operatorname{SecretBits}(i, r+5, \mathcal{D})$
        if $x=\perp$ then output $\left(V_{1}, V_{2}, \ldots, V_{l}\right)$
        assign priority $(U) \leftarrow \operatorname{hash}(x \| U) \in\{0,1\}^{\lambda}$
    let $\left(U_{1}, U_{2}, \ldots, U_{k}\right)$ be the units in $\mathcal{D}$ of round $r$ not created
    by $\mathcal{P}_{i_{0}}$ sorted by priority $(\cdot)$
10 output $\left(V_{1}, V_{2}, \ldots, V_{l}, U_{1}, U_{2}, \ldots, U_{k}\right)$
```

Common Votes. In QuickAleph we use a slightly modified sequence of initial deterministic common votes as compared to Aleph. This change is rather technical but, roughly speaking, is geared towards making the head decision as quickly as possible in the protocol. Here is a short summary of the new CommonVote scheme. Let $U_{0}$ be a unit created by $i$ and $r:=\mathrm{R}\left(U_{0}\right)$, then for $d=2,3, \ldots$ we define

$$
\text { CommonVote }\left(U_{0}, r+d\right):= \begin{cases}1 & \text { for } d=2 \\ 0 & \text { for } d=3 \\ \operatorname{SecretBits}(i, r+d+1) & \text { for } d \geqslant 4\end{cases}
$$

Where in the last case we extract a single random bit from SecretBits, as previously.

\section{A. 3 Reducing size of units}

We note that encoding parents of a unit by a set of their hashes is rather inefficient as it takes roughly $N \cdot \lambda$ bits to store just this information ( $\lambda$ bits per parent). Since the reasonable choices for lambda in this setting are 256 or 512 , this is a rather significant overhead. In this section we propose a solution that reduces this to just a small constant number (i.e. around 2) of bits per parent.

Recall that in the absence of forks every unit is uniquely characterized by its creator id and by round number. Since forking is a protocol violation that is severely penalized (for instance, by slashing the node's stake), one should expect it to happen rarely if at all. For this reason it makes sense to just use this simple encoding of units: $(i, r)$ (meaning the unit created by $i$ at round $r$ ) and add a simple fallback mechanism that detects forked parents. More specifically, the parents of a unit $U$ are encoded as a combination of the following two pieces of data

(1) A list $L_{U}=\left[r_{1}, r_{2}, \ldots, r_{N}\right]$ of length $N$ that contains the round numbers of parents of $U$ corresponding to creators $1,2, \ldots, N$. In the typical situation the list has only a small number of distinct elements, hence can be compressed to just 2 or 3 bits per entry.

(2) The "control hash"

$$
h_{U}:=h\left(h_{1}\left\|h_{2}\right\| \ldots \| h_{N}\right)
$$

where $h_{1}, h_{2}, \ldots, h_{N}$ are the hashes of parents of $U$.

Consequently, the above encoding requires $O(N+\lambda)$ bits instead of $\Omega(N \lambda)$ bits as the original one. While such a construction does not allow to identify forks straight away, it does allow to identify inconsistencies. Indeed, if an honest node $k$ receives a unit $U$ from node $j$, then it will read the parent rounds and check if all parent units are already present in $\mathcal{D}_{k}$. If not, then these units will be requested from the sender of $U$, and failing to obtain them will result in dropping the unit $U$ by node $k$. In case $k$ knows all parent units, it may verify the control hash and if inconsistency is detected, node $k$ should request ${ }^{21}$ all parent units present in $\mathcal{D}_{j}$ from node $j$. If node $j$ is also honest, then node $k$ will reveal a forking unit among parents of $U$ and issue an alert before adding $U$ to $\mathcal{D}_{k}$. On the other hand, in case $\mathcal{P}_{j}$ does not provide parents of $U, \mathcal{P}_{k}$ can simply drop the unit $U$.

\section{B ANALYSIS OF PROTOCOLS CONSTRUCTING CH-DAGS}

Within the paper, two possible protocols constructing ch-DAG are considered, namely, DAG-Grow and Quick-DAG-Grow. In this section, we analyze these protocols in the context of desirable properties as in definition 3.2.

\section{B. 1 DAG-Grow}

We provide a proof of Theorem 3.1, i.e., we show that the DAG-Grow protocol is reliable, ever-expanding, fork-free and advances the DAG rounds at least as quickly as the asynchronous are progressing

Proof of Theorem 3.1.

Reliable. Node $\mathcal{P}_{i}$ adds a unit $U$ to its ch-DAG only if it is locally output by ch-RBC. By the properties of ch-RBC we conclude that if a unit is locally output for one honest node, it is eventually locally output by every other honest node.

Ever-expanding. Assume the contrary, i.e., there exists $r$ such that no honest node $\mathcal{P}_{i}$ can gather $2 f+1$ units of round $r$ in $\mathcal{D}_{i}$ and hence no node is able to produce a unit of round $r+1$. Let $r_{0}$ be the minimum such $r$. We know that $r_{0}>0$, since every honest node participates in broadcasting of units of round 0 without waiting for parents. The units of round $r_{0}-1$ created by honest nodes are eventually added to local copies of all honest nodes, hence at some point, every honest node can create and broadcast a unit of round $r$, which then is eventually included in all local copies of ch-DAG As there are $2 f+1$ honest nodes, we arrive at a contradiction.

Fork-free. This is a direct consequence of the "reliability" of chRBC, see Lemma F.1(Fast Agreement)
\footnotetext{
${ }^{21}$ The request should include signed hashes of all units from $\mathcal{D}_{k}$ that are indeed parents of $U$ to prevent $k$ from sending false requests.
}

DAG rounds vs asynchronous rounds. By Lemma D. 1 each honest node produces at least one new unit each 5 async-rounds. Since each new unit needs to have higher DAG-round, it concludes the proof.

\section{B. 2 Quick-DAG-Grow}

Since Quick-DAG-Grow does not rely on reliable broadcast, it does not enjoy all the good properties of the DAG-Grow protocol. Most notably, it allows nodes to process forks and add forked units to the ch-DAG. We prove that it is still reliable and ever-growing, and additionally that the number of forks created by a single node is bounded by $N$ for each round, i.e., we prove Theorem A.1.

Proof of Theorem A.1.

Reliable. First, we observe that while the alert system may slow the protocol down for some time, it may not cause it to stall indefinitely. Indeed, each honest node needs to engage in at most $N$ alert RBC instances per each discovered forker, i.e., at most $\frac{1}{3} N^{2}$ instances in total. Since an RBC instance started by an honest node is guaranteed to terminate locally for each node, there is some point in time of the protocol execution, name it $T_{0}$, after which no more alerts instantiated by honest nodes are active anymore. Consequently, honest nodes can gossip their local copies of ch-DAG after $T_{0}$ without any obstacles. This means in particular that every pair of honest nodes exchange their ch-DAG information infinitely often (with probability one), which implies reliability.

Ever-expanding. This follows from reliability as the set of honest is large enough $(2 f+1)$ to advance the rounds of the ch-DAG by themselves. Indeed, by induction, for every round number $r>0$ every honest node eventually receives at least $2 f+1$ units created by non-forkers (for instance honest nodes) in round $r-1$ and hence can create a correct unit of round $r$ by choosing these units as parents.

Bounding number of forks. An honest node $i$ adds a fork variant to its local copy of ch-DAG only in one of the two scenarios:
- It was the first version of that fork that $i$ has received,
- Some other node has publicly committed to this version via the alert system.

Since there are only $N-1$ other nodes that could cause alerts, the limit of maximum of $N$ versions of a fork follows.

\section{ANALYSIS OF CONSENSUS PROTOCOLS}

The organization of this section is as follows: In Subsection C. 1 we analyze the consensus mechanism in the Aleph protocol. In particular we show that ChooseHead is consistent between nodes and incurs only a constant number of rounds of delay in expectation. This also implies that every unit (created by an honest node) waits only a constant number of asynchronous rounds until it is added to the total order, which is necessary in the proof of Theorem 2.1. The Subsection C. 2 is analogous but discusses QuickAleph (introduced in Section A) instead. Finally in Subsection C. 3 we show an optimistic bound of 3 rounds validation for QuickAleph.

Since the proofs in Subsection C. 2 are typically close adaptations of these in Subsection C. 1 we recommend the reader to start with the latter. In the analysis of QuickAleph we distinguish between two cases: when the adversary attacks using forks, and when no
forks have happened recently. In the former case the latency might increase from expected $O(1)$ to $O(\log N)$ rounds, yet each time this happens at least one of the nodes gets banned ${ }^{22}$ and thus it can happen only $f$ times during the whole execution of the protocol. We do not attempt to optimize constants in this section, but focus only on obtaining optimal asymptotic guarantees. A result of practical importance is obtained in Subsection C. 3 where we show that 3 rounds are enough in the "optimistic" case.

Throughout this section we assume that the SecretBits primitive satisfies the properties stated in Definition 3.3. An implementation of SecretBits is provided in Section 4.4.

\section{1 Aleph}

We would to prove that the expected number of rounds needed for a unit to be ordered by the primitive OrderUnits is constant, and that each honest node is bound to produce the same ordering. To achieve this, first we need a series of technical lemmas.

Lemma C. 1 (Vote latency). Let $X$ be a random variable which, for a unit $U_{0}$, indicates the number of rounds after which all units vote unanimously on $U_{0}$. Formally, let $U_{0}$ be a unit of round $r$ and define $X=X\left(U_{0}\right)$ to be the smallest $l$ such that there exists $\sigma \in\{0,1\}$ such that for every unit $U$ of round $r+l$ we have $\operatorname{Vote}\left(U_{0}, U\right)=\sigma$ Then for $K \in \mathbb{N}$ we have

$$
P(X \geqslant K) \leqslant 2^{4-K}
$$

Proof. Fix a unit $U_{0}$. Let $r^{\prime}>r+4$ be a round number and let $\mathcal{P}_{k}$ be the first honest node to create a unit $U$ of the round $r^{\prime}$. Let $\sigma$ denote a vote on $U_{0}$ that was cast by at least $f+1$ units in $\downarrow(U)$. Then, every unit of round $r^{\prime}$ will have at least one parent that votes $\sigma$. It is easy to check that if also $\sigma=$ CommonVote $\left(U_{0}, r^{\prime}, \mathcal{D}_{k}\right)$, then every unit of round $r^{\prime}$ is bound to vote $\sigma$.

By Definition 3.3 (1), the votes of units in $\downarrow(U)$ are independent of CommonVote $\left(U_{0}, r^{\prime}, \mathcal{D}_{k}\right)$ since it uses SecretBits $\left(i, r^{\prime}, \mathcal{D}_{k}\right)$. Therefore, if at round $r^{\prime}-1$ voting was not unanimous, then with probability at least $1 / 2$ it will be unanimous starting from round $r^{\prime}$ onward. Since $P(X \geqslant 5) \leqslant \frac{1}{2}$, we may calculate $P(X \geqslant K) \leqslant 2^{-K+4}$, for $K>4$ by induction, and observe that it is trivially true for $K=1,2,3,4$.

Lemma C. 2 (Decision latency). Let Y be a random variable that, for a unit $U_{0}$, indicates the number of rounds after which all honest nodes decide on $U_{0}$. Formally, let $U_{0}$ be a unit of a round $r$ and define $Y=Y\left(U_{0}\right)$ to be the smallest $l$ such that there exists $\sigma$ such that for every honest node $\mathcal{P}_{i}$ if $\mathrm{R}\left(\mathcal{D}_{i}\right) \geqslant r+l$, then $\operatorname{Decide}\left(U_{0} ; \mathcal{D}_{i}\right)=\sigma$. Then for $K \in \mathbb{N}, K>0$ we have

$$
P(Y \geqslant K) \leqslant \frac{K}{2^{K-5}}=O\left(K \cdot 2^{-K}\right)
$$

Proof. First, we need to check that $Y$ is well-defined, i.e. for every $U_{0}$ there is $l \in \mathbb{N}$ and there is $\sigma$ such that for every honest node $\mathcal{P}_{i}$ if $\mathrm{R}\left(\mathcal{D}_{i}\right) \geqslant r+l$, then $\operatorname{Decide}\left(U_{0} ; \mathcal{D}_{i}\right)=\sigma$.

We observe that if there is an honest node $\mathcal{P}_{k}$ and a unit $U \in \mathcal{D}_{k}$ such that UnitDecide $\left(U_{0}, U, \mathcal{D}_{k}\right)=\sigma \neq \perp$, then eventually for every honest node $\mathcal{P}_{i}$, we will have $\operatorname{Decide}\left(U_{0} ; \mathcal{D}_{i}\right)=\sigma$. Indeed, fix an honest node $\mathcal{P}_{k}$ and a unit $U \in \mathcal{D}_{k}$ of round $r^{\prime}>r+4$
\footnotetext{
${ }^{22}$ Banning forkers is implemented by adding one simple rule to the growing protocol never pick forking nodes as parents.
}

such that UnitDecide $\left(U_{0}, U, \mathcal{D}_{k}\right)=\sigma \neq \perp$. Then, at least $2 f+1$ units of round $r^{\prime}-1$ vote $\sigma$ and CommonVote $\left(U_{0}, r^{\prime}\right)=\sigma$. Now by the definitions of UnitVote, we see that every other unit $U^{\prime}$ of round $r^{\prime}$ must have UnitVote $\left(U_{0}, U^{\prime}, \mathcal{D}_{k}\right)=\sigma$, and thus by a simple induction we get that, every unit of round greater than $r^{\prime}$ will also vote $\sigma$ on $U_{0}$. Finally, by the definition of Decide we see that once UnitDecide outputs a $\sigma \neq \perp$, then the result of UnitVote is $\sigma$ and does not change with the growth of the local copy of ch-DAG.

By the above, if we prove that UnitDecide will always eventually output non- $\perp$, then the definition of $Y$ will be correct.

Let $Y^{\prime}=Y^{\prime}\left(U_{0}\right)$ be defined as the smallest $l$ such that for every unit $U$ of round $r+l$ the function UnitDecide $\left(\mathrm{U}_{0}, \mathrm{U}\right)$ outputs $\sigma$. If $X\left(U_{0}\right)=l$, then either $Y^{\prime}\left(U_{0}\right)=l$, or $Y^{\prime}\left(U_{0}\right)$ is the index of the first round after $l$ at which CommonVote equals $\sigma$. The probablity that at round $r+l$ nodes vote unanimously $\sigma$ for the first time, but then no CommonVote was equal to $\sigma$ before round $r+L$ equals $P(X=l) / 2^{L-1-l}$, and thus we have:

$$
\begin{aligned}
P\left(Y^{\prime} \geqslant L\right) & \leqslant \sum_{l=0}^{L-1} 2^{-L+1+l} P(X=l)+P(X \geqslant L) \\
& =\sum_{l=0}^{L-1} 2^{-L+1+l} P(X=l)+\sum_{l=L}^{+\infty} P(X=l) \\
& =2^{-L} P(X \geqslant 0)+\sum_{l=0}^{L-1} 2^{-L+l} P(X \geqslant l) \\
& \leqslant \frac{L+1}{2^{L-4}}
\end{aligned}
$$

Since $X$ is well-defined, then so is $Y^{\prime}$ and we see that UnitDecide eventually outputs a non $-\perp$. Moreover, observe that $Y \leqslant Y^{\prime}+1$. Indeed, if every node of round $r+l$ has decided $\sigma$, then the ch-DAG of height $\geqslant r+l+1$ includes at least one of the decisions, and we can read the last required secret. Finally, $Y \leqslant Y^{\prime}+1$ implies

$$
P(Y \geqslant K) \leqslant P\left(Y^{\prime} \geqslant K-1\right) \leqslant \frac{K}{2^{K-5}}
$$

The following lemma guarantees that at every level there are a lot of units that are decided on 1 .

Lemma C. 3 (Fast positive decisions). Assume that an honest node $\mathcal{P}_{i}$ has just created a unit $U$ of round $r+3$. At this point in the protocol execution, there exists a set $\mathcal{S}_{r}$ of at least $2 f+1$ units of round $r$ such that for every $U_{0} \in \mathcal{S}_{r}$, every honest node will eventually have $\operatorname{Decide}\left(U_{0}, \mathcal{D}_{i}\right)=1$.

Proof. Let $K$ be a set of $2 f+1$ nodes that created units in $\downarrow(U)$. Additionally, let $\mathcal{T}$ be the set of $2 f+1$ units of round $r+1$ created by nodes in $K$. Every unit in $\downarrow(U)$ is above $2 f+1$ units of round $r+1$, hence it is above at least $f+1$ units in $\mathcal{T}$ (the remaining $f$ units may be created by nodes outside of $K$ ). By the pigeonhole hole principle, there exists a unit $U_{0} \in \mathcal{T}$ such that at least $f+1$ units in $\downarrow(U)$ are above it. Set $\mathcal{S}_{r}:=\downarrow\left(U_{0}\right)$.

Let $V$ be a unit of round $r+3$ and $V^{\prime} \in \downarrow(V)$ be its parent that is above $U_{0}$ (which has to exist since every subset of $2 f+1$ units of round $r+2$ must include at least one unit above $U_{0}$ ). Since CommonVote $\left(U_{r}, W\right)$ equals 1 for all $W$ of rounds $r+1, r+2, r+3$,
for $U_{r} \in S_{r}$ we have $\operatorname{Vote}\left(U_{r}, U_{0}\right)=1$, hence $\operatorname{Vote}\left(U_{r}, V^{\prime}\right)=1$, and finally, Vote $\left(U_{r}, V\right)=1$.

Thus, during all subsequent rounds $\operatorname{Vote}\left(U_{r}, \cdot\right)=1$ and $U_{r}$ will be decided 1 as soon as the next CommonVote equals 1.

Intuitively, this lemma states that the set of potential heads that will be eventually positively decided is large. Additionally, it is defined relatively quickly, i.e., before the adversary sees the content of any unit of round $r+3$. Importantly, it is defined before the permutation in which potential heads will be considered is revealed Note that it has some resemblance to the spread protocol defined in [5].

In general, the above result cannot be improved, as the adversary can always slow down $f$ nodes, thus their units, unseen by others, may not be considered "ready" to put in the linear order. Also, note that this lemma does not require any common randomness, as the votes in round $r+3$ depend only on the structure of the ch-DAG.

Lemma C. 4 (Fast negative decisions). Let $U$ be a unit of round $r$ such that for some honest node $i, U \notin \mathcal{D}_{i}$ even though $\mathrm{R}\left(\mathcal{D}_{i}\right) \geqslant r+4$. Then, for any local view $\mathcal{D}$, Decide $(U, \mathcal{D}) \neq 1$.

Proof. Assume for contradiction that $\mathcal{D}$ is local view such that $\operatorname{Decide}(U, \mathcal{D})=1$. By definition of Decide, there has to be unit $V_{1} \in \mathcal{D}$ such that UnitDecide $\left(U, V_{1}, \mathcal{D}\right)=1$. Let now $\mathcal{D}^{\prime}$ be a local view such that $\mathcal{D}, \mathcal{D}_{i} \subseteq \mathcal{D}^{\prime}$. Such $\mathcal{D}^{\prime}$ is possible to construct since all local views have to be compatible by point (2) of Theorem 3.1.

Let $V_{0}$ be a unit of DAG-round $\mathrm{R}\left(\mathcal{D}_{i}\right)+4$ in $\mathcal{D}_{i}$ (and hence in $\left.\mathcal{D}^{\prime}\right)$. Since $V_{0}$ can't be above $U$, for every $V \in \downarrow\left(V_{0}\right)$ we have $\operatorname{Vote}\left(U, V, \mathcal{D}_{i}\right)=0$. Since $\left|\downarrow\left(V_{0}\right)\right| \geqslant 2 f+1$, each unit of DAGround $r+4$ in $\mathcal{D}^{\prime}$ is above at least one unit in $\downarrow\left(V_{0}\right)$ and hence votes either 0 or by CommonVote. But CommonVote for $U$ always equals 0 at DAG-round $\mathrm{R}(U)+4$ by definition, and consequently, each unit of DAG-round $r+4$ has to Vote 0 for $U$. If all units votes unanimously at any given DAG-round, such vote is passed over to the next DAG-rounds contradicting the existence of $V_{1}$ deciding 1 , what concludes the proof.

Fact C.1. Let $X_{1}, \ldots, X_{M}$ be random variables, $K \in \mathbb{R}$. Then

$$
P\left(\max \left(X_{1}, \ldots, X_{M}\right) \geqslant K\right) \leqslant \sum_{m=1}^{M} P\left(X_{m} \geqslant K\right)
$$

Proof. Simply observe that if $\max \left(X_{1}, \ldots, X_{M}\right) \geqslant K$, then for at least one $m \in[M]$ it must be $X_{m} \geqslant K$

$$
\left\{\max \left(X_{1}, \ldots, X_{M}\right) \geqslant K\right\} \subseteq \bigcup_{m=1}^{M}\left\{X_{m} \geqslant K\right\}
$$

The next lemma shows the bound on the number of additional rounds that are needed to choose a head.

Lemma C. 5 (ChooseHead latency). The function ChooseHead satisfies the following properties:
- Agreement. For every round $r$ there is a uniquely chosen head $U$, i.e., for every ch-DAG $\mathcal{D}$ maintained by an honest node, ChooseHead $(r, \mathcal{D}) \in\{\perp, U\}$.
- Low latency. Let $Z_{r}$ be a random variable defined as the smallest $l$ such that for every local copy $\mathcal{D}$ of height $r+l$ we have ChooseHead $(r, \mathcal{D}) \neq \perp$. Then for $K \in \mathbb{N}, K>0$ we have $P\left(Z_{r} \geqslant K\right)=O\left(K \cdot 2^{-K}\right)$

Proof.

Agreement. Suppose for the sake of contradiction that there exist two ch-DAGs $\mathcal{D}_{i}$ and $\mathcal{D}_{j}$ maintained by honest nodes $i, j$, such that ChooseHead $\left(r, \mathcal{D}_{i}\right)=U$ and ChooseHead $\left(r, \mathcal{D}_{j}\right)=U^{\prime}$ for two distinct units $U, U^{\prime}$. Note that for this we necessarily have

$$
\mathrm{R}\left(\mathcal{D}_{i}\right) \geqslant r+4 \quad \text { and } \mathrm{R}\left(\mathcal{D}_{j}\right) \geqslant r+4
$$

as otherwise GeneratePermutation $(r, \cdot)$ would return $\perp$.

Further, as necessarily Decide $\left(U, \mathcal{D}_{i}\right)=1$ and $\operatorname{Decide}\left(U^{\prime}, \mathcal{D}_{j}\right)=$ 1, by Lemma C. 4 we obtain that both $U$ and $U^{\prime}$ need to be present in $\mathcal{D}_{i}$ and $\mathcal{D}_{j}$.

To conclude the argument it remains to observe that the priorities computed by GeneratePermutation $(r, \cdot)$ are consistent between different ch-DAGs as they are computed deterministically based on the common outcome of SecretBits. We thus arrive at the desired contradiction: if, say, $U$ has higher priority than $U^{\prime}$ then the $j$ th node should have choosen $U$ instead of $U^{\prime}$.

Constant latency. Consider the set $\mathcal{S}_{r}$ from Lemma C.3. Let $\mathcal{P}_{i}$ be a node, $x=\operatorname{SecretBits}\left(i, r+4, \mathcal{D}_{i}\right)$, and let

$$
\pi_{r}=\left(U_{1}, \ldots, U_{k}\right)=\text { GeneratePermutation }\left(r, \mathcal{D}_{i}\right)
$$

The permutation given by priorities hash $\left(x \| U_{i}\right)$ for $i=1,2, \ldots, N$ is uniformly random by Definition 3.3 (1) and the assumption that hash gives random bits in $\{0,1\}^{\lambda}$. Moreover, it is independent of the set $\mathcal{S}_{r}$, as this set is defined when the first unit of round $r+3$ is created and $\pi_{r}$ was unknown before it.

Let $S_{r}$ denote a random variable defined as the smallest index $s$ such that $s$-th element of the permutation $\pi_{r}$ is in $\mathcal{S}_{r}$. Since the permutation is uniformly random and independent of $\mathcal{S}_{r}$, then for $s=1, \ldots, f+1$ we have

$$
P\left(S_{r}=s\right)=\frac{2 f+1}{3 f+2-s} \prod_{j=1}^{s-1} \frac{f+1-j}{3 f+2-j}
$$

and $P\left(S_{r}>f+1\right)=0$, hence for $s=1, \ldots, k$, we have

$$
P\left(S_{r}=s\right) \leqslant 3^{-s+1}
$$

By Lemma C.3, all units from $\mathcal{S}_{r}$ must be decided 1, then to calculate ChooseHead $\left(r, \mathcal{D}_{i}\right)$ we need to wait for decisions on at most $S_{r}$ units of round $r$. Using Fact C. 1 for random variables from Lemma C. 2 we see that for $K>4$

$$
P\left(Z_{r} \geqslant K \mid S_{r}=s\right) \leqslant \sum_{s=1}^{k} P\left(Y\left(U_{s}\right) \geqslant K\right) \leqslant s \cdot \frac{K}{2^{K-5}}
$$

therefore

$$
P\left(Z_{r} \geqslant K\right) \leqslant \sum_{s=1}^{N} P\left(S_{r}=s\right) P\left(Z_{r} \geqslant K \mid S_{r}=s\right)=O\left(K \cdot 2^{-K}\right)
$$

We end this section with a theorem that shows how long does it take to append a head to the linear order.

THEOREM C. 6 (ORderUNiTS LATENCY). Let $W_{r}$ be a random variable that indicates number of rounds required to append a head of round $r$ to the linear order. Formally, $W_{r}$ is defined as the smallest $l$ such that for every local copy $\mathcal{D}_{i}$ of height $r+l$ we have

$$
\text { ChooseHead }\left(r, \mathcal{D}_{i}\right) \in \operatorname{OrderUnits}\left(\mathcal{D}_{i}\right)
$$

Then

$$
\mathbb{E}\left(W_{r}\right)=O(1)
$$

Proof. The first part of the proof will show that for all $K \in$ $\mathbb{N}, K>0$ we have

$$
P\left(W_{r} \geqslant K\right)=O\left(K \cdot 2^{-K}\right)
$$

Let $\mathcal{P}_{i}$ be a node. ChooseHead $\left(r, \mathcal{D}_{i}\right) \in \operatorname{OrderUnits}\left(\mathcal{D}_{i}\right)$ implies that for all $j=0,1, \ldots, r$ we have

$$
\text { ChooseHead }\left(j, \mathcal{D}_{i}\right) \neq \perp
$$

Therefore, if $W_{r} \geqslant K$, then for at least one round $j$ we have $Z_{j} \geqslant$ $K+r-j$, where $Z_{j}$ is a random variable from Lemma C.5. Since

$$
\left(W_{r} \geqslant K\right) \subseteq \bigcup_{j=0}^{r}\left(Z_{j} \geqslant K+r-j\right)
$$

then

$$
\begin{aligned}
P\left(W_{r} \geqslant K\right) & \leqslant \sum_{j=0}^{r} P\left(Z_{j} \geqslant K+r-j\right) \leqslant O(1) \sum_{j=0}^{r} \frac{K+j}{2^{K+j}} \\
& =O(1) \sum_{j=K}^{K+r} \frac{j}{2^{j}} \leqslant O(1) \sum_{j=K}^{+\infty} \frac{j}{2^{j}}=O\left(K \cdot 2^{-K}\right) .
\end{aligned}
$$

Since $W_{r}$ has values in $\mathbb{N}$, then

$$
\mathbb{E}\left(W_{r}\right)=\sum_{K=1}^{+\infty} P\left(W_{r} \geqslant K\right)
$$

and finally we have

$$
\mathbb{E}\left(Z_{r}\right)=\sum_{K=1}^{+\infty} P\left(Z_{r} \geqslant K\right)=O(1) \sum_{K=1}^{+\infty} \frac{K}{2^{K}}=O(1)
$$

\section{2 QuickAleph}

In this section we analyze the QuickAleph consensus mechanism. The main difference when compared to Aleph is that now we allow the adversary to create forks. Below, we introduce two definitions that allow us to capture forking situations in the ch-DAG so that we can prove how the latency of the protocol behaves in their presence. At the end of this subsection we also show that these situations are rare and can happen only $O(N)$ times.

DeFinition C.1. Let $r$ be a round number. We say that a unit $U$ of round $r+4$ is a fork witness if it is above at least two variants of the same unit of round $r$ or $r+1$.

DEFINITION C.2. Let $\mathcal{U}$ be a set of all $2 f+1$ units of round $r+4$ created by honest nodes. We say that the rth round of the ch-DAG is publicly forked, if there are at least $f+1$ fork witnesses among units $\mathcal{U}$.
We proceed with the analysis of the latency of QuickAleph starting again by determining the delay in rounds until all nodes start to vote unanimously.

Lemma C. 7 (Vote latency [For QuickAleph]). Let $X$ be a random variable that for a unit $U_{0}$ represents the number of rounds after which all units unanimously vote on $U_{0}$. Formally, let $U_{0}$ be a unit of round $r$ and define $X=X\left(U_{0}\right)$ to be the smallest $l$ such that for some $\sigma \in\{0,1\}$, for every unit $U$ of round $r+l$ we have $\operatorname{Vote}\left(U_{0}, U\right)=\sigma$. Then for $K \in \mathbb{N}$ we have

$$
P(X \geqslant K) \leqslant\left(\frac{3}{4}\right)^{(K-6) / 2}
$$

Proof. Fix a unit $U_{0}$. Let $r^{\prime}>r+5$ be an odd round number and let $\mathcal{P}_{k}$ be the first honest node to create a unit $U$ of round $r^{\prime}$.

Let $\sigma$ denote a vote such that there is no unit $V \in \downarrow(U)$ for which every unit in $\downarrow(V)$ votes $\sigma$ on $U_{0}$. The value of $\sigma$ is well-defined, i.e., at least one value from $\{0,1\}$ satisfies this property (pick $\sigma=0$ if both 0 and 1 satisfy the above property). Indeed, every two units $V, V^{\prime} \in \downarrow(U)$ have at least one parent unit in common, thus it is not possible that every unit in $\downarrow(V)$ votes $\sigma$, and every unit in $\downarrow\left(V^{\prime}\right)$ votes $1-\sigma$.

Recall that the adversary cannot predict CommonVote $\left(U_{0}, r^{\prime}-1\right)$ or CommonVote $\left(U_{0}, r^{\prime}\right)$ before $U$ is created. Now, if

CommonVote $\left(U_{0}, r^{\prime}-1, \mathcal{D}_{k}\right)=$ CommonVote $\left(U_{0}, r^{\prime}, \mathcal{D}_{k}\right)=1-\sigma$,

then every unit in $\downarrow(U)$ votes $1-\sigma$ on $U_{0}$, and so does $U$. If some other unit $U^{\prime}$ of round $r^{\prime}$ were to vote $\sigma$ on $U_{0}$, then it would mean that at least $2 f+1$ units in $\downarrow\left(U^{\prime}\right)$ vote $\sigma$ on $U_{0}$, which is necessary to overcome the $r^{\prime}$-round common vote. But this is impossible, as this set of $2 f+1$ units must include at least one unit from $\downarrow(U)$. Therefore, if at round $r^{\prime}-2$ voting was not unanimous, then with probability at least $1 / 4$ it will be unanimous starting from round $r^{\prime}$ onward. Hence, we may check $P(X \geqslant K) \leqslant\left(\frac{3}{4}\right)^{(K-6) / 2}$, for $K>5$ by induction, and observe that it is trivially true for $K=1,2,3,4,5$.

Here we obtain a slightly worse bound than in the corresponding Lemma C. 1 for the Aleph protocol. The main reason for that is that the property that units $U$ and $U^{\prime}$ have at least $f+1$ common parent units does not longer hold, as now up to $f$ of them may be forked. One can try to improve this result by introducing different voting rules, and analyzing forked and non-forked cases separately, but this would only make the argument cumbersome, and would not help to decrease the asymptotic latency in the asynchronous case, nor the numerical latency in the partially synchronous case.

LemmA C. 8 (Decision LATENCY [For QuickAleph]). Let $Y$ be a random variable that for a unit $U_{0}$ indicates the number of rounds after which all honest nodes decide on $U_{0}$. Formally, let $U_{0}$ be a unit of a round $r$ and define $Y=Y\left(U_{0}\right)$ to be the smallest $l$ such that there exists $\sigma$ such that for every honest node $\mathcal{P}_{i}$ if $\mathrm{R}\left(\mathcal{D}_{i}\right) \geqslant r+l$, then Decide $\left(U_{0} ; \mathcal{D}_{i}\right)=\sigma$. Then for $K \in \mathbb{N}$ we have

$$
P(Y \geqslant K)=O\left(K \cdot\left(\frac{3}{4}\right)^{K / 2}\right)
$$

Proof. The proof is analogous to that of Lemma C.2. First, we show that $Y$ is well-defined. The argument is exactly the same, not affected by introducing forks. If at least one unit decides a non- $\perp$ value $\sigma$, then every unit of equal or higher round votes $\sigma$, and eventually there will be a round with common vote equal to $\sigma$, which will trigger the decision among all nodes.

We define $Y^{\prime}=Y^{\prime}\left(U_{0}\right)$ as in the proof of Lemma C. 2 and obtain

$$
\begin{aligned}
P\left(Y^{\prime} \geqslant L\right) & \leqslant 2^{-L} P(X \geqslant 0)+\sum_{l=0}^{L-1} 2^{-L+l} P(X \geqslant l) \\
& \leqslant(L+1)\left(\frac{3}{4}\right)^{(L-6) / 2}
\end{aligned}
$$

The final difference is that now $Y \leqslant Y^{\prime}+2$, because we need one additional round to read the value of the last required secret:

$$
P(Y \geqslant K) \leqslant P\left(Y^{\prime} \geqslant K-2\right) \leqslant(K-1)\left(\frac{3}{4}\right)^{(K-8) / 2}
$$

Similarly as for Aleph, for $O(1)$ latency it is crucial to show that a large set of units of round $r$ is guaranteed to be decided positively.

Lemma C. 9 (Fast positive decisions [For QuickAleph]). Let $r \geqslant 0$ be a round number. Assume that an honest node $\mathcal{P}_{i}$ has created a unit $U$ of round $r+4$. There exists at least one unit $U_{0}$ of round $r$ such that Decide $\left(U_{0}, \mathcal{D}_{i}\right)=1$. If additionally $U$ is not a fork witness, then there exists a set $\mathcal{S}_{i, r}$ of at least $1.5 f+1$ units of round $r$ such that for every $U_{0} \in \mathcal{S}_{i, r}$ we have Decide $\left(U_{0}, \mathcal{D}_{i}\right)=1$.

Proof. For convenience, denote by $\mathcal{U}_{r^{\prime}}$ the set of all units of round $r^{\prime}$ below $U$.

Let us first observe that whenever for a unit $U_{0}$ every unit $V \in$ $\mathcal{U}_{r+2}$ votes 1 on $U_{0}$, then it is decided 1 by $U$, i.e.,

$\left(\forall V \in \mathcal{U}_{r+2} \quad \operatorname{Vote}\left(U_{0}, V\right)=1\right) \Rightarrow \operatorname{UnitDecide}\left(U_{0}, U\right)=1$,

This can be seen as follows:
- every unit $W \in \mathcal{U}_{r+3}$ votes 1, i.e., $\operatorname{Vote}\left(U_{0}, W\right)=1$ because $\downarrow(W)$ unanimously vote 1 ,
- since the common vote at round $r+4$ is 1 and all round- $(r+$ 3) parents of $U$ unanimously vote 1 , UnitDecide $\left(U_{0}, U\right)=$ 1.

The remaining part of the proof is to show that in the general case there is at least one unit $U_{0} \in \mathcal{U}_{r}$ that is voted 1 by every unit in $\mathcal{U}_{r+2}$ and that there is at least $1.5 f+1$ such units in the case when $U$ is not a forking witness.

For the first part, note that every unit $W$ of round $r+1$ has at least $f+1$ honest parents at round $r$, thus by reverse counting there exists a unit $U_{0}$ or round $r$, created by an honest node that is a parent of at least $f+1$ honest units at round $r+1$. Note that $U_{0}$ is as desired, i.e., every unit $V$ of round $r+2$ votes 1 for $U_{0}$ because at least one honest unit $W \in \downarrow(V)$ is above $U_{0}$.

For the second part, suppose that $U$ is not a fork witness, in which case the sets $\mathcal{U}_{r}$ and $\mathcal{U}_{r+1}$ contain no forks and hence if we denote

$$
\begin{aligned}
& a:=N-\left|\mathcal{U}_{r}\right|, \\
& b:=N-\left|\mathcal{U}_{r+1}\right|,
\end{aligned}
$$

then $0 \leqslant a, b \leqslant f$. We define the set $\mathcal{S}_{i, r}$ to be all units $U_{0} \in \mathcal{U}_{r}$ that are parents of at least $f-b+1$ units from $\mathcal{U}_{r+1}$.

Suppose now that $U_{0}$ is such a unit, we show that all units in $\mathcal{U}_{r+2}$ vote 1 on $U_{0}$. Indeed, take any $V \in \mathcal{U}_{r+2}$. Since $V$ has at least $2 f+1$ parents in $\mathcal{U}_{r+1}$, at least one of them is above $U_{0}$ (by definition of $\mathcal{S}_{i, r}$ ) and thus $\downarrow(V)$ does not unanimously vote on 0 . The default vote at round $r+2$ is 1 therefore $V$ has to vote 1 on $U_{0}$, as desired.

Finally, it remains to show that $\mathcal{S}_{i, r}$ is large. For brevity denote the number of elements in $\mathcal{S}_{i, r}$ by $x$. We start by observing that the number of edges connecting units from $\mathcal{U}_{r}$ and $\mathcal{U}_{r+1}$ is greater or equal than

$$
(2 f+1) \cdot\left|\mathcal{U}_{r+1}\right|=(2 f+1) \cdot(N-b)
$$

On the other hand, we can obtain an upper bound on the number of such edges by assuming that each unit from $\mathcal{S}_{i, r}$ has the maximum number of $N-b$ incoming edges and each unit from $\mathcal{U}_{r} \backslash \mathcal{S}_{i, r}$ has the maximum number of $f-b$ incoming edges, thus:

$$
x \cdot(N-b)+(N-a-x)(f-b) \geqslant(2 f+1)(N-b)
$$

which after simple rearrangements leads to

$$
x \geqslant 2 f+1-\frac{(f-a)(f-b)}{2 f+1}
$$

Since $0 \leqslant a, b \leqslant f$ we obtain

$$
x \geqslant 1.5 f+1
$$

It is instructive to compare Lemma C. 9 regarding QuickAleph to Lemma C. 3 regarding Aleph. Note that in the non-forked case every unit from the set $\mathcal{U}_{r+2}$ has at least $2 f+1$ parents in the set $\mathcal{U}_{r+1}$, and by pigeonhole principle must be above all units from $\mathcal{S}_{i, r}$. Therefore, we decrease the number of rounds after which units from $\mathcal{S}_{i, r}$ become "popular" from 3 to 2 . This is achieved at the cost of reducing the size of this set by $0.5 f$ units, but is necessary to speed up the protocol in the optimistic case.

Lemma C. 10 (Fast negative Decisions [For QuickAleph]). Let $r \geqslant 0$ be a round number. Let $U$ be a unit of round $r+3$, and $U_{0} \notin U$ be a unit of round $r$. Then, for any local view $\mathcal{D} \ni U_{0}$ with $\mathrm{R}(\mathcal{D}) \geqslant r+5$ we have $\operatorname{Decide}\left(U_{0}, \mathcal{D}\right)=0$.

Proof. Let $U^{\prime}$ be any unit of round $r+3$. Then $U^{\prime}$ votes 0 on $U_{0}$. Indeed, if $U^{\prime}$ were to vote 1 , then there would be at least $2 f+1$ units in $\downarrow\left(U^{\prime}\right)$ above $U_{0}$, and therefore at least one unit in $\downarrow(U)$ above $U_{0}$, but this would imply $U \geqslant U_{0}$. Hence, every unit of round greater or equal to $r+3$ votes 0 on $U_{0}$, and finally every unit of round $r+5$ decides 0 on $U_{0}$ due to the common vote being equal to 0 .

In QuickAleph the latency of choosing head can differ depending on whether we are in the forking situation or not. The following lemma states this formally.

Lemma C. 11 (ChooseHead latency [for QuickAleph]). The function ChooseHead satisfies the following properties:
- Agreement. For every round $r$ there is a uniquely chosen head $U$, i.e., for every ch-DAG $\mathcal{D}$ maintained by an honest node, ChooseHead $(r, \mathcal{D}) \in\{\perp, U\}$.
- Low latency. Let $Z_{r}$ be a random variable defined as the smallest $l$ such that for every local copy $\mathcal{D}$ of height $r+l$ we have ChooseHead $(r, \mathcal{D}) \neq \perp$. Then for $K \in \mathbb{N}$ we have

$$
P\left(Z_{r} \geqslant K\right)=O\left(K \cdot\left(\frac{3}{4}\right)^{K / 2}\right)
$$

if the protocol is locally non-forked at round $r$, and

$$
P\left(Z_{r} \geqslant K\right)=O\left(N^{2} \cdot K \cdot\left(\frac{3}{4}\right)^{K / 2}\right)
$$

otherwise.

Proof. Agreement. Similarly as in the corresponding part of the proof of Lemma C. 5 we observe that the permutation (computed by different nodes) along which the head at a given round is chosen is consistent in the sense that: if two nodes both hold a pair of units $U_{0}, U_{1}$ of round $r$ then these units will end up obtaining the same priorities at both nodes and thus will be placed in the same relative order in the permutation.

Consequently all we need to show is that at the time when an honest node $i$ decides a unit $U$ as the head of round $r$ then every unit $U^{\prime}$ that is not part of its local copy $\mathcal{D}_{i}$ will be decided 0 . This indeed follows from Lemma C. 10 since for ChooseHead $\left(r, \mathcal{D}_{i}\right)$ to not be $\perp$ the ch-DAG $\mathcal{D}_{i}$ needs to have $\mathrm{R}\left(\mathcal{D}_{i}\right) \geqslant r+3$.

Low latency. The proof of the non-forked case differs from the proof of Lemma C. 5 in two ways. Firstly, the set of units that will eventually be decided 1 is smaller, but still has size $\Omega(N)$. Secondly, the first unit in the permutation is known in advance. Here we prove a slightly stronger result, namely we assume that the first $c \geqslant 0$ units are known, where $c$ should be thought of as a small constant (independent of $N$ ). For concreteness one can simply think of $c=1$.

Let $r \geqslant 0$ be a round number. Let $U$ be the first $(r+5)$-round unit created by an honest node. If the protocol is locally non-forked at round $r$, then there are at least $f+1(r+4)$-round units that are not fork witnesses, hence there is at least one unit $V \in \downarrow(U)$, created by node $i$, that is not a fork witness.

Let $\mathcal{U}_{r}$ denote all round- $r$ units below $V$, and consider the set $\mathcal{S}_{i, r} \subseteq \mathcal{U}_{r}$ from Lemma C.9. The set $\mathcal{U}_{r}$ has at most $N$ elements and contains no forks. From Lemma C. 10 we know that all round-r units not in $\mathcal{U}_{r}$ (possibly including some forks of units from $\mathcal{U}_{r}$ ) are decided 0 by every node at round $r+5$. Therefore, we can only consider units from $\mathcal{U}_{r}$ when calculating GeneratePermutation $(r)$. Note that the unit priorities are independent from sets $\mathcal{S}_{i, r}, \mathcal{U}_{r}$, because the unit $V$ exists before the adversary can read SecretBits $(\cdot, r+5)$ using shares from the unit $U$.

Let $s_{i, r}$ denote a random variable defined as the smallest index $s$ such that $s$-th element of the permutation on units $\mathcal{U}_{r}$ is in $\mathcal{S}_{i, r}$, excluding the first $c$ default indices (in other words, $P\left(s_{i, r} \leqslant c\right)=0$ ). The permutation is uniformly random and independent of $\mathcal{S}_{i, r}$ Since we want to calculate an upper bound on the probability distribution of $s_{i, r}$, then we might assume w.l.o.g. that the set $\mathcal{U}_{r}$ has exactly $3 f+1-c$ elements, and that the set $\mathcal{S}_{i, r}$ has $1.5 f+1-c$, in both cases excluding units created by the $c$ default nodes.
Then, for $s=1, \ldots, 1.5 f+1$ we have

$$
P\left(s_{i, r}=s\right)=\frac{1.5 f+1-c}{3 f+2-c-s} \prod_{j=1}^{s-1} \frac{1.5 f+1-j}{3 f+2-c-j}
$$

and $P\left(s_{i, r}>1.5 f+1\right)=0$, hence for $s=1,2, \ldots$ we have

$$
P\left(s_{i, r}=s\right) \leqslant 2^{-s+c}
$$

By Lemma C.9, all units from $\mathcal{S}_{i, r}$ must be decided 1, then to calculate ChooseHead $\left(r, \mathcal{D}_{i}\right)$ we need to wait for decisions on at most $s_{i, r}+c$ units of round $r$ (here we include the default indices with highest priority). Using Fact C. 1 for random variables from Lemma C. 8 we see that for $K>5$

$$
\begin{aligned}
P\left(Z_{r} \geqslant K \mid S_{r}=s\right) & \leqslant \sum_{k=1}^{s+c} P\left(Y\left(U_{k}\right) \geqslant K\right) \\
& \leqslant(s+c)(K-1)\left(\frac{3}{4}\right)^{(K-8) / 2}
\end{aligned}
$$

therefore

$$
\begin{aligned}
P\left(Z_{r} \geqslant K\right) & \leqslant \sum_{s=1}^{N} P\left(S_{r}=s\right) P\left(Z_{r} \geqslant K \mid S_{r}=s\right) \\
& =O\left(K \cdot\left(\frac{3}{4}\right)^{K / 2}\right)
\end{aligned}
$$

On the other hand, if the round is publicly forked, then we observe that we need to wait for decision about at most $N^{2}$ units of round $r$ (since every unit can be forked at most $N$ times), and use Fact C.1.

Theorem C. 12 (OrderUnits latency [For QuickAleph]). Let $W_{r}$ be a random variable representing the number of rounds required to append a head of round $r$ to the linear order, assuming that the default indices for rounds $0, \ldots, r$ are known. Formally, $W_{r}$ is defined as the smallest $l$ such that for every local copy $\mathcal{D}_{i}$ of height $r+l$ we have

$$
\text { ChooseHead }\left(r, \mathcal{D}_{i}\right) \in \operatorname{OrderUnits}\left(\mathcal{D}_{i}\right)
$$

Then

$$
\mathbb{E}\left(W_{r}\right)=O(1)
$$

if the $\frac{4}{\log \frac{4}{3}} \log N$ rounds prior tor are not publicly forked, and

$$
\mathbb{E}\left(W_{r}\right)=O(\log N)
$$

otherwise.

Proof. Let $\mathcal{P}_{i}$ be a node. ChooseHead $\left(r, \mathcal{D}_{i}\right) \in \operatorname{OrderUnits}\left(\mathcal{D}_{i}\right)$ is equivalent to the statement that for $j=0,1, \ldots, r$ we have

$$
\text { ChooseHead }\left(j, \mathcal{D}_{i}\right) \neq \perp
$$

Therefore, if $W_{r} \geqslant K$, then for at least one round $j$ we have $Z_{j} \geqslant$ $K+r-j$, where $Z_{j}$ is a random variable from Lemma C.11.

We start with the forked variant. Since

$$
\left(W_{r} \geqslant K\right) \subseteq \bigcup_{j=0}^{r}\left(Z_{j} \geqslant K+r-j\right)
$$

then

$$
\begin{aligned}
P\left(W_{r} \geqslant K\right) & \leqslant \sum_{j=0}^{r} P\left(Z_{j} \geqslant K+r-j\right) \\
& \leqslant O(1) \cdot N^{2} \sum_{j=0}^{r}(K+j)\left(\frac{3}{4}\right)^{(K+j) / 2} \\
& =O(1) \cdot N^{2} \sum_{j=K}^{K+r} j\left(\frac{3}{4}\right)^{j / 2} \\
& \leqslant O(1) \cdot N^{2} \sum_{j=K}^{+\infty} j\left(\frac{3}{4}\right)^{j / 2} \\
& =O\left(N^{2} \cdot K \cdot\left(\frac{3}{4}\right)^{K / 2}\right)
\end{aligned}
$$

Note that above we used the fact that the constant hidden under the big O notation is independent of the round number.

Observe now that for $K>\frac{4}{\log \frac{4}{3}} \log N$

$$
P\left(W_{r} \geqslant K-4 \log \frac{3}{4} \log N\right)=O\left(K \cdot\left(\frac{3}{4}\right)^{K / 2}\right)
$$

and since $W_{r}$ has values in $\mathbb{N}$, then

$$
\mathbb{E}\left(W_{r}\right)=\sum_{K=1}^{+\infty} P\left(W_{r} \geqslant K\right)=O(\log N)
$$

To analyze the non-forked case we make a very similar argument, but we use the tighter bound for the last $\frac{4}{\log \frac{4}{3}} \log N$ rounds. Denote $A:=\frac{4}{\log \frac{4}{3}} \log N$ for brevity.

$P\left(W_{r} \geqslant K\right) \leqslant \sum_{j=0}^{r} P\left(Z_{j} \geqslant K+r-j\right)$

$\leqslant O(1) \cdot\left(N^{2} \sum_{j=0}^{r-A}(K+j)\left(\frac{3}{4}\right)^{(K+j) / 2}+\sum_{j=r-A+1}^{r}(K+j)\left(\frac{3}{4}\right)^{(K+j) / 2}\right)$

$\leqslant O(1) \cdot\left(\sum_{j=A}^{r}(K+j)\left(\frac{3}{4}\right)^{(K+j) / 2}+\sum_{j=r-A+1}^{r}(K+j)\left(\frac{3}{4}\right)^{(K+j) / 2}\right)$

$\leqslant O(1) \cdot \sum_{j=0}^{r}(K+j)\left(\frac{3}{4}\right)^{(K+j) / 2}=O(1) \cdot \sum_{j=K}^{K+r} j\left(\frac{3}{4}\right)^{j / 2}$

$\leqslant O(1) \cdot \sum_{j=K}^{+\infty} j\left(\frac{3}{4}\right)^{j / 2}=O\left(K \cdot\left(\frac{3}{4}\right)^{K / 2}\right)$,

and therefore

$$
\mathbb{E}\left(W_{r}\right)=\sum_{K=1}^{+\infty} P\left(W_{r} \geqslant K\right)=O(1)
$$

Bounding the number of forked rounds. For completeness we prove that there can be only a small, finite number of publicly forked rounds for which the latency could be $O(\log N)$, and for the remaining rounds (the default case) the latency is $O(1)$ as in Lemma C.11.

Note that according to the Definition C.2, when a dishonest node creates a single fork and shows it to one (or a small number) of honest nodes, this does not yet make the corresponding round publicly forked. Indeed, to increase the protocol latency the adversary must show the fork to the majority of honest nodes, and it must happen during the next 4 rounds. This definition captures a certain trade-off inherent in all possible forking strategies, namely if the attack is to increase the protocol latency, it cannot stay hidden from the honest parties.

Definition C.3. We say that node $i$ is a discovered forker, if there are two variants of the same unit $U, U^{\prime}$ created by $i$, and there is a round $r$, such that every round-r unit is above $U$ and $U^{\prime}$.

Being a discovered forker has two straightforward consequences. First, no unit of round $r$ or higher can connect directly to units created by the forker, hence the node $i$ is effectively banned from creating units of round higher than $r-2$. Moreover, the round- $r$ head unit is also above the fork, and thus we can easily introduce some consensus-based mechanisms of punishing the forker.

Lemma C.13. Let $r \geqslant 0$ be a round number. Among all $2 f+1$ round-r units created by honest nodes there is at least one unit $U_{0}$ such that at least $f+1$ units of round $r+1$ created by honest nodes are above $U_{0}$, and every unit of round $r+2$ (ever created) is above $U_{0}$.

Proof. Recall that units created by honest nodes cannot be forked. Every honest unit of round $r+1$ has at least $2 f+1$ parents created by distinct nodes, hence it has at least $f+1$ parents created by honest nodes. By the pigeonhole principle, there exists at least one round- $r$ unit $U_{0}$, created by an honest node, such that it is a parent of at least $f+1$ units of round $r+1$ created by honest nodes (denote these units by $\mathcal{U}$ ). Now, every unit $U$ of round $r+2$ has at least $2 f+1$ parents created by distinct nodes, hence must have at least one parent in $\mathcal{U}$, which implies $U \geqslant U_{0}$.

Lemma C.14. Whenever a a round is publicly forked then during the next 7 rounds at least one forker is discovered. Consequently, if $f^{\prime}$ denotes the total number of discovered forkers then at most $7 f^{\prime}=O(N)$ rounds of the protocol can be publicly forked.

Proof. Let $U_{0}$ be the round- $(r+5)$ unit whose existence is proved in Lemma C.13. Then $U_{0}$ is above at least one of $f+1$ fork witnesses, and therefore every unit of round $r+7$ is above a fork created by some node $i$ (of course, this could happen earlier). Since no unit of round $r+7$ or higher can connect directly to units created by $i$, then no unit of round higher than $r+5$ created by node $i$ can be present in the ch-DAG. Indeed, every such unit must have children to be added to the local copy held by nodes other than $i$, and those children units would be invalid. Therefore the node $i$ can create forks at six rounds, from $r$ to $r+5$, and these rounds can possibly be publicly forked, hence the $6 f^{\prime}$ upper bound. Note that node $i$ could have forked at some publicly forked rounds with number less than $r$, but then some other malicious node must have been discovered.

\section{3 QuickAleph- The Optimistic Case}

Here, we conduct an analysis of the QuickAleph protocol behavior in a partially synchronous scenario. It shows that the protocol apart from being capable of operating with constant latency in an asynchronous environment (as proved in the previous subsection) but also matches the validation time achieved by state-of-the-art synchronous protocols.

Since the protocol itself does not operate on the network layer, but on ch-DAG, for sake of this subsection we need to translate partial synchrony to the language of ch-DAGs.

Definition C.4. A ch-DAG is synchronous at round $r$ if every unit of round $r$ produced by an honest node has units of round $r-1$ produced by all honest nodes as its parents.

Note that in case of synchronous network the above condition can be enforced by restricting each honest node to wait $\Delta$ before producing the next unit. Since the protocol is secure even with no synchrony assumptions, the delay $\Delta$ can be adaptively adjusted to cope with partial synchrony in a similar manner as, say, in Tendermint.

THEOREM C.15. If $\mathcal{D}$ is a ch-DAG that is synchronous in rounds $r+1$ and $r+2$ and DefaultIndex $(r, \mathcal{D})$ is honest, then any unit of round $r+3$ is enough to compute the head at round $r$.

Proof. Let $i$ be the DefaultIndex at round $r$ and $U$ be the unit created by $\mathcal{P}_{i}$ at round $r$. Since the ch-DAG is synchronous at round $r+1$, all honest nodes create a unit that is above $U$ and thus vote 1 on $U$. Further, again by synchrony, each unit $V$ of round $r+2$ created by an honest node is above at least $2 f+1$ units that are above $U$, hence $V$ "sees" $2 f+1$ votes on 1 for $U$ and since the common vote at this round is 1 we can conclude that

$$
\text { UnitDecide }(U, V, \mathcal{D})=1
$$

Finally, at round $r+3$ the unit $U$ is returned as the first proposal for the head by the GeneratePermutation function ${ }^{23}$ and thus

$$
\text { ChooseHead }(r, \mathcal{D})=U
$$

\section{TRANSACTION ARRIVAL MODEL AND A PROOF OF THEOREM 2.1}

We are interested in the strongest possible form of protocol latency, i.e., not only measuring time between the moment a transaction is placed in a unit and the moment when it is commonly output as a part of total ordering but instead to start the "timer" already when the transaction is input to a node. For this, we introduce the following model.

Transaction arrival model. We assume that nodes continuously receive transactions as input and store them in their unbounded buffers. We make the following assumption regarding the buffer sizes:
\footnotetext{
${ }^{23}$ It may seem that the decision could have been made at round $r+2$ already. Such an "optimization" though breaks safety of the protocol in case of a malicious proposer Indeed, if the proposer forks the proposed unit, some node can decide it as head in round $r+2$, while another node could miss this variant and instead decide a differen variant on 1 later during the protocol execution. This cannot happen at round 3 by Lemma C. 4
}

Definition D. 1 (Uniform BUfFER DISTRIBUTion). The ratio between buffer sizes of two honest nodes at the time of creation of their respective units of a given DAG-round is bounded by a constant $C_{B} \geqslant 1$.

While it may look as a limiting assumption, it is not hard to ensure it in a real-life scenario by requiring each transaction to be sent to a randomly chosen set of nodes. An adversary could violate this assumption by inputting many transactions to the system in a skewed manner (i.e., sending them only to several chosen nodes), but such attack would be expensive to conduct (since usually a small fee is paid for each transaction) and hence each node with such artificially inflated buffer could be considered Byzantine.

Including transactions in units. Assuming such a model, each honest node randomly samples $\frac{1}{N}$ transactions from its buffer to be included in each new unit. Additionally, it removes transactions that are already included in units of its local copy of ch-DAG from its buffer.

DAG-rounds and async-rounds. There are two different notions of a "round" used throughout the paper - DAG-round as defined in Section 3.1 and async-round, as defined by [19] and in Section G. While the DAG-round is more commonly used in the paper, it should be thought as a structural property of ch-DAGs and hence not necessarily a measure of passing time. Hence, to be in line with the literature, all proofs regarding latency are conducted in terms of async-rounds. The following lemma provides a useful onesided estimate of unit production speed in terms of async-rounds, roughly it says that DAG-rounds (asymptotically) progress at least as quickly as async-rounds.

LemmA D.1. Each honest node initializes ch-RBC for a newly produced unit at least once during each 5 consecutive async-rounds.

Proof. Let $\mathcal{P}_{i}$ be an honest node. We prove that if $\mathcal{P}_{i}$ have instantiated ch-RBC for unit of DAG-round $r_{d}$ at async-round $r_{a}$ then it necessarily instantiates it again (for a different unit) before async-round $r_{a}+5$ ends. We show it by induction on $r_{d}$.

If $r_{d}=1$, then necessarily $r_{a}=1$ since every node is required to produce a unit in its first atomic step. Then, by Lemma F.1(Latency), $\mathcal{P}_{i}$ will have at least $2 f+1$ units of DAG-round 1 in its local copy of ch-DAG by the end of async-round 4 and hence will be required to instantiate ch-RBC at that time, what concludes the proof in this case.

Now assume $r_{d}>1$. To produce a unit of DAG-round $r_{d}$, there must have been at least $2 f+1$ units of DAG-round $r_{d}-1$ in $\mathcal{D}_{i}$. Then at least $2 f+1$ instances of ch-RBC for units of that DAGround have had to locally terminate for $\mathcal{P}_{i}$ before $r_{a}$. Hence, by Lemma F.1(Fast Agreement), we get that every honest node needs to have at least $2 f+1$ units of DAG-round $r_{d}-1$ in its local copy of ch-DAG before async-round $r_{a}+2$ ends. Consequently, each honest node is bound to instantiate ch-RBC for its unit of DAG-round $r_{d}$ or higher before async-round $r_{a}+2$ ends. We consider the following two cases.

Case 1. Every honest node has instantiated ch-RBC for a unit of DAG-round $r_{d}$ before async-round $r_{a}+2$ ended. Then, every honest node (including $\mathcal{P}_{i}$ ) will locally output the result of this ch-RBC instances before DAG-round $r_{d}+5$ ends, by Lemma F.1(Latency).

Hence, $\mathcal{P}_{i}$ is bound to instantiate ch-RBC for a unit of DAG-round at least $r_{d}+1$ before async-round $r_{a}+5$ ends.

Case 2. If Case 1 does not hold, then by the above, at least one honest node has instantiated ch-RBC for a unit of DAG-round $r_{d}^{\prime}>r_{d}$ before async-round $r_{a}+2$ ended. Then, again using Lemma F.1(Fast Agreement), we conclude that every honest node (including $\mathcal{P}_{i}$ ) needs to hold at least $2 f+1$ units of DAG-round $r_{d}^{\prime}-1 \geqslant r_{d}$ before async-round $r_{a}+4$ ends. Then $\mathcal{P}_{i}$ is bound to instantiate ch-RBC for a unit of DAG-round at least $r_{d}+1$ before async-round $r_{a}+4$ ends.

Next, we formulate a lemma connecting the concept of async-round and the transaction arrival model described on the beginning of this section.

LemmA D.2. If a transaction $t x$ is input to at least $k$ honest nodes during async-round $r_{a}$ then it is placed in an honest unit in asyncround $r_{a}+O\left(\frac{N}{k}\right)$.

Proof. By Lemma D. 1 each of the $k$ honest nodes with $t x$ in buffer is bound to produce at least one unit per 5 async-rounds. Since the honest nodes are sampling transactions to be input to units independently, the probability of $t x$ being input to at least one unit during any quintuple of async-rounds after $r_{a}$ is at least

$$
1-\left(1-\frac{1}{N}\right)^{k}>1-\left(1-\frac{k}{N}+\frac{k^{2}}{2 N^{2}}\right)=\frac{k}{N}-\frac{k^{2}}{2 N^{2}}>\frac{k}{2 N}
$$

by extended Bernoulli's inequality, since $\frac{k}{N} \leqslant 1$.

Hence the expected number of async-rounds before $t x$ is included in a unit is $O\left(\frac{N}{k}\right)$.

We now proceed to proving two technical lemmas that are required to show that, on average, every transaction input to the system is placed in a unit only a small (constant) number of times. We begin with a simple probability bound that is then used in the second lemma.

LEMMA D.3. Let $n, m \in \mathbb{N}$ and suppose that $Q_{1}, Q_{2}, \ldots, Q_{n}$ are finite sets with $\left|Q_{i}\right| \geqslant m$ for every $i=1,2, \ldots, n$. For every $i=$ $1,2, \ldots, n$ define a random variable $S_{i} \subseteq Q_{i}$ that is obtained by including independently at random each element of $Q_{i}$ with probability $\frac{1}{n}$. If $m \geqslant 48 n$ then

$$
P\left(\min _{T \subseteq[n],|T|=n / 3}\left|\bigcup_{i \in T} S_{i}\right| \leqslant \frac{m}{6}\right) \leqslant e^{-n}
$$

Proof. For convenience denote $Q=\bigcup_{i \in[n]} Q_{i}$. Let us fix any subset $T \subseteq[n]$ of size $n / 3$ and denote $S^{T}=\bigcup_{i \in T} S_{i}$. For every element $q \in Q$ define a random variable $Y_{q} \in\{0,1\}$ that is equal to 1 if $q \in S^{T}$ and 0 otherwise. Under this notation, we have

$$
\left|S^{T}\right|=\sum_{q \in Q} Y_{q}
$$

We will apply Chernoff Bound to upper bound the probability of

$$
\sum_{q \in Q} Y_{q} \leqslant \frac{m}{6}
$$

For this, note first that $\left\{Y_{q}\right\}_{q \in Q}$ are independent. Moreover, we have $\mathbb{E}\left[\sum_{q \in Q} Y_{q}\right] \geqslant \frac{m}{3}$ from linearity of expectation. Hence from the Chernoff Bound we derive

$$
P\left(\left|S^{T}\right| \leqslant \frac{m}{6}\right)=P\left(\sum_{q \in Q} Y_{q} \leqslant \frac{m}{6}\right) \leqslant e^{-\frac{m}{24}} \leqslant e^{-2 n}
$$

Finally, by taking a union bound over all (at most $2^{n}$ many) sets $T$ of size $n / 3$ we obtain

$$
P\left(\min _{T \subseteq[n],|T|=n / 3}\left|S^{T}\right| \leqslant \frac{m}{6}\right) \leqslant 2^{n} \cdot e^{-2 n}=e^{-n}
$$

LemmA D.4. Let $T_{R}$ be the number of, not necessarily different, transactions (i.e., counted with repetitions) input to units during the first $R=\operatorname{poly}(N)$ rounds of the protocol, and $T_{R}^{\prime}$ be the number of pairwise different transactions in these units, then $T_{R}=O\left(T_{R}^{\prime}+R N\right)$ except with probability $e^{-\Omega(N)}$.

Proof. For every round $r$ denote by $B_{r}$ the common (up to a constant) size of this round's buffer of any honest node. Since every unit contains at most $O\left(B_{r} / N\right)$ transactions in round $r$, we have that the total number of transactions input in units this round is $O\left(B_{r}\right)$ and thus $T_{R}=O\left(\sum_{r=0}^{R} B_{r}\right)$.

From Lemma E. 2 at every round $r$, there exists a set $\mathcal{S}_{r}$ of units at round $r$ such that every unit of round at least $r+3$ is above every unit in $\mathcal{S}_{r}$. Let also $\mathcal{H}_{r} \subseteq \mathcal{S}_{r}$ be any subset of at least $f+1$ honest units and $\mathcal{T}_{r}$ be the set of all transactions in $\mathcal{H}_{r}$. Since for any $r_{1}, r_{2}$ such that $r_{2} \geqslant r_{1}+3$ we have $\mathcal{H}_{r_{1}} \geqslant \mathcal{H}_{r_{2}}$ and thus $\mathcal{T}_{r_{1}} \cap \mathcal{T}_{r_{2}}=\emptyset$. In particular

$$
T_{R}^{\prime} \geqslant\left|\bigcup_{r=0}^{R} \mathcal{T}_{r}\right| \geqslant \frac{1}{3} \sum_{r=0}^{R}\left|\mathcal{T}_{r}\right|
$$

From Lemma D. 3 we obtain that unless $B_{r}<48 N$ it holds that $\left|\mathcal{T}_{r}\right| \geqslant \frac{B_{r}}{6}$ with probability $1-e^{-N}$. Thus finally, we obtain that with high probability (no matter how large $B_{r}$ is) $\left|\mathcal{T}_{r}\right| \geqslant \frac{B_{r}}{6}-8 N$, and consequently

$$
T_{R}^{\prime} \geqslant \frac{1}{3} \sum_{r=0}^{R}\left|\mathcal{T}_{r}\right| \geqslant \frac{1}{18} \sum_{r=0}^{R} B_{r}-\frac{8}{3} R N
$$

and therefore

$$
T_{R}=O\left(\sum_{r=0}^{R} B_{r}\right)=O\left(T_{R}^{\prime}+R N\right)
$$

Proof of Theorem 2.1.

Total Order and Agreement. It is enough to show that all the honest nodes produce a common total order of units in ch-DAG. Indeed, the transactions within units can be ordered in an arbitrary yet deterministic way, and then a consistent total order of units trivially yields a consistent total order of transactions. The total order is deterministically computed based on the function ChooseHead applied to every round, which is consistent by Lemma C.5(Agreement).

Censorship Resilience and Latency. By Lemma D. 2 we obtain that each transaction input to $k$ honest nodes is placed in a unit $U$ created by some honest node $\mathcal{P}_{i}$ after $O(N / k)$ async-rounds.

By Lemma F.1(Latency) each honest node receives $U$ via ch-RBC within 3 async-rounds from its creation and hence will append it as a parent of its next unit, which will be created within 5 asyncrounds by Lemma D. 1 and added to ch-DAGs of all honest nodes within next 3 async-rounds, again by Lemma F.1(Latency). Thus, after $O\left(\frac{N}{k}\right)$, the unit $U$ containing $t x$ is below all maximal units created by honest nodes. Denote by $r_{d}$ the maximum DAG - round over all honest nodes at that moment and by $r_{a}$ the currenct asyncround.

Since each unit has at least $2 f+1$ parents, it follows that every unit of round $r_{d}+1$ (even created by an dishonest node) is above $U$. For this reason, the head of round $r_{d}+1$ is also above $U$ and in particular the unit $U$ is ordered at latest by the time when the head of this round is chosen. Note that by Lemma C.5(latency), the head at round $r_{d}+1$ is determined after only $O(1)$ DAG-rounds. Combining this fact with Lemma D. 1 it follows that after $5 \cdot O(1)$ async-round each honest node can determine the head from its version of the poset. We obtain that the total latency is bounded by

$$
O(N / k)+O(1) \cdot O(1)=O(N / k)
$$

Communication Complexity. Let $t_{r}$ be the number of transactions that have been included in honest units at level $r$. By Lemma F.1(Communication Complexity) we obtain that the total communication complexity utilized by instances of ch-RBC over any sequence of $R$ DAG-rounds is

$$
O\left(N \sum_{r=1}^{R} t_{r}+R N^{2} \log N\right)=O\left(T_{R}+R N^{2} \log N\right)
$$

where $T_{R}$ is the total number of transactions included in units at all rounds $r=0,1, \ldots, R$. From Lemma D. 4 we conclude that $T_{R}=O\left(T_{R}^{\prime}+R N\right)$ where $T_{R}^{\prime}$ is the number of distinct transactions in these units. The final bound on communication complexity is

$$
O\left(T_{R}^{\prime}+R N^{2} \log N\right)
$$

\section{E RANDOMNESS BEACON}

The main goals of this section are to show that ABFT-Beacon implements a Randomness Beacon (thus providing a proof of Theorem 2.2), and to prove correctness of two implementations of SecretBits that are provided in Section 4, i.e., prove Lemma 4.1 and Lemma 4.2.

Let us now briefly summarize what is already clear and what is left to prove regarding the randomness beacon.

(1) Every node $\mathcal{P}_{i}$ reliably broadcasts its key box $K B_{i}$ (in $U[i ; 0]$ ) that contains a commitment to a polynomial $A_{i}$ (of degree at most $f$ ) and encrypted tossing keys for all the nodes. Some of the tossing keys might be incorrect if $\mathcal{P}_{i}$ is dishonest.

(2) For every $i \in[N]$ the unit $U[i ; 6]$ defines a set $T_{i} \subseteq[N]$ and the $i$ th MultiCoin is defined as a a threshold signature scheme as if it were generated using the polynomial

$$
B_{i}(x)=\sum_{j \in T_{i}} A_{j}(x)
$$

(3) A threshold signature with respect to MutliCoin ${ }_{i}$, i.e., $m^{B_{i}(0)}$ for some $m \in G$ can be generated by simply multiplying together the threshold signatures $m^{A_{j}(0)}$ for $j \in T_{i}$.

(4) Thus, on a high level, what remains to prove is that:
- For every $i \in[N]$ such that $U[i ; 6]$ is available, it is possible for the nodes to commonly generate threshold signatures with respect to key set $K S_{j}$ for every $j \in T_{i}$.
- For every $i \in[N]$ the key $B_{i}(0)$ remains secret, or more precisely, the adversary cannot forge signatures $m^{B_{i}(0)}$ for a randomly chosen $m \in G$.

Below, we proceed to formalizing and proving these properties.

Properties of Key Sets. From now on we will use the abstraction of Key Sets to talk about pairs (TK, VK) where

$$
\begin{aligned}
& T K=(A(1), A(2), \ldots, A(N)) \\
& V K=\left(g^{A(1)}, g^{A(2)}, \ldots, g^{A(N)}\right)
\end{aligned}
$$

for some degree $\leqslant f$ polynomial $A \in \mathbb{Z}_{q}[x]$. In the protocol, the vector $V K$ is always publicly known (computed from a commitment or several commitments) while $T K$ is distributed among the nodes. It can be though distributed dishonestly, in which case it might be that several (possibly all) honest nodes are not familiar with their corresponding tossing key. In particular, every Key Box defines a key set and every MultiCoin defines a key set.

In the definition below we formalize what does it mean for a key set to be "usable" for generating signatures (and thus really generating randomness)

Definition E. 1 (Usable Key Set). We say that a Key Set KS $=$ (TK, VK) is usable with a set of key holders $T \subseteq[N]$ if:

(1) every honest node is familiar with $V K$ and $T$,

(2) $|T| \geqslant 2 f+1$,

(3) for every honest node $\mathcal{P}_{i}$, if $i \in T$, then $i$ holds $t k_{i}$.

The next lemma demonstrates that indeed a usable key set can be used to generate signatures.

Lemma E.1. Suppose that $(T K, V K)$ is a usable key set, then if the nodes being key holders include the shares for a nonce $m$ in the ch-DAG at round $r$, then every honest node can recover the corresponding signature $\sigma_{m}$ at round $r+1$.

Proof. Since the key set is usable, we know that there is exactly one underlying polynomial $A \in \mathbb{Z}_{q}[x]$ of degree $f$ that was used to generate ( $T K, V K$ ), hence every unit at round $r$ created by a share dealer $i \in T$ can be verified using VerifyShare and hence it follows that any set of $f+1$ shares decodes the same value $\sigma_{m}=m^{A(0)}$ using ExtractShares (see also [6]). It remains to observe that among a set of (at least $2 f+1$ ) round- $r$ parents of a round- $(r+1)$ unit, at least $f+1$ are created by nodes in $T$ and thus contain appropriate shares.

Generating Signatures using MultiCoins. We proceed to proving that the nodes can generate threshold signatures with respect
to MultiCoins. The following structural lemma says, intuitively, that at every round $r$ there is a relatively large set of units that are below every unit of round $r+3$.

Lemma E. 2 (Spread). Assume that an honest node $\mathcal{P}_{i}$ has just created a unit $U$ of round $r+3$. At this moment, a set $\mathcal{S}_{r}$ of $2 f+1$ units of round $r$ is determined such that for every unit $V$ of round $r+3$ and for any unit $W \in \mathcal{S}_{r}$, we have $W \leqslant V$.

Proof. Let $K$ be a set of $2 f+1$ nodes that created units in $\downarrow(U)$. Let $\mathcal{T}$ be a set of $2 f+1$ units of round $r+1$ created by nodes in $K$. Every unit in $\downarrow(U)$ is above $2 f+1$ units of round $r+1$, hence it is above at least $f+1$ units in $\mathcal{T}$ (the remaining $f$ units may be created by nodes outside of $K$ ). By the pigeonhole hole principle there exists a unit $U_{0} \in \mathcal{T}$ such that at least $f+1$ units in $\downarrow(U)$ are above it. Thus, every subset of $2 f+1$ units of round $r+2$ must include at least one unit above $U_{0}$, so every valid unit $V$ of round $r+3$ will have at least one parent above $U^{\prime}$, thus $V \geqslant U^{\prime}$. Finally, choose $\mathcal{S}_{r}:=\downarrow(V)$.

We now proceed to proving a structural lemma that confirms several claims that were made in Subsection 4.3 without proofs. We follow the notation of Subsection 4.3.

Lemma E.3. Let $T_{i}$ for $i \in[N]$ be the sets of indices of Key Sets forming the ith MultiCoin, then

(1) For every $i$ such that $U[i ; 6]$ exists, $T_{i}$ has at least $f+1$ elements.

(2) For every $i \in[N]$ and every $j \in T_{i}$, the Key Set $K S_{j}$ is usable.

(3) For every unit $V$ of round $r+1$ with $r \geqslant 6$, there exists a set of $f+1$ units in $\downarrow(V)$ such that they contain all shares required to reconstruct all Key Sets that are below $V$.

Proof. Let $\mathcal{D}$ here be the local view of an honest node that has created a unit of round at least 6 . From Lemma E. 2 we obtain sets $\mathcal{S}_{0}$ and $\mathcal{S}_{3}$, which are in $\mathcal{D}$ because it includes at least one unit of round 6 .

For any $i \in[N]$ such that $U[i ; 6]$ exists in $\mathcal{D}$, the set $T_{i}$ is defined as a function of the unit $U[i ; 6]$ and all units below it, therefore it is consistent between honest nodes. Additionally, $U[i ; 6]$ is above at least $f+1$ units from $\mathcal{S}_{0}$ created by honest nodes ( $2 f+1$ units minus at most $f$ created by malicious nodes), and creators of those units must be included in $T_{i}$. Indeed, if a unit of round zero was created by an honest node, then it is valid, and every round-3 unit must vote 1 on it, as false-negative votes cannot be forged.

From now on, let $T$ denote the union of all $T_{i} \mathrm{~s}$ that are well defined (i.e., exist) in $\mathcal{D}$. Every unit $W$ in the set $\mathcal{S}_{3}$ must vote 1 on every Key Box $K B_{l}$ for $l \in T$, because $W$ is seen by every unit of round 6. Since $\left|\mathcal{S}_{3}\right| \geqslant 2 f+1$ and due to the definition of votes, we see that every Key Set $K S_{l}$ for $l \in T$ is usable by (possibly a superset of) creators of $\mathcal{S}_{3}$.

If we take the intersection of the set of creators of $\mathcal{S}_{3}$, and the set of $\downarrow(V)$ creators, then the resulting set contains at least $f+1$ nodes. Units in $\downarrow(V)$ created by these $f+1$ nodes must contain valid shares from round $r$ to all Key Sets created by nodes from $T$. Indeed, this is required by the protocol, since these nodes declared at round 3 that they will produce valid shares (recall that $f+1$ valid shares are enough to read the secret).
Finally we remark that the node who holds $\mathcal{D}$ might not be familiar with $\mathcal{S}_{3}$, but, nevertheless, it can find at least $f+1$ units in $\downarrow(V)$ containing all required shares, because it knows $T$.

In part (3) of the above lemma we show that we can always collect shares from a single set of $f+1$ units, which yields an optimization from the viewpoint of computational complexity. The reason is that each round of the setup requires generating $O(N)$ threshold signatures, but if we can guarantee that each signature has shares generated by the same set of $f+1$ nodes, then we may compute the Lagrange coefficients for interpolation only once (as they depend only on node indices that are in the set) thus achieve computational complexity $O\left(N^{3}\right)$ instead of $O\left(N^{4}\right)$ as in the basic solution.

Security of MultiCoins. The remaining piece is unpredictability of multicoins, which is formalized below as

LemmA E.4. If a polynomially bounded adversary is able to forge a signature generated using MultiCoin $i_{0}$ (for any $i_{0} \in[N]$ ) for a fresh nonce $m$ with probability $\varepsilon>0$, then there exists a polynomial time algorithm for solving the computational Diffe-Hellman problem over the group $G$ that succeeds with probability $\varepsilon$.

Proof. We start by formalizing our setting with regard to the encryption scheme that is used to encrypt the tossing keys in key boxes. Since it is semantically secure we can model encryption as a black box. More specifically we assume that for a key pair ( $s k, p k$ ), an adversary that does not know $s k$ has to query a black box encryption oracle $\operatorname{Enc}_{p k}(d)$ to create the encryption of a string $d$ for this public key. Further, we assume that whenever a string $s$ is fed into the decryption oracle $\operatorname{Dec}_{s k}(\cdot)$ such that $s$ was not obtained by calling the encryption oracle by the adversary, then the output $\operatorname{Dec}_{s k}(\cdot)$ is a uniformly random string of length $|s|$. Since the key pairs we use in the protocol are dedicated, the adversary has no way of obtaining ciphertexts that were not created by him, for the key sets he is forced to use.

Given this formalization we are ready to proceed with the proof. We fix any $i_{0} \in[N]$ and consider the MultiCoin $i_{0}$. We use a simulation argument (see e.g. [43]) to show a reduction from forging signatures to solving the Diffie-Hellman problem. To this end, let $\left(g^{\alpha}, g^{\beta}\right)$ be a random input to the Diffie-Hellman problem, i.e., the goal is to compute $g^{\alpha \beta}$.

Suppose that an adversary can forge a signature of a random element $h \in G$ with probability $\varepsilon>0$ when the honest nodes behave honestly. We now define a simulated run of the protocol (to distinguish from an honest run in which all honest node behave honestly) which is indistinguishable from an honest run from the perspective of the adversary, yet we can transform a successful forgery into a solution of the given Diffie-Hellman triplet.

Description of the simulated run. For convenience assume that the nodes controlled by the adversary have indices $1,2, \ldots, f$. At the very beginning, every honest node $i \in\{f+1, f+2, \ldots, N\}$ generates uniformly random values $r_{i, 0}, r_{i, 1}, \ldots, r_{i, f} \in \mathbb{Z}_{q}$ and defines a polynomial $A_{i}$ of degree at most $f$ by specifying its values
at $f+1$ points as follows:

$$
\begin{aligned}
& A_{i}(0)=\alpha+r_{i, 0} \\
& A_{i}(1)=r_{i, 1} \\
& \vdots \\
& A_{i}(f)=r_{i, f}
\end{aligned}
$$

where $\alpha \in Z_{q}$ is the unknown value from the Diffie-Hellman instance. Note that since $\alpha$ is unknown the $i$ th node has no way to recover the polynomial $A_{i}(x)=\sum_{j=0}^{f} a_{i, j} x^{j}$ but still, it can write each of the coefficients in the following form

$$
a_{i, j}=a_{i, j, 0}+\alpha a_{i, j, 1} \quad \text { for } j=0,1, \ldots, f
$$

where $a_{i, j, 0}, a_{i, j, 1} \in \mathbb{Z}_{q}$ are known values. Since $g^{\alpha}$ is known as well, this allows the $i$ th node to correctly compute the commitment

$$
C_{i}=\left(g^{a_{i, 0}}, g^{a_{i, 1}}, \ldots, g^{a_{i, f}}\right)
$$

pretending that he actually knows $A_{i}$. Similarly he can forge a key box $K B_{i}$ so that from the viewpoint of the adversary it looks honest. This is the case because $\mathcal{P}_{i}$ knows the tossing keys for the adversary $t k_{1}, t k_{2}, \ldots, t k_{f}$ since these are simply $r_{i, 1}, r_{i, 2}, \ldots, r_{i, f}$ Thus $\mathcal{P}_{i}$ can include in $E_{i}$ the correct values $e_{i, 1}, e_{i, 2}, \ldots, e_{i, f}$ and simply fill the remaining slots with random strings of appropriate length (since these are meant for the remaining honest nodes which will not complain about wrong keys in the simulation).

In the simulation all the honest nodes are requested to vote positively on key boxes created by honest nodes, even though the encrypted tossing keys are clearly not correct. For the key boxes dealt by the adversary the honest nodes act honestly, i.e., they respect the protocol and validate as required. Suppose now that the setup is over and that the honest nodes are requested to provide shares for signing a particular nonce $m$. The value of the hash $h=\operatorname{hash}(m) \in G$ is sampled in the simulation as follows: first sample a random element $\delta_{m} \in \mathbb{Z}_{q}$ and subsequently set

$$
\operatorname{hash}(m)=g^{\delta_{m}}
$$

Since in the simulation the value $\delta_{m}$ is revealed to the honest nodes, the honest node $\mathcal{P}_{j}$ can compute its share corresponding to the $i$ th key set (for $i \geqslant f+1$ ) as

$$
h^{A_{i}(j)}=\left(g^{\delta_{m}}\right)^{A_{i}(j)}=\left(g^{A_{i}(j)}\right)^{\delta_{m}}
$$

where the last expression can be computes since $g^{A_{i}(j)}$ is $v k_{i, j}$ (thus computable from the commitment) and $\delta_{m}$ is known to $j$.

Solving Diffie-Hellman from a forged signature. After polynomially many rounds of signing in which the hash function is simulated to work as above, we let the hash function output $g^{\beta}$ (the element from the Diffie-Hellman input) for the final nonce $m^{\prime}$ for which the adversary attempts forgery. The goal of the adversary is now to forge the signature, i.e., compute $\left(g^{\beta}\right)^{A(0)}$, where

$$
A(0)=\sum_{i \in T_{i_{0}}} A_{i}(0)
$$

is the cumulative secret key for the multicoin MultiCoin $i_{0}$. For that, the adversary does not see any shares provided by honest nodes. Note that if we denote by $H$ and $D$ the set of indices of honest and dishonest nodes (respectively) within $T_{i_{0}}$ then the value of $A(0)$ is of the following form

$$
\begin{aligned}
A(0) & =\sum_{i \in T_{i_{0}}} A_{i}(0) \\
& =\sum_{i \in H} A_{i}(0)+\sum_{i \in D} A_{i}(0) \\
& =|H| \cdot \alpha+\sum_{i \in H} r_{i, 0}+\sum_{i \in D i} A_{i}(0)
\end{aligned}
$$

We observe that each value $A_{i}(0)$ for $i \in D$ can be recovered by honest nodes, since at least $f+1$ honest nodes must have validated their keys when voting on $K B_{i}$ and thus honest nodes collectively know at least $f+1$ evaluations of $A_{i}$ and thus can interpolate $A_{i}(0)$ as well. Similarly, the values $r_{i, 0}$ for $i \in H$ are known by honest nodes. Finally we observe that since by Lemma E. 3 (a), $\left|T_{i}\right| \geqslant f+1$, it follows that $|H|>0$. Thus consequently, we obtain that $A(0)$ is of the form

$$
A(0)=\gamma \alpha+\mu
$$

where $\gamma, \mu \in \mathbb{Z}_{q}$ are known to the honest nodes and $\gamma \neq 0$. Hence, the signature forged by the adversary has the form

$$
\left(g^{\beta}\right)^{A(0)}=g^{\gamma \alpha \beta+\beta \mu}=\left(g^{\alpha \beta}\right)^{\gamma} \cdot\left(g^{\beta}\right)^{\mu}
$$

and can be used to recover $g^{\alpha \beta}$ since $\gamma, \mu$ and $g^{\beta}$ are known.

The above reasoning shows that we can solve the Diffie-Hellman problem given an adversary that can forge signatures; however to formally conclude that, we still need to show that the view of the adversary in such a simulated run is indistinguishable from an honest run.

Indistinguishability of honest and simulated runs. We closely investigate all the data that is emitted by honest nodes throughout the protocol execution to see that these distributions are indistinguishable between honest and simulated runs. Below we list all possible sources of data that adversary observes by listening to honest nodes.

(1) The key boxes in an honest run and a simulated run look the same to the adversary, as they are always generated from uniformly random polynomials, and the evaluations at points $1,2, \ldots, f$ are always honestly encrypted for the nodes controlled by the adversary. The remaining encrypted keys are indistinguishable from random strings and thus the adversary cannot tell the difference between honest runs and simulated runs, since the values encrypted by honest nodes are never decrypted.

(2) The votes (i.e., the VerKey $(\cdot, \cdot)$ values) of honest nodes on honest nodes are always positive, hence the adversary does not learn anything here.

(3) The shares generated by honest nodes throughout the execution of the protocol are always correct, no matter whether the run is honest or simulated (in which case the honest nodes do not even know their keys).

(4) The votes of honest nodes on dishonest key boxes: this is by far the trickiest to show and the whole reason why we need dedicated key pairs for encryption in key boxes. We argue that the adversary can with whole certainty predict
the votes of honest nodes, hence this piece of data is insignificant: intuitivily the adversary cannot discover that he "lives in simulation" by inspecting this data. Here is the reason why the adversary learns nothing from these votes. There are two cases:

(a) a dishonest node $i \in[f]$ includes as $e_{i, j}$ some ciphertext that was generated using the encryption black box $\operatorname{Enc}_{i \rightarrow j}$. That means that the adversary holds a string $d$ such that $e_{i, j}=\operatorname{Enc}_{i \rightarrow j}(d)$. In this case the adversary can itself check whether $d$ is the correct tossing key. Thus the adversary can predict the vote of $\mathcal{P}_{j}$ : either it accepts if $d$ is the correct key, or it rejects in which case it reveals $\operatorname{Dec}_{i \rightarrow j}\left(e_{i, j}\right)=d$.

(b) a dishonest node $i \in[f]$ includes as $e_{i, j}$ some string that was never generated by the encryption black box $\mathrm{Enc}_{i \rightarrow j}$. In this case, the decryption of $e_{i, j}$ is simply a random string and thus agrees with the key only with negligible probability. Thus the adversary can tell with whole certainty that the vote on $e_{i, j}$ will be negative and he only learns a random string $\operatorname{Dec}_{i \rightarrow j}\left(e_{i, j}\right)$.

All the remaining data generated by honest nodes throughout the execution of the protocol is useless to the adversary as he could simulate it locally. This concludes the proof of indistinguishability between honest and simulated runs from the viewpoint of the adversary.

Proofs regarding SecretBits and ABFT-Beacon. We are now ready to prove Lemma 4.2 , i.e. show that the implementation of SecretBits with MultiCoin during the ABFT-Beacon setup satisfies the Definition 3.3 of SecretBits.

Proof of Lemma 4.2. Let $\mathcal{P}_{k}$ be any honest node. We start by showing that during the run of the ABFT - Beacon setup if $\mathscr{P}_{k}$ invokes SecretBits $(i, r)$, then $r \geqslant 10$ and the unit of round 6 created by $\mathcal{P}_{i}$ is already present in the local copy $\mathcal{D}_{k}$.

All SecretBits invocations happen inside ChooseHead(6) or, more precisely, in GeneratePermutation $(6, \mathcal{D})$ and CommonVote $(U, \cdot)$, for units $U$ of round 6 . Let $\mathcal{U}$ be a set of units of round 6 present in $\mathcal{D}_{k}$ at the time of any particular call to ChooseHead(6). Additionally, let $C$ be the set of all nodes that created units in $\mathcal{U}$. The procedure GeneratePermutation $(6, \mathcal{D})$ calls SecretBits $(i, 6+4)$ for all $i \in C$, and function CommonVote $(U, V)$ is deterministic if $\mathrm{R}(V) \leqslant \mathrm{R}(U)+4$ and hence calls only SecretBits $(i, k)$, where $i$ is $U$ 's creator in $C$ and $k \geqslant 10$.

Thus, from the above, the function SecretBits will be invoked sufficiently late, hence all units of round 6 defining required MultiCoins will be already visible at that time. Lemma E. 3 guarantees that then the MultiCoin can be used to generate signatures since all the keys sets comprising it are usable. The secrecy follows from the fact that the signatures generated by MultiCoins are unpredictable for the adversary (Lemma E.4).

The above proof justifies why is it easier to implement SecretBits as a function of two arguments, instead of only using the round number.

The next lemma shows that by combining key sets as in our construction, we obtain a single key set that is secure, i.e., that there are enough nodes holding tossing keys and that the generated bits are secret.

Given the above lemma, we are ready to conclude the proof of Theorem 2.2.

Proof of Theorem 2.2. From Lemma 4.2 we obtain that the Setup phase of ABFT - Beacon executes correctly and a common Key Set $(T K, V K)$ is chosen as its result. Moreover, by Lemma E. 4 the output Key Set is secure and thus can be used to generate secrets. Consequently, Setup + Toss indeed implement a Randomness Beacon according to Definition 2.2.

Communication Complexity. What remains, is to calculate the communication complexity of the protocol. Observe that during the setup, every unit might contain at most:
- $O(N)$ hashes of parent nodes,
- $O(N)$ verification keys,
- $O(N)$ encrypted tossing keys,
- $O(N)$ votes,
- $O(N)$ secret shares.

Since ChooseHead builds only an expected constant number of DAG-rounds before terminating (see Lemma C.5), every node reliably broadcasts $O(1)$ units during the setup in expectation each of which of size $O(N)$, thus the total communication complexity per node is $O\left(N^{2} \log N\right)$.

Latency. Assume that honest nodes decide to instantiate Toss for a given nonce $m$ during async-round $r_{a}$. Then, each honest node multicasts its respective share before async-round $r_{a}+1$ ends, thus all such shares are bound to be delivered before the end of asyncround $r_{a}+2$. Thus, each honest node may reveal the value of a Toss after at most 2 async-round, what concludes the proof.

The last remaining piece is to prove Lemma 4.1 that claims correctness of the main implementation of SecretBits.

Proof of Lemma 4.1. The case with a trusted dealer follows for instance from [6]. Consider now the harder case: when the Setup of ABFT - Beacon is used to deal the keys. Then, the nodes end up holding a Key Set that is secure by Lemma E. 4 and can be used for signing messages by Lemma E.1.

\section{F RELIABLE BROADCAST}

In this section we describe ch-RBC - a modified version of Reliable Broadcast (RBC) protocol introduced by [9] and improved by [17] by utilizing erasure codes. The protocol is designed to transfer a message from the sender (who instantiates the protocol) to the set of receiving nodes in such a way that receivers can be certain that they all received the same version of the message. More formally, a protocol implements Reliable Broadcast if it meets the following conditions:

(1) Termination. If an honest node instantiates RBC, all honest nodes eventually terminate.

(2) Validity. If an honest node instantiates RBC with a message $m$, then any honest node can only output $m$.

(3) Agreement. If an honest node outputs a message $m$, eventually all honest nodes output $m$.

We implement four rather straightforward modifications to the original RBC protocol. First, before multicasting prevote message
each node checks the size of the proposed unit (line 10) to prevent the adversary from sending too big units and hence consuming too much bandwidth of honest nodes ${ }^{24}$. Second, before multicasting prevote, we wait until ch-DAG of a node reaches at least one round lower then round of prevoted unit (line 11). This check counters an attack in which adversary initializes ch-RBC for meaningless pieces of data of rounds which do not exist in the ch-DAG yet, hence wasting bandwidth of honest nodes. Third, after reconstructing a unit $U$, we check if $U$ is valid (line 16), i.e., we check if it has enough parents, if one of its parents was created by $U$ 's creator, and if the data field of $U$ contains all required components. Finally before multicasting commit message for unit $U$, nodes wait until they receive all parents of $U$ via RBC (line 17) to ensure that it will be possible to attach $U$ to ch-DAG ${ }^{25}$.

The following lemma summarizes the most important properties of ch-RBC protocol. Note that Latency is a strictly stronger condition then Termination, and similarly Fast Agreement is stronger then Agreement. Thus, in particular, this lemma proves that ch-RBC implements Reliable Broadcast.

Lemma F.1. (reliable broadcast) For any given round $r$, all ch-RBC runs instantiated for units of that round have the following properties:

(1) Latency. ch-RBC instantiated by an honest node terminates within three async-rounds for all honest nodes.

(2) Validity If an honest node outputs a unit $U$ in some instance of ch-RBC, $U$ had to be proposed in that instance.

(3) Fast Agreement. If an honest node outputs a unit $U$ at async-round $r_{a}$, all honest nodes output $U$ before asyncround $r_{a}+2$ ends,

(4) Message Complexity. In total, all the ch-RBC instances have communication complexity ${ }^{26}$ at most $O\left(t_{r}+N^{2} \log N\right)$ per node, where $t_{r}$ is the total number of transactions input in honest units of round $r$.

Proof. Latency. Let $\mathcal{P}_{i}$ be an honest node instantiating ch-RBC for unit $U_{i}$ during the async-round $r$ so that the propose messages are sent during $r$ as well. Each honest node during the atomic step in which it receives the propose message multicasts the corresponding prevote. Since proposals were sent during async-round $r$, async-round $r+1$ cannot end until all of them are delivered, all honest nodes multicast prevotes during async-round $r+1$. Similarly, all prevotes sent by honest nodes need to reach their honest recipients during async-round $r+2$, hence honest nodes multicast their commits during async-round $r+2^{27}$. Finally, the commits of all honest nodes reach their destination before the end of asyncround $r+3$ causing all honest nodes to locally terminate at that time, what concludes the proof.
\footnotetext{
${ }^{24}$ Since in this version of RBC nodes learn about the content of broadcast message only after multicasting it as prevote, without this modification adversary could propose arbitrarily big chunks of meaningless data and hence inflate message complexity arbitrarily high.

${ }^{25}$ Note that this condition is not possible to check before multicasting prevote since nodes are not able to recover $U$ at that stage of the protocol

${ }^{26}$ We remark that $N^{2} \log (N)$ can be improved to $N^{2}$ in the communication complexity bound by employing RSA accumulators instead of Merkle Trees in the RBC protocol ${ }^{27} \mathrm{~A}$ cautious reader probably wonders about the wait in line 16 . It will not cause an execution of ch-RBC instantiated by an honest node to halt since before sending proposals during async-round $r_{a}$ a node must have had all $U$ 's parents output locally by other instances of ch-RBC, hence each other node will have them output locally by the end of async-round $r_{a}+2$, by condition Fast Agreement
}

```
ch-RBC:
    /* (Protocol for node $\mathcal{P}_{i}$ with sender $\mathcal{P}_{s}$
        broadcasting unit $U$ of round $r$ )
    */
    if $\mathcal{P}_{i}=\mathcal{P}_{s}$ then
        $\left\{s_{j}\right\}_{j \in N} \leftarrow$ shares of $(f+1, N)$-erasure coding of $U$
        $h \leftarrow$ Merkle tree root of $\left\{s_{j}\right\}_{j \in N}$
        for $j \in N$ do
            $b_{i} \leftarrow$ Merkle branch of $s_{j}$
            send propose $\left(h, b_{j}, s_{j}\right)$ to $\mathcal{P}_{j}$
    upon receiving propose $\left(h, b_{j}, s_{j}\right)$ from $\mathcal{P}_{s}$ do
        if received_propose $\left(\mathcal{P}_{i}, r\right)$ then
            terminate
        if check_size $\left(s_{j}\right)$ then
            wait until $\mathcal{D}_{i}$ reaches round $r-1$
            multicast prevote $\left(h, b_{j}, s_{j}\right)$
        received_propose $\left(\mathcal{P}_{i}, r\right)=$ True
    upon receiving $2 f+1$ valid prevote $(h, \cdot, \cdot)$ do
        reconstruct $U$ from $s_{j}$
        if $U$ is not valid then terminate
        wait until all $U$ 's parents are locally output by $\mathcal{P}_{i}$
        interpolate all $\left\{s_{j}^{\prime}\right\}_{j \in N}$ from $f+1$ shares
        $h^{\prime} \leftarrow$ Merkle tree root of $\left\{s_{j}^{\prime}\right\}_{j \in N}$
        if $h=h^{\prime}$ and $\operatorname{commit}\left(\mathcal{P}_{s}, r, \cdot\right)$ has not been send then
            multicast $\operatorname{commit}\left(\mathcal{P}_{s}, r, h\right)$
    upon receiving $f+1 \operatorname{commit}\left(\mathcal{P}_{s}, r, h\right)$ do
        if commit $\left(\mathcal{P}_{s}, r, \cdot\right)$ has not been send then
            multicast commit $\left(\mathcal{P}_{s}, r, h\right)$
    upon receiving $2 f+1 \operatorname{commit}\left(\mathcal{P}_{s}, r, h\right)$ do
        output $U$ decoded from $s j$ 's
    check_size ( $s_{j}$ ):
        $T \leftarrow$ number of transactions in $U$ inferred from $s_{j}$-th size
        /* $C_{B}$ is a global parameter of the protocol */
        if $T>C_{B} \cdot(i$-th batch size in round $r)$ then
            output False
        else output True
```

Validity. The first honest node to produce the commit message for a given unit $U$ has to do it after receiving $2 f+1$ precommit messages for $U$, out of which at least $f+1$ had to be multicast by other honest nodes. Since honest nodes are sending prevotes only for shares they received as proposals, $U$ had to be proposed by node initiating ch-RBC.

Fast Agreement. Assume that an honest node $\mathcal{P}_{i}$ has output a unit $U$ at async-round $r_{a}$ during some instance of ch-RBC. Since it can happen only after receiving $2 f+1$ commit messages for $U$, at least $f+1$ honest nodes had to multicast commit for $U$. Each honest node engages in ch-RBC for only one version of each node's unit for a given DAG-round, so it is not possible for any other node to output a unit different than $U$ for that instance of ch-RBC.

On the other hand, $\mathcal{P}_{i}$ must have received at least $f+1$ commit messages for $U$, which have been multicast before async-round $r_{a}$ ended. By definition of async-round, all of these commits must have reached their honest recipients before async-round $r_{a}+1$ ended.

Consequently, after gathering $f+1$ commits for $U$, all honest multicasted their commits before async-round $r_{a}+1$ ended. Thus, each honest node received commits from every other honest node before the end of async-round $r_{a}+2$, and terminated outputting $U$.

Message Complexity. Each honest node engages in at most $N$ ch-RBC instances for each DAG-round $r$, since it accepts only one proposal from each node. Let now $\mathcal{P}_{i}$ be an honest node with buffer of size $B_{r}$. Due to the condition in line $10, \mathcal{P}_{i}$ engages only in chRBC instances broadcasting units of size at most $C_{B} \frac{B_{r}}{N}$, where $C_{B}$ is the constant from the uniform buffer distribution assumption.

A single instance of ch-RBC proposing a unit $U$ has communication complexity of $O(\operatorname{size}(U)+N \log N)$, see [17] (here the communication complexity is computed per honest node). Now because of the unifform buffer distribution property we know that every valid unit of round $r$ includes between $C_{B}^{-1} \frac{B_{r}}{N}$ and $C_{B} \frac{B_{r}}{N}$ transactions (with $C_{B}=\Theta(1)$ ) thus if $U$ is such a unit, we have

$$
\operatorname{size}(U)=O\left(\frac{B_{r}}{N}+N\right)
$$

where the $+N$ comes from the fact that such a unit contains $O(N)$ parent hashes. Consequently the total communication complexity in round $r$ is upper bounded by

$$
\begin{aligned}
\sum_{i=1}^{N}(\operatorname{size}(U[i ; r])+N \log N) & =N \cdot O\left(\frac{B_{r}}{N}+N+N \log N\right) \\
& =O\left(B_{r}+N^{2} \log N\right) \\
& =O\left(t_{r}+N^{2} \log N\right)
\end{aligned}
$$

Where the last equality follows because the total number of transactions in honest units is $t_{r}=2 N / 3 \cdot \Theta\left(B_{r} / N\right)=\Theta\left(B_{r}\right)$.

General RBC with proofs of termination. Note that the above scheme differs from classical RBC only in four mentioned ch-DAGrelated aspects, hence after deleting additional checks it can be used to broadcast arbitrary pieces of data. Additionally, one simple modification can be done, which provides a proof that RBC has terminated for each node which had received the output locally. Namely, together with commit message, each node can send share of a threshold signature of hash $h$ with threshold $2 f+1$. Then, each node that gathered $2 f+1$ is able to construct the threshold signature and use it as a proof of the fact that RBC had finalized at its end.

\section{G ASYNCHRONOUS MODEL}

Slightly informally, the difference between synchronous an asynchronous network is typically explained as the absence of a global bound $\Delta$ on all message delays in asynchronous setting. In this paper, we use the following formalization adopted from Canetti and Rabin [19], where the adversary has a full control over the network delays, but is committed to eventually deliver each message to its recipient.

Definition G. 1 (Asynchronous Network Model). The execution of the protocol proceeds in consecutive atomic steps. In each

![](https://cdn.mathpix.com/cropped/2024_07_17_a7219064445e496da8a3g-33.jpg?height=518&width=835&top_left_y=275&top_left_x=1100)

Figure 1: A "Fork Bomb" for $K=3$. Only the top unit (marked $H$ ) is created by an honest node. Every collection of units within a dashed oval depicts forks produced by a certain dishonest node.

atomic step the adversary chooses a single node $\mathcal{P}_{i}$ which is then allowed to perform the following actions:
- read a subset S, chosen by the adversary, among all its pending messages. Messages in $S$ are considered delivered,
- perform a polynomially bounded computation,
- send messages to other nodes; they stay pending until delivered.

Thus, in every atomic step one of the nodes may read messages, perform computations, and send messages. While such a model may look surprisingly single-threaded, note that an order in which nodes act is chosen by an adversary, hence intuitively corresponds to the worst possible arrangements of network delays. The following definition by [19] serves as a main measure of time for the purpose of estimating latency of our protocols.

Definition G. 2 (Asynchronous Round). Let $l_{0}$ be an atomic step in which the last node is chosen by adversary to perform an action, i.e., after $l_{0}$ each node acted at least once. The asynchronous rounds of the protocol and the subsequent $l$ are defined as follows:
- All atomic steps performed after $l_{i-1}$ until (and including) $l_{i}$ have asynchronous round $i$,
- $l_{i}$ is the atomic step in which the last message sent during asynchronous round $l_{i-1}$ is delivered.

\section{H FORK BOMB ATTACK}

In this section we describe a resource-exhaustion attack scenario on DAG-based protocols. Generally speaking, this attack applies to protocols that allows adding nodes (which in our case are called units - we stick to this name for the purpose of explaining the attack) to the DAG performing only "local validation". By this we mean that upon receiving a unit, the node checks only that its parents are correct and that the unit satisfies some protocol-specific rules that are checked based only on the data included in this unit and on its parents (possibly recursively), and adds this unit to the DAG if the checks are successful. In particular, such a protocol has to allow adding forked units to the DAG, as it can happen that two distinct honest nodes built upon two distinct variants of a fork,
hence there is no way to roll it back. At this point we also remark that a mechanism of banning nodes that are proved to be forking (or malicious in some other way) is not sufficient to prevent this attack.

We note that while the QuickAleph protocol (without the alert system) satisfies the above condition, the Aleph protocol and the QuickAleph extended with the alert system, do not. This is because Aleph uses reliable broadcast to disseminate units (which gives a non-local validation mechanism) and similarly the alert system for QuickAleph adds new non-local checks before adding units created by forkers. On the other hand, protocols such as Hashgraph [4] or Blockmania [23] satisfy this condition and thus are affected, if not equipped with an appropriate defense mechanism.

The main idea of the attack is for a group of malicious nodes to create an exponentially large number of forking units, which all must be accepted by honest nodes if the protocol relies only on locally validating units. The goal is to create so many valid forked units that honest nodes which end up downloading them (and they are forced to do so since these units are all valid) are likely to crash or at least slow down significantly because of exhausting RAM or disc memory. Even though this leads to revealing the attacking nodes, there might be no way to punish them, if most of the nodes are unable to proceed with the protocol.

Description of the attack. We describe the attack using the language and rules specific to the QuickAleph protocol (without the alert system). For simplicity, when describing parents of units we only look at their dishonest parents and assume that apart from that, they also connect to all honest units of an appropriate round. At the end of this section we also give some remarks on how one can adjust this description to any other DAG-based protocol.

Technically, the attack is performed as follows: for $K$ consecutive rounds all malicious nodes create forking units, but abstain from sending them to any honest node, after thesee $K$ rounds, they start broadcasting forks from this huge set. Assume that the malicious nodes participating in the attack are numbered $1,2, \ldots, 2 K-1,2 K$ Recall that $U[j ; r]$ denotes the round- $r$ unit created by node $j$. We extend this notation to include forks - let $U[j ; r ; i]$ denote the $i$-th fork, hence $U[j ; r]=U[j ; r ; 1]$.

Suppose it is round $r$ when the attack starts, it will then finish in round $r+K$. We refer to Figure 1 for an example of this construction for $K=3$. A formal description follows: fix a $k \in\{1,2, \ldots, K\}$ the goal of nodes $2 k-1$ and $2 k$ is to fork at round $r+k$, more specifically:
(1) For the initial $k-1$ rounds (starting from round $r$ ) these nodes create regular, non-forking units.

(2) At round $r+k$ the nodes create $2^{K-k}$ forks each, which are sent only to other malicious nodes if $k<K$, and broadcast to every other node if $k=K$. Note that nodes $2 K-1$ and $2 K$ in fact do not fork, hence their units must be eventually accepted by honest nodes.

(3) Units $U[1 ; r+1 ; i]$ and $U[2 ; r+1 ; i]$ can have any round-r units as parents, for $i=1, \ldots, 2^{G-1}$.

(4) For $k>1$ unit $U[2 k-1 ; r+k ; i]$ has $U[2 k-3 ; r+k-1 ; 2 i-1]$ and $U[2 k-2 ; r+k-1 ; 2 i-1]$ as parents, for $i=1, \ldots, 2^{K-k}$.

(5) For $k>1$ unit $U[2 k ; r+k ; i]$ has $U[2 k-3 ; r+k-1 ; 2 i]$ and $U[2 k-2 ; r+k-1 ; 2 i]$ as parents, for $i=1, \ldots, 2^{K-k}$.

Validity of all the forks is easily checked by induction: all units $U[1 ; r+1 ; i]$ and $U[2 ; r+1 ; i]$ are valid, because there are no forks below them (although they are invalid in the sense that they are forked, this fact always becomes apparent only after some other valid nodes has built upon them). Now assume that units $U[2 k-$ $3 ; r+k-1 ; 2 i-1]$ and $U[2 k-2 ; r+k-1 ; 2 i-1]$ are valid. Then the unit $U[2 k-1 ; r+k ; i]$ is valid, because there are no forks created by nodes $2 k-3$ or $2 k-2$ below it. Analogously, the unit $U[2 k ; r+k ; i]$ is valid.

Since the nodes $2 K-1$ and $2 K$ cannot be banned, then units $U[2 K-1 ; r+K ; 1]$ and $U[2 K ; r+K ; 1]$ with their lower cones must be accepted by all honest nodes, and both of them contain at least $2^{K+1}-2$ units in total, counting only $K$ most recent rounds.

To conclude let us give a couple of remarks regarding this attack:

(1) The attack is rather practical, as it does not require any strong assumptions about the adversary (e.g. no control over message delays is required). It is enough for the adversary to coordinate its pool of malicious nodes.

(2) The attack allows any number of parents for a unit. If the protocol requires exactly two parents per unit, with the previous unit created by the same node (as in Hashgraph) then one round of the attack can be realized in two consecutive rounds.

(3) If a larger number of parents is required, then the adversary can connect in addition to any subset of honest units. This is possible because honest nodes can always advance the protocol without the help of malicious nodes, and the number of required parents cannot be higher than the number of honest nodes.