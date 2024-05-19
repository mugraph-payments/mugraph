# µgraph: Fast, Untraceable Payments in Cardano

## Table of Contents

1. Abstract
1. Introduction
1. µ
1. License

## Abstract

In this document, we describe **µgraph (mugraph)**, a novel open-source protocol for instant, private payments, built on top of the Cardano Blockchain. It massively increases transaction speed and throughput, with a focus of making real-world payments fast, reliable and accessible, instead of 

## Introduction

µgraph is our interpretation on how cryptocurrencies could be used to enable real-world payments between people. We think blockchains can be great agents for change, to bring back economic power to the people, but it seems that all the things we do are for ourselves, not for the average Joe.

You can see it easily in the wild, most "real-world" crypto businesses still have to do most or all of their payments in Fiat, and the most proeminent commerce use-cases are usually things related to privacy, like VPNs. For many years now, [Travala](https://travala.com) is probably still the only travel provider selling plane tickets that you can pay using crypto.

ZeroHedge explains this phenomena perfectly, in their article ["What Happened to Bitcoin?"](https://www.zerohedge.com/crypto/what-happened-bitcoin):

> At the same time, new technologies were becoming available that vastly improved the efficiency and availability of exchange in fiat dollars. They included Venmo, Zelle, CashApp, FB payments, and many others besides, in addition to smartphone attachments and iPads that enabled any merchant of any size to process credit cards. These technologies were completely different from Bitcoin because they were permission-based and mediated by financial companies. But to users, they seemed great and their presence in the marketplace crowded out the use case of Bitcoin at the very time that my beloved technology had become an unrecognizable version of itself.

In our point of view, there are five main problems we need to tackle if we want to make crypto widely available for anyone:

1. **Volatility:** Because of their nature as assets (as well as the lack of government price controls), crypto assets are much more volatile than most currencies.
1. **Scalability:** No blockchain today is scalable enough for global payments.
1. **Privacy:** Having to make your own financial identity public just to send a payment is a price that many won't pay.
1. **Ease of Use:** You shouldn't need to read the Bitcoin Paper and watch all of Charles Hoskinson videos just to send and receive payments.

We think that, while there has been at least some very solid attempts at covering volatility, like Stablecoins or Synthetic Assets, the other ones are still ripe for the taking, and we think we can tackle them all by going simpler instead of more complicated, and by making sacrifices only where it makes sense.

## Technical Overview

Distributed systems have for a long time used something called the [CAP Theorem](https://en.wikipedia.org/wiki/CAP_theorem) to describe the guarantees associated with the system. The concept has since received it fair share of expansions and critiques, in special, from very respected authors like Martin Kleppman, in his paper [A critique of the CAP Theorem](https://www.cl.cam.ac.uk/research/dtg/archived/files/publications/public/mk428/cap-critique.pdf).

In it, Kleppman talks a lot about the [Consistency Models](https://en.wikipedia.org/wiki/Consistency_model), which can be thought of a "contract" between the user/developer and a system, stating the certainty of predictability of the [consistency](https://en.wikipedia.org/wiki/Data_consistency) of reads, writes and updates.

> [!IMPORTANT]
> We are going to talk about Byzantine Fault Tolerance (BFT) later in the document.

Those models have been first described on the ["Session Guarantees for Weakly Consistent Replicated Data"](https://www.cs.cornell.edu/courses/cs734/2000FA/cached%20papers/SessionGuaranteesPDIS_1.html) Paper. We are not talking about all of them, but these are some we are interested in:

1. **Monotonic Read Consistency:**

   - User $A$ sends update $\Delta_0$ to node $\alpha$.
   - Then, $A$ reads from node $\beta$ and reads $\Delta_1$.
   - A system has this property if $\Delta_0 \leq \Delta_1$.

1. **Monotonic Write Consistency:**

   - User $B$ writes $\Delta_0$ to node $\gamma$.
   - Then, $B$ writes $\Delta_1$ to node $\delta$.
   - A system has this property if the writes are observed in the order $\Delta_0 \rightarrow \Delta_1$.

1. **Read-Your-Writes Consistency:**

   - User $C$ writes $\Delta_0$ to node $\epsilon$.
   - Then, $C$ reads from node $\zeta$ and should see $\Delta_0$ or a more recent value.

1. **Write-Follows-Reads Consistency:**

   - User $D$ reads $\Delta_0$ from node $\eta$.
   - Then, $D$ writes $\Delta_1$ to node $\theta$.
   - A system has this property if $\Delta_1$ is based on $\Delta_0$ and respects the order of the reads and writes.

1. **Strong Consistency:**

   - User $E$ writes $\Delta_0$ to node $\iota$.
   - User $F$ reads from node $\kappa$ immediately after.
   - In a strongly consistent system, $F$ will see $\Delta_0$ without any delay.

1. **Sequential Consistency:**

   - User $G$ writes $\Delta_0$ to node $\lambda$.
   - User $H$ writes $\Delta_1$ to node $\mu$.
   - Users reading from any node will see the writes in the same order, but not necessarily in the real-time order.

1. **Causal Consistency:**

   - User $I$ writes $\Delta_0$ to node $\nu$.
   - User $J$ reads $\Delta_0$ from node $\xi$ and writes $\Delta_1$ to node $\pi$.
   - All users must see $\Delta_0$ before $\Delta_1$, but concurrent writes (e.g., by User $K$) can be seen in any order.

1. **Causal+ Consistency:**

   - User $L$ writes $\Delta_0$ to node $\rho$.
   - User $M$ writes $\Delta_1$ to node $\sigma$ concurrently.
   - The system ensures that all nodes eventually agree on the order of updates, resolving any conflicts.

This list is in an specific order, such that an earlier item implies a strong consistency level. Using this definition, we could surely put blockchains like Bitcoin and Cardano into the range from 1 to 6.

Along with higher consistency guarantees, comes separate costs and trade-offs. Latency, single points of failure, loss of resilience against network partitions, or simply throughput, are problems any distributed system will have to handle, but the stronger the consistency guarantee, the more those trade-offs become apparent.

But, and this is the question we asked ourselves, what if we could relax the requirements a bit, and maintain the guarantees as much as we could? Would that be enough?

### CALM: Consistency as Logical Monotonicity

The CALM Theorem was first described on the paper ["Consistency Analysis in Bloom: a CALM and Collected Approach"](https://dsf.berkeley.edu/papers/cidr11-bloom.pdf), and it connects the idea of distributed consistency to logical monoticity, that is, a consistent partial (or total) ordering of all the inputs that build a system.

This definition from the [Bloom Language website](http://bloom-lang.net/calm/) is very informative:

> Informally, a block of code is logically monotonic if it satisfies a simple property: adding things to the input can only increase the output.  By contrast, non-monotonic code may need to “retract” a previous output if more is added to its input.
>
> In general terms, the CALM principle says that:
>
> - Logically monotonic distributed code is eventually consistent without any need for coordination protocols (distributed locks, two-phase commit, paxos, etc.)
> - Eventual consistency can be guaranteed in any program by protecting non-monotonic statements (“points of order”) with coordination protocols.

### Being CALM Around Blockchains

How can we apply this to blockchains, where we need the strongest consistency levels? We can't have users messing with the state of other users on the chain, nor we want to rely on it for everything (which would make us not much better in regards to scaling).

We can think about this in another way, though. Assuming we are on top of a UTXO blockchain, a block (in a simplified way) contains only transactions, which we call $\Delta$ (Delta).

A $\Delta$ is just a mapping between inputs and outputs. $N$ inputs create $M$ outputs, as long as they follow a rule: the amounts in the inputs and the amounts on the outputs must be equal, no value can be created or destroyed.

If an input appears in a $\Delta$, it can not be consumed again. Doing so is called a "Double Spend". We can assume that, barring user or node misbehavings, avoiding double spend is the only thing that we need to do.

We can also assume that, as long as the inputs are different from two transactions, they are completely independent! They can be processed in parallel, or even out of order, it doesn't matter.

Let's then summarize our thesis:

> On a UTXO blockchain and in the absence of Double Spend, the system can be considered **Strongly Eventually Consistent**, meaning that all nodes will reach the same output, given the same inputs.

It becomes clear that if our goal is to have a high throughput, we need to embrace this characteristic, do more in parallel, and only pay the price of strong consistency when we actually need to.

More specifically, our goal is to go "as fast as gossip can go".

### Looking Around for Double Spending

TODO.

## Bibliography

- ["Session Guarantees for Weakly Consistent Replicated Data"](https://www.cs.cornell.edu/courses/cs734/2000FA/cached%20papers/SessionGuaranteesPDIS_1.html)
- ["A Critique of the CAP Theorem"](https://arxiv.org/abs/1509.05393)
- ["CAP Twelve Years Later: How the 'Rules' Have Changed"](https://www.infoq.com/articles/cap-twelve-years-later-how-the-rules-have-changed/) by Eric Brewer
- [Consistency Analysis in Bloom: a CALM and Collected Approach](https://dsf.berkeley.edu/papers/cidr11-bloom.pdf)

## License

µgraph (and all related projects inside the organization) is dual licensed under the [MIT](./LICENSE) and [Apache 2.0](./LICENSE.apache2) licenses. You are free to choose which one of the two choose your use-case the best, or please contact me if you need any form of expecial exceptions.

## Contributing Guidelines

All contributions are welcome, as long as they align with the goal of the project. If you are not sure whether or not what you want to implement is aligned with the goals of the project, just ask!

Don't be an asshole to anyone inside and out of the project and you'll be fine.
