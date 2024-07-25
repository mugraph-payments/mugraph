\title{
Bulletproofs: Short Proofs for Confidential Transactions and More
}

\author{
Benedikt Bünz ${ }^{* 1}$, Jonathan Bootle ${ }^{\dagger 2}$, Dan Boneh ${ }^{\ddagger 1}$, \\ Andrew Poelstra ${ }^{83}$, Pieter Wuille ${ }^{\mathbb{I} 3}$, and Greg Maxwell \\ ${ }^{1}$ Stanford University \\ ${ }^{2}$ University College London \\ ${ }^{3}$ Blockstream
}

\author{
Full Version**
}

\begin{abstract}
We propose Bulletproofs, a new non-interactive zero-knowledge proof protocol with very short proofs and without a trusted setup; the proof size is only logarithmic in the witness size. Bulletproofs are especially well suited for efficient range proofs on committed values: they enable proving that a committed value is in a range using only $2 \log _{2}(n)+9$ group and field elements, where $n$ is the bit length of the range. Proof generation and verification times are linear in $n$.
Bulletproofs greatly improve on the linear (in $n$ ) sized range proofs in existing proposals for confidential transactions in Bitcoin and other cryptocurrencies. Moreover, Bulletproofs supports aggregation of range proofs, so that a party can prove that $m$ commitments lie in a given range by providing only an additive $O(\log (m)$ ) group elements over the length of a single proof. To aggregate proofs from multiple parties, we enable the parties to generate a single proof without revealing their inputs to each other via a simple multi-party computation (MPC) protocol for constructing Bulletproofs. This MPC protocol uses either a constant number of rounds and linear communication, or a logarithmic number of rounds and logarithmic communication. We show that verification time, while asymptotically linear, is very efficient in practice. Moreover, the verification of multiple Bulletproofs can be batched for further speed-up. Concretely, the marginal time to verify an aggregation of 16 range proofs is about the same as the time to verify 16 ECDSA signatures.
Bulletproofs build on the techniques of Bootle et al. (EUROCRYPT 2016). Beyond range proofs, Bulletproofs provide short zero-knowledge proofs for general arithmetic circuits while only relying on the discrete logarithm assumption and without requiring a trusted setup. We discuss many applications that would benefit from Bulletproofs, primarily in the area of cryptocurrencies. The efficiency of Bulletproofs is particularly well suited for the distributed and trustless nature of blockchains.
\end{abstract}


\footnotetext{
*benedikt@cs.stanford.edu

† jonathan.bootle.14@ucl.ac.uk

‡dabo@cs.stanford.edu

§apoelstra@blockstream.io

${ }^{1}$ pieter@blockstream.com

"greg@xiph.org

${ }^{* *}$ An extended abstract of this work appeared at IEEE S\&P 2018 [BBB $\left.{ }^{+} 18\right]$
}

\section{Introduction}

Blockchain-based cryptocurrencies enable peer-to-peer electronic transfer of value by maintaining a global distributed but synchronized ledger, the blockchain. Any independent observer can verify both the current state of the blockchain as well as the validity of all transactions on the ledger. In Bitcoin, this innovation requires that all details of a transaction are public: the sender, the receiver, and the amount transferred. In general, we separate privacy for payments into two properties: (1) anonymity, hiding the identities of sender and receiver in a transaction and (2) confidentiality, hiding the amount transferred. While Bitcoin provides some weak anonymity through the unlinkability of Bitcoin addresses to real world identities, it lacks any confidentiality. This is a serious limitation for Bitcoin and could be prohibitive for many use cases. Would employees want to receive their salaries in bitcoin if it meant that their salaries were published on the public blockchain?

To address the confidentiality of transaction amounts, Maxwell [Max16] introduced confidential transactions (CT), in which every transaction amount involved is hidden from public view using a commitment to the amount. This approach seems to prevent public validation of the blockchain; an observer can no longer check that the sum of transaction inputs is greater than the sum of transaction outputs, and that all transaction values are positive. This can be addressed by including in every transaction a zero-knowledge proof of validity of the confidential transaction.

Current proposals for CT zero-knowledge proofs $\left[\mathrm{PBF}^{+}\right]$have either been prohibitively large or required a trusted setup. Neither is desirable. While one could use succinct zero-knowledge proofs (SNARKs) [BSCG ${ }^{+} 13$, GGPR13], they require a trusted setup, which means that everyone needs to trust that the setup was performed correctly. One could avoid trusted setup by using a STARK [BSBTHR18], but the resulting range proofs while asymptotically efficient are practically larger than even the currently proposed solutions.

Short non-interactive zero-knowledge proofs without a trusted setup, as described in this paper, have many applications in the realm of cryptocurrencies. In any distributed system where proofs are transmitted over a network or stored for a long time, short proofs reduce overall cost.

\subsection{Our Contributions}

We present Bulletproofs, a new zero-knowledge argument of knowledge ${ }^{1}$ system, to prove that a secret committed value lies in a given interval. Bulletproofs do not require a trusted setup. They rely only on the discrete logarithm assumption, and are made non-interactive using the Fiat-Shamir heuristic.

Bulletproofs builds on the techniques of Bootle et al. $\left[\mathrm{BCC}^{+} 16\right]$, which yield communicationefficient zero-knowledge proofs. We present a replacement for their inner-product argument that reduces overall communication by a factor of 3 . We make Bulletproofs suitable for proving statements on committed values. Examples include a range proof, a verifiable shuffle, and other applications discussed below. We note that a range proof using the protocol of $\left[\mathrm{BCC}^{+} 16\right]$ would have required implementing the commitment opening algorithm as part of the verification circuit, which we are able to eliminate.

Distributed Bulletproofs generation. We show that Bulletproofs support a simple and efficient multi-party computation (MPC) protocol that allows multiple parties with secret committed values


\footnotetext{
${ }^{1}$ Proof systems with computational soundness like Bulletproofs are sometimes called argument systems. We will use the terms proof and argument interchangeably.
}
to jointly generate a single small range proof for all their values, without revealing their secret values to each other. One version of our MPC protocol is constant-round but with linear communication. Another variant requires only logarithmic communication, but uses a logarithmic number of rounds. When a confidential transaction has inputs from multiple parties (as in the case of CoinJoin), this MPC protocol can be used to aggregate all the proofs needed to construct the transaction into a single short proof.

Proofs for arithmetic circuits. While we focus on confidential transactions (CT), where our work translates to significant practical savings, we stress that the improvements are not limited to CT. We present Bulletproofs for general NP languages. The proof size is logarithmic in the number of multiplication gates in the arithmetic circuit for verifying a witness. The proofs are much shorter than $\left[\mathrm{BCC}^{+} 16\right]$ and allow inputs to be Pedersen commitments to elements of the witness.

Optimizations and evaluation. We provide a complete implementation of Bulletproofs that includes many further optimizations described in Section 6. For example, we show how to batch the verification of multiple Bulletproofs so that the cost of verifying every additional proof is significantly reduced. We also provide efficiency comparisons with the range proofs currently used for confidential transactions [Max16, Poe] and with other proof systems. Our implementation includes a general tool for constructing Bulletproofs for any NP language. The tool reads in arithmetic circuits in the Pinocchio [PHGR13] format which lets users use their toolchain. This toolchain includes a compiler from C to the circuit format. We expect this to be of great use to implementers who want to use Bulletproofs.

\subsection{Applications}

We first discuss several applications for Bulletproofs along with related work specific to these applications. Additional related work is discussed in Section 1.3.

\subsubsection{Confidential Transactions and Mimblewimble}

Bitcoin and other similar cryptocurrencies use a transaction-output-based system where each transaction fully spends the outputs of previously unspent transactions. These unspent transaction outputs are called UTXOs. Bitcoin allows a single UTXO to be spent to many distinct outputs, each associated with a different address. To spend a UTXO a user must provide a signature, or more precisely a scriptSig, that enables the transaction SCRIPT to evaluate to true $\left[\mathrm{BMC}^{+} 15\right]$. Apart from the validity of the scriptSig, miners verify that the transaction spends previously unspent outputs, and that the sum of the inputs is greater than the sum of the outputs.

Maxwell [Max16] introduced the notion of a confidential transaction, where the input and output amounts in a transaction are hidden in Pedersen commitments $\left[\mathrm{P}^{+} 91\right]$. To enable public validation, the transaction contains a zero-knowledge proof that the sum of the committed inputs is greater than the sum of the committed outputs, and that all the outputs are positive, namely they lie in the interval $\left[0,2^{n}\right]$, where $2^{n}$ is much smaller than the group size. All current implementations of confidential transactions $\left[\mathrm{Max} 16, \mathrm{MP} 15, \mathrm{PBF}^{+}, \mathrm{NM}^{+} 16\right]$ use range proofs over committed values, where the proof size is linear in $n$. These range proofs are the main contributor to the size of a confidential transaction. In current implementations [Max16], a confidential transaction with only two outputs and 32 bits of precision is 5.4 KB bytes, of which 5 KB are allocated to the range proof. Even with recent optimizations the range proofs would still take up 3.8 KB .

We show in Section 6 that Bulletproofs greatly improve on this, even for a single range proof while simultaneously doubling the range proof precision at marginal additional cost ( 64 bytes). The logarithmic proof size additionally enables the prover to aggregate multiple range proofs, e.g. for transactions with multiple outputs, into a single short proof. With Bulletproofs, $m$ range proofs are merely $O(\log (m))$ additional group elements over a single range proof. This is already useful for confidential transactions in their current form as most Bitcoin transactions have two or more outputs. It also presents an intriguing opportunity to aggregate multiple range proofs from different parties into one proof, as would be needed, for example, in a CoinJoin transaction [Max13]. In Section 4.5, we present a simple and efficient MPC protocol that allows multiple users to generate a single transaction with a single aggregate range proof. The users do not have to reveal their secret transaction values to any of the other participants.

Confidential transaction implementations are available in side-chains $\left[\mathrm{PBF}^{+}\right]$, private blockchains [And17], and in the popular privacy-focused cryptocurrency Monero $\left[\mathrm{NM}^{+} 16\right]$. All these implementations would benefit from Bulletproofs.

At the time of writing, Bitcoin has roughly 50 million UTXOs from 22 million transactions (see statoshi.info). Using a 52-bit representation of bitcoin that can cover all values from 1 satoshi up to 21 million bitcoins, this results in roughly 160GB of range proof data using the current systems. Using aggregated Bulletproofs, the range proofs for all UTXOs would take less than 17GB, about a factor 10 reduction in size.

Mimblewimble. Recently an improvement was proposed to confidential transactions, called Mimblewimble [Jed16, Poe], which provides further savings.

Jedusor [Jed16] realized that a Pedersen commitment to 0 can be viewed as an ECDSA public key, and that for a valid confidential transaction the difference between outputs, inputs, and transaction fees must be 0 . A prover constructing a confidential transaction can therefore sign the transaction with the difference of the outputs and inputs as the public key. This small change removes the need for a scriptSig which greatly simplifies the structure of confidential transactions. Poelstra [Poe] further refined and improved Mimblewimble and showed that these improvements enable a greatly simplified blockchain in which all spent transactions can be pruned and new nodes can efficiently validate the entire blockchain without downloading any old and spent transactions. Along with further optimizations, this results in a highly compressed blockchain. It consists only of a small subset of the block-headers as well as the remaining unspent transaction outputs and the accompanying range proofs plus an un-prunable 32 bytes per transaction. Mimblewimble also allows transactions to be aggregated before sending them to the blockchain.

A Mimblewimble blockchain grows with the size of the UTXO set. Using Bulletproofs, it would only grow with the number of transactions that have unspent outputs, which is much smaller than the size of the UTXO set. Overall, Bulletproofs can not only act as a drop-in replacement for the range proofs in confidential transactions, but it can also help make Mimblewimble a practical scheme with a blockchain that is significantly smaller than the current Bitcoin blockchain.

\subsubsection{Provisions}

Dagher et al. $\left[\mathrm{DBB}^{+} 15\right]$ introduced the Provisions protocol which allows Bitcoin exchanges to prove that they are solvent without revealing any additional information. The protocol crucially relies on range proofs to prevent an exchange from inserting fake accounts with negative balances. These range proofs, which take up over 13 GB , are the main contributors to the proof sizes of almost 18GB
for a large exchange with 2 million customers. The proof size is in fact linear in the number of customers. Since in this protocol, one party (the exchange) has to construct many range proofs at once, the general Bulletproofs protocol from Section 4.3 is a natural replacement for the NIZK proof used in Provisions. With the proof size listed in Section 6, we obtain that the range proofs would take up less than 2 KB with our protocol. Additionally, the other parts of the proof could be similarly compressed using the protocol from Section 5. The proof would then be dominated by one commitment per customer, with size 62 MB . This is roughly 300 times smaller then the current implementation of Provisions.

\subsubsection{Verifiable shuffles}

Consider two lists of committed values $x_{1}, \ldots, x_{n}$ and $y_{1}, \ldots, y_{n}$. The goal is to prove that the second list is a permutation of the first. This problem is called a verifiable shuffle. It has many applications in voting [FS01,Nef01], mix-nets [Cha82], and solvency proofs [DBB ${ }^{+}$15]. Neff [Nef01] gave a practical implementation of a verifiable shuffle and later work improved on it [Gro03, GI08a].

Currently the most efficient shuffle [BG12] has size $O(\sqrt{n})$.

Bulletproofs can be used to create a verifiable shuffle of size $O(\log n)$. The two lists of commitments are given as inputs to the circuit protocol from Section 5. The circuit can implement a shuffle by sorting the two lists and then checking that they are equal. A sorting circuit can be implemented using $O(n \cdot \log (n))$ multiplications which means that the proof size will be only $O(\log (n))$. This is much smaller than previously proposed protocols. Given the concrete efficiency of Bulletproofs, a verifiable shuffle using Bulletproofs would be very efficient in practice. Constructing the proof and verifying it takes linear time in $n$.

\subsubsection{NIZK Proofs for Smart Contracts}

The Ethereum [Woo14] system uses highly expressive smart contracts to enable complex transactions. Smart contracts, like any other blockchain transaction, are public and provide no inherent privacy. To bring privacy to smart contracts, non-interactive zero-knowledge (NIZK) proofs have been proposed as a tool to enable complex smart contracts that do not leak the user inputs [KMS ${ }^{+} 16$, MSH17, CGGN17]. However, these protocols are limited as the NIZK proof itself is not suitable for verification by a smart contract. The reason is that communication over the blockchain with a smart contract is expensive, and the smart contract's own computational power is highly limited. SNARKs, which have succinct proofs and efficient verifiers, seem like a natural choice, but current practical SNARKs $\left[\mathrm{BSCG}^{+}\right.$13] require a complex trusted setup. The resulting common reference strings (CRS) are long, specific to each application, and possess trapdoors. In Hawk $\left[\mathrm{KMS}^{+} 16\right]$, for instance, a different CRS is needed for each smart contract, and either a trusted party is needed to generate it, or an expensive multi-party computation is needed to distribute the trust among a few parties. On the other hand, for small applications like boardroom voting, one can use classical sigma protocols [MSH17], but the proof-sizes and expensive verification costs are prohibitive for more complicated applications. Recently, Campanelli et al. [CGGN17] showed how to securely perform zero-knowledge contingent payments (ZKCPs) in Bitcoin, while attacking and fixing a previously proposed protocol [Max]. ZKCPs enable the trustless, atomic and efficient exchange of a cryptocurrency vs. some digital good. While ZKCPs support a wide area of applications they fundamentally work for only a single designated verifier and do not allow for public verification. For some smart contracts that have more than two users, public verification is
often crucial. In an auction, for example, all bidders need to be convinced that all bids are well formed.

Bulletproofs improves on this by enabling small proofs that do not require a trusted setup. The Bulletproofs verifier is not cheap, but there are multiple ways to work around this. First, a smart contract may act optimistically and only verify a proof if some party challenges its validity. Incentives can be used to ensure that rational parties never create an incorrect proof nor challenge a correct proof. This can be further improved by using an interactive referee delegation model [CRR11], previously proposed for other blockchain applications [BGB17,TR]. In this model, the prover provides a proof along with a succinct commitment to the verifier's execution trace. A challenger that disagrees with the computation also commits to his computation trace and the two parties engage in an interactive binary search to find the first point of divergence in the computation. The smart contract can then execute this single computation step and punish the party which provided a faulty execution trace. The intriguing property of this protocol is that even when a proof is challenged, the smart contract only needs to verify a single computation step, i.e. a single gate of the verification circuit. In combination with small Bulletproofs, this can enable more complex but privacy preserving smart contracts. Like in other applications, these NIZK proofs would benefit from the MPC protocol that we present in Section 4.5 to generate Bulletproofs distributively. Consider an auction smart contract where bidders in the first round submit commitments to bids and in the second round open them. A NIZK can be used to prove properties about the bids, e.g. they are in some range, without revealing them. Using Bulletproofs' MPC multiple bidders can combine their Bulletproofs into a single proof. Furthermore, the proof will hide which bidder submitted which bid.

\subsubsection{Short Non-Interactive Proofs for Arithmetic Circuits without a Trusted Setup}

Non-interactive zero-knowledge protocols for general statements are not possible without using a common reference string, which should be known by both the prover and the verifier. Many efficient non-interactive zero-knowledge proofs and arguments for arithmetic circuit satisfiability have been developed [Mic94,KP95, GS08, GGPR13, BSCG ${ }^{+}$13, BSBTHR18], and highly efficient protocols are known. However, aside from their performance, these protocols differ in the complexity of their common reference strings. Some, such as those in $\left[\mathrm{BSCG}^{+}\right.$13], are highly structured, and sometimes feature a trapdoor, while some are simply chosen uniformly at random. Security proofs assume that the common reference string was honestly generated. In practice, the common reference string can be generated by a trusted third party, or using a secure multi-party computation protocol. The latter helps to alleviate concerns about embedded trapdoors, as with the trusted setup ceremony used to generate the public parameters for $\left[\mathrm{BSCG}^{+} 14\right]$.

Zero-knowledge SNARKs have been the subject of extensive research [Gro10,BCCT12,GGPR13, BCCT13,PHGR16, BSCG ${ }^{+}$13, Gro16]. They generate constant-sized proofs for any statement, and have extremely fast verification time. However, they have highly complex common reference strings which require lengthy and computationally intensive protocols [BGG17] to generate distributively. They also rely on strong unfalsifiable assumptions such as the knowledge-of-exponent assumption.

A uniformly-random common reference string, on the other hand, can be derived from common random strings, like the digits of $\pi$ or by assuming that hash functions behave like a random oracle. Examples of non-interactive protocols that do not require a trusted setup include $\left[\mathrm{Mic} 94, \mathrm{BCC}^{+} 16\right.$, $\left.\mathrm{BCG}^{+} 17 \mathrm{~b}, \mathrm{BSBC}^{+} 17, \mathrm{BSBTHR} 18\right]$.

Ben-Sasson et al. present a proof system $\left[\mathrm{BCG}^{+} 17 \mathrm{a}\right]$ and implementation $\left[\mathrm{BSBC}^{+} 17\right]$ called

Scalable Computational Integrity (SCI). While SCI has a simple setup, and relies only on collisionresistant hash functions, the system is not zero-knowledge and still experiences worse performance than $\left[\mathrm{BSCG}^{+} 13, \mathrm{BCC}^{+} 16\right]$. The proof sizes are roughly 42 MB large in practice for a reasonable circuit. In subsequent work Ben-Sasson et al. presented STARKs [BSBTHR18], which are zeroknowledge and more efficient than SCI. However even with these improvements the proof size is still over 200 KB (and grows logarithmically) at only 60 -bit security for a circuit of size $2^{17}$. A Bulletproof for such a circuit at twice the security would be only about 1 KB . Constructing STARKs is also costly in terms of memory requirements because of the large FFT that is required to make proving efficient.

Ames et al. [AHIV17] presented a proof system with linear verification time but only square root proof size building on the MPC in the head technique. Wahby $\left[\mathrm{WTs}^{+}\right]$recently present a cryptographic zero-knowledge proof system which achieves square root verifier complexity and proof size based on the proofs for muggles [GKR08] techniques in combination with a sub-linear polynomial commitment scheme.

\subsection{Additional Related Work}

Much of the research related to electronic payments that predates Bitcoin [Nak08] focused on efficient anonymous and confidential payments [CHL05, Cha82] . With the advent of blockchainbased cryptocurrencies, the question of privacy and confidentiality in transactions has gained a new relevance. While the original Bitcoin paper [Nak08] claimed that Bitcoin would provide anonymity through pseudonymous addresses early work on Bitcoin showed that the anonymity is limited $\left[\mathrm{MPJ}^{+} 13, \mathrm{AKR}^{+} 13\right]$. Given these limitations, various methods have been proposed to help improve the privacy of Bitcoin transactions. CoinJoin [Max13], proposed by Maxwell, allows users to hide information about the amounts of transactions by merging two or more transactions. This ensures that among the participants who join their transactions, it is impossible to tell which transaction inputs correspond to which transaction outputs. However, users do require some way of searching for other users, and furthermore, should be able to do so without relying on a trusted third party. CoinShuffle [RMSK14] tried to fulfill this requirement by taking developing the ideas of CoinJoin and proposing a new Bitcoin mixing protocol which is completely decentralized. Monero [Mon] is a cryptocurrency which employs cryptographic techniques to achieve strong privacy guarantees. These include stealth addresses, ring-signatures [vS13], and ring confidential transactions $\left[\mathrm{NM}^{+} 16\right]$. ZeroCash $\left[\mathrm{BSCG}^{+} 14\right]$ offers optimal privacy guarantees but comes at the cost of expensive transaction generation and the requirement of a trusted setup.

Range proofs. Range proofs are proofs that a secret value, which has been encrypted or committed to, lies in a certain interval. Range proofs do not leak any information about the secret value, other than the fact that they lie in the interval. Lipmaa [Lip03] presents a range proof which uses integer commitments, and Lagrange's four-square theorem which states that every positive integer $y$ can be expressed as a sum of four squares. Groth [Gro05] notes that the argument can be optimized by considering $4 y+1$, since integers of this form only require three squares. The arguments require only a constant number of commitments. However, each commitment is large, as the security of the argument relies on the Strong RSA assumption. Additionally, a trusted setup is required to generate the RSA modulus or a prohibitively large modulus needs to be used [San99]. Camenisch et al. [CCs08] use a different approach. The verifier provides signatures on a small set of digits. The prover commits to the digits of the secret value, and then proves in zero-knowledge that the
value matches the digits, and that each commitment corresponds to one of the signatures. They show that their scheme can be instantiated securely using both RSA accumulators [BdM93] and the Boneh-Boyen signature scheme [BB04]. However, these range proofs require a trusted setup. Approaches based on the $n$-ary digits of the secret value are limited to proving that the secret value is in an interval of the form $\left[0, n^{k}-1\right]$. One can produce range proofs for more general intervals by using homomorphic commitments to translate intervals, and by using a combination of two different range proofs to conduct range proofs for intervals of different widths. However, [CLas10] presented an alternative digital decomposition which enables an interval of general width to be handled using a single range proof.

\section{Preliminaries}

Before we present Bulletproofs, we first review some of the underlying tools. In what follows, a PPT adversary $\mathcal{A}$ is a probabilistic interactive Turing Machine that runs in polynomial time in the security parameter $\lambda$. We will drop the security parameter $\lambda$ from the notation when it is implicit.

\subsection{Assumptions}

Definition 1 (Discrete Log Relation). For all PPT adversaries $\mathcal{A}$ and for all $n \geqslant 2$ there exists a negligible function $\mu(\lambda)$ such that

