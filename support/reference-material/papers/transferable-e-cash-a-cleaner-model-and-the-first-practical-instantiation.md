\title{
Transferable E-cash: A Cleaner Model and the First Practical Instantiation
}

\author{
Balthazar Bauer ${ }^{1}$, Georg Fuchsbauer ${ }^{2}$, and Chen Qian ${ }^{3}$ \\ ${ }^{1}$ Inria, ENS, CNRS, PSL, Paris, France \\ ${ }^{2}$ TU Wien, Austria \\ ${ }^{3}$ NTNU, Trondheim, Norway \\ first.last@\{ens.fr,tuwien.ac.at,ntnu.no\}
}

\begin{abstract}
Transferable e-cash is the most faithful digital analog of physical cash, as it allows users to transfer coins between them in isolation, that is, without interacting with a bank or a "ledger". Appropriate protection of user privacy and, at the same time, providing means to trace fraudulent behavior (double-spending of coins) have made instantiating the concept notoriously hard. Baldimtsi et al. (PKC'15) gave a first instantiation, but, as it relies on a powerful cryptographic primitive, the scheme is not practical. We also point out a flaw in their scheme.

In this paper we revisit the model for transferable e-cash and propose simpler yet stronger security definitions. We then provide the first concrete construction, based on bilinear groups, give rigorous proofs that it satisfies our model, and analyze its efficiency in detail.
\end{abstract}

Keywords: Transferable/offline e-cash, strong anonymity.

\section{Introduction}

Contrary to so-called "crypto"-currencies like Bitcoin [Nak08], a central ambition of the predating cryptographic e-cash has been user anonymity. Introduced by Chaum [Cha83], the goal was to realize a digital analog of physical cash, which allows users to pay without revealing their identity; and there has been a long line of research since [CFN88, Bra93, CHL05, BCKL09, FHY13, CPST16, BPS19] (to name only a few). In e-cash, a bank issues electronic coins to users, who can then spend them with merchants, who in turn can deposit them at the bank to get their account credited. User privacy should be protected in that not even the bank can link the withdrawing of a coin to its spending.

The main difference to the physical world is that digital coins can easily be duplicated, and therefore a so-called "double-spending" of a coin must be prevented. This can be readily achieved when all actors are online and connected (as with cryptocurrencies), since every spending is broadcast and payees simply refuse a coin that has already been spent.

Even in "anonymous" cryptocurrencies like Monero [vS13], which now also uses confidential transactions [Max15], or systems based on the Zerocoin/-cash [MGGR13, $\mathrm{BCG}^{+}$14] protocol, like Zcash $[Z e c 20]$, or on Mimblewimble [Poe16,

FOS19], users must be connected when they accept a payment, in order to prevent double-spending.

When users are allowed to spend coins to other users (or merchants) without continuous connectivity, then double-spending cannot be prevented; however, starting with [CFN88], ingenious methods have been devised in order to reveal a double-spender's identity while guaranteeing the privacy of all honest users.

Transferable e-cash. In all traditional e-cash schemes, including such "offline" e-cash, once a coin is spent (transferred) after withdrawal, it must be deposited at the bank by the payee. A more powerful concept, and much more faithful to physical e-cash, is transferable e-cash, which allows users to re-transfer obtained coins, while at the same time remaining offline. Note that cryptocurrencies are inherently online, and every transfer of a coin could be seen as depositing a coin (and marking it spent) and re-issuing a new one (in the ledger).

Transferable e-cash was first proposed by Okamoto and Ohta [OO89, OO91], but the constructions only guaranteed very weak forms of anonymity. It was then shown [CP93] that unbounded adversaries can recognize coins they owned before and that a coin must grow in size with every transfer (since information about potential double-spenders needs to be encoded in it).

While other schemes [Bla08, CGT08] only achieve unsatisfactory anonymity notions, Canard and Gouget [CG08] define a stronger notion (which we call coin transparency): it requires that a (polynomial-time) adversary cannot recognize a coin he has already owned when it is later given back to him. This is not achieved by physical cash, as banknotes can be marked by users (or the bank); however, if an e-cash scheme allowed a merchant to identify users by tracing the coins given out as change, then it would violate the central claim of ecash, namely anonymous payments. (Anonymous cryptocurrencies also satisfy a notion analogous to coin transparency.) A limitation of this notion is that the bank (more specifically, the part dealing with deposits) must be honest, as it needs to link occurrences of the same coin when detecting double-spending.

Prior schemes. The first scheme achieving coin transparency [CG08] was completely impractical, as at every transfer, the payer sends a proof of (a proof of $(\ldots))$ a coin that she received earlier. The first practical scheme was given by Fuchsbauer et al. [FPV09], but it makes unacceptable compromises elsewhere: when a double-spending is detected, all (even innocent) users up to the doublespender must give up their anonymity.

Blazy et al. $\left[\mathrm{BCF}^{+} 11\right]$ overcome this problem and propose a scheme that assumes a trusted party (called the "judge") that can trace all coins and users in the system and has to actively intervene in order to identify double-spenders. The scheme thus reneges on the promise that users remain anonymous as long as they follow the protocol. Moreover, their proof of anonymity was flawed, as shown by Baldimtsi et al. [BCFK15].

Despite all its problems, Blazy et al.'s $\left[\mathrm{BCF}^{+} 11\right]$ scheme, which elegantly combined randomizable non-interactive zero-knowledge (NIZK) proofs $\left[\mathrm{BCC}^{+} 09\right]$ and commuting signatures [Fuc11], serves as starting point for our construction.

In their scheme a coin consists of a signature by the bank and at every transfer the spender adds her own signature (thereby committing to her spending). To achieve anonymity, these signatures are not given in the clear; instead, coins are NIZK proofs of knowledge of signatures. Since the proofs can be rerandomized (that is, from a proof, anyone can produce a new proof of the same statement that looks unrelated to the original one), coins can change appearance after every transfer. Users will thus not recognize a coin when they see it again later, that is, the scheme satisfies coin transparency.

Baldimtsi et al. [BCFK15] give an instantiation that avoids the "judge" by using a double-spending-tracing mechanism from classical offline e-cash. They add "tags" to the coin that hide the identity of the owner of the coin, except when she spends the coin twice, then the bank can from two such tags compute the user's identity. Users must also include signatures in the coin during transfer, which represent irrefutable proof of double-spending.

The main drawback of their scheme is efficiency. They rely on the concept of malleable signatures [CKLM14], a generalization of digital signatures, where a signature on a message $m$ can be transformed into a signature on a message $T(m)$ for any allowed transformation $T$. Simulation unforgeability requires that from a signature one can extract all transformations it has undergone (even when the adversary that created it has seen "simulated" signatures).

In their scheme [BCFK15] a coin is a malleable signature computed by the bank, which can be transformed by a user if she correctly encodes her identity in a double-spending tag, adds an encryption (under the bank's public key) to it and randomizes all encryptions of previous tags cointained in the coin.

None of the previous schemes explicitly considers denominations of coins (and neither do we). This is because efficient ("compact") withdrawing and spending can be easily achieved if the bank associates different keys to different denominations (since giving change is readily supported in transferable e-cash). Note that, in contrast to cryptocurrencies, where every transaction is publicly posted, hiding the amount of a payment is meaningless in transferable e-cash.

Our contribution. Our contribution is two-fold:

Security model. We revisit the formal model for transferable e-cash, starting from [BCFK15], whose model was a refined version of earlier ones. We first give a definition of correctness, which was lacking in previous works. We then exhibit attacks against users who follow the protocol, against which previous models did not protect:
- When a user receives a coin (that is, the protocol accepts the received coin), then previous models did not guarantee that this coin will be accepted by other (honest) users when transferred. An adversary could thus send a malformed coin to a user, which the latter accepts but can then not spend.
- There were also no guarantees against a malicious bank which at coin deposit refuses to credit the user's account (e.g., by claiming that the coin was invalid or had been double-spent). In our model, when the bank refuses a coin, it must accuse a user of double-spending and provide a proof for this.

We then simplify the anonymity definitions, which in earlier version had been cluttered with numerous oracles the adversary has access to, and for which the intuitive notion that they were formalizing was hard to grasp. While our definitions are simpler, they are stronger in that they imply previous definitions (except for the previous notion of "spend-then-receive (StR) anonymity", whose existing formalizations we argue are not relevant in practice).

We also show that the proof of "StR anonymity" (a notion similar to coin transparency) of the scheme from [BCFK15] is flawed and that it only satisfies a weakening of the notion (Sect. 3.2).

Instantiation. Our main contribution is a transferable e-cash scheme, which we prove satisfies our security model, and which is much more efficient than the only previous realization [BCFK15]. Unfortunately, the authors do not provide concrete numbers, as they use malleable signatures in a blackbox way. These signatures are the main source of inefficiency, due to their generality and the strong security notions in the spirit of simulation-sound extractability, requiring that a coin (i.e., a malleable signature) stores every transformation it has undergone.

In contrast, we give a direct construction from the following primitives: Groth-Sahai proofs [GS08], which are randomizable; structure-preserving signatures $\left[\mathrm{AFG}^{+} 10\right]$, which are compatible with GS proofs; and rerandomizable encryption satisfying RCCA-security [CKN03] (the corresponding variant of CCA security, see Fig. 6). While we use signature schemes from the literature [AGHO11, Fuc11], we construct a new RCCA-secure encryption scheme that is tailored to our scheme, basing it on prior work [LPQ17]. Finally, our scheme also uses the (efficient) double-spending tags used previously [BCFK15].

Due to the existence of an omnipotent "judge", no such tags were required by Blazy et al. $\left[\mathrm{BCF}^{+} 11\right]$. Interestingly, although we do not assume any active trusted parties, we achieve a comparable efficiency, which is a result of realizing the full potential of the tags: previously [BCFK15], they had only served to encode a user's identity; but, as we show, they can in addition be used to commit the user. This allows us, contrary to all previous instantiations, to completely forgo the inclusion of user signatures in the coins, which considerably reduces their size. For a more detailed (informal) overview of our scheme see Sect. 5.1.

In terms of efficiency, our coins grow by around 100 elements from a bilinear group per transfer (see table on p. 30). We view this as practical by current standards, especially in view of numbers for deployed schemes: e.g., the parameters for Zcash consist of several 100000 bilinear-group elements [Zec20].

\section{Definition of transferable e-cash}

The syntax and security definitions we present in the following are refinements of earlier work [CG08, $\mathrm{BCF}^{+} 11$, BCFK15].

\subsection{Algorithms and protocols}

An e-cash scheme is set up by running ParamGen and the bank generating its key pair via BKeyGen. The bank maintains a list of users $\mathcal{U} \mathcal{L}$ and a list of deposited
coins $\mathcal{D C L}$. Users run the protocol Register with the bank to obtain their secret key, and their public keys are added to $\mathcal{U}$. With her secret key a user can run Withdraw with the bank to obtain coins, which she can transfer to others via the protocol Spend.

Spend is also used when a user deposits a coin at the bank. After receiving a coin, the bank runs CheckDS (for "double-spending") on it and the previously deposited coins in $\mathcal{D C L}$, which determines whether to accept the coin. If so, it is added to $\mathcal{D C} \mathcal{L}$; if not (in case of double-spending), CheckDS returns the public key of the accused user and a proof $\Pi$, which can be verified using VfyGuilt.

ParamGen $\left(1^{\lambda}\right)$, on input the security parameter $\lambda$ in unary, outputs public parameters par, which are an implicit input to all of the following algorithms.

BKeyGen() is run by the bank $\mathcal{B}$ and outputs its public key $p k_{\mathcal{B}}$ and its secret key $s k_{\mathcal{B}}=\left(s k_{\mathcal{W}}, s k_{\mathcal{D}}, s k_{\mathcal{C K}}\right)$, where $s k_{\mathcal{W}}$ is used to issue coins in Withdraw and to register users in Register; $s k_{\mathcal{D}}$ is used as the secret key of the receiver when coins are deposited via Spend; and $s k_{\mathcal{C K}}$ is used for CheckDS.

Register $\left\langle\mathcal{B}\left(s k_{\mathcal{W}}\right), \mathcal{U}\left(p k_{\mathcal{B}}\right)\right\rangle$ is a protocol between the bank and a user. The user obtains a secret key sk and the bank gets $p k$, which it adds to $\mathcal{U}$. In case of error, they both obtain $\perp$.

Withdraw $\left\langle\mathcal{B}\left(s k_{\mathcal{W}}\right), \mathcal{U}\left(s k_{\mathcal{U}}, p k_{\mathcal{B}}\right)\right\rangle$ is run between the bank and a user, who outputs a coin $c$ (or $\perp$ ), while the bank outputs ok (in which case it debits the user's account) or $\perp$.

Spend $\left\langle\mathcal{U}\left(c, s k, p k_{\mathcal{B}}\right), \mathcal{U}^{\prime}\left(s k^{\prime}, p k_{\mathcal{B}}\right)\right\rangle$ is run between two users and lets $\mathcal{U}$ spend a coin $c$ to $\mathcal{U}^{\prime}$ (who could be the bank). $\mathcal{U}^{\prime}$ outputs a coin $c^{\prime}$ (or $\perp$ ), while $\mathcal{U}$ outputs ok (or $\perp$ ).

CheckDS $\left(\operatorname{sk}_{\mathcal{C}}, \mathcal{U} \mathcal{L}, \mathcal{D C L}, c\right)$, run by the bank, takes as input its checking key, the lists of registered users $\mathcal{U} \mathcal{L}$ and of deposited coins $\mathcal{D C L}$ and a coin $c$. It outputs an updated list $\mathcal{D C} \mathcal{L}$ (when the coin is accepted) or a user public key $p k_{\mathcal{U}}$ and an incrimination proof $\Pi$.

VfyGuilt $\left(p k_{\mathcal{U}}, \Pi\right)$ can be executed by anyone. It takes a user public key and an incrimination proof and returns 1 (acceptance of $\Pi$ ) or 0 (rejection).

Note that we define a transferable e-cash scheme as stateless, in that there is no state information shared between the algorithms. A withdrawn coin, whether it was the first or the $n$-th coin issues to a specific user, is always distributed the same. Moreover, a received coin will only depend on the spent coin (and not on other spent or received coins). Thus, the bank and the users need not store anything about past transactions for transfer; the coin itself must be sufficient.

In particular, the bank can separate withdrawing from depositing, in that CheckDS, used during deposit, need not be aware of the withdrawn coins.

\subsection{Correctness properties}

These properties were not stated in previous models. They are important in that they preclude schemes that satisfy security notions by not doing anything.

Let par be an output of ParamGen(1 $\left.1^{\lambda}\right)$ and $\left(s k_{\mathcal{B}}=\left(s k_{\mathcal{W}}, s k_{\mathcal{D}}, s k_{\mathcal{C K}}\right), p k_{\mathcal{B}}\right)$ be output by BKeyGen(par). Then the following holds:
- none of the outputs is $\perp$;
- any execution of Register $\left\langle\mathcal{B}\left(s k_{\mathcal{W}}\right), \mathcal{U}\left(p k_{\mathcal{B}}\right)\right\rangle$ yields output $p k$ for $\mathcal{B}$ and $s k$ for $\mathcal{U}$.

Further, let $s k$ and $s k^{\prime}$ be two user outputs of Register; then:
- any execution of Withdraw $\left\langle\mathcal{B}\left(s k_{\mathcal{W}}\right), \mathcal{U}\left(s k, p k_{\mathcal{B}}\right)\right\rangle$ yields ok for $\mathcal{B}$ and $c$ for $\mathcal{U}$;
- in an execution of Spend $\left\langle\mathcal{U}\left(c, s k, p k_{\mathcal{B}}\right), \mathcal{U}^{\prime}\left(s k^{\prime}, p k_{\mathcal{B}}\right)\right\rangle$, no party outputs $\perp$;
- $s k_{\mathcal{D}}$ works as a user secret key $s k^{\prime}$.

(Note that correctness of CheckDS and VfyGuilt is implied by the security notions below.)

\subsection{Security definitions}

Global variables. In our security games, we store all information about users and their keys in the user list $\mathcal{U} \mathcal{L}$. Its entries are of the form $\left(p k_{i}, s k_{i}, u d s_{i}\right)$, where $u d s_{i}$ indicates how many times user $\mathcal{U}_{i}$ has double-spent.

In the coin list $\mathcal{C L}$, we keep information about the coins created in the system. For each withdrawn or spent coin $c$, we store a tuple (owner, $c, c d s$, origin), where owner stores the index $i$ of the user who withdrew or received the coin (coins withdrawn or received by the adversary are not stored). We also include cds, which counts how often this specific instance of the coin has been spent. We set origin to " $\mathcal{B}$ " if the coin was issued by the honest bank and to " $\mathcal{A}$ " if it originates from the adversary; if the coin was originally spent by the challenger itself, we store a pointer indicating which original coin this transferred coin corresponds to. Finally, we maintain a list of deposited coins $\mathcal{D C L}$.

Oracles. We now define oracles used in the security definitions, which differ depending on whether the adversary impersonates a corrupt bank or users. If during the oracle execution an algorithm fails (i.e., it outputs $\perp$ ) then the oracle also stops. Otherwise the call to the oracle is considered successful; a successful deposit oracle call must also not detect any double-spending.

Registration and corruption of users. The adversary can instruct the creation of honest users and either play the role of the bank during registration, or passively observe registration. It can moreover "spy" on users, meaning it can learn the user's secret key. This will strengthen yet simplify our anonymity games compared to [BCFK15], where once the adversary had learned the secret key of a user (by "corrupting" her), the user could not be a challenge user in the anonymity games anymore (yielding selfless anonymity, while we achieve full anonymity).

BRegist() plays the bank side of Register and interacts with $\mathcal{A}$. If successful, it adds $(p k, \perp, u d s=0)$ to $\mathcal{U} \mathcal{L}$ (where $u d s$ is the number of double-spends).

URegist() plays the user side of the Register protocol when the bank is controlled by the adversary. Upon successful execution, it adds $(p k, s k, 0)$ to $\mathcal{U} \mathcal{L}$.

Regist() plays both parties in the Register protocol and adds $(p k, s k, 0)$ to $\mathcal{U} \mathcal{L}$. $\operatorname{Spy}(i)$, for $i \leq|\mathcal{U}|$, returns user $i$ 's secret key $\operatorname{sk}_{i}$.

Withdrawal oracles. The adversary can either withdraw a coin from the bank, play the role of the bank, or passively observe a withdrawal.

BWith() plays the bank side of the Withdraw protocol. Coins withdrawn by $\mathcal{A}$ (and thus unknown to the experiment) are not added to the coin list $\mathcal{C L}$.

UWith $(i)$ plays user $i$ in Withdraw when the bank is controlled by the adversary. Upon obtaining a coin $c$, it adds (owner $=i, c, c d s=0$, origin $=\mathcal{A}$ ) to $\mathcal{C L}$.

With(i) simulates a Withdraw protocol execution playing both $\mathcal{B}$ and user $i$. It adds (owner $=i, c, c d s=0$, origin $=\mathcal{B}$ ) to $\mathcal{C} \mathcal{L}$.

Spend and deposit oracles.

$\operatorname{Spd}(j)$ spends the coin from the $j$-th entry (owner ${ }_{j}, c_{j}, c d s_{j}$, origin $_{j}$ ) in $\mathcal{C} \mathcal{L}$ to $\mathcal{A}$, who could be impersonating a user, or the bank during a deposit. The oracle plays $\mathcal{U}$ in the Spend protocol with secret key $s k_{\text {owner }_{j}}$. It increments the coin spend counter $c d s_{j}$ by 1 . If afterwards $c d s_{j}>1$, then the owner's double-spending counter $u d s_{\text {owner }_{j}}$ is incremented by 1 .

$\operatorname{Rcv}(i)$ makes honest user $i$ receive a coin from $\mathcal{A}$. The oracle plays $\mathcal{U}^{\prime}$ with user $i$ 's secret key in the Spend protocol. It adds a new entry (owner $=i, c, c d s=0$, origin $=\mathcal{A}$ ) to $\mathcal{C} \mathcal{L}$.

$\operatorname{S} \& \mathrm{R}(j, i)$ spends the $j$-th coin in $\mathcal{C}$ to user $i$. It runs $(o k, c) \leftarrow \operatorname{Spend}\left\langle\mathcal{U}\left(c_{j}\right.\right.$, sk owner $\left.\left._{j}, p k_{\mathcal{B}}\right), \mathcal{U}^{\prime}\left(s k_{i}, p k_{\mathcal{B}}\right)\right\rangle$ and adds (owner $=i, c, c d s=0$, pointer $=j$ ) to $\mathcal{C}$. It increments the coin spend counter $c d s_{j}$ by 1 . If afterwards $c d s_{j}>1$, then $u d s_{\text {owner }_{j}}$ is incremented by 1.

BDepo() lets $\mathcal{A}$ deposit a coin. It runs $\mathcal{U}^{\prime}$ in Spend using the bank's secret key $s k_{\mathcal{D}}$ with the adversary playing $\mathcal{U}$. If successful, it runs CheckDS on the received coin and updates $\mathcal{D C L}$ accordingly; else it outputs a pair $(p k, \Pi)$.

$\operatorname{Depo}(j)$, the honest deposit oracle, runs Spend between the owner of the $j$-th coin in $\mathcal{C L}$ and an honest bank. If successful, it increments $c d s_{j}$ by 1 ; if afterwards $c d s_{j}>1$, it also increments $u d s_{\text {owner }_{j}}$. It runs CheckDS on the received coin and either updates $\mathcal{D C L}$ or returns a pair $(p k, \Pi)$.

(Note that no oracle "UDepo" is required, since Spd lets the adversarial bank have an honest user deposit a coin.)

\subsection{Economic properties}

We distinguish two types of security properties of transferable e-cash schemes. Besides anonymity notions, economic properties ensure that neither the bank nor users will incur an economic loss when participating in the system.

The following property was not required in any previous formalization of transferable e-cash in the literature and is analogous the property clearing defined for classical e-cash [BPS19].

```
Expt
    par \leftarrowParamGen(\mp@subsup{1}{}{\lambda}); p\mp@subsup{k}{\mathcal{B}}{}\leftarrow\mathcal{A}(par)

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-08.jpg?height=41&width=322&top_left_y=473&top_left_x=717)
```
    If b=0 then run UWith(i
    Else run }\operatorname{Rcv}(\mp@subsup{i}{1}{})\mathrm{ with }\mathcal{A
    If this outputs }\perp\mathrm{ then return 0

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-08.jpg?height=38&width=750&top_left_y=629&top_left_x=720)
```
    Return 0
```

Fig. 1. Game for soundness (protecting users from financial loss)

Soundness. If an honest user accepted a coin during a withdrawal or a transfer, then she is guaranteed that the coin will be accepted by others, either honest users when transferring, or the bank when depositing. The game is formalized in Fig. 1 where $i_{2}$ plays the role of the receiver of a spending or the bank. For convenience, we define probabilistic polynomial-time (PPT) adversaries $\mathcal{A}$ to be stateful in all our security games.

Definition 1 (Soundness). A transferable e-cash system is sound if for any $P P T \mathcal{A}$, we have $\operatorname{Adv}_{\mathcal{A}}^{\text {sound }}(\lambda):=\operatorname{Pr}\left[\operatorname{Expt}_{\mathcal{A}}^{\text {sound }}(\lambda)=1\right]$ is negligible in $\lambda$.

Unforgeability. This notion covers both unforgeability and user identification from [BCFK15] (which were not consistent as we explain in Sect. 3.2). It protects the bank, ensuring that no (coalition of) users can spend more coins than the number of coins they withdrew.

Unforgeability also guarantees that whenever a coin is deposited and refused by CheckDS, the latter also returns the identity of a registered user, who is accused of double-spending. (Exculpability, below, ensures that no innocent user will be accused.) The game is formalized in Fig. 2 and lets the adversary impersonate all users.

Definition 2 (Unforgeability). A transferable e-cash system is unforgeable if $\operatorname{Adv}_{\mathcal{A}}^{\text {unforg }}(\lambda):=\operatorname{Pr}\left[\operatorname{Expt}_{\mathcal{A}}^{\text {unforg }}(\lambda)=1\right]$ is negligible in $\lambda$ for any $P P T \mathcal{A}$.

$$
\begin{aligned}
& \operatorname{Expt}_{\mathcal{A}}^{\text {unforg }}(\lambda) \text { : } \\
& p \operatorname{par} \leftarrow \text { ParamGen }\left(1^{\lambda}\right) ;\left(s k_{\mathcal{B}}, p k_{\mathcal{B}}\right) \leftarrow \text { BKeyGen }(p a r) \\
& \mathcal{A}^{\text {BRegist,BWith,BDepo }}\left(p \text { ar, } p k_{\mathcal{B}}\right) \\
& \text { If in a BDepo call, CheckDS does not return a coin list: } \\
& \text { Return } 1 \text { if any of the following hold: } \\
& \quad-\text { CheckDS outputs } \perp \\
& \quad-\text { CheckDS outputs }(p k, \Pi) \text { and } V \text { fyGuilt }(p k, \Pi)=0 \\
& \quad-\text { CheckDS outputs }(p k, \Pi) \text { and } p k \notin \mathcal{U L} \\
& \text { Let } q_{W} \text { be the number of calls to BWith } \\
& \text { If } q_{W}<|\mathcal{D C L}| \text {, then return } 1 \\
& \text { Return } 0
\end{aligned}
$$

Fig. 2. Game for unforgeability (protecting the bank from financial loss)

```
Expt 
    par \leftarrow ParamGen(1^); pk

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-09.jpg?height=46&width=634&top_left_y=470&top_left_x=778)
```
    Return 1 if all of the following hold:
        - VfyGuilt(p\mp@subsup{k}{\mp@subsup{i}{}{*}}{},\mp@subsup{\Pi}{}{*})=1
        - There was no call Spy ( i*)
        - uds
```
Return 0

Fig. 3. Game for exculpability (protecting honest users from accusation)

Exculpability. This notion, a.k.a. non-frameability, ensures that the bank, even when colluding with malicious users, cannot wrongly accuse an honest user of double-spending. Specifically, it guarantees that an adversarial bank cannot produce a double-spending proof $\Pi^{*}$ that verifies for the public key of a user $i^{*}$ that has never double-spent. The game is formalized as in Fig. 3.

Definition 3 (Exculpability). A transferable e-cash system is exculpable if $\operatorname{Adv}_{\mathcal{A}}^{\text {excul }}(\lambda):=\operatorname{Pr}\left[\operatorname{Expt}_{\mathcal{A}}^{\mathrm{excul}}(\lambda)=1\right]$ is negligible in $\lambda$ for any $P P T \mathcal{A}$.

\subsection{Anonymity properties}

Instead of following previous anonymity notions $\left[\mathrm{BCF}^{+} 11, \mathrm{BCFK} 15\right]$, we introduce new ones which clearly distinguish between the adversary's capabilities; in particular, whether it is able to detect double-spending. When the adversary impersonates the bank, we consider two cases: user anonymity and coin anonymity (and explain why this distinction is necessary).

As transferred coins necessarily grow in size [CP93], we can only guarantee indistinguishability of comparable coins. We therefore define $\operatorname{comp}\left(c_{1}, c_{2}\right)=1 \mathrm{iff}$ $\operatorname{size}\left(c_{1}\right)=\operatorname{size}\left(c_{2}\right)$, where $\operatorname{size}(c)=1$ after $c$ was withdrawn and it increases by 1 after each transfer.

Coin anonymity. This notion is closest to (and implies) the anonymity notion of classical e-cash: an adversary, who also impersonates the bank, issues two coins to the challenger and when she later receives them (via a deposit in classical ecash), she should not be able to associate them to their issuances. In transferable e-cash, we allow the adversary to determine two series of honest users via which the coins are respectively transfered before being given back to the adversary.

The experiment is specified on the left of Fig. 4: users $i_{0}^{(0)}$ and $i_{0}^{(1)}$ withdraw a coin from the adversarial bank, user $i_{0}^{(0)}$ passes it to $i_{1}^{(0)}$, who passes it to $i_{2}^{(0)}$, etc., In the end, the last users of the two chains spend the coins to the adversary, but the order in which this happens depends on a bit $b$ that parametrizes the game, and which the adversary must decide.

User anonymity. Coin anonymity required that users who transfer the coin are honest. If one of the users through which the coin passes colluded with the bank,

```
$\operatorname{Expt}_{\mathcal{A}, b}^{\mathrm{c}-\mathrm{an}}(\lambda)$
    $\operatorname{par} \leftarrow \operatorname{ParamGen}\left(1^{\lambda}\right)$
    $p k_{\mathcal{B}} \leftarrow \mathcal{A}(p a r)$
    $i_{0}^{(0)} \leftarrow \mathcal{A}^{\text {URegist,Spy }} ;$ run UWith $\left(i_{0}^{(0)}\right)$ with $\mathcal{A}$
    $i_{0}^{(1)} \leftarrow \mathcal{A}^{\text {URegist,Spy }} ;$ run UWith $\left(i_{0}^{(1)}\right)$ with $\mathcal{A}$
    $\left(\left(i_{1}^{(0)}, \ldots, i_{k_{0}}^{(0)}\right),\left(i_{1}^{(1)}, \ldots, i_{k_{1}}^{(1)}\right)\right)$
                                    $\leftarrow \mathcal{A}^{\text {URegist,Spy }}$
    If $k_{0} \neq k_{1}$ then return 0
    For $j=1, \ldots, k_{0}$ :
        Run S\&R $\left(2 j-1, i_{j}^{(0)}\right)$
        Run $\mathrm{S} \& \mathrm{R}\left(2 j, i_{j}^{(1)}\right)$
    Run $\operatorname{Spd}\left(2 k_{0}+1+b\right)$ with $\mathcal{A}$
    $\operatorname{Expt}_{\mathcal{A}, b}^{\mathrm{u}-\mathrm{an}}(\lambda):$
    Run $\operatorname{Spd}\left(2 k_{0}+2-b\right)$ with $\mathcal{A}$
    par $\leftarrow$ ParamGen $\left(1^{\lambda}\right)$
    $p k_{\mathcal{B}} \leftarrow \mathcal{A}(p a r)$
    $\left(i_{0}^{(0)}, i_{0}^{(1)}\right) \leftarrow \mathcal{A}^{\text {URegist,Spy }}$
    Run $\operatorname{Rcv}\left(i_{b}\right)$ with $\mathcal{A}$
    $\left(\left(i_{1}^{(0)}, \ldots, i_{k_{0}}^{(0)}\right),\left(i_{1}^{(1)}, \ldots, i_{k_{1}}^{(1)}\right)\right)$
$\leftarrow \mathcal{A}^{\text {URegist,Spy }}$
    If $k_{0} \neq k_{1}$ then return 0
    For $j=1, \ldots, k_{0}$ :
        $\operatorname{Run} \operatorname{S} \& \mathrm{R}\left(j, i_{j}^{(b)}\right)$
    $\operatorname{Run} \operatorname{Spd}\left(k_{0}+1\right)$ with $\mathcal{A}$
    $b^{*} \leftarrow \mathcal{A}$; return $b^{*}$
    $b^{*} \leftarrow \mathcal{A}$; return $b^{*}$
```

Fig. 4. Games for coin and user anonymity (protecting users from a malicious bank)

there would be a trivial attack: after receiving the two challenge coins, the bank simulates the deposit of one of them and the deposit of the coin intercepted by the colluding user. If a double-spending is detected, it knows that the received coin corresponds to the sequence of users which the colluder was part of.

Since double-spending detection is an essential feature of e-cash, attacks of this kind are impossible to prevent. However, we still want to guarantee that, while the bank can trace coins, the involved users remain anonymous. We formalize this in the game on the right of Fig. 4, where, in contrast to coin anonymity, there is only one coin and the adversary must distinguish the sequence of users through which the coin passes before returning to her. In contrast to coin anonymity, we now allow the coin to already have some "history", rather than being freshly withdrawn.

Coin transparency. This is in some sense the strongest anonymity notion and it implies that a user that transfers a coin cannot recognize it if she receives it again. As the bank can necessarily trace coins (for double-spending detection), it is assumed to be honest for this notion. Actually, only the detection key $s k_{\mathcal{C K}}$ must remain hidden from the adversary, while $s k_{\mathcal{W}}$ and $s k_{\mathcal{D}}$ can be given.

The game formalizing this notion, specified in Fig. 5, is analogous to coin anonymity, except that the challenge coins are not freshly withdrawn; instead, the adversary spends two coins of its choice to users of its choice, both are passed through a sequence of users of the adversary's choice and one of them is returned to the adversary.

There is another trivial attack that we need to exclude: the adversary could deposit the coin that is returned to him and one, say the first, of the coins he initially transfered to an honest user. Now if the deposit does not succeed because of double-spending, the adversary knows that it was the first coin that was returned to him. Again, this attack is unavoidable due to the necessity of

```
Expt 

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-11.jpg?height=43&width=892&top_left_y=434&top_left_x=562)
```
    DCL'

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-11.jpg?height=38&width=895&top_left_y=518&top_left_x=561)
```
    i(0)}\leftarrow\mp@subsup{\mathcal{A}}{}{\mathrm{ URegist,BDepo},\textrm{Spy }}(par,p\mp@subsup{k}{\mathcal{B}}{},sk\mathcal{W},s\mp@subsup{k}{\mathcal{D}}{}
        // BDepo' uses CheckDS' (},\cdot,\cdot,\cdot,\mathcal{DCL
    Run }\operatorname{Rcv}(\mp@subsup{i}{}{(0)})\mathrm{ with }\mathcal{A}\mathrm{ ; let }\mp@subsup{c}{0}{}\mathrm{ be the received coin stored in }\mathcal{CL}[1
    xo}\leftarrow\mathrm{ CheckDS(sk

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-11.jpg?height=41&width=805&top_left_y=711&top_left_x=562)
```
    DCL}\mp@subsup{\mathcal{L}}{}{\prime}\leftarrow\mathrm{ CheckDS(sk
    i ^ { ( 1 ) } \leftarrow \mathcal { A } ^ { \text { URegist,BDepo,Spy} }

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-11.jpg?height=46&width=908&top_left_y=823&top_left_x=560)
```
    x

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-11.jpg?height=40&width=808&top_left_y=907&top_left_x=561)
```
    If }\operatorname{comp}(\mp@subsup{c}{0}{},\mp@subsup{c}{1}{})\neq1\mathrm{ then abort
    x}\leftarrow\mathrm{ CheckDS(sk

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-11.jpg?height=41&width=1041&top_left_y=1020&top_left_x=561)
```
    ((i
    If }\mp@subsup{k}{0}{}\neq\mp@subsup{k}{1}{}\mathrm{ then abort
    If (}\mp@subsup{k}{b}{}\neq0)\mathrm{ then run S&R (b+1,i,il
    For j = 2, ,., ko: // ...the received coin is placed in \mathcal{CL}[3]

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-11.jpg?height=60&width=705&top_left_y=1214&top_left_x=642)
```

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-11.jpg?height=44&width=780&top_left_y=1260&top_left_x=564)
```
    b*}\leftarrow\mp@subsup{\mathcal{A}}{}{\mathrm{ BDepo}};\mathrm{ return b b
CheckDS'(sk
    x \leftarrow \text { CheckDS (sk}

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-11.jpg?height=47&width=936&top_left_y=1451&top_left_x=562)
```
        ctr}\leftarrowc\operatorname{tr}+
        If ctr > 1 then abort

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-11.jpg?height=55&width=477&top_left_y=1569&top_left_x=561)

Fig. 5. Game for coin transparency (protecting users from malicious users)

double-spending detection. It is a design choice that lies outside of our model to implement sufficient deterrence from double-spending, so it would exceed the utility of breaking anonymity.

This is the reason why the game aborts if the adversary deposits twice a coin from the set of "challenge coins" (consisting of the two coins the adversary transfers and the one it receives). The variable ctr counts how many times a coin from this set was deposited. Note also that because $\mathcal{A}$ has $s k_{\mathcal{W}}$, and can therefore create unregistered users, we do not consider $\mathcal{U} \mathcal{L}$ in this game.

Definition 4 (Anonymity). For $\mathrm{x} \in\{\mathrm{c}-\mathrm{an}, \mathrm{u}-\mathrm{an}, \mathrm{c}-\mathrm{tr}\}$ a transferable e-cash scheme satisfies x if $\operatorname{Adv}_{\mathcal{A}}^{\mathrm{x}}(\lambda):=\operatorname{Pr}\left[\operatorname{Expt}_{\mathcal{A}, 1}^{\mathrm{x}}(\lambda)=1\right]-\operatorname{Pr}\left[\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{x}}(\lambda)=1\right]$ is negligible in $\lambda$ for any PPT adversary $\mathcal{A}$.

\section{Comparison with previous work}

\subsection{Model comparison}

In order to justify our new model, we start with discussing a security vulnerability of the previous model [BCFK15].

Issues with economical notions. As already pointed out in Sect. 2.2, the correctness properties were missing in previous models.

No soundness guarantees. In none of the previous models was there a security notion that guaranteed that an honest user could successfully transfer a coin to another honest user or the bank, even if the coin was obtained regularly.

Fuzzy definition of "unsuccessful deposit". Previous models defined a protocol called "Deposit", which we separated into an interactive (Spend) and a static part (CheckDS). In their definition of unforgeability, the authors [BCFK15] use the concept of "successful deposit", which was not clearly defined, since an "unsuccessful deposit" could mean one of the following:
- The bank detects a double-spending and provides a proof accusing the cheater (who could be different from the depositer).
- The user did not follow the protocol (e.g., by sending a malformed coin), in which case we cannot expect a proof of guilt from the bank.
- The user followed the protocol but using a coin that was double-spent (either earlier or during deposit); however, the bank does not obtain a valid proof of guilt and outputs $\perp$.

Our interpretation of the definition in [BCFK15] is that it does not distinguish the second and the third case. This is an issue, as the second case cannot be avoided (and must be dealt with outside the model, e.g. by having users sign their messages). But the third case should be avoided so the bank does not lose money without being able to accuse the cheater. This is now guaranteed by our unforgeability notion in Def. 2.

Simplification of anonymity definitions. We believe that our notions are more intuitive and simpler (e.g. by reducing the number of oracles of previous work). Our notions imply prior notions from the literature: we can prove that the existence of an adversary in a game from a prior notion implies the existence of an adversary in one of our games. (The general idea is to simulate most of the oracles using the secret keys of the bank or users, which in our notions can be obtained via the Spy oracle.) In particular, the implications are the following:

$$
\mathrm{c}-\mathrm{an} \Rightarrow \text { OtR-fa } \quad \text { and } \quad u-\mathrm{an} \Rightarrow \mathrm{StR} *-\mathrm{fa}
$$

where OtR-fa is observe-then-receive full anonymity $\left[\mathrm{CG} 08, \mathrm{BCF}^{+} 11\right.$, BCFK15] and StR*-fa is a variant of spend-then-receive full anonymity from [BCFK15].

The earlier notion StR-fa [CG08, $\mathrm{BCF}^{+}$11] is similar to our coin transparency c-tr, with the following differences: in StR-fa, when the adversary deposits a coin, the bank provides a guilt proof when it can; and StR-fa lets the adversary obtain user secret keys. Coin transparency would imply StR-fa if CheckDS replaced its argument $\mathcal{U} \mathcal{L}$ by $\emptyset$. This change is justified since (in both StR-fa and $c-t r$ ) the adversary can create unregistered users (using $s k_{\mathcal{W}}$ ), and thus CheckDS could return $\perp$ because it cannot accuse anyone in $\mathcal{U} \mathcal{L}$.

Moreover, no previous scheme, including [BCFK15] achieves StR-fa, as we show next.

\subsection{A flaw in a proof in BCFK15}

The authors of [BCFK15] claim that their scheme satisfies StR-fa as defined in $\left[\mathrm{BCF}^{+} 11\right]$ (after having discovered an error in the StR-fa proof of the scheme of that paper). To achieve this anonymity notion (the most difficult one, as they note), they use malleable signatures, which guarantee that whenever an adversary, after obtaining simulated signatures, outputs a valid message/signature pair $(m, \sigma)$, it must have derived the pair from received signatures. Formally, there exists an extractor that can extract a transformation from $\sigma$ that links $m$ to the messages on which the adversary queried signatures.

In the game formalizing StR-fa $\left[\mathrm{BCF}^{+} 11\right]$ (analogously to Expt ${ }^{\mathrm{c} \text { tr }}$ in Fig. 5) the adversary receives $s k_{\mathcal{W}}$, which formalizes the notion that the part of the bank that issues coins can be corrupt. In their scheme [BCFK15], sk $\mathcal{W}_{\text {w }}$ contains the signing key for the malleable signatures. However, with this the adversary can easily compute a fresh signature, and thus no extractor can recover a transformation explaining the signed message. This shows that a scheme based on malleable signatures only satisfies a weaker notion of StR-fa/c-tr, where all parts of the bank must be honest.

In contrast, we prove that our scheme satisfies c-tr, and it can therefore be seen as the first scheme to satisfy the "spirit" of StR-fa, which is captured by our notion c-tr.

\section{Primitives used in our construction}

\subsection{Bilinear groups}

The building blocks of our scheme will be defined over a (Type-3, i.e., asymmetric) bilinear group, which is a tuple $G r=\left(p, \mathbb{G}, \hat{\mathbb{G}}, \mathbb{G}_{T}, e, g, \hat{g}\right)$, where $\mathbb{G}, \hat{\mathbb{G}}$ and $\mathbb{G}_{T}$ are groups of prime order $p ;\langle g\rangle=\mathbb{G},\langle\hat{g}\rangle=\hat{G}$, and $e: \mathbb{G} \times \hat{\mathbb{G}} \rightarrow \mathbb{G}_{T}$ is a bilinear map (i.e., for all $a, b \in \mathbb{Z}_{p}: e\left(g^{a}, \hat{g}^{b}\right)=e(g, \hat{g})^{a b}$ ) so that $e(g, \hat{g})$ generates $\mathbb{G}_{T}$. We assume that the groups are discrete-log-hard and other computational assumptions (DDH, CDH, SXDH, etc. defined in Appendix D) hold as well. We assume that there exists an algorithm GrGen that, on input the security parameter $\lambda$ in unary, outputs the description of a bilinear group with $p \geq 2^{\lambda-1}$.

\subsection{Randomizable proofs of knowledge and signatures}

Commit-and-prove proof systems. As coins must be unforgeable, at their core lie digital signatures. To achieve anonymity, these must be hidden, which can be achieved via non-interactive zero-knowledge (NIZK) proofs of knowledge; if these proofs are re-randomizable, then they can not even be recognized by a past owner. We will use Groth-Sahai NIZK proofs [GS08], which are randomizable [FP09, $\mathrm{BCC}^{+}$09] and include commitments to the witnesses.

We let $\mathcal{V}$ be set of values that can be committed, $\mathcal{C}$ be the set of commitments, $\mathcal{R}$ the randomness space and $\mathcal{E}$ the set of equations (containing equality) whose satisfiability can be proved. We assume that $\mathcal{V}$ and $\mathcal{R}$ are groups. We will use an extractable commitment scheme, which consists of the following algorithms:

C.Setup(Gr) takes as input a description of a bilinear group and returns a commitment key $c k$, which implicitly defines the sets $\mathcal{V}, \mathcal{C}, \mathcal{R}$ and $\mathcal{E}$.

C.ExSetup(Gr) returns an extraction key $x k$ in addition to a commitment key $c k$. C.SmSetup $(G r)$ returns a commitment key $c k$ and a simulation trapdoor $t d$.

C.Cm $(c k, v, \rho)$, on input a key $c k$, a value $v \in \mathcal{V}$ and randomness $\rho \in \mathcal{R}$, returns a commitment in $\mathcal{C}$.

C.ZCm $(c k, \rho)$, used when simulating proofs, is defined as C.Cm $\left(c k, 0_{\mathcal{V}}, \rho\right)$.

C.RdCm $(c k, c, \rho)$ randomizes a commitment $c$ to a fresh $c^{\prime}$ using randomness $\rho$.

C.Extr(xk, $c$ ), on input extraction key xk and a commitment $c$, outputs a value in $\mathcal{V}$. (This is the only algorithm that might not be polynomial-time.)

We extend C.Cm to vectors in $\mathcal{V}^{n}$ : for $M=\left(v_{1}, \ldots, v_{n}\right)$ and $\rho=\left(\rho_{1}, \ldots, \rho_{n}\right)$ we define C.Cm $(c k, M, \rho):=\left(\operatorname{C.Cm}\left(c k, v_{1}, \rho_{1}\right), \ldots, \operatorname{C.Cm}\left(c k, v_{n}, \rho_{n}\right)\right)$ and likewise $\operatorname{C.Extr}\left(\mathrm{xk},\left(c_{1}, \ldots, c_{n}\right)\right):=\left(\mathrm{C} . \operatorname{Extr}\left(\mathrm{xk}, c_{1}\right), \ldots, \mathrm{C} . \operatorname{Extr}\left(\mathrm{xk}, c_{n}\right)\right)$.

We now define a NIZK proof system that proves that committed values satisfy given equations from $\mathcal{E}$. Given a proof for commitments, the proof can be adapted to a randomization (via C.RdCm) of the commitments using C.AdptPrf.

C.Prv $\left(c k, E,\left(v_{1}, \rho_{1}\right), \ldots,\left(v_{n}, \rho_{n}\right)\right)$, on input a key $c k$, a set of equations $E \subset \mathcal{E}$, values $\left(v_{1}, \ldots, v_{n}\right)$ and randomness $\left(\rho_{1}, \ldots, \rho_{n}\right)$, outputs a proof $\pi$.

C.Verify $\left(c k, E, c_{1}, \ldots, c_{n}, \pi\right)$, on input a commitment key $c k$, a set of equations in $\mathcal{E}$, a commitment vector $\left(c_{1}, \ldots, c_{n}\right)$, and a proof $\pi$, outputs a bit $b$.

C.AdptPrf(ck, $\left.E, c_{1}, \rho_{1}, \ldots, c_{n}, \rho_{n}, \pi\right)$, on input a set of equations, commitments $\left(c_{1}, \ldots, c_{n}\right)$, randomness $\left(\rho_{1}, \ldots, \rho_{n}\right)$ and a proof $\pi$, outputs a proof $\pi^{\prime}$.

C.SmPrv $\left(t d, E, \rho_{1}, \ldots, \rho_{n}\right)$, on input the simulation trapdoor, a set of equations $E$ with $n$ variables and randomness $\left(\rho_{1}, \ldots, \rho_{n}\right)$, outputs a proof $\pi$.

$\mathcal{M}$-structure-preserving signatures. To prove knowledge of signatures, we require a scheme that is compatible with Groth-Sahai proofs $\left[\mathrm{AFG}^{+} 10\right]$.

S.Setup $(G r)$, on input the bilinear group description, outputs signature parameters $\operatorname{par}_{S}$, defining a message space $\mathcal{M}$. We require $\mathcal{M} \subseteq \mathcal{V}^{n}$ for some $n$.

S.KeyGen $\left(\operatorname{par}_{S}\right)$, on input the parameters $\operatorname{par}_{S}$, outputs a signing key and a verification key (sk, vk). We require that vk is composed of values in $\mathcal{V}$.

S.Sign(sk, $M$ ), on input a signing key sk and a message $M \in \mathcal{M}$, outputs a signature $\Sigma$. We require that $\Sigma$ is composed of values in $\mathcal{V}$.

S.Verify $(v k, M, \Sigma)$, on input a verification key $v k$, a message $M$ and a signature $\Sigma$, outputs a bit $b$. We require that S.Verify proceeds by evaluating equations from $\mathcal{E}$ (which we denote by $E_{\mathrm{S} . \text { Verify }(\cdot, \cdot, \cdot)}$ ).

$\mathcal{M}$-commuting signatures. As in a previous construction of transferable ecash $\left[\mathrm{BCF}^{+}\right.$11], we will use commuting signatures [Fuc11], which let the signer, given a commitment to a message, produce a commitment to a signature on that message, together with a proof, via the following functionality:

SigCm(ck, sk, $c)$, given a signing key sk and a commitment $c$ of a message $M \in$ $\mathcal{M}$, outputs a committed signature $c_{\Sigma}$ and a proof $\pi$ that the signature in $c_{\Sigma}$ is valid on the value in $c$, i.e., the committed values satisfy S.Verify $(v k, \cdot, \cdot)$.

$\operatorname{SmSigCm}(x k, v k, c, \Sigma)$, on input the extraction key $x k$, a verification key $v k$, a commitment $c$ and a signature $\Sigma$, outputs a committed signature $c_{\Sigma}$ and a proof $\pi$ of validity for $c_{\Sigma}$ and $c$ (the key xk is needed to compute $\pi$ for $c$ ).

Correctness and soundness properties. We require the following properties of commitments, proofs and signatures, when the setup algorithms are run on any output $G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right)$ for any $\lambda \in \mathbb{N}$ :

Perfectly binding commitments: C.Setup and the first output of C.ExSetup are distributed equivalently. Let $(c k, x k) \leftarrow C$.ExSetup; then for every $c \in \mathcal{C}$ there exists exactly one $v \in \mathcal{V}$ such that $c=\operatorname{C} . \mathrm{Cm}(c k, v, \rho)$ for some $\rho \in \mathcal{R}$. Moreover, C.Extr(xk, c) extracts that value $v$.

$\mathcal{V}^{\prime}$-extractability: We require that committed values from a subset $\mathcal{V}^{\prime} \subset \mathcal{V}$ can be efficiently extracted. Let $(c k, x k) \leftarrow$ C.ExSetup; then C.Extr $(x k, \cdot)$ is efficient on all values $c=\operatorname{C.Cm}(c k, v, \rho)$ for any $v \in \mathcal{V}^{\prime}$ and $\rho \in \mathcal{R}$

Proof completeness: Let $c k \leftarrow$ C.Setup; then for all $\left(v_{1}, \ldots, v_{n}\right) \in \mathcal{V}^{n}$ satisfying $E \subset \mathcal{E}$, and $\left(\rho_{1}, \ldots, \rho_{n}\right) \in \mathcal{R}^{n}$ and $\pi \leftarrow \operatorname{C.Prv}\left(c k, E,\left(v_{1}, \rho_{1}\right), \ldots,\left(v_{n}, \rho_{n}\right)\right)$ we have C.Verify $\left(c k, E, \operatorname{C.Cm}\left(c k, v_{1}, \rho_{1}\right), \ldots, \operatorname{C.Cm}\left(c k, v_{n}, \rho_{n}\right), \pi\right)=1$.

Proof soundness: Let $(c k, x k) \leftarrow$ C.ExSetup, $E \subset \mathcal{E}$, and $\left(c_{1}, \ldots, c_{n}\right) \in \mathcal{C}^{n}$. If C.Verify $\left(c k, E, c_{1}, \ldots, c_{n}, \pi\right)=1$ for some $\pi$, then letting $v_{i}:=\operatorname{C} . \operatorname{Extr}\left(\mathrm{xk}, c_{i}\right)$, for all $i$, we have that $\left(v_{1}, \ldots, v_{n}\right)$ satisfy $E$.

Randomizability: Let $c k \leftarrow$ C.Setup and $E \subset \mathcal{E}$; then for all $\left(v_{1}, \ldots, v_{n}\right) \in \mathcal{V}^{n}$ that satisfy $E$ and $\rho_{1}, \rho_{1}^{\prime}, \ldots, \rho_{n}, \rho_{n}^{\prime} \in \mathcal{R}$ the following two are distributed equivalently:

$$
\begin{aligned}
& \left(\operatorname{C.RdCm}\left(\operatorname{C.Cm}\left(c k, v_{1}, \rho_{1}\right), \rho_{1}^{\prime}\right), \ldots, \operatorname{C} . \operatorname{RdCm}\left(\operatorname{C.Cm}\left(c k, v_{n}, \rho_{n}\right), \rho_{n}^{\prime}\right),\right. \\
& \operatorname{C.AdptPrf}\left(c k, E, \operatorname{C.Cm}\left(c k, v_{1}, \rho_{1}\right), \rho_{1}^{\prime}, \ldots, \operatorname{C.Cm}\left(c k, v_{n}, \rho_{n}\right), \rho_{n}^{\prime}\right. \\
& \left.\left.\operatorname{C.Prv}\left(c k, E,\left(v_{1}, \rho_{1}\right), \ldots,\left(v_{n}, \rho_{n}\right)\right)\right)\right) \text { and } \\
& \left(\operatorname{C.Cm}\left(c k, v_{1}, \rho_{1}+\rho_{1}^{\prime}\right), \ldots, \operatorname{C.Cm}\left(c k, v_{n}, \rho_{n}+\rho_{n}^{\prime}\right)\right. \\
& \left.\operatorname{C.Prv}\left(c k, E,\left(v_{1}, \rho_{1}+\rho_{1}^{\prime}\right), \ldots,\left(v_{n}, \rho_{n}+\rho_{n}^{\prime}\right)\right)\right)
\end{aligned}
$$

Signature correctness: Let $(s k, v k) \leftarrow$ S.KeyGen(S.Setup) and $M \in \mathcal{M}$; then we have S.Verify $(v k, M$, S.Sign $(s k, M))=1$.

Correctness of signing committed messages: Let $(c k, x k) \leftarrow C$.ExSetup and let $(s k, v k) \leftarrow$ S.KeyGen(S.Setup), and $M \in \mathcal{M}$; if $\rho, \rho^{\prime} \leftarrow \mathcal{R}$, then the following three are distributed equivalently:

(C.Cm $\left.\left(c k, \operatorname{S.Sign}(s k, M), \rho^{\prime}\right), \operatorname{C.Prv}\left(c k, E_{\mathrm{S} . V e r i f y(v k, \cdot \cdot \cdot)},(M, \rho),\left(\Sigma, \rho^{\prime}\right)\right)\right)$ and

$\operatorname{SigCm}(c k, s k$, C.Cm $(c k, M, \rho))$ and

SmSigCm(xk, vk, C.Cm $(c k, M, \rho)$, S.Sign $(s k, M))$

The first equality also holds for $c k \leftarrow$ C.Setup, since it is distributed like $c k$ output by C.ExSetup.

\section{Security properties}

Mode indistinguishability: Let $G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right)$; then the outputs of C.Setup $(G r)$ and the first output of C.SmSetup $(G r)$ are computationally indistinguishable.

Perfect zero-knowledge in hiding mode: Let $(c k, t d) \leftarrow C . \operatorname{SmSetup}(G r), E \subset \mathcal{E}$ and $v_{1}, \ldots, v_{n} \in \mathcal{V}$ such that $E\left(v_{1}, \ldots, v_{n}\right)=1$. For $\rho_{1}, \ldots, \rho_{n} \leftarrow \mathcal{R}$ the following are distributed equivalently:

$$
\begin{aligned}
& \left.\operatorname{C.Cm}\left(c k, v_{1}, \rho_{1}\right), \ldots, \operatorname{C.Cm}\left(c k, v_{n}, \rho_{n}\right), \operatorname{C.Prv}\left(c k, E,\left(v_{1}, \rho_{1}\right), \ldots,\left(v_{n}, \rho_{n}\right)\right)\right) \\
& \quad \text { and }\left(\operatorname{C.ZCm}\left(c k, \rho_{1}\right), \ldots, \operatorname{C.ZCm}\left(c k, \rho_{n}\right), \operatorname{C.SmPrv}\left(t d, E, \rho_{1}, \ldots, \rho_{n}\right)\right)
\end{aligned}
$$

Signature unforgeability (under chosen message attack): No PPT adversary that is given vk output by S.KeyGen and an oracle for adaptive signing queries on messages $M_{1}, M_{2}, \ldots$ of its choice can output a pair $(M, \Sigma)$, such that S.Verify $(v k, M, \Sigma)=1$ and $M \notin\left\{M_{1}, M_{2}, \ldots\right\}$.

\subsection{Rerandomizable encryption schemes}

In order to trace double-spenders, some information must be retrievable from the coin by the bank. For anonymity, we encrypt this information. Since coins must change appearance in order to achieve coin transparency (Def. 4), we use rerandomizable encryption. In our e-cash scheme we will prove consistency of encrypted messages with values used elsewhere, and to produce such a proof, knowledge of parts of the randomness is required; we therefore make this an explicit input of some algorithms, which thus are still probabilistic.

A rerandomizable encryption scheme E consists of 4 poly.-time algorithms:

E.KeyGen $(G r)$, on input the group description, outputs an encryption key ek and a corresponding decryption key $d k$.

E.Enc $(e k, M, \nu)$ is probabilistic and on input an encryption key ek, a message $M$ and (partial) randomness $\nu$ outputs a ciphertext.

E.ReRand(ek, $\left.C, \nu^{\prime}\right)$, on input an encryption key, a ciphertext and (partial) randomness, outputs a new ciphertext. If no randomness is explicitly given to E.Enc or E.ReRand then it is assumed to be chosen uniformly.
$\operatorname{E} . \operatorname{Dec}(d k, C)$, on input a decryption key and a ciphertext, outputs either a message or $\perp$ indicating an error.

In order to prove statements about encrypted messages, we add two functionalities: E.Verify lets one check that a ciphertext encrypts a given message $M$, for which it is also given partial randomness $\nu$. This will allow us to prove that a commitment $c_{M}$ and a ciphertext $C$ contain the same message. For this, we require that the equations defining E.Verify are in the set $\mathcal{E}$ supported by C.Prv.

This lets us define an equality proof $\tilde{\pi}=\left(\pi, c_{\nu}\right)$, where $c_{\nu}$ is a commitment of the randomness $\nu$, and $\pi$ proves that the values in $c_{M}$ and $c_{\nu}$ verify the equations E.Verify $(\mathrm{ek}, \cdot, \cdot, C)$. To support rerandomization of ciphertexts, we define a functionality E.AdptPrf, which adapts a proof $\left(\pi, c_{\nu}\right)$ to a rerandomization.

E.Verify $(e k, M, \nu, C)$, on input an encryption key, a message, randomness and a ciphertext, outputs a bit.

E.AdptPrf( $\left.c k, e k, c_{M}, C, \tilde{\pi}=\left(\pi, c_{\nu}\right), \nu^{\prime}\right)$, a probabilistic algorithm which, on input a commitment key, an encryption key, a commitment, a ciphertext, an equality proof (i.e., a proof and a commitment) and randomness, outputs a new equality proof $\left(\pi^{\prime}, c_{\nu}^{\prime}\right)$.

Correctness properties. We require the scheme to satisfy the following correctness properties for all key pairs $(e k, d k) \leftarrow$ E.KeyGen $(G r)$ for $G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right)$ :
- For all $M \in \mathcal{M}$ and randomness $\nu$ we have: E.Enc $(e k, M, \nu)=C$ if and only if E.Verify $(e k, M, \nu, C)=1$.
- For all $M \in \mathcal{M}$ and $\nu$ : E.Verify $(e k, M, \nu, C)=1$ implies $\operatorname{E} . \operatorname{Dec}(d k, C)=M$. (These two notions imply the standard correctness notion.)
- For all $M \in \mathcal{M}$ and randomness $\nu, \nu^{\prime}$, if $C \leftarrow \operatorname{E} . \operatorname{Enc}(e k, M, \nu)$ then the following are equally distributed: E.ReRand(ek, $\left.C, \nu^{\prime}\right)$ and E.Enc $\left(e k, M, \nu+\nu^{\prime}\right)$.

- For all $c k \leftarrow$ C.Setup, all $(e k, d k) \leftarrow$ E.KeyGen, $M \in \mathcal{M}$ and randomness $\nu, \nu^{\prime}, \rho_{M}, \rho_{\nu}$, if we let

$$
\begin{aligned}
c_{M} & \leftarrow \operatorname{C.Cm}\left(c k, M, \rho_{M}\right) & C \leftarrow \operatorname{E} . \operatorname{Enc}(e k, M, \nu) \\
c_{\nu} & \leftarrow \operatorname{C.Cm}\left(c k, \nu, \rho_{\nu}\right) & \pi \leftarrow \operatorname{C.Prv}\left(c k, \operatorname{E} . \operatorname{Verify}(e k, \cdot, \cdot, C),\left(M, \rho_{M}\right),\left(\nu, \rho_{\nu}\right)\right)
\end{aligned}
$$

then the following are equivalently distributed (with $\rho_{\nu}^{\prime}$ is picked uniformly at random in $\mathcal{R}$ ):

$$
\begin{aligned}
& \text { E.AdptPrf(ck, ek, } \left.c_{M}, \operatorname{E} . \operatorname{Enc}(e k, C, \nu),\left(\pi, c_{\nu}\right), \nu^{\prime}\right) \text { and } \\
& \left(\operatorname{C.Prv}\left(c k, \operatorname{E} . \operatorname{Verify}\left(e k, \cdot, \cdot, \operatorname{ReRand}\left(e k, C, \nu^{\prime}\right)\right),\left(M, \rho_{M}\right),\left(\nu+\nu^{\prime}, \rho_{\nu}+\rho_{\nu}^{\prime}\right)\right)\right. \\
& \text { C.RdCm } \left.\left(c k, c_{\nu}, \rho_{\nu}^{\prime}\right)\right)
\end{aligned}
$$

Security properties. We require two properties from rerandomizable encryption: the first one is the standard (strongest possible) variant of CCA security; the second one is a new notion, which is easier to achieve.

```
-xpt {

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-18.jpg?height=46&width=694&top_left_y=436&top_left_x=539)
```
    (mo, m
    C\leftarrowE.Enc(ek, mb ) Return m
    b'}\leftarrow\mp@subsup{\mathcal{A}}{}{\operatorname{GDec}(\cdot)}(C)\quad\quad\mathrm{ Else return replay
    Return b'.

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-18.jpg?height=214&width=336&top_left_y=430&top_left_x=1296)

Fig. 6. Security games for rerandomizable encryption schemes

Replayable-CCA (RCCA) security. We use the definition from Canetti et al. [CKN03], formalized in Fig. 6.

Indistinguishability of adversarially chosen and randomized ciphertexts (IACR). An adversary that is given a public key, chooses two ciphertexts and is then given the randomization of one of them cannot, except with a negligible advantage, distinguish which one it was given. The game is formalized in Fig. 6.

Definition 5. For $\mathrm{x} \in\{\mathrm{RCCA}, \mathrm{IACR}\}$, a rerandomizable encryption scheme is x -secure if $\operatorname{Pr}\left[\operatorname{Expt}_{\mathcal{A}, 1}^{\mathrm{x}}(\lambda)=1\right]-\operatorname{Pr}\left[\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{x}}(\lambda)=1\right]$ is negligible in $\lambda$ for any $P P T \mathcal{A}$.

\subsection{Double-spending tag schemes}

Our e-cash scheme will follow earlier approaches [BCFK15], where the bank represents a coin in terms of its serial number $s n=s n_{0}\|\ldots\| s n_{k}$, which grows with every transfer. In addition, a coin contains a tag tag $=\operatorname{tag}_{1}\|\ldots\| \operatorname{tag}_{k}$, which enables tracing of double-spenders. The part $s n_{i}$ is chosen by a user when she receives the coin, while the tag $\operatorname{tag}_{i}$ is computed by the sender as a function of $s n_{i-1}, s n_{i}$ and her secret key.

Baldimtsi et al. [BCFK15] show how to construct such tags so they perfectly hide user identities, except when a user computes two tags with the same $s n_{i-1}$ but different values $S n_{i}$, in which case her identity can be computed from the two tags. Note that this precisely corresponds to double-spending the coin that ends in $s n_{i-1}$ to two users that choose different values for $s n_{i}$ when receiving it.

We use the tags from [BCFK15], which we first formally define, and then show that its full potential had not been leveraged yet: in particular, we realize that the tag can also be used as method for users to authenticate the coin transfer. In earlier works $\left[\mathrm{BCF}^{+} 11\right.$, BCFK15], at each transfer the spender computed a signature that was included in a coin, and that committed the user to the spending (and made her accountable in case of double-spending). Our construction does not require any user signatures and thus gains in efficiency.

Furthermore, in [BCFK15] (there were no tags in $\left[\mathrm{BCF}^{+} 11\right]$ ), the malleable signatures took care of ensuring well-formedness of the tags, while we give an explicit construction. To be compatible with Groth-Sahai proofs, we define structure-preserving proofs of well-formedness for serial numbers and tags.

Syntax. An $\mathcal{M}$-double-spending tag scheme T is composed of the following polynomial-time algorithms:

T.Setup(Gr), on input a group description, outputs the parameters $\operatorname{par}_{\mathrm{T}}$ (which are an implicit input to all of the following).

T.KeyGen(), on (implicit) input the parameters, outputs a tag key pair (sk, pk).

T.SGen(sk, $n)$, the serial-number generation function, on input a secret key and a nonce $n \in \mathcal{N}$ (the nonce space), outputs a serial-number component sn and a proof sn-pf of well-formedness.

$\operatorname{T.SGen}_{\text {init }}(s k, n)$, a variant of T.SGen, outputs a message $M \in \mathcal{M}$ instead of a

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-19.jpg?height=46&width=1150&top_left_y=823&top_left_x=520)
bank using a signature scheme that requires messages to be in $\mathcal{M}$.)

T.SVfy(pk, sn, sn-pf), on input a public key, a serial number and a proof verifies that sn is consistent with $p k$ by outputting a bit $b$.

T.SVfy $_{\text {init }}(p k, s n, M)$, on input a public key, a serial number and a message in $\mathcal{M}$, checks their consistency by outputting a bit $b$.

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-19.jpg?height=54&width=1103&top_left_y=1084&top_left_x=457)

T.TGen(sk, $n, s n)$, the double-spending tag function, takes as input a secret key, a nonce $n \in \mathcal{N}$ and a serial number, and outputs a double-spending tag tag $\in \mathcal{T}$ (the set of the double-spending tags) and a tag proof $t$-pf.

T.TVfy(pk, sn, sn' , tag, t-pf), on input a public key, two serial numbers, a doublespending tag, and a proof, checks consistency of the tag w.r.t. the key and the serial numbers by outputting a bit $b$.

T. Detect $\left(\mathrm{sn}, \mathrm{sn}^{\prime}, \operatorname{tag}, \operatorname{tag}^{\prime}, \mathcal{L}\right)$, double-spending detection, takes as input two serial numbers sn and $s n^{\prime}$, two tags tag, $\operatorname{tag}^{\prime} \in \mathcal{T}$ and a list of public keys $\mathcal{L}$ and outputs a public key $p k$ (of the accused user) and a proof $\Pi$.

T.VfyGuilt $(p k, \Pi)$, the incrimination-proof verification function, takes as input a public key and a proof and outputs a bit $b$.

Correctness properties. For any double-spending tag scheme T we require that for all $\operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r)$ the following hold:

Verifiability: For every $n, n^{\prime} \in \mathcal{N}$, and after computing

$-(s k, p k) \leftarrow$ T.KeyGen $;\left(s k^{\prime}, p k^{\prime}\right) \leftarrow$ T.KeyGen

$-(s n, X) \leftarrow$ T.SGen $(s k, n)$ or $(s n, X) \leftarrow$ T.SGen $_{\text {init }}(s k, n)$

$-\left(s n^{\prime}, s n-p f^{\prime}\right) \leftarrow \mathrm{T} . \mathrm{SGen}\left(s k^{\prime}, n^{\prime}\right)$

$-($ tag, $t-p f) \leftarrow$ T.TGen $\left(s k, n, s n^{\prime}\right)$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-19.jpg?height=49&width=971&top_left_y=2014&top_left_x=520)

$S N$-identifiability: For all tag public keys $p k_{1}$ and $p k_{2}$, all serial numbers sn and all $X_{1}$ and $X_{2}$, which can be messages in $\mathcal{M}$ or SN proofs, if

$$
\operatorname{T.SVfy}_{\text {all }}\left(p k_{1}, s n, X_{1}\right)=\operatorname{T.SVfy}_{\text {all }}\left(p k_{2}, s n, X_{2}\right)=1
$$

then $p k_{1}=p k_{2}$.

```
$\operatorname{Expt}_{\mathcal{A}, b}^{\mathrm{tag}-\text { anon }}(\lambda)$ :
    $G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right)$
    $\operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T}$.Setup(Gr)
    $O_{1}(s k):$
    $k:=0$
    $n \stackrel{\$}{\leftarrow} \mathcal{N} ; T[k]:=n ; k:=k+1$
    $(s n, s n-p f) \leftarrow$ T.SGen $(s k, n)$
    Return sn.
    $\left(s k_{0}, s k_{1}\right) \leftarrow \mathcal{A}\left(\operatorname{par}_{\mathrm{T}}\right)$
    $\mathrm{O}_{2}\left(s k, s n^{\prime}, i\right)$ :
    $b^{*} \leftarrow \mathcal{A}^{O_{1}\left(s k_{b}\right), O_{2}\left(s k_{b}, \cdot, \cdot\right)}\left(\operatorname{par}_{\mathrm{T}}, s k_{0}, s k_{1}\right)$
    If $T[i]=\perp$, abort the oracle call
    $n:=T[i] ; T[i]:=\perp$
    $($ tag, $t-p f) \leftarrow$ T.TGen $\left(s k, n, s n^{\prime}\right)$
    Return tag
```

Fig. 7. Game for tag anonymity (with oracles also used in exculpability) for doublespending tag schemes

Bootability: There do not exist an SN message $M$, serial numbers $s n_{1} \neq s n_{2}$ and tag keys (not necessarily distinct) $p k_{1}, p k_{2}$ such that:

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-20.jpg?height=59&width=815&top_left_y=1063&top_left_x=688)

2-show extractability: Let $p k_{0}, p k_{1}$ and $p k_{2}$ be tag public keys, $s n_{0}, s n_{1}$ and $s n_{2}$ be serial numbers, $X_{0}$ be either an SN proof or a message in $\mathcal{M}$, and $s n-p f_{1}$ and sn-pf $2_{2}$ be SN proofs. Let $\operatorname{tag}_{1}$ and $\operatorname{tag}_{2}$ be tags, and $t-p f_{1}$ and $t-p f_{2}$ be tag proofs, and let $\mathcal{L}$ be a set of tag public keys with $p k_{0} \in \mathcal{L}$. If

$$
\begin{array}{r}
\text { T.SVfy } \\
\text { T.Sll }\left(p k_{0}, s n_{0}, X_{0}\right)=1 \\
\text { T.TVfy }\left(p k_{1}, s n_{0}, s n_{1}, \operatorname{tag}_{1}, t-p f_{1}\right)=\text { T.TVfy }\left(p k_{2}, s n_{0}, s n_{2}, \operatorname{tag}_{2}, t-p f_{2}\right)=1
\end{array}
$$

and $s n_{1} \neq s n_{2}$ then T.Detect $\left(s n_{1}, s n_{2}, \operatorname{tag}_{1}, \operatorname{tag}_{2}, \mathcal{L}\right)$ extracts $\left(p k_{0}, \Pi\right)$ efficiently and we have T.VfyGuilt $\left(p k_{0}, \Pi\right)=1$.

$\mathcal{N}$-injectivity: For any secret key sk, the function T.SGen $(s k, \cdot)$ is injective.

\section{Security properties.}

Exculpability: This notion formalizes soundness of double-spending proofs, in that no honestly behaving user can be accused. Let $p a r_{\mathrm{T}} \leftarrow$ T.Setup and $(s k, p k) \leftarrow$ T.KeyGen $\left(p a r_{\mathrm{T}}\right)$. Then we require that for an adversary $\mathcal{A}$ that is given $p k$ and can obtain SNs and tags for receiver SNs of its choice, both produced with sk (but no two tags for the same sender SN), is computationally hard to return a proof $\Pi$ with T.VfyGuilt $(p k, \Pi)=1$. Formally, $\mathcal{A}$ gets access to oracles $O_{1}(s k)$ and $O_{2}(s k, \cdot, \cdot)$ defined in Fig. 7.

Tag anonymity: Finally, our anonymity notions for transferable e-cash should hold even against a malicious bank, which gets to see the serial numbers and double-spending tags for deposited coins, and the secret keys of the users. Thus, we require that as long as the nonce $n$ is random and only used once, serial numbers and tags reveal nothing about the user-specific values, such as sk and $p k$, that were used to generate them. The game is given in Fig. 7 .

Definition 6 (Tag anonymity). A double-spending tag scheme is anonymous if $\operatorname{Pr}\left[\operatorname{Expt}_{\mathcal{A}, 1}^{\mathrm{tag}-\text { anon }}(\lambda)=1\right]-\operatorname{Pr}\left[\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{tag}-\text { anon }}(\lambda)=1\right]$ is negligible in $\lambda$ for any $P P T \mathcal{A}$.

\section{Our transferable e-cash construction}

\subsection{Overview}

The bank validates new users in the system and creates money, and digital signatures can be used for both purposes: when a new user joins, the bank signs her public key, which serves as proof of being registered; during a coin issuing, the bank signs a message $M_{\mathrm{sn}}$ that is associated to the initial serial-number (SN) component $s n_{0}$ of a coin (chosen by the user withdrawing the coin), and this signature makes the coin unforgeable.

After a coin has been transferred $k$ times, its core consists of a list of SNs $s n_{0}, s n_{1}, \ldots, s n_{k}$, together with a list of tags $\operatorname{tag}_{1}, \ldots, \operatorname{tag}_{k}$ (for a freshly withdrawn coin, we have $k=0$ ). When a user spends such a coin, the receiver generates a fresh SN component $s n_{k+1}$, for which the spender must generate a tag $\operatorname{tag}_{k+1}$, which is also associated with her public key and the last serial number $\mathrm{Sn}_{k}$ (which she generated when she received the coin.)

These tags allow the bank to identify the cheater in case of double-spending, while they preserve honest users' anonymity, also towards the bank. A coin moreover contains the users' public key w.r.t. which the tags were created, as well as certificates from the bank on them. To provide anonymity, all these components are not given in the clear, but as a zero-knowledge proof of knowledge. As we use a commit-and-prove proof system, a coin contains commitments to its serial number, its tags, the user public keys and their certificates and proofs that ensure all of them are consistent.

Recall that a coin also includes a signature by the bank on (a message related to) the initial SN component. To achieve anonymity towards the bank (coin anonymity), the bank must sign this message blindly, which is achieved by using the SigCm functionality: the user sends a commitment to the serial number, and the bank computes a committed signature on the committed value.

Finally, the bank needs to be able to detect whether a double-spending occurred and identify the user that committed it. One way would be to give the serial numbers and the tags (which protect the anonymity of honest users) in the clear. This would yield a scheme that satisfies coin anonymity and user anonymity (note that in these two notions the bank is adversarially controlled). In contrast, coin transparency, the most intricate anonymity notion, would not be achieved, since the owner of a coin could easily recognize it when she receives it again by looking at its serial number.

Coin transparency requires to hide the serial numbers (and the associated tags), and to use a randomizable proof system, since the appearance of a coin needs to change after every transfer. At the same time we need to provide the bank access to them; we thus include encryptions, under the bank's public key, in the coin. And we add proofs of consistency of the encrypted values. Now
all of this must interoperate with the randomization of the coin, which is why we require rerandomizable encryption. Moreover, this has to be tied into the machinery of updating the proofs, which is necessary every time the ciphertexts and the commitments contained in a coin are refreshed.

\subsection{Technical description}

Primitives used. The basis of our transferable e-cash scheme is a randomizable extractable NIZK commit-and-prove scheme C to which we add compatible schemes: an $\mathcal{M}$-structure-preserving signature scheme $S$ that admits an $\mathcal{M}$-commuting signature add-on SigCm, as well as a (standard) $\mathcal{M}^{\prime}$-structurepreserving signature scheme $\mathrm{S}^{\prime}$ (all defined in Sect. 4.2).

Our scheme moreover uses rerandomizable encryption (Sect. 4.3), a scheme E, which only needs to be IACR-secure, and an RCCA-secure scheme $E^{\prime}$, which will only be used for a single ciphertext per coin. (One can instantiate $E$ with a possibly more efficient scheme.) Finally, we use a double-spending tag scheme T (Sect. 4.4). We require $\mathrm{E}, \mathrm{E}^{\prime}$ and T to be compatible with the proof system C ,

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-22.jpg?height=43&width=1211&top_left_y=1125&top_left_x=457)
$\mathrm{E}^{\prime}$.Verify, are all in the set $\mathcal{E}$ of equations supported by C .

Auxiliary functions. To simplify the description of our scheme, we first define several auxiliary functions. We let Rand denote an algorithm that randomizes a given tuple of commitments and ciphertext, as well as proofs for them (and adapts the proofs to the randomizations) by internally running C.RdCm, E.ReRand, C.AdptPrf and E.AdptPrf with the same randomness.

Below, we define C.Prv ${ }_{\mathrm{sn}, \text { init }}$ that produces a proof that a committed initial serial number sn was correctly generated w.r.t. a committed key $p k_{\mathrm{T}}$ and a committed message $M$ (given the used randomness $\rho_{p k}, \rho_{s n}$ and $\rho_{M}$ ); and

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-22.jpg?height=43&width=1211&top_left_y=1583&top_left_x=457)
non-initial serial numbers (for which there are no messages, but which require a proof of well-formedness instead).

$$
\begin{aligned}
& \operatorname{C.Prv}_{\mathrm{sn}, \text { init }}\left(c k, p k_{\mathrm{T}}, s n, M, \rho_{p k}, \rho_{\mathrm{sn}}, \rho_{M}\right) \text { : } \\
& - \text { Return } \pi \leftarrow \operatorname{C.Prv}\left(c k, \text { T.SVfy }_{\text {init }}(\cdot, \cdot, \cdot)=1,\left(p k_{\mathrm{T}}, \rho_{p k}\right),\left(s n, \rho_{\text {sn }}\right),\left(M, \rho_{M}\right)\right)
\end{aligned}
$$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-22.jpg?height=54&width=513&top_left_y=1827&top_left_x=524)

$$
\begin{aligned}
& \text { - Return (C.Verify } \left.\left(c k, \operatorname{T.SVfy}_{\text {init }}(\cdot, \cdot, \cdot)=1, c_{p k}, c_{\mathrm{sn}}, c_{M}, \pi_{s n}\right)\right) \\
& \operatorname{C.Prv}_{\mathrm{sn}}\left(c k, p k_{\mathrm{T}}, s n, s n-p f, \rho_{p k}, \rho_{\mathrm{sn}}, \rho_{\mathrm{sn}-p f}\right) \\
& -\pi \leftarrow \operatorname{C.Prv}\left(c k, \operatorname{T} . \operatorname{SVfy}(\cdot, \cdot, \cdot)=1,\left(p k_{\mathrm{T}}, \rho_{p k}\right),\left(s n, \rho_{\mathrm{sn}}\right),\left(s n-p f, \rho_{\mathrm{sn}-p f}\right)\right) \\
& -\operatorname{Return}\left(\pi, \mathrm{C} . \mathrm{Cm}\left(c k, s n-p f, \rho_{\text {sn-pf }}\right)\right) \\
& \text { C.Verify }{ }_{\mathrm{sn}}\left(c k, c_{p k}, c_{\mathrm{sn}}, \tilde{\pi}_{s n}=\left(\pi_{s n}, c_{\mathrm{sn}-p f}\right)\right) \\
& \text { - Return C.Verify(ck, T.SVfy } \left.(\cdot, \cdot, \cdot)=1, c_{p k}, c_{s n}, c_{s n-p f}, \pi_{s n}\right)
\end{aligned}
$$

C.Prv ${ }_{\text {tag }}$ produces a proof that a committed tag was correctly generated w.r.t.

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-22.jpg?height=48&width=1125&top_left_y=2269&top_left_x=457)

$$
\begin{aligned}
& \operatorname{C.Prv}_{\text {tag }}\left(c k, p k_{\mathrm{T}}, s n, s n^{\prime}, \operatorname{tag}, \rho_{p k}, \rho_{\mathrm{sn}}, \rho_{\mathrm{sn}}^{\prime}, \rho_{\text {tag }}, t-p f, \rho_{t-p f}\right) \\
& -\pi \leftarrow \operatorname{C.Prv}\left(c k, \mathrm{~T} . \mathrm{TVfy}(\cdot, \cdot, \cdot \cdot \cdot, \cdot)=1,\left(p k_{\mathrm{T}}, \rho_{p k}\right),\left(s n, \rho_{\mathrm{sn}}\right),\left(s n^{\prime}, \rho_{\mathrm{sn}}^{\prime}\right)\right. \\
& \left.\left(\text { tag }, \rho_{\text {tag }}\right),\left(t-p f, \rho_{t-p f}\right)\right)
\end{aligned}
$$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-23.jpg?height=49&width=1212&top_left_y=694&top_left_x=458)

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-23.jpg?height=46&width=1211&top_left_y=736&top_left_x=457)

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-23.jpg?height=46&width=1211&top_left_y=779&top_left_x=457)
not used outside of C.E.Prvenc , it can be sampled locally.)

$$
\begin{aligned}
& \text { C.E.Prv } \mathrm{enc}\left(c k, e k, M, \rho_{M}, \nu_{M}, \tilde{c}\right): \\
& -\rho_{\nu} \stackrel{\Phi}{\leftarrow} \mathcal{R} ; \pi \leftarrow \operatorname{C.Prv}\left(c k, \text { E.Verify }(e k, \cdot, \cdot, \tilde{c})=1,\left(M, \rho_{M}\right),\left(\nu_{M}, \rho_{\nu}\right)\right) \\
& \text { - Return }\left(\pi, \operatorname{C.Cm}\left(c k, \nu_{M}, \rho_{\nu}\right)\right) \\
& \text { C.E.Verify }{ }_{\mathrm{enc}}\left(c k, e k, c_{M}, \tilde{c}_{M}, \tilde{\pi}_{\mathrm{eq}}=\left(\pi_{\mathrm{eq}}, c_{\nu}\right)\right) \text { : } \\
& \text { - Return C.Verify(ck, E.Verify } \left.\left(\mathrm{ek}, \cdot, \cdot, \tilde{c}_{M}\right)=1, c_{M}, c_{\nu}, \pi_{\mathrm{eq}}\right)
\end{aligned}
$$

Components of the coin. There are two types of components, the initial

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-23.jpg?height=44&width=1211&top_left_y=1214&top_left_x=457)
form

$$
\begin{equation*}
\operatorname{coin}_{\text {init }}=\left(c_{p k}^{0}, c_{\text {cert }}^{0}, \pi_{c e r t}^{0}, c_{\mathrm{sn}}^{0}, \pi_{\mathrm{sn}}^{0}, \varepsilon, \varepsilon, c_{M}, c_{\sigma}^{0}, \pi_{\sigma}^{0}, \tilde{c}_{\mathrm{sn}}^{0}, \tilde{\pi}_{\mathrm{sn}}^{0}, \varepsilon, \varepsilon\right) \tag{1}
\end{equation*}
$$

where the " $c$-values" are commitments to the withdrawer's key $p k$, her certificate cert, the initial serial number sn and the related message $M$, the bank's signature $\sigma$ on $M$; and $\tilde{c}_{\text {sn }}$ is an encryption of $s n$. Moreover, $\pi_{c e r t}$ and $\pi_{s n}$ prove validity of cert and sn, and $\tilde{\pi}_{\mathrm{sn}}$ proves that $c_{s n}$ and $\tilde{c}_{\mathrm{sn}}$ contain the same value. We use "empty values" $\varepsilon$ to pad so that both coin-component types have the same format. Validity of an initial component is verified w.r.t. an encryption key for $\mathrm{E}^{\prime}$ and two signature verification keys for S and $\mathrm{S}^{\prime}$ :

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-23.jpg?height=57&width=1200&top_left_y=1668&top_left_x=468)
- C.Verify $\left(c k, S^{\prime}\right.$.Verify $\left.\left(v k^{\prime}, \cdot, \cdot\right)=1, c_{p k}^{0}, c_{\text {cert }}^{0}, \pi_{\text {cert }}^{0}\right)$
- C.Verify $\left(c k, \operatorname{S} . \operatorname{Verify}(v k, \cdot, \cdot)=1, c_{M}, c_{\sigma}^{0}, \pi_{\sigma}^{0}\right)$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-23.jpg?height=60&width=1085&top_left_y=1824&top_left_x=539)

Standard components of a coin are of the form

$$
\begin{equation*}
\operatorname{coin}_{\mathrm{std}}=\left(c_{p k}^{i}, c_{\text {cert }}^{i}, \pi_{\text {cert }}^{i}, c_{\mathrm{sn}}^{i}, \pi_{\mathrm{sn}}^{i}, c_{\text {tag }}^{i}, \pi_{\text {tag }}^{i}, \varepsilon, \varepsilon, \varepsilon, \tilde{c}_{\mathrm{sn}}^{i}, \tilde{\pi}_{\mathrm{sn}}^{i}, \tilde{c}_{\text {tag }}^{i}, \tilde{\pi}_{\text {tag }}^{i}\right) \tag{2}
\end{equation*}
$$

and instead of $M$ and the bank's signature they contain a commitment $c_{\text {tag }}$ and an encryption $\tilde{c}_{\text {tag }}$ of the tag produced by the spender (and a proof $\pi_{\text {tag }}$ of validity and $\tilde{\pi}_{\text {tag }}$ proving that the values in $c_{\text {tag }}$ and $\tilde{c}_{\text {tag }}$ are equal). A coin is verified by checking the validity and consistency of each two consecutive components. If the

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-23.jpg?height=59&width=1211&top_left_y=2212&top_left_x=457)
is a standard component then $c_{M}, c_{\sigma}^{i-1}$ and $\overline{\pi_{\sigma}^{i-1}} \overline{\text { are } \varepsilon}$.

$$
\begin{aligned}
& \operatorname{VER}_{\mathrm{std}}\left(e k, v k^{\prime},\left(c_{p k}^{i-1}, c_{\text {cert }}^{i-1}, \pi_{\text {cert }}^{i-1}, c_{\mathrm{sn}}^{i-1}, \pi_{\mathrm{sn}}^{i-1}, c_{\text {tag }}^{i-1}, \pi_{\text {tag }}^{i-1}, c_{M}, c_{\sigma}^{i-1}, \pi_{\sigma}^{i-1}, \tilde{c}_{\mathrm{sn}}^{i-1}\right.\right. \\
& \left.\left.\tilde{\pi}_{\mathrm{sn}}^{i-1}, \tilde{c}_{\text {tag }}^{i-1}, \tilde{\pi}_{\text {tag }}^{i-1}\right), \text { coin }_{\text {std }}\right): / / \operatorname{coin}_{\mathrm{std}} \text { as in }(2)
\end{aligned}
$$

Return 1 iff the following hold:
- heoneinC.Verify $\left(c k, \mathrm{~S}^{\prime}\right.$.Verify $\left.\left(v k^{\prime}, \cdot, \cdot\right)=1, c_{p k}^{i}, c_{\text {cert }}^{i}, \pi_{\text {cert }}^{i}\right)$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-24.jpg?height=52&width=1046&top_left_y=608&top_left_x=542)

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-24.jpg?height=56&width=1084&top_left_y=655&top_left_x=542)

Our scheme. We now formally define our transferable e-cash scheme.

```
ParamGen $\left(1^{\lambda}\right):$
    $-G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right)$
    $-\operatorname{par}_{\mathrm{S}} \leftarrow \mathrm{S} . \operatorname{Setup}(G r) ; \operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r)$
    $-\operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r) ; c k \leftarrow \operatorname{C.Setup}(G r)$
    - Return par $=\left(1^{\lambda}, G r\right.$, par $\left._{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$
```

Recall that par is an implicit input to all other algorithms; we assume that they parse par as $\left(1^{\lambda}, G r, p a r_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$.

BKeyGen():

$-(s k, v k) \leftarrow \operatorname{S} . \operatorname{KeyGen}\left(p a r_{\mathrm{S}}\right) ;\left(s k^{\prime}, v k^{\prime}\right) \leftarrow \mathrm{S}^{\prime} . \operatorname{KeyGen}\left(p a \mathrm{~S}_{\mathrm{S}^{\prime}}\right)$

$-\left(e k^{\prime}, d k^{\prime}\right) \leftarrow \mathrm{E}^{\prime} . \operatorname{KeyGen}(G r) ;(e k, d k) \leftarrow \operatorname{E.KeyGen}(G r)$

$-\left(s k_{\mathrm{T}}, p k_{\mathrm{T}}\right) \leftarrow \mathrm{T} . \operatorname{KeyGen}\left(p a r_{\mathrm{T}}\right) \quad / /\left(s k_{\mathrm{T}}, p k_{\mathrm{T}}\right.$, cert) let the bank act...
- cert $\leftarrow \mathrm{S}^{\prime} . \operatorname{Sign}\left(s k^{\prime}, p k_{\mathrm{T}}\right) \quad / / \ldots$ as $\mathcal{U}^{\prime}$ in Spend during deposit
- Return $\left(s k_{\mathcal{W}}=\left(s k, s k^{\prime}\right), s k_{\mathcal{C K}}=\left(d k^{\prime}, d k\right)\right.$,

$$
\left.s k_{\mathcal{D}}=\left(c e r t, p k_{\mathrm{T}}, s k_{\mathrm{T}}\right), p k_{\mathcal{B}}=\left(e k^{\prime}, e k, v k, v k^{\prime}\right)\right)
$$

$\underline{\text { Register }}\left\langle\mathcal{B}\left(s k_{\mathcal{W}}=\left(s k, s k^{\prime}\right)\right), \mathcal{U}\left(p k_{\mathcal{B}}=\left(e k^{\prime}, e k, v k, v k^{\prime}\right)\right)\right\rangle:$

$\mathcal{U}:\left(s k_{\mathrm{T}}, p k_{\mathrm{T}}\right) \leftarrow$ T.KeyGen $\left(p a r_{T}\right)$; send $p k_{\mathrm{T}}$ to $\mathcal{B}$

$\mathcal{B}:$ cert $\mathcal{U}^{\leftarrow} \leftarrow \mathrm{S}^{\prime} . \operatorname{Sign}\left(s k^{\prime}, p k_{\mathrm{T}}\right)$; send cert $t_{\mathcal{U}}$ to $\mathcal{U}$; output $p k_{\mathrm{T}}$

$\mathcal{U}$ : If $S^{\prime} . V e r i f y\left(v k^{\prime}, p k_{\mathrm{T}}, c_{2} t_{\mathcal{U}}\right)=1$, output $s k_{\mathcal{U}} \leftarrow\left(c e r t_{\mathcal{U}}, p k_{\mathrm{T}}, s k_{\mathrm{T}}\right)$; else $\perp$

```
$\underline{\text { Withdraw }}\left\langle\mathcal{B}\left(s k_{\mathcal{W}}=\left(s k, s k^{\prime}\right), p k_{\mathcal{B}}=\left(e k^{\prime}, e k, v k, v k^{\prime}\right)\right)\right.$,
$\mathcal{U}:-n \stackrel{\&}{\leftarrow} \mathcal{N} ; \rho_{p k}, \rho_{\text {cert }}, \rho_{\text {sn }}, \rho_{M} \stackrel{\&}{\leftarrow}$
                        $\left.\mathcal{U}\left(s k_{\mathcal{U}}=\left(c e r t_{\mathcal{U}}, p k_{\mathrm{T}}, s k_{\mathrm{T}}\right), p k_{\mathcal{B}}\right)\right\rangle:$
    $-\left(s n, M_{\text {sn }}\right) \leftarrow$ T.SGen $_{\text {init }}\left(s k_{\mathrm{T}}, n\right)$
    $-c_{p k} \leftarrow \mathrm{C} . \mathrm{Cm}\left(c k, p k_{\mathrm{T}}, \rho_{p k}\right)$
    $-c_{\text {cert }} \leftarrow \operatorname{C.Cm}\left(c k\right.$, cert $\left.\mathcal{U}_{\mathcal{U}}, \rho_{\text {cert }}\right)$
    $-c_{\mathrm{sn}} \leftarrow \mathrm{C} . \mathrm{Cm}\left(c k, \mathrm{sn}, \rho_{\mathrm{sn}}\right)$
    $-c_{M} \leftarrow \operatorname{C.Cm}\left(c k, M_{s n}, \rho_{M}\right)$
    $-\pi_{\text {cert }} \leftarrow \operatorname{C.Prv}\left(c k, \mathrm{~S}^{\prime} . \operatorname{Verify}\left(v k^{\prime}, \cdot, \cdot\right)=1,\left(p k_{\mathrm{T}}, \rho_{p k}\right),\left(c e r t \mathcal{U}, \rho_{\text {cert }}\right)\right)$
    $-\pi_{\mathrm{sn}} \leftarrow \operatorname{C.Prv}_{\mathrm{sn}, \mathrm{init}}\left(c k, p k_{\mathrm{T}}, \mathrm{sn}, M_{\mathrm{sn}}, \rho_{p k}, \rho_{\mathrm{sn}}, \rho_{M}\right)$
    - Send $\left(c_{p k}, c_{\text {cert }}, \pi_{c e r t}, c_{s n}, c_{M}, \pi_{\text {sn }}\right)$ to $\mathcal{B}$
```
$\mathcal{B}:$ - if C.Verify $\left(c k, \mathrm{~S}^{\prime}\right.$. Verify $\left.\left(v k^{\prime}, \cdot, \cdot\right)=1, c_{p k}, c_{\text {cert }}, \pi_{\text {cert }}\right)$ or

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-25.jpg?height=49&width=963&top_left_y=450&top_left_x=646)
$-\left(c_{\sigma}, \pi_{\sigma}\right) \leftarrow \operatorname{SigCm}\left(c k, s k, c_{M}\right)$; send $\left(c_{\sigma}, \pi_{\sigma}\right)$ to $\mathcal{U}^{\prime}$; return ok

$\mathcal{U}$ : - if C.Verify(ck, S.Verify(vk, $\left.\cdot, \cdot)=1, c_{M}, c_{\sigma}, \pi_{\sigma}\right)$ fails, abort and output $\perp$.

$-\nu_{s n} \stackrel{\&}{\leftarrow} ; \tilde{c}_{s n} \leftarrow \mathrm{E}^{\prime} . \operatorname{Enc}\left(e k^{\prime}, s n, \nu_{s n}\right)$

$-\tilde{\pi}_{\mathrm{sn}} \leftarrow \mathrm{C}^{\prime} . \mathrm{E}^{\prime} \operatorname{Prv}_{\mathrm{enc}}\left(c k, e k^{\prime}, \mathrm{sn}, \rho_{\mathrm{sn}}, \nu_{\mathrm{sn}}, \tilde{c}_{\mathrm{sn}}\right)$

$-\rho_{p k}^{\prime}, \rho_{\text {cert }}^{\prime}, \rho_{\mathrm{sn}}^{\prime}, \rho_{M}^{\prime}, \rho_{\sigma}^{\prime}, \nu_{\text {sn }}^{\prime}, \rho_{\tilde{\pi}, s n}^{\prime} \stackrel{\$}{\leftarrow} \quad / /$ since $\tilde{\pi}_{\text {sn }}$ contains a commitment, we also sample randomness for it

$-c^{0} \leftarrow \operatorname{Rand}\left(\left(c_{p k}, c_{c e r t}, \pi_{c e r t}, c_{s n}, \pi_{s n}, c_{M}, c_{\sigma}, \pi_{\sigma}, \tilde{c}_{\mathrm{sn}}, \tilde{\pi}_{\mathrm{sn}}\right)\right.$,

$\left.\left(\rho_{p k}^{\prime}, \rho_{c e r t}^{\prime}, \rho_{s n}^{\prime}, \rho_{M}^{\prime}, \rho_{\sigma}^{\prime}, \nu_{s n}^{\prime}, \rho_{\tilde{\pi}, s n}^{\prime}\right)\right)$
- Output $\left(c^{0}, n, s n, \rho_{\mathrm{sn}}+\rho_{\mathrm{sn}}^{\prime}, \rho_{p k}+\rho_{p k}^{\prime}\right)$

$\underline{\text { Spend }}\left\langle\mathcal{U}\left(c, s k_{\mathcal{U}}=\left(c e r t, p k_{\mathrm{T}}, s k_{\mathrm{T}}\right), p k_{\mathcal{B}}=\left(e k^{\prime}, e k, v k, v k^{\prime}\right)\right)\right.$, $\left.\mathcal{U}^{\prime}\left(s k_{\mathcal{U}}^{\prime}=\left(c e r t^{\prime}, p k_{\mathrm{T}}^{\prime}, s k_{\mathrm{T}}^{\prime}\right), p k_{\mathcal{B}}\right)\right\rangle:$

$\mathcal{U}^{\prime}:-n^{\prime} \stackrel{\otimes}{\leftarrow} \mathcal{N} ; \rho_{p k}^{\prime}, \rho_{\text {cert }}^{\prime}, \rho_{\mathrm{sn}}^{\prime}, \rho_{\mathrm{sn}-p f}^{\prime}, \nu_{\mathrm{sn}}^{\prime} \stackrel{\&}{\leftarrow} \mathcal{R}$

$-\left(s n^{\prime}, s n-p f^{\prime}\right) \leftarrow \mathrm{T} . \mathrm{SGen}\left(p a r_{\mathrm{T}}, s k_{\text {tag }}^{\prime}, n^{\prime}\right)$

$-c_{p k}^{\prime} \leftarrow \mathrm{C.Cm}\left(c k, p k_{\mathrm{T}}^{\prime}, \rho_{p k}^{\prime}\right) ; c_{\text {cert }}^{\prime} \leftarrow \mathrm{C} . \mathrm{Cm}\left(c k, c e r t^{\prime}, \rho_{\text {cert }}^{\prime}\right)$

$-c_{\mathrm{sn}}^{\prime} \leftarrow \mathrm{C} . \mathrm{Cm}\left(c k, \mathrm{sn}^{\prime}, \rho_{\mathrm{sn}}^{\prime}\right) ; c_{\mathrm{sn}-p f}^{\prime} \leftarrow \mathrm{C.Cm}\left(c k, s n-p f^{\prime}, \rho_{\mathrm{sn}-p f}^{\prime}\right)$

$-\tilde{c}_{\mathrm{sn}}^{\prime} \leftarrow \mathrm{E} . \operatorname{Enc}\left(e k, s n^{\prime}, \nu_{s n}^{\prime}\right)$

$-\pi_{\text {cert }}^{\prime} \leftarrow \operatorname{C.Prv}\left(c k, \operatorname{S} . \operatorname{Verify}\left(v k^{\prime}, \cdot, \cdot\right)=1,\left(p k_{\mathrm{T}}^{\prime}, \rho_{p k}^{\prime}\right),\left(c e r t^{\prime}, \rho_{c e r t}^{\prime}\right)\right)$

$-\pi_{s n}^{\prime} \leftarrow \operatorname{C.Prv}_{\mathrm{sn}}\left(c k, p k_{\mathrm{T}}^{\prime}, s n^{\prime}, s n-p f, \rho_{p k}^{\prime}, \rho_{\mathrm{sn}}^{\prime}, \rho_{\mathrm{sn}-p f}^{\prime}\right)$

$-\tilde{\pi}_{\mathrm{sn}}^{\prime} \leftarrow$ C.E.Prvenc $\left(c k, e k, s n^{\prime}, \rho_{\mathrm{sn}}^{\prime}, \nu_{\mathrm{sn}}^{\prime}, \tilde{c}_{\mathrm{sn}}^{\prime}\right)$
- Send $\left(s n^{\prime}, \rho_{s n}^{\prime}\right)$ to $\mathcal{U}$

$\mathcal{U}:-$ Parse $c$ as $\left(c^{0},\left(c^{j}=\left(c_{p k}^{j}, c_{\text {cert }}^{j}, \pi_{\text {cert }}^{j}, c_{\mathrm{sn}}^{j}, \pi_{\mathrm{sn}}^{j}, c_{\text {tag }}^{j}, \pi_{\text {tag }}^{j}\right.\right.\right.$,

$-\rho_{\text {tag }}, \nu_{\text {tag }}, \rho_{t-p f} \stackrel{\&}{\leftarrow}$

$\left.\left.\tilde{c}_{\mathrm{sn}}^{j}, \tilde{c}_{\text {tag }}^{j}, \tilde{\pi}_{\mathrm{sn}}^{j}, \tilde{\pi}_{\text {tag }}^{j}\right)\right)_{j=1}^{i}, n$, sn, $\left.\rho_{\mathrm{sn}}, \rho_{p k}\right) \quad / / i$ could be 0

$-($ tag, t-pf $) \leftarrow$ T.TGen $\left(p \mathrm{Tr}_{\mathrm{T}}, s k_{\mathrm{T}}, n, s n^{\prime}\right)$

$-c_{\text {tag }} \leftarrow$ C.Cm(ck, tag, $\left.\rho_{\text {tag }}\right) ; \tilde{c}_{\text {tag }} \leftarrow$ E.Enc(ek, tag,$\left.\nu_{\text {tag }}\right)$

$-\pi_{\text {tag }} \leftarrow \operatorname{C.Prv}_{\text {tag }}\left(c k, p k_{\mathrm{T}}, s n, s n^{\prime}\right.$, tag, $\left.t-p f, \rho_{p k}, \rho_{\mathrm{sn}}, \rho_{\mathrm{sn}}^{\prime}, \rho_{\text {tag }}, \rho_{t-p f}\right)$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-25.jpg?height=52&width=689&top_left_y=1801&top_left_x=542)
- Send $c^{\prime}=\left(c^{0},\left(c^{j}\right)_{j=1}^{i}, c_{\text {tag }}, \pi_{\text {tag }}, \tilde{c}_{\text {tag }}, \tilde{\pi}_{\text {tag }}\right)$ to $\mathcal{U}^{\prime}$; output ok

$\mathcal{U}^{\prime}$ : - If any of the following fail then abort and output $\perp$ :
- $\operatorname{VER}_{\text {init }}\left(e k^{\prime}, v k, v k^{\prime}, c^{0}\right)$
- $\operatorname{VER}_{\text {std }}\left(e k, v k, v k^{\prime}, c^{j-1}, c^{j}\right)$, for $j=1, \ldots, i$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-25.jpg?height=57&width=566&top_left_y=2061&top_left_x=606)

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-25.jpg?height=46&width=545&top_left_y=2118&top_left_x=606)
- pick uniform random $\overrightarrow{\rho^{\prime \prime}}$

$-c^{\prime \prime} \leftarrow \operatorname{Rand}\left(\left(\left(c^{j}\right)_{j=0}^{i}, c_{p k}^{\prime}, c_{\text {cert }}^{\prime}, \pi_{c e r t}^{\prime}, c_{\mathrm{sn}}^{\prime}, \pi_{\mathrm{sn}}^{\prime}, c_{\text {tag }}, \pi_{\text {tag }}, \tilde{c}_{\mathrm{sn}}^{\prime}, \tilde{\pi}_{\mathrm{sn}}^{\prime}, \tilde{c}_{\text {tag }}^{\prime}, \tilde{\pi}_{\text {tag }}^{\prime}\right), \overrightarrow{\rho^{\prime \prime}}\right)$
- Output $\left(c^{\prime \prime}, n^{\prime}, s n^{\prime}, \rho_{s n}^{\prime}+\left(\overrightarrow{\rho^{\prime \prime}}\right)_{s n^{\prime}}, \rho_{p k}^{\prime}+\left(\overrightarrow{\rho^{\prime \prime}}\right)_{p k^{\prime}}\right)$

```
CheckDS $\left(s k_{\mathcal{C K}}=\left(d k^{\prime}, d k\right), \mathcal{D C L}, \mathcal{U L}, c\right)$
    - Parse $c$ as $\left(c^{0}=\left(c_{p k}^{0}, c_{\text {cert }}^{0}, \pi_{\text {cert }}^{0}, c_{\mathrm{sn}}^{0}, \pi_{\mathrm{sn}}^{0}, c_{M}^{0}, c_{\sigma}, \pi_{\sigma}, \tilde{c}_{\mathrm{sn}}^{0}, \tilde{\pi}_{\mathrm{sn}}^{0}\right)\right.$,
        $\left.\left(c^{j}=\left(c_{p k}^{j}, c_{c e r t}^{j}, \pi_{\text {cert }}^{j}, c_{s n}^{j}, \pi_{s n}^{j}, c_{\text {tag }}^{j}, \pi_{t a g}^{j}, \tilde{c}_{s n}^{j}, \tilde{\pi}_{\mathrm{sn}}^{j}, \tilde{c}_{\text {tag }}^{j}, \tilde{\pi}_{\text {tag }}^{j}\right)\right)_{j=1}^{i}, n, s n, \rho_{\mathrm{sn}}, \rho_{p k}\right)$
    $-\overrightarrow{\operatorname{sn}} \leftarrow\left(\mathrm{E}^{\prime} . \operatorname{Dec}\left(d k^{\prime}, \tilde{c}_{\mathrm{sn}}^{0}\right), \operatorname{E} . \operatorname{Dec}\left(d k, \tilde{c}_{\mathrm{sn}}^{1}\right), \ldots, \operatorname{E} . \operatorname{Dec}\left(d k, \tilde{c}_{\mathrm{sn}}^{i}\right)\right)$
    $-\operatorname{tag} \leftarrow\left(\operatorname{E} \cdot \operatorname{Dec}\left(d k, \tilde{c}_{\text {tag }}^{1}\right), \ldots, \operatorname{E} \cdot \operatorname{Dec}\left(d k, \tilde{c}_{\text {tag }}^{i}\right)\right)$
    - If for all $\left(\overrightarrow{s^{\prime}}, \overrightarrow{\operatorname{tag}}^{\prime}\right) \in \mathcal{D C L}:(\overrightarrow{\operatorname{sn}})_{0} \neq\left(\overrightarrow{s n}^{\prime}\right)_{0} \quad / /$ initial SN of checked coin...
        then return $\mathcal{D C L} \|(\overrightarrow{\operatorname{sn}}, \mathrm{tag}) \quad / / \ldots$ different from those of deposited coins
    - Else let $j$ be minimal so that $(\overrightarrow{\mathrm{sn}})_{j} \neq\left(\overrightarrow{\mathrm{sn}^{\prime}}\right)_{j} \quad / /$ double-spent at $j$-th transfer
    $-\left(p k_{\mathrm{T}}, \Pi\right) \leftarrow$ T.Detect $\left((\overrightarrow{s n})_{j},\left(\overrightarrow{s n^{\prime}}\right)_{j},(t \overrightarrow{a g})_{j},\left(\operatorname{tag}^{\prime}\right)_{j}, \mathcal{U} \mathcal{L}\right)$
    - Return $\left(p k_{\mathrm{T}}, \Pi\right)$
$\underline{\operatorname{VfyGuilt}}\left(p k_{\mathrm{T}}, \Pi\right)$ : Return T.VfyGuilt $\left(p k_{\mathrm{T}}, \Pi\right)$
```

\subsection{Correctness and security analysis}

Theorem 7. Our transferable e-cash scheme satisfies all correctness properties and is perfectly sound.

The first four correctness properties follow in a straightforward way from the correctness properties of $S, S^{\prime}$ and C , and verifiability of T . The fifth property follows from the fact that $s k_{\mathcal{D}}$ has the form of a user secret key.

Because a user verifies the validity of all components of a coin before accepting it, perfect soundness of our scheme is a direct consequence of the correctness properties of $S, S^{\prime}$ and $C$, as well as perfect soundness of $C$ and verifiability of $T$.

Detailed proofs of the following theorems can be found in Appendix A. We omit the proof for $u-a n$ as it is analogous to the one for $\mathrm{c}-\mathrm{an}$.

Theorem 8. Let $\mathcal{N}$ be the nonce space and $\mathcal{S}$ be the space of signatures of scheme S . Let $\mathcal{A}$ be an adversary that wins the unforgeability game with advantage $\epsilon$ and makes at most $d$ calls to BDepo. Suppose that C is perfectly sound and $(\mathcal{M} \cup \mathcal{S})$-extractable. Then there exist adversaries against the unforgeability of the signature schemes S and $\mathrm{S}^{\prime}$ with advantages $\epsilon_{\text {sig }}$ and $\epsilon_{\text {sig }}^{\prime}$, resp., such that

$$
\epsilon \leq \epsilon_{\mathrm{sig}}+\epsilon_{\mathrm{sig}}^{\prime}+d^{2} /|\mathcal{N}|
$$

Assume that during the adversary's deposits the bank never picks the same final nonce twice. (The probability that there is a collision is at most $d^{2} /|\mathcal{N}|$.) In this case, there are two ways for the adversary to win:

(1) CheckDS outputs $\perp$, or an invalid proof, or an unregistered user: Suppose that, during a BDepo call for a coin $c$, CheckDS does not return a coin list. Recall that, by assumption, the final part (chosen by the bank at deposit) of the serial number of $c$ is fresh. Since CheckDS runs T.Detect, by soundness of C and twoextractability of T , this will output a pair $(p k, \Pi)$, such that $\operatorname{VfyGuilt}(p k, \Pi)=1$. Since a coin contains a commitment to a certificate for the used tag key (and proofs of validity), we can, again by soundness of $C$, extract an $S^{\prime}$-signature on
pk. Now if $p k$ is not in $\mathcal{U} \mathcal{L}$, then it was never signed by the bank, and $\mathcal{A}$ has thus broken unforgeability of $S^{\prime}$.

(2) $q_{W}<|\mathcal{D C} \mathcal{L}|$ : If the adversary creates a valid coin that has not been withdrawn, then by soundness of C , we can extract a signature by the bank on a new initial serial number and therefore break unforgeability of S .

Theorem 9. Let $\mathcal{A}$ be an adversary that wins the game exculpability with advantage $\epsilon$ and makes $u$ calls to the oracle URegist. Then there exist adversaries against mode-indistinguishability of C and tag-exculpability of T with advantages $\epsilon_{\mathrm{m} \text {-ind }}$ and $\epsilon_{\mathrm{t} \text {-exc }}$, resp., such that

$$
\epsilon \leq \epsilon_{\mathrm{m} \text {-ind }}+u \cdot \epsilon_{\mathrm{t} \text { texc }}
$$

An incrimination proof in our e-cash scheme is simply an incrimination proof of the tag scheme T. Thus, if the reduction correctly guesses the user $u$ that will be wrongfully incriminated by $\mathcal{A}$ (which it can with probability $1 / u$ ), then we can construct an adversary against exculpability of T . The term $\epsilon_{\mathrm{m} \text {-ind }}$ comes from the fact that we first need to switch $C$ to hiding mode, so we can simulate $\pi_{\text {sn }}$ and $\pi_{t a g}$ for the target user, since the oracles $O_{1}$ and $O_{2}$ in the game for tag exculpability (see Fig. 7) do not return sn-pf and t-pf.

Theorem 10. Let $\mathcal{A}$ be an adversary that wins the coin anonymity game (c-an) with advantage $\epsilon$ and let $k$ be an upper-bound on the number of users transferring the challenge coins. Then there exist adversaries against modeindistinguishability of C and tag-anonymity of \top with advantages $\epsilon_{\mathrm{m} \text {-ind }}$ and $\epsilon_{\mathrm{t} \text {-an }}$, resp., such that

$$
\epsilon \leq 2\left(\epsilon_{\mathrm{m} \text {-ind }}+(k+1) \epsilon_{\mathrm{t} \text {-an }}\right)
$$

Theorem 11. Let $\mathcal{A}$ be an adversary that wins the user anonymity game (u-an) with advantage $\epsilon$ and let $k$ be a bound on the number of users transferring the challenge coin. Then there exist adversaries against mode-indistinguishability of C and tag-anonymity of T with advantages $\epsilon_{\mathrm{m} \text {-ind }}$ and $\epsilon_{\mathrm{t}-\mathrm{an}}$, resp., such that

$$
\epsilon \leq 2 \epsilon_{\mathrm{m}-\mathrm{ind}}+(k+1) \epsilon_{\mathrm{t}-\mathrm{an}}
$$

In the proof of both theorems, we first define a hybrid game in which the commitment key is switched to hiding mode (hence the loss $\epsilon_{\mathrm{m} \text {-ind }}$, which occurs twice for $b=0$ and $b=1$ ). All commitments are then perfectly hiding (and proofs reveal nothing either) and the only information contained in a coin are the serial numbers and tags. They are encrypted, but the adversary, impersonating the bank, can decrypt them.

We then argue that, by tag anonymity of T , the adversary cannot link a user to a pair (sn, tag), even when it knows the users' secret keys. We define a sequence of $k+1$ hybrid games (as $k$ transfers involve $k+1$ users); going through the user vector output by the adversary, we can switch, one by one, all users from the first two the second vector. Each switch can be detected by the adversary with probability at most $\epsilon_{\mathrm{t}-\mathrm{an}}$. Note that the additional factor 2 for $\epsilon_{\mathrm{t} \text {-an }}$ in game c-an is due to the fact that there are two coins for which we switch users, whereas there is only one in game u-an.

Theorem 12. Let $\mathcal{A}$ be an adversary that wins the coin-transparency game (c-tr) with advantage $\epsilon$, let $\ell$ be the size of the two challenge coins, and $k$ be an upper-bound on the number of users transferring the challenge coins. Then there exist adversaries against mode-indistinguishability of C , tag-anonymity of T , IACR-security of E and RCCA-security of $\mathrm{E}^{\prime}$ with advantages $\epsilon_{\mathrm{m} \text {-ind }}, \epsilon_{\mathrm{t} \text {-an }}, \epsilon_{\mathrm{iacr}}$ and $\epsilon_{\text {rcca }}$, resp., such that

$$
\epsilon \leq 2 \epsilon_{\mathrm{m}-\mathrm{ind}}+(k+1) \epsilon_{\mathrm{t} \text {-an }}+(2 \ell+1) \epsilon_{\mathrm{iacr}}+\epsilon_{\text {rcca }}
$$

The crucial difference to the previous anonymity theorems is that the bank is honest (which makes this strong notion possible). We therefore must rely on the security of the encryptions, for which the reduction thus does not know the decryption key. At the same time, the reduction must be able to detect doublespendings, when the adversary deposits coins. Since we use RCCA encryption, the reduction can do so by using its own decryption oracle.

As for c-an and $u-a n$, the reduction first makes all commitments perfectly hiding and proofs perfectly simulatable (which loses $\epsilon_{\mathrm{m} \text {-ind }}$ twice). Since all ciphertexts in the challenge coin given to the adversary are randomized, the reduction can replace all of them, except the initial one, by IACR-security of $\mathbf{E}$. (Note that in the game these ciphertexts never need to be decrypted.) The factor $2 \ell$ is due to the fact that there are at most $\ell$ encryptions of SN/tag pairs. Finally, replacing the initial ciphertext (the one that enables detection of doublespending) can be done by a reduction to RCCA-security of $E^{\prime}$ : the oracle Depo can be simulated by using the reduction's own oracles Dec and GDec (depending on whether Depo' is called before or after the reduction receives the challenge ciphertext) in the RCCA-security game. Note that, when during a simulation of CheckDS, oracle GDec outputs replay, the reduction knows that a challenge coin was deposited, and uses this information to increase ctr.

\section{Instantiation of the building blocks and efficiency}

The instantiations we use are all proven secure in the standard model under non-interactive hardness assumptions.

Commitments and proofs. The commit-and-prove system C will be instantiated with Groth-Sahai proofs [GS08], of which we use the instantiation based on SXDH (defined in Appendix D).

Theorem 13 ([GS08]). The Groth-Sahai scheme, allowing to commit values from $\mathcal{V}:=\mathbb{Z}_{p} \cup \mathbb{G} \cup \hat{\mathbb{G}}$ is perfectly complete, perfectly sound and randomizable; it is $(\mathbb{G} \cup \hat{\mathbb{G}})$-extractable, mode-indistinguishable assuming $S X D H$, and perfectly hiding in hiding mode.

We note that moreover, all our proofs can be made zero-knowledge [GS08], and thus simulatable, because all pairing-product equations we use are homogeneous
(i.e., the right-hand term is the neutral element). We have (efficient) extractability, as we only need to efficiently extract group elements from commitments (and no scalars) in our reductions. (Note that for information-theoretic arguments concerning soundness, Extr can also be inefficient.)

Signature schemes. For efficiency and type-compatibility reasons, we use two different signature schemes. The first one, $S$, must support the functionality Sig Cm , which imposes a specific format of messages. The second scheme, $\mathrm{S}^{\prime}$, is less restrictive, which allows for more efficient instantiations. While all our other components rely on standard assumptions, we instantiate $S$ with a scheme that relies on a non-interactive $q$-type assumption defined in $\left[\mathrm{AFG}^{+} 10\right]$.

Theorem 14. The signature scheme from $\left[A F G^{+} 10\right.$, Sect. 4] with message space $\mathcal{M}:=\left\{\left(g^{m}, \hat{g}^{m}\right) \mid m \in \mathbb{Z}_{p}\right\}$ is (strongly) unforgeable assuming $q-A D H S D H$ and AWFCDH (see Appendix D), and it supports the SigCm functionality [Fuc11].

Theorem 15. The signature scheme from [AGHO11, Sect. 5] is structure-preserving with message space $\mathcal{M}^{\prime}:=\hat{\mathbb{G}}$ and (strongly) unforgeable assuming $S X D H$.

Randomizable encryption schemes. To instantiate the RCCA-secure scheme $\mathrm{E}^{\prime}$ we follow the approach from Libert et al. [LPQ17]. Their construction is only for one group element, but by adapting the scheme, it can support encryption of a vector in $\mathbb{G}^{n}$ for arbitrary $n$. In our e-cash scheme, we need to encrypt a vector in $\mathbb{G}^{2}$, and since it is not clear whether more recent efficient schemes like [FFHR19] can be adapted to this, we give an explicit construction, which we detail in Appendix B.2.

Recall that the RCCA-secure scheme $\mathrm{E}^{\prime}$ is only used to encrypt the initial part of the serial number; using a less efficient scheme does thus not have a big impact on the efficiency of our scheme. From all other ciphertexts contained in a coin (which are under scheme E) we only require IACR security, which standard ElGamal encryption satisfies under $\mathrm{DDH}(!)$. Thus, we instantiate E with ElGamal vector encryption. (Note that our instantiation of $E^{\prime}$ is also built on top of ElGamal). We prove the following in the appendix.

Theorem 16. Assuming SXDH, our randomizable encryption scheme in Appendix B.2 is RCCA-secure and the one in Appendix B. 3 is IACR-secure.

Double-spending tags. We will use a scheme that builds on the one given in [BCFK15]. We have optimized the size of the tags and made explicit all the functionalities not given previously. We defer this to Appendix B.1.

\section{Efficiency analysis}

We conclude by summarizing the sizes of objects in our scheme in the table below and refer to Appendix C for the details of our analysis.

For a group $G \in\left\{\mathbb{G}, \hat{\mathbb{G}}, \mathbb{Z}_{p}\right\}$, let $|G|$ denote the size of an element of $G$. Let $c_{\text {btsrap }}$ denote the coin output by $\mathcal{U}$ at the end of the Withdraw protocol (which corresponds to $c_{\text {init }}$ plus secret values, like $n, \rho_{\mathrm{sn}}$, etc., to be used when
transferring the coin), and let $c_{\text {std }}$ denote one (non-initial) component of the coin. After $k$ transfers the size of a coin is $\left|c_{\mathrm{btsrap}}\right|+k\left|c_{\mathrm{std}}\right|$.

\begin{tabular}{|c|c|c|c|}
\hline$\left|s k_{\mathcal{B}}\right|$ & $9\left|\mathbb{Z}_{p}\right|+2|\mathbb{G}|+2|\hat{\mathbb{G}}|$ & $\left|\Pi_{\text {guilt }}\right|$ & $2|\mathbb{G}|$ \\
\hline$\left|p k_{\mathcal{B}}\right|$ & $15|\mathbb{G}|+8|\hat{\mathbb{G}}|$ & $\left|c_{\text {btstrap }}\right|$ & $6\left|\mathbb{Z}_{p}\right|+147|\mathbb{G}|+125|\hat{\mathbb{G}}|$ \\
\hline$\left|s k_{\mathcal{U}}\right|$ & $\left|\mathbb{Z}_{p}\right|+2|\mathbb{G}|+2|\hat{\mathbb{G}}|$ & $\left|c_{\text {std }}\right|$ & $54|\mathbb{G}|+50|\hat{\mathbb{G}}|$ \\
\hline$\left|p k_{\mathcal{U}}\right|$ & $|\hat{\mathbb{G}}|$ & $|(\overrightarrow{s n}, t \vec{a} g)|$ & $(4 t+2)|\mathbb{G}|$ \\
\hline
\end{tabular}

Acknowledgements. The first two authors were supported by the French ANR EfTrEC project (ANR-16-CE39-0002). This work is funded in part by the MSRInria Joint Centre. The second author is supported by the Vienna Science and Technology Fund (WWTF) through project VRG18-002.

\section{References}

$\mathrm{AFG}^{+}$10. Masayuki Abe, Georg Fuchsbauer, Jens Groth, Kristiyan Haralambiev, and Miyako Ohkubo. Structure-preserving signatures and commitments to group elements. CRYPTO'10.

AGHO11. Masayuki Abe, Jens Groth, Kristiyan Haralambiev, and Miyako Ohkubo. Optimal structure-preserving signatures in asymmetric bilinear groups. CRYPTO'11.

$\mathrm{BCC}^{+}$09. Mira Belenkiy, Jan Camenisch, Melissa Chase, Markulf Kohlweiss, Anna Lysyanskaya, and Hovav Shacham. Randomizable proofs and delegatable anonymous credentials. CRYPTO'09.

$\mathrm{BCF}^{+}$11. Olivier Blazy, Sébastien Canard, Georg Fuchsbauer, Aline Gouget, Hervé Sibert, and Jacques Traoré. Achieving optimal anonymity in transferable e-cash with a judge. AFRICACRYPT'11.

BCFK15. Foteini Baldimtsi, Melissa Chase, Georg Fuchsbauer, and Markulf Kohlweiss. Anonymous transferable E-cash. PKC'15.

$\mathrm{BCG}^{+}$14. Eli Ben-Sasson, Alessandro Chiesa, Christina Garman, Matthew Green, Ian Miers, Eran Tromer, and Madars Virza. Zerocash: Decentralized anonymous payments from Bitcoin. IEEE SBP'14.

BCKL09. M. Belenkiy, M. Chase, M. Kohlweiss, and A. Lysyanskaya. Compact e-cash and simulatable VRFs revisited. Pairing'09.

Bla08. Marina Blanton. Improved conditional e-payments. ACNS'08.

BPS19. Florian Bourse, David Pointcheval, and Olivier Sanders. Divisible e-cash from constrained pseudo-random functions. In ASIACRYPT'19

Bra93. Stefan Brands. Untraceable off-line cash in wallets with observers (extended abstract). CRYPTO'93.

CFN88. David Chaum, Amos Fiat, and Moni Naor. Untraceable electronic cash. CRYPTO'88.

CG08. Sébastien Canard and Aline Gouget. Anonymity in transferable e-cash. ACNS'08.

CGT08. Sébastien Canard, Aline Gouget, and Jacques Traoré. Improvement of efficiency in (unconditional) anonymous transferable e-cash. Fin. Crypto.'08.

Cha83. David Chaum. Blind signature system. CRYPTO'83.

CHL05. Jan Camenisch, Susan Hohenberger, and Anna Lysyanskaya. Compact ecash. EUROCRYPT'05.

CKLM12. Melissa Chase, Markulf Kohlweiss, Anna Lysyanskaya, and Sarah Meiklejohn. Malleable proof systems and applications. EUROCRYPT'12.

CKLM14. Melissa Chase, Markulf Kohlweiss, Anna Lysyanskaya, and Sarah Meiklejohn. Malleable signatures: New definitions and delegatable anonymous credentials. IEEE CSF'14.

CKN03. Ran Canetti, Hugo Krawczyk, and Jesper Buus Nielsen. Relaxing chosenciphertext security. CRYPTO'03.

CP93. David Chaum and Torben P. Pedersen. Transferred cash grows in size. EUROCRYPT'92

CPST16. Sébastien Canard, David Pointcheval, Olivier Sanders, and Jacques Traoré. Divisible e-cash made practical. IET Inf. Security, 10(6):332-347, 2016.

FFHR19. Antonio Faonio, Dario Fiore, Javier Herranz, and Carla Ràfols. Structurepreserving and re-randomizable RCCA-secure public key encryption and its applications. ASIACRYPT'19.

FHY13. Chun-I Fan, Vincent Shi-Ming Huang, and Yao-Chun Yu. User efficient recoverable off-line e-cash scheme with fast anonymity revoking. Mathematical and Computer Modelling, 58(1-2):227-237, 2013.

FOS19. Georg Fuchsbauer, Michele Orrù, and Yannick Seurin. Aggregate cash systems: A cryptographic investigation of Mimblewimble. EUROCRYPT'19.

FP09. Georg Fuchsbauer and David Pointcheval. Proofs on encrypted values in bilinear groups and an application to anonymity of signatures. PAIRING'09.

FPV09. Georg Fuchsbauer, David Pointcheval, and Damien Vergnaud. Transferable constant-size fair e-cash. CANS'09.

Fuc11. Georg Fuchsbauer. Commuting signatures and verifiable encryption. EUROCRYPT'11.

GS08. Jens Groth and Amit Sahai. Efficient non-interactive proof systems for bilinear groups. EUROCRYPT'08.

LPJY13. Benoît Libert, Thomas Peters, Marc Joye, and Moti Yung. Linearly homomorphic structure-preserving signatures and their applications. CRYPTO'13.

LPQ17. Benoît Libert, Thomas Peters, and Chen Qian. Structure-preserving chosenciphertext security with shorter verifiable ciphertexts. PKC'17.

Max15. Gregory Maxwell. Confidential Transactions, 2015. Available at https: //people.xiph.org/ greg/confidential_values.txt.

MGGR13. Ian Miers, Christina Garman, Matthew Green, and Aviel D. Rubin. Zerocoin: Anonymous distributed e-cash from Bitcoin. IEEE S\&P'13.

Nak08. S. Nakamoto. Bitcoin: A peer-to-peer electronic cash. bitcoin.org/ bitcoin.pdf, 2008.

OO89. Tatsuaki Okamoto and Kazuo Ohta. Disposable zero-knowledge authentications and their applications to untraceable electronic cash. CRYPTO'89.

OO91. Tatsuaki Okamoto and Kazuo Ohta. Universal electronic cash. CRYPTO'91.

Poe16. Andrew Poelstra. Mimblewimble, 2016. Available at https://download. wpsoftware.net/bitcoin/wizardry/mimblewimble.pdf.

vS13. Nicolas van Saberhagen. Cryptonote v 2.0, 2013. https://cryptonote. org/whitepaper.pdf.

Zec20. Zcash Protocol Specification 2020.1.15. https://zips.z.cash/protocol/ protocol.pdf.

\section{A Security proofs}

Some of our theorems in the appendix are more general than stated in the body of the paper: they also work for a scheme C that only satisfies computational soundness (whereas in the body we assumed perfect soundness).

\section{A. 1 Unforgeability}

Theorem 17. Suppose that there exists an adversary $\mathcal{A}$ against unforgeability (Def. 2) of our transferable e-cash scheme with advantage $\epsilon_{\text {unforg }}$ making at most $d$ calls to oracle BDepo. Suppose that $\mathcal{M}$ and the signature space of S are contained in $\mathcal{V}^{\prime}$. Then we can build a polynomial-time adversary $\mathcal{B}_{1}$ against the unforgeability of the signature scheme S with advantage $\epsilon_{\text {sig }}$, an adversary $\mathcal{B}_{2}$ against the unforgeability of $\mathrm{S}^{\prime}$ with advantage $\epsilon_{\text {sig }}^{\prime}$, and $\mathcal{B}_{3}$ and $\mathcal{B}_{4}$ against the soundness of the commitment scheme C with advantage $\epsilon_{h, 1}$ and $\epsilon_{h, 2}$. Then

$$
\epsilon_{\text {unforg }} \leq \epsilon_{h, 1}+\epsilon_{h, 2}+\epsilon_{\text {sig }}+\epsilon_{\text {sig }}^{\prime}+\frac{d^{2}}{|\mathcal{N}|}
$$

Proof. Note that the adversary has two possibilities to win the game: either it creates a counterfeit (i.e., $q_{W}<|\mathcal{C}|$ ), or it wins by making a deposit fail (i.e., CheckDS does neither output a list nor a valid pair with a registered user key). In our proof we will consider these two aspects separately. First we will prove in Proposition 18, that creating counterfeit is harder than breaking the unforgeability of S, or proving a false statement in C. In Proposition 19, we prove that if fresh nonces are picked during each deposits, then it is harder to make Deposit fail than breaking the unforgeability of $S^{\prime}$, or proving a false statement in C.

We first recall the unforgeability against the e-cash system:

```
$\operatorname{Expt}_{\mathcal{A}}^{\text {unforg }}(\lambda)$ :
    par $\leftarrow \operatorname{ParamGen}\left(1^{\lambda}\right) ;\left(s k_{\mathcal{B}}, p k_{\mathcal{B}}\right) \leftarrow$ BKeyGen $($ par $)$
    $\mathcal{A}^{\text {BRegist,BWith,BDepo }}\left(p a r, p k_{\mathcal{B}}\right.$ )
```

If in a BDepo call, CheckDS does not return a coin list

Return 1 if any of the following hold:

- CheckDS did not output a pair $(p k, \Pi)$

- VfyGuilt $(p k, \Pi)=0$

- $p k \notin \mathcal{U}$

Let $q_{W}$ be the number of calls to BWith

If $q_{W}<|\mathcal{D C L}|$ then return 1

Return 0

and the unforgeability game against a signature scheme:

$$
\begin{array}{lc}
\operatorname{Expt}_{\mathrm{S}, \mathcal{B}}^{\mathrm{sig}-\mathrm{uf}}(\lambda) & \text { Oracle: S.Sign }(s k, m, Q): \\
\text { par } \leftarrow \mathrm{S} . \text { Setup }\left(1^{\lambda}\right) & Q:=Q \cup\{m\} \\
(s k, v k) \leftarrow \text { S.KeyGen }(p a r) & \text { Return S.Sign }(s k, m) \\
Q:=\emptyset & \\
(m, \sigma) \leftarrow \mathcal{B}^{\mathrm{s} . \operatorname{Sign}^{*}(s k, \cdot, Q)}(\text { par, vk }) & \\
\text { Return }(m \notin Q \wedge \text { S.Verify }(v k, m, \sigma)) &
\end{array}
$$

Finally, the soundness of the commitment scheme:

```
$\operatorname{Expt}_{C, \mathcal{B}}^{\text {soundness }}(\lambda)$ :
    $(c k, x k) \leftarrow$ C.ExSetup $\left(1^{\lambda}\right)$
    $Q:=\emptyset ;\left(E, c_{1}, \ldots, c_{n}\right) \leftarrow \mathcal{B}(c k)$
    Return (C.Verify $\left(c k, E, c_{1}, \ldots, c_{n}\right) \wedge \neg E\left(\right.$ C.Extr $\left(x k, c_{1}\right), \ldots$, C.Extr $\left.\left.\left(x k, c_{n}\right)\right)\right)$
```

Let $E_{\text {unforg }}$ be the event that $\mathcal{A}$ wins the game, that is, at some point after a call to BDepo, CheckDS did not output a list, or $q_{W}<|\mathcal{D C L}|$. We partition $E_{\text {unforg }}$ as follows:
- $E_{\text {Decrypt-fails }}$ : In CheckDS, a decryption fails or does not output any serial number and tag, when it is supposed to

$-E_{\text {same }}$ : In CheckDS, there is no $j$ such that $\overrightarrow{\operatorname{sn}}_{j} \neq \overrightarrow{\mathrm{sn}}_{j}^{\prime}$
- $E_{\text {DDS-fails }}$ In CheckDS, algorithm T.Detect does not output any $\left(p k_{\mathrm{T}}, \Pi_{G}\right)$
- $E_{\text {incorrect }}$ : CheckDS outputs $\left(p k_{i^{*}}, \Pi_{G}\right)$ such that VfyGuilt $\left(p k_{i^{*}}, \Pi_{G}\right)=0$
- $E_{\text {not-register }}: p k_{i^{*}} \notin \mathcal{U} \mathcal{L}$

$-E_{\text {counterfeit }}: q_{W}<|\mathcal{D C L}|$

We first build an adversary $\mathcal{B}_{1}$ against the unforgeability of $S$, which will bet on $E_{\text {counterfeit }}: q_{W}<|\mathcal{D C L}|$ (i.e., $\mathcal{A}$ creates valid money). Thus, $\mathcal{A}$ has produced a committed signature for a fresh serial number and thus forged a signature for a fresh serial number or a false proof for the equation S.Verify (In this second case adversary $\mathcal{B}_{3}$ will break soundness). Note that to simulate SigCm, adversary $\mathcal{B}_{1}$ needs $x k$ and SmSigCm.

```
Adversary $\mathcal{B}_{1}^{\mathcal{A}, \mathrm{S} . \operatorname{Sign}^{*}(s k, \cdot)}\left(\operatorname{par}_{\mathrm{S}}, v k\right)$ :
    Obtain Gr from par ${ }_{S}$
    $(c k, x k) \leftarrow$ C.ExSetup $(G r)$
    $\operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime}$. Setup $(G r)$
    $\left(s k^{\prime}, v k^{\prime}\right) \leftarrow S^{\prime}$.KeyGen $\left(p a r_{S^{\prime}}\right)$
    $\operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T}$.Setup $(G r)$
    $\operatorname{par} \leftarrow\left(1^{\lambda}, G r, p \operatorname{cr}_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$
    Coins $_{\mathrm{D}}:=\emptyset$
    Coins $_{\mathrm{W}}:=\emptyset$
    $(e k, d k) \leftarrow$ E.KeyGen $(G r)$
    $\left(e k^{\prime}, d k^{\prime}\right) \leftarrow \mathrm{E}^{\prime}$.KeyGen $(G r)$
    $p k_{\mathcal{B}} \leftarrow\left(e k^{\prime}, e k, v k, v k^{\prime}\right)$
    $\left(s k_{\mathrm{T}}, p k_{\mathrm{T}}\right) \leftarrow$ T.KeyGen $(G r)$
    $s k_{\mathcal{D}} \leftarrow\left(\varepsilon, s k_{\mathrm{T}}, p k_{\mathrm{T}}\right)$
```

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-34.jpg?height=54&width=517&top_left_y=396&top_left_x=522)

In each call of BWith, add the coin received to the list Coins $\mathrm{W}_{\mathrm{W}}$

In each call of BDepo, add the coin received to the list Coins $\mathrm{D}_{\mathrm{D}}$

BWith* is similar to BWith, except instead of using SigCm, it uses SigCm*:

$\mathrm{SigCm}(c)$ :

$m \leftarrow$ C.Extr $(x k, c)$

Use the oracle to obtain $\Sigma \leftarrow$ S.Sign ${ }^{*}(s k, m)$

$\left(c_{\sigma}, \pi\right) \leftarrow \operatorname{SmSigCm}(\mathrm{xk}, v k, c, \Sigma)$

Return $\left(c_{\sigma}, \pi\right)$

Let $q_{W}$ be the number of successful calls to BWith

If $q_{W} \geq|\mathcal{D C L}|$ then abort

Let $D:=\emptyset$

Let $W:=\emptyset$

For $c \in$ Coins $_{\mathrm{W}}$ :

Parse $c$ as $\left(c^{0},\left(c^{j}\right)_{j=1}^{i}, n, s n, \rho_{\mathrm{sn}}, \rho_{p k}\right)$

Parse $c^{0}$ as $\left(c_{p k}^{0}, c_{\text {cert }}^{0}, \pi_{\text {cert }}^{0}, c_{\mathrm{sn}}^{0}, \pi_{\mathrm{sn}}^{0}, c_{M}, c_{\sigma}^{0}, \pi_{\sigma}^{0}, \tilde{c}_{s n}^{0}, \tilde{\pi}_{\mathrm{sn}}^{0}\right)$

$M:=$ C.Extr $\left(x k, c_{M}\right)$

$\sigma:=$ C.Extr $\left(\mathrm{xk}, c_{\sigma}^{0}\right)$

$W:=W \cup\{(M, \sigma)\}$

For $c \in$ Coins $_{\mathrm{D}}$ :

Parse $c$ as $\left(c^{0},\left(c^{j}\right)_{j=1}^{i}, n, s n, \rho_{s n}, \rho_{p k}\right)$

Parse $c^{0}$ as $\left(c_{p k}^{0}, c_{\text {cert }}^{0}, \pi_{\text {cert }}^{0}, c_{\mathrm{sn}}^{0}, \pi_{\mathrm{sn}}^{0}, c_{M}, c_{\sigma}^{0}, \pi_{\sigma}^{0}, \tilde{c}_{\mathrm{sn}}^{0}, \tilde{\pi}_{\mathrm{sn}}^{0}\right)$

$M:=$ C.Extr $\left(x k, c_{M}\right)$

$\sigma:=$ C.Extr $\left(x k, c_{\sigma}^{0}\right)$

$D:=D \cup\{(M, \sigma)\}$

If $\exists(M, \sigma) \in W \backslash D$ :

then return $(M, \sigma)$

Else abort

We let $\varepsilon$ identify the part of the secret key that is ignored in the entire game (because the bank never spent a coin). By correctness of the committed signature of $S$, the simulation will be perfect. And by $(\mathcal{M} \cup \mathcal{S})$-extractability of C , we deduce that $\mathcal{B}_{1}$ is efficient.

We now construct a first adversary $\mathcal{B}_{3}$ against soundness:

Adversary $\mathcal{B}_{3}^{\mathcal{A}}(c k)$ :

Obtain Gr from $c k$

$\operatorname{par}_{\mathrm{S}} \leftarrow$ S.Setup $(G r)$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-34.jpg?height=51&width=342&top_left_y=1975&top_left_x=520)

$\operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r)$

$p a r \leftarrow\left(1^{\lambda}, G r, p a r_{\mathrm{S}}, p a r_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$

$S:=\emptyset$

$\left(p k_{\mathcal{B}}=\left(e k^{\prime}, e k, v k, v k^{\prime}\right), s k_{\mathcal{W}}=\left(s k, s k^{\prime}\right), s k_{\mathcal{D}}, s k_{\mathcal{C K}}\right) \leftarrow$ BKeyGen ()

Run $\mathcal{A}^{\text {BRegist,BWith,BDepo }}\left(p a r, p k_{\mathcal{B}}\right.$ )

In each call of BDepo, add the entire coin received in the list Coins $\mathrm{D}_{\mathrm{D}}$

Let $q_{W}$ be the number of successful calls to BWith

$$
\begin{aligned}
& \text { If } q_{W} \geq|\mathcal{D C L}| \text {, then abort } \\
& \text { Let } D:=\emptyset \\
& \text { For } c \in \text { Coins }_{\mathrm{D}} \text { : } \\
& \text { Parse } c \text { as }\left(c_{0},\left(c_{j}\right)_{j=1}^{i}, n, s n, \rho_{s n}, \rho_{p k}\right) \\
& \text { Add } c_{0} \text { to } S \\
& \text { Parse } S \text { as }\left\{\left(c_{p k_{T}}^{i}, c_{\text {cert }}^{i}, \pi_{c e r t}^{i}, c_{\mathrm{sn}}^{i}, \pi_{\mathrm{sn}}^{i}, c_{M}^{i}, c_{\sigma}^{i}, \pi_{\sigma}^{i}, \tilde{c}_{\mathrm{sn}}^{i}, \tilde{\pi}_{\mathrm{sn}}^{i}\right)\right\}_{1 \leq i \leq|S|} \\
& \text { Parse } \widetilde{\pi}_{s n}^{i} \text { as }\left(c^{i}, \pi^{i}\right) \\
& \text { Return }\left(\bigwedge _ { i = 1 } ^ { | S | } \left(\mathrm{E}^{\prime} . \text { Verify }\left(e k^{\prime}, X_{1}^{i}, X_{2}^{i}, \tilde{c}_{\mathrm{sn}}^{i}\right) \wedge \text { S.Verify }\left(v k, X_{3}^{i}, X_{4}^{i}\right)=1 \wedge\right.\right. \\
& \left.\operatorname{T} . \mathrm{SVfy}_{\text {init }}\left(X_{5}^{i}, X_{1}^{i}, X_{3}^{i}\right)\right) \\
& \left.c_{\mathrm{sn}}^{1}, c^{1}, c_{M}^{1}, c_{\sigma}^{1}, c_{p k_{T}}^{1}, \ldots, c_{s n}^{|S|}, c^{|S|}, c_{M}^{|S|}, c_{\sigma}^{|S|}, c_{p k_{T}}^{|S|}, \bigwedge_{i=1}^{|S|}\left(\pi^{i} \wedge \pi_{\sigma}^{i} \wedge \pi_{s n}^{i}\right)\right)
\end{aligned}
$$

We define the following events:

$E_{\text {sig }}: \mathcal{B}_{1}$ breaks the unforgeability of the signature scheme S ; and $E_{\text {com }, 1}: \mathcal{B}_{3}$ breaks the soundness of the commitment scheme C .

Proposition 18. $E_{\text {counterfeit }} \subset E_{\text {sig }} \cup E_{\text {com }, 1}$.

Suppose that we are in case $E_{\text {counterfeit }} \backslash E_{\text {sig. }}$. Because the coin has been accepted during Spend in a call to BDepo, the proofs output by $\mathcal{B}_{3}$ are correct. Let $\left(s n^{1}, \nu^{1}, M^{1}, p k_{\mathrm{T}}^{1}, \sigma^{1}, \ldots, s n^{|S|}, \nu^{|S|}, M^{|S|}, p k_{\mathrm{T}}^{|S|}, \sigma^{|S|}\right)$ be the values that the challenger of the soundness game extracts from the commitments output by $\mathcal{B}_{3}$. If for some $i$ : S.Verify $\left(\mathrm{vk}, M^{i}, \sigma^{i}\right) \neq 1$, then $\mathcal{B}_{3}$ wins the soundness game.

Suppose that $\bigwedge_{i=1}^{|S|}$ S.Verify $\left(v k, M^{i}, \sigma^{i}\right)=1$. Since we are not in $E_{\text {sig }}$, all values $M^{i}$ correspond to coins that have been withdrawn. But there are only $q_{W}$ such coins and thus $q_{W}$ such messages $M$. Thus, we have $\left|\left\{M^{i}\right\}_{i=1}^{|S|}\right| \leq q_{W}$.

If $\bigwedge_{i=1}^{|S|} \mathrm{T} . \mathrm{SVfy}_{\text {init }}\left(p k_{\mathrm{T}}^{i}, \mathrm{sn}^{i}, M^{i}\right) \neq 1$, then $\mathcal{B}_{3}$ won the soundness game.

Assume $\bigwedge_{i=1}^{|S|} \mathrm{T} . \mathrm{SVfy}_{\text {init }}\left(p k_{\mathrm{T}}^{i}, s n^{i}, M^{i}\right)=1$. Since T is bootable, it must hold that $\left|\left\{s^{n}\right\}_{i=1}^{|S|}\right| \leq\left|\left\{M^{i}\right\}_{i=1}^{|S|}\right|$, from which we get $\left|\left\{s n^{i}\right\}_{i=1}^{|S|}\right| \leq q_{W}<|\mathcal{D C} \mathcal{L}|$ (the last inequality follows since we assumed to be in $E_{\text {counterfeit }}$.

Note that by construction of $\mathcal{D C L}$, all the initial serial numbers of the elements of $\mathcal{D C L}$ are different. Let us call this set $I$. From $|I|=|\mathcal{D C L}|$, we deduce $|I|>\left|\left\{s n^{i}\right\}_{i=1}^{\mid S S}\right|$. By construction, $|I|=\left|\left\{\mathrm{E}^{\prime} . \operatorname{Dec}\left(d k^{\prime}, \tilde{c}_{\mathrm{sn}}^{i}\right)\right\}_{i=1}^{|S|}\right|$, and thus $\left|\left\{\mathrm{E}^{\prime} . \operatorname{Dec}\left(d k^{\prime}, \tilde{c}_{\text {sn }}^{i}\right)\right\}_{i=1}^{|S|}\right|>\left|\left\{s n^{i}\right\}_{i=1}^{|S|}\right|$. Let $i_{0}$ be such that $\mathrm{E}^{\prime} . \operatorname{Dec}\left(d k^{\prime}, \tilde{c}_{\mathrm{sn}}^{i_{0}}\right) \notin$ $\left|\left\{s n^{i}\right\}_{i=1}^{|S|}\right|$. By correctness of $\mathrm{E}^{\prime}$, we have $\mathrm{E}^{\prime}$. Enc $\left(p k, s n^{i_{0}}, \nu^{i_{0}}\right) \neq \tilde{c}_{\mathrm{sn}}^{i_{0}}$, and thus (again by correctness of $\mathrm{E}^{\prime}$ ) $\mathrm{E}^{\prime}$. Verify $\left(p k, s n^{i_{0}}, \nu^{i_{0}}, \tilde{c}_{\text {sn }}^{i_{0}}\right) \neq 1$.

We deduce that $\bigwedge_{i=1}^{|S|} E^{\prime}$. Verify $\left(p k, s n^{i}, \nu^{i}, c_{\text {sn }}^{i}\right) \neq 1$, and consequently $\mathcal{B}_{3}$ won the soundness game. We thus have $E_{\text {counterfeit }} \backslash E_{\text {sig }} \subset E_{\text {com }, 1}$.

We build an algorithm to break unforgeability of $\mathrm{S}^{\prime}$ :

Adversary $\mathcal{B}_{2}^{\mathcal{A}, S^{\prime} . \operatorname{Sign}^{*}\left(\mathrm{sk}^{\prime}, \cdot\right)}\left(\operatorname{par}_{\mathrm{S}^{\prime}}, v k^{\prime}\right)$ :

Initialize $\mathcal{U} \mathcal{L}$ as empty list

Obtain Gr from par $_{\mathrm{S}^{\prime}}$

$(c k, x k) \leftarrow$ C.ExSetup $(G r)$

$\operatorname{par}_{\mathrm{S}} \leftarrow$ S.Setup $(G r)$

$(s k, v k) \leftarrow$ S.KeyGen $(G r)$

$p a r_{\mathrm{T}} \leftarrow \mathrm{T}$.Setup $(G r)$

$\operatorname{par} \leftarrow\left(1^{\lambda}, G r, p a r_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$

$\left(e k^{\prime}, d k^{\prime}\right) \leftarrow \mathrm{E}^{\prime}$. KeyGen (Gr)

$(e k, d k) \leftarrow$ E.KeyGen $(G r)$

$\left(s k_{\mathrm{T}}, p k_{\mathrm{T}}\right) \leftarrow$ T.KeyGen $(G r)$

$s k_{\mathcal{D}} \leftarrow\left(\varepsilon, p k_{\mathrm{T}}, s k_{\mathrm{T}}\right)$

$p k_{\mathcal{B}} \leftarrow\left(e k^{\prime}, e k, v k, v k^{\prime}\right)$

Run $\mathcal{A}^{\text {BRegist,BWith,BDepo }}\left(\right.$ par, $p k_{\mathcal{B}}$ )

Each time we would use $\mathrm{S}^{\prime}$. Sign in an oracle call we use

$\mathrm{S}^{\prime} . \mathrm{Sign}^{*}\left(s k^{\prime}, \cdot\right)$, and we add the input to $\mathcal{U} \mathcal{L}$

Let $(p k, \Pi)$ be the output of the last call to BDepo

If such a pair is never returned by BDepo, then abort

Let $c_{1}$ be the last coin sent by the user

Parse $c_{1}$ as $\left(c^{0},\left(c^{j}\right)_{j=1}^{i}, n\right.$, sn, $\left.\rho_{\mathrm{sn}}, \rho_{p k}\right)$

Let $j$ be minimal such that $(\overrightarrow{s n})_{j-1} \neq\left(\overrightarrow{s n^{\prime}}\right)_{j-1}$

(using the notation from CheckDS)

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-36.jpg?height=71&width=1060&top_left_y=1336&top_left_x=522)

$$
\left.\tilde{c}_{\mathrm{sn}}^{j-1}, \tilde{\pi}_{s n}^{j-1}, \underline{\tilde{c}_{t a g}^{j-1}}, \underline{\tilde{\pi}_{t a g}^{j-1}}\right)
$$

$p k_{\mathrm{T}}:=\mathrm{C} . \operatorname{Extr}\left(\mathrm{xk}, c_{p k_{\mathrm{T}}}^{j-1}\right)$

$\sigma:=\mathrm{C} . \operatorname{Extr}\left(\mathrm{xk}, c_{\text {cert }}^{j-1}\right)$

If $p k_{\mathrm{T}} \notin \mathcal{U} \mathcal{L}$ :

then return $\left(p k_{\mathrm{T}}, \sigma\right)$

Else abort

We denote by $\varepsilon$ the part of the secret key that could be ignored in the protocols (e.g., the certificate cert of a receiver is never used). Let $E_{\text {sig }}^{\prime}$ be the event that $\mathcal{B}_{2}$ breaks the unforgeability of $\mathrm{S}^{\prime}$.

We construct a second adversary against soundness of C:

Adversary $\mathcal{B}_{4}^{\mathcal{A}}(c k)$ :

Obtain $G r$ from $c k$

$\operatorname{par}_{\mathrm{S}} \leftarrow$ S.Setup $(G r) ; \operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime}$.Setup $(G r) ; \operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T}$.Setup $(G r)$

$\operatorname{par} \leftarrow\left(1^{\lambda}, G r, p a r_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$

$\left(p k_{\mathcal{B}}=\left(e k^{\prime}, e k, v k, v k^{\prime}\right), s k_{\mathcal{W}}=\left(s k, s k^{\prime}\right), s k_{\mathcal{D}}, s k_{\mathcal{C K}}\right) \leftarrow \operatorname{BKeyGen}()$

Run $\mathcal{A}^{\text {BRegist,BWith,BDepo }}$ ( par, $p k_{\mathcal{B}}$ )

Let $(p k, \Pi)$ be the output of the last call to BDepo

If such a pair is never returned by BDepo, then abort

Let $c$ be the last coin sent by the user and $i$ be its size

Parse $c$ as $\left(c^{0},\left(c^{k}\right)_{k=1}^{i}, n, s n, \rho_{\mathrm{sn}}, \rho_{p k}\right)$

Let $j$ be minimal such that $(\overrightarrow{\operatorname{sn}})_{j-1} \neq\left(\overrightarrow{s n}^{\prime}\right)_{j-1}$

(using the notation from CheckDS)

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-37.jpg?height=68&width=1047&top_left_y=524&top_left_x=520)

$$
c_{\sigma}^{(j-1)}, \pi_{\sigma}^{(j-1)}, \tilde{c}_{s n}^{(j-1)}, \tilde{\pi}_{s n}^{(j-1)}, \underline{\tilde{c}_{\text {tag }}^{(j-1)}}, \underline{\left.\tilde{\pi}_{\text {tag }}^{(j-1)}\right)}
$$

Do the same for all $k \in\{0, \ldots, i\}$

If $j \neq 1$, then parse $\pi_{s n}^{(j-1)}$ as $\left(\pi_{s n, \text { valid }}^{(j-1)}, c_{s n-p f}^{(j-1)}\right)$

Else $\left(\pi_{s n, \text { valid }}^{(j-1)}, c_{s n-p f}^{(j-1)}\right) \leftarrow\left(\pi_{s n}^{(j-1)}, c_{M}\right)$

Parse $\pi_{s n}^{j}$ as $\left(\pi_{s n, \text { valid }}^{j}, c_{s n-p f}^{j}\right)$ and $\pi_{t a g}^{j}$ as $\left(\pi_{t a g, \text { valid }}^{j}, c_{t-p f}^{j}\right)$

For all $k \in\{0, \ldots, i\}$ :

Parse $\tilde{\pi}_{\mathrm{sn}}^{k}$ as $\left(c_{\nu_{\mathrm{sn}}}^{k}, \pi_{\mathrm{sn}, \mathrm{eq}}^{k}\right)$ and parse $\tilde{\pi}_{\text {tag }}^{k}$ as $\left(c_{\nu_{\text {tag }}}^{k}, \pi_{\text {tag }, \mathrm{eq}}^{k}\right)$

Let $c^{\prime}$ be the coin that collides with $c$ and $i^{\prime}$ be its size

Parse $c^{\prime}$ as $\left(c^{\prime 0},\left(c^{\prime k}\right)_{k=1}^{i^{\prime}}, n^{\prime}, s n^{\prime}, \rho_{s n}^{\prime}, \rho_{p k}^{\prime}\right)$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-37.jpg?height=76&width=949&top_left_y=1019&top_left_x=520)

$$
\left.\underline{\pi_{t a g}^{\prime(j-1)},}, c_{M}^{\prime(j-1)}, c_{\sigma}^{\prime(j-1)}, \pi_{\sigma}^{\prime(j-1)}, \tilde{c}_{\mathrm{sn}}^{\prime(j-1)}, \tilde{\pi}_{\mathrm{sn}}^{\prime(j-1)}, \tilde{c}_{\text {tag }}^{\prime(j-1)}, \underline{\tilde{\pi}_{\text {tag }}^{\prime(j-1)}}\right)
$$

Do the same for all $k \in\left\{0, \ldots, i^{\prime}\right\}$

Parse $\pi_{s n}^{\prime j}$ as $\left(\pi_{s n, \text { valid }}^{\prime j}, c_{s n-p f}^{\prime j}\right)$

Parse $\pi_{t a g}^{\prime j}$ as $\left(\pi_{t a g, \text { valid }}^{\prime j}, c_{t-p f}^{\prime j}\right)$

Parse $\tilde{\pi}_{\mathrm{sn}}^{\prime(j-1)}$ as $\left(c_{\nu_{s n}}^{\prime(j-1)}, \pi_{\mathrm{sn}, \mathrm{eq}}^{\prime(j-1)}\right)$

Parse $\tilde{\pi}_{s n}^{\prime j}$ as $\left(c_{\nu_{s n}}^{\prime j}, \pi_{s n, \text { eq }}^{\prime j}\right)$

Parse $\tilde{\pi}_{\text {tag }}^{\prime j}$ as $\left(c_{\nu_{\text {tag }}^{\prime j}}^{\prime j}, \pi_{\text {tag }, \text { eq }}^{\prime j}\right)$

Return $\left(\mathrm{E}^{\prime}\right.$.Verify $\left(e k^{\prime}, Y_{0}, Y_{1}, \tilde{c}_{\mathrm{sn}}^{0}\right) \wedge \bigwedge_{k=1}^{i}$ E.Verify $\left(e k, Y_{2 k}, Y_{2 k+1}, \tilde{c}_{\mathrm{sn}}^{k}\right) \wedge$

$$
\begin{aligned}
& \bigwedge_{k=1}^{i} \text { E.Verify }\left(e k, Y_{2 i+2 k}, Y_{2 i+2 k+1}, \tilde{c}_{t a g}^{k}\right) \wedge \\
& \mathrm{S}^{\prime} . \text { Verify }\left(v k^{\prime}, X_{1}, X_{2}\right)=1 \wedge \\
& \text { E.Verify }\left(e k, X_{5}, X_{6}, \tilde{c}_{\mathrm{sn}}^{(j-1)}\right) \wedge \text { E.Verify }\left(e k, X_{7}, X_{8}, \tilde{c}_{s n}^{\prime(j-1)}\right) \wedge \\
& \text { E.Verify }\left(e k, X_{9}, X_{10}, \tilde{c}_{\mathrm{sn}}^{j}\right) \wedge \operatorname{E} . \operatorname{Enc}\left(e k, X_{11}, X_{12}, \tilde{c}_{\mathrm{sn}}^{\prime j}\right) \wedge \\
& \text { T.SVfy }_{\text {all }}\left(X_{1}, X_{5}, X_{13}\right)=1 \wedge \\
& \mathrm{T} . \mathrm{SVfy}\left(X_{1}, X_{9}, X_{14}\right)=1 \wedge \mathrm{T} . \operatorname{SVfy}\left(X_{3}, X_{11}, X_{15}\right)=1 \wedge \\
& \text { E.Verify }\left(e k, X_{16}, X_{17}, \tilde{c}_{\text {tag }}^{j}\right) \wedge \text { E.Enc }\left(e k, X_{18}, X_{19}, \tilde{c}_{\text {tag }}^{\prime j}\right) \wedge \\
& \text { T.TVfy }\left(X_{1}, X_{5}, X_{9}, X_{16}, X_{20}\right)=1 \wedge \\
& \text { T.TVfy }\left(X_{1}, X_{7}, X_{11}, X_{18}, X_{21}\right)=1 \text {, } \\
& c_{s n}^{0}, c_{\nu_{s n}}^{0}, \ldots, c_{s n}^{k}, c_{\nu_{\mathrm{sn}}}^{k}, c_{\text {tag }}^{1}, c_{\nu_{\text {tag }}}^{1}, \ldots, c_{\text {tag }}^{k}, c_{\nu_{\text {tag }}}^{k} \\
& c_{p k}^{(j-1)}, c_{c e r t}^{(j-1)}, c_{p k}^{j}, c_{p k}^{\prime j}, c_{\mathrm{sn}}^{(j-1)}, c_{\nu_{s n}}^{(j-1)}, c_{\mathrm{sn}}^{\prime(j-1)}, c_{\nu_{s n}}^{(j-1)}, c_{s n}^{j}, c_{\nu_{s n}}^{j}, c_{\mathrm{sn}}^{\prime j}, c_{\nu_{s n}}^{\prime j} \\
& c_{\mathrm{sn-pf}}^{(j-1)}, c_{\mathrm{sn}-p f}^{j}, c_{\mathrm{sn}-p f}^{\prime j}, c_{\text {tag }}^{j}, c_{\nu_{\text {tag }}}^{j}, c_{\text {tag }}^{\prime j}, c_{\nu_{\text {tag }}^{\prime}}^{\prime j}, c_{t-p f}^{j}, c_{t-p f}^{\prime j} \\
& \bigwedge_{k=0}^{i} \pi_{\mathrm{sn}, \mathrm{eq}}^{k} \bigwedge_{k=1}^{i} \pi_{\text {tag,eq }}^{k} \wedge \pi_{\mathrm{cert}}^{(j-1)} \wedge \pi_{\mathrm{sn}, \mathrm{eq}}^{(j-1)} \wedge \pi_{\mathrm{sn}, \mathrm{eq}}^{(j-1)} \wedge \pi_{\mathrm{sn}, \mathrm{eq}}^{j} \wedge \pi_{\mathrm{sn}, \mathrm{eq}}^{\prime j} \wedge \\
& \left.\pi_{\text {sn,valid }}^{(j-1)} \wedge \pi_{\text {sn,valid }}^{j} \wedge \pi_{\text {sn,valid }}^{\prime j} \wedge \pi_{\text {tag,eq }}^{j} \wedge \pi_{\text {tag,eq }}^{\prime j} \wedge \pi_{\text {tag,valid }}^{j} \wedge \pi_{\text {tag,valid }}^{\prime j}\right)
\end{aligned}
$$
where $Y_{j}$ is the variable in the equations representing the purported values in the $j$-th commitment, and the $X_{i}$ 's are the last 21 variables.

We define the following two events:

$E_{\text {com }, 2}: \mathcal{B}_{4}$ breaks the soundness of C , and

$E_{\text {same-nonce }}$ : the same nonce is picked twice by the bank during two different calls to BDepo.

Proposition 19. $E_{\text {unforg }} \backslash E_{\text {counterfeit }} \subset E_{\text {same }} \cup E_{\text {com }, 2} \cup E_{\text {unforg }}^{\prime}$.

Suppose that we are in $E_{\text {unforg }} \backslash\left(E_{\text {com }, 2} \cup E_{\text {counterfeit }} \cup E_{\text {same }}\right)$. Because the coin has been accepted, the proofs are correct (as they are verified in the Spend protocol, during a call to BDepo). We are thus in a case where the extracted commitment will verify the equations. Let

$\left(\mathrm{sn}^{0}, \ldots, \mathrm{sn}^{i}, \operatorname{tag}^{1}, \ldots, \operatorname{tag}^{i}, p k_{\text {tag }}^{(j-1)}, \operatorname{cert}^{(j-1)}, p k_{\text {tag }}^{j}, p k_{\text {tag }}^{\prime j}, s n^{(j-1)}, \nu_{\mathrm{sn}}^{(j-1)}, \mathrm{sn}^{\prime(j-1)}\right.$, $\left.\nu_{s n}^{\prime(j-1)}, s n^{j}, \nu_{s n}^{j}, s n^{\prime j}, \nu_{s n}^{\prime j}, s n-p f^{(j-1)}, s n-p f^{j}, s n-p f^{\prime j}, \operatorname{tag}^{j}, \nu_{t a g}^{j}, \operatorname{tag}^{j \prime}, \nu_{t a g}^{j \prime}, t-p f^{j}, t-p f^{\prime j}\right)$

be what the challenger of the soundness game extracts from the commitments output by $\mathcal{B}_{4}$. Since we are not in $E_{\text {com }, 2}$ we have that $\mathcal{B}_{4}$ loses the game: for all $k \in\{1, \ldots, i\}$ :

$$
\mathrm{E}^{\prime} . \text { Verify }\left(e k^{\prime}, s n^{0}, \nu_{\mathrm{sn}}^{0}, \tilde{c}_{\mathrm{sn}}^{0}\right)=\text { E.Verify }\left(e k, s n^{k}, \nu_{\mathrm{sn}}^{k}, \tilde{c}_{\mathrm{sn}}^{k}\right)=1
$$

and for all $k \in\{1, \ldots, i\}$ :

$$
\text { E.Verify }\left(e k, \operatorname{tag}^{k}, \nu_{\mathrm{sn}}^{k}, \tilde{c}_{\text {tag }}^{k}\right)=1
$$

By correctness of E and $\mathrm{E}^{\prime}$, we deduce that $E_{\text {Decrypt-fails }}$ will not happen. Being in $E_{\text {unforg }} \backslash E_{\text {counterfeit }}$ means that CheckDS detected that the first SN-component of $c$ is the same as that of another coin (here $c^{\prime}$ ). Note that the last sn of a deposited coin is generated (with the key $s k_{\top}$ ) and encrypted by the bank itself. Now because we are not in $E_{\text {same }}$, we have that CheckDS will find some $j$, such that $\overrightarrow{\operatorname{sn}}_{j} \neq \overrightarrow{\operatorname{sn}}^{\prime}$.

By construction, E.Dec $\left(d k, \tilde{c}_{\mathrm{sn}}^{j}\right)=\mathrm{E} . \operatorname{Dec}\left(d k, \tilde{c}_{\mathrm{sn}}^{\prime j}\right)$, which by correctness of E (and because we are not in $E_{\text {com, 2 }}$ ) means $s n^{(j-1)}=s n^{\prime(j-1)}$. Since

$$
\begin{aligned}
1 & =\mathrm{T} . \mathrm{SVfy}\left(p k_{\mathrm{T}}^{j}, s n^{j}, s n-p f^{j}\right) \\
& =\mathrm{T} . \mathrm{SVfy}\left(p k_{\mathrm{T}}^{\prime j}, s n^{\prime j}, s n-p f^{\prime j}\right) \\
& =\mathrm{T} . \mathrm{TVfy}\left(p k_{\mathrm{T}}^{(j-1)}, s n^{(j-1)}, s n^{j}, t a g^{j}, t-p f^{j}\right) \\
& =\mathrm{T} . \mathrm{TVfy}\left(p k_{\mathrm{T}}^{(j-1)}, s n^{(j-1)}, s n^{\prime j}, t a g^{\prime j}, t-p f^{\prime j}\right) \\
& =\mathrm{T}^{\prime S} \mathrm{SVfy}_{\text {all }}\left(p k_{\mathrm{T}}^{(j-1)}, s n^{(j-1)}, s n-p f^{(j-1)}\right)
\end{aligned}
$$

and, because T is SN -identifiable, we get that $p k_{\text {tag }}^{j}=p k_{\text {tag }}^{\prime j}$.

Moreover, since T is two-extractable, we deduce that if $p k_{\text {tag }}^{j} \in \mathcal{U} \mathcal{L}$, and that $E_{\text {DDSfails }}, E_{\text {incorrect }}$ and $E_{\text {not-register }}$ will not happen.

We have proved that $\left(E_{\text {unforg }} \backslash E_{\text {counterfeit }} \cup E_{\text {same }} \cup E_{\text {com, 2 }}\right) \Longrightarrow p k_{\text {tag }}^{j} \notin \mathcal{U} \mathcal{L}$. Finally note that if $p k_{\text {tag }}^{j} \notin \mathcal{U} \mathcal{L}$, and if $E_{\text {com, } 2} \cup E_{\text {same }} \cup E_{\text {counterfeit }}$ does not happen, then $\mathcal{B}_{2}$ will win the unforgeability game against $\mathrm{S}^{\prime}$. This yields:

$$
E_{\text {unforg }} \backslash\left(E_{\text {com }, 2} \cup E_{\text {counterfeit }}\right) \subset E_{\text {same }} \cup E_{\text {sig }}^{\prime}
$$

Now suppose we are in $E_{\text {same }}$. By correctness of E, we deduce that the serial numbers were also identical before their encryption. Then by $\mathcal{N}$-injectivity, we have that the nonces picked during the deposits were the same, and we are therefore in $E_{\text {same-nonce }}$. Thus $E_{\text {same }} \subset E_{\text {same-nonce }}$. From this we deduce

$$
E_{\text {unforg }} \subset E_{\text {com }, 2} \cup E_{\text {same-nonce }} \cup E_{\text {sig }}^{\prime} \cup E_{\text {com }, 2} \cup E_{\text {sig. }}
$$

By considering the probabilities, we finally conclude that

$$
\epsilon \leq \epsilon_{\mathrm{h}, 1}+\epsilon_{\mathrm{h}, 2}+\epsilon_{\mathrm{sig}}+\epsilon_{\mathrm{sig}}^{\prime}+\frac{d^{2}}{N}
$$

\section{A. 2 Exculpability}

Theorem 20. Suppose there is an adversary $\mathcal{A}$ against exculpability (Def. 3) of our scheme with advantage $\epsilon$ that makes at most $u$ calls to the oracle URegist. Then there exist adversaries $\mathcal{B}_{1}$ against tag-exculpability with advantage $\epsilon_{\text {tag }}$, and $\mathcal{B}_{2}$ against mode-hiding of C with advantage $\epsilon_{\mathrm{m} \text {-ind }}$ such that

$$
\epsilon \leq u \epsilon_{\text {tag }}+\epsilon_{\mathrm{m}-\mathrm{ind}}
$$

We start with recalling the tag-exculpability game:

$$
\begin{aligned}
& \text { Experiment } \operatorname{Expt}_{\mathrm{T} \mathcal{B}}^{\mathrm{tag} \text {-exculpability }}(G r): \quad O_{1}(s k) \\
& \operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r) \quad n \stackrel{\$}{\leftarrow} \mathcal{N} ; T[k]:=n ; k:=k+1 \\
& \left(s k_{\mathrm{T}}, p k_{\mathrm{T}}\right) \leftarrow \text { T.KeyGen }\left(1^{\lambda}\right) \quad(s n, s n-p f) \leftarrow \text { T.SGen }(s k, n) \\
& \mathcal{L}:=\emptyset \quad \text { Return sn }
\end{aligned}
$$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-39.jpg?height=65&width=773&top_left_y=1732&top_left_x=546)

$$
\begin{aligned}
& \text { Return T.VfyGuilt }\left(p k_{\mathrm{T}}, \Pi^{\prime}\right) \quad \text { If } T[i]=\perp \text {, abort the oracle call } \\
& n:=T[i] T[i]:=\perp \\
& (\text { tag }, t-p f) \leftarrow \text { T.TGen }\left(s k, n, s n^{\prime}\right) \\
& \text { Return tag }
\end{aligned}
$$

We construct the following adversary against tag-exculpability of T .

$$
\begin{aligned}
& \text { Adversary } \mathcal{B}_{1}^{O_{1}\left(s k_{\mathrm{T}}\right), O_{2}\left(s k_{\mathrm{T}}, \cdot\right)}\left(p \operatorname{ar}_{\mathrm{T}}, p k_{\mathrm{T}}\right): \\
& \text { Obtain } G r \text { from } \operatorname{par}_{\mathrm{T}} \\
& (c k, t d) \leftarrow \mathrm{C} . \operatorname{SmSetup}(G r) \\
& \operatorname{par}_{\mathrm{S}} \leftarrow \mathrm{S} . \operatorname{Setup}(G r) \\
& \operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r) \\
& \operatorname{par} \leftarrow\left(1^{\lambda}, p \operatorname{ar}_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, p \operatorname{ar}_{\mathrm{T}}, c k\right)
\end{aligned}
$$
$p k_{\mathcal{B}} \leftarrow \mathcal{A}(p a r)$

$u^{*} \stackrel{\leftarrow}{\leftarrow}\{1, \ldots, u\}$

$\left(i^{*}, \Pi^{*}\right) \leftarrow \mathcal{A}^{\text {URegist,Spy,UWith,Rcv,Spd,S\&R,UDepo }}\left(\right.$ par, $p k_{\mathcal{B}}$ )

In the $u^{*}$-th call of URegist, use $p k_{\mathrm{T}}$ (instead of running T.KeyGen)

If the adversary queries $\operatorname{Spy}\left(u^{*}\right)$, abort

If the adversary queries UWith, Rcv, Spd, S\&R, UDepo on $u^{*}$,

use $O_{1}$ and $O_{2}$, and $t d$ (since $s k_{\mathrm{T}}$ is unknown)

If $\mathrm{O}_{2}$ fails, abort the entire procedure

Output $\Pi^{*}$

The game is perfectly simulated from $\mathcal{A}$ 's point of view, except when it calls $\operatorname{Spy}\left(u^{*}\right)$, or makes that user double-spend, or if it detects that we are in hidingmode (which happens with probability at most $\epsilon_{\mathrm{m} \text {-ind }}$ ). Let $E_{\mathrm{ex}}$ and $E_{\mathrm{tag}}$ be the events that $\mathcal{A}$ wins and that $\mathcal{B}_{1}$ wins, respectively. Suppose that we are in $E_{\text {ex }}$. This means that $\mathcal{A}$ forges a proof against one of the registered users (and does not spy on her). The probability that this user is $u^{*}$ is at least $\frac{1}{u}$. In this case, we have:
- $\mathcal{A}$ did not spy on $u^{*}$ or make her double-spend (as in both cases we would not be in $E_{\text {ex }}$ ).

$-\operatorname{VfyGuilt}\left(p k_{\mathrm{T}}, \Pi^{*}\right)=1$ (because we are in $\left.E_{\mathrm{ex}}\right)$; thus T.VfyGuilt $\left(s k_{\mathrm{T}}, \Pi^{*}\right)=1$.

We thus deduce that

$$
\operatorname{Pr}\left[E_{\mathrm{ex}}\right] \leq u \operatorname{Pr}\left[E_{\mathrm{tag}}\right]
$$

\section{A. 3 Coin anonymity}

Theorem 21. Suppose there is an $\mathcal{A}$ against coin anonymity (c-an) of our scheme with advantage $\epsilon$ and let $k$ be an upper-bound on the number of users transferring the challenge coins. Then there exist adversaries against modeindistinguishability of C and tag-anonymity of \top with advantages $\epsilon_{\mathrm{m} \text {-ind }}$ and $\epsilon_{\mathrm{t} \text {-an }}$, resp., such that

$$
\epsilon \leq 2\left(\epsilon_{\mathrm{m}-\mathrm{ind}}+(k+1) \epsilon_{\mathrm{t}-\mathrm{an}}\right)
$$

Proof sketch. In the proof, we first define a hybrid game in which the commitment key is switched to hiding mode (hence the loss $\epsilon_{\mathrm{m} \text {-ind }}$, which occurs twice for $b=0$ and $b=1$ ). All commitments are then perfectly hiding and the only information available to the adversary are the serial numbers and tags. (They are encrypted in the coin, but the adversary, impersonating the bank, can decrypt them.)

We then argue that, by tag anonymity of T , the adversary cannot link a user to a pair (sn, tag), even when it knows the users' secret keys. We define a sequence of $k+1$ hybrid games (as $k$ transfers involve $k+1$ users); going through the user vector output by the adversary, we can switch, one by one, all users from the first two the second vector. Each switch can be detected by the adversary with probability at most $2 \epsilon_{\text {t-an }}$.

A technical difficulty occurs during the first swap: We would like to switch the two initial serial numbers of $c_{0}$ and $c_{1}$, but this seems problematic, as during the first withdraw (of $c_{0}$ ), the challenger does not yet know $i_{1}$ (and possibly this user has not even been defined yet), and thus the initial serial number of $c_{1}$. But fortunately, we note (in Proposition 25) that in hiding mode of the proof system, we do not need to compute the initial serial numbers during the withdraws! This is because we only send to the adversary (playing the bank) committed elements and proofs that reveal no information. We can therefore compute theses serial numbers after these withdraws, and switch them at this later moment.

\section{Full proof. We recall $\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{c}-\mathrm{an}}$ :}

```
$\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{c}-\mathrm{an}}(\lambda)$
    par $\leftarrow \operatorname{ParamGen}\left(1^{\lambda}\right) ; p k_{\mathcal{B}} \leftarrow \mathcal{A}(p a r)$
    $i_{0} \leftarrow \mathcal{A}^{\text {URegist,Spy }}$
    Run UWith $\left(i_{0}\right)$ with $\mathcal{A}$
    $i_{1} \leftarrow \mathcal{A}^{\mathrm{URegist}, \mathrm{Spy}}$
    Run UWith $\left(i_{1}\right)$ with $\mathcal{A}$
        $\left(i^{(\overrightarrow{0})}, i^{(\overrightarrow{1})}\right) \leftarrow \mathcal{A}^{\text {URegist, Spy }}$
        Let $k:=\left|i^{(\overrightarrow{0})}\right|$; if $k \neq\left|i^{(\overrightarrow{1})}\right|$, abort the entire procedure
        Then repeat the following step for $j=1, \ldots, k$ :
            Run S\&R $\left(2 j-1,\left(i^{(\overrightarrow{0})}\right)_{j}\right) ; \operatorname{Run} \operatorname{S} \& \mathrm{R}\left(2 j,\left(i^{(\overrightarrow{1})}\right)_{j}\right)$
        Run $\operatorname{Spd}(2 k+1+b)$ with $\mathcal{A}$
        Run $\operatorname{Spd}(2 k+2-b)$ with $\mathcal{A}$
        $b^{*} \leftarrow \mathcal{A} ;$ return $b^{*}$
```

In the game $\operatorname{Expt}_{\mathcal{A}, 0, \text { hiding }}^{\mathrm{c} \text {-an }}$, we will change the commitment key. If the adversary detects this, it breaks the mode-indistinguishability of C . Thus the distribution of the experiment will not change except with probability $\epsilon_{\mathrm{m} \text {-ind }}$ (Property 22 ).

```
Experiment $\operatorname{Expt}_{\mathcal{A}, 0, \text { hiding }}^{\mathrm{c} \text { an }}(\lambda)$ :
    $G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right)$
    $\operatorname{par}_{\mathrm{T}} \leftarrow$ T.Setup $(G r)$
    $\operatorname{par}_{\mathrm{S}} \leftarrow \operatorname{S.Setup}(G r)$
    $p \operatorname{ar}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r)$
    $(c k, t d) \leftarrow$ C.SmSetup $(G r)$
$\operatorname{par} \leftarrow\left(1^{\lambda}, G r\right.$, par $_{\mathrm{S}}$, par $\left._{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$
$p k_{\mathcal{B}} \leftarrow \mathcal{A}(p a r)$
$i_{0} \leftarrow \mathcal{A}^{\mathrm{URegist}, \mathrm{Spy}}$
Run UWith $\left(i_{0}\right)$ with $\mathcal{A}$
$i_{1} \leftarrow \mathcal{A}^{\text {URegist,Spy }}$
Run UWith $\left(i_{1}\right)$ with $\mathcal{A}$
```
$\left(i^{(\overrightarrow{0})}, i^{(\overrightarrow{1})}\right) \leftarrow \mathcal{A}^{\text {URegist,Spy }}$

Let $k:=\left|i^{\overrightarrow{0})}\right| ;$ if $k \neq\left|i^{\overrightarrow{1})}\right|$, abort the entire procedure

Then repeat the following for $j=1, \ldots, k$ :

Run S\&R $\left(2 j-1,\left(i^{(\overrightarrow{0})}\right)_{j}\right)$ : Run $\mathrm{S} \& \mathrm{R}\left(2 j,\left(i^{(\overrightarrow{1})}\right)_{j}\right)$

Run $\operatorname{Spd}(2 k+1+b)$ with $\mathcal{A}$

$\operatorname{Run} \operatorname{Spd}(2 k+2-b)$ with $\mathcal{A}$

$b^{*} \leftarrow \mathcal{A}$; return $b^{*}$

Proposition 22. $\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{c}-\mathrm{an}}(\lambda)$ and $\operatorname{Expt}_{\mathcal{A}, 0, \text { hiding }}^{\mathrm{c} \text { can }}(\lambda)$ are $\epsilon_{\mathrm{m} \text {-ind-statistically close }}$

Note that $t d$ is never used in $\operatorname{Expt}_{\mathcal{A}, 0, \text { hiding }}^{\mathrm{c}-\mathrm{an}}(\lambda)$. Therefore, the game can be simulated using a mode-indistinguishability challenge (Gr, ck). If ck has been generated by C.Setup, this simulates $\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{c}-\mathrm{an}}(\lambda)$; if $c k$ has been generated by C.SmSetup, this simulates $\operatorname{Expt}_{\mathcal{A}, 0, \text { hiding }}^{\mathrm{c}-a n}$. This experiment can therefore be seen as a mode-distinguisher.

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-42.jpg?height=52&width=1211&top_left_y=1042&top_left_x=457)

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-42.jpg?height=43&width=1211&top_left_y=1087&top_left_x=457)
stituted by C.SmPrv and every C.Cm by C.ZCm.

Now each time the challenger is using an oracle, it uses C.ZCm instead of C.Cm, C.SmPrv instead of C.Prv, C.SmPrv enc instead of C.Prvenc, etc.

Let $\mathrm{S} \& \mathrm{R}_{\mathrm{ZK}}$, UWith $_{\mathrm{ZK}}, \mathrm{Spd}_{\mathrm{ZK}}$ denote these modified oracles.

Experiment $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZK}}^{\mathrm{c}-\mathrm{an}}(\lambda)$ :

$$
G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right)
$$

$$
\begin{aligned}
& \operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r) \\
& \operatorname{par}_{\mathrm{S}} \leftarrow \mathrm{S} . \operatorname{Setup}(G r) \\
& \operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r) \\
& (c k, t d) \leftarrow \text { C.SmSetup }(G r) \\
& p a r \leftarrow\left(1^{\lambda}, G r, p a r_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right) \\
& p k_{\mathcal{B}} \leftarrow \mathcal{A}(p a r) \\
& i_{0} \leftarrow \mathcal{A}^{\mathrm{URegist}, \mathrm{Spy}}
\end{aligned}
$$

Run UWith $\mathrm{ZK}\left(i_{0}\right)$ with $\mathcal{A}$

$i_{1} \leftarrow \mathcal{A}^{\text {URegist,Spy }}$

Run $\mathrm{UWith}_{\mathrm{ZK}}\left(i_{1}\right)$ with $\mathcal{A}$

$\left(i^{(\overrightarrow{0})}, i^{(\overrightarrow{1})}\right) \leftarrow \mathcal{A}^{\text {URegist,Spy }}$

Let $k:=\left|i^{\overrightarrow{(0)}}\right|$; if $k \neq\left|i^{\overrightarrow{1})}\right|$, abort the entire procedure

Then repeat the following step for $j=1, \ldots, k$ :

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-42.jpg?height=65&width=849&top_left_y=1992&top_left_x=642)

$$
\begin{aligned}
& \text { Run } \operatorname{Spd}_{\mathrm{ZK}}(2 k+1+b) \text { with } \mathcal{A} \\
& \text { Run } \operatorname{Spd}_{\mathrm{ZK}}(2 k+2-b) \text { with } \mathcal{A} \\
& b^{*} \leftarrow \mathcal{A} ; \text { return } b^{*}
\end{aligned}
$$

Because we are in the hiding mode, the following follows directly from perfect zero-knowledge in hiding mode:

Proposition 23. $\operatorname{Expt}_{\mathcal{A}, 0, \text { hiding }}^{\mathrm{c} \text {-an }}(\lambda)$ and $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZK}}^{c-a n}(\lambda)$ are equivalently distributed.

We now consider the following part of $\left.\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZK}}^{\mathrm{c}-\mathrm{an}}(\lambda)\right)$ in more detail:

```
i _ { 0 } \leftarrow \mathcal { A } ^ { \mathrm { URegist,Spy } }
Run UWith
i}\leftarrow\leftarrow\mp@subsup{\mathcal{A}}{}{\mathrm{ URegist,Spy}
Run UWith
```

We would like to swap the serial numbers of $i_{0}$ and $i_{1}$ by using tag-anonymity. The issue here is that in the first call to $\mathrm{UWith}_{\mathrm{ZK}}$, we do not know $i_{1}$ yet (because it is only chosen in a second round). Fortunately, at this step we only sent $\mathcal{A}$ data that is unrelated to this serial number, since we are using ZCm. Thus, at the end of this part, we can compute the ciphertexts of both initial coins.

We can decompose this part of the game as follows:

$$
\begin{aligned}
& i_{0} \leftarrow \mathcal{A}^{\mathrm{URegist}, \mathrm{Spy}} \\
& n^{(0)} \stackrel{\&}{\leftarrow} ; \rho_{s n}^{(0)}, \rho_{c e r t}^{(0)}, \rho_{p k}^{(0)}, \rho_{M}^{(0)} \stackrel{\&}{\leftarrow} \\
& \left(s n^{(0)}, M_{s n}^{(0)}\right) \leftarrow \text { T.SGen }_{\text {init }}\left(s k_{i_{0}}, n^{(0)}\right) \\
& c_{c e r t}^{(0)}, c_{s n}^{(0)}, c_{n k}^{(0)}, c_{M}^{(0)} \leftarrow \mathrm{C} . \mathrm{ZCm}\left(c k, \rho_{c e r t}^{(0)}, \rho_{\mathrm{sn}}^{(0)}, \rho_{n k}^{(0)}, \rho_{M}^{(0)}\right) \\
& \pi_{c e r t}^{(0)} \leftarrow \operatorname{C.SmPrv}\left(t d, \mathrm{~S}^{\prime} . \operatorname{Verify}\left(v k^{\prime}, \cdot \cdot \cdot\right)=1, \rho_{p k}^{(0)}, \rho_{c e r t}^{(0)}\right) \\
& \pi_{\mathrm{sn}}^{(0)} \leftarrow \operatorname{C.SmPrv}_{s n, \text { init }}\left(t d, \rho_{D k}^{(0)}, \rho_{s n}^{(0)}, \rho_{M}^{(0)}\right) \\
& \nu_{s n}^{(0)} \leftarrow \mathcal{R} \\
& \tilde{c}_{s n}^{(0)} \leftarrow \operatorname{E} . \mathrm{Enc}\left(e k, s n^{(0)}, \nu_{s n}^{(0)}\right) \\
& \tilde{\pi}_{s n}^{(0)} \leftarrow C . \operatorname{SmPrv}_{\mathrm{enc}}\left(t d, e k, \rho_{s n}^{(0)}, \tilde{c}_{\mathrm{sn}}^{(0)}\right)
\end{aligned}
$$

Pick $\rho^{\overrightarrow{0})^{\prime}}$ long enough to compute:

$$
\begin{aligned}
& c_{1}^{(0)}=\left(\operatorname{Rand}\left(\left(c_{D k}^{(0)}, c_{c e r t}^{(0)}, \pi_{c e r t}^{(0)}, c_{\mathrm{sn}}^{(0)}, \pi_{\mathrm{sn}}^{(0)}, c_{M}^{(0)}, c_{\sigma}^{(0)}, \pi_{\sigma}^{(0)}, \tilde{c}_{\mathrm{sn}}^{(0)}, \tilde{\pi}_{\mathrm{sn}}^{(0)}\right), \rho^{(\overrightarrow{0}) \prime}\right)\right. \\
& \left.n^{(0)}, s n^{(0)}, \rho_{s n}^{(0)}+\left(\rho^{(\overrightarrow{0}) \prime}\right)_{s n}, \rho_{p k}^{(0)}+\left(\rho^{(\overrightarrow{0})}\right)_{p k}\right) \\
& \mathcal{C L} \leftarrow\left[\left(i_{0}, c_{1}^{(0)}, 0, \mathcal{A}\right)\right] \\
& i_{1} \leftarrow \mathcal{A}^{\text {URegist,Spy }} \\
& n^{(1)} \stackrel{\S}{\leftarrow} \mathcal{N} ; \rho_{\mathrm{sn}}^{(1)}, \rho_{c e r t}^{(1)}, \rho_{p k}^{(1)}, \rho_{M}^{(1)} \stackrel{\&}{\leftarrow}
\end{aligned}
$$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-43.jpg?height=54&width=551&top_left_y=1908&top_left_x=646)

$$
\begin{aligned}
& c_{\text {cert }}^{(1)}, c_{\mathrm{sn}}^{(1)}, c_{p k}^{(1)}, c_{M}^{(1)} \leftarrow \mathrm{C} . \mathrm{ZCm}\left(c k, \rho_{c e r t}^{(1)}, \rho_{\mathrm{sn}}^{(1)}, \rho_{D k}^{(1)}, \rho_{M}^{(1)}\right) \\
& \pi_{c e r t}^{(1)} \leftarrow \operatorname{C.SmPrv}\left(t d, \mathrm{~S}^{\prime} . \operatorname{Verify}\left(v k^{\prime}, \cdot, \cdot \cdot\right)=1, \rho_{p k}^{(1)}, \rho_{\text {cert }}^{(1)}\right) \\
& \pi_{s n}^{(1)} \leftarrow \operatorname{CoSmPrv}_{s n, \text { init }}\left(t d, \rho_{p k}^{(1)}, \rho_{s n}^{(1)}, \rho_{M}^{(1)}\right)
\end{aligned}
$$

Send $\left(c_{p k}^{(1)}, c_{c e r t}^{(1)}, \pi_{c e r t}^{(1)}, c_{s n}^{(1)}, c_{M}^{(1)}, \pi_{s n}^{(1)}\right)$ to $\mathcal{A}$

Receive $\left(c_{\sigma}^{(1)}, \pi_{\sigma}^{(1)}\right)$ from $\mathcal{A}$

If C.Verify $\left(c k, \operatorname{S.Verify}(v k, \cdot, \cdot)=1, c_{M}^{(1)}, c_{\sigma}^{(1)}, \pi_{\sigma}^{(1)}\right)=0$, then return $\perp$ $\nu_{\text {sn }}^{(1)} \stackrel{\&}{\leftarrow}$

$$
\begin{aligned}
& \tilde{c}_{\mathrm{sn}}^{(1)} \leftarrow \operatorname{E} . \operatorname{Enc}\left(e k, s n^{(1)}, \nu_{s n}^{(1)}\right) \\
& \tilde{\pi}_{\mathrm{sn}}^{(1)} \leftarrow \operatorname{C.SmPrv}_{\text {enc }}\left(t d, e k, \rho_{\mathrm{sn}}^{(1)}, \tilde{c}_{\mathrm{sn}}^{(1)}\right)
\end{aligned}
$$

Pick $\rho^{(\overrightarrow{1}) \prime}$ long enough to compute the following:

$$
\begin{aligned}
& c_{1}^{(1)}=\left(\operatorname{Rand}\left(\left(c_{p k}^{(1)}, c_{c e r t}^{(1)}, \pi_{c e r t}^{(1)}, c_{s n}^{(1)}, \pi_{s n}^{(1)}, c_{M}^{(1)}, c_{\sigma}^{(1)}, \pi_{\sigma}^{(1)}, \tilde{c}_{s n}^{(1)}, \tilde{\pi}_{s n}^{(1)}\right), \rho^{(\overrightarrow{1}) \prime}\right)\right. \\
& \left.n^{(1)}, s n^{(1)}, \rho_{s n}^{(1)}+\left(\rho^{(1) \prime}\right)_{s n}, \rho_{p k}^{(1)}+\left(\rho^{(\overrightarrow{1}) \prime}\right)_{p k}\right) \\
& \mathcal{C L}[2] \leftarrow\left(i_{1}, c_{1}^{(1)}, 0, \mathcal{A}\right)
\end{aligned}
$$

We can do the sn-computations and the encryptions at the end of this part (because they are not related to data sent to $\mathcal{A}$ ). We can therefore replace the previous instructions by the following algorithm DoubleUWith:

```
DoubleUWith ${ }^{\mathcal{A}}$ :
    $i_{0} \leftarrow \mathcal{A}^{\text {URegist, } \mathrm{Spy}}$
    $\rho_{\mathrm{sn}}^{(0)}, \rho_{c e r t}^{(0)}, \rho_{p k}^{(0)}, \rho_{M}^{(0)} \stackrel{\leftarrow}{\leftarrow}$; Compute:
        $c_{c e r t}^{(0)}, c_{s n}^{(0)}, c_{p k}^{(0)}, c_{M}^{(0)} \leftarrow \mathrm{C.ZCm}\left(c k, \rho_{c e r t}^{(0)}, \rho_{\mathrm{sn}}^{(0)}, \rho_{p k}^{(0)}, \rho_{M}^{(0)}\right)$
        $\pi_{c e r t}^{(0)} \leftarrow$ C.SmPrv $\left(t d, \mathrm{~S}^{\prime} . \operatorname{Verify}\left(v k^{\prime}, \cdot, \cdot\right)=1, \rho_{p k}^{(0)}, \rho_{c e r t}^{(0)}\right)$
        $\pi_{s n}^{(0)} \leftarrow \operatorname{C.SmPrv}_{\text {sn,init }}\left(t d, \rho_{p k}^{(0)}, \rho_{\mathrm{sn}}^{(0)}, \rho_{M}^{(0)}\right)$
    Send $\left(c_{p k}^{(0)}, c_{\text {cert }}^{(0)}, \pi_{c e r t}^{(0)}, c_{\mathrm{sn}}^{(0)}, c_{M}^{(0)}, \pi_{\mathrm{sn}}^{(0)}\right)$ to $\mathcal{A}$
    Receive $\left(c_{\sigma}^{(0)}, \pi_{\sigma}^{(0)}\right)$ from $\mathcal{A}$
    If C.Verify $\left(c k, \operatorname{S.Verify}(v k, \cdot, \cdot)=1, c_{M}^{(0)}, c_{\sigma}^{(0)}, \pi_{\sigma}\right)=0$ then output $\perp$
    $i_{1} \leftarrow \mathcal{A}^{\text {URegist, Spy }}$
    $\rho_{s n}^{(1)}, \rho_{c e r t}^{(1)}, \rho_{p k}^{(1)}, \rho_{M}^{(1)} \leftarrow \mathcal{R}$; Compute:
        $c_{p k}^{(1)} \leftarrow \operatorname{C.ZCm}\left(c k, \rho_{p k}^{(1)}\right) ; c_{\text {cert }}^{(1)} \leftarrow \operatorname{C.ZCm}\left(c k, \rho_{\text {cert }}^{(1)}\right)$
        $c_{\mathrm{sn}}^{(1)} \leftarrow \mathrm{C.ZCm}\left(c k, \rho_{\mathrm{sn}}^{(1)}\right) ; c_{M}^{(1)} \leftarrow \mathrm{C.ZCm}\left(c k, \rho_{M}^{(1)}\right)$
        $\pi_{c e r t}^{(1)} \leftarrow \operatorname{C.SmPrv}\left(t d, \mathrm{~S}^{\prime} . \operatorname{Verify}\left(v k^{\prime}, \cdot, \cdot\right)=1, \rho_{p k}^{(1)}, \rho_{c e r t}^{(1)}\right)$
        $\pi_{s n}^{(1)} \leftarrow \operatorname{C.SmPrv}_{\text {sn,init }}\left(t d, \rho_{p k}^{(1)}, \rho_{\mathrm{sn}}^{(1)}, \rho_{M}^{(1)}\right)$
    Send $\left(c_{p k}^{(1)}, c_{\text {cert }}^{(1)}, \pi_{c e r t}^{(1)}, c_{\mathrm{sn}}^{(1)}, c_{M}^{(1)}, \pi_{\mathrm{sn}}^{(1)}\right)$ to $\mathcal{A}$
    Receive $\left(c_{\sigma}^{(1)}, \pi_{\sigma}^{(1)}\right)$ from $\mathcal{A}$
    If C.Verify $\left(c k\right.$, S.Verify $\left.(v k, \cdot, \cdot)=1, c_{M}^{(1)}, c_{\sigma}^{(1)}, \pi_{\sigma}^{(1)}\right)=0$ then output $\perp$
    $n^{(0)}, n^{(1)} \stackrel{\leftarrow}{\leftarrow} \mathcal{N} ;\left(n^{(0)}, M_{s n}^{(0)}\right) \leftarrow \mathrm{T}_{\mathrm{sm}} \mathrm{SGn}_{\text {init }}\left(\mathrm{sk}_{i_{0}}, n^{(0)}\right)$
    $\left(s n^{(1)}, M_{s n}^{(1)}\right) \leftarrow$ T.SGen $_{\text {init }}\left(k_{i_{1}}, n^{(1)}\right)$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-44.jpg?height=51&width=225&top_left_y=1850&top_left_x=554)
```
    $\tilde{c}_{\mathrm{sn}}^{(0)} \leftarrow \operatorname{E} . \operatorname{Enc}\left(e k, s n^{(0)}, \nu_{s n}^{(0)}\right) ; \tilde{c}_{s n}^{(1)} \leftarrow \operatorname{E} . \operatorname{Enc}\left(e k, s n^{(1)}, \nu_{s n}^{(1)}\right)$
    $\tilde{\pi}_{\mathrm{sn}}^{(0)} \leftarrow \mathrm{C.SmPrv} \mathrm{Snc}_{\mathrm{enc}}\left(t d, e k, \rho_{\mathrm{sn}}^{(0)}, \tilde{c}_{\mathrm{sn}}^{(0)}\right)$
    $\tilde{\pi}_{\mathrm{sn}}^{(1)} \leftarrow \operatorname{CiSmPrv}_{\text {enc }}\left(t d, e k, \rho_{\mathrm{sn}}^{(1)}, \tilde{c}_{\mathrm{sn}}^{(1)}\right)$
    Pick uniformly at random $\rho^{(\overrightarrow{0})}, \rho^{(\overrightarrow{1}) \prime}$ long enough to compute:
    $c_{1}^{(0)}=\left(\operatorname{Rand}\left(\left(c_{p k}^{(0)}, c_{c e r t}^{(0)}, \pi_{c e r t}^{(0)}, c_{s n}^{(0)}, \pi_{s n}^{(0)}, c_{M}^{(0)}, c_{\sigma}^{(0)}, \pi_{\sigma}^{(0)}, \tilde{c}_{\mathrm{sn}}^{(0)}, \tilde{\pi}_{s n}^{(0)}\right), \rho^{(\overrightarrow{0}) \prime}\right)\right.$,
            $\left.n^{(0)}, s n^{(0)}, \rho_{\mathrm{sn}}^{(0)}+\left(\rho^{(\overrightarrow{0})}\right)_{s n}, \rho_{p k}^{(0)}+\left(\rho^{(\overrightarrow{0})}\right)_{p k}\right)$
    $c_{1}^{(1)}=\left(\operatorname{Rand}\left(\left(c_{p k}^{(1)}, c_{c e r t}^{(1)}, \pi_{c e r t}^{(1)}, c_{s n}^{(1)}, \pi_{s n}^{(1)}, c_{M}^{(1)}, c_{\sigma}^{(1)}, \pi_{\sigma}^{(1)}, \tilde{c}_{s n}^{(1)}, \tilde{\pi}_{s n}^{(1)}\right), \rho^{(\overrightarrow{1}) \prime}\right)\right.$,
                        $\left.n^{(1)}, s n^{(1)}, \rho_{s n}^{(1)}+\left(\rho^{(\overrightarrow{1}) \prime}\right)_{s n}, \rho_{p k}^{(1)}+\left(\rho^{(\overrightarrow{1})}\right)_{p k}\right)$
```

$$
\mathcal{C L}[1] \leftarrow\left(i_{0}, c_{1}^{(0)}, 0, \mathcal{A}\right) ; \mathcal{C} \mathcal{L}[2] \leftarrow\left(i_{1}, c_{1}^{(1)}, 0, \mathcal{A}\right)
$$

$$
\text { Return }\left(i_{0}, i_{1}\right)
$$

To express these instruction changes, we define the following game.

\section{Experiment $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2}^{\mathrm{c} \text { an }}(\lambda)$ :}

$$
\begin{aligned}
& G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right) \\
& \operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r) \\
& \operatorname{par}_{\mathrm{S}} \leftarrow \mathrm{S} . \operatorname{Setup}(G r) \\
& \operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r) \\
& (c k, t d) \leftarrow \text { C.SmSetup }(G r) \\
& \text { par } \leftarrow\left(1^{\lambda}, \operatorname{par}_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, p_{\mathrm{T}}, c k\right) \\
& p k_{\mathcal{B}} \leftarrow \mathcal{A}(p a r) \\
& \left(i_{0}, i_{1}\right) \leftarrow \text { DoubleUWith }^{\mathcal{A}} \\
& \left(i^{(\overrightarrow{0})}, i^{(\overrightarrow{1})}\right) \leftarrow \mathcal{A}^{\text {URegist,Spy }}
\end{aligned}
$$

Let $k:=\left|i^{(\overrightarrow{0})}\right|$; if $k \neq\left|i^{(\overrightarrow{1})}\right|$, abort the entire procedure

Then repeat the following for $j=1, \ldots, k$ :

Run S\&R $\mathrm{R}_{\mathrm{ZK}}\left(2 j-1,\left(i^{(\overrightarrow{0})}\right)_{j},\right) ; \operatorname{Run} \mathrm{S} \& \mathrm{R}_{\mathrm{ZK}}\left(2 j,\left(i^{(\overrightarrow{1})}\right)_{j}\right)$

Run $\operatorname{Spd}_{\mathrm{ZK}}(2 k+1+b)$ with $\mathcal{A}$

Run $\operatorname{Spd}_{\mathrm{ZK}}(2 k+2-b)$ with $\mathcal{A}$

$b^{*} \leftarrow \mathcal{A} ;$ return $b^{*}$

Since this change is transparent for the adversary, we get the following:

Proposition 24. $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZK}}^{\mathrm{c}-\mathrm{an}}(\lambda)$ and $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2}^{\mathrm{c}-\mathrm{n}}(\lambda)$ are equally distributed.

Next we have to swap the serial numbers. We define two new procedures:

```
DoubleUWith {
    io}\leftarrow\mp@subsup{\mathcal{A}}{}{\textrm{URegist,Spy}
    \rhosn
        c
        csn (0)}\leftarrow\textrm{C.ZCm}(ck,\mp@subsup{\rho}{sn}{(0)});\mp@subsup{c}{pk}{(0)}\leftarrow\textrm{C.ZCm}(ck,\mp@subsup{\rho}{pk}{(0)}
        c
        \pi cert \leftarrow L.SmPrv (td, S', Verify (vk
        \pi
    Send (}(\mp@subsup{c}{pk}{(0)},\mp@subsup{c}{cert}{(0)},\mp@subsup{\pi}{cert}{(0)},\mp@subsup{c}{sn}{(0)},\mp@subsup{c}{M}{(0)},\mp@subsup{\pi}{sn}{(0)})\mathrm{ to }\mathcal{A

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-45.jpg?height=57&width=428&top_left_y=2015&top_left_x=545)
```

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-45.jpg?height=63&width=1017&top_left_y=2061&top_left_x=543)
```
    i
    \rhosn
```

$$
\begin{aligned}
& c_{\text {cert }}^{(1)} \leftarrow \mathrm{C} . \mathrm{ZCm}\left(c k, \rho_{\text {cert }}^{(1)}\right) \\
& c_{\mathrm{sn}}^{(1)} \leftarrow \operatorname{C.ZCm}\left(c k, \rho_{\mathrm{sn}}^{(1)}\right)
\end{aligned}
$$

```
$c_{p k}^{(1)} \leftarrow \operatorname{C.ZCm}\left(c k, \rho_{p k}^{(1)}\right)$
                    $c_{M}^{(1)} \leftarrow \operatorname{C.ZCm}\left(c k, \rho_{M}^{(1)}\right)$
                    $\pi_{c e r t}^{(1)} \leftarrow \mathrm{C} . \operatorname{SmPrv}\left(t d, \mathrm{~S}^{\prime} . \operatorname{Verify}\left(v k^{\prime}, \cdot, \cdot\right)=1, \rho_{p k}^{(1)}, \rho_{\text {cert }}^{(1)}\right)$
                    $\pi_{s n}^{(1)} \leftarrow \operatorname{C.SmPrv}_{\text {sn,init }}\left(t d, \rho_{p k}^{(1)}, \rho_{\mathrm{sn}}^{(1)}, \rho_{M}^{(1)}\right)$
    Send $\left(c_{p k}^{(1)}, c_{c e r t}^{(1)}, \pi_{c e r t}^{(1)}, c_{\mathrm{sn}}^{(1)}, c_{M}^{(1)}, \pi_{s n}^{(1)}\right)$ to $\mathcal{A}$
    Receive $\left(c_{\sigma}^{(1)}, \pi_{\sigma}^{(1)}\right)$ from $\mathcal{A}$
    If C.Verify $\left(c k\right.$, S.Verify $\left.(v k, \cdot \cdot \cdot)=1, c_{M}^{(1)}, c_{\sigma}^{(1)}, \pi_{\sigma}^{(1)}\right)=0$ then output $\perp$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-46.jpg?height=54&width=824&top_left_y=748&top_left_x=545)
```
    $\left(s n^{(1)}, M_{s n}^{(1)}\right) \leftarrow$ T.SGen $_{\text {init }}\left(\operatorname{sk}_{i_{0}}, n^{(1)}\right)$
    $\nu_{\mathrm{sn}}^{(1)}, \nu_{\mathrm{sn}}^{(0)} \stackrel{\mathcal{R}}{\leftarrow}$
    $\tilde{c}_{\mathrm{sn}}^{(0)} \leftarrow \operatorname{E} . \operatorname{Enc}\left(e k, s n^{(0)}, \nu_{s n}^{(0)}\right) ; \tilde{c}_{\mathrm{sn}}^{(1)} \leftarrow \mathrm{E} . \operatorname{Enc}\left(e k, s n^{(0)}, \nu_{\mathrm{sn}}^{(1)}\right)$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-46.jpg?height=57&width=540&top_left_y=953&top_left_x=543)
```
    $\tilde{\pi}_{s n}^{(0)} \leftarrow C . \operatorname{SmPrv}_{\text {enc }}\left(t d, e k, \rho_{s n}^{(0)}, \tilde{c}_{s n}^{(0)}\right)$
    Pick uniformly at random $\rho^{(\overrightarrow{0}) \prime}, \rho^{(\overrightarrow{1}) \prime}$ long enough to compute:
    $c_{1}^{(0)}=\left(\operatorname{Rand}\left(\left(c_{p k}^{(0)}, c_{c e r t}^{(0)}, \pi_{\text {cert }}^{(0)}, c_{\mathrm{sn}}^{(0)}, \pi_{\mathrm{sn}}^{(0)}, c_{M}^{(0)}, c_{\sigma}^{(0)}, \pi_{\sigma}^{(0)}, \tilde{c}_{\mathrm{sn}}^{(0)}, \tilde{\pi}_{\mathrm{sn}}^{(0)}\right), \rho^{(\overrightarrow{0}) \prime}\right)\right.$,
            $\left.n^{(0)}, s n^{(0)}, \rho_{s n}^{(0)}+\left(\rho^{(\overrightarrow{0})}\right)_{s n}, \rho_{p k}^{(0)}+\left(\rho^{(\overrightarrow{0})}\right)_{p k}\right)$
    $c_{1}^{(1)}=\left(\operatorname{Rand}\left(\left(c_{p k}^{(1)}, c_{c e r t}^{(1)}, \pi_{c e r t}^{(1)}, c_{s n}^{(1)}, \pi_{s n}^{(1)}, c_{M}^{(1)}, c_{\sigma}^{(1)}, \pi_{\sigma}^{(1)}, \tilde{c}_{\mathrm{sn}}^{(1)}, \tilde{\pi}_{s n}^{(1)}\right), \rho^{(1) \prime}\right)\right.$,
            $\left.n^{(1)}, s n^{(1)}, \rho_{s n}^{(1)}+\left(\rho^{(\overrightarrow{1}) \prime}\right)_{s n}, \rho_{p k}^{(1)}+\left(\rho^{(\overrightarrow{1}) \prime}\right)_{p k}\right)$
    $\mathcal{C L}[1] \leftarrow\left(1, i_{0}, c_{1}^{(0)}, 0, \mathcal{A}\right)$
    $\mathcal{C} L 2] \leftarrow\left(i_{1}, c_{1}^{(1)}, 0, \mathcal{A}\right)$
$\mathrm{S} \& \mathrm{R}_{\mathrm{ZK}, \mathrm{inv}}\left(j, i, s k_{1}, s k_{2}\right):$
    $c \leftarrow \mathcal{C L}[j] . c$
    $n^{\prime} \stackrel{\leftarrow}{\leftarrow} \mathcal{N} ; \rho_{s n}^{\prime}, \rho_{c e r t}^{\prime}, \rho_{p k}^{\prime}, \rho_{s n-p f}^{\prime}, \nu_{s n}^{\prime} \stackrel{\leftarrow}{\leftarrow} \mathcal{R}$; Compute:
        $\left(s n^{\prime}, s n-p f^{\prime}\right) \leftarrow \mathrm{T} . \mathrm{SGen}\left(p a r_{\mathrm{T}}, \overline{s k_{2}}, n^{\prime}\right)$
        $c_{\text {cert }}^{\prime}, c_{p k}^{\prime}, c_{s n}^{\prime}, c_{s n-p f}^{\prime} \leftarrow$ C.ZCm $\left(c k, \rho_{c e r t}^{\prime}, \rho_{p k}^{\prime}, \rho_{s n}^{\prime}, \rho_{s n-p f}^{\prime}\right)$
        $\tilde{c}_{\mathrm{sn}}^{\prime} \leftarrow \mathrm{E} . \operatorname{Enc}\left(e k, s n^{\prime}, \nu_{\mathrm{sn}}^{\prime}\right)$
        $\pi_{\text {cert }}^{\prime} \leftarrow \operatorname{C.SmPrv}\left(t d, \operatorname{S} . \operatorname{Verify}\left(v k^{\prime}, \cdot, \cdot\right)=1, \rho_{v k}^{\prime}, \rho_{p k}^{\prime}, \rho_{\text {cert }}^{\prime}\right)$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-46.jpg?height=57&width=783&top_left_y=1782&top_left_x=644)
```
        $\tilde{\pi}_{\mathrm{sn}}^{\prime} \leftarrow \mathrm{C}_{\text {. }} \mathrm{SmPrv}_{\mathrm{enc}}\left(t d, e k, \rho_{\mathrm{sn}}^{\prime}, \tilde{c}_{\mathrm{sn}}^{\prime}\right)$
    Parse $c$ as $\left(c^{0},\left(c^{j}=\left(c_{p k}^{j}, c_{\text {cert }}^{j}, \pi_{\text {cert }}^{j}, c_{s n}^{j}, \pi_{\mathrm{sn}}^{j}, c_{\text {tag }}^{j}, \pi_{\text {tag }}^{j}, \tilde{c}_{\mathrm{sn}}^{j}, \tilde{c}_{\text {tag }}^{j}, \tilde{\pi}_{\mathrm{sn}}^{j}, \tilde{\pi}_{\text {tag }}^{j}\right)\right)_{j=1}^{i}\right.$,
                                $\left.n, s n, \rho_{s n}, \rho_{p k}\right)$
    $\rho_{\text {tag }}, \nu_{\text {tag }}, \rho_{t-p f} \stackrel{\leftarrow}{\leftarrow}$
    $($ tag, $t-p f) \leftarrow \mathrm{T} . \mathrm{Gen}\left(p a \mathrm{~T}_{\mathrm{T}}, \sqrt{s k_{1}}, n, s n^{\prime}\right)$
    $c_{t a g} \leftarrow$ C.ZCm $\left(c k, \rho_{\text {tag }}\right)$
    $c_{\text {tag }} \leftarrow$ E.Enc $\left(e k\right.$, tag,$\left.\nu_{\text {tag }}\right)$
    $\pi_{\text {tag }} \leftarrow \operatorname{C.SmPrv}_{\text {tag }}\left(t d, p k_{\text {tag }}, s n, s n^{\prime}\right.$, tag $\left., t-p f, \rho_{p k}, \rho_{\mathrm{sn}}, \rho_{\mathrm{sn}}^{\prime}, \rho_{\text {tag }}, \rho_{t-p f}\right)$
    $\tilde{\pi}_{\text {tag }} \leftarrow \mathrm{C}_{\text {SmPrvenc }}\left(t d, e k, \rho_{\text {tag }}, \tilde{c}_{\text {tag }}\right)$
    Check $\operatorname{VER}_{\text {init }}\left(c^{0}\right) \wedge \bigwedge_{j=1}^{i} \operatorname{VER}_{\text {std }}\left(c^{j-1}, c^{j}\right) \wedge$
```

T.Verify $\left(c k, c_{p k}^{i}, c_{\mathrm{sn}}^{\prime}, c_{\text {tag }}, \pi_{\text {tag }}\right) \wedge$ C.Verify ${ }_{\text {enc }}\left(c k, e k, c_{\text {tag }}, \tilde{c}_{\text {tag }}, \tilde{\pi}_{\text {tag }}\right)$, if any of them rejects then output $\perp$

Else choose a sufficiently long vector of randomness $\overrightarrow{\rho^{\prime \prime}}$ to compute:

$$
\begin{aligned}
& c^{\prime \prime} \leftarrow \operatorname{Rand}\left(\left(c^{0},\left(c^{j}\right)_{j=1}^{i}, c_{p k}^{\prime}, c_{\text {cert }}^{\prime}, \pi_{c e r t}^{\prime}, c_{\mathrm{sn}}^{\prime}, \pi_{s n}^{\prime}, c_{\text {tag }}, \pi_{t a g}, \tilde{c}_{s n}^{\prime}, \tilde{\pi}_{s n}^{\prime}, \tilde{c}_{\text {tag }}^{\prime}, \tilde{\pi}_{\text {tag }}^{\prime}\right), \overrightarrow{\rho^{\prime \prime}}\right) \\
& c_{\text {new }}:=\left(c^{\prime \prime}, n^{\prime}, s n^{\prime}, \rho_{s n}^{\prime}+\left(\overrightarrow{\rho^{\prime \prime}}\right)_{s n^{\prime}}, \rho_{p k}^{\prime}+\left(\overrightarrow{\rho^{\prime \prime}}\right)_{p k^{\prime}}\right) \\
& \mathcal{C L}[|\mathcal{C L}|+1]:=\left(i, c_{\text {new }}, 0, j\right)
\end{aligned}
$$

We define a new game for all $l \in\{0, \ldots, k-1\}$ :

$$
\begin{aligned}
& \text { Experiment } \operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2, l}^{\mathrm{c}}(\lambda): \\
& G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right) \\
& p a r_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r) \\
& \operatorname{par}_{\mathrm{S}} \leftarrow \mathrm{S} . \operatorname{Setup}(G r) \\
& \operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r) \\
& (c k, t d) \leftarrow \mathrm{C}^{\prime} \operatorname{SmSetup}(G r) \\
& p a r \leftarrow\left(1^{\lambda}, p a r_{\mathrm{S}}, p a r_{\mathrm{S}^{\prime}}, p a r_{\mathrm{T}}, c k\right) \\
& p_{\mathcal{B}} \leftarrow \mathcal{A}(p a r) \\
& \mid\left(i_{0}, i_{1}\right) \leftarrow \text { DoubleUWith }_{\mathrm{rev}}^{\mathcal{A}} \\
& \left(i^{(\overrightarrow{0})}, i^{(\overrightarrow{1})}\right) \leftarrow \mathcal{A}^{\text {URegist,Spy }}
\end{aligned}
$$

Let $k:=\left|i^{(\overrightarrow{0})}\right| ;$ if $k \neq\left|i^{(\overrightarrow{1})}\right|$, abort the entire procedure

Consider $i_{0}$ as $\left(i^{(\overrightarrow{0})}\right)_{0}$, and $i_{1}$ as $\left(i^{(\overrightarrow{1})}\right)_{0}$

For all $b, j: \operatorname{sk}_{j}^{(b)} \leftarrow \mathcal{U} \mathcal{L}\left[\left(i^{(\vec{b})}\right)_{j}\right]$.sk

Repeat the following for $j=1, \ldots, l$ :

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-47.jpg?height=92&width=678&top_left_y=1499&top_left_x=642)

$$
\begin{aligned}
& \text { Run S\&R } \mathrm{R}_{\mathrm{ZK}, \text { inv }}\left(2 j,\left(i^{(\overrightarrow{1})}\right)_{j}, s k_{j-1}^{(0)}, s k_{j}^{(0)}\right) \\
& \text { Run S\&R } \mathrm{R}_{\mathrm{ZK}, \mathrm{inv}}\left(2 l+1,\left(i^{(\overrightarrow{0})}\right)_{l+1}, s k_{l}^{(1)}, s k_{l+1}^{(0)}\right) \\
& \text { Run S\&R } \mathrm{R}_{\mathrm{ZK}, \mathrm{inv}}\left(2 l+2,\left(i^{(\overrightarrow{1})}\right)_{l+1}, s k_{l}^{(0)}, s k_{l+1}^{(1)}\right)
\end{aligned}
$$

Proposition 25. $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2}^{\mathrm{c}-\mathrm{an}}(\lambda)$ and $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2,0}^{\mathrm{c} \text { an }}(\lambda)$ are $2 \epsilon_{\mathrm{t}-\mathrm{an}}$-statistically close.

We receive a challenge $p a r_{\mathrm{T}}$ in the tag-anon game for T (and not the tag exculpability game, in contrast to the proof of Theorem 20), and we use $p a r_{\mathrm{T}}$ as parameter for the tags instead of generating it in $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2}^{\mathrm{C}}$.

In DoubleUWith, we send to the tag-anon-challenger the secret keys of $i_{0}$ and $i_{1}$, and we use $O_{1}$ to generate the serial number of $i_{0}$ in DoubleUWith and $O_{2}(0)$ to generate the corresponding tag in the first $\mathrm{S} \& \mathrm{R}_{\mathrm{ZK}}{ }^{4}$.

If the challenger was in mode 0 , this will not change the experiment. But if the challenger was in mode 1 , it will replace $i_{0}$ by $i_{1}$. Let $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2,-1}^{\mathrm{c} \text {-an }}$ denote the game corresponding to this swap and $\Delta$ be the statistical distance.

We have just proved that

$$
\Delta\left(\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2}^{\mathrm{c}-\mathrm{an}}(\lambda), \operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2,-1}^{\mathrm{c}-\mathrm{an}}(\lambda)\right) \leq \epsilon_{\mathrm{t-an}}
$$

Analogously, we replace $i_{1}$ by $i_{0}$, i.e., we show that

$$
\Delta\left(\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2,-1}^{\mathrm{c}-\mathrm{an}}(\lambda), \operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2,0}^{\mathrm{c}-\mathrm{an}}(\lambda)\right) \leq \epsilon_{\mathrm{t} \text {-an }}
$$

and therefore $\Delta\left(\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2}^{\mathrm{c}-\mathrm{an}}(\lambda), \operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2,0}^{\mathrm{c}-\mathrm{an}}(\lambda)\right) \leq 2 \epsilon_{\mathrm{t} \text {-an }}$.

The proof is completely analogous for the following property, which lets us swap multiple games.

Proposition 26. For all $l \in\{0, \ldots, k-2\}$, we have that $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2, l}^{\mathrm{c}-\mathrm{l}}(\lambda)$ and $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2, l+1}^{\mathrm{c}-\mathrm{an}}(\lambda)$ are $2 \epsilon_{\mathrm{t} \text {-an }}$-statistically close.

Finally, we define a last oracle to swap the last keys (and the corresponding game):

$$
\begin{aligned}
& \operatorname{Spd}_{\mathrm{ZK} \text {.inv }}\left(k, s k_{1}\right): \\
& \text { Receive }\left(s n^{\prime}, \rho_{s n}^{\prime}\right) \text { from } \mathcal{A} \\
& c \leftarrow \mathcal{C} \mathcal{L}[k] . c \\
& \text { Parse } c \text { as }\left(c^{0},\left(c^{j}=\left(c_{p k}^{j}, c_{\text {cert }}^{j}, \pi_{\text {cert }}^{j}, c_{\mathrm{sn}}^{j}, \pi_{\mathrm{sn}}^{j}, c_{\text {tag }}^{j}, \pi_{\text {tag }}^{j}, \tilde{c}_{\mathrm{sn}}^{j}, \tilde{c}_{\text {tag }}^{j}, \tilde{\pi}_{\mathrm{sn}}^{j}, \tilde{\pi}_{\text {tag }}^{j}\right)\right)_{j=1}^{i}\right. \\
& \rho_{\text {tag }}, \nu_{\text {tag }}, \rho_{t-p f} \stackrel{\$}{\leftarrow} \\
& (\operatorname{tag}, t-p f) \leftarrow \mathrm{T} . \operatorname{Gen}\left(p a r_{\mathrm{T}}, \overline{s k_{1}}, n, s n^{\prime}\right) \\
& c_{\text {tag }} \leftarrow \mathrm{C} . \mathrm{ZCm}\left(c k, \rho_{\text {tag }}\right) \\
& \tilde{c}_{\text {tag }} \leftarrow \mathrm{E} . \mathrm{Enc}\left(e k, \text { tag }, \nu_{\text {tag }}\right) \\
& \pi_{\text {tag }} \leftarrow \mathrm{C}_{\mathrm{Sm}} \operatorname{Smrv}_{\text {tag }}\left(t d, p k_{\text {tag }}, s n, \mathrm{sn}{ }^{\prime}, \operatorname{tag}, t-p f, \rho_{p k}, \rho_{\mathrm{sn}}, \rho_{\mathrm{sn}}^{\prime}, \rho_{\text {tag }}, \rho_{t-p f}\right) \\
& \tilde{\pi}_{\text {tag }} \leftarrow \text { C.SmPrvenc }\left(t d, e k, \rho_{\text {tag }}, \tilde{c}_{\text {tag }}\right) \\
& \text { Send }\left(c^{0},\left(c^{j}\right)_{j=1}^{i}, c_{\text {tag }}, \pi_{\text {tag }}, \tilde{c}_{\text {tag }}, \tilde{\pi}_{\text {tag }}\right) \text { to } \mathcal{A}
\end{aligned}
$$

Experiment $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2, \mathrm{k}}^{\mathrm{c}-\mathrm{an}}(\lambda)$ :

$$
\begin{aligned}
& G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right) \\
& \operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r) \\
& \operatorname{par}_{\mathrm{S}} \leftarrow \mathrm{S} . \operatorname{Setup}(G r) \\
& \operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r) \\
& (c k, t d) \leftarrow \operatorname{C.SmSetup}(G r)
\end{aligned}
$$
\footnotetext{
${ }^{4}$ We use the oracle only in these step; for the other serial number and tag generations, we use the secret keys (which we have generated) like in $\operatorname{Expt}_{\mathcal{A}, 0, Z K V 2}^{\mathrm{c}-a n}$.
}

$$
\begin{aligned}
& \operatorname{par} \leftarrow\left(1^{\lambda}, \operatorname{par}_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right) \\
& p k_{\mathcal{B}} \leftarrow \mathcal{A}(p a r) \\
& \left(i_{0}, i_{1}\right) \leftarrow \text { DoubleUWith } \mathrm{rev}_{\mathcal{A}}^{\mathcal{A}} \\
& \left(i^{(\overrightarrow{0})}, i^{(\overrightarrow{1})}\right) \leftarrow \mathcal{A}^{\text {URegist,Spy }} \\
& \text { Let } k:=\left|i^{(\overrightarrow{0})}\right| ; \quad \text { if } k \neq\left|i^{(\overrightarrow{1})}\right| \text {, abort the entire procedure } \\
& \text { Consider } i_{0} \text { as }\left(i^{(\overrightarrow{0})}\right)_{0} \text {, and } i_{1} \text { as }\left(i^{(\overrightarrow{1})}\right)_{0} \\
& \text { For all } b, j: s k_{j}^{(b)} \leftarrow \mathcal{U} \mathcal{L}\left[\left(i^{(\vec{b})}\right)_{j}\right] . s k \\
& \text { Then repeat the following for } j=1, \ldots, \overline{\mathrm{k}} \text { : }
\end{aligned}
$$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-49.jpg?height=114&width=1028&top_left_y=772&top_left_x=554)

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-49.jpg?height=143&width=767&top_left_y=888&top_left_x=554)

$$
\begin{aligned}
& b^{*} \leftarrow \mathcal{A} ; \text { return } b^{*}
\end{aligned}
$$

Analogously to previous two propositions, we get:

Proposition 27. $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2, k-1}^{\mathrm{c}-\mathrm{an}}(\lambda)$ and $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZKV} 2, k}^{\mathrm{c}-\mathrm{an}}(\lambda)$ are $2 \epsilon_{\mathrm{t}-\mathrm{an}}$-statistically close.

By noting that two randomized commitments of the same type in the hidingmode have the (exact) same distribution, we get that $\operatorname{Expt}_{\mathcal{A}, 0, Z K V 2, k}^{\mathrm{c}-\mathrm{an}}(\lambda)$ is equally distributed as $\operatorname{Expt}_{\mathcal{A}, 1, \mathrm{ZK}}^{\mathrm{c}-\mathrm{an}}(\lambda)$.

From a similar reasoning, we get that $\operatorname{Expt}_{\mathcal{A}, 1, \mathrm{ZK}}^{\mathrm{c}-\mathrm{an}}(\lambda)$ is $\epsilon_{\mathrm{m} \text {-ind }}$ statistically close to $\operatorname{Expt}_{\mathcal{A}, 1}^{\mathrm{c}-\mathrm{an}}(\lambda)$. Finally, we deduce that $\operatorname{Expt}_{\mathcal{A}, 1}^{\mathrm{c} \text { an }}(\lambda)$ is $2\left(\epsilon_{\mathrm{ZK}}+(k+1) \epsilon_{\mathrm{t} \text { tan }}\right)$ statistically-close to $\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{c}-\mathrm{an}}(\lambda)$.

Note that $\epsilon_{\mathrm{t} \text {-an }}$ is the advantage against tag-anonymity of an adversary that is making just one call to $O_{1}$ and one to $O_{2}$.

\section{A. 4 Coin transparency}

Theorem 28. Let $\mathcal{A}$ be an adversary against coin-transparency (c-tr) of our scheme with advantage $\epsilon$, and let $\ell$ be the size of the challenge coins, and $k$ be an upper-bound on the number of users transferring the challenge coins. Then there exist adversaries against mode-indistinguishability of C , tag-anonymity of T , IACR-security of E and RCCA-security of $\mathrm{E}^{\prime}$ with advantages $\epsilon_{\mathrm{m} \text {-ind }}, \epsilon_{\mathrm{t}-\mathrm{an}}, \epsilon_{\mathrm{iacr}}$ and $\epsilon_{\text {rcca }}$, resp., such that

$$
\epsilon \leq 2 \epsilon_{\mathrm{m}-\mathrm{ind}}+(k+1) \epsilon_{\mathrm{t}-\mathrm{an}}+(2 \ell+1) \epsilon_{\mathrm{iacr}}+\epsilon_{\mathrm{rcca}}
$$

The proof proceeds via an hybrid argument. We first recall game $\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{c}-\mathrm{tr}}$.

Experiment $\operatorname{Expt}_{\mathcal{A}, 0}^{c-\operatorname{tr}}(\lambda)$ :

$$
\operatorname{par} \leftarrow \operatorname{ParamGen}\left(1^{\lambda}\right) ;\left(s k_{\mathcal{B}}, p k_{\mathcal{B}}\right) \leftarrow \operatorname{BKeyGen}(p a r)
$$

```
$\mathcal{D C L} \mathcal{L}^{\prime}:=\emptyset ; \operatorname{ctr} \leftarrow 0$
$i_{0} \leftarrow \mathcal{A}^{\text {URegist,BDepo}, \mathrm{Spy}}\left(\right.$ par, $\left.p k_{\mathcal{B}}, \mathrm{sk}_{\mathcal{W}}, \mathrm{sk}_{\mathcal{D}}\right)$
$\operatorname{Run} \operatorname{Rcv}\left(i_{0}\right)$ with $\mathcal{A}$
Let $c_{0}$ be the coin received
$x_{0} \leftarrow \operatorname{CheckDS}\left(\mathrm{sk}_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{0}\right)$
If $x_{0}=\perp$ then $c t r \leftarrow c t r+1$
$\mathcal{D C} \mathcal{L}^{\prime} \leftarrow$ CheckDS $\left(\mathrm{sk}_{\mathcal{C}}, \emptyset, \emptyset, c_{0}\right)$
$i_{1} \leftarrow \mathcal{A}^{\text {URegist,BDepo',Spy }}$
$\operatorname{Run} \operatorname{Rcv}\left(i_{1}\right)$ with $\mathcal{A}$
Let $c_{1}$ be the coin received
$x_{1} \leftarrow$ CheckDS $\left(s_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{1}\right)$
If $x_{1}=\perp$ then $c t r \leftarrow c t r+1$
If $\operatorname{comp}\left(c_{0}, c_{1}\right) \neq 1$ then return 0
$x_{2} \leftarrow$ CheckDS $\left(s_{\mathcal{C K}}, \emptyset, \mathcal{D C} \mathcal{L}^{\prime}, c_{1}\right)$
If $x_{2} \neq \perp$ then $\mathcal{D C L ^ { \prime }} \leftarrow x_{2}$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-50.jpg?height=54&width=437&top_left_y=1027&top_left_x=557)
```
Let $k:=\left|\vec{i}^{(0)}\right|$; If $k \neq\left|\vec{i}^{(1)}\right|$ then return 0
If $k \neq 0$ then run $\mathrm{S} \& \mathrm{R}\left(1,\left(\vec{i}^{(0)}\right)_{1}\right)$
For $j=2, \ldots, k$ :
    Run $\mathrm{S} \& \mathrm{R}\left(j+1,\left(\vec{i}^{(0)}\right)_{j}\right)$
$\operatorname{Run} \operatorname{Spd}(k+2)$ with $\mathcal{A}$
$b^{*} \leftarrow \mathcal{A}^{\text {BDepó }}$; return $b^{*} \quad / /$ instead of CheckDS* , BDepo
                                    uses CheckDS' $\left(\cdot, \cdot, \cdot, \cdot, \mathcal{D C} \mathcal{L}^{\prime}\right)$ defined as follows:
CheckDS' $\left(s k_{\mathcal{C K}}, \mathcal{U L}, \mathcal{D C L}, c, \mathcal{D C} \mathcal{L}^{\prime}\right):$
    $x \leftarrow$ CheckDS $\left(s_{\mathcal{C K}}, \emptyset, \mathcal{D C} \mathcal{L}^{\prime}, c\right)$
    If $x=\perp$ :
                                    $c t r \leftarrow c \operatorname{tr}+1$
                                    If $c t r>1$ then return 0
Return CheckDS $\left(s_{\mathcal{C K}}, \emptyset, \mathcal{D C L}, c\right)$
```

Note that we are only interested in detecting double spending, and not tracing the cheater (because CheckDS always run on an empty user list, which in our instantiation implies that it will never accuse someone and will output $\perp$ when it detects a double-spending). We can therefore simplify CheckDS as follows (and the distribution of the output of the experiment will be unchanged):

$$
\begin{aligned}
& \text { CheckDS } \\
& \quad s n \leftarrow E^{\prime} \text {. Dec }\left(d k_{\text {inint }}, \tilde{c}_{\text {s. }}^{0}\right) \\
& \text { If sn } \in \mathcal{D C L} \text { then return } \perp \\
& \text { Else } \mathcal{D C L}:=\mathcal{D C L} \cup\{s n\} ; \text { return } \mathcal{D C L}
\end{aligned}
$$

Let CheckDS ssimple respectively, that use CheckDS simple instead of CheckDS. The beginning of the proof will be very similar to the one of coin-anonymity.

Note that we choose to only keep the initial serial numbers in $\mathcal{D C} \mathcal{L}$, since in this game we only check if there is double-spending or not (in particular, we do not send any proof of culpability). Thus only the first serial-number component of a coin matters, and it will not change in the following game.

Using the same arguments as in Sect. A.3, we get the following.

Proposition 29. $\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{c}-\mathrm{tr}}(\lambda)$ and $\operatorname{Expt}_{\mathcal{A}, 0, Z K}^{\mathrm{c}-\mathrm{tr}}(\lambda)$ are $\left(\epsilon_{\mathrm{m} \text {-ind }}+t \epsilon_{\mathrm{t} \text { tan }}\right)$ statistically close.

```
Experiment $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZK}}^{\mathrm{c}-\operatorname{tr}}(\lambda)$ :
    $G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right)$
    $\operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r)$
    $\operatorname{par}_{\mathrm{S}} \leftarrow$ S.Setup $(G r)$
    $\operatorname{ar}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r)$
    $(c k, t d) \leftarrow$ C.SmSetup $(G r)$
    par $\leftarrow\left(1^{\lambda}, \operatorname{par}_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$
    $\left(p k_{\mathcal{B}}, s k_{\mathcal{B}}\right) \leftarrow$ BKeyGen
    $\mathcal{D C} \mathcal{L}^{\prime}:=\emptyset ; \operatorname{ctr} \leftarrow 0$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-51.jpg?height=54&width=699&top_left_y=1244&top_left_x=553)
```
    Run $\operatorname{Rcv}_{\mathrm{ZK}}\left(i_{0}\right)$ with $\mathcal{A}$; let $c_{0}$ be the received coin
    $x_{0} \leftarrow \operatorname{CheckDS}_{\text {simple }}\left(\operatorname{sk}_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{0}\right)$
    If $x_{0}=\perp$ then $c t r \leftarrow c t r+1$
    $\mathcal{D C} \mathcal{L}^{\prime} \leftarrow$ CheckDS $_{\text {simple }}\left(s k_{\mathcal{C K}}, \emptyset, \emptyset, c_{0}\right)$
    $i_{1} \leftarrow \mathcal{A}^{\text {URegist,BDeposimple }}$, Spy
    Run $\operatorname{Rcv}_{\mathrm{ZK}}\left(i_{1}\right)$ with $\mathcal{A}$; let $c_{1}$ be the received coin
    $x_{1} \leftarrow \operatorname{CheckDS}_{\text {simple }}\left(\operatorname{sk}_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{1}\right)$
    If $x_{1}=\perp$ then $c t r \leftarrow c t r+1$
    If $\operatorname{comp}\left(c_{0}, c_{1}\right) \neq 1$ then return 0
    $x_{2} \leftarrow \operatorname{CheckDS}_{\text {simple }}\left(\operatorname{sk}_{\mathcal{C K}}, \emptyset, \mathcal{D C} \mathcal{L}^{\prime}, c_{1}\right)$
    If $x_{2} \neq \perp$ then $\mathcal{D C} \mathcal{L}^{\prime} \leftarrow x_{2}$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-51.jpg?height=46&width=499&top_left_y=1771&top_left_x=558)
```
    Let $k$ be the size of $\vec{i}(0)$
    If $k$ is different of the size of $\vec{i}(1)$ : return 0
    If $k \neq 0$, then run $\mathrm{S} \& \mathrm{R}_{\mathrm{ZK}}\left(1,\left(\overline{i^{(1)}}\right)_{1}\right)$
    Then repeat the following for $j=3, \ldots,(k+1)$ :
    $\operatorname{Run} \mathrm{S} \& \mathrm{R}_{\mathrm{ZK}}\left(j,\left(\overline{i^{(1)}}\right)_{j-1}\right)$
    Run $\operatorname{Spd}_{\mathrm{ZK}}(k+1)$ with $\mathcal{A}$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-51.jpg?height=63&width=1019&top_left_y=2134&top_left_x=542)

Now we can leverage zero-knowledge and randomizability to partially "remelt (all the commits and proofs in) $c_{0}$. The strategy is the following: Using Extract,
defined below, we "break" $c_{0}$ and $c_{1}$ to extract all the relevant information (serial numbers, tags and nonce). Using Remelt we remelt both coins, after switching the following content:
- In $\operatorname{Expt}_{\mathcal{A}, 0, \text { iacr }}^{\mathrm{c}-\mathrm{tr}}$, we switch all the tags, and serial numbers (except the first serial number).
- In $\operatorname{Expt}_{\mathcal{A}, 0, \text { rcca1 }}^{\mathrm{c}-\mathrm{tr}}$, we switch the first serial numbers of both coins.
- In $\operatorname{Expt}_{\mathcal{A}, 0, \text { final }}^{\mathrm{c}-\mathrm{tr}}$, we switch the nonces.

Thus we define the two followings procedures:

$\operatorname{Extract}\left(\operatorname{sk}_{\mathcal{C}}, c\right)$ :

Parse $c$ as:

$$
\left(c^{0},\left(c^{k}=\left(c_{p k}^{k}, c_{c e r t}^{k}, \pi_{c e r t}^{k}, c_{s n}^{k}, \pi_{s n}^{k}, c_{t a g}^{k}, \pi_{t a g}^{k}, \tilde{c}_{\mathrm{sn}}^{k}, \tilde{c}_{t a g}^{k}, \tilde{\pi}_{s n}^{k}, \tilde{\pi}_{t a g}^{k}\right)\right)_{k=1}^{\ell}\right.
$$

$$
\begin{aligned}
& s n_{0}=\mathrm{E}^{\prime} . \operatorname{Dec}\left(d k_{\mathrm{init}}, \tilde{c}_{\mathrm{sn}}^{0}\right) \\
& \text { Return }\left(s n_{0}, n,\left(\tilde{c}_{\mathrm{sn}}^{1}, \ldots, \tilde{c}_{\mathrm{sn}}^{\ell}\right),\left(\tilde{c}_{\text {tag }}^{1}, \ldots, \tilde{c}_{\text {tag }}^{\ell}\right)\right)
\end{aligned}
$$

$\operatorname{Remelt}\left(t d, \tilde{c}_{\mathrm{sn}}, n,\left(\tilde{c}_{\mathrm{sn}}^{1}, \ldots, \tilde{c}_{\mathrm{sn}}^{\ell}\right),\left(\tilde{c}_{\text {tag }}^{1}, \ldots, \tilde{c}_{\text {tag }}^{\ell}\right)\right)$ :

$$
\rho_{\mathrm{sn}}^{0}, \rho_{c e r t}^{0}, \rho_{p k}^{0}, \rho_{M}^{0}, \rho_{\sigma}^{0} \stackrel{\$}{\leftarrow}
$$

$$
\begin{aligned}
& c_{s n}, c_{c e r t}, c_{p k}, c_{M}, c_{\sigma} \leftarrow \operatorname{C.ZCm}\left(c k, \rho_{\mathrm{sn}}^{0}, \rho_{c e r t}^{0}, \rho_{p k}^{0}, \rho_{M}^{0}, \rho_{\sigma}^{0}\right) \\
& \pi_{c e r t} \leftarrow \operatorname{C.SmPrv}\left(t d, \mathrm{~S}^{\prime} . \operatorname{Verify}\left(v k^{\prime}, \cdot, \cdot\right)=1, \rho_{p k}^{0}, \rho_{\text {cert }}^{0}\right) \\
& \pi_{\sigma} \leftarrow \operatorname{C.SmPrv}\left(t d, \operatorname{SiVerify}(v k, \cdot, \cdot)=1, \rho_{M}^{0}, \rho_{\sigma}^{0}\right) \\
& \pi_{s n} \leftarrow \operatorname{CiSmPrv}_{s n}, \mathrm{init}\left(t d, \rho_{p k}^{0}, \rho_{s n}^{0}, \rho_{M}^{0}\right) \\
& \tilde{\pi}_{\mathrm{sn}} \leftarrow \operatorname{C.SmPrv}_{\mathrm{enc}}\left(t d, e k^{\prime}, \rho_{\mathrm{sn}}^{0}, \tilde{c}_{s n}^{0}\right) \\
& c^{0} \leftarrow\left(c_{p k}, c_{c e r t}, \pi_{c e r t}, c_{s n}, \pi_{s n}, c_{M}, c_{\sigma}, \pi_{\sigma}, \tilde{c}_{s n}, \tilde{\pi}_{s n}\right) \\
& \text { For } k \in\{1, \ldots, \ell\}:
\end{aligned}
$$

$$
\rho_{s n}^{k}, \rho_{\text {cert }}^{k}, \rho_{p k}^{k}, \rho_{\text {tag }}^{k}, \stackrel{\Phi}{\leftarrow} \mathcal{R}
$$

$$
\begin{aligned}
& c_{\mathrm{sn}}^{k}, c_{t a g}^{k}, c_{p k}^{k}, c_{M}^{k}, c_{\sigma}^{k} \leftarrow \operatorname{C.ZCm}\left(c k, \rho_{\mathrm{sn}}^{k}, \rho_{t a g}^{k}, \rho_{p k}^{k}, \rho_{M}^{k}, \rho_{\sigma}^{k}\right) \\
& \pi_{c e r t}^{k} \leftarrow \text { C.SmPrv }\left(t d, \mathrm{~S}^{\prime} \text {.Verify }\left(v k^{\prime}, \cdot, \cdot\right)=1, \rho_{p k}^{k}, \rho_{c e r t}^{k}\right) \\
& \pi_{\mathrm{sn}}^{k} \leftarrow \operatorname{C.SmPrv}_{\mathrm{sn}}\left(t d, \rho_{p k}^{k}, \rho_{\mathrm{sn}}^{k}\right) \\
& \tilde{\pi}_{\mathrm{sn}}^{k} \leftarrow \operatorname{C.SmPrv} \mathrm{S}_{\mathrm{enc}}\left(t d, e k, \rho_{\mathrm{sn}}^{k}, \tilde{c}_{\mathrm{sn}}^{k}\right) \\
& \pi_{t a g}^{k} \leftarrow \operatorname{CoSmPrv}_{s n}\left(t d, \rho_{p k}^{k}, \rho_{s n}^{k-1}, \rho_{s n}^{k}, \rho_{t a g}^{k}\right) \\
& \tilde{\pi}_{t a g}^{k} \leftarrow \operatorname{CoSmPrv}_{\mathrm{enc}}\left(t d, e k, \rho_{t a g}^{k}, \tilde{c}_{t a g}^{k}\right) \\
& c^{k} \leftarrow\left(c_{p k}^{k}, c_{c e r t}^{k}, \pi_{c e r t}^{k}, c_{\mathrm{sn}}^{k}, \pi_{s n}^{k}, \tilde{c}_{s n}^{k}, \tilde{\pi}_{s n}^{k}, \tilde{c}_{s n}^{k}, \pi_{t a g}^{k}, \tilde{\pi}_{t a g}^{k}\right)
\end{aligned}
$$

Return $\left(\left(c^{k}\right)_{k=0}^{\ell}, n, s n, \rho_{\mathrm{sn}}, \rho_{p k}\right)$

By the zero-knowledge property of $\mathbf{C}$, the outputs of $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZK}}^{\mathrm{c} \text { tr }}(\lambda)$ and

$\operatorname{Expt}_{\mathcal{A}, 0, \text { remelt }}^{\mathrm{c}-\mathrm{tr}}(\lambda)$ will follow perfectly the same distribution:

Proposition 30. $\operatorname{Expt}_{\mathcal{A}, 0, \text { remelt }}^{\mathrm{c}-\mathrm{tr}}(\lambda)$ and $\operatorname{Expt}_{\mathcal{A}, 0, \mathrm{ZK}}^{\mathrm{c}-\mathrm{tr}}(\lambda)$ are equally distributed.

Experiment $\operatorname{Expt}_{\mathcal{A}, 0, \text { remelt }}^{c-\operatorname{tr}}(\lambda)$ :

$$
\begin{aligned}
& G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right) \\
& \operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r) ; \operatorname{par}_{\mathrm{S}} \leftarrow \mathrm{S} . \operatorname{Setup}(G r) ; \operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r) \\
& (c k, t d) \leftarrow \operatorname{C.SmSetup}(G r)
\end{aligned}
$$

```
$\operatorname{par} \leftarrow\left(1^{\lambda}, \operatorname{par}_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$
$\left(p k_{\mathcal{B}}, s k_{\mathcal{B}}\right) \leftarrow$ BKeyGen ()
$\mathcal{D C} \mathcal{L}^{\prime}:=\emptyset ;$ ctr $\leftarrow 0$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-53.jpg?height=57&width=697&top_left_y=524&top_left_x=554)
```
Run $\operatorname{Rcv}_{\mathrm{ZK}}\left(i_{0}\right)$ with $\mathcal{A}$; let $c_{0}$ be the received coin
$x_{0} \leftarrow \operatorname{CheckDS}_{\text {simple }}\left(\operatorname{sk}_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{0}\right)$
If $x_{0}=\perp$ then $c t r \leftarrow c t r+1$
$i_{1} \leftarrow \mathcal{A}^{\text {URegist,BDepo',Spy }}$
$\operatorname{Run} \operatorname{RcvZK}\left(i_{1}\right)$ with $\mathcal{A}$; let $c_{1}$ be the received coin
$x_{1} \leftarrow \operatorname{CheckDS}_{\text {simple }}\left(\operatorname{sk}_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{1}\right)$
If $x_{1}=\perp$ then $c t r \leftarrow c t r+1$
If $\operatorname{comp}\left(c_{0}, c_{1}\right) \neq 1$ then abort

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-53.jpg?height=433&width=642&top_left_y=911&top_left_x=557)
```
Let $k$ be the size of $\overrightarrow{i^{(0)}}$
If $k$ is different of the size of $\vec{i}^{(1)}$ : return 0
$\mathcal{C} \mathcal{L}[1] . c \leftarrow c_{0}^{\prime}$
Run S\&R $\mathrm{ZKK}\left(1,\left(\vec{i}^{(1)}\right)_{1}\right)$
Then repeat the following for $j=3, \ldots,(k+1)$ :
                    Run S\&R $\mathrm{R}_{\mathrm{ZK}}\left(j,\left(\vec{i}^{(1)}\right)_{j-1}\right)$
Run $\operatorname{Spd}_{\mathrm{ZK}}(k+1)$ with $\mathcal{A}$
$b^{*} \leftarrow \mathcal{A}^{\text {BDeposimple }} ;$ return $b^{*}$
```

The serial numbers and tags are encrypted. We can change the ciphertexts encrypted with E in Extract; each switch will affect the distribution of the output of the overall experiment with probability at most $\epsilon_{\mathrm{iacr}}$. We deduce the following:

Proposition 31. $\operatorname{Expt}_{\mathcal{A}, 0, \text { remelt }}^{\mathrm{c}-\mathrm{rr}}(\lambda)$ and $\operatorname{Expt}_{\mathcal{A}, 0, \text { iacr }}^{\mathrm{c}-\mathrm{rr}}(\lambda)$ are $2 \ell$-statistically close.

```
Experiment Expt 
    Gr}\leftarrow\operatorname{GrGen(1 (
    par

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-53.jpg?height=43&width=393&top_left_y=2101&top_left_x=557)
```

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-53.jpg?height=52&width=491&top_left_y=2137&top_left_x=546)
```

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-53.jpg?height=46&width=350&top_left_y=2183&top_left_x=557)
```

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-53.jpg?height=44&width=290&top_left_y=2225&top_left_x=554)
```
    iof {
```

Run $\operatorname{Rcv}\left(i_{0}\right)$ with $\mathcal{A}$; let $c_{0}$ be the received coin

$x_{0} \leftarrow$ CheckDS $_{\text {simple }}\left(s k_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{0}\right)$

If $x_{0}=\perp$ then $c t r \leftarrow c t r+1$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-54.jpg?height=54&width=349&top_left_y=523&top_left_x=541)

Run $\operatorname{Rcv}\left(i_{1}\right)$ with $\mathcal{A}$; let $c_{1}$ be the received coin

$x_{1} \leftarrow$ CheckDS $_{\text {simple }}\left(s k_{\mathcal{C K}}, \emptyset, \mathcal{C} \mathcal{L}, c_{1}\right)$

If $x_{1}=\perp$ then $c t r \leftarrow c t r+1$

If $\operatorname{comp}\left(c_{0}, c_{1}\right) \neq 1$ abort the entire procedure

$s n_{0}, n^{(0)}, \overrightarrow{\tilde{c}}_{\text {sn }, 0}, \overrightarrow{\tilde{c}}_{\text {tag }, 0} \leftarrow \operatorname{Extract}\left(\operatorname{sk}_{\mathcal{C K}}, c_{0}\right)$

$s n_{1}, n^{(1)}, \overrightarrow{\tilde{c}}_{\text {sn }, 1}, \overrightarrow{\tilde{c}}_{\text {tag }, 1} \leftarrow \operatorname{Extract}\left(\operatorname{sk}_{\mathcal{C K}}, c_{1}\right)$

$\nu \stackrel{\$}{\leftarrow}$

$\tilde{c}_{\text {sn }} \leftarrow \mathrm{E}^{\prime} . \operatorname{Enc}\left(e k^{\prime}, s n_{0}, \nu\right)$

$c_{0}^{\prime} \leftarrow \operatorname{Remelt}\left(t d, \tilde{c}_{\mathrm{sn}}, n^{(0)}, \overrightarrow{\tilde{c}}_{\mathrm{sn}, 1}, \overrightarrow{\tilde{c}}_{\text {tag }, 1}\right.$

$\mathcal{D C} \mathcal{L}^{\prime}:=\left\{s n_{0}, s n_{1}\right\}$

$\left(\vec{i}^{(0)}, \vec{i}^{(1)}\right) \leftarrow \mathcal{A}^{\text {URegist, BDepo }}$ simple, Spy

Let $k$ be the size of $\vec{i}^{(0)}$

If $k$ is different of the size of $\vec{i}^{(1)}$ abort

$\mathcal{C L}[1] . c \leftarrow c_{0}^{\prime}$

Run $\mathrm{S} \& \mathrm{R}_{\mathrm{ZK}}\left(1,\left(\vec{i}^{(1)}\right)_{1}\right)$

Then repeat the following step for $j=3, \ldots,(k+1)$ :

Run S\&R $\mathrm{RKK}_{\mathrm{ZK}}\left(j,\left(\vec{i}^{(1)}\right)_{j-1}\right)$

Run $\operatorname{Spd}_{\mathrm{ZK}}(k+1)$ with $\mathcal{A}$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-54.jpg?height=46&width=428&top_left_y=1386&top_left_x=545)

For the next step, we will rely on RCCA-security of $E^{\prime}$.

```
Experiment $\operatorname{Expt}_{\mathcal{A}, 0, \text { rcca }}^{\mathrm{c}-\mathrm{tr}}\left(G r, e k^{\prime}\right)$ :
    $\operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T}$.Setup $(G r) ; p \operatorname{ar}_{\mathrm{S}} \leftarrow \mathrm{S} . \operatorname{Setup}(G r) ; p \operatorname{Sr}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r)$
    $(c k, t d) \leftarrow$ C.SmSetup $(G r)$
    $\operatorname{par} \leftarrow\left(1^{\lambda}, \operatorname{par}_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$
    $(v k, s k) \leftarrow$ S.KeyGen
    $\left(v k^{\prime}, s k^{\prime}\right) \leftarrow$ S.KeyGen
    $(e k, d k) \leftarrow$ E.KeyGen
    $p k_{\mathcal{B}}:=\left(e k^{\prime}, e k, v k, v k^{\prime}\right)$
    $s k_{\mathcal{W}}:=\left(s k, s k^{\prime}\right)$
    $\mathcal{D C L}^{\prime}:=\emptyset ; c \operatorname{tr} \leftarrow 0$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-54.jpg?height=63&width=913&top_left_y=2034&top_left_x=557)
```

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-54.jpg?height=49&width=640&top_left_y=2095&top_left_x=553)
```
    $\operatorname{Run} \operatorname{Rcv}\left(i_{0}\right)$ with $\mathcal{A}$; et $c_{0}$ be the received coin
    $x_{0} \leftarrow$ CheckDS $_{\text {simple }}\left(s k_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{0}\right)$
    If $x_{0}=\perp$ then $c t r \leftarrow c t r+1$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-54.jpg?height=49&width=330&top_left_y=2263&top_left_x=556)
$\operatorname{Run} \operatorname{Rcv}\left(i_{1}\right)$ with $\mathcal{A}$; let $c_{1}$ be the received coin

$x_{1} \leftarrow \operatorname{CheckDS}_{\text {simple }}\left(s k_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{1}\right)$

If $x_{1}=\perp$ then $c t r \leftarrow$ ctr +1

If $\operatorname{comp}\left(c_{0}, c_{1}\right) \neq 1$ abort the entire procedure

$s n_{0}, n^{(0)}, \overrightarrow{s n}_{0}, t \overrightarrow{a g} g_{0} \leftarrow \operatorname{Extract}\left(s k_{\mathcal{C K}}, c_{0}\right)$

$s n_{1}, n^{(1)}, \overrightarrow{\operatorname{sn}}_{1}, \operatorname{tag}_{1} \leftarrow \operatorname{Extract}\left(\operatorname{sk}_{\mathcal{C K}}, c_{1}\right)$

$\mathcal{D C} \mathcal{L}^{\prime}:=\left\{s n_{0}, s n_{1}\right\}$

$\left(\vec{i}^{(0)}, \vec{i}^{(1)}\right) \leftarrow \mathcal{A}^{\text {URegist,BDepos }}$ simple, Spy

Let $k$ be the size of $\vec{i}^{(0)}$

If $k$ is different of the size of $\vec{i}^{(1)}$ abort

```
Send $s n_{0}, s n_{1}$ as challenge for the rcca-security game and receive $\tilde{c}$
    $c_{0}^{\prime} \leftarrow \operatorname{Remelt}\left(t d, \tilde{c}, n^{(0)}, \overrightarrow{\tilde{c}}_{\text {sn }, 1}, \overrightarrow{\tilde{c}}_{\text {tag }, 1}\right)$
(insert the challenge $\tilde{c}$ in this step as $\tilde{c}_{\mathrm{sn}}^{0}$ )
$\mathcal{C}$ L $[1] . c \leftarrow c_{0}^{\prime}$
Run $\mathrm{S} \& \mathrm{R}_{\mathrm{ZK}}\left(1,\left(\vec{i}^{(1)}\right)_{1}\right)$
Then repeat the following step for $j=3, \ldots,(k+1)$ :
    Run S\&R $\mathrm{ZKK}_{\mathrm{ZK}}\left(j,\left(\overrightarrow{i^{(1)}}\right)_{j-1}\right)$
Run $\operatorname{Spd}_{\mathrm{ZK}}(k+1)$ with $\mathcal{A}$
$b^{*} \leftarrow \mathcal{A}^{\text {BDepo }_{\text {rcca }}^{\prime}} ;$ return $b^{*}$
```

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-55.jpg?height=52&width=486&top_left_y=1302&top_left_x=581)
    $t \leftarrow \operatorname{E} . \operatorname{GDec}\left(d k_{\text {init }}, \tilde{c}_{\mathrm{sn}}^{0}\right)$
    If $t=$ "replay" and ctr $>0$ then abort the entire procedure
    Else if $t=$ "replay" then $c t r \leftarrow c t r+1$
    Else if $t \in \mathcal{D C} \mathcal{L}$ then return $\perp$
    Else add sn to $\mathcal{D C} \mathcal{L}$, and return $\mathcal{D C L}$
```

If the challenger of the RCCA security game encrypts $s n_{0}$, the resulting experiment will be $\operatorname{Expt}_{\mathcal{A}, 0, \text { iacr }}^{c-\operatorname{tr}}(\lambda)$; otherwise it will be $\operatorname{Expt}_{\mathcal{A}, 0, \text { rcca1 }}^{\mathrm{c}-\mathrm{tr}}(\lambda)$. We thus deduce:

Proposition 32. $\operatorname{Expt}_{\mathcal{A}, 0, \text { iacr }}^{\mathrm{c} \text { tr }}(\lambda)$ and $\operatorname{Expt}_{\mathcal{A}, 0, \text { rcca1 }}^{\mathrm{c} \text { tr }}(\lambda)$ are $\epsilon_{\text {rcca }}$-statistically close.

```
Experiment $\operatorname{Expt}_{\mathcal{A}, 0, \text { rcca1 } 1}^{\mathrm{c}-\mathrm{tr}}(\lambda)$
    $G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right)$
    $\operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r) ; \operatorname{par}_{\mathrm{S}} \leftarrow \mathrm{S} . \operatorname{Setup}(G r) ; \operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime}$.Setup $(G r)$
    $(c k, t d) \leftarrow$ C.SmSetup $(G r)$
    $\operatorname{par} \leftarrow\left(1^{\lambda}, \operatorname{par}_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$
    $\left(p k_{\mathcal{B}}, s k_{\mathcal{B}}\right) \leftarrow$ BKeyGen
    $\mathcal{D C} \mathcal{L}^{\prime}:=\emptyset ; \operatorname{ctr} \leftarrow 0$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-55.jpg?height=51&width=648&top_left_y=2094&top_left_x=543)
```
    Run $\operatorname{Rcv}\left(i_{0}\right)$ with $\mathcal{A}$; let $c_{0}$ be the received coin
    $x_{0} \leftarrow$ CheckDS $_{\text {simple }}\left(\operatorname{sk}_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{0}\right)$
    If $x_{0}=\perp$ then $c t r \leftarrow c t r+1$

```
![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-55.jpg?height=51&width=345&top_left_y=2262&top_left_x=543)

Run $\operatorname{Rcv}\left(i_{1}\right)$ with $\mathcal{A}$; let $c_{1}$ be the received coin

$x_{1} \leftarrow$ CheckDS $_{\text {simple }}\left(\operatorname{sk}_{\mathcal{C K}}, \emptyset, \mathcal{C L}, c_{1}\right)$

If $x_{1}=\perp$ then $c t r \leftarrow c t r+1$

If $\operatorname{comp}\left(c_{0}, c_{1}\right) \neq 1$ abort the entire procedure

$s n_{0}, n^{(0)}, \overrightarrow{\tilde{c}}_{\mathrm{sn}, 0}, \overrightarrow{\tilde{c}}_{\text {tag }, 0} \leftarrow \operatorname{Extract}\left(\mathrm{sk}_{\mathcal{C K}}, c_{0}\right)$

$\mathrm{sn}_{1}, n^{(1)}, \overrightarrow{\tilde{c}}_{\text {sn }, 1}, \overrightarrow{\tilde{c}}_{\text {tag }, 1} \leftarrow \operatorname{Extract}\left(\mathrm{sk}_{\mathcal{C K}}, c_{1}\right)$

$\nu \stackrel{\$}{\leftarrow} \mathcal{R}$

$\tilde{c}_{s n} \leftarrow \mathrm{E}^{\prime} . \operatorname{Enc}\left(e k^{\prime}, \Delta n_{1}, \nu\right)$

$c_{0}^{\prime} \leftarrow \operatorname{Remelt}\left(t d, \tilde{c}_{\text {sn }}, n^{(0)}, \overrightarrow{\tilde{c}}_{\text {sn }, 1}, \overrightarrow{\tilde{c}}_{\text {tag }, 1}\right)$

$\mathcal{D C L} \mathcal{L}^{\prime}:=\left\{s n_{0}, s n_{1}\right\}$

$\left(\vec{i}^{(0)}, \vec{i}^{(1)}\right) \leftarrow \mathcal{A}^{\text {URegist, BDepo }}$ simple ,Spy

Let $k$ be the size of $\overrightarrow{i^{(0)}}$

If $k$ is different of the size of $\vec{i}(1)$ : return 0

$\mathcal{C L}[1] . c \leftarrow c_{0}^{\prime}$

Run $\mathrm{S} \& \mathrm{R}_{\mathrm{ZK}}\left(1,\left(\vec{i}^{(1)}\right)_{1}\right)$

Then repeat the following step for $j=3, \ldots,(k+1)$ :

Run $\operatorname{S} \& \mathrm{R}_{\mathrm{ZK}}\left(j,\left(\vec{i}^{(1)}\right)_{j-1}\right)$

Run $\operatorname{Spd}_{\mathrm{ZK}}(k+1)$ with $\mathcal{A}$

$b^{*} \leftarrow \mathcal{A}^{\text {BDepo }_{\text {simple }}^{\prime}} ;$ return $b^{*}$

We define:

$$
\begin{aligned}
& \mathrm{S} \& \mathrm{R}_{\mathrm{ZK}, \operatorname{tag}}\left(j, i, n^{(0)}, s k^{(0)}\right) \\
& c:=\mathcal{C} \mathcal{L}[j] . c \\
& u:=\mathcal{C L}[j] \text {.owner } \\
& n^{\prime} \stackrel{\$}{\leftarrow} \mathcal{N} ; \rho_{s n}^{\prime}, \rho_{c e r t}^{\prime}, \rho_{p k}^{\prime}, \rho_{s n-p f}^{\prime}, \nu_{s n}^{\prime} \stackrel{\$}{\leftarrow} \text {; Compute: } \\
& \left(s n^{\prime}, s n-p f^{\prime}\right) \leftarrow \mathrm{T} . \mathrm{SGen}\left(p a r_{\mathrm{T}}, \mathcal{U} \mathcal{L}[i] . s k, n^{\prime}\right) \\
& c_{c e r t}^{\prime}, c_{p k}^{\prime}, c_{s n}^{\prime}, c_{s n-p f}^{\prime} \leftarrow \operatorname{C.ZCm}\left(c k, \rho_{c e r t}^{\prime}, \rho_{p k}^{\prime}, \rho_{s n}^{\prime}, \rho_{s n-p f}^{\prime}\right) \\
& \tilde{c}_{\mathrm{s} n}^{\prime} \leftarrow \text { E.Enc }\left(e k, s n^{\prime}, \nu_{\mathrm{s} n}^{\prime}\right) \\
& \pi_{c e r t}^{\prime} \leftarrow \text { C.SmPrv }\left(t d, \text { S.Verify }\left(v k^{\prime}, \cdot, \cdot\right)=1, \rho_{v k}^{\prime}, \rho_{p k}^{\prime}, \rho_{c e r t}^{\prime}\right) \\
& \pi_{s n}^{\prime} \leftarrow \operatorname{C.SmPrv}_{s n}\left(t d, p k_{t a g}^{\prime}, s n^{\prime}, s n-p f, \rho_{p k}^{\prime}, \rho_{s n}^{\prime}, \rho_{s n-p f}^{\prime}\right) \\
& \tilde{\pi}_{s n}^{\prime} \leftarrow C . \operatorname{SmPrv}_{\mathrm{enc}}\left(t d, e k, \rho_{s n}^{\prime}, \tilde{c}_{s n}^{\prime}\right)
\end{aligned}
$$

Decompose $c$ as

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-56.jpg?height=57&width=965&top_left_y=1863&top_left_x=556)

$$
\left.n, s n, \rho_{s n}, \rho_{p k}\right)
$$

$\rho_{\text {tag }}, \nu_{\text {tag }}, \rho_{t-p f} \stackrel{\$}{\leftarrow}$

(tag, t-pf $) \leftarrow$ T.TGen $\left(p a \mathrm{~T}_{\mathrm{T}}, \mathcal{U} \mathcal{L}[u] . s k, n, s n^{\prime}\right)$

$$
\left(\operatorname{tag}^{(0)}, P_{\text {tag }}^{(0)}\right) \leftarrow \text { T.TGen }\left(p \operatorname{ar}_{\mathrm{T}}, \mathrm{sk}^{(0)}, n^{(0)}, s n^{\prime}\right)
$$

$c_{\text {tag }} \leftarrow \mathrm{C} . \mathrm{ZCm}\left(c k, \rho_{\text {tag }}\right)$

$\tilde{c}_{\text {tag }} \leftarrow \operatorname{E.Enc}\left(e k, \operatorname{tag}{ }^{(0)}, \nu_{\text {tag }}\right)$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-56.jpg?height=41&width=998&top_left_y=2229&top_left_x=558)

$\tilde{\pi}_{\text {tag }} \leftarrow \mathrm{C}_{\text {SmPrvenc }}\left(t d, e k, \rho_{\text {tag }}, \tilde{c}_{\text {tag }}\right)$

```
Compute $\operatorname{VER}_{\text {init }}\left(c^{0}\right) \wedge \bigwedge_{j=1}^{i} \operatorname{VER}_{\text {std }}\left(c^{j-1}, c^{j}\right) \wedge$
    T.TVfy $\left(c k, c_{p k}^{i}, c_{\mathrm{sn}}^{\prime}, c_{\text {tag }}, \pi_{\text {tag }}\right) \wedge$ C.Verify $\mathrm{enc}\left(c k, e k, c_{\text {tag }}, \tilde{c}_{\text {tag }}, \tilde{\pi}_{\text {tag }}\right)$
Pick uniformly at random a vector of randomness $\overrightarrow{\rho^{\prime \prime}}$
$c^{\prime \prime} \leftarrow$
$\operatorname{Rand}\left(\left(c^{0},\left(c^{j}\right)_{j=1}^{i}, c_{p k}^{\prime}, c_{\text {cert }}^{\prime}, \pi_{\text {cert }}^{\prime}, c_{\mathrm{sn}}^{\prime}, \pi_{\mathrm{sn}}^{\prime}, c_{\text {tag }}, \pi_{\text {tag }}, \tilde{c}_{\mathrm{sn}}^{\prime}, \tilde{\pi}_{\mathrm{sn}}^{\prime}, \tilde{c}_{\text {tag }}^{\prime}, \tilde{\pi}_{\text {tag }}^{\prime}\right), \overrightarrow{\rho^{\prime \prime}}\right)$
$c_{n e w}:=\left(c^{\prime \prime}, n^{\prime}, s n^{\prime}, \rho_{s n}^{\prime}+\left(\overrightarrow{\rho^{\prime \prime}}\right)_{s n^{\prime}}, \rho_{p k}^{\prime}+\left(\overrightarrow{\rho^{\prime \prime}}\right)_{p k^{\prime}}\right)$
$\mathcal{C L}[|\mathcal{C L}|+1]:=\left(i, c_{\text {new }}, 0, j\right)$
```

We substituted one ciphertext for another in the previous algorithm and get:

Proposition 33. $\operatorname{Expt}_{\mathcal{A}, 0, \text { rcca1 }}^{c-t r}$ and $\boldsymbol{E x p t} \boldsymbol{A}_{\mathcal{A}, 0, \text { inal }}^{c-t r}$ are $\epsilon_{\text {iacr }}$ statistically close.

Experiment $\operatorname{Expt}_{\mathcal{A}, 0, \text { final }}^{\mathrm{c}-\mathrm{tr}}(\lambda)$ :

$G r \leftarrow \operatorname{GrGen}\left(1^{\lambda}\right)$

$\operatorname{par}_{\mathrm{T}} \leftarrow \mathrm{T} . \operatorname{Setup}(G r) ; \operatorname{par}_{\mathrm{S}} \leftarrow \mathrm{S} . \operatorname{Setup}(G r) ; \operatorname{par}_{\mathrm{S}^{\prime}} \leftarrow \mathrm{S}^{\prime} . \operatorname{Setup}(G r)$

$(c k, t d) \leftarrow$ C.SmSetup $(G r)$

$p a r \leftarrow\left(1^{\lambda}, \operatorname{par}_{\mathrm{S}}, \operatorname{par}_{\mathrm{S}^{\prime}}, \operatorname{par}_{\mathrm{T}}, c k\right)$

$\left(p k_{\mathcal{B}}, s k_{\mathcal{B}}\right) \leftarrow$ BKeyGen

$\mathcal{D C} \mathcal{L}^{\prime}:=\emptyset ; c \operatorname{tr} \leftarrow 0$

$i_{0} \leftarrow \mathcal{A}^{\text {URegist,BDepo',Spy }}\left(\right.$ par, $\left.p k_{\mathcal{B}}, s k_{\mathcal{W}}, \mathrm{sk}_{\mathcal{D}}\right)$

$\operatorname{Run} \operatorname{Rcv}\left(i_{0}\right)$ with $\mathcal{A}$; let $c_{0}$ be the received coin

$i_{1} \leftarrow \mathcal{A}^{\text {URegist,BDepo }}$,Spy

Run $\operatorname{Rcv}\left(i_{1}\right)$ with $\mathcal{A}$; let $c_{1}$ be the received coin

If $\operatorname{comp}\left(c_{0}, c_{1}\right) \neq 1$ abort the entire procedure

$s n_{0}, n^{(0)}, \overrightarrow{\tilde{c}}_{\text {sn }, 0}, \overrightarrow{\tilde{c}}_{\text {tag }, 0} \leftarrow \operatorname{Extract}\left(\mathrm{sk}_{\mathcal{C K}}, c_{0}\right)$

$s n_{1}, n^{(1)}, \overrightarrow{\tilde{c}}_{\text {sn,1 }}, \overrightarrow{\tilde{c}}_{\text {tag }, 1} \leftarrow \operatorname{Extract}\left(\mathrm{sk}_{\mathcal{C K}}, c_{1}\right)$

$\nu \stackrel{\$}{\leftarrow}$

$\tilde{c}_{\mathrm{sn}} \leftarrow \mathrm{E}^{\prime} . \operatorname{Enc}\left(e k^{\prime}, s n_{1}, \nu\right)$

$c_{0}^{\prime} \leftarrow \operatorname{Remelt}\left(t d, \tilde{c}_{\mathrm{sn}}, n^{(0)}, \overrightarrow{\tilde{c}}_{\mathrm{sn}, 1}, \overrightarrow{\tilde{c}}_{\text {tag }, 1}\right)$

$\mathcal{D C L} \mathcal{L}^{\prime}:=\left\{s n_{0}, s n_{1}\right\}$

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-57.jpg?height=52&width=499&top_left_y=1676&top_left_x=558)

Let $k$ be the size of $\vec{i}^{(0)}$

If $k$ is different of the size of $\vec{i}(1)$ : return 0

$\mathcal{C L}[1] . c \leftarrow c_{0}^{\prime}$

Run $\mathrm{S} \& \mathrm{R}_{\mathrm{ZK}, \operatorname{tag}}\left(1,\left(\vec{i}^{(1)}\right)_{1}, n^{(1)}, s k\right)$

Then repeat the following step for $j=3, \ldots,(k+1)$ :

$$
\operatorname{Run} \mathrm{S} \& \mathrm{R}_{\mathrm{ZK}}\left(j,\left(\vec{i}^{(1)}\right)_{j-1}\right)
$$

Run $\operatorname{Spd}_{\mathrm{ZK}}(k+1)$ with $\mathcal{A}$

$b^{*} \leftarrow \mathcal{A}^{\mathrm{BDepo}_{\text {simple }}^{\prime}} ;$ return $b^{*}$

We note that

$$
\operatorname{Expt}_{\mathcal{A}, 0, \text { final }}^{c-\operatorname{tr}}(\lambda)=\operatorname{Expt}_{\mathcal{A}, 1, \mathrm{ZK}}^{\mathrm{c}-\mathrm{tr}}(\lambda)
$$

It is $\epsilon_{\mathrm{m} \text {-ind }}$-close to $\operatorname{Expt} \operatorname{ta}_{\mathcal{A}, 1}^{\mathrm{c} \mathrm{tr}}(\lambda)$. By combining this last remark with Propositions $29,30,31,32$ and 33 , this proves the theorem.

\section{B Instantiation}

\section{B. 1 Instantiation and proofs of the double spending tag scheme}

We will reuse the scheme introduced in [BCFK15], which we recall here.

```
T.Setup $(G r)$ :
    - Parse $G r$ as $\left(p, \mathbb{G}, \hat{\mathbb{G}}, \mathbb{G}_{T}, e, g_{1}, \hat{g}\right)$
    $-g_{2}, h_{1}, h_{2} \stackrel{\$}{\leftarrow} \mathbb{G}$
    - Return $\left(g_{1}=g, g_{2}, h_{1}, h_{2}\right)$
    We define $\mathcal{M}=\left\{\left(g_{1}^{m}, \hat{g}^{m}\right) \in \mathbb{G} \times \hat{\mathbb{G}}\right\}_{m \in \mathbb{Z}_{p}}$
T.KeyGen $\left(p \operatorname{ar}_{\mathrm{T}}\right)$ :
    $-s k \stackrel{\&}{\leftarrow} \mathbb{Z}_{p}$
    - Return $\left(s k_{\mathrm{\top}}:=s k, p k_{\mathrm{\top}}:=\hat{g}^{s k}\right)$
T.SGen $_{\text {init }}\left(\right.$ par $_{\mathrm{T}}$, sk $\left._{\mathrm{T}}, n\right)$ :
    $-M \leftarrow g_{1}^{n} ; N \leftarrow g_{2}^{n+s k_{T}}$
    $-M_{\mathrm{sn}}^{(1)}=\left(g_{1}^{n}, \hat{g}^{n}\right) ; M_{s n}^{(2)}=\left(g_{1}^{s k_{T}}, \hat{g}^{s k_{\top}}\right)$
    - Return $\left(s n=(M, N), M_{s n}=\left(M_{s n}^{(1)}, M_{s n}^{(2)}\right)\right)$
T.SGen $\left(\operatorname{par}_{\mathrm{T}}, s k_{\mathrm{T}}, n\right)$ :
    $-M \leftarrow g_{1}^{n} ; N \leftarrow g_{2}^{n+s k_{T}}$
    $-s n-p f=\hat{g}^{n}$
    - Return $(s n=(M, N), s n-p f)$
T.TGen $\left(p \operatorname{ar}_{\mathrm{T}}, s k, n, s n=(M, N)\right)$ :
    $-M_{0} \leftarrow g_{1}^{n}$
    $-\operatorname{tag}:=\left(M^{s k} h_{1}^{n}, N^{s k} h_{2}^{n}\right)$
    $-t-p f \leftarrow \hat{g}^{n}$
    - Return $(\operatorname{tag}:=(A, B), t-p f)$.
T. Detect (sn, sn', tag, $\operatorname{tag}^{\prime}, \mathcal{L}$ ):
    - Parse sn as $(M, N)$; parse $s n^{\prime}$ as $\left(M^{\prime}, N^{\prime}\right)$
    - Parse tag as $(A, B)$; parse tag ${ }^{\prime}$ as $\left(A^{\prime}, B^{\prime}\right)$
    $-A^{\prime \prime}:=\frac{A}{A^{\prime}} ; B^{\prime \prime}:=\frac{B}{B^{\prime}}$
    $-M^{\prime \prime}:=\frac{M}{M^{\prime}} ; N^{\prime \prime}:=\frac{N}{N^{\prime}}$
    - If $A^{\prime \prime}=0_{G_{1}}$ then:
        $A^{\prime \prime}:=B^{\prime \prime} ; M^{\prime \prime}:=N^{\prime \prime}$
    - Search $p k_{\mathrm{\top}}$ in $\mathcal{L}$ such that $e\left(A^{\prime \prime}, \hat{g}\right)=e\left(M^{\prime \prime}, p k\right)$
    - Return $\left(p k_{\mathrm{T}},\left(A^{\prime \prime}, M^{\prime \prime}\right)\right)$
T.VfyGuilt $(p k, \pi)$ :
    - Parse $\pi$ as $(A, N)$;
    - Return $\left(e(A, \hat{g})=e(N, p k) \wedge A \neq 0_{G_{1}}\right)$.
T.SVfy $_{\text {init }}\left(\operatorname{par}_{\mathrm{T}}, p k_{\mathrm{T}}, s n, M_{\text {sn }}\right):$
    - Parse sn as $(M, N)$
    - Parse $M_{s n}$ as $\left(\left(M_{1}, \hat{M}_{1}\right),\left(M_{2}, \hat{M}_{2}\right)\right)$
    - Return $\left(e(M, \hat{g}) e\left(g_{1}^{-1}, \hat{M}_{1}\right)=1_{G_{T}} \wedge e(M, \hat{g}) e\left(g_{2}^{-1}, \hat{M}_{2}\right) e\left(g_{2}^{-1}, p k_{\mathrm{T}}\right)=\right.$
        $\left.1_{G_{T}} \wedge \hat{M}_{2}=p k_{\mathrm{T}} \wedge e\left(M_{1}, \hat{g}\right)=e\left(g_{1}, \hat{M}_{1}\right) \wedge e\left(M_{2}, \hat{g}\right)=e\left(g_{1}, \hat{M}_{2}\right)\right)$
```

```
T.SVfy $\left(p a r_{\mathrm{T}}, p k_{\mathrm{T}}, s n, s n-p f\right):$
    - Parse sn as $(M, N)$
    - Return $\left(e(M, \hat{g}) e\left(g_{1}^{-1}\right.\right.$, sn-pf $)=1_{G_{T}} \wedge$
                $e(N, \hat{g}) e\left(g_{2}^{-1}\right.$, sn-pf $\left.) e\left(g_{2}^{-1}, p k_{\mathrm{T}}\right)=1_{G_{T}}\right)$
T.TVfy $\left(p a{ }_{\mathrm{T}}, p k, s n, s n^{\prime}\right.$, tag, t-pf $)$ :
    - Parse sn as $(M, N)$
    - Parse tag as $(A, B)$
    - Parse $s n^{\prime}$ as $\left(M^{\prime}, N^{\prime}\right)$
    - Return $\left(e(M, \hat{g}) e\left(g_{1}^{-1}, t-p f\right)=1_{G_{T}} \wedge\right.$
                        $e\left(A, \hat{g}^{-1}\right) e\left(M^{\prime}, p k\right) e\left(h_{1}, t-p f\right)=1_{G_{T}} \wedge$
$\left.e\left(B, \hat{g}^{-1}\right) e\left(N^{\prime}, p k\right) e\left(h_{2}, t-p f\right)=1_{G_{T}}\right)$
```

\section{Proofs}

Theorem 34. This above scheme is extractable, bootable, $S N$-verifiable, tagverifiable and $\mathcal{N}$ injective.

These properties are all straightforward to show and we therefore omit the proof.

Proposition 35. The above scheme is SN-collision-resistant.

Let ( $p k, s n-p f)$ and $\left(p k^{\prime}, s n-p f^{\prime}\right)$ such that for some sn:

$$
\mathrm{T} . \mathrm{SVfy}\left(p a \mathrm{~T}_{\mathrm{T}}, p k, s n, s n-p f\right)=\mathrm{T} . \mathrm{SVfy}\left(p a r, p k^{\prime}, s n, s n-p f^{\prime}\right)=1
$$

We parse sn as $(M, N)$, and deduce the followings equations:

$$
e(M, \hat{g})=e\left(g_{1}, s n-p f\right)=e\left(g_{1}, s n-p f^{\prime}\right)
$$

from which we get $s n-p f=s n-p f^{\prime}$. Then we can deduce

$$
e(M, \hat{g}) e\left(g_{2}^{-1}, \text { sn-pf }\right) e\left(g_{2}^{-1}, p k\right)=0_{G_{T}}=e(M, \hat{g}) e\left(g_{2}^{-1}, \text { sn-pf }\right) e\left(g_{2}^{-1}, p k^{\prime}\right)
$$

and thus $e\left(g_{2}^{-1}, p k\right)=e\left(g_{2}^{-1}, p k^{\prime}\right)$, which finally yields $p k=p k^{\prime}$. The reasoning

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-59.jpg?height=41&width=409&top_left_y=1671&top_left_x=457)

To prove the two other results, we will use the following lemma.

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-59.jpg?height=249&width=1044&top_left_y=1838&top_left_x=497)

Lemma 36. For any adversary $\mathcal{A}$ against the game Tuple defined above with advantage $\epsilon$, there exists an adversary $\mathcal{B}$ against DDH in $\mathbb{G}$ with advantage $\epsilon_{\mathrm{DDH}}$ such that $\frac{\epsilon}{3 K} \leq \epsilon_{\mathrm{DDH}}$, with $K$ the number of oracles calls to $O_{b}^{\prime}$.

We define the following oracles:

$$
\begin{array}{lll}
O_{0.3}^{\prime}: & O_{0.7}^{\prime}: & \\
& n, n_{2} \stackrel{\&}{\leftarrow} \mathbb{Z}_{q} & n, n_{2}, n_{3} \stackrel{\&}{\leftarrow} \mathbb{Z}_{q} \\
\text { Return }\left(g_{1}^{n}, g_{2}^{n_{2}}, h_{1}^{n}, h_{2}^{n}\right) & & \text { Return }\left(g_{1}^{n}, g_{2}^{n_{2}}, h_{1}^{n_{3}}, h_{2}^{n}\right)
\end{array}
$$

Viewing the tuples $\left(g_{2}, g_{1}^{n}, g_{2}^{n}\right),\left(h_{1}, g_{1}^{n}, h_{1}^{n}\right)$ and $\left(h_{2}, g_{1}^{n}, h_{2}^{n}\right)$ as DDH challenge tuples, we get taht the adversary cannot distinguish $O_{0}^{\prime}$ from $O_{0.3}^{\prime}$ nor $O_{0.3}^{\prime}$ from $O_{0.7}^{\prime}$, or $O_{0.7}^{\prime}$ from $O_{1}^{\prime}$, with probability more than $\epsilon_{\mathrm{DDH}}$ each respectively. (Note that we can compute all other elements, since we know the discrete logarithms of the elements which are not part of the respective DDH-tuple; and we can answer all other oracle calls honestly). This proves the lemma.

Corollary 37. No adversary can break tag-anonymity with an advantage better than $6 K \epsilon_{\mathrm{DDH}}$, where $K$ is the number of calls to $O_{1}$.

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-60.jpg?height=193&width=1054&top_left_y=1031&top_left_x=454)

$$
\begin{aligned}
& \left(s k_{0}, s k_{1}\right) \leftarrow \mathcal{A}\left(p \mathrm{ar}_{\mathrm{T}}\right) \\
& k:=0 \\
& b^{*} \leftarrow \mathcal{A}_{1}^{O_{1}^{\text {perfect }}\left(s k_{b}\right), O_{2}^{\text {perfect }}\left(s k_{b}, \cdot, \cdot\right)}\left(p a a_{\mathrm{T}}, s k_{0}, s k_{1}\right) \\
& \text { Return }\left(b=b^{*}\right)
\end{aligned}
$$

Lemma 36 implies that the adversary cannot, except with probability $3 K \epsilon_{\mathrm{DDH}}$, distinguish Perfect-tag-anonymity from tag-anonymity: if we replace $O_{0}^{\prime}$ by $O_{1}^{\prime}$, the game becomes exactly tag-anonymity. In the latter game, we can replace $b$ by $(1-b)$ without changing the distribution of the adversary's input.

Theorem 38. Let $\mathcal{A}$ be an adversary that wins the exculpability game with probability $\epsilon$ after $K$ oracle calls to $O_{1}$, then there exist $\mathcal{B}_{1}$ against DDH in $\mathbb{G}$ with advantage $\epsilon_{\mathrm{DDH}}$ and $\mathcal{B}_{2}$ against DDH in $\hat{\mathbb{G}}$ with advantage $\hat{\epsilon}_{\mathrm{DDH}}$, such that:

$$
\epsilon \leq 3 K \epsilon_{\mathrm{DDH}}+\hat{\epsilon}_{\mathrm{DDH}}
$$

Using the same argument as in Corollary 37, we deduce, incurring a loss of $3 K \epsilon_{\mathrm{DDH}}$, that we can consider that oracle calls do not yield any information to the adversary. After receiving a triple $\left(\hat{g}_{1}, \hat{g}_{2}, \hat{g}_{3}\right)$ in $\hat{\mathbb{G}}$, we send $\hat{g_{1}}$ as the public key. When we receive $(N, A)$ such that

$$
e(N, p k)=e(A, \hat{g})
$$

with $A \neq 0_{\mathbb{G}_{1}}$, this means that $A=N^{\log _{g_{1}}(p k)}$ and we can check if $e\left(N, \hat{g}_{3}\right)=$ $e\left(A, \hat{g}_{2}\right)$ to decide whether we received a DDH triple or not.

Efficiency We summarize all the efficiency results as follows (where "m.s.w.u" means multiscalar with unkown):

\begin{tabular}{|c|c|}
\hline$\left|p a r_{T}\right|$ & $3|\mathbb{G}|$ \\
\hline$\left|s k_{T}\right|$ & $\left|\mathbb{Z}_{p}\right|$ \\
\hline$\left|p k_{\mathrm{T}}\right|$ & $|\hat{\mathbb{G}}|$ \\
\hline$|s n|=|t a g|$ & $2|\mathbb{G}|$ \\
\hline$|s n-p f|=|t-p f|$ & $|\hat{\mathbb{G}}|$ \\
\hline$\pi$ & $2|\mathbb{G}|$ \\
\hline Number of pairing equations in T.SVfy & 2 generic eq. \\
\hline Number of pairing equations in T.SVfy \\
init & 4 generic, 1 m.s.w.u eq. in $\hat{\mathbb{G}}$ \\
\hline Number of pairing equations in T.TVfy & 3 generic eq. \\
\hline$\left|\pi_{\text {sn }}\right|$ & $8|\mathbb{G}|+8|\hat{\mathbb{G}}|$ \\
\hline$\left|\pi_{\text {sn,init }}\right|$ & $16|\mathbb{G}|+16|\hat{\mathbb{G}}|+2\left|\mathbb{Z}_{p}\right|$ \\
\hline$\left|\pi_{\text {tag }}\right|$ & $12|\mathbb{G}|+12|\hat{\mathbb{G}}|$ \\
\hline$\left|\tilde{\pi}_{\text {sn }}\right|$ & $8|\mathbb{G}|+10|\hat{\mathbb{G}}|$ \\
\hline$\left|\tilde{\pi}_{\text {tag }}\right|$ & $12|\mathbb{G}|+14|\hat{\mathbb{G}}|$ \\
\hline
\end{tabular}

\section{B. 2 Instantiation of the encryption scheme $E^{\prime}$}

Construction overview. We roughly follow the framework proposed by Chase et al. [CKLM12]. The first part of the ciphertext is an encryption

$$
\vec{C}=\left(c_{0}, c_{1}, \ldots, c_{n+1}\right)=\left(f^{\theta}, g^{\theta},\left\{h_{i}^{\theta} \cdot m_{i}\right\}_{i=1}^{n}\right)
$$

of the message vector $\vec{m}=\left(m_{1}, \ldots, m_{n}\right) \in \mathbb{G}_{n}$. As in [LPQ17], we use the same one-time linearly homomorphic structure-preserving signature scheme [LPJY13] LHSPS $=($ KeyGen, Sign.Verify $)$, for which let

$$
\vec{v}_{1}=(f, g, 1, \ldots, 1), \vec{v}_{2}=\left(1,1,1, g, h_{1}, \ldots, h_{n}\right)
$$

the signing key of the LHSPS is composed of two linearly homomorphic signatures of $\vec{v}_{1}$ and $\vec{v}_{2}$, that is, sk $=\left(\sigma_{\vec{v}_{1}}, \sigma_{\vec{v}_{2}}\right)$. Using this signing key, anyone can generate the signature $\sigma_{\vec{m}}$ of any message $\vec{m} \in \operatorname{Span}\left(\vec{v}_{1}, \vec{v}_{2}\right)$.

The second part of the ciphertext is a zero-knowledge proof for the language $\mathcal{L}_{\vee}=\left\{\left(\vec{C},\left(b, \theta,\left\{m_{i}\right\}_{i=1}^{n}, \sigma_{\vec{v}}\right)\right) \mid b \in\{0,1\} \vee \operatorname{LHSPS}\right.$. Verify $\left.\left(v k_{\text {LHSPS }}, \sigma_{\vec{v}}, \vec{v}\right)=1\right\}$

where $\vec{v}=\left(c_{0}^{b}, c_{1}^{b}, g^{1-b}, c_{1}^{(1-b)}, c_{2}^{(1-b)}, \ldots, c_{n+1}^{1-b}\right)$. Note that when $b=1, \vec{v} \in$ $\operatorname{Span}\left(\vec{v}_{1}, \vec{v}_{2}\right)$ means that $\log _{f}\left(c_{0}\right)=\log _{g}\left(c_{1}\right)$, then the ciphertext is a valid ciphertext. Also note that signatures on $\vec{v}$ with $b=1$ will only be generated in the security proof.

To enable re-randomization, we generate a signature $\sigma_{\vec{w}}$ on the vector $\vec{w}=$ $\left(f^{b}, g^{b}, 1, g^{(1-b) \cdot \theta}, h_{1}^{(1-b) \cdot \theta}, \ldots, h_{n}^{(1-b) \cdot \theta}\right)$ and add a zero-knowledge proof of knowledge of the valid signature $\sigma_{\vec{w}}$. It is easy to see that with $\sigma_{\vec{w}}$, we can generate signatures for all re-randomization of the vector $\vec{v}$.

One-time linearly homomorphic structure-preserving signature. To construct the re-randomizable CCA encryption scheme, we need the one-time linearly homomorphic structure-preserving signature.

Definition 39 ((One-time) linearly homomorphic structure-preserving signature [LPJY13]). A one-time linearly homomorphic structure-preserving signature is tuple of 4 algorithms LHSPS $=($ Setup, Sign, SignDerive, Verify) with the following specifications:

Setup $(G r, n)$ is a probabilistic algorithm taking the group parameter Gr and an integer $n$ denoting the dimension of the message to be signed. It outputs the public verification key vk and the signature key sk.

Sign (sk, $\vec{m}$ ) is a deterministic algorithm that takes the signing key sk and the message $\vec{m} \in \mathbb{G}^{n}$, and outputs a signature $\sigma$

SignDerive $\left(v k,\left\{\left(w_{i}, \sigma^{(i)}\right)\right\}_{i=1}^{\ell}\right)$ is a deterministic algorithm taking the verification key vk and $\ell$ pairs $\left(w_{i} \sigma^{(i)}\right)$ where $w_{i} \in \mathbb{Z}_{p}$ and $\sigma^{(i)}$ is an LHSPS signature. It outputs a signature $\sigma$ on the message $\vec{m}=\prod_{i=1}^{\ell} \vec{m}_{i}^{w_{i}}$.

Verify $(v k, \vec{m}, \sigma)$ is a deterministic algorithm taking the verification key vk, the message vector $\vec{m}$ and a signature $\sigma$. It outputs 1 if the signature is valid, 0 otherwise.

Definition 40 (One-time unforgeability). A one-time linearly homomorphic SPS scheme $\Sigma=$ (KeyGen, Sign, Verify) is secure if no adversary has nonnegligible advantage in the following game:

1. The adversary $\mathcal{A}$ outputs an integer $n$, sends it to the challenger $\mathcal{C}$. The challenger generates $(v k, s k) \leftarrow \operatorname{Setup}\left(1^{\lambda}, n\right)$ and sends the public verification key back to $\mathcal{A}$.

2. The adversary $\mathcal{A}$ has access to the signing oracle
- Sign $(s k, \cdot): \mathcal{A}$ can request the challenger $\mathcal{C}$ to sign the message vectors $\left\{\vec{m}_{i}\right\}_{i=1}^{Q_{s}}$ where $Q_{s}$ denotes the number of signing queries.
3. $\mathcal{A}$ outputs $\left(\vec{m}^{\star}, \sigma_{\star}\right)$. The adversary wins if and only if Verify $\left(v k, \vec{m}^{\star}, \sigma_{\star}\right)=1$ and $\vec{m}^{\star} \notin \operatorname{Span}\left(\left\{\vec{m}_{i}\right\}_{i=1}^{Q_{s}}\right)$.

We recall the following construction of the one-time linearly homomorphic structure-preserving signature scheme.
- LHSPS.Setup $(G r, n)$ :

1. Parse $G r$ as $\left(\mathbb{G}, \hat{\mathbb{G}}, \mathbb{G}_{T}, e\right)$.

2. Chose $\hat{g}_{z}, \hat{g}_{r} \stackrel{\$}{\leftarrow} \hat{\mathbb{G}}$. For $i \in\{1, \ldots, n\}$, randomly chose $\chi_{i}, \gamma_{i}$ and compute $\hat{g}_{i}=\hat{g}_{z}^{\chi_{i}} \hat{g}_{r}^{\gamma_{i}}$

3. Output the verification key $p k=\left(\hat{g}_{z}, \hat{g}_{r},\left\{\hat{g}_{i}\right\}^{n}\right) \in \hat{\mathbb{G}}^{n+2}$ and the signing key sk $=\left(\left\{\chi_{i}, \gamma_{i}\right\}_{i=1}^{n}\right)$.
- LHSPS.Sign(vk, sk, $\vec{M})$ :

1. Parse the verification key $v k=\left(\hat{g}_{z}, \hat{g}_{r},\left\{\hat{g}_{i}\right\}^{n}\right) \in \hat{\mathbb{G}}^{n+2}$, the signing key sk $=\left(\left\{\chi_{i}, \gamma_{i}\right\}_{i=1}^{n}\right)$ and the message $\vec{M}=\left(M_{1}, \ldots, M_{n}\right) \in \mathbb{G}^{n}$.

2. Output the signature $\vec{\sigma}=(z, r) \in \mathbb{G}^{2}$ such that $z=\prod_{i=1}^{n} M_{i}^{\chi_{i}}$ and $r=\prod_{i=1}^{n} M_{i}^{\gamma_{i}}$.
- LHSPS.SignDerive $\left(v k,\left(\vec{\sigma},\left\{w_{i}, \sigma^{(i)}\right\}_{i=1}^{\ell}\right)\right)$ :

1. For all $i \in\{1, \ldots, \ell\}$, parse $\sigma^{(i)}$ as $\left(z_{i}, r_{i}\right)$.

2. Output the signature $\sigma=\left(\prod_{i=1}^{\ell} z_{i}^{w_{i}}, \prod_{i=1}^{\ell} r_{i}^{w_{i}}\right)$.

![](https://cdn.mathpix.com/cropped/2024_07_18_259004b7e1af0344488dg-63.jpg?height=54&width=436&top_left_y=651&top_left_x=481)

1. Parse the signature as $\sigma=(z, r)$ and the message $\vec{M}=\left(M_{1}, \ldots, M_{n}\right)$.

2. Return 1 iff $\left(M_{1}, \ldots, M_{n}\right) \neq\left(1_{\mathbb{G}}, \ldots, 1_{\mathbb{G}}\right)$ and the following equation is verified.

$$
e\left(z, \hat{g}_{z}\right) \cdot e\left(r, \hat{g}_{r}\right)=\prod_{i=1}^{n} e\left(M_{i}, \hat{g}_{i}\right)
$$

Theorem 41 ([LPJY13, Theorem 1]). The above construction of a one-time linearly homomorphic structure-preserving signature scheme is unforgeable if the SXDH assumption holds in the underlying group.

The above scheme was proven to be unforgeable under the DP assumption, which is implied by the SXDH assumption. As in the remaining part of the construction of RCCA requires SXDH to hold, we state this theorem with SXDH assumption.

Replayable-CCA encryption scheme. An RCCA encryption scheme E consists of six PPT algorithms $\mathrm{E}=$ (KeyGen, Enc, ReRand, Dec, Verify, AdptPrf). It should verify the following specifications:
- E.KeyGen $(G r)$ : a randomized algorithm which takes as input the group description and outputs an encryption public key $p k$ and a corresponding decryption key $d k$.
- E.Enc $(p k, m, \nu)$ : a randomized encryption algorithm which takes as input a public encryption key $p k$, a plaintext (from a plaintext space), some randomness and outputs a ciphertext.
- E.ReRand $(p k, c, \nu)$ : a randomized algorithm which takes as input a public key, a ciphertext and some randomness,. and outputs another ciphertext.
- E.Dec $(d k, c)$ : a deterministic decryption algorithm which takes a decryption key and a ciphertext, and outputs either a plaintext or an error indicator $\perp$.
- E.Verify $(p k, m, \rho, c)$ : a deterministic algorithm which takes as input a public key, a message, some randomness, and a ciphertext and outputs a bit.
- E.AdptPrf $\left(c k, p k, c_{M}, c,\left(\pi, c_{\nu}\right), \nu^{\prime}\right)$ a randomized algorithm which takes as input a commitment key, an encryption public key, a commitment, an equality proof (i.e a Groth-Sahai proof and a commitment), a ciphertext, a proof, some randomness, and outputs an equality proof.

We give the following explicit construction of the RCCA scheme supporting encryption of vectors of group elements.

\section{E.KeyGen $(G r)$ :}

1. Parse $G r$ as $\left(\mathbb{G}, \hat{\mathbb{G}}, \mathbb{G}_{T}, e\right)$.

2. Choose two random group elements $f, g \stackrel{\$}{\leftarrow} \mathbb{G}^{2}$.

3. Choose random exponents $\left\{\alpha_{i}\right\}_{i=1}^{n} \stackrel{\leftarrow}{\leftarrow} \mathbb{Z}_{p}$ and compute $\left\{h_{i}\right\}_{i=1}^{n}=g^{\alpha_{i}}$.

4. Generate the Groth-Sahai crs $c \vec{r} s_{G S}$ by choosing random $\overrightarrow{u_{1}}, \overrightarrow{u_{2}} \leftarrow \mathbb{G}^{2}$ and $\hat{\vec{u}}_{1}, \hat{\vec{u}}_{2} \stackrel{\otimes}{\leftarrow} \hat{\mathbb{G}}^{2}$.

5. Define two vectors $\vec{v}_{1}, \vec{v}_{2}$ such that

$$
\vec{v}_{1}=(f, g, 1,1, \ldots, 1) \in \mathbb{G}^{n+2} \quad \vec{v}_{2}=\left(1,1,1, h_{1}, \ldots, h_{n}\right) \in \mathbb{G}^{n+2}
$$

then generate two LHSPS signatures $\sigma_{\vec{v}_{1}}$ and $\sigma_{\vec{v}_{2}}$ which will be used to proof that a vector is in the $\operatorname{Span}\left(\vec{v}_{1}, \vec{v}_{2}\right)$. together with the signing key $t \vec{k}$.

6. Output the decryption key $d k=\alpha$ and the public key

$$
p k=\left(f, g,\left\{h^{\alpha_{i}}\right\}_{i=1}^{n}, c \vec{r} s_{G S}, \sigma_{\mathrm{LHSPS}}=\left(\sigma_{\vec{v}_{1}}, \sigma_{\vec{v}_{2}}\right)\right)
$$

Notice that the LHSPS signing key $t \vec{k}$ will never be published by the key generation algorithm, it will only be used in the security proofs.

E.Enc $(p k, m, \nu)$ :

1. Randomly pick a number $\theta \in \mathbb{Z}_{p}$. Compute $\vec{C}=\left(c_{0}, c_{1}, \ldots, c_{n+1}\right)=$ $\left(f^{\theta}, g^{\theta}, M_{1} \cdot h_{1}^{\theta}, \ldots, M_{n} \cdot h_{n}^{\theta}\right)$.

2. Define the bit $b=1$ and denote $G=g^{b} \in \mathbb{G}$ and $\hat{G}=\hat{g}^{b} \in \hat{\mathbb{G}}$.

3. Generate the Groth-Sahai proof $\pi_{b}$ of

$$
e(G, \hat{g})=e(g, \hat{G})
$$

4. For all $i \in\{1, \ldots n+1\}$, compute $\Theta_{i}=c_{i}^{b}$. Compute also the Groth-Sahai $\pi_{\Theta}$ proof of the equations:

$$
e\left(\Theta_{i}, \hat{g}\right)=e\left(c_{i}, \hat{G}\right)
$$

5. Define the vector $\vec{v}=\left(c_{0}^{b}, c_{1}^{b}, g^{1-b}, c_{1}^{1-b}, \ldots, c_{n+1}^{1-b}\right)$. Generate a LHSPS signature $\sigma_{\vec{v}}$ such that $\vec{v} \in \operatorname{Span}\left(\vec{v}_{1}, \vec{v}_{2}\right)$.

6. Compute a Groth-Sahai proof $\pi_{\vec{v}}$ of the validity of the LHSPS signature $\sigma_{\vec{v}}$.

7. To enable the re-randomization, compute

$$
\left(F, G,\left\{H_{i}\right\}_{i=1}^{n}\right)=\left(f^{b}, g^{b},\left\{h_{i}^{b}\right\}_{i=1}^{n}\right)
$$

and Groth-Sahai proof $\pi_{F G H}$ of them.

8. Define the vector $\vec{w}=\left(f^{b}, g^{b}, 1, h_{1}^{1-b}, \ldots, h_{n}^{1-b}\right)$. Compute a LHSPS signature $\sigma_{\vec{w}}$ of the fact that $\vec{w} \in \operatorname{Span}\left(\vec{v}_{1}, \vec{v}_{2}\right)$.

9. Generate a Groth-Sahai proof $\pi_{\vec{w}}$ of the validity of LHSPS signature $\sigma_{\vec{w}}$.

10. Output the ciphertext $c=\left(\left\{c_{i}\right\}_{i=1}^{n}, \pi_{b}, \pi_{\theta}, \pi_{\vec{v}}, \pi_{F G H}, \pi_{\vec{w}}\right)$.

\section{E.ReRand $(p k, c, \nu)$ :}

1. Parse $c=\left(\left\{c_{i}\right\}_{i=1}^{n+1}, \pi_{b}, \pi_{\theta}, \pi_{\vec{v}}, \pi_{F G H}, \pi_{\vec{w}}\right)$.

2. Compute $c_{0}^{\prime}=c_{0} \cdot f^{\nu}, c_{1}^{\prime}=c_{1} \cdot g^{\nu}$ and for $i \in\{2, \ldots, n+1\}$, compute $c_{i}^{\prime}=c_{i} \cdot h_{i-1}^{\nu}$.

3. We update the proof $\pi_{b}, \pi_{\theta}$ using the commitment $C_{F}, C_{G}, C_{H}$ in $\pi_{F G H}$ to get $\pi_{b}^{\prime}, \pi_{\theta}^{\prime}$.

4. We update the commitment of the LHSPS signature $C_{\sigma_{\vec{v}}}^{\prime}=C_{\sigma_{\vec{v}}} \cdot C_{\sigma_{\vec{w}}}^{\nu}$ and the update the proof $\pi_{\vec{v}}$ accordingly to get $\pi_{\vec{v}}^{\prime}$.

5. We re-randomize all the updated Groth-Sahai proofs

$$
\pi_{b}^{\prime}, \pi_{\theta}^{\prime}, \pi_{\vec{v}}^{\prime}, \pi_{F G H}, \pi_{\vec{w}}
$$

to get the new proofs $\pi_{b}^{\prime \prime}, \pi_{\theta}^{\prime \prime}, \pi_{\vec{v}}^{\prime \prime}, \pi_{F G H}^{\prime \prime}, \pi_{\vec{w}}^{\prime \prime}$.

6. Output the new ciphertext $c^{\prime}=\left(\left\{c_{i}^{\prime}\right\}_{i=1}^{n}, \pi_{b}^{\prime \prime}, \pi_{\theta}^{\prime \prime}, \pi_{\vec{v}}^{\prime \prime}, \pi_{F G H}^{\prime \prime}, \pi_{\vec{w}}^{\prime \prime}\right)$.

$\operatorname{E} . \operatorname{Dec}(d k, c):$

1. Parse $c$ as $\sigma=\left(\left\{c_{i}\right\}_{i=1}^{n}, \pi_{b}, \pi_{\theta}, \pi_{\vec{v}}, \pi_{F G H}, \pi_{\vec{w}}\right)$.

2. Check all proofs $\left(\pi_{b}, \pi_{\theta}, \pi_{\vec{v}}, \pi_{F G H}, \pi_{\vec{w}}\right)$ are valid.

3. For $i \in\{1, \ldots n\}$, compute $M_{i}=c_{i+1} /\left(c_{1}^{\alpha_{i}}\right)$.

4. Output $\left\{M_{i}\right\}_{i=1}^{n}$.

E.Verify $(p k, \vec{m}, \nu, c)$ :

1. Parse $c$ as $\sigma=\left(\left\{c_{i}\right\}_{i=1}^{n}, \pi_{b}, \pi_{\theta}, \pi_{\vec{v}}, \pi_{F G H}, \pi_{\vec{w}}\right)$.

2. Verify that $\pi_{b}, \pi_{\theta}, \pi_{\vec{v}}, \pi_{F G H}, \pi_{\vec{w}}$ are all correct.

3. Verify the following pairing equations:

$$
c_{0}=g^{\nu} \quad c_{1}=f^{\nu} \quad c_{i+1}=h^{\nu} \cdot m_{i}
$$

where $i \in\{1, \ldots, n\}$.

E.AdptPrf $\left(c k, p k, c_{M}, c,\left(\pi, c_{\nu}\right), \nu^{\prime}\right)$ :

1. We just update the Groth-Sahai proof the new randomness $\nu^{\prime}$ by multiplying $c_{\nu}^{\prime}=c_{\nu} \cdot \hat{g}^{\nu}$.

2. As the equality proofs consists of the following pairing equations:

$$
c_{0}=g^{\nu} \quad c_{1}=f^{\nu} \quad c_{i+1}=h^{\nu} \cdot m_{i}
$$

where $i \in\{1, \ldots, n\}$.

\begin{tabular}{|c|c|}
\hline Encryption key & $(10+n) \mathbb{G}+4 \hat{\mathbb{G}}$ \\
\hline Decryption key & $n \mathbb{Z}_{p}$ \\
\hline Ciphertext & $(6 n+19) \mathbb{G}+(16+4 n) \hat{\mathbb{G}}$ \\
\hline Verification equations & 2 linear $+n$ quadratic \\
\hline Size of the equality proof & $(2+2 n) \mathbb{G}+(2+4 n) \hat{\mathbb{G}}$ \\
\hline
\end{tabular}

Proof (of Theorem 16). The completeness and the correctness of the above RCCA encryption scheme are straightforward to verify. We will focusing on the Replayable-CCA property.

We proceed by the series of hybrid games $\mathrm{Game}_{0}, \ldots, \mathrm{Game}_{5}$, we denote by $\operatorname{Adv}_{i}$ the advantage of the adversary $\mathcal{A}$ to win the game $\mathrm{Game}_{i}$.

Game $_{0}$ : We have Game ${ }_{0}$ is identical to the original RCCA security game and thus by definition:

$$
\operatorname{Adv}_{0}=\operatorname{Adv}_{\mathcal{A}}^{\mathrm{RCCA}}\left(1^{\lambda}\right)
$$

Game $_{1}$ : In this game, we will modify the challenge ciphertext provided to the adversary in the RCCA security game. The new challenge ciphertext is:

$$
c^{\star}=\left(\left\{c_{i}^{\star}\right\}_{i=1}^{n}, \pi_{b}^{\star}, \pi_{\theta}^{\star}, \pi_{\vec{v}}^{\star}, \pi_{F G H}^{\star}, \pi_{\vec{w}}^{\star}\right)
$$

We only modify $\pi_{\vec{v}}^{\star}$ and $\pi_{\vec{w}}^{\star}$. Instead of generating these two proofs using the signing key $\left(\sigma_{\vec{v}_{1}}, \sigma_{\vec{v}_{2}}\right)$ of the LHSPS, we will use the signing key $t d$ to directly compute the signatures of $\vec{v}^{\star}$ and $\vec{w}$, where $\vec{v}^{\star}=\left(1,1, g, c_{2}, \ldots, c_{n+1}\right)$. (Notice that the secret signing key is never used in the real game.)

As this change is only conceptional, the distribution of the challenge ciphertext is identical in Game ${ }_{1}$ as in Game ${ }_{0}$. We have Adv ${ }_{1}=$ Adv $_{0}$

Game $_{2}$ : In this game, we modify the $c \vec{r} s$ of the Groth-Sahai proof system. We generate two random values $\xi, \zeta \stackrel{\$}{\leftarrow} \mathbb{Z}_{p}$, then compute $\vec{u}_{1}, \vec{u}_{2}, \hat{\vec{u}}_{1}, \hat{\vec{u}}_{2}$ such that $\vec{u}_{1}=\vec{u}_{2}^{\xi}$ and $\hat{\vec{u}}_{1}=\hat{\vec{u}}_{2}^{\zeta}$.

Notice that this is the perfect sound setting of the Groth-Sahai proof system. $\xi$ and $\zeta$ can be used to extract the witness. Since the only difference between Game ${ }_{1}$ and Game ${ }_{2}$ is the change of $\vec{u}_{1}, \vec{u}_{2}, \hat{\vec{u}}_{1}, \hat{\vec{u}}_{2}$, the indistinguishability can be proven using the SXDH assumption. Thus, we have $\operatorname{Adv}_{2} \leq \operatorname{Adv}_{1}+2$. $\operatorname{Adv}_{S X D H}$.

Game $_{3}$ : In this game, we modify the decryption oracle. We will add a manual verification of the underlying LHSPS for the decryption queries. To do this, since the Groth-Sahai proof is settled in the soundness mode $\left(\vec{u}_{1}=\vec{u}^{\xi}\right.$ and $\hat{\vec{u}}_{1}=\hat{\vec{u}}_{2}^{\zeta}$. We can use the trapdoors $\xi, \zeta$ to extract the witness in the commitments of the Groth-Sahai proof. We extract $\vec{v}$ and $\sigma_{\vec{v}}=(z, r)$ from the proof $\pi_{\vec{v}}$. We use the signing key $t d$ of the linearly homomorphic structurepreserving signature $\sigma_{\vec{v}}^{\dagger}=\left(z^{\dagger}, r^{\dagger}\right)$ to generate a signature $\sigma_{\vec{v}}^{\dagger}$ of the vector $\vec{v}$. The challenger will reject the decryption query if $\sigma_{\vec{v}}^{\dagger} \neq \sigma_{\vec{v}}$.

We can see that, if an adversary can distinguish Game ${ }_{3}$ from Game ${ }_{2}$ then he can forge a valid signature of the underlying LHSPS. Since the unforgeability of the LHSPS is based on the SXDH problem, we have $\operatorname{Adv}_{3} \leq \operatorname{Adv}_{2}+$ $\operatorname{Adv}_{D P}\left(1^{\lambda}\right)$.

Game $_{4}$ : We will modify all the decryption oracles (both pre-challenge and postchallenge ones) to avoid the use of $\log _{g}\left(h_{i}\right)=\alpha_{i}$. After making these changes, we can modify the generation of $h_{i}$ to $h_{i}=f^{x_{i}} g^{y_{i}}$.

Pre-challenge decryption queries: We use the trapdoor of the Groth-Sahai proof to extract the witness of the proof, if we have $b=0$ then we directly reject the proof.

Post-challenge decryption queries: We also use the trapdoor of the GrothSahai proof to extract the witness of the proof, if $b=0$ and the ciphertext is not rejected by the rule of $\mathrm{Game}_{3}$, the challenger outputs Replay. Additionally, both in pre-challenge and post-challenge decryption queries.

Since we don't have $\alpha_{i}$ anymore, we decrypt the ciphertext by computing $M_{i}=c_{i+1} /\left(c_{0}^{x_{i}} \cdot c_{1}^{y^{i}}\right)$.

We now analyse the change of the decryption oracles:

Pre-challenge: It is easy to see that in case of $b=0$, the challenger only issued two LHSPS signatures of $\vec{v}_{1}$ and $\vec{v}_{2}$. And the vector $\vec{v}$ is clearly not in the span of $\operatorname{Span}\left(\vec{v}_{1}, \vec{v}_{2}\right)$. So the adversary is statistically impossible to forge a correct signature.

Post-challenge: Note that the Groth-Sahai proof is in the perfect soundness setting of the Groth-Sahai proof, thus the challenger $\mathcal{C}$ can use the trapdoor to extract all the witness used in the proof. We will now separate two case:
- If $g^{b}=1$, we have $\vec{v}=\left(c_{0}, c_{1}, 1,1, \ldots, 1\right)$. But $\vec{c}$ is not rejected in the Game $_{3}$, with a overwhelming probability, we will have $\vec{v} \in \operatorname{Span}\left(\vec{v}_{1}\right)$. Thus we have $M_{i}=c_{i+1} /\left(c_{0}^{x} \cdot c_{1}^{y}\right)$.
- If $g^{b}=0$, we have $\vec{v}=\left(1,1, g, c_{2}, \ldots, c_{n+1}\right)$. As the third element is $g, \vec{v}=\vec{v} \cdot \vec{v}_{2}^{\theta} \cdot \vec{w}^{\rho}$. This means that $\vec{v}$ is a randomization of $\vec{v}^{\star}$, thus we can answer Replay to the adversary.

Game $_{5}$ : We modify the distribution of the challenge ciphertext. Instead of choosing them as an encryption of $\vec{M}_{0}$ or $\vec{M}_{1}$. We Choose them all random elements. By the self-rerandomizability of the DDH assumption in $\mathbb{G}$, the game 5 is indistinguishable from the game 4.

During the Game ${ }_{5}$, as the challenge ciphertext is only random group elements, the adversary cannot have more advantage than a random guess.

\section{B. 3 Instantiation of the encryption scheme $E$}

Let $G r=(p, \mathbb{G}, g)$.

E.KeyGen():

$-\left(d k_{1}, d k_{2}\right) \stackrel{\&}{\leftarrow} \mathbb{Z}_{p}^{2}$
- Return $\left(\left(g^{d k_{1}}, g^{d k_{2}}\right),\left(d k_{1}, d k_{2}\right)\right)$

$\operatorname{E} . \operatorname{Enc}\left(\left(D_{1}, D_{2}\right),\left(M_{1}, M_{2}\right), \nu\right)$ :
- Return $\left(g^{\nu}, M_{1} \cdot D_{1}^{\nu}, M_{2} \cdot D_{2}^{\nu}\right)$

E.ReRand $\left(\left(D_{1}, D_{2}\right),\left(C_{0}, C_{1}, C_{2}\right), \nu\right)$ :
- Return $\left(C_{0} \cdot g^{\nu}, C_{1} \cdot D_{1}^{\nu}, C_{2} \cdot D_{2}^{\nu}\right)$

$\operatorname{E} . \operatorname{Dec}\left(\left(d k_{1}, d k_{2}\right),\left(C_{0}, C_{1}, C_{2}\right)\right)$ :
- Return $\left(C_{0} \cdot C_{1}^{d k_{1}}, C_{2} \cdot C_{0}^{d k_{2}}\right)$

E.Verify $\left(\left(D_{1}, D_{2}\right),\left(M_{1}, M_{2}\right), \nu,\left(C_{0}, C_{1}, C_{2}\right)\right)$ :
- Return $\left(g^{\nu}, M_{1} \cdot D_{1}^{\nu}, M_{2} \cdot D_{2}^{\nu}\right)=\left(C_{0}, C_{1}, C_{2}\right)$

E.AdptPrf(ck, ek, $\left.\left(\operatorname{com}_{M_{1}}, \operatorname{com}_{M_{2}}\right), c, \tilde{\pi}=\left(\pi, \operatorname{com}_{\nu}\right), \nu^{\prime}\right)$ :
- Analog to B. 2

Proposition 42. If there exists an adversary $\mathcal{A}$ that breaks the IACR property of the scheme with advantage $\epsilon_{\mathrm{IACR}}$, then there exists an adversary $\mathcal{B}$ that breaks SXDH with advantage $\epsilon_{\mathrm{SXDH}}$, with

$$
\epsilon_{\mathrm{IACR}} \leq 4 \epsilon_{\mathrm{SXDH}}
$$

We define the following experiments:

```
$\operatorname{Expt}_{\mathcal{A}, b}^{\mathrm{IACR}}((\mathbb{G}, g, p)):$
    $\left(\left(P_{1}, P_{2}\right),\left(d k_{1}, d k_{2}\right)\right) \leftarrow \operatorname{KeyGen}(G r)$
    $\left(\left(C_{0}^{(0)}, C_{1}^{(0)}, C_{2}^{(0)}\right),\left(C_{0}^{(1)}, C_{1}^{(1)}, C_{2}^{(1)}\right)\right) \leftarrow \mathcal{A}\left(\left(P_{1}, P_{2}\right)\right)$
    $\nu \stackrel{\otimes}{\leftarrow} \mathbb{Z}_{p}$
    $\left(C_{0}, C_{1}, C_{2}\right) \leftarrow\left(C_{0}^{(b)} \cdot g^{\nu}, C_{1}^{(b)} \cdot P_{1}^{\nu}, C_{2}^{(b)} \cdot P_{2}^{\nu}\right)$
    $b^{\prime} \leftarrow \mathcal{A}\left(C_{0}, C_{1}, C_{2}\right)$
    Return $b^{\prime}$
$\operatorname{Expt}_{\mathcal{A}, b}^{\mathrm{IACR} V 2}((\mathbb{G}, g, p))$
    $\left(\left(P_{1}, P_{2}\right),\left(d k_{1}, d k_{2}\right)\right) \leftarrow \operatorname{KeyGen}(G r)$
    $\left(\left(C_{0}^{(0)}, C_{1}^{(0)}, C_{2}^{(0)}\right),\left(C_{0}^{(1)}, C_{1}^{(1)}, C_{2}^{(1)}\right)\right) \leftarrow \mathcal{A}\left(\left(P_{1}, P_{2}\right)\right)$
    $\nu, \nu_{2} \leftarrow \mathbb{Z}_{p}$
    $\left(C_{0}, C_{1}, C_{2}\right) \leftarrow\left(C_{0}^{(b)} \cdot g^{\nu}, C_{1}^{(b)} \cdot P_{1}^{\nu_{2}}, C_{2}^{(b)} \cdot P_{2}^{\nu}\right)$
    $b^{\prime} \leftarrow \mathcal{A}\left(C_{0}, C_{1}, C_{2}\right)$
    Return $b^{\prime}$
$\operatorname{Expt}_{\mathcal{A}, b}^{\mathrm{IACR} V 3}((\mathbb{G}, g, p)):$
    $\left(\left(P_{1}, P_{2}\right),\left(d k_{1}, d k_{2}\right)\right) \leftarrow \operatorname{KeyGen}(G r)$
    $\left(\left(C_{0}^{(0)}, C_{1}^{(0)}, C_{2}^{(0)}\right),\left(C_{0}^{(1)}, C_{1}^{(1)}, C_{2}^{(1)}\right)\right) \leftarrow \mathcal{A}\left(\left(P_{1}, P_{2}\right)\right)$
    $\nu, \nu_{2}, \nu_{3} \stackrel{\Phi}{\leftarrow} \mathbb{Z}_{p}$
    $\left(C_{0}, C_{1}, C_{2}\right) \leftarrow\left(C_{0}^{(b)} \cdot g^{\nu}, C_{1}^{(b)} \cdot P_{1}^{\nu_{2}}, C_{2}^{(b)} \cdot P_{2}^{\nu_{3}}\right)$
    $b^{\prime} \leftarrow \mathcal{A}\left(C_{0}, C_{1}, C_{2}\right)$
    Return $b^{\prime}$
```

By noticing that $\left|\operatorname{Pr}\left(\operatorname{Expt}_{\mathcal{A}, b}^{\mathrm{IACR}}((\mathbb{G}, g, p))=1\right)-\operatorname{Pr}\left(\operatorname{Expt}_{\mathcal{A}, b}^{\mathrm{IACR} V 2}((\mathbb{G}, g, p))=1\right)\right|$ and $\mid \operatorname{Pr}\left(\operatorname{Expt}_{\mathcal{A}, b}^{\mathrm{IACR} V 2}((\mathbb{G}, g, p))=1\right)-\operatorname{Pr}\left(\boldsymbol{E x p t}_{\mathcal{A}, b}^{\mathrm{IACR} V 3}((\mathbb{G}, g, p))=1 \mid\right.$ are less or equal to $\epsilon_{\mathrm{SXDH}}$, and because $\operatorname{Expt}_{\mathcal{A}, 0}^{\mathrm{IACR} V 3}((\mathbb{G}, g, p))$ and $\operatorname{Expt}_{\mathcal{A}, 1}^{\mathrm{IACR} V 3}((\mathbb{G}, g, p))$ are distributed equally, we deduce $\epsilon_{\mathrm{IACR}} \leq 4 \epsilon_{\mathrm{SXDH}}$.

\section{Efficiency analysis}

We summarize the efficiency of the the building blocks $\mathrm{C}, \mathrm{S}, \mathrm{S}^{\prime}$ and E in Tables 1, 2 and 3 , where "m-s" stands for "multi-scalar".

\section{Computational assumptions}

Definition 43 (SXDH). The Symmetric External Diffie-Hellman Assumption states that given $\left(g^{r}, g^{s}, g^{t}\right)$ for random $r, s \in \mathbb{Z}_{p}$, it is hard to decide whether $t=r s$ or $t$ is random; moreover, given $\left(\hat{g}^{r^{\prime}}, \hat{g}^{s^{\prime}}, \hat{g}^{t^{\prime}}\right)$ for random $r^{\prime}, s^{\prime} \in \mathbb{Z}_{p}$, it is hard to decide whether $t^{\prime}=r^{\prime} s^{\prime}$ or $t^{\prime}$ is random.

The Asymetric Double Hidden Strong Diffie Hellman (ADHSDH) assumption and the Asymetric Weak Flexible Computational Diffie Hellman (AWFCDH) assumption have been introduced in $\left[\mathrm{AFG}^{+} 10\right]$.

Table 1. Sizes of components of the commit-and-prove scheme C

\begin{tabular}{|c|c|}
\hline$|c k|$ & $3|\mathbb{G}|+3|\hat{\mathbb{G}}|$ \\
\hline$\left|\mathrm{Cm}\left(g_{1}\right)\right|$ & $2|\mathbb{G}|$ \\
\hline$|\mathrm{Cm}(\hat{g})|$ & $2|\hat{\mathbb{G}}|$ \\
\hline$\left|\mathrm{Cm}\left(1_{\mathbb{Z}_{p}}\right)\right|$ & $2|\hat{\mathbb{G}}|$ \\
\hline Homogeneous pairing product equation with variables in $\mathbb{G}$ & $2|\hat{\mathbb{G}}|$ \\
\hline Homogeneous pairing product equation with variables in $\hat{\mathbb{G}}$ & $2|\mathbb{G}|$ \\
\hline General homogeneous pairing product equation & $4|\mathbb{G}|+4|\hat{\mathbb{G}}|$ \\
\hline M-s equation in $\mathbb{G}$ with variables in $\mathbb{Z}_{p}$ & $|\mathbb{G}|$ \\
\hline Homogeneous m-s equation in $\hat{\mathbb{G}}$ with variables in $\hat{\mathbb{G}}$ & $2|\mathbb{Z}|$ \\
\hline General m-s equation in $\mathbb{G}$ & $2|\mathbb{G}|+4|\hat{\mathbb{G}}|$ \\
\hline
\end{tabular}

Table 2. Characteristics of the signature schemes $S$ and $S^{\prime}$ for message spaces $\mathcal{M}^{\prime}=\hat{\mathbb{G}}$ and $\mathcal{M}=\left\{\left(g^{m}, \hat{g}^{m}\right) \mid m \in \mathbb{Z}_{p}\right\}^{2}$, resp.

\begin{tabular}{|c|c|c|}
\hline Signature scheme & S [Fuc11] & S $^{\prime}[$ AGHO11] \\
\hline $\mid$ par $\mid$ & $3|\mathbb{G}|$ & 0 \\
\hline$|s k|$ & $|\mathbb{Z}|$ & $3\left|\mathbb{Z}_{p}\right|$ \\
\hline $\mid$ vk $\mid$ & $|\mathbb{G}|+|\hat{\mathbb{G}}|$ & $3|\hat{\mathbb{G}}|$ \\
\hline$|\sigma|$ & $13|\mathbb{G}|+9|\hat{\mathbb{G}}|$ & $2|\mathbb{G}|+|\hat{\mathbb{G}}|$ \\
\hline Nb of pairing eqs in S.Verify & 12 general equations & 1 linear in $\hat{\mathbb{G}}, 1$ general \\
\hline$\left|\pi_{\sigma}\right|$ & $48|\mathbb{G}|+48|\hat{\mathbb{G}}|$ & $6|\mathbb{G}|+4|\hat{\mathbb{G}}|$ \\
\hline
\end{tabular}

Table 3. Characteristics of the ElGamal encryption $E^{\prime}$ with message space $\mathbb{G}^{2}$

\begin{tabular}{|c|c|}
\hline$|s k|$ & $2\left|\mathbb{Z}_{p}\right|$ \\
\hline$|p k|$ & $2|\mathbb{G}|$ \\
\hline$|c|$ & $3|\mathbb{G}|$ \\
\hline$|\nu|$ & $\left|\mathbb{Z}_{p}\right|$ \\
\hline Nb ms eqs in E.Verify & 2 general equations 1 linear with unkown in $\mathbb{Z}_{p}$ \\
\hline$\left|\tilde{\pi}_{\text {eq }}\right|$ & $5|\mathbb{G}|+10|\hat{\mathbb{G}}|$ \\
\hline
\end{tabular}

Definition 44 ( $q$-ADHSDH). Given $\left(g, f, k, x=g^{\xi}, \hat{g}\right) \stackrel{\otimes}{\leftarrow} \mathbb{G}^{4} \times \hat{\mathbb{G}}$ and $\hat{y}=\hat{g}^{\xi}$ and $\left(a_{i}=\left(k g^{\omega_{i}}\right)^{\frac{1}{\xi+\gamma_{i}}}, c_{i}=f^{\gamma_{i}}, v_{i}=g^{\omega_{i}}, \hat{d}_{i}=\hat{g}^{\gamma_{i}}, \hat{w}_{i}=\hat{g}^{\omega_{i}}\right)_{i=1}^{q}$, for $\gamma_{i}, \omega_{i} \stackrel{\&}{\leftarrow}$, it is hard to output a new tuple $(a, c, v, \hat{d}, \hat{w}) \in \mathbb{G}^{3} \times \hat{\mathbb{G}}^{2}$ of this form, i.e., a tuple that satisfies

$$
e(a, \hat{y} \hat{d})=e(k v, \hat{g}) \wedge e(c, \hat{g})=e(f, \hat{d}) \wedge e(v, \hat{g})=e(g, \hat{w})
$$

Definition 45 (AWFCDH). Given random generators $\left(g, a=g^{\alpha}, \hat{g}\right) \stackrel{\leftarrow}{\leftarrow}\left(\mathbb{G}^{*}\right)^{2}$ $\times \hat{\mathbb{G}}$, it is hard to output $\left(g^{\nu}, g^{\nu \alpha}, \hat{g}^{\nu}, \hat{g}^{\nu \alpha}\right)$, i.e., a tuple $(r, m, \hat{s}, \hat{n})$ that satisfies:

$$
e(a, \hat{s})=e(m, \hat{g}) \wedge e(m, \hat{g})=e(g, \hat{n}) \wedge e(r, \hat{g})=e(g, \hat{s})
$$