$$
P\left[\begin{array}{l}
\mathbb{G}=\operatorname{Setup}\left(1^{\lambda}\right), g_{1}, \ldots, g_{n} \stackrel{\$}{\leftarrow} \mathbb{G} ; \\
a_{1}, \ldots, a_{n} \in \mathbb{Z}_{p} \leftarrow \mathcal{A}\left(G, g_{1}, \ldots, g_{n}\right)
\end{array}: \exists a_{i} \neq 0 \wedge \prod_{i=1}^{n} g_{i}^{a_{i}}=1\right] \leqslant \mu(\lambda)
$$

We say $\prod_{i=1}^{n} g_{i}^{a_{i}}=1$ is a non trivial discrete log relation between $g_{1}, \ldots, g_{n}$. The Discrete Log Relation assumption states that an adversary can't find a non-trivial relation between randomly chosen group elements. For $n \geqslant 1$ this assumption is equivalent to the discrete-log assumption.

\subsection{Commitments}

Definition 2 (Commitment). A non-interactive commitment scheme consists of a pair of probabilistic polynomial time algorithms (Setup, Com). The setup algorithm $\mathrm{pp} \leftarrow \operatorname{Setup}\left(1^{\lambda}\right)$ generates public parameters pp for the scheme, for security parameter $\lambda$. The commitment algorithm $\mathrm{Com}_{\mathrm{pp}}$ defines a function $\mathrm{M}_{\mathrm{pp}} \times \mathrm{R}_{\mathrm{pp}} \rightarrow C_{\mathrm{pp}}$ for message space $\mathrm{M}_{\mathrm{pp}}$, randomness space $\mathrm{R}_{\mathrm{pp}}$ and commitment space $\mathrm{C}_{\mathrm{pp}}$ determined by pp. For a message $x \in \mathrm{M}_{\mathrm{pp}}$, the algorithm draws $r \stackrel{\$}{\leftarrow} \mathrm{R}_{\mathrm{pp}}$ uniformly at random, and computes commitment $\mathbf{c o m}=\operatorname{Com}_{\mathrm{pp}}(x ; r)$.

For ease of notation we write $\mathrm{Com}=\mathrm{Com}_{\mathrm{pp}}$.

Definition 3 (Homomorphic Commitments). A homomorphic commitment scheme is a noninteractive commitment scheme such that $\mathrm{M}_{\mathrm{pp}}, \mathrm{R}_{\mathrm{pp}}$ and $\mathrm{C}_{\mathrm{pp}}$ are all abelian groups, and for all $x_{1}, x_{2} \in \mathrm{M}_{\mathrm{pp}}, r_{1}, r_{2} \in \mathrm{R}_{\mathrm{pp}}$, we have

$$
\operatorname{Com}\left(x_{1} ; r_{1}\right)+\operatorname{Com}\left(x_{2} ; r_{2}\right)=\operatorname{Com}\left(x_{1}+x_{2} ; r_{1}+r_{2}\right)
$$

Definition 4 (Hiding Commitment). A commitment scheme is said to be hiding if for all PPT adversaries $\mathcal{A}$ there exists a negligible function $\mu(\lambda)$ such that.

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-09.jpg?height=192&width=1197&top_left_y=351&top_left_x=464)

where the probability is over $b, r$, Setup and $\mathcal{A}$. If $\mu(\lambda)=0$ then we say the scheme is perfectly hiding.

Definition 5 (Binding Commitment). A commitment scheme is said to be binding if for all PPT adversaries $\mathcal{A}$ there exists a negligible function $\mu$ such that.

$$
\mathrm{P}\left[\operatorname{Com}\left(x_{0} ; r_{0}\right)=\operatorname{Com}\left(x_{1} ; r_{1}\right) \wedge x_{0} \neq x_{1} \left\lvert\, \begin{array}{l}
\mathrm{pp} \leftarrow \operatorname{Setup}\left(1^{\lambda}\right), \\
x_{0}, x_{1}, r_{0}, r_{1} \leftarrow \mathcal{A}(\mathrm{pp})
\end{array}\right.\right] \leqslant \mu(\lambda)
$$

where the probability is over Setup and $\mathcal{A}$. If $\mu(\lambda)=0$ then we say the scheme is perfectly binding.

In what follows, the order $p$ of the groups used is implicitly dependent on the security parameter $\lambda$ to ensure that discrete log in these groups is intractable for PPT adversaries.

Definition 6 (Pedersen Commitment). $\mathrm{M}_{\mathrm{pp}}, \mathrm{R}_{\mathrm{pp}}=\mathbb{Z}_{p}, \mathrm{C}_{\mathrm{pp}}=\mathbb{G}$ of order $p$.

Setup : $g, h \stackrel{\$}{\leftarrow} \mathbb{G}$

$\operatorname{Com}(x ; r)=\left(g^{x} h^{r}\right)$

Definition 7 (Pedersen Vector Commitment). $\mathrm{M}_{\mathrm{pp}}=\mathbb{Z}_{p}^{n}, \mathrm{R}_{\mathrm{pp}}=\mathbb{Z}_{p}, \mathrm{C}_{\mathrm{pp}}=\mathbb{G}$ with $\mathbb{G}$ of order $p$ Setup : $\mathbf{g}=\left(g_{1}, \ldots, g_{n}\right), h \stackrel{\$}{\rightleftarrows}$

$\operatorname{Com}\left(\mathbf{x}=\left(x_{1}, \ldots, x_{n}\right) ; r\right)=h^{r} \mathbf{g}^{\mathbf{x}}=h^{r} \prod_{i} g_{i}^{x_{i}} \in \mathbb{G}$

The Pedersen vector commitment is perfectly hiding and computationally binding under the discrete logarithm assumption. We will often set $r=0$, in which case the commitment is binding but not hiding.

\subsection{Zero-Knowledge Arguments of Knowledge}

Bulletproofs are zero-knowledge arguments of knowledge. A zero-knowledge proof of knowledge is a protocol in which a prover can convince a verifier that some statement holds without revealing any information about why it holds. A prover can for example convince a verifier that a confidential transaction is valid without revealing why that is the case, i.e. without leaking the transacted values. An argument is a proof which holds only if the prover is computationally bounded and certain computational hardness assumptions hold. We now give formal definitions.

We will consider arguments consisting of three interactive algorithms (Setup, $\mathcal{P}, \mathcal{V}$ ), all running in probabilistic polynomial time. These are the common reference string generator Setup, the prover $\mathcal{P}$, and the verifier $\mathcal{V}$. On input $1^{\lambda}$, algorithm Setup produces a common reference string $\sigma$. The transcript produced by $\mathcal{P}$ and $\mathcal{V}$ when interacting on inputs $s$ and $t$ is denoted by $\operatorname{tr} \leftarrow\langle\mathcal{P}(s), \mathcal{V}(t)\rangle$. We write $\langle\mathcal{P}(s), \mathcal{V}(t)\rangle=b$ depending on whether the verifier rejects, $b=0$, or accepts, $b=1$.

Let $\mathcal{R} \subset\{0,1\}^{*} \times\{0,1\}^{*} \times\{0,1\}^{*}$ be a polynomial-time-decidable ternary relation. Given $\sigma$, we call $w$ a witness for a statement $u$ if $(\sigma, u, w) \in \mathcal{R}$, and define the CRS-dependent language

$$
\mathcal{L}_{\sigma}=\{x \mid \exists w:(\sigma, x, w) \in \mathcal{R}\}
$$

as the set of statements $x$ that have a witness $w$ in the relation $\mathcal{R}$.

Definition 8 (Argument of Knowledge). The triple (Setup, $\mathcal{P}, \mathcal{V}$ ) is called an argument of knowledge for relation $\mathcal{R}$ if it satisfies the following two definitions.

Definition 9 (Perfect completeness). (Setup, $\mathcal{P}, \mathcal{V}$ ) has perfect completeness if for all non-uniform polynomial time adversaries $\mathcal{A}$

$$
\mathrm{P}\left[\begin{array}{l|l}
(\sigma, u, w) \notin \mathcal{R} \text { or }\langle\mathcal{P}(\sigma, u, w), \mathcal{V}(\sigma, u)\rangle=1 & \begin{array}{l}
\sigma \leftarrow \operatorname{Setup}\left(1^{\lambda}\right) \\
(u, w) \leftarrow \mathcal{A}(\sigma)
\end{array}
\end{array}\right]=1
$$

Definition 10 (Computational Witness-Extended Emulation). (Setup, $\mathcal{P}, \mathcal{V}$ ) has witness-extended emulation if for all deterministic polynomial time $\mathcal{P}^{*}$ there exists an expected polynomial time emulator $\mathcal{E}$ such that for all pairs of interactive adversaries $\mathcal{A}_{1}, \mathcal{A}_{2}$ there exists a negligible function $\mu(\lambda)$ such that

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-10.jpg?height=313&width=1284&top_left_y=841&top_left_x=418)

where the oracle is given by $\mathcal{O}=\left\langle\mathcal{P}^{*}(\sigma, u, s), \mathcal{V}(\sigma, u)\right\rangle$, and permits rewinding to a specific point and resuming with fresh randomness for the verifier from this point onwards. We can also define computational witness-extended emulation by restricting to non-uniform polynomial time adversaries $\mathcal{A}_{1}$ and $\mathcal{A}_{2}$.

We use witness-extended emulation to define knowledge-soundness as used for example in $\left[\mathrm{BCC}^{+} 16\right]$ and defined in [GI08b, Lin03]. Informally, whenever an adversary produces an argument which satisfies the verifier with some probability, then there exists an emulator producing an identically distributed argument with the same probability, but also a witness. The value $s$ can be considered to be the internal state of $\mathcal{P}^{*}$, including randomness. The emulator is permitted to rewind the interaction between the prover and verifier to any move, and resume with the same internal state for the prover, but with fresh randomness for the verifier. Whenever $\mathcal{P}^{*}$ makes a convincing argument when in state $s, \mathcal{E}$ can extract a witness, and therefore, we have an argument of knowledge of $w$ such that $(\sigma, u, w) \in \mathcal{R}$.

Definition 11 (Public Coin). An argument of knowledge (Setup, $\mathcal{P}, \mathcal{V}$ ) is called public coin if all messages sent from the verifier to the prover are chosen uniformly at random and independently of the prover's messages, i.e., the challenges correspond to the verifier's randomness $\rho$.

An argument of knowledge is zero knowledge if it does not leak information about $w$ apart from what can be deduced from the fact that $(\sigma, x, w) \in \mathcal{R}$. We will present arguments of knowledge that have special honest-verifier zero-knowledge. This means that given the verifier's challenge values, it is possible to efficiently simulate the entire argument without knowing the witness.

Definition 12 (Perfect Special Honest-Verifier Zero-Knowledge). A public coin argument of knowledge (Setup, $\mathcal{P}, \mathcal{V}$ ) is a perfect special honest verifier zero knowledge (SHVZK) argument of knowledge for $\mathcal{R}$ if there exists a probabilistic polynomial time simulator $\mathcal{S}$ such that for all pairs of
interactive adversaries $\mathcal{A}_{1}, \mathcal{A}_{2}$

$$
\begin{aligned}
& \operatorname{Pr}\left[\begin{array}{l|l}
(\sigma, u, w) \in \mathcal{R} \text { and } \mathcal{A}_{1}(\operatorname{tr})=1 & \begin{array}{l}
\sigma \leftarrow \operatorname{Setup}\left(1^{\lambda}\right),(u, w, \rho) \leftarrow \mathcal{A}_{2}(\sigma), \\
\operatorname{tr} \leftarrow\langle\mathcal{P}(\sigma, u, w), \mathcal{V}(\sigma, u ; \rho)\rangle
\end{array}
\end{array}\right] \\
& =\operatorname{Pr}\left[\begin{array}{l|l}
(\sigma, u, w) \in \mathcal{R} \text { and } \mathcal{A}_{1}(\operatorname{tr})=1 & \begin{array}{l}
\sigma \leftarrow \operatorname{Setup}\left(1^{\lambda}\right),(u, w, \rho) \leftarrow \mathcal{A}_{2}(\sigma), \\
\operatorname{tr} \leftarrow \mathcal{S}(u, \rho)
\end{array}
\end{array}\right]
\end{aligned}
$$

where $\rho$ is the public coin randomness used by the verifier.

In this definition the adversary chooses a distribution over statements and witnesses but is still not able to distinguish between the simulated and the honestly generated transcripts for valid statements and witnesses.

We now define range proofs, which are proofs that the prover knows an opening to a commitment, such that the committed value is in a certain range. Range proofs can be used to show that an integer commitment is to a positive number or that two homomorphic commitments to elements in a field of prime order will not overflow modulo the prime when they are added together.

Definition 13 (Zero-Knowledge Range Proof). Given a commitment scheme (Setup, Com) over a message space $\mathrm{M}_{\mathrm{pp}}$ which is a set with a total ordering, a Zero-Knowledge Range Proof is a SHVZK argument of knowledge for the relation $\mathcal{R}_{\text {Range }}$ :

$$
\mathcal{R}_{\text {Range }}:(\mathrm{pp},(\operatorname{com}, l, r),(x, \rho)) \in \mathcal{R}_{\text {Range }} \leftrightarrow \operatorname{com}=\operatorname{Com}(x ; \rho) \wedge l \leqslant x<r
$$

\subsection{Notation}

Let $\mathbb{G}$ denote a cyclic group of prime order $p$, and let $\mathbb{Z}_{p}$ denote the ring of integers modulo $p$. Let $\mathbb{G}^{n}$ and $\mathbb{Z}_{p}^{n}$ be vector spaces of dimension $n$ over $\mathbb{G}$ and $\mathbb{Z}_{p}$ respectively. Let $\mathbb{Z}_{p}^{\star}$ denote $\mathbb{Z}_{p} \backslash\{0\}$. Generators of $\mathbb{G}$ are denoted by $g, h, v, u \in \mathbb{G}$. Group elements which represent commitments are capitalized and blinding factors are denoted by Greek letters, i.e. $C=g^{a} h^{\alpha} \in \mathbb{G}$ is a Pedersen commitment to $a$. If not otherwise clear from context $x, y, z \in \mathbb{Z}_{p}^{\star}$ are uniformly distributed challenges.

$x \stackrel{\$}{\leftarrow} \mathbb{Z}_{p}^{\star}$ denotes the uniform sampling of an element from $\mathbb{Z}_{p}^{\star}$. Throughout the paper, we will also be using vector notations defined as follows. Bold font denotes vectors, i.e. $\mathbf{a} \in \mathbb{F}^{n}$ is a vector with elements $a_{1}, \ldots, a_{n} \in \mathbb{F}$. Capitalized bold font denotes matrices, i.e. $\mathbf{A} \in \mathbb{F}^{n \times m}$ is a matrix with $n$ rows and $m$ columns such that $a_{i, j}$ is the element of $\mathbf{A}$ in the $i$ th row and $j$ th column. For a scalar $c \in \mathbb{Z}_{p}$ and a vector $\mathbf{a} \in \mathbb{Z}_{p}^{n}$, we denote by $\mathbf{b}=c \cdot \mathbf{a} \in \mathbb{Z}_{p}^{n}$ the vector where $b_{i}=c \cdot a_{i}$. Furthermore, let $\langle\mathbf{a}, \mathbf{b}\rangle=\sum_{i=1}^{n} a_{i} \cdot b_{i}$ denotes the inner product between two vectors $\mathbf{a}, \mathbf{b} \in \mathbb{F}^{n}$ and $\mathbf{a} \circ \mathbf{b}=\left(a_{1} \cdot b_{1}, \ldots, a_{n} \cdot b_{n}\right) \in \mathbb{F}^{n}$ the Hadamard product or entry wise multiplication of two vectors.

We also define vector polynomials $p(X)=\sum_{i=0}^{d} \mathbf{p}_{\mathbf{i}} \cdot X^{i} \in \mathbb{Z}_{p}^{n}[X]$ where each coefficient $\mathbf{p}_{\mathbf{i}}$ is a vector in $\mathbb{Z}_{p}^{n}$. The inner product between two vector polynomials $l(X), r(X)$ is defined as

$$
\begin{equation*}
\langle l(X), r(X)\rangle=\sum_{i=0}^{d} \sum_{j=0}^{i}\left\langle\mathbf{l}_{\mathbf{i}}, \mathbf{r}_{\mathbf{j}}\right\rangle \cdot X^{i+j} \in \mathbb{Z}_{p}[X] \tag{1}
\end{equation*}
$$

Let $t(X)=\langle\mathbf{l}(X), \mathbf{r}(X)\rangle$, then the inner product is defined such that $t(x)=\langle l(x), r(x)\rangle$ holds for all $x \in \mathbb{Z}_{p}$, i.e. evaluating the polynomials at $x$ and then taking the inner product is the same as evaluating the inner product polynomial at $x$.

For a vector $\mathbf{g}=\left(g_{1}, \ldots, g_{n}\right) \in \mathbb{G}^{n}$ and $\mathbf{a} \in \mathbb{Z}_{p}^{n}$ we write $C=\mathbf{g}^{\mathbf{a}}=\prod_{i=1}^{n} g_{i}^{a_{i}} \in \mathbb{G}$. This quantity is a binding (but not hiding) commitment to the vector $\mathbf{a} \in \mathbb{Z}_{p}^{n}$. Given such a commitment $C$ and a vector $\mathbf{b} \in \mathbb{Z}_{p}^{n}$ with non-zero entries, we can treat $C$ as a new commitment to $\mathbf{a} \circ \mathbf{b}$. To so do, define $g_{i}^{\prime}=g_{i}^{\left(b_{i}^{-1}\right)}$ such that $C=\prod_{i=1}^{n}\left(g_{i}^{\prime}\right)^{a_{i} \cdot b_{i}}$. The binding property of this new commitment is inherited from the old commitment.

Let $\mathbf{a} \| \mathbf{b}$ denote the concatenation of two vectors: if $\mathbf{a} \in \mathbb{Z}_{p}^{n}$ and $\mathbf{b} \in \mathbb{Z}_{p}^{m}$ then $\mathbf{a} \| \mathbf{b} \in \mathbb{Z}_{p}^{n+m}$. For $0 \leqslant \ell \leqslant n$, we use Python notation to denote slices of vectors:

$$
\mathbf{a}_{[: \ell]}=\left(a_{1}, \ldots, a_{\ell}\right) \in \mathbb{F}^{\ell}, \quad \mathbf{a}_{[\ell:]}=\left(a_{\ell+1}, \ldots, a_{n}\right) \in \mathbb{F}^{n-\ell}
$$

For $k \in \mathbb{Z}_{p}^{\star}$ we use $\mathbf{k}^{n}$ to denote the vector containing the first $n$ powers of $k$, i.e.

$$
\mathbf{k}^{n}=\left(1, k, k^{2}, \ldots, k^{n-1}\right) \in\left(\mathbb{Z}_{p}^{\star}\right)^{n}
$$

For example, $\mathbf{2}^{n}=\left(1,2,4, \ldots, 2^{n-1}\right)$. Equivalently $\mathbf{k}^{-n}=\left(\mathbf{k}^{-1}\right)^{n}=\left(1, k^{-1}, \ldots, k^{-n+1}\right)$.

Finally, we write $\{$ (Public Input; Witness) : Relation $\}$ to denote the relation Relation using the specified Public Input and Witness.

\section{Improved Inner-Product Argument}

Bootle et al. $\left[\mathrm{BCC}^{+} 16\right]$ introduced a communication efficient inner-product argument and show how it can be leveraged to construct zero-knowledge proofs for arithmetic circuit satisfiability with low communication complexity. The argument is an argument of knowledge that the prover knows the openings of two binding Pedersen vector commitments that satisfy a given inner product relation.

We reduce the communication complexity of the argument from $6 \log _{2}(n)$ in $\left[\mathrm{BCC}^{+} 16\right]$ to only $2 \log _{2}(n)$, where $n$ is the dimension of the two vectors. We achieve this improvement by modifying the relation being proved. Our argument is sound, but is not zero-knowledge. We then show that this protocol gives a public-coin, communication efficient, zero-knowledge range proof on a set of committed values, and a zero-knowledge proof system for arbitrary arithmetic circuits (Sections 4 and 5). By applying the Fiat-Shamir heuristic we obtain short non-interactive proofs (Section 4.4).

Overview. The inputs to the inner-product argument are independent generators $\mathbf{g}, \mathbf{h} \in \mathbb{G}^{n}$, a scalar $c \in \mathbb{Z}_{p}$, and $P \in \mathbb{G}$. The argument lets the prover convince a verifier that the prover knows two vectors $\mathbf{a}, \mathbf{b} \in \mathbb{Z}_{p}^{n}$ such that

$$
P=\mathbf{g}^{\mathbf{a}} \mathbf{h}^{\mathbf{b}} \quad \text { and } \quad c=\langle\mathbf{a}, \mathbf{b}\rangle
$$

We refer to $P$ as a binding vector commitment to $\mathbf{a}, \mathbf{b}$. Throughout the section we assume that the dimension $n$ is a power of 2 . If need be, one can easily pad the inputs to ensure that this holds.

More precisely, the inner product argument is an efficient proof system for the following relation:

$$
\begin{equation*}
\left\{\left(\mathbf{g}, \mathbf{h} \in \mathbb{G}^{n}, P \in \mathbb{G}, c \in \mathbb{Z}_{p} ; \mathbf{a}, \mathbf{b} \in \mathbb{Z}_{p}^{n}\right): \quad P=\mathbf{g}^{\mathbf{a}} \mathbf{h}^{\mathbf{b}} \wedge c=\langle\mathbf{a}, \mathbf{b}\rangle\right\} \tag{2}
\end{equation*}
$$

The simplest proof system for (2) is one where the prover sends the vectors $\mathbf{a}, \mathbf{b} \in \mathbb{Z}_{p}^{n}$ to the verifier. The verifier accepts if these vectors are a valid witness for (2). This is clearly sound, however, it requires sending $2 n$ elements to the verifier. Our goal is to send only $2 \log _{2}(n)$ elements.

We show how to do this when the inner product $c=\langle\mathbf{a}, \mathbf{b}\rangle$ is given as part of the vector commitment $P$. That is, for a given $P \in \mathbb{G}$, the prover proves that it has vectors $\mathbf{a}, \mathbf{b} \in \mathbb{Z}_{p}^{n}$ for which $P=\mathbf{g}^{\mathbf{a}} \mathbf{h}^{\mathbf{b}} \cdot u^{\langle\mathbf{a}, \mathbf{b}\rangle}$. More precisely, we design a proof system for the relation:

$$
\begin{equation*}
\left\{\left(\mathbf{g}, \mathbf{h} \in \mathbb{G}^{n}, u, P \in \mathbb{G} ; \mathbf{a}, \mathbf{b} \in \mathbb{Z}_{p}^{n}\right): P=\mathbf{g}^{\mathbf{a}} \mathbf{h}^{\mathbf{b}} \cdot u^{\langle\mathbf{a}, \mathbf{b}\rangle}\right\} \tag{3}
\end{equation*}
$$

We show in Protocol 1 below that a proof system for (3) gives a proof system for (2) with the same complexity. Hence, it suffices to give a proof system for (3).

To give some intuition for how the proof system for the relation (3) works let us define a hash function $H: \mathbb{Z}_{p}^{2 n+1} \rightarrow \mathbb{G}$ as follows. First, set $n^{\prime}=n / 2$ and fix generators $\mathbf{g}, \mathbf{h} \in \mathbb{G}^{n}, u \in \mathbb{G}$. Then the hash function $H$ takes as input $\mathbf{a}, \mathbf{a}^{\prime}, \mathbf{b}, \mathbf{b}^{\prime} \in \mathbb{Z}_{p}^{n^{\prime}}$ and $c \in \mathbb{Z}_{p}$, and outputs

$$
H\left(\mathbf{a}, \mathbf{a}^{\prime}, \mathbf{b}, \mathbf{b}^{\prime}, c\right)=\mathbf{g}_{\left[: n^{\prime}\right]}^{\mathbf{a}} \cdot \mathbf{g}_{\left.\left[n^{\prime}:\right]\right]}^{\mathbf{a}^{\prime}} \cdot \mathbf{h}_{\left[: n^{\prime}\right]}^{\mathbf{b}} \cdot \mathbf{h}_{\left[n^{\prime}:\right]}^{\mathbf{b}^{\prime}} \cdot u^{c} \in \mathbb{G}
$$

Now, using the setup in (3), we can write $P$ as $P=H\left(\mathbf{a}_{\left[: n^{\prime}\right]}, \mathbf{a}_{\left.\left[n^{\prime}\right]\right]}, \mathbf{b}_{\left[: n^{\prime}\right]}, \mathbf{b}_{\left[n^{\prime}:\right]},\langle\mathbf{a}, \mathbf{b}\rangle\right)$. Note that $H$ is additively homomorphic in its inputs, i.e.

$$
H\left(\mathbf{a}_{1}, \mathbf{a}_{1}^{\prime}{ }_{1}, \mathbf{b}_{1}, \mathbf{b}_{1}^{\prime}, c_{1}\right) \cdot H\left(\mathbf{a}_{2}, \mathbf{a}_{2}^{\prime}, \mathbf{b}_{2}, \mathbf{b}_{2}^{\prime}{ }_{2}, c_{2}\right)=H\left(\mathbf{a}_{1}+\mathbf{a}_{2}, \mathbf{a}_{1}^{\prime}+\mathbf{a}_{2}^{\prime}, \mathbf{b}_{1}+\mathbf{b}_{2}, \mathbf{b}_{1}^{\prime}+\mathbf{b}_{2}^{\prime}, c_{1}+c_{2}\right)
$$

Consider the following protocol for the relation (3), where $P \in \mathbb{G}$ is given as input:
- The prover computes $L, R \in \mathbb{G}$ as follows:

$$
\begin{aligned}
& L=H\left(\mathbf{0}^{n^{\prime}}, \quad \mathbf{a}_{\left[: n^{\prime}\right]}, \mathbf{b}_{\left[n^{\prime}:\right]}, \quad \mathbf{0}^{n^{\prime}},\left\langle\mathbf{a}_{\left[: n^{\prime}\right]}, \mathbf{b}_{\left[n^{\prime}:\right]}\right\rangle\right) \\
& R=H\left(\begin{array}{llll}
\mathbf{a}_{\left[n^{\prime}:\right]}, & \mathbf{0}^{n^{\prime}}, & \mathbf{0}^{n^{\prime}}, & \left.\mathbf{b}_{\left[: n^{\prime}\right]},\left\langle\mathbf{a}_{\left[n^{\prime}:\right]}, \mathbf{b}_{\left[: n^{\prime}\right]}\right\rangle\right)
\end{array}\right. \\
& \text { and recall that } P=H\left(\mathbf{a}_{\left[: n^{\prime}\right]}, \mathbf{a}_{\left[n^{\prime}:\right]}, \mathbf{b}_{\left[: n^{\prime}\right]}, \mathbf{b}_{\left[n^{\prime}:\right]}, \quad\langle\mathbf{a}, \mathbf{b}\rangle\right) \text {. }
\end{aligned}
$$

It sends $L, R \in \mathbb{G}$ to the verifier.
- The verifier chooses a random $x \stackrel{\$}{\leftarrow} \mathbb{Z}_{p}$ and sends $x$ to the prover.
- The prover computes $\quad \mathbf{a}^{\prime}=x \mathbf{a}_{\left[: n^{\prime}\right]}+x^{-1} \mathbf{a}_{\left[n^{\prime}:\right]} \in \mathbb{Z}_{p}^{n^{\prime}} \quad$ and $\quad \mathbf{b}^{\prime}=x^{-1} \mathbf{b}_{\left[: n^{\prime}\right]}+x \mathbf{b}_{\left[n^{\prime}:\right]} \in \mathbb{Z}_{p}^{n^{\prime}}$ and sends $\mathbf{a}^{\prime}, \mathbf{b}^{\prime} \in \mathbb{Z}_{p}^{n^{\prime}}$ to the verifier.
- Given $\left(L, R, \mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right)$, the verifier computes $P^{\prime}=L^{\left(x^{2}\right)} \cdot P \cdot R^{\left(x^{-2}\right)}$ and outputs "accept" if

$$
\begin{equation*}
P^{\prime}=H\left(x^{-1} \mathbf{a}^{\prime}, x \mathbf{a}^{\prime}, x \mathbf{b}^{\prime}, x^{-1} \mathbf{b}^{\prime},\left\langle\mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right\rangle\right) \tag{4}
\end{equation*}
$$

It is easy to verify that a proof from an honest prover will always be accepted. Indeed, the left hand side of (4) is

$$
L^{x^{2}} \cdot P \cdot R^{x^{-2}}=H\left(\mathbf{a}_{\left[: n^{\prime}\right]}+x^{-2} \mathbf{a}_{\left[n^{\prime}:\right]}, \quad x^{2} \mathbf{a}_{\left[: n^{\prime}\right]}+\mathbf{a}_{\left[n^{\prime}:\right]}, \quad x^{2} \mathbf{b}_{\left[n^{\prime}:\right]}+\mathbf{b}_{\left[: n^{\prime}\right]}, \quad \mathbf{b}_{\left[n^{\prime}:\right]}+x^{-2} \mathbf{b}_{\left[: n^{\prime}\right]}, \quad\left\langle\mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right\rangle\right)
$$

which is the same as the right hand side of (4).

In this proof system, the proof sent from the prover is the four tuple $\left(L, R, \mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right)$ and contains only $n+2$ elements. This is about half the length of the trivial proof where the prover sends the complete $\mathbf{a}, \mathbf{b} \in \mathbb{Z}_{p}^{n}$ to the verifier.

To see why this protocol is a proof system for (3) we show how to extract a valid witness $\mathbf{a}, \mathbf{b} \in \mathbb{Z}_{p}^{n}$ from a successful prover. After the prover sends $L, R$ we rewind the prover three times to obtain three tuples $\left(x_{i}, \mathbf{a}_{i}{ }_{i}, \mathbf{b}^{\prime}{ }_{i}\right.$ ) for $i=1, \ldots, 3$, where each tuple satisfies (4), namely

$$
\begin{equation*}
L^{\left(x_{i}^{2}\right)} \cdot P \cdot R^{\left(x_{i}^{-2}\right)}=H\left(x_{i}^{-1} \mathbf{a}_{i}^{\prime}, x_{i} \mathbf{a}_{i}^{\prime}, x_{i} \mathbf{b}_{i}^{\prime}, x_{i}^{-1} \mathbf{b}_{i}^{\prime}{ }_{i},\left\langle\mathbf{a}_{i}^{\prime}, \mathbf{b}_{i}^{\prime}{ }_{i}\right\rangle\right) \tag{5}
\end{equation*}
$$

Assuming $x_{i} \neq \pm x_{j}$ for $1 \leqslant i<j \leqslant 3$, we can find $\nu_{1}, \nu_{2}, \nu_{3} \in \mathbb{Z}_{p}$ such that

$$
\sum_{i=1}^{3} x_{i}^{2} \nu_{i}=0 \quad \text { and } \quad \sum_{i=1}^{3} \nu_{i}=1 \quad \text { and } \quad \sum_{i=1}^{3} x_{i}^{-2} \nu_{i}=0
$$

Then setting

$$
\mathbf{a}=\sum_{i=1}^{3}\left(\nu_{i} \cdot x_{i}^{-1} \mathbf{a}_{i}^{\prime}, \quad \nu_{i} \cdot x_{i} \mathbf{a}_{i}^{\prime}\right) \in \mathbb{Z}_{p}^{n} \quad \text { and } \quad \mathbf{b}=\sum_{i=1}^{3}\left(\nu_{i} \cdot x_{i} \mathbf{b}_{i}^{\prime}, \quad \nu_{i} \cdot x_{i}^{-1} \mathbf{b}_{i}^{\prime}\right) \in \mathbb{Z}_{p}^{n}
$$

we obtain that $P=H\left(\mathbf{a}_{\left[: n^{\prime}\right]}, \mathbf{a}_{\left[n^{\prime}:\right]}, \mathbf{b}_{\left[: n^{\prime}\right]}, \mathbf{b}_{\left.\left[n^{\prime}\right]\right]}, c\right)$ where $c=\sum_{i=1}^{3} \nu_{i} \cdot\left\langle\mathbf{a}_{i}^{\prime}, \mathbf{b}_{i}^{\prime}\right\rangle$. We will show in the proof of Theorem 1 below that with one additional rewinding, to obtain a fourth relation satisfying (5), we must have $c=\langle\mathbf{a}, \mathbf{b}\rangle$ with high probability. Hence, the extracted $\mathbf{a}, \mathbf{b}$ are a valid witness for the relation (3), as required.

Shrinking the proof by recursion. Observe that the test in (4) is equivalent to testing that

$$
P^{\prime}=\left(\mathbf{g}_{\left[n^{\prime}\right]}^{x^{-1}} \circ \mathbf{g}_{\left[n^{\prime}:\right]}^{x}\right)^{\mathbf{a}^{\prime}} \cdot\left(\mathbf{h}_{\left[: n^{\prime}\right]}^{x} \circ \mathbf{h}_{\left[n^{\prime}:\right]}^{x^{-1}}\right)^{\mathbf{b}^{\prime}} \cdot u^{\left\langle\mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right\rangle}
$$

Hence, instead of the prover sending the vectors $\mathbf{a}^{\prime}, \mathbf{b}^{\prime}$ to the verifier, they can recursively engage in an inner-product argument for $P^{\prime}$ with respect to generators $\left(\mathbf{g}_{\left[: n^{\prime}\right]}^{x^{-1}} \circ \mathbf{g}_{\left[n^{\prime}:\right]}^{x}, \mathbf{h}_{\left[: n^{\prime}\right]}^{x} \circ \mathbf{h}_{\left[n^{\prime}:\right]}^{x^{-1}}, u\right)$. The dimension of this problem is only $n^{\prime}=n / 2$.

The resulting $\log _{2} n$ depth recursive protocol is shown in Protocol 2. This $\log _{2} n$ round protocol is public coin and can be made non-interactive using the Fiat-Shamir heuristic. The total communication of Protocol 2 is only $2\left\lceil\log _{2}(n)\right]$ elements in $\mathbb{G}$ plus 2 elements in $\mathbb{Z}_{p}$. Specifically, the prover sends the following terms:

$$
\left(L_{1}, R_{1}\right), \ldots,\left(L_{\log _{2} n}, R_{\log _{2} n}\right), \quad a, b
$$

where $a, b \in \mathbb{Z}_{p}$ are sent at the tail of the recursion. The prover's work is dominated by $8 n$ group exponentiations and the verifier's work by $4 n$ exponentiations. In Section 3.1 we present a more efficient verifier that performs only 1 multi-exponentiation of size $2 n+2 \log (n)$. In Section 6 we present further optimizations.

Proving security. The inner product protocol for the relation (2) is presented in Protocol 1. This protocol uses internally a fixed group element $u \in \mathbb{G}$ for which there is no known discrete$\log$ relation among $\mathbf{g}, \mathbf{h}, u$. The heart of Protocol 1 is Protocol 2 which is a proof system for the relation (3). In Protocol 1 the element $u$ is raised to a verifier chosen power $x$ to ensure that the extracted vectors $\mathbf{a}, \mathbf{b}$ from Protocol 2 satisfy $\langle\mathbf{a}, \mathbf{b}\rangle=c$.

The following theorem shows that Protocol 1 is a proof system for (2).

$$
\begin{align*}
& \mathcal{P}_{\mathrm{IP}} \text { 's input: }(\mathbf{g}, \mathbf{h}, P, c, \mathbf{a}, \mathbf{b}) \\
& \mathcal{I}_{\mathrm{IP}} \text { 's input: }(\mathbf{g}, \mathbf{h}, P, c) \\
& \qquad \mathcal{V}_{\mathrm{IP}}: x \stackrel{\$}{\leftarrow} \mathbb{Z}_{p}^{\star}  \tag{6}\\
& \mathcal{V}_{\mathrm{IP}} \rightarrow \mathcal{P}_{\mathrm{IP}}: x  \tag{7}\\
& P^{\prime}=P \cdot u^{x \cdot c}  \tag{8}\\
& \text { Run Protocol } 2 \text { on Input }\left(\mathbf{g}, \mathbf{h}, u^{x}, P^{\prime} ; \mathbf{a}, \mathbf{b}\right) \tag{9}
\end{align*}
$$

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-15.jpg?height=57&width=755&top_left_y=622&top_left_x=715)

Protocol 1: Proof system for Relation (2) using Protocol 2. Here $u \in \mathbb{G}$ is a fixed group element with an unknown discrete-log relative to $\mathbf{g}, \mathbf{h} \in \mathbb{G}^{n}$.

Theorem 1 (Inner-Product Argument). The argument presented in Protocol 1 for the relation (2) has perfect completeness and statistical witness-extended-emulation for either extracting a nontrivial discrete logarithm relation between $\mathbf{g}, \mathbf{h}, u$ or extracting a valid witness $\mathbf{a}, \mathbf{b}$.

The proof for Theorem 1 is given in Appendix B.

\subsection{Inner-Product Verification through Multi-Exponentiation}

Protocol 2 has a logarithmic number of rounds and in each round the prover and verifier compute a new set of generators $\mathbf{g}^{\prime}, \mathbf{h}^{\prime}$. This requires a total of $4 n$ exponentiations: $2 n$ in the first round, $n$ in the second and $\frac{n}{2^{j-3}}$ in the $j$ th. We can reduce the number of exponentiations to a single multi-exponentiation of size $2 n$ by delaying all the exponentiations until the last round. This technique provides a significant speed-up if the proof is compiled to a non interactive proof using the Fiat-Shamir heuristic (as in Section 4.4).

Let $g$ and $h$ be the generators used in the final round of the protocol and $x_{j}$ be the challenge from the $j$ th round. In the last round the verifier checks that $g^{a} h^{b} u^{a \cdot b}=P$, where $a, b \in \mathbb{Z}_{p}$ are given by the prover. By unrolling the recursion we can express these final $g$ and $h$ in terms of the input generators $\mathbf{g}, \mathbf{h} \in \mathbb{G}^{n}$ as:

$$
g=\prod_{i=1}^{n} g_{i}^{s_{i}} \in \mathbb{G}, \quad h=\prod_{i=1}^{n} h_{i}^{1 / s_{i}} \in \mathbb{G}
$$

where $\mathbf{s}=\left(s_{1}, \ldots, s_{n}\right) \in \mathbb{Z}_{p}^{n}$ only depends on the challenges $\left(x_{1}, \ldots, x_{\log _{2}(n)}\right)$. The scalars $s_{1}, \ldots, s_{n} \in$ $\mathbb{Z}_{p}$ are calculated as follows:

$$
\text { for } i=1, \ldots, n: \quad s_{i}=\prod_{j=1}^{\log _{2}(n)} x_{j}^{b(i, j)} \quad \text { where } \quad b(i, j)= \begin{cases}1 & \text { the } j \text { th bit of } i-1 \text { is } 1 \\ -1 & \text { otherwise }\end{cases}
$$

Now the entire verification check in the protocol reduces to the following single multi-exponentiation

$$
\begin{align*}
& \text { input: }\left(\mathbf{g}, \mathbf{h} \in \mathbb{G}^{n}, u, P \in \mathbb{G} ; \mathbf{a}, \mathbf{b} \in \mathbb{Z}_{p}^{n}\right)  \tag{10}\\
& \mathcal{P}_{\text {IP's input: }}(\mathbf{g}, \mathbf{h}, u, P, \mathbf{a}, \mathbf{b})  \tag{11}\\
& \mathcal{V}_{\text {IP }} \text { 's input: }(\mathbf{g}, \mathbf{h}, u, P)  \tag{12}\\
& \text { output: }\left\{\mathcal{V}_{\text {IP }} \text { accepts or } \mathcal{V}_{\text {IP }} \text { rejects }\right\}  \tag{13}\\
& \text { if } n=1:  \tag{14}\\
& \mathcal{P}_{\text {IP }} \rightarrow \mathcal{V}_{\text {IP }}: a, b \in \mathbb{Z}_{p}  \tag{15}\\
& \mathcal{V}_{\text {IP }} \text { computes } c=a \cdot b \text { and checks if } P=g^{a} h^{b} u^{c}: \tag{16}
\end{align*}
$$

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-16.jpg?height=55&width=576&top_left_y=574&top_left_x=536)

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-16.jpg?height=55&width=169&top_left_y=631&top_left_x=653)

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-16.jpg?height=63&width=814&top_left_y=744&top_left_x=731)

if yes, $\mathcal{V}_{\text {IP }}$ accepts; otherwise it rejects

else: $(n>1)$

$\mathcal{P}_{\mathrm{IP}}$ computes:

$$
\begin{equation*}
n^{\prime}=\frac{n}{2} \tag{19}
\end{equation*}
$$

$$
\begin{equation*}
L=\mathbf{g}_{\left.\left[n^{\prime}\right]\right]}^{\left.\mathbf{a}_{\left[n^{\prime}\right]}\right]} \mathbf{h}_{\left[: n^{\prime}\right]}^{\mathbf{b}_{\left[n^{\prime} \cdot\right]}} u^{c_{L}} \in \mathbb{G} \tag{22}
\end{equation*}
$$

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-16.jpg?height=89&width=419&top_left_y=1297&top_left_x=731)

$\mathcal{P}_{\mathrm{IP}} \rightarrow \mathcal{V}_{\mathrm{IP}}: L, R$

$\mathcal{V}_{\text {IP }}: x \stackrel{\$}{\leftarrow} \mathbb{Z}_{p}^{\star}$

$\mathcal{V}_{\mathrm{IP}} \rightarrow \mathcal{P}_{\mathrm{IP}}: x$

$\mathcal{P}_{\mathrm{IP}}$ and $\mathcal{V}_{\mathrm{IP}}$ compute:

$$
\begin{align*}
& \mathbf{g}^{\prime}=\mathbf{g}_{\left[: n^{\prime}\right]}^{x^{-1}} \circ \mathbf{g}_{\left[n^{\prime}:\right]}^{x} \in \mathbb{G}^{n^{\prime}}  \tag{29}\\
& \mathbf{h}^{\prime}=\mathbf{h}_{\left[:: n^{\prime}\right]}^{x} \circ \mathbf{h}_{\left[n^{\prime}:\right]}^{\left.x^{\prime}\right]} \in \mathbb{G}^{n^{\prime}}  \tag{30}\\
& P^{\prime}=L^{x^{2}} P R^{x^{-2}} \in \mathbb{G}
\end{align*}
$$

$\mathcal{P}_{\mathrm{IP}}$ computes:

$$
\begin{align*}
& \quad \mathbf{a}^{\prime}=\mathbf{a}_{\left[: n^{\prime}\right]} \cdot x+\mathbf{a}_{\left[n^{\prime}:\right]} \cdot x^{-1} \in \mathbb{Z}_{p}^{n^{\prime}}  \tag{33}\\
& \mathbf{b}^{\prime}=\mathbf{b}_{\left[: n^{\prime}\right]} \cdot x^{-1}+\mathbf{b}_{\left[n^{\prime}:\right]} \cdot x \in \mathbb{Z}_{p}^{n^{\prime}} \\
& \text { recursively run Protocol } 2 \text { on input }\left(\mathbf{g}^{\prime}, \mathbf{h}^{\prime}, u, P^{\prime} ; \mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right)
\end{align*}
$$

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-16.jpg?height=60&width=933&top_left_y=2084&top_left_x=650)

Protocol 2: Improved Inner-Product Argument
of size $2 n+2 \log _{2}(n)+1$ :

$$
\mathbf{g}^{a \cdot \mathbf{s}} \cdot \mathbf{h}^{b \cdot \mathbf{s}^{-1}} \cdot u^{a \cdot b} \stackrel{?}{=} P \cdot \prod_{j=1}^{\log _{2}(n)} L_{j}^{\left(x_{j}^{2}\right)} \cdot R_{j}^{\left(x_{j}^{-2}\right)}
$$

Because a multi-exponentiation can be done much faster than $n$ separate exponentiations, as we discuss in Section 6, this leads to a significant savings.

\section{Range Proof Protocol with Logarithmic Size}

We now present a novel protocol for conducting short and aggregatable range proofs. The protocol uses the improved inner product argument from Protocol 1. First, in Section 4.1, we describe how to construct a range proof that requires the verifier to check an inner product between two vectors. Then, in Section 4.2, we show that this check can be replaced with an efficient inner-product argument. In Section 4.3, we show how to efficiently aggregate $m$ range proofs into one short proof. In Section 4.4, we discuss how interactive public coin protocols can be made non-interactive by using the Fiat-Shamir heuristic, in the random oracle model. In Section 4.5 we present an efficient MPC protocol that allows multiple parties to construct a single aggregate range proof. Finally, in Section 4.6, we discuss an extension that enables a switch to quantum-secure range proofs in the future.

\subsection{Inner-Product Range Proof}

We present a protocol which uses the improved inner-product argument to construct a range proof. The proof convinces the verifier that a commitment $V$ contains a number $v$ that is in a certain range, without revealing $v$. Bootle et al. $\left[\mathrm{BCC}^{+} 16\right]$ give a proof system for arbitrary arithmetic circuits, and in Section 5 we show that our improvements to the inner product argument also transfer to this general proof system. It is of course possible to prove that a commitment is in a given range using an arithmetic circuit, and the work of $\left[\mathrm{BCC}^{+} 16\right]$ could be used to construct an asymptotically logarithmic sized range proof (in the length of $v$ ). However, the circuit would need to implement the commitment function, namely a multi-exponentiation for Pedersen commitments, leading to a large complex circuit.

We construct a range proof more directly by exploiting the fact that a Pedersen commitment $V$ is an element in the same group $\mathbb{G}$ that is used to perform the inner product argument. We extend this idea in Section 5 to construct a proof system for circuits that operate on committed inputs.

Formally, let $v \in \mathbb{Z}_{p}$ and let $V \in \mathbb{G}$ be a Pedersen commitment to $v$ using randomness $\gamma$. The proof system will convince the verifier that $v \in\left[0,2^{n}-1\right]$. In other words, the proof system proves the following relation which is equivalent to the range proof relation in Definition 13:

$$
\begin{equation*}
\left\{\left(g, h \in \mathbb{G}, V, n ; v, \gamma \in \mathbb{Z}_{p}\right): V=h^{\gamma} g^{v} \wedge v \in\left[0,2^{n}-1\right]\right\} \tag{36}
\end{equation*}
$$

Let $\mathbf{a}_{L}=\left(a_{1}, \ldots, a_{n}\right) \in\{0,1\}^{n}$ be the vector containing the bits of $v$, so that $\left\langle\mathbf{a}_{L}, \mathbf{2}^{n}\right\rangle=v$. The prover $\mathcal{P}$ commits to $\mathbf{a}_{L}$ using a constant size vector commitment $A \in \mathbb{G}$. It will convince the verifier that $v$ is in $\left[0,2^{n}-1\right]$ by proving that it knows an opening $\mathbf{a}_{L} \in \mathbb{Z}_{p}^{n}$ of $A$ and $v, \gamma \in \mathbb{Z}_{p}$ such that $V=h^{\gamma} g^{v}$ and

$$
\begin{equation*}
\left\langle\mathbf{a}_{L}, \mathbf{2}^{n}\right\rangle=v \quad \text { and } \quad \mathbf{a}_{L} \circ \mathbf{a}_{R}=\mathbf{0}^{n} \quad \text { and } \quad \mathbf{a}_{R}=\mathbf{a}_{L}-\mathbf{1}^{n} \tag{37}
\end{equation*}
$$

This proves that $a_{1}, \ldots, a_{n}$ are all in $\{0,1\}$, as required and that $\mathbf{a}_{L}$ is composed of the bits of $v$. The high level goal of the following protocol is to convert these $2 n+1$ constraints as a single inner-product constraint. This will allow us to use Protocol 1 to efficiently argue that an innerproduct relation holds. To do this we take a random linear combination (chosen by the verifier) of the constraints. If the original constraints were not satisfied then it is inversely proportional in the challenge space unlikely that the combined constraint holds.

Concretley, we use the following observation: to prove that a committed vector $\mathbf{b} \in \mathbb{Z}_{p}^{n}$ satisfies $\mathbf{b}=\mathbf{0}^{n}$ it suffices for the verifier to send a random $y \in \mathbb{Z}_{p}$ to the prover and for the prover to prove that $\left\langle\mathbf{b}, \mathbf{y}^{n}\right\rangle=0$. If $\mathbf{b} \neq \mathbf{0}^{n}$ then the equality will hold with at most negligible probability $n / p$. Hence, if $\left\langle\mathbf{b}, \mathbf{y}^{n}\right\rangle=0$ the verifier is convinced that $\mathbf{b}=\mathbf{0}^{n}$.

Using this observation, and using a random $y \in \mathbb{Z}_{p}$ from the verifier, the prover can prove that (37) holds by proving that

$$
\begin{equation*}
\left\langle\mathbf{a}_{L}, \mathbf{2}^{n}\right\rangle=v \quad \text { and } \quad\left\langle\mathbf{a}_{L}, \mathbf{a}_{R} \circ \mathbf{y}^{n}\right\rangle=0 \quad \text { and } \quad\left\langle\mathbf{a}_{L}-\mathbf{1}^{n}-\mathbf{a}_{R}, \mathbf{y}^{n}\right\rangle=0 \tag{38}
\end{equation*}
$$

We can combine these three equalities into one using the same technique: the verifier chooses a random $z \in \mathbb{Z}_{p}$ and then the prover proves that

$$
z^{2} \cdot\left\langle\mathbf{a}_{L}, \mathbf{2}^{n}\right\rangle+z \cdot\left\langle\mathbf{a}_{L}-\mathbf{1}^{n}-\mathbf{a}_{R}, \mathbf{y}^{n}\right\rangle+\left\langle\mathbf{a}_{L}, \mathbf{a}_{R} \circ \mathbf{y}^{n}\right\rangle=z^{2} \cdot v
$$

This equality can be re-written as:

$$
\begin{equation*}
\left\langle\mathbf{a}_{L}-z \cdot \mathbf{1}^{n}, \mathbf{y}^{n} \circ\left(\mathbf{a}_{R}+z \cdot \mathbf{1}^{n}\right)+z^{2} \cdot \mathbf{2}^{n}\right\rangle=z^{2} \cdot v+\delta(y, z) \tag{39}
\end{equation*}
$$

where $\delta(y, z)=\left(z-z^{2}\right) \cdot\left\langle\mathbf{1}^{n}, \mathbf{y}^{n}\right\rangle-z^{3}\left\langle\mathbf{1}^{n}, \mathbf{2}^{n}\right\rangle \in \mathbb{Z}_{p}$ is a quantity that the verifier can easily calculate. We thus reduced the problem of proving that (37) holds to proving a single inner-product identity.

If the prover could send to the verifier the two vectors in the inner product in (39) then the verifier could check (39) itself, using the commitment $V$ to $v$, and be convinced that (37) holds. However, these two vectors reveal information about $\mathbf{a}_{L}$ and therefore the prover cannot send them to the verifier. We solve this problem by introducing two additional blinding terms $\mathbf{s}_{L}, \mathbf{s}_{R} \in \mathbb{Z}_{p}^{n}$ to blind these vectors.

Specifically, to prove the statement (36), $\mathcal{P}$ and $\mathcal{V}$ engage in the following zero knowledge protocol:
$\mathcal{P}_{\mathrm{IP}}$ on input $v, \gamma$ computes:

$$
\begin{equation*}
\mathbf{a}_{L} \in\{0,1\}^{n} \text { s.t. }\left\langle\mathbf{a}_{L}, \mathbf{2}^{n}\right\rangle=v \tag{40}
\end{equation*}
$$

$\alpha \stackrel{\$}{\leftarrow} \mathbb{Z}_{p}$

$$
\begin{equation*}
A=h^{\alpha} \mathbf{g}^{\mathbf{a}_{L}} \mathbf{h}^{\mathbf{a}_{R}} \in \mathbb{G} \quad / / \quad \text { commitment to } \mathbf{a}_{L} \text { and } \mathbf{a}_{R} \tag{43}
\end{equation*}
$$

$$
\begin{equation*}
\rho \stackrel{\$}{\leftarrow} \mathbb{Z}_{p} \tag{45}
\end{equation*}
$$

$S=h^{\rho} \mathbf{g}^{\mathbf{s}_{L}} \mathbf{h}^{\mathbf{s}_{R}} \in \mathbb{G} \quad / /$ commitment to $\mathbf{s}_{L}$ and $\mathbf{s}_{R}$

$\mathcal{P} \rightarrow \mathcal{V}: A, S$

$\mathcal{V}: y, z \stackrel{\$}{\leftarrow} \mathbb{Z}_{p}^{\star} \quad / /$ challenge points

$\mathcal{V} \rightarrow \mathcal{P}: y, z$

With this setup, let us define two linear vector polynomials $l(X), r(X)$ in $\mathbb{Z}_{p}^{n}[X]$, and a quadratic polynomial $t(X) \in \mathbb{Z}_{p}[X]$ as follows:

$$
\begin{array}{ll}
l(X)=\left(\mathbf{a}_{L}-z \cdot \mathbf{1}^{n}\right)+\mathbf{s}_{L} \cdot X & \in \mathbb{Z}_{p}^{n}[X] \\
r(X)=\mathbf{y}^{n} \circ\left(\mathbf{a}_{R}+z \cdot \mathbf{1}^{n}+\mathbf{s}_{R} \cdot X\right)+z^{2} \cdot \mathbf{2}^{n} & \in \mathbb{Z}_{p}^{n}[X] \\
t(X)=\langle l(X), r(X)\rangle=t_{0}+t_{1} \cdot X+t_{2} \cdot X^{2} & \in \mathbb{Z}_{p}[X]
\end{array}
$$

where the inner product in the definition of $t(X)$ is as in (1). The constant terms of $l(X)$ and $r(X)$ are the inner product vectors in (39). The blinding vectors $\mathbf{s}_{L}$ and $\mathbf{s}_{R}$ ensure that the prover can publish $l(x)$ and $r(x)$ for one $x \in \mathbb{Z}_{p}^{\star}$ without revealing any information about $\mathbf{a}_{L}$ and $\mathbf{a}_{R}$.

The constant term of $t(x)$, denoted $t_{0}$, is the result of the inner product in (39). The prover needs to convince the verifier that this $t_{0}$ satisfies (39), namely

$$
t_{0}=v \cdot z^{2}+\delta(y, z)
$$

To so do, the prover commits to the remaining coefficients of $t(X)$, namely $t_{1}, t_{2} \in \mathbb{Z}_{p}$. It then convinces the verifier that it has a commitment to the coefficients of $t(X)$ by checking the value of $t(X)$ at a random point $x \in \mathbb{Z}_{p}^{\star}$. Specifically, they do:
$\mathcal{P}_{\mathrm{IP}}$ computes:

$$
\begin{aligned}
& \tau_{1}, \tau_{2} \stackrel{\$}{\rightleftarrows} \mathbb{Z}_{p} \\
& T_{i}=g^{t_{i}} h^{\tau_{i}} \in \mathbb{G}, \quad i=\{1,2\} \quad / / \quad \text { commit to } t_{1}, t_{2} \\
& \mathcal{P} \rightarrow \mathcal{V}: T_{1}, T_{2} \\
& \mathcal{V}: x \stackrel{\$}{\rightleftarrows} \mathbb{Z}_{p}^{\star} \\
& \text { // a random challenge }
\end{aligned}
$$

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-20.jpg?height=47&width=183&top_left_y=654&top_left_x=315)

$\mathcal{P}_{\mathrm{IP}}$ computes:

$\mathbf{l}=l(x)=\mathbf{a}_{L}-z \cdot \mathbf{1}^{n}+\mathbf{s}_{L} \cdot x \in \mathbb{Z}_{p}^{n}$

$\mathbf{r}=r(x)=\mathbf{y}^{n} \circ\left(\mathbf{a}_{R}+z \cdot \mathbf{1}^{n}+\mathbf{s}_{R} \cdot x\right)+z^{2} \cdot \mathbf{2}^{n} \in \mathbb{Z}_{p}^{n}$

$$
\begin{equation*}
\hat{t}=\langle\mathbf{l}, \mathbf{r}\rangle \in \mathbb{Z}_{p} \quad / / \quad \hat{t}=t(x) \tag{60}
\end{equation*}
$$

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-20.jpg?height=61&width=209&top_left_y=891&top_left_x=1332)

$\tau_{x}=\tau_{2} \cdot x^{2}+\tau_{1} \cdot x+z^{2} \cdot \gamma \in \mathbb{Z}_{p}$

// blinding value for $\hat{t}$

$\mu=\alpha+\rho \cdot x \in \mathbb{Z}_{p}$

// $\alpha, \rho$ blind $A, S$

$\mathcal{P} \rightarrow \mathcal{V}: \tau_{x}, \mu, \hat{t}, \mathbf{l}, \mathbf{r}$

The verifier checks that $\mathbf{l}$ and $\mathbf{r}$ are in fact $l(x)$ and $r(x)$ and checks that $t(x)=\langle\mathbf{l}, \mathbf{r}\rangle$. In order to construct a commitment to $\mathbf{a}_{R} \circ \mathbf{y}^{n}$ the verifier switches the generators of the commitment from $\mathbf{h} \in$ $\mathbb{G}^{n}$ to $\mathbf{h}^{\prime}=\mathbf{h}^{\left(\mathbf{y}^{-n}\right)}$. This has the effect that $A$ is now a vector commitment to $\left(\mathbf{a}_{L}, \mathbf{a}_{R} \circ \mathbf{y}^{n}\right)$ with respect to the new generators $\left(\mathbf{g}, \mathbf{h}^{\prime}, h\right)$. Similarly $S$ is now a vector commitment to $\left(\mathbf{s}_{L}, \mathbf{s}_{R} \circ \mathbf{y}^{n}\right)$. The remaining steps of the protocol are:

$$
\begin{aligned}
& h_{i}^{\prime}=h_{i}^{\left(y^{-i+1}\right)} \in \mathbb{G}, \quad \forall i \in[1, n] \quad / / \quad \mathbf{h}^{\prime}=\left(h_{1}, h_{2}^{\left(y^{-1}\right)}, h_{3}^{\left(y^{-2}\right)}, \ldots, h_{n}^{\left(y^{-n+1}\right)}\right) \\
& g^{\hat{t}} h^{\tau_{x}} \stackrel{?}{=} V^{z^{2}} \cdot g^{\delta(y, z)} \cdot T_{1}^{x} \cdot T_{2}^{x^{2}} \quad / / \quad \text { check that } \hat{t}=t(x)=t_{0}+t_{1} x+t_{2} x^{2} \\
& P=A \cdot S^{x} \cdot \mathbf{g}^{-z} \cdot\left(\mathbf{h}^{\prime}\right)^{z \cdot \mathbf{y}^{n}+z^{2} \cdot \mathbf{2}^{n}} \in \mathbb{G} \quad / / \quad \text { compute a commitment to } l(x), r(x) \\
& P \stackrel{?}{=} h^{\mu} \cdot \mathbf{g}^{\mathbf{1}} \cdot\left(\mathbf{h}^{\prime}\right)^{\mathbf{r}} \quad / / \text { check that } \mathbf{l}, \mathbf{r} \text { are correct } \\
& \hat{t} \stackrel{?}{=}\langle\mathbf{l}, \mathbf{r}\rangle \in \mathbb{Z}_{p} \quad / / \text { check that } \hat{t} \text { is correct }
\end{aligned}
$$

Equation (65) is the only place where the verifier uses the given Pedersen commitment $V$ to $v$.

Corollary 2 (Range Proof). The range proof presented in Section 4.1 has perfect completeness, perfect special honest verifier zero-knowledge, and computational witness extended emulation.

Proof. The range proof is a special case of the aggregated range proof from section 4.3 with $m=1$. This is therefore a direct corollary of Theorem 3.

\subsection{Logarithmic Range Proof}

Finally, we can describe the efficient range proof that uses the improved inner product argument.

In the range proof protocol from Section 4.1, $\mathcal{P}$ transmits $\mathbf{l}$ and $\mathbf{r}$, whose size is linear in $n$. Our goal is a proof whose size is logarithmic in $n$.

We can eliminate the transfer of $\mathbf{l}$ and $\mathbf{r}$ using the inner-product argument from Section 3 . These vectors are not secret and hence a protocol the only provides soundness is sufficient.

To use the inner-product argument observe that verifying (67) and (68) is the same as verifying that the witness $\mathbf{l}, \mathbf{r}$ satisfies the inner product relation (2) on public input $\left(\mathbf{g}, \mathbf{h}^{\prime}, P h^{-\mu}, \hat{t}\right)$. That is, $P \in \mathbb{G}$ is a commitment to two vectors $\mathbf{l}, \mathbf{r} \in \mathbb{Z}_{p}^{n}$ whose inner product is $\hat{t}$. We can therefore replace (63) with a transfer of $\left(\tau_{x}, \mu, \hat{t}\right)$, as before, and an execution of an inner product argument. Then instead of transmitting $l$ and $\mathbf{r}$, which has a communication cost of $2 \cdot n$ elements, the inner-product argument transmits only $2 \cdot\left\lceil\log _{2}(n)\right\rceil+2$ elements. In total, the prover sends only $2 \cdot\left\lceil\log _{2}(n)\right]+4$ group elements and 5 elements in $\mathbb{Z}_{p}$.

\subsection{Aggregating Logarithmic Proofs}

In many of the range proof applications described in Section 1.2, a single prover needs to perform multiple range proofs at the same time. For example, a confidential transaction often contains multiple outputs, and in fact, most transactions require a so-called change output to send any unspent funds back to the sender. In Provisions $\left[\mathrm{DBB}^{+} 15\right]$ the proof of solvency requires the exchange to conduct a range proof for every single account. Given the logarithmic size of the range proof presented in Section 4.2, there is some hope that we can perform a proof for $m$ values which is more efficient than conducting $m$ individual range proofs. In this section, we show that this can be achieved with a slight modification to the proof system from Section 4.1.

Concretely, we present a proof system for the following relation:

$$
\begin{equation*}
\left\{\left(g, h \in \mathbb{G}, \quad \mathbf{V} \in \mathbb{G}^{m} \quad ; \quad \mathbf{v}, \boldsymbol{\gamma} \in \mathbb{Z}_{p}^{m}\right): V_{j}=h^{\gamma_{j}} g^{v_{j}} \wedge v_{j} \in\left[0,2^{n}-1\right] \quad \forall j \in[1, m]\right\} \tag{69}
\end{equation*}
$$

The prover is very similar to the prover for a simple range proof with $n \cdot m$ bits, with the following slight modifications. In line (41), the prover should compute $\mathbf{a}_{L} \in \mathbb{Z}_{p}^{n \cdot m}$ such that $\left\langle\mathbf{2}^{n}, \mathbf{a}_{L}[(j-1) \cdot n: j \cdot n-1]\right\rangle=v_{j}$ for all $j$ in $[1, m]$, i.e. $\mathbf{a}_{L}$ is the concatenation of all of the bits for every $v_{j}$. We adjust $l(X)$ and $r(X)$ accordingly so that

$$
\begin{align*}
& l(X)=\left(\mathbf{a}_{L}-z \cdot \mathbf{1}^{n \cdot m}\right)+\mathbf{s}_{L} \cdot X \in \mathbb{Z}_{p}^{n \cdot m}[X]  \tag{70}\\
& r(X)=\mathbf{y}^{n \cdot m} \circ\left(\mathbf{a}_{R}+z \cdot \mathbf{1}^{n \cdot m}+\mathbf{s}_{R} \cdot X\right)+\sum_{j=1}^{m} z^{1+j} \cdot\left(\mathbf{0}^{(j-1) \cdot n}\left\|\mathbf{2}^{n}\right\| \mathbf{0}^{(m-j) \cdot n}\right) \in \mathbb{Z}_{p}^{n \cdot m} \tag{71}
\end{align*}
$$

In the computation of $\tau_{x}$, we need to adjust for the randomness of each commitment $V_{j}$, so that $\tau_{x}=\tau_{1} \cdot x+\tau_{2} \cdot x^{2}+\sum_{j=1}^{m} z^{1+j} \cdot \gamma_{j}$. Further, $\delta(y, z)$ is updated to incorporate more cross terms.

$$
\delta(y, z)=\left(z-z^{2}\right) \cdot\left\langle\mathbf{1}^{n \cdot m}, \mathbf{y}^{n \cdot m}\right\rangle-\sum_{j=1}^{m} z^{j+2} \cdot\left\langle\mathbf{1}^{n}, \mathbf{2}^{n}\right\rangle
$$

The verification check (65) needs to be updated to include all the $V_{j}$ commitments.

$$
\begin{equation*}
g^{\hat{t}} h^{\tau_{x}} \stackrel{?}{=} g^{\delta(y, z)} \cdot \mathbf{V}^{z^{2} \cdot \mathbf{z}^{m}} \cdot T_{1}^{x} \cdot T_{2}^{x^{2}} \tag{72}
\end{equation*}
$$

Finally, we change the definition of $P$ (66) such that it is a commitment to the new $\mathbf{r}$.

$$
P=A S^{x} \cdot \mathbf{g}^{-z} \cdot \mathbf{h}^{\prime z \cdot \mathbf{y}^{n \cdot m}} \prod_{j=1}^{m} \mathbf{h}_{[(j-1) \cdot n: j \cdot n-1]}^{\prime z^{j+1} \cdot \mathbf{2}^{n}}
$$

The aggregated range proof which makes use of the inner product argument uses $2 \cdot\left\lceil\log _{2}(n \cdot m)\right]+4$ group elements and 5 elements in $\mathbb{Z}_{p}$. Note that the proof size only grows by an additive term of $2 \cdot \log _{2}(m)$ when conducting multiple range proofs as opposed to a multiplicative factor of $m$ when creating $m$ independent range proofs.

Theorem 3. The aggregate range proof presented in Section 4.3 has perfect completeness, perfect honest verifier zero-knowledge and computational witness extended emulation.

The proof for Theorem 3 is presented in Appendix C. It is analogous to the proof of Theorem 4 which is described in greater detail in Appendix D.

\subsection{Non-Interactive Proof through Fiat-Shamir}

So far we presented the proof as an interactive protocol with a logarithmic number of rounds. The verifier is a public coin verifier, as all the honest verifier's messages are random elements from $\mathbb{Z}_{p}^{\star}$. We can therefore convert the protocol into a non-interactive protocol that is secure and full zero-knowledge in the random oracle model using the Fiat-Shamir transform [BR93]. All random challenges are replaced by hashes of the transcript up to that point, including the statement itself. Subsequent works have shown that this approach is secure, even for multi-round protocols [Wik21, AFK21].

For example, one could set $y=\mathrm{H}(\mathrm{st}, A, S)$ and $z=\mathrm{H}(A, S, y)$, where st is the statement. For a range proof st would be $\{V, n\}$, and for a circuit proof it would be the description of the circuit. It is very important to include the statement st in the hash as otherwise an adversary can prove invalid statements, as pointed out in a blog post ${ }^{2}$. Since implementing Fiat-Shamir can be error-prone, we recommend using an established library to do so, such as Merlin ${ }^{3}$, which was developed as part of an implementation of Bulletproofs in Rust ${ }^{4}$.

To avoid a trusted setup we can use a hash function to generate the public parameters $\mathbf{g}, \mathbf{h}, g, h$ from a small seed. The hash function needs to map $\{0,1\}^{*}$ to $\mathbb{G}$, which can be built as in [BLS01]. This also makes it possible to provide random access to the public parameters. Alternatively, a common random string can be used.

\subsection{A Simple MPC Protocol for Bulletproofs}

In several of the applications described in Section 1.2, the prover could potentially consist of multiple parties who each want to generate a single range proof. For instance, multiple parties may want to create a single joined confidential transaction, where each party knows some of the inputs and outputs and needs to create range proofs for their known outputs. The joint transaction would not only be smaller than the sum of multiple transactions, it would also hide which inputs correspond to which outputs and provide some level of anonymity. These kinds of transactions are called CoinJoin transactions [Max13]. In Provisions, an exchange may distribute the private keys to multiple servers and split the customer database into separate chunks, but it still needs to produce a single short proof of solvency. Can these parties generate one Bulletproof without sharing the entire witness with each other? The parties could certainly use generic multi-party computation techniques to generate a single proof, but this might be too expensive and incur significant communication costs.


\footnotetext{
${ }^{2}$ https://blog.trailofbits.com/2022/04/13/part-1-coordinated-disclosure-of-vulnerabilities-affecting-girault-bul

${ }^{3}$ https://github.com/zkcrypto/merlin

${ }^{4}$ https://github.com/zkcrypto/bulletproofs
}

This motivates the need for a simple MPC protocol specifically designed for Bulletproofs which requires little modification to the prover and is still efficient.

Note that for aggregate range proofs, the inputs of one range proof do not affect the output of another range proof. Given the composable structure of Bulletproofs, it turns out that $m$ parties each having a Pedersen commitment $\left(V_{k}\right)_{k=1}^{m}$ can generate a single Bulletproof that each $V_{k}$ commits to a number in some fixed range. The protocol either uses a constant number of rounds but communication that is linear in both $m$ and the binary encoding of the range, or it uses a logarithmic number of rounds and communication that is only linear in $m$. We assume for simplicity that $m$ is a power of 2 , but the protocol could be easily adapted for other $m$. We use the same notation as in the aggregate range proof protocol, but use $k$ as an index to denote the $k$ th party's message. That is $A^{(k)}$ is generated just like $A$ but using only the inputs of party $k$.

The MPC protocol works as follows, we assign a set of distinct generators $\left(\mathbf{g}^{(k)}, \mathbf{h}^{(k)}\right)_{k=1}^{m}$ to each party and define $\mathbf{g}$ as the interleaved concatenation of all $\mathbf{g}^{(k)}$ such that $g_{i}=g_{\left\lceil\frac{i}{m}\right\rceil}^{((i-1) \bmod m+1)}$. Define $\mathbf{h}$ and $\mathbf{h}^{(k)}$ in an analogous way.

We first describe the protocol with linear communication. In each of the 3 rounds of the protocol, the ones that correspond to the rounds of the range proof in Section 4.1, each party simply generates its part of the proof, i.e. the $A^{(k)}, S^{(k)} ; T_{1}^{(k)}, T_{2}^{(k)} ; \tau_{x}^{(k)}, \mu^{(k)}, \hat{t}^{(k)}, \mathbf{l}^{(k)}, \mathbf{r}^{(k)}$ using its inputs and generators. These shares are then sent to a dealer (which could be one of the parties), who simply adds them homomorphically to generate the respective proof component, e.g.

$A=\prod_{k=1}^{l} A^{(k)}$ and $\tau_{x}=\sum_{k=1}^{l} \tau_{x}^{(k)}$. In each round, the dealer generates the challenges using the Fiat-Shamir heuristic and the combined proof components and sends them to each party. Finally, each party sends $\mathbf{l}^{(k)}, \mathbf{r}^{(k)}$ to the dealer who computes $\mathbf{l}, \mathbf{r}$ as the interleaved concatenation of the shares. The dealer runs the inner product argument and generates the final proof. The protocol is complete as each proof component is simply the (homomorphic) sum of each parties' proof components, and the challenges are generated as in the original protocol. It is also secure against honest but curious adversaries as each share constitutes part of a separate zero-knowledge proof.

The communication can be reduced by running a second MPC protocol for the inner product argument. The generators were selected in such a way that up to the last $\log _{2}(l)$ rounds each parties' witnesses are independent and the overall witness is simply the interleaved concatenation of the parties' witnesses. Therefore, parties simply compute $L^{(k)}, R^{(k)}$ in each round and a dealer computes $L, R$ as the homomorphic sum of the shares. The dealer then again generates the challenge and sends it to each party. In the final round the parties send their witness to the dealer who completes Protocol 2. A similar protocol can be used for arithmetic circuits if the circuit is decomposable into separate independent circuits. Constructing an efficient MPC protocol for more complicated circuits remains an open problem.

\subsection{Perfectly Binding Commitments and Proofs}

Bulletproofs, like the range proofs currently used in confidential transactions, are computationally binding. An adversary that could break the discrete logarithm assumption could generate acceptable range proofs for a value outside the correct range. On the other hand, the commitments are perfectly hiding and Bulletproofs are perfect zero-knowledge, so that even an all powerful adversary cannot learn which value was committed to. Commitment schemes which are simultaneously perfectly-binding and perfectly-hiding commitments are impossible, so when designing commitment schemes and proof systems, we need to decide which properties are more important. For cryptocur-
rencies, the binding property is more important than the hiding property [RM]. An adversary that can break the binding property of the commitment scheme or the soundness of the proof system can generate coins out of thin air and thus create uncontrolled but undetectable inflation rendering the currency useless. Giving up the privacy of a transaction is much less harmful as the sender of the transaction or the owner of an account is harmed at worst. Unfortunately, it seems difficult to create Bulletproofs from binding commitments. The efficiency of the system relies on vector commitments which allow the commitment to a long vector in a single group element. By definition, for perfectly binding commitment schemes, the size of the commitment must be at least the size of the message and compression is thus impossible. The works [GH98, GVW02] show that in general, interactive proofs cannot have communication costs smaller than the witness size, unless some very surprising results in complexity theory hold.

While the discrete logarithm assumption is believed to hold for classical computers, it does not hold against a quantum adversary. It is especially problematic that an adversary can create a perfectly hiding UTXO at any time, planning to open to an arbitrary value later when quantum computers are available. To defend against this, we can use the technique from Ruffing and Malavolta $[\mathrm{RM}]$ to ensure that even though the proof is only computationally binding, it is later possible to switch to a proof system that is perfectly binding and secure against quantum adversaries. In order to do this, the prover simply publishes $g^{\gamma}$, which turns the Pedersen commitment to $v$ into an ElGamal commitment. Ruffing and Malavolta also show that given a small message space, e.g. numbers in the range $\left[0,2^{n}\right]$, it is impossible for a computationally bounded prover to construct a commitment that an unbounded adversary could open to a different message in the small message space.

Note that the commitment is now only computationally hiding, but that switching to quantumsecure range proofs is possible. Succinct quantum-secure range proofs remain an open problem, but with a slight modification, the scheme from Poelstra et al. $\left[\mathrm{PBF}^{+}\right]$can achieve statistical soundness. Instead of using Pedersen commitments, we propose using ElGamal commitments in every step of the protocol. An ElGamal commitment is a Pedersen commitment with an additional commitment $g^{r}$ to the randomness used. The scheme can be improved slightly if the same $g^{r}$ is used in multiple range proofs. In order to retain the hiding property, a different $h$ must be used for every proof.

\section{Zero-Knowledge Proof for Arithmetic Circuits}

Bootle et al. $\left[\mathrm{BCC}^{+} 16\right]$ present an efficient zero-knowledge argument for arbitrary arithmetic circuits using $6 \log _{2}(n)+13$ elements, where $n$ is the multiplicative complexity of the circuit. We can use our improved inner product argument to get a proof of size $2 \log _{2}(n)+13$ elements, while simultaneously generalizing to include committed values as inputs to the arithmetic circuit. Including committed input wires is important for many applications (notably range proofs) as otherwise the circuit would need to implement a commitment algorithm. Concretely a statement about Pedersen commitments would need to implement the group exponentiation for the group that the commitment is an element of.

Following $\left[\mathrm{BCC}^{+} 16\right]$, we present a proof for a Hadamard-product relation. A multiplication gate of fan-in 2 has three wires; 'left' and 'right' for the input wires, and 'output' for the output wire. In the relation, $\mathbf{a}_{L}$ is the vector of left inputs for each multiplication gate. Similarly, $\mathbf{a}_{R}$ is the vector of right inputs, and $\mathbf{a}_{O}=\mathbf{a}_{L} \circ \mathbf{a}_{R}$ is the vector of outputs. $\left[\mathrm{BCC}^{+} 16\right]$ shows how to convert an arbitrary arithmetic circuit with $n$ multiplication gates into a relation containing a

Hadamard-product as above, with an additional $Q \leqslant 2 \cdot n$ linear constraints of the form

$$
\left\langle\mathbf{w}_{L, q}, \mathbf{a}_{L}\right\rangle+\left\langle\mathbf{w}_{R, q}, \mathbf{a}_{R}\right\rangle+\left\langle\mathbf{w}_{O, q}, \mathbf{a}_{O}\right\rangle=c_{q}
$$

for $1 \leqslant q \leqslant Q$, with $\mathbf{w}_{L, q}, \mathbf{w}_{R, q}, \mathbf{w}_{O, q} \in \mathbb{Z}_{p}^{n}$ and $c_{q} \in \mathbb{Z}_{p}$.

We include additional commitments $V_{i}$ as part of our statement, and give a protocol for a more general relation, where the linear consistency constraints include the openings $v_{j}$ of the commitments $V_{j}$. For simplicity and efficiency we present the scheme with $V_{i}$ being Pedersen commitments. The scheme can be trivially adapted to work with other additively homomorphic schemes by changing the commitments to $t(X)$ and adapting the verification in line (90).

\subsection{Inner-Product Proof for Arithmetic Circuits}

The high level idea of the protocol is to convert the Hadamard-product relation along with the linear constraints into a single inner product relation. Similar to the range proof protocol the prover verifiably produces a random linear combination of the Hadamard and the linear constraints to form a single inner product constraint. If the combination is chosen randomly by the verifier, as in our protocol, then with overwhelming probability the inner-product constraint implies the other constraints.

In Section 5.2 we show that the inner product relation can be replaced with an efficient inner product argument which yields short proofs for arbitrary circuits where input wires can come from Pedersen commitments. Formally we present a proof system for the following relation.

$$
\begin{align*}
& \left\{\left(g, h \in \mathbb{G}, \mathbf{g}, \mathbf{h} \in \mathbb{G}^{n}, \mathbf{V} \in \mathbb{G}^{m}, \mathbf{W}_{L}, \mathbf{W}_{R}, \mathbf{W}_{O} \in \mathbb{Z}_{p}^{Q \times n}, \mathbf{W}_{V} \in \mathbb{Z}_{p}^{Q \times m}, \mathbf{c} \in \mathbb{Z}_{p}^{Q} ; \mathbf{a}_{L}, \mathbf{a}_{R}, \mathbf{a}_{O} \in \mathbb{Z}_{p}^{n}, \mathbf{v}, \boldsymbol{\gamma} \in \mathbb{Z}_{p}^{m}\right):\right. \\
& \left.V_{j}=g^{v_{j}} h^{\gamma_{j}} \forall j \in[1, m] \wedge \mathbf{a}_{L} \circ \mathbf{a}_{R}=\mathbf{a}_{O} \wedge \mathbf{W}_{L} \cdot \mathbf{a}_{L}+\mathbf{W}_{R} \cdot \mathbf{a}_{R}+\mathbf{W}_{O} \cdot \mathbf{a}_{O}=\mathbf{W}_{V} \cdot \mathbf{v}+\mathbf{c}\right\} \tag{73}
\end{align*}
$$

Let $\mathbf{W}_{V} \in \mathbb{Z}_{p}^{Q \times m}$ be the weights for a commitment $V_{j}$. The presented proof system only works for relations where $\mathbf{W}_{V}$ is of rank $m$, i.e. the columns of the matrix are all linearly independent. This restriction is minor as we can construct commitments that fulfill these linearly dependent constraints as a homomorphic combination of other commitments. Consider a vector $\mathbf{w}_{V}^{\prime}=\mathbf{a} \cdot \mathbf{W}_{V} \in \mathbb{Z}_{p}^{m}$ for a

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-25.jpg?height=61&width=1644&top_left_y=1582&top_left_x=238)
holds then we can conclude that $\left\langle\mathbf{w}_{L, j}, \mathbf{a}_{L}\right\rangle+\left\langle\mathbf{w}_{R, j}, \mathbf{a}_{R}\right\rangle+\left\langle\mathbf{w}_{O, j}, \mathbf{a}_{O}\right\rangle=\left\langle\mathbf{w}_{V}^{\prime}, \mathbf{v}\right\rangle+\mathbf{c}$. The protocol is presented in Protocol 3. It is split into two parts. In the first part $\mathcal{P}$ commits to $l(X), r(X), t(X)$ in the second part $\mathcal{P}$ convinces $\mathcal{V}$ that the polynomials are well formed and that $\langle l(X), r(X)\rangle=t(X)$.

Theorem 4. The proof system presented in Protocol 3 has perfect completeness, perfect honest verifier zero-knowledge and computational witness extended emulation.

The proof of Theorem 4 is presented in Appendix D.

\subsection{Logarithmic-Sized Protocol}

As for the range proof, we can reduce the communication cost of the protocol by using the inner product argument. Concretely transfer (82) is altered to simply $\tau_{x}, \mu, \hat{t}$ and additionally $\mathcal{P}$ and $\mathcal{V}$ engage in an inner product argument on public input $\left(\mathbf{g}, \mathbf{h}^{\prime}, P \cdot h^{-\mu}, \hat{t}\right)$. Note that the statement proven is equivalent to the verification equations (92) and (88). The inner product argument has only logarithmic communication complexity and is thus highly efficient. Note that instead

Input: $\left(g, h \in \mathbb{G}, \mathbf{g}, \mathbf{h} \in \mathbb{G}^{n}, \mathbf{W}_{L}, \mathbf{W}_{R}, \mathbf{W}_{O} \in \mathbb{Z}_{p}^{Q \times n}\right.$,

$\left.\mathbf{W}_{V} \in \mathbb{Z}_{p}^{Q \times m}, \mathbf{c} \in \mathbb{Z}_{p}^{Q} ; \mathbf{a}_{L}, \mathbf{a}_{R}, \mathbf{a}_{O} \in \mathbb{Z}_{p}^{n}, \gamma \in \mathbb{Z}_{p}^{m}\right)$

$\mathcal{P}$ 's input: $\left(g, h, \mathbf{g}, \mathbf{h}, \mathbf{W}_{L}, \mathbf{W}_{R}, \mathbf{W}_{O}, \mathbf{W}_{V}, \mathbf{c} ; \mathbf{a}_{L}, \mathbf{a}_{R}, \mathbf{a}_{O}, \boldsymbol{\gamma}\right)$

$\mathcal{V}$ 's input: $\left(g, h, \mathbf{g}, \mathbf{h}, \mathbf{W}_{L}, \mathbf{W}_{R}, \mathbf{W}_{O}, \mathbf{W}_{V}, \mathbf{c}\right)$

Output: $\{\mathcal{V}$ accepts, $\mathcal{V}$ rejects $\}$

$\mathcal{P}$ computes:

$$
\begin{array}{lll}
\alpha, \beta, \rho \stackrel{\$}{\leftarrow} \mathbb{Z}_{p} & & \\
A_{I}=h^{\alpha} \mathbf{g}^{\mathbf{a}_{L}} \mathbf{h}^{\mathbf{a}_{R}} \in \mathbb{G} & & \text { commit to } \mathbf{a}_{L}, \mathbf{a}_{R} \\
A_{O}=h^{\beta} \mathbf{g}^{\mathbf{a}} \in \mathbb{G} & & \text { commitment to } \mathbf{a}_{O} \\
& / / & \text { choose blinding vectors } \mathbf{s}_{L}, \mathbf{s}_{R} \\
\mathbf{s}_{L}, \mathbf{s}_{R} \stackrel{\$}{\leftarrow} \mathbb{Z}_{p}^{n} & / / & \text { commitment to } \mathbf{s}_{L}, \mathbf{s}_{R} \\
S=h^{\rho} \mathbf{s}^{\mathbf{s}_{L}} \mathbf{h}^{\mathbf{s}_{R}} \in \mathbb{G} & \\
\mathcal{P} \rightarrow \mathcal{V}: A_{I}, A_{O}, S & & \\
\mathcal{V}: y, z \stackrel{\$}{\leftarrow} \mathbb{Z}_{p}^{\star} & & \\
\mathcal{V} \rightarrow \mathcal{P}: y, z & & \\
\mathcal{P} \text { and } \mathcal{V} \text { compute: } & \text { challenge per witness } \\
\mathbf{y}^{n}=\left(1, y, y^{2}, \ldots, y^{n-1}\right) \in \mathbb{Z}_{p}^{n} & & \text { challenge per constraint } \\
\mathbf{z}_{[1:]}^{Q+1}=\left(z, z^{2}, \ldots, z^{Q}\right) \in \mathbb{Z}_{p}^{Q} & \text { independent of the witness } \\
\delta(y, z)=\left\langle\mathbf{y}^{-n} \circ\left(\mathbf{z}_{[1:]}^{Q+1} \cdot \mathbf{W}_{R}\right), \mathbf{z}_{[1:]}^{Q+1} \cdot \mathbf{W}_{L}\right\rangle &
\end{array}
$$

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-26.jpg?height=48&width=285&top_left_y=1212&top_left_x=302)

$\mathcal{P}$ computes:

$$
\begin{aligned}
& l(X)=\mathbf{a}_{L} \cdot X+\mathbf{a}_{O} \cdot X^{2}+\mathbf{y}^{-n} \circ\left(\mathbf{z}_{[1:]}^{Q+1} \cdot \mathbf{W}_{R}\right) \cdot X \\
& \quad+\mathbf{s}_{L} \cdot X^{3} \in \mathbb{Z}_{p}^{n}[X] \\
& r(X)=\mathbf{y}^{n} \circ \mathbf{a}_{R} \cdot X-\mathbf{y}^{n}+\mathbf{z}_{[1:]}^{Q+1} \cdot\left(\mathbf{W}_{L} \cdot X+\mathbf{W}_{O}\right) \\
& \quad+\mathbf{y}^{n} \circ \mathbf{s}_{R} \cdot X^{3} \in \mathbb{Z}_{p}^{n}[X] \\
& t(X)=\langle l(X), r(X)\rangle=\sum_{i=1}^{6} t_{i} \cdot X^{i} \in \mathbb{Z}_{p}[X] \\
& \mathbf{w}=\mathbf{W}_{L} \cdot \mathbf{a}_{L}+\mathbf{W}_{R} \cdot \mathbf{a}_{R}+\mathbf{W}_{O} \cdot \mathbf{a}_{O} \\
& t_{2}=\left\langle\mathbf{a}_{L}, \mathbf{a}_{R} \circ \mathbf{y}^{n}\right\rangle-\left\langle\mathbf{a}_{O}, \mathbf{y}^{n}\right\rangle+\left\langle\mathbf{z}_{[1:]}^{Q+1}, \mathbf{w}\right\rangle+\delta(y, z) \in \mathbb{Z}_{p} \quad / \quad t_{2}=d(y, z)+\left\langle\mathbf{z}_{[1:]}^{Q+1}, \mathbf{c}+\mathbf{W}_{V} \cdot \mathbf{v}\right\rangle \\
& \tau_{i} \stackrel{\$}{\leftarrow} \mathbb{Z}_{p} \quad \forall i \in[1,3,4,5,6] \\
& T_{i}=g^{t_{i}} h^{\tau_{i}} \quad \forall i \in[1,3,4,5,6] \\
& \mathcal{P} \rightarrow \mathcal{V}: T_{1}, T_{3}, T_{4}, T_{5}, T_{6}
\end{aligned}
$$

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-26.jpg?height=60&width=541&top_left_y=2103&top_left_x=1231)

Protocol 3: Part 1: Computing commitments to $l(X), r(X)$ and $t(X)$

$$
\begin{aligned}
& \mathcal{V}: x \stackrel{\$}{\rightleftarrows} \mathbb{Z}_{p}^{\star} \\
& \text { // Random challenge } \\
& \mathcal{V} \rightarrow \mathcal{P}: x \\
& \mathcal{P} \text { computes: } \\
& \mathbf{l}=l(x) \in \mathbb{Z}_{p}^{n} \\
& \mathbf{r}=r(x) \in \mathbb{Z}_{p}^{n} \\
& \hat{t}=\langle\mathbf{l}, \mathbf{r}\rangle \in \mathbb{Z}_{p} \\
& \tau_{x}=\sum_{i=1, i \neq 2}^{6} \tau_{i} \cdot x^{i}+x^{2} \cdot\left\langle\mathbf{z}_{[1:]}^{Q+1}, \mathbf{W}_{V} \cdot \gamma\right\rangle \in \mathbb{Z}_{p} \quad / / \quad \text { blinding value for } \hat{t} \\
& \mu=\alpha \cdot x+\beta \cdot x^{2}+\rho \cdot x^{3} \in \mathbb{Z}_{p} \quad / / \quad \text { Blinding value for } P \\
& \mathcal{P} \rightarrow \mathcal{V}: \tau_{x}, \mu, \hat{t}, \mathbf{l}, \mathbf{r} \\
& h_{i}^{\prime}=h_{i}^{y^{-i+1}} \quad \forall i \in[1, n] \\
& W_{R}=\mathbf{g}^{\mathbf{y}^{-n} \circ\left(\mathbf{z}_{[1:]}^{Q+1} \cdot \mathbf{W}_{R}\right)} \\
& W_{O}=\mathbf{h}^{\mathbf{z}_{[1:]}^{Q+1} \cdot \mathbf{W}_{O}} \\
& g^{\hat{t}} h^{\tau_{x}} \stackrel{?}{=} g^{x^{2} \cdot\left(\delta(y, z)+\left\langle\mathbf{z}_{[1:]}^{Q+1}, \mathbf{c}\right\rangle\right)} \cdot \mathbf{V}^{x^{2} \cdot\left(\mathbf{z}_{[1:]}^{Q+1} \cdot \mathbf{W}_{V}\right)} \cdot T_{1}^{x} \\
& \cdot \prod_{i=3}^{6} T_{i}^{\left(x^{i}\right)} \\
& / / \quad \hat{t}=t(x)=\sum_{i=1}^{6} t_{i} \cdot x^{i} \\
& P=A_{I}^{x} \cdot A_{O}^{\left(x^{2}\right)} \cdot \mathbf{h}^{\prime-\mathbf{y}^{n}} \cdot W_{L}^{x} \cdot W_{R}^{x} \cdot W_{O} \cdot S^{\left(x^{3}\right)} \\
& \text { // commitment to } l(x), r(x) \\
& P \stackrel{?}{=} h^{\mu} \cdot \mathbf{g}^{\mathbf{1}} \cdot \mathbf{h}^{\prime \mathbf{r}} \\
& \text { // Check that } \mathbf{l}=l(x) \text { and } \mathbf{r}=r(x)
\end{aligned}
$$

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-27.jpg?height=52&width=386&top_left_y=1161&top_left_x=325)

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-27.jpg?height=63&width=481&top_left_y=1215&top_left_x=1153)

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-27.jpg?height=64&width=264&top_left_y=1280&top_left_x=361)

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-27.jpg?height=53&width=316&top_left_y=1296&top_left_x=1151)

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-27.jpg?height=52&width=318&top_left_y=1362&top_left_x=1153)

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-27.jpg?height=55&width=318&top_left_y=1431&top_left_x=1153)

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-27.jpg?height=69&width=156&top_left_y=1489&top_left_x=361)

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-27.jpg?height=58&width=426&top_left_y=1494&top_left_x=1153)

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-27.jpg?height=52&width=500&top_left_y=1866&top_left_x=327)

Protocol 3: Part 2: Polynomial identity check for $\langle l(x), r(x)\rangle=t(x)$
of transmitting $\mathbf{l}, \mathbf{r}$ the inner product argument only requires communication of $2 \cdot\left\lceil\log _{2}(n)\right\rceil+2$ elements instead of $2 \cdot n$. In total the prover sends $2 \cdot\left[\log _{2}(n)\right]+8$ group elements and 5 elements in $\mathbb{Z}_{p}$. Using the Fiat-Shamir heuristic as in 4.4 the protocol can be turned into an efficient non interactive proof. We report implementation details and evaluations in Section 6.

Theorem 5. The arithmetic circuit protocol using the improved inner product argument (Protocol 2) has perfect completeness, statistical zero-knowledge and computational soundness under the discrete logarithm assumption.

Proof. Completeness follows from the completeness of the underlying protocols. Zero-knowledge follows from the fact that $\mathbf{l}$ and $\mathbf{r}$ can be efficiently simulated, and because the simulator can simply run Protocol 2 given the simulated witness $(\mathbf{l}, \mathbf{r})$. The protocol also has a knowledgeextractor, as the extractor of the range proof can be extended to extract $\mathbf{l}$ and $\mathbf{r}$ by calling the extractor of Protocol 2. The extractor uses $O\left(n^{3}\right)$ valid transcripts in total, which is polynomial in $\lambda$ if $n=O(\lambda)$. The extractor is thus efficient and either extracts a discrete logarithm relation or a valid witness. However, if the generators $\mathbf{g}, \mathbf{h}, g, h$ are independently generated, then finding a discrete logarithm relation between them is as hard as breaking the discrete log problem. If the discrete $\log$ assumption holds in $\mathbb{G}$ then a computationally bounded $\mathcal{P}$ cannot produce discretelogarithm relations between independent generators. The proof system is therefore computationally sound.

\section{Performance}

\subsection{Theoretical Performance}

In Table 1 we give analytical measurements for the proof size of different range proof protocols. We compare both the proof sizes for a single proof and for $m$ proofs for the range $\left[0,2^{n}-1\right]$. We compare Bulletproofs against $\left[\mathrm{PBF}^{+}\right]$and a $\Sigma$-protocol range proof where the proof commits to each bit and then shows that the commitment is to 0 or 1 . The table shows that Bulletproofs

Table 1: Range proof size for $m$ proofs. $m=1$ is the special case of a single range proof

\begin{tabular}{l|l|l|}
\multicolumn{2}{l}{$m$ range proofs for range $\left[0,2^{n}-1\right]$} \\
& $\# \mathbb{G}$ elements & $\mid \# \mathbb{Z}_{p}$ elements \\
\hline$\Sigma$ Protocol [CD98] & $m n$ & $3 m n+1$ \\
Poelstra et al. $\left[\mathrm{PBF}^{+}\right]$ & $0.63 \cdot m n$ & $1.26 \cdot m n+1$ \\
Bulletproofs & $2\left(\log _{2}(n)+\log _{2}(m)\right)+4$ & 5 \\
\hline
\end{tabular}

have a significant advantage when providing multiple range proofs at once. The proof size for the protocol presented in Section 4.3 only grows by an additive logarithmic factor when conducting $m$ range proofs, while all other solutions grow multiplicatively in $m$.

\subsection{An Optimized Verifier Using Multi-Exponentiation and Batch Verification}

In many of the applications discussed in Section 1.2 the verifier's runtime is of particular interest. For example, with confidential transactions every full node needs to check all confidential transac-
tions and all associated range proofs. We therefore now present a number of optimizations for the non-interactive verifier. We present the optimizations for a single range proof but they all carry over to aggregate range proofs and the arithmetic circuit protocol.

Single multi-exponentiation. In Section 3.1 we showed that the verification of the inner product can be reduce to a single multi-exponentiation. We can further extend this idea to verify the whole range proof using a single multi-exponentiation of size $2 n+2 \log _{2}(n)+7$. Notice that the Bulletproofs verifier only performs two checks (68) and (16). The idea is to delay exponentiation until those checks are actually performed and then to combine them into a single check. We, therefore, unroll the inner product argument as described in Section 3.1 using the input from the range proof. The resulting protocol is presented below with $x_{u}$ being the challenge from Protocol 1 , and $x_{j}$ being the challenge from round $j$ of Protocol 2. $L_{j}$ and $R_{j}$ are the $L, R$ values from round $j$ of Protocol 2 . The verifier runs the following verification procedure:

$$
\begin{align*}
& \text { input: proof } \pi=\left\{A, S, T_{1}, T_{2},\left(L_{j}, R_{j}\right)_{j=1}^{\log (n)} \in \mathbb{G}, \tau, \hat{t}, \mu, a, b \in \mathbb{Z}_{p}\right\}  \tag{95}\\
& \text { compute challenges from } \pi:\left\{y, z, x, x_{u},\left(x_{j}\right)_{j=1}^{\log _{2}(n)}\right\}  \tag{96}\\
& \delta(y, z)=\left(z-z^{2}\right) \cdot\left\langle\mathbf{1}^{n}, \mathbf{y}^{n}\right\rangle-z^{3}\left\langle\mathbf{1}^{n}, \mathbf{2}^{n}\right\rangle  \tag{97}\\
& g^{\hat{t}-\delta(y, z)} h^{\tau_{x}} \cdot V^{-z^{2}} \cdot T_{1}^{-x} \cdot T_{2}^{-x^{2}} \stackrel{?}{=} 1  \tag{98}\\
& b(i, j)= \begin{cases}1 & \text { if the } j \text { th bit of } i-1 \text { is } 1 \\
-1 & \text { otherwise }\end{cases}  \tag{99}\\
& \text { for } i=1, \ldots, n \text { : }  \tag{100}\\
& \quad l_{i}=\prod_{j=1}^{\log _{2} n} x_{j}^{b(i, j)} \cdot a+z \in \mathbb{Z}_{p}  \tag{101}\\
& \quad r_{i}=y^{1-i} \cdot\left(\prod_{j=1}^{\log _{2} n} x_{j}^{-b(i, j)} \cdot b-z^{2} \cdot 2^{i-1}\right)-z \in \mathbb{Z}_{p}  \tag{102}\\
& \mathbf{l}=\left(l_{1}, \ldots, l_{n}\right) \in \mathbb{Z}_{p}^{n}  \tag{103}\\
& \mathbf{r}=\left(r_{1}, \ldots, r_{n}\right) \in \mathbb{Z}_{p}^{n}  \tag{104}\\
& \mathbf{g}^{\mathbf{l}} \mathbf{h}^{\mathbf{r}} g^{x_{u} \cdot(a \cdot b-\hat{t})} h^{\mu} \cdot A^{-1} S^{-x} \prod_{j=1}^{\log _{2}(n)} L_{j}^{-x_{j}^{2}} R_{j}^{-x_{j}^{-2}} \stackrel{?}{=} 1 \tag{105}
\end{align*}
$$

We can combine the two multi-exponentiations in line (98) and (105) by using a random value $c \stackrel{\$}{\leftarrow} \mathbb{Z}_{p}$. This is because if $A^{c} B=1$ for a random $c$ then with high probability $A=1 \wedge B=1$.

Various algorithms are known to compute the multi-exponentiations (105) and (98) efficiently. As explained in [BDLO12], algorithms like Pippenger's [Pip80] perform a number of group operations that scales with $O \frac{n}{\log (n)}$, i.e. sub-linearly. For realistic problem sizes these dominate verification time.

Computing scalars. A further optimization concerns the computation of the $l_{i}$ and $r_{i}$ values. Instead of computing $x^{(i)}=\prod_{j=1}^{\log _{2} n} x_{j}^{b(i, j)}$ for each $i$, we can compute each challenge product using only one multiplication in $\mathbb{Z}_{p}$ by applying batch division. First we compute $x^{(1)}=\left(\prod_{j=1}^{\log _{2} n} x_{j}\right)^{-1}$ to
get the first challenge value using a single inversion. Then computing $x^{(2)}=x^{(1)} x_{1}^{2}, \quad x^{3}=x^{(1)} x_{2}^{2}$, and for example $x^{(7)}=x^{(3)} x_{5}^{2}$. In general in order to compute $x^{(i)}$ we let $k$ be the next lower power of 2 of $i-1$ and compute $x^{(i)}=x^{(i-k)} \cdot x_{k+1}^{2}$ which takes only one additional multiplication in $\mathbb{Z}_{p}$ and no inversion. Further, note that the squares of the challenges are computed anyway in order to check equation (105).

Batch verification. A further important optimization concerns the verification of multiple proofs. In many applications described in Section 1.2 the verifier needs to verify multiple (separate) range proofs at once. For example a Bitcoin node receiving a block of transactions needs to verify all transactions and thus range proofs in parallel. As noted above, verification boils down to a large multi-exponentiation. In fact, $2 n+2$ of the generators only depend on the public parameters, and only $2 \log (n)+5$ are proof-dependent. We can therefore apply batch verification [BGR98] in order to reduce the number of expensive exponentiations. Batch verification is based on the observation that checking $g^{x}=1 \wedge g^{y}=1$ can be checked by drawing a random scalar $\alpha$ from a large enough domain and checking $g^{\alpha \cdot x+y}=1$. With high probability, the latter equation implies that $g^{x}=1 \wedge g^{y}=1$, but the latter is more efficient to check. The same trick applies to multi-exponentiations and can save $2 n$ exponentiations per additional proof. This is equivalent to the trick that is used for combining multiple exponentiations into one with the difference that the bases are equivalent. Verifying $m$ distinct range proofs of size $n$ now only requires a single multi-exponentiation of size $2 n+2+m \cdot(2 \cdot \log (n)+5)$ along with $O(m \cdot n)$ scalar operations.

Note that this optimization can even be applied for circuits and proofs for different circuits if the same public parameter are used.

Even for a single verification we can take advantage of the fact that most generators are fixed in the public parameters. Both the verifier and the prover can used fast fixed-base exponentiation with precomputation [Gor98] to speed-up all the multi-exponentiations.

\subsection{Implementation and Performance}

To evaluate the performance of Bulletproofs in practice we give a reference implementation in C and integrate it into the popular library libsecp 256 k 1 which is used in many cryptocurrency clients. libsecp256k1 uses the elliptic curve secp256k $1^{5}$ which has 128 bit security.

In their compressed form, secp256k1 points can be stored as 32 bytes plus one bit. We use all of the optimizations described above, except the pre-computation of generators. The prover uses constant time operations until the computation of $\mathbf{l}$ and $\mathbf{r}$. By Theorem 2, the inner product argument does not need to hide $\mathbf{l}$ and $\mathbf{r}$ and can therefore use variable time operations. The verifier has no secrets and can therefore safely use variable time operations like the multi-exponentiations.

All experiments were performed on an Intel i7-6820HQ system throttled to 2.00 GHz and using a single thread. Less than 100 MB of memory was used in all experiments. For reference, verifying an ECDSA signature takes $86 \mu \mathrm{s}$ on the same system. Table 2 shows that in terms of proof size Bulletproofs bring a significant improvement over the 3.8 KB proof size in $\left[\mathrm{PBF}^{+}\right]$. A single 64 -bit range proof is 688 bytes. An aggregated proof for 32 ranges is still just 1 KB whereas 32 proofs from $\left[\mathrm{PBF}^{+}\right]$would have taken up 121 KB . The cost to verify a single 64 -bit range proof is 3.9 ms but using batch verification of many proofs the amortized cost can be brought down to $450 \mu \mathrm{s}$ or 5.2 ECDSA verifications. Verifying an aggregated proof for 64 ranges takes 61 ms or 1.9 ms per


\footnotetext{
${ }^{5}$ http://www.secg.org/SEC2-Ver-1.0.pdf
range. The marginal cost of verifying an additional proof is 2.67 ms or $83 \mu \mathrm{s}$ per range. This is less than verifying an ECDSA signature, which cannot take advantage of the same batch validation.

To aid future use of Bulletproofs we also implemented Protocol 3 for arithmetic circuits and provide a parser for circuits in the Pinocchio [PHGR13] format to the Bulletproofs format. This hooks Bulletproofs up to the Pinocchio toolchain which contains a compiler from a subset of C to the circuit format. To evaluate the implementation we analyze several circuits for hash preimages in Table 3 and Figure 3.

Specifically, a SHA256 circuit generated by jsnark ${ }^{6}$ and a Pedersen hash function over an embedded elliptic curve similar to Jubjub ${ }^{7}$ are benchmarked. A Bulletproof for knowing a 384-bit Pedersen hash preimage is about 1 KB and takes 61 ms to verify. The marginal cost of verifying an additional proof is 2.1 ms . The SHA256 preimage proof is 1.4 KB and takes 750 ms to verify. The marginal cost of verifying additional proofs is 41.5 ms . Figure 3 shows that the proving and verification time grow linearly. The batch verification first grows logarithmically and then linearly. For small circuits the logarithmic number of exponentiations dominate the cost while for larger circuits the linear scalar operations do.

Figure 1: Sizes for range proofs

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-31.jpg?height=611&width=860&top_left_y=1120&top_left_x=622)


\footnotetext{
${ }^{6}$ See https://github.com/akosba/jsnark.

${ }^{7}$ See https://z.cash/technology/jubjub.html.
}

Table 2: Range proofs: performance and proof size

\begin{tabular}{c|r|r|r|r|r} 
Problem size & Gates & $\pi$ Size & \multicolumn{3}{|c}{ Timing $(\mathrm{ms})$} \\
(bytes) & prove & verify & batch \\
\hline Range proofs & range $\times$ aggregation size) \\
8 bit & 8 & 482 & 3.7 & 0.9 & 0.28 \\
16 bit & 16 & 546 & 7.2 & 1.4 & 0.33 \\
32 bit & 32 & 610 & 15 & 2.4 & 0.38 \\
64 bit & 64 & 675 & 29 & 3.9 & 0.45 \\
64 bit $\times 2$ & 128 & 739 & 57 & 6.2 & 0.55 \\
per range & 64 & 370 & 29 & 3.1 & 0.28 \\
64 bit $\times 4$ & 256 & 803 & 111 & 10.4 & 0.71 \\
per range & 64 & 201 & 28 & 2.6 & 0.18 \\
64 bit $\times 8$ & 512 & 932 & 213 & 18.8 & 1.08 \\
per range & 64 & 117 & 27 & 2.4 & 0.13 \\
64 bit $\times 16$ & 1024 & 932 & 416 & 33.2 & 1.58 \\
per range & 64 & 59 & 26 & 2.1 & 0.10 \\
64 bit $\times 32$ & 2048 & 996 & 812 & 61.0 & 2.67 \\
per range & 64 & 32 & 25 & 1.9 & 0.083 \\
64 bit $\times 64$ & 4096 & 1060 & 1594 & 114 & 4.91 \\
per range & 64 & 17 & 25 & 1.8 & 0.077 \\
64 bit $\times 128$ & 8192 & 1124 & 3128 & 210 & 9.75 \\
per range & 64 & 8.8 & 25 & 1.6 & 0.076 \\
64 bit $\times 256$ & 16384 & 1189 & 6171 & 392 & 21.03 \\
per range & 64 & 4.6 & 24 & 1.5 & 0.082 \\
64 bit $\times 512$ & 32768 & 1253 & 12205 & 764 & 50.7 \\
per range & 64 & 2.5 & 24 & 1.5 & 0.10 \\
\hline
\end{tabular}

The first 4 instances are $n$-bit range proofs and the later ones are $m$ aggregated 64 -bit proofs and the normalized costs per range. "Batch" is the marginal cost of verifying an additional proof, computed by batch-verifying 100 proofs, subtracting the cost to verify one, and dividing by 99 .

Table 3: Protocol 3: Performance numbers and proof sizes

\begin{tabular}{l|r|r|r|r|r} 
Input size & Gates & \begin{tabular}{c}
$\pi$ Size \\
(bytes)
\end{tabular} & \multicolumn{3}{|c}{ Timing (ms) } \\
prove & verify & batch \\
\hline Pedersen hash preimage (input size) \\
48 bit & 128 & 864 & 88 & 6.4 & 0.72 \\
96 bit & 256 & 928 & 172 & 10.6 & 0.93 \\
192 bit & 512 & 992 & 335 & 19.1 & 1.33 \\
384 bit & 1024 & 1056 & 659 & 33.6 & 2.12 \\
768 bit & 2048 & 1120 & 1292 & 61.6 & 3.66 \\
1536 bit & 4096 & 1184 & 2551 & 114.9 & 6.93 \\
3072 bit & 8192 & 1248 & 5052 & 213.4 & 13.20 \\
\hline Unpadded SHAS56 preimage \\
512 bit & $\mid 25400$ & 1376 & 19478 & 749.9 & 41.52 \\
\hline
\end{tabular}

Bulletproofs for proving knowledge of $x$ s.t. $H(x)=y$ for different sized $x$ 's. The first 7 rows are for the Pedersen hash function and the final row is for SHA256. "Batch" is the marginal cost of verifying an additional proof, computed by batch-verifying 100 proofs, subtracting the cost to verify one, and dividing by 99 .

Figure 2: Timings for range proofs

![](https://cdn.mathpix.com/cropped/2024_07_25_8b0392360f8aed6595afg-33.jpg?height=619&width=1636&top_left_y=1634&top_left_x=253)

Figure 3: Timings for arithmetic circuits (Pedersen Hash)

\section{Acknowledgments}

We thank Shashank Agrawal for coming up with the Bulletproof name (short like a bullet with bulletproof security assumptions). We thank Peter Dettman for pointing out the batch inversion trick. We thank Sean Bowe and Daira Hopwood for various optimizations applicable to arithmetic circuits for Pedersen hash functions. Further, we thank Philip Hayes, Cathie Yun, and the anonymous reviewers for helpful corrections. We also thank Jim Miller for pointing out a critical vulnerability in the Fiat-Shamir description of a prior version of the paper. This work was supported by NSF, DARPA, a grant from ONR, and the Simons Foundation.

\section{References}

[AFK21] Thomas Attema, Serge Fehr, and Michael Klooß. Fiat-shamir transformation of multi-round interactive proofs. IACR Cryptol. ePrint Arch., page 1377, 2021.

[AHIV17] Scott Ames, Carmit Hazay, Yuval Ishai, and Muthuramakrishnan Venkitasubramaniam. Ligero: Lightweight sublinear arguments without a trusted setup. In Proceedings of the 2017 ACM SIGSAC Conference on Computer and Communications Security, pages 2087-2104. ACM, 2017.

$\left[\mathrm{AKR}^{+}\right.$13] Elli Androulaki, Ghassan O Karame, Marc Roeschlin, Tobias Scherer, and Srdjan Capkun. Evaluating User Privacy in Bitcoin. In Financial Cryptography, 2013.

[And17] Oleg Andreev. Hidden in Plain Sight: Transacting Privately on a Blockchain. blog. chain.com, 2017.

[BB04] Dan Boneh and Xavier Boyen. Short signatures without random oracles. In Advances in Cryptology - EUROCRYPT 2004, pages 56-73, 2004.

$\left[\mathrm{BBB}^{+} 18\right] \quad$ Benedikt Bünz, Jonathan Bootle, Dan Boneh, Andrew Poelstra, Pieter Wuille, and Greg Maxwell. Bulletproofs: Short proofs for confidential transactions and more (conference version). In Security and Privacy (SP), 2018 IEEE Symposium on, pages 319-338. IEEE, 2018.

$\left[\mathrm{BCC}^{+} 16\right]$ Jonathan Bootle, Andrea Cerulli, Pyrros Chaidos, Jens Groth, and Christophe Petit. Efficient zero-knowledge arguments for arithmetic circuits in the discrete log setting. In Annual International Conference on the Theory and Applications of Cryptographic Techniques, pages 327-357. Springer, 2016 .

[BCCT12] Nir Bitansky, Ran Canetti, Alessandro Chiesa, and Eran Tromer. From extractable collision resistance to succinct non-interactive arguments of knowledge, and back again. In Innovations in Theoretical Computer Science 2012, pages 326-349, 2012.

[BCCT13] Nir Bitansky, Ran Canetti, Alessandro Chiesa, and Eran Tromer. Recursive composition and bootstrapping for SNARKS and proof-carrying data. In Symposium on Theory of Computing Conference, STOC'13, Palo Alto, CA, USA, June 1-4, 2013, pages 111-120, 2013.
$\left[\mathrm{BCG}^{+}\right.$17a] Eli Ben-Sasson, Alessandro Chiesa, Ariel Gabizon, Michael Riabzev, and Nicholas Spooner. Interactive oracle proofs with constant rate and query complexity. In 44 th International Colloquium on Automata, Languages, and Programming, ICALP 2017, July 10-14, 2017, Warsaw, Poland, pages 40:1-40:15, 2017.

$\left[\mathrm{BCG}^{+}\right.$17b] Jonathan Bootle, Andrea Cerulli, Essam Ghadafi, Jens Groth, Mohammad Hajiabadi, and Sune K. Jakobsen. Linear-time zero-knowledge proofs for arithmetic circuit satisfiability. Cryptology ePrint Archive, Report 2017/872, 2017. http: //eprint.iacr.org/2017/872.

[BDLO12] Daniel J Bernstein, Jeroen Doumen, Tanja Lange, and Jan-Jaap Oosterwijk. Faster batch forgery identification. In International Conference on Cryptology in India, pages 454-473. Springer, 2012.

[BdM93] Josh Cohen Benaloh and Michael de Mare. One-way accumulators: A decentralized alternative to digital sinatures (extended abstract). In Advances in Cryptology EUROCRYPT '93, pages 274-285, 1993.

[BG12] Stephanie Bayer and Jens Groth. Efficient zero-knowledge argument for correctness of a shuffle. In Annual International Conference on the Theory and Applications of Cryptographic Techniques, pages 263-280. Springer, 2012.

[BGB17] Benedikt Bünz, Steven Goldfeder, and Joseph Bonneau. Proofs-of-delay and randomness beacons in ethereum. IEEE SECURITY and PRIVACY ON THE BLOCKCHAIN (IEEE SళB), 2017.

[BGG17] Sean Bowe, Ariel Gabizon, and Matthew D. Green. A multi-party protocol for constructing the public parameters of the pinocchio zk-snark. IACR Cryptology ePrint Archive, 2017:602, 2017.

[BGR98] Mihir Bellare, Juan A. Garay, and Tal Rabin. Fast batch verification for modular exponentiation and digital signatures. In Kaisa Nyberg, editor, Advances in Cryptology - EUROCRYPT'98, pages 236-250, Berlin, Heidelberg, 1998. Springer Berlin Heidelberg.

[BLS01] Dan Boneh, Ben Lynn, and Hovav Shacham. Short signatures from the weil pairing. In International Conference on the Theory and Application of Cryptology and Information Security, pages 514-532. Springer, 2001.

$\left[\mathrm{BMC}^{+}\right.$15] Joseph Bonneau, Andrew Miller, Jeremy Clark, Arvind Narayanan, Joshua A. Kroll, and Edward W. Felten. Research Perspectives and Challenges for Bitcoin and Cryptocurrencies. IEEE Symposium on Security and Privacy, 2015.

[BR93] Mihir Bellare and Phillip Rogaway. Random oracles are practical: A paradigm for designing efficient protocols. In $C C S$ '93, pages 62-73, 1993.

$\left[\mathrm{BSBC}^{+} 17\right]$ Eli Ben-Sasson, Iddo Bentov, Alessandro Chiesa, Ariel Gabizon, Daniel Genkin, Matan Hamilis, Evgenya Pergament, Michael Riabzev, Mark Silberstein, Eran

Tromer, et al. Computational integrity with a public random string from quasilinear pcps. In Annual International Conference on the Theory and Applications of Cryptographic Techniques, pages 551-579. Springer, 2017.

[BSBTHR18] Eli Ben-Sasson, Iddo Ben-Tov, Yinon Horesh, and Michael Riabzev. Scalable, transparent, and post-quantum secure computational integrity. https://eprint.iacr. org/2018/046.pdf, 2018.

$\left[\mathrm{BSCG}^{+}\right.$13] Eli Ben-Sasson, Alessandro Chiesa, Daniel Genkin, Eran Tromer, and Madars Virza. SNARKs for C: Verifying program executions succinctly and in zero knowledge. In CRYPTO, 2013.

$\left[\mathrm{BSCG}^{+}\right.$14] Eli Ben-Sasson, Alessandro Chiesa, Christina Garman, Matthew Green, Ian Miers, Eran Tromer, and Madars Virza. Zerocash: Decentralized anonymous payments from Bitcoin. In IEEE Symposium on Security and Privacy. IEEE, 2014.

[CCs08] Jan Camenisch, Rafik Chaabouni, and abhi shelat. Efficient protocols for set membership and range proofs. Advances in Cryptology-ASIACRYPT 2008, pages 234-252, 2008 .

[CD98] Ronald Cramer and Ivan Damgård. Zero-knowledge proofs for finite field arithmetic, or: Can zero-knowledge be for free? In CRYPTO 98, pages 424-441. Springer, 1998.

[CGGN17] Matteo Campanelli, Rosario Gennaro, Steven Goldfeder, and Luca Nizzardo. Zeroknowledge contingent payments revisited: Attacks and payments for services. Commun. ACM, 2017.

[Cha82] David Chaum. Blind signatures for untraceable payments. In CRYPTO, 1982.

[CHL05] Jan Camenisch, Susan Hohenberger, and Anna Lysyanskaya. Compact e-cash. In EUROCRYPT, 2005.

[CLas10] Rafik Chaabouni, Helger Lipmaa, and abhi shelat. Additive combinatorics and discrete logarithm based range protocols. In Information Security and Privacy - 15th Australasian Conference, ACISP 2010, Sydney, Australia, July 5-7, 2010. Proceedings, pages 336-351, 2010 .

[CRR11] Ran Canetti, Ben Riva, and Guy N Rothblum. Practical delegation of computation using multiple servers. In Proceedings of the 18th ACM conference on Computer and communications security, pages 445-454. ACM, 2011.

$\left[\mathrm{DBB}^{+}\right.$15] G Dagher, B Bünz, Joseph Bonneau, Jeremy Clark, and D Boneh. Provisions: Privacy-preserving proofs of solvency for bitcoin exchanges (full version). Technical report, IACR Cryptology ePrint Archive, 2015.

[FS01] Jun Furukawa and Kazue Sako. An efficient scheme for proving a shuffle. In Crypto, volume 1, pages 368-387. Springer, 2001.

[GGPR13] Rosario Gennaro, Craig Gentry, Bryan Parno, and Mariana Raykova. Quadratic span programs and succinct nizks without pcps. In Advances in Cryptology - EUROCRYPT 2013, pages 626-645, 2013.

[GH98] Oded Goldreich and Johan Håstad. On the complexity of interactive proofs with bounded communication. Inf. Process. Lett., 67(4):205-214, 1998.

[GI08a] Jens Groth and Yuval Ishai. Sub-linear zero-knowledge argument for correctness of a shuffle. Advances in Cryptology-EUROCRYPT 2008, pages 379-396, 2008.

[GI08b] Jens Groth and Yuval Ishai. Sub-linear zero-knowledge argument for correctness of a shuffle. In Advances in Cryptology - EUROCRYPT 2008, pages 379-396, 2008.

[GKR08] Shafi Goldwasser, Yael Tauman Kalai, and Guy N Rothblum. Delegating computation: interactive proofs for muggles. In Proceedings of the fortieth annual ACM symposium on Theory of computing, pages 113-122. ACM, 2008.

[Gor98] Daniel M Gordon. A survey of fast exponentiation methods. Journal of algorithms, $27(1): 129-146,1998$.

[Gro03] Jens Groth. A verifiable secret shuffle of homomorphic encryptions. In Public Key Cryptography, volume 2567, pages 145-160. Springer, 2003.

[Gro05] Jens Groth. Non-interactive zero-knowledge arguments for voting. In International Conference on Applied Cryptography and Network Security, pages 467-482. Springer, 2005.

[Gro10] Jens Groth. Short pairing-based non-interactive zero-knowledge arguments. In $A d-$ vances in Cryptology - ASIACRYPT 2010, pages 321-340, 2010.

[Gro16] Jens Groth. On the size of pairing-based non-interactive arguments. In Advances in Cryptology - EUROCRYPT 2016, pages 305-326, 2016.

[GS08] Jens Groth and Amit Sahai. Efficient non-interactive proof systems for bilinear groups. In Advances in Cryptology - EUROCRYPT 2008, pages 415-432, 2008.

[GVW02] Oded Goldreich, Salil P. Vadhan, and Avi Wigderson. On interactive proofs with a laconic prover. Computational Complexity, 11(1-2):1-53, 2002.

[Jed16] TE Jedusor. Mimblewimble, 2016.

$\left[\mathrm{KMS}^{+} 16\right]$ Ahmed Kosba, Andrew Miller, Elaine Shi, Zikai Wen, and Charalampos Papamanthou. Hawk: The blockchain model of cryptography and privacy-preserving smart contracts. In Security and Privacy (SP), 2016 IEEE Symposium on, pages 839-858. IEEE, 2016.

[KP95] Joe Kilian and Erez Petrank. An efficient non-interactive zero-knowledge proof system for NP with general assumptions. Electronic Colloquium on Computational Complexity (ECCC), 2(38), 1995.

[Lin03] Yehuda Lindell. Parallel coin-tossing and constant-round secure two-party computation. J. Cryptology, 16(3):143-184, 2003.

[Lip03] Helger Lipmaa. On diophantine complexity and statistical zero-knowledge arguments. In International Conference on the Theory and Application of Cryptology and Information Security, pages 398-415. Springer, 2003.

[Max] G Maxwell. Zero knowledge contingent payment. 2011. URl: https://en. bitcoin. it/wiki/Zero_Knowledge_Contingent_Payment (visited on 05/01/2016).

[Max13] Gregory Maxwell. CoinJoin: Bitcoin privacy for the real world. bitcointalk.org, August 2013.

[Max16] Greg Maxwell. Confidential transactions. https://people.xiph.org/ greg/ confidential_values.txt, 2016.

[Mic94] Silvio Micali. Cs proofs. In Foundations of Computer Science, 1994 Proceedings., 35th Annual Symposium on, pages 436-453. IEEE, 1994.

[Mon] Monero - Private Digital Currency . https://getmonero.org/.

[MP15] Gregory Maxwell and Andrew Poelstra. Borromean ring signatures. http://diyhpl. us/ bryan/papers2/bitcoin/Borromean\%20ring\%20signatures.pdf, 2015.

$\left[\mathrm{MPJ}^{+}\right.$13] Sarah Meiklejohn, Marjori Pomarole, Grant Jordan, Kirill Levchenko, Damon McCoy, Geoffrey M Voelker, and Stefan Savage. A fistful of bitcoins: characterizing payments among men with no names. In IMC, 2013.

[MSH17] Patrick McCorry, Siamak F Shahandashti, and Feng Hao. A smart contract for boardroom voting with maximum voter privacy. IACR Cryptology ePrint Archive, 2017:110, 2017.

[Nak08] S Nakamoto. Bitcoin: A peer-to-peer electionic cash system. Unpublished, 2008.

[Nef01] C Andrew Neff. A verifiable secret shuffle and its application to e-voting. In Proceedings of the 8th ACM conference on Computer and Communications Security, pages 116-125. ACM, 2001.

$\left[\mathrm{NM}^{+}\right.$16] Shen Noether, Adam Mackenzie, et al. Ring confidential transactions. Ledger, 1:1-18, 2016 .

$\left[\mathrm{P}^{+} 91\right]$ Torben P Pedersen et al. Non-interactive and information-theoretic secure verifiable secret sharing. In Crypto, volume 91, pages 129-140. Springer, 1991.

$\left[\mathrm{PBF}^{+}\right]$Andrew Poelstra, Adam Back, Mark Friedenbach, Gregory Maxwell, and Pieter Wuille. Confidential assets.

[PHGR13] Bryan Parno, Jon Howell, Craig Gentry, and Mariana Raykova. Pinocchio: Nearly practical verifiable computation. In Security and Privacy (SP), 2013 IEEE Symposium on, pages 238-252. IEEE, 2013.

[PHGR16] Bryan Parno, Jon Howell, Craig Gentry, and Mariana Raykova. Pinocchio: nearly practical verifiable computation. Commun. ACM, 59(2):103-112, 2016.

[Pip80] Nicholas Pippenger. On the evaluation of powers and monomials. SIAM Journal on Computing, 9:230-250, 1980.

[Poe] Andrew Poelstra. Mimblewimble.

[RM] Tim Ruffing and Giulio Malavolta. Switch commitments: A safety switch for confidential transactions.

[RMSK14] Tim Ruffing, Pedro Moreno-Sanchez, and Aniket Kate. CoinShuffle: Practical decentralized coin mixing for Bitcoin. In ESORICS, 2014.

[San99] Tomas Sander. Efficient accumulators without trapdoor extended abstract. Information and Communication Security, pages 252-262, 1999.

[TR] Jason Teutsch and Christian Reitwießner. A scalable verification solution for blockchains.

[vS13] Nicolas van Saberhagen. Cryptonote v 2. 0, 2013.

[Wik21] Douglas Wikstöm. Special soundness in the random oracle model. IACR Cryptol. ePrint Arch., page 1265, 2021.

[Woo14] Gavin Wood. Ethereum: A secure decentralized transaction ledger. http:// gavwood.com/paper.pdf, 2014.

$\left[\mathrm{WTs}^{+}\right] \quad$ Riad S Wahby, Ioanna Tzialla, abhi shelat, Justin Thaler, and Michael Walfish. Doubly-efficient zksnarks without trusted setup.

\section{A A General Forking Lemma}

We briefly describe the forking lemma of $\left[\mathrm{BCC}^{+} 16\right]$ that will be needed in the proofs.

Suppose that we have a $(2 \mu+1)$-move public-coin argument with $\mu$ challenges, $x_{1}, \ldots, x_{\mu}$ in sequence. Let $n_{i} \geqslant 1$ for $1 \leqslant i \leqslant \mu$. Consider $\prod_{i=1}^{\mu} n_{i}$ accepting transcripts with challenges in the following tree format. The tree has depth $\mu$ and $\prod_{i=1}^{\mu} n_{i}$ leaves. The root of the tree is labeled with the statement. Each node of depth $i<\mu$ has exactly $n_{i}$ children, each labeled with a distinct value of the $i$ th challenge $x_{i}$.

This can be referred to as an $\left(n_{1}, \ldots, n_{\mu}\right)$-tree of accepting transcripts. Given a suitable tree of accepting transcripts, one can compute a valid witness for our inner-product argument, range proof, and argument for arithmetic circuit satisfiability. This is a natural generalization of specialsoundness for Sigma-protocols, where $\mu=1$ and $n=2$. Combined with Theorem 6 , this shows that the protocols have witness-extended emulation, and hence, the prover cannot produce an accepting transcript unless they know a witness. For simplicity in the following lemma, we assume that the challenges are chosen uniformly from $\mathbb{Z}_{p}$ where $|p|=\lambda$, but any sufficiently large challenge space would suffice. The success probability of a cheating prover scales inversely with the size of the challenge space and linearly with the number of accepting transcripts that an extractor needs. Therefore if $\prod_{i=1}^{\mu} n_{i}$ is negligible in $2^{\lambda}$, then a cheating prover can create a proof that the verifier accepts with only negligible probability.

Theorem 6 (Forking Lemma, $\left[\mathrm{BCC}^{+} 16\right]$ ). Let (Setup, $\left.\mathcal{P}, \mathcal{V}\right)$ be a $(2 k+1)$-move, public coin interactive protocol. Let $\chi$ be a witness extraction algorithm that succeeds with probability $1-\mu(\lambda)$ for some negligible function $\mu(\lambda)$ in extracting a witness from an $\left(n_{1}, \ldots, n_{k}\right)$-tree of accepting tran-

scripts in probabilistic polynomial time. Assume that $\prod_{i=1}^{k} n_{i}$ is bounded above by a polynomial in the security parameter $\lambda$. Then (Setup, $\mathcal{P}, \mathcal{V}$ ) has witness-extended emulation.

The theorem is slightly different than the one from $\left[\mathrm{BCC}^{+} 16\right]$. We allow the extractor $\chi$ to fail with negligible probability. Whenever this happens the Emulator $\mathcal{E}$ as defined by Definition 10 also simply fails. Even with this slight modification this slightly stronger lemma still holds as $\mathcal{E}$ overall still only fails with negligible probability.

\section{B Proof of Theorem 1}

Proof. Perfect completeness follows directly because Protocol 1 converts an instance for relation (2) into an instance for relation (3). Protocol 2 is trivially complete. For witness extended emulation we show that there exists an efficient extractor $\chi$ that uses $n^{2}$ transcripts, as needed by Theorem 6 .

First we show how to construct an extractor $\chi_{1}$ for Protocol 2 which on input $(\mathbf{g}, \mathbf{h}, u, P)$, either extracts a witness $\mathbf{a}, \mathbf{b}$ such that relation (3) holds, or discovers a non-trivial discrete logarithm relation between $\mathbf{g}, \mathbf{h}, u$. Note that the hardness of computing a discrete log relation between $\mathbf{g}^{\prime}, \mathbf{h}^{\prime}, u$ implies the hardness of computing one between $\mathbf{g}, \mathbf{h}, u$ as defined in Protocol 2. We will, therefore, use an inductive argument showing that in each step we either extract a witness or a discrete log relation.

If $n=|\mathbf{g}|=1$, then the prover reveals the witness $(a, b)$ in the protocol and the relation $P=g^{a} h^{b} u^{a \cdot b}$ can simply be checked directly.

Next, we show that for each recursive step that on input $(\mathbf{g}, \mathbf{h}, u, P)$, we can efficiently extract from the prover a witness $\mathbf{a}, \mathbf{b}$ or a non-trivial discrete logarithm relation between $\mathbf{g}, \mathbf{h}, u$. The extractor runs the prover to get $L$ and $R$. Then, by rewinding the prover four times and giving it four challenges $x_{1}, x_{2}, x_{3}, x_{4}$, such that $x_{i} \neq \pm x_{j}$ for $1 \leqslant i<j \leqslant 4$, the extractor obtains four pairs $\mathbf{a}_{i}^{\prime}, \mathbf{b}_{i}^{\prime} \in \mathbb{Z}_{p}^{n^{\prime}}$ such that

$$
\begin{equation*}
L^{x_{i}^{2}} P R^{x_{i}^{-2}}=\left(\mathbf{g}_{\left[: n^{\prime}\right]}^{x_{i}^{-1}} \circ \mathbf{g}_{\left[n^{\prime}:\right]}^{x_{i}}\right)^{\mathbf{a}_{i}^{\prime}} \cdot\left(\mathbf{h}_{\left[: n^{\prime}\right]}^{x_{i}} \circ \mathbf{h}_{\left[n^{\prime}:\right]}^{x_{i}^{-1}}\right)^{\mathbf{b}_{i}^{\prime}} \cdot u^{\left\langle\mathbf{a}_{i}^{\prime}, \mathbf{b}_{i}^{\prime}\right\rangle} \quad \text { for } i=1, \ldots, 4 \tag{106}
\end{equation*}
$$

We can use the first three challenges $x_{1}, x_{2}, x_{3}$, to compute $\nu_{1}, \nu_{2}, \nu_{3} \in \mathbb{Z}_{p}$ such that

$$
\sum_{i=1}^{3} \nu_{i} \cdot x_{i}^{2}=1, \quad \sum_{i=1}^{3} \nu_{i}=0, \quad \sum_{i=1}^{3} \nu_{i} \cdot x_{i}^{-2}=0
$$

Then taking a linear combination of the first three equalities in (106), with $\nu_{1}, \nu_{2}, \nu_{3}$ as the coefficients, we can compute $\mathbf{a}_{L}, \mathbf{b}_{L} \in \mathbb{Z}_{p}^{n}$ and $c_{L} \in \mathbb{Z}_{p}$ such that $L=\mathbf{g}^{\mathbf{a}_{L}} \mathbf{h}^{\mathbf{b}_{L}} u^{c_{L}}$. Repeating this process with different combinations, we can also compute $\mathbf{a}_{P}, \mathbf{a}_{R}, \mathbf{b}_{P}, \mathbf{b}_{R} \in \mathbb{Z}_{p}^{n}$ and $c_{P}, c_{R} \in \mathbb{Z}_{p}$ such that

$$
R=\mathbf{g}^{\mathbf{a}_{R}} \mathbf{h}^{\mathbf{b}_{R}} u^{c_{R}}, \quad P=\mathbf{g}^{\mathbf{a}_{P}} \mathbf{h}^{\mathbf{b}_{P}} u^{c_{P}}
$$

Now, for each $x \in\left\{x_{1}, x_{2}, x_{3}, x_{4}\right\}$ and the corresponding $\mathbf{a}^{\prime}, \mathbf{b}^{\prime} \in \mathbb{Z}_{p}^{n^{\prime}}$ we can rewrite (106) as:

$$
\mathbf{g}^{\mathbf{a}_{L} \cdot x^{2}+\mathbf{a}_{P}+\mathbf{a}_{R} \cdot x^{-2}} \cdot \mathbf{h}^{\mathbf{b}_{L} \cdot x^{2}+\mathbf{b}_{P}+\mathbf{b}_{R} \cdot x^{-2}} \cdot u^{c_{L} \cdot x^{2}+c_{P}+c_{R} \cdot x^{-2}}=L^{x^{2}} P R^{x^{-2}}=\mathbf{g}_{\left[: n^{\prime}\right]}^{\mathbf{a}^{\prime} \cdot x^{-1}} \mathbf{g}_{\left[n^{\prime}:\right]}^{\mathbf{a}^{\prime} \cdot x} \mathbf{h}_{\left[: n^{\prime}\right]}^{\mathbf{b}^{\prime} \cdot x} \mathbf{h}_{\left.\left[n^{\prime}:\right]\right]}^{\mathbf{b}^{\prime} \cdot x^{-1}} u^{\left\langle\mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right\rangle}
$$

This implies that

$$
\begin{align*}
\mathbf{a}^{\prime} \cdot x^{-1} & =\mathbf{a}_{L,\left[: n^{\prime}\right]} \cdot x^{2}+\mathbf{a}_{P,\left[: n^{\prime}\right]}+\mathbf{a}_{R,\left[: n^{\prime}\right]} \cdot x^{-2} \\
\mathbf{a}^{\prime} \cdot x & =\mathbf{a}_{L,\left[n^{\prime}:\right]} \cdot x^{2}+\mathbf{a}_{P,\left[n^{\prime}:\right]}+\mathbf{a}_{R,\left[n^{\prime}:\right]} \cdot x^{-2} \\
\mathbf{b}^{\prime} \cdot x & =\mathbf{b}_{L,\left[: n^{\prime}\right]} \cdot x^{2}+\mathbf{b}_{P,\left[: n^{\prime}\right]}+\mathbf{b}_{R,\left[: n^{\prime}\right]} \cdot x^{-2}  \tag{107}\\
\mathbf{b}^{\prime} \cdot x^{-1} & =\mathbf{b}_{L,\left[n^{\prime}:\right]} \cdot x^{2}+\mathbf{b}_{P,\left[n^{\prime}:\right]}+\mathbf{b}_{R,\left[n^{\prime}:\right]} \cdot x^{-2} \\
\left\langle\mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right\rangle & =c_{L} \cdot x^{2}+c_{P}+c_{R} \cdot x^{-2}
\end{align*}
$$

If any of these equalities do not hold, we directly obtain a non-trivial discrete logarithm relation between the generators $\left(g_{1}, \ldots, g_{n}, h_{1}, \ldots, h_{n}, u\right)$.

If the equalities hold, we can deduce that for each challenge $x \in\left\{x_{1}, x_{2}, x_{3}, x_{4}\right\}$

$$
\begin{array}{r}
\mathbf{a}_{L,\left[: n^{\prime}\right]} \cdot x^{3}+\left(\mathbf{a}_{P,\left[: n^{\prime}\right]}-\mathbf{a}_{L,\left[n^{\prime}:\right]}\right) \cdot x+\left(\mathbf{a}_{R,\left[: n^{\prime}\right]}-\mathbf{a}_{P,\left[n^{\prime}:\right]}\right) \cdot x^{-1}-\mathbf{a}_{R,\left[n^{\prime}:\right]} \cdot x^{-3}=0 \\
\mathbf{b}_{L,\left[n^{\prime}:\right]} \cdot x^{3}+\left(\mathbf{b}_{P,\left[n^{\prime}:\right]}-\mathbf{b}_{L,\left[: n^{\prime}\right]}\right) \cdot x+\left(\mathbf{b}_{R,\left[n^{\prime}:\right]}-\mathbf{b}_{P,\left[: n^{\prime}\right]}\right) \cdot x^{-1}-\mathbf{b}_{R,\left[: n^{\prime}\right]} \cdot x^{-3}=0 \tag{109}
\end{array}
$$

The equality (108) follows from the first two equations in (107). Similarly, (109) follows from the third and fourth equations in (107).

The only way (108) and (109) hold for all 4 challenges $x_{1}, x_{2}, x_{3}, x_{4} \in \mathbb{Z}_{p}$ is if

$$
\begin{align*}
& \mathbf{a}_{L,\left[: n^{\prime}\right]}=\mathbf{a}_{R,\left[n^{\prime}:\right]}=\mathbf{b}_{R,\left[: n^{\prime}\right]}=\mathbf{b}_{L,\left[n^{\prime}:\right]}=0 \\
& \mathbf{a}_{L,\left[n^{\prime}:\right]}=\mathbf{a}_{P,\left[:: n^{\prime}\right]}, \quad \mathbf{a}_{R,\left[: n^{\prime}\right]}=\mathbf{a}_{P,\left[n^{\prime}:\right]}  \tag{110}\\
& \mathbf{b}_{L,\left[: n^{\prime}\right]}=\mathbf{b}_{P,\left[n^{\prime}:\right]}, \quad \mathbf{b}_{R,\left[n^{\prime}:\right]}=\mathbf{b}_{P,\left[: n^{\prime}\right]}
\end{align*}
$$

Plugging these relations into (107) we obtain that for every $x \in\left\{x_{1}, x_{2}, x_{3}, x_{4}\right\}$ we have that

$$
\mathbf{a}^{\prime}=\mathbf{a}_{P,\left[: n^{\prime}\right]} \cdot x+\mathbf{a}_{P,\left[n^{\prime}:\right]} \cdot x^{-1} \quad \text { and } \quad \mathbf{b}^{\prime}=\mathbf{b}_{P,\left[: n^{\prime}\right]} \cdot x^{-1}+\mathbf{b}_{P,\left[n^{\prime}:\right]} \cdot x
$$

Now, using these values we can see that the extracted $c_{L}, c_{P}$ and $c_{R}$ have the expected form:

$$
\begin{aligned}
c_{L} \cdot x^{2} & +c_{P}+c_{R} \cdot x^{-2}=\left\langle\mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right\rangle \\
& =\left\langle\mathbf{a}_{P,\left[: n^{\prime}\right]} \cdot x+\mathbf{a}_{P,\left[n^{\prime}:\right]} \cdot x^{-1}, \mathbf{b}_{P,\left[n^{\prime}\right]} \cdot x^{-1}+\mathbf{b}_{P,\left[n^{\prime}:\right]} \cdot x\right\rangle \\
& =\left\langle\mathbf{a}_{P,\left[: n^{\prime}\right]}, \mathbf{b}_{P,\left[n^{\prime}:\right]}\right\rangle \cdot x^{2}+\left\langle\mathbf{a}_{P,\left[\left[n^{\prime}\right]\right.}, \mathbf{b}_{P,\left[: n^{\prime}\right]}\right\rangle+\left\langle\mathbf{a}_{P,\left[n^{\prime}:\right]}, \mathbf{b}_{P,\left[n^{\prime}:\right]}\right\rangle+\left\langle\mathbf{a}_{P,\left[n^{\prime}:\right]}, \mathbf{b}_{P,\left[\left[n^{\prime}\right]\right.}\right\rangle \cdot x^{-2} \\
& =\left\langle\mathbf{a}_{P,\left[: n^{\prime}\right]}, \mathbf{b}_{P,\left[n^{\prime}:\right]}\right\rangle \cdot x^{2}+\left\langle\mathbf{a}_{P}, \mathbf{b}_{P}\right\rangle+\left\langle\mathbf{a}_{P,\left[n^{\prime}:\right]}, \mathbf{b}_{P,\left[: n^{\prime}\right]}\right\rangle \cdot x^{-2}
\end{aligned}
$$

Since this relation holds for all $x \in\left\{x_{1}, x_{2}, x_{3}, x_{4}\right\}$ it must be that

$$
\left\langle\mathbf{a}_{P}, \mathbf{b}_{P}\right\rangle=c_{P}
$$

The extractor, thus, either extracts a discrete logarithm relation between the generators, or the witness $\left(\mathbf{a}_{P}, \mathbf{b}_{P}\right)$ for the relation (3).

Using Theorem 6 we can see that the extractor uses $4^{\log _{2}(n)}=n^{2}$ transcripts in total and thus runs in expected polynomial time in $n$ and $\lambda$.

We now show that using Protocol 1 we can construct an extractor $\chi$ that extracts a valid witness for relation (3). The extractor uses the extractor $\chi_{1}$ of Protocol 2. On input ( $\mathbf{g}, \mathbf{h}, u, P, c$ ) $\chi$ runs the prover with on a challenge $x$ and uses the extractor $\chi_{1}$ to obtain a witness $\mathbf{a}, \mathbf{b}$ such that:
$P \cdot u^{x \cdot c}=\mathbf{g}^{\mathbf{a}} \mathbf{h}^{\mathbf{b}} u^{x \cdot\langle\mathbf{a}, \mathbf{b}\rangle}$. Rewinding $\mathcal{P}$, supplying him with a different challenge $x^{\prime}$ and rerunning the extractor $\chi_{1}$ yields a second witness $\left(\mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right)$. Again the soundness of Protocol 2 implies that $P \cdot u^{x^{\prime} \cdot c}=\mathbf{g}^{\mathbf{a}^{\prime}} \mathbf{h}^{\mathbf{b}^{\prime}} u^{x^{\prime} \cdot\left\langle\mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right\rangle}$. From the two witnesses, we can compute:

$$
u^{\left(x-x^{\prime}\right) \cdot c}=\mathbf{g}^{\mathbf{a}-\mathbf{a}^{\prime}} \mathbf{h}^{\mathbf{b}-\mathbf{b}^{\prime}} u^{x \cdot\langle\mathbf{a}, \mathbf{b}\rangle-x^{\prime} \cdot\left\langle\mathbf{a}^{\prime}, \mathbf{b}^{\prime}\right\rangle}
$$

Unless $\mathbf{a}=\mathbf{a}^{\prime}$ and $\mathbf{b}=\mathbf{b}^{\prime}$ we get a not trivial discrete $\log$ relation between $\mathbf{g}, \mathbf{h}$ and $u$. Otherwise we get $u^{\left(x-x^{\prime}\right) \cdot c}=u^{\left(x-x^{\prime}\right) \cdot\langle\mathbf{a}, \mathbf{b}\rangle} \Longrightarrow c=\langle\mathbf{a}, \mathbf{b}\rangle$. Thus, $(\mathbf{a}, \mathbf{b})$ is a valid witness for relation (3). Since $\chi$ forks the prover once, and uses the efficient extractor $\chi_{1}$ twice, it is also efficient. Using the forking lemma (Theorem 6) we conclude that the protocol has witness extended emulation.

\section{Proof of Theorem 3}

Proof. Perfect completeness follows from the fact that $t_{0}=\delta(y, z)+z^{2} \cdot\left\langle\mathbf{z}^{m}, \mathbf{v}\right\rangle$ for all valid witnesses. To prove perfect honest-verifier zero-knowledge we construct a simulator that produces a distribution of proofs for a given statement $\left(g, h \in \mathbb{G}, \mathbf{g}, \mathbf{h} \in \mathbb{G}^{n \cdot m}, \mathbf{V} \in \mathbb{G}^{m}\right.$ ) that is indistinguishable from valid proofs produced by an honest prover interacting with an honest verifier. The simulator chooses all proof elements and challenges according to the randomness supplied by the adversary from their respective domains or computes them directly as described in the protocol. $S$ and $T_{1}$ are computed according to the verification equations, i.e.:

$$
\begin{aligned}
S & =\left(h^{-\mu} \cdot A \cdot \mathbf{g}^{-z \cdot \mathbf{1}^{n \cdot m}-1} \cdot \mathbf{h}^{\prime z \cdot \mathbf{y}^{n \cdot m}-\mathbf{r}} \prod_{j=1}^{m} \mathbf{h}_{[(j-1) \cdot m: j \cdot m]}^{z^{j+1} \cdot \mathbf{2}^{n}}\right)^{-x^{-1}} \\
T_{1} & =\left(h^{-\tau_{x}} g^{\delta(y, z)-\hat{t}} \cdot \mathbf{V}^{z^{2} \cdot \mathbf{z}^{m}} \cdot T_{2}^{x^{2}}\right)^{-x^{-1}}
\end{aligned}
$$

Finally, the simulator runs the inner-product argument with the simulated witness $(\mathbf{l}, \mathbf{r})$ and the verifier's randomness. All elements in the proof are either independently randomly distributed or their relationship is fully defined by the verification equations. The inner product argument remains zero knowledge as we can successfully simulate the witness, thus revealing the witness or leaking information about it does not change the zero-knowledge property of the overall protocol. The simulator runs in time $O\left(\mathcal{V}+\mathcal{P}_{\text {InnerProduct }}\right)$ and is thus efficient.

In order to prove computational witness extended emulation, we construct an extractor $\chi$ as follows. The extractor $\chi$ runs the prover with $n \cdot m$ different values of $y,(m+2)$ different values of $z$, and 3 different values of the challenge $x$. Additionally it invokes the extractor for the inner product argument on each of the transcripts. This results in $3 \cdot(m+2) \cdot n \cdot m \cdot O\left(n^{2}\right)$ valid proof transcripts.

For each transcript the extractor $\chi$ first runs the extractor $\chi_{\text {InnerProduct }}$ for the inner-product argument to extract a witness $\mathbf{l}, \mathbf{r}$ to the inner product argument such that $h^{\mu} \mathbf{g}^{\mathbf{l}} \mathbf{h}^{\mathbf{r}}=P \wedge\langle\mathbf{l}, \mathbf{r}\rangle=\hat{t}$. Using 2 valid transcripts and extracted inner product argument witnesses for different $x$ challenges, we can compute linear combinations of (67) such that in order to compute $\alpha, \rho, \mathbf{a}_{L}, \mathbf{a}_{R}, \mathbf{s}_{L}, \mathbf{s}_{R}$ such that $A=h^{\alpha} \mathbf{g}^{\mathbf{a}_{L}} \mathbf{h}^{\mathbf{a}_{R}}$, as well as $S=h^{\rho} \mathbf{g}^{\mathbf{s}_{L}} \mathbf{h}^{\mathbf{s}_{R}}$.

If for any other set of challenges $(x, y, z)$ the extractor can compute a different representation of $A$ or $S$, then this yields a non-trivial discrete logarithm relation between independent generators $h, \mathbf{g}, \mathbf{h}$ which contradicts the discrete logarithm assumption.

Using these representations of $A$ and $S$, as well as $\mathbf{l}$ and $\mathbf{r}$, we then find that for all challenges $x, y$ and $z$

$$
\begin{aligned}
& \mathbf{l}=\mathbf{a}_{L}-z \cdot \mathbf{1}^{n \cdot m}+\mathbf{s}_{L} \cdot x \\
& \mathbf{r}=\mathbf{y}^{n \cdot m} \circ\left(\mathbf{a}_{R}+z \cdot \mathbf{1}^{n \cdot m}+\mathbf{s}_{R} \cdot x\right)+\sum_{j=1}^{m} z^{1+j} \cdot\left(0^{(j-1) \cdot n}\left\|\mathbf{2}^{n}\right\| 0^{(m-j) \cdot n}\right)
\end{aligned}
$$

If these equalities do not hold for all challenges and $\mathbf{l}, \mathbf{r}$ from the transcript, then we have two distinct representations of the same group element using a set of independent generators. This would be a non-trivial discrete logarithm relation.

For given values of $y$ and $z$, we now takes 3 transcripts with different $x$ 's and uses linear combinations of equation (72) to compute $\tau_{1}, \tau_{2}, t_{1}, t_{2}$ such that

$$
T_{1}=g^{t_{1}} h^{\tau_{1}} \wedge T_{2}=g^{t_{1}} h^{\tau_{2}}
$$

Additionally we can compute a $v, \gamma$ such that $g^{v} h^{\gamma}=\prod_{j=1}^{m} V_{j}^{z^{j+1}}$ Repeating this for $m$ different $z$ challenges, we can compute $\left(v_{j}, \gamma_{j}\right)_{j=1}^{m}$ such that $g^{v_{j}} h^{\gamma_{j}}=V_{j} \forall j \in[1, m]$. If for any transcript $\delta(y, z)+\sum_{j=1}^{m} z^{j+2} \cdot v_{j}+t_{1} \cdot x+t_{2} \cdot x^{2} \neq \hat{t}$ then this directly yields a discrete log relation between $g$ and $h$, i.e. a violation of the binding property of the Pedersen commitment. If not, then for all $y, z$ challenges and 3 distinct challenges $X=x_{j}, j \in[1,3]$ :

$$
\sum_{i=0}^{2} t_{i} \cdot X^{i}-p(X)=0
$$

with $t_{0}=\delta(y, z)+\sum_{j=1}^{m} z^{j+2} \cdot\left\langle\mathbf{v}_{j}, \mathbf{2}^{n}\right\rangle$ and $p(X)=\sum_{i=0}^{2} p_{i} \cdot X^{i}=\langle l(X), r(X)\rangle$. Since the polynomial $t(X)-p(X)$ is of degree 2 , but has at least 3 roots (each challenge $x_{j}$ ), it is necessarily the zero polynomial, i.e. $t(X)=\langle l(X), r(X)\rangle$.

Since this implies that $t_{0}=p_{0}$, the following holds for all $y, z$ challenges:

$$
\begin{gathered}
\sum_{j=1}^{m} z^{j+2} \cdot\left\langle\mathbf{v}_{j}, \mathbf{2}^{n}\right\rangle+\delta(y, z) \\
= \\
\left\langle\mathbf{a}_{L}, \mathbf{y}^{n \cdot m} \circ \mathbf{a}_{R}\right\rangle+z \cdot\left\langle\mathbf{a}_{L}-\mathbf{a}_{R}, \mathbf{y}^{n \cdot m}\right\rangle+\sum_{j=1}^{m} z^{j+1}\left\langle\mathbf{a}_{L,[(j-1) \cdot n: j \cdot n]}, \mathbf{2}^{n}\right\rangle \\
-z^{2} \cdot\left\langle\mathbf{1}^{n \cdot m}, \mathbf{y}^{n \cdot m}\right\rangle-\sum_{j=1}^{m} z^{j+2} \cdot\left\langle\mathbf{1}^{n}, \mathbf{2}^{n}\right\rangle \in \mathbb{Z}_{p}
\end{gathered}
$$

If this equality holds for $n \cdot m$ distinct $y$ challenges and $m+2$ distinct $z$ challenges, then we can infer the following.

$$
\begin{array}{rlr}
\mathbf{a}_{L} \circ \mathbf{a}_{R} & =\mathbf{0}^{n \cdot m} & \in \mathbb{Z}_{p}^{n \cdot m} \\
\mathbf{a}_{R} & =\mathbf{a}_{L}-\mathbf{1}^{n \cdot m} & \in \mathbb{Z}_{p}^{n \cdot m} \\
v_{j} & =\left\langle\mathbf{a}_{L,[(j-1) \cdot n: j \cdot n]}, \mathbf{2}^{n}\right\rangle & \in \mathbb{Z}_{p} \forall j \in[1, m]
\end{array}
$$

The first two equations imply that $\mathbf{a}_{L} \in\{0,1\}^{n \cdot m}$. The last equation imply that $v_{j} \in\left[0,2^{n-1}\right]$ for all $j \in[1, m]$. Since $g^{v_{i}} h^{\gamma_{i}}=V_{i} \quad \forall j \in[1, m]$ we have that $(\mathbf{v}, \gamma)$ is valid witness for relation (69). The extractor rewinds the prover $3 \cdot(m+2) \cdot n \cdot m \cdot O\left(n^{2}\right)$ times. Extraction is efficient and the number of transcripts is polynomial in $\lambda$ because $n, m=O(\lambda)$. Note that extraction either returns a valid witness or a discrete logarithm relation between independently chosen generators. We define $\chi^{\prime}$ being equal to $\chi$ but failing whenever $\chi$ extracts a discrete log relation. By the Discrete Log Relation assumption this happens with at most negligible probability. We can, therefore, apply the forking lemma and see that computational witness emulation holds.

\section{Droof of Theorem 4}

Proof. Perfect completeness follows from the fact that

$$
\begin{equation*}
t_{2}=\delta(y, z)+\left\langle\mathbf{z}_{[1:]}^{Q+1}, \mathbf{W}_{L} \cdot \mathbf{a}_{L}+\mathbf{W}_{R} \cdot \mathbf{a}_{R}+\mathbf{W}_{O} \cdot \mathbf{a}_{O}\right\rangle=\delta(y, z)+\left\langle\mathbf{z}_{[1:]}^{Q+1}, \mathbf{W}_{V} \cdot \mathbf{v}+\mathbf{c}\right\rangle \tag{111}
\end{equation*}
$$

whenever the prover knows a witness to the relation and is honest.

To prove perfect honest-verifier zero-knowledge we construct an efficient simulator that produces a distribution of proofs for a given statement

$$
\left(g, h \in \mathbb{G}, \mathbf{g}, \mathbf{h} \in \mathbb{G}^{n}, \mathbf{V} \in \mathbb{G}^{m},\left(\mathbf{w}_{L, q}, \mathbf{w}_{R, q}, \mathbf{w}_{O, q}\right)_{q=1}^{Q} \in \mathbb{Z}_{p}^{n \times 3},\left(\mathbf{w}_{V, q}\right)_{q=1}^{Q} \in \mathbb{Z}_{p}^{m}, \mathbf{c} \in \mathbb{Z}_{p}^{Q}\right)
$$

and the verifier's randomness that is indistinguishable from valid proofs produced by an honest prover interacting with an honest verifier. The simulator acts as follows:

$$
\begin{align*}
& \text { Compute } x, y, z \text { using } \mathcal{V}^{\prime} \text { s randomness }  \tag{112}\\
& \mu, \tau_{x} \stackrel{\&}{\leftarrow} \mathbb{Z}_{p}  \tag{113}\\
& \mathbf{1}, \mathbf{r} \stackrel{\&}{\leftarrow} \mathbb{Z}_{p}^{n}  \tag{114}\\
& \hat{t}=\langle\mathbf{l}, \mathbf{r}\rangle  \tag{115}\\
& A_{I}, A_{O} \stackrel{\&}{\leftarrow} \mathbb{G}  \tag{116}\\
& S=\left(A_{I}^{x} \cdot A_{O}^{x^{2}} \cdot \mathbf{g}^{-1} \mathbf{h}^{\prime-\mathbf{y}^{n}-\mathbf{r}} \cdot W_{L}^{x} \cdot W_{R}^{x} \cdot W_{O} \cdot h^{-\mu}\right)^{-x^{-3}}  \tag{117}\\
& T_{3}, T_{4}, T_{5}, T_{6} \stackrel{\leftrightarrow}{\leftarrow} \mathbb{G}  \tag{118}\\
& T_{1}=\left(h^{-\tau_{x}} g^{x^{2} \cdot\left(\delta(y, z)+\left\langle\mathbf{z}_{[1: 1}^{Q+1}, \mathbf{c}\right\rangle\right)-\hat{t}} \cdot \mathbf{V}^{x^{2} \cdot\left(\mathbf{z}_{[1:]}^{Q+1} \cdot \mathbf{w}_{V}\right)} \cdot \prod_{i=3}^{6} T_{i}^{x^{i}}\right)^{-x^{-1}}  \tag{119}\\
& \text { Output: }\left(A_{I}, A_{O}, S ; y, z ; T_{1},\left(T_{i}\right)_{3}^{6} ; x ; \tau_{x}, \mu, \hat{t}, \mathbf{1}, \mathbf{r}\right) \tag{120}
\end{align*}
$$

The values $A_{I}, A_{O}, \mathbf{l}, \mathbf{r}, \mu, \tau_{x}$ produced by an honest prover interacting with an honest verifier are random independent elements, i.e. if $\mathbf{s}, \rho, \alpha, \tau_{1},\left(\tau_{i}\right)_{3}^{6}, \rho$ as well as $x, y, z$ are chosen independently and randomly. $\hat{t}$ is the inner product of $\mathbf{l}, \mathbf{r}$ as in any verifying transcript. The simulated $S$ is fully defined by equations (92). The honestly produced $T$ are perfectly hiding commitments and as such random group elements. Their internal relation given $\hat{t}$ and $\tau_{x}$ is fully defined by equation (90), which is ensured by computing $T_{1}$ accordingly. Therefore, the transcript of the proof is identically distributed to an honestly computed proof with uniformly selected challenges. The simulator runs in time $O(\mathcal{V})$ and is thus efficient.

In order to prove computational witness extended emulation we construct an extractor $\chi$ as follows. The $\chi$ runs the prover with $n$ different $y,(Q+1)$ different $z$ and 7 different $x$ challenges. This results in $7 \cdot(Q+1) \cdot n$ valid proof transcripts. We takes 3 valid transcripts for $x \in\left\{x_{1}, x_{2}, x_{3}\right\}$ and fixed $y$ and $z$. From the transmitted $\mathbf{l}, \mathbf{r}, \hat{t}$ for each combination of challenges, we compute $\nu_{1}, \nu_{2}, \nu_{3}$ such that

$$
\sum_{i=1}^{3} \nu_{i} \cdot x_{i}=1 \wedge \sum_{i=1}^{3} \nu_{i} \cdot x^{2}=\sum_{i=1}^{3} \nu_{i} \cdot x_{i}^{3}=0
$$

Taking the linear combinations of equation (92) with $\left(\nu_{1}, \nu_{2}, \nu_{3}\right)$ as coefficients, we compute $\alpha \in$ $\mathbb{Z}_{p}, \mathbf{a}_{L}, \mathbf{a}_{R} \in \mathbb{Z}_{p}^{n}$ such that $h^{\alpha} \mathbf{g}^{\mathbf{a}_{L}} \mathbf{h}^{\mathbf{a}_{R}}=A_{I}$. If for any other set of challenges we can compute
different $\alpha^{\prime}, \mathbf{a}_{L}^{\prime}, \mathbf{a}_{R}^{\prime}$ such that $h^{\alpha^{\prime}} \mathbf{g}^{\mathbf{a}_{L}^{\prime}} \mathbf{h}^{\mathbf{a}_{R}^{\prime}}=A_{I}=h^{\alpha} \mathbf{g}^{\mathbf{a}_{L}} \mathbf{h}^{\mathbf{a}_{R}}$, then this yields a non-trivial discrete log relation between independent generators $h, \mathbf{g}, \mathbf{h}$ which contradicts the discrete log relation assumption. Similarly, we can use the same challenges and equation (92) to compute unique $\beta, \rho \in \mathbb{Z}_{p}, \mathbf{a}_{O, L}, \mathbf{a}_{O, R}, \mathbf{s}_{L}, \mathbf{s}_{R} \in \mathbb{Z}_{p}^{n}$ such that $h^{\beta} \mathbf{g}^{\mathbf{a}} \mathrm{o}_{0, L} \mathbf{h}^{\mathbf{a} O, R}=A_{O}$ and $h^{\rho} \mathbf{g}^{\mathbf{s}_{L}} \mathbf{h}^{\mathbf{s}_{R}}=S$.

Using Equation (92), we can replace $A_{I}, A_{O}, S$ with the computed representations and read $\mathbf{l}, \mathbf{r}, \hat{t}$ from the transcripts. We then find that for all challenges $x, y, z$ :

$$
\begin{aligned}
& \mathbf{l}=\mathbf{a}_{L} \cdot x+\mathbf{a}_{O, L} \cdot x^{2}+\mathbf{y}^{-n} \circ\left(\mathbf{z}_{[1:]}^{Q+1} \cdot \mathbf{W}_{R}\right) \cdot X+\mathbf{s}_{L} \cdot x^{3} \\
& \mathbf{r}=\mathbf{y}^{n} \circ \mathbf{a}_{R} \cdot x-\mathbf{y}^{n}+\mathbf{z}_{[1:]}^{Q+1} \cdot\left(\mathbf{W}_{L} \cdot x+\mathbf{W}_{O}\right)+\mathbf{y}^{n} \circ \mathbf{a}_{O, R} \cdot x^{2}+\mathbf{y}^{n} \circ \mathbf{s}_{R} \cdot x^{3} \\
& \hat{t}=\langle\mathbf{l}, \mathbf{r}\rangle
\end{aligned}
$$

If these equalities do not hold for all challenges and $\mathbf{l}, \mathbf{r}$ from the transcript, then we necessarily have a non-trivial discrete $\log$ relation between the generators $\mathbf{g}, \mathbf{h}$ and $h$.

We now show that $t_{2}$ indeed has the form described in (111). For a given $y, z$ the extractor takes 6 transcripts with different $x$ 's and uses linear combinations of equation (90) to compute $\left(\tau_{i}, t_{i}\right), i \in$ $[1,3, \ldots, 6]$ such that $T_{i}=g^{t_{i}} h^{\tau_{i}}$. Note that the linear combinations have to cancel out the other $T_{i}^{x^{i}}$ terms as well as $\left(\mathbf{v}_{[1:]}^{\mathbf{z}_{[1: 1}^{Q+1}} \cdot \mathbf{W}_{V}\right)^{x^{2}}$. Using these $\left(\tau_{i}, t_{i}\right)$ we can compute $v, \gamma$ such that $g^{v} h^{\gamma}=\mathbf{V}^{\mathbf{Z}_{[1:]}^{Q+1}} \cdot \mathbf{W}_{V}$. Repeating this for $m$ different $z$ challenges, we can compute $\left(v_{j}, \gamma_{j}\right)_{j=1}^{m}$ using linear combinations of $g^{v} h^{\gamma}=\mathbf{V}^{\mathbf{Z}_{[1: 1]}^{Q+1}} \cdot \mathbf{W}_{V}$ such that $g^{v_{j}} h^{\gamma_{j}}=V_{j} \forall j \in[1, m]$. This will however only succeed if the weight vectors $\mathbf{w}_{V, j}$ are linearly independent, i.e if the matrix $\mathbf{W}_{V}$ has rank $m$. This necessarily implies that $Q \geqslant m$. If for any transcript $t_{1} \cdot x+\sum_{i=3}^{6} t_{i} \cdot x^{i}+x^{2} \cdot\left(\left\langle\mathbf{z}_{[1:]}^{Q+1}, \mathbf{W}_{V} \cdot \mathbf{v}+\mathbf{c}\right\rangle+\delta(y, z)\right) \neq \hat{t}$ then this directly yields a a discrete log relation between $g$ and $h$.

If not, then for all $y, z$ challenges and 7 distinct challenges $x=x_{j}, j \in[1,7]$ :

$$
\begin{equation*}
\sum_{i=1}^{6} t_{i} \cdot x-p(x)=0 \tag{121}
\end{equation*}
$$

with $t_{2}=\left\langle\mathbf{z}_{[1:]}^{Q+1}, \mathbf{W}_{V} \cdot \mathbf{v}+\mathbf{c}\right\rangle+\delta(y, z)$ and $p(x)=\sum_{i=1}^{6} p_{i} \cdot x^{i}=\langle\mathbf{l}(x), \mathbf{r}(x)\rangle$. Since the polynomial $t(x)-p(x)$ is of degree 6 , but has at least 7 roots (each challenge $x_{j}$ ), it is necessarily the zero polynomial, i.e. $t(x)=\langle\mathbf{l}(x), \mathbf{r}(x)\rangle$. Finally, we show that this equality implies that we can extract a witness $\left(\mathbf{a}_{L}, \mathbf{a}_{R}, \mathbf{a}_{O} \in \mathbb{Z}_{p}^{n}, \mathbf{v}, \boldsymbol{\gamma} \in \mathbb{Z}_{p}^{m}\right)$ which satisfies the relation.

The quadratic coefficient of $p$ is:

$$
p_{2}=\left\langle\mathbf{a}_{L}, \mathbf{y}^{n} \circ \mathbf{a}_{R}\right\rangle-\left\langle\mathbf{a}_{O, L}, \mathbf{y}^{n}\right\rangle+\left\langle\mathbf{z}_{[1:]}^{Q+1}, \mathbf{W}_{L} \cdot \mathbf{a}_{L}+\mathbf{W}_{R, q} \cdot \mathbf{a}_{R}+\mathbf{W}_{O} \cdot \mathbf{a}_{O, L}\right\rangle+\delta(y, z) \in \mathbb{Z}_{p}
$$

The polynomial equality implies that any challenge $y, z, p_{2}=t_{2}$. Using a fixed $y$ and $(Q+1)$ different $z$ challenges we can infer that all coefficients of $p_{2}(z)-t_{2}(z)$ have to be zero. Using $n$ different $y$ challenges, i.e. $n \cdot(Q+1)$ total transcripts we can infer the following equalities:

$$
\begin{array}{r}
\mathbf{a}_{L} \circ \mathbf{a}_{R}-\mathbf{a}_{O, L}=\mathbf{0}^{n} \in \mathbb{Z}_{p}^{n} \\
\mathbf{W}_{L} \cdot \mathbf{a}_{L}+\mathbf{W}_{R} \cdot \mathbf{a}_{R}+\mathbf{W}_{O} \cdot \mathbf{a}_{O, L}=\mathbf{W}_{V} \cdot \mathbf{v}+\mathbf{c} \in \mathbb{Z}_{p}^{Q} \tag{123}
\end{array}
$$

From equation (122) we can directly infer that $\mathbf{a}_{L} \circ \mathbf{a}_{R}=\mathbf{a}_{O, L}$. Equations (123) are exactly the linear constraints on the circuit gates.

Defining $\mathbf{a}_{O}=\mathbf{a}_{O, L}$, we can conclude that $\left(\mathbf{a}_{L}, \mathbf{a}_{R}, \mathbf{a}_{O}, \mathbf{v}, \boldsymbol{\gamma}\right)$ is indeed a valid witness. Extraction is efficient and the number of transcripts is polynomial in $\lambda$ because $n, m=O(\lambda)$. Note
that extraction either returns a valid witness or a non-trivial discrete logarithm relation between independently chosen generators. We define $\chi^{\prime}$ being equal to $\chi$ but failing whenever $\chi$ extracts a discrete log relation. By the discrete log relation assumption this happens with at most negligible probability. We can, therefore, apply the forking lemma and see that computational witness emulation holds.