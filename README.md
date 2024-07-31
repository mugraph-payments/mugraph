<p align="center">
  <picture>
    <source srcset="support/assets/logo-white.svg" media="(prefers-color-scheme: dark)">
    <img src="support/assets/logo-dark.svg" alt="Mugraph Logo" width="300">
  </picture>

<p align="center"><em>Instant, untraceable payments for Cardano.</em></p>

<p align="center">
    <img src="https://github.com/mugraph-payments/mugraph/actions/workflows/build.yml/badge.svg" alt="Build Status" />
    <a href="https://opensource.org/licenses/Apache-2.0">
      <img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="Apache 2.0 Licensed" />
    </a>
    <a href="https://opensource.org/licenses/MIT">
      <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="MIT Licensed" />
    </a>
    <a href="https://discord.gg/npSJU6Qk">
      <img src="https://dcbadge.limes.pink/api/server/npSJU6Qk?style=social" alt="Mugraph Discord Server" />
    </a>
  </p>
</p>

Mugraph (pronounced *"mew-graph"*) is a Layer 2 Network for the Cardano
blockchain for untraceable payments with instant finality.

By **untraceable**, we mean that, inside a given group $A$ of users:

- All transactions between users inside $A$ are untraceable, meaning senders
and receivers are not bound in any way.
- All transactions to users outside $A$ come from a single, shared identity for
all group participants.

This shared identity comes from **Delegates**, similar to Payment Networks in
the traditional banking system, like Paypal, Venmo or CashApp, but with some
crucial distinctions:

- Delegates hold funds in a fully auditable Smart Contract Vault, held in the
  Layer 1.
- Delegates can not spend user funds without their authorization.
- Delegates are **blind**, meaning they don't know who is transacting.
- Delegates provide **group concealing** for their users, signing transactions
  on behalf of them.

An user can, and usually will, hold balance in multiple Delegates at once, and
they do not need to have balance in a Delegate to receive payments there.

## Motivation

> Why are people not using cryptocurrencies for payments?

This is the question I wanted to address when Mugraph started. Bitcoin was
created to address this specific problem, yet [The Lightning
Network](https://lightning.network) is not widely supported even in countries
that adopted Bitcoin as legal tender.

Even if Cardano has much more advanced technology than Bitcoin does (like eUTXO
or Smart Contracts), buying groceries with [USDM](https://mehen.io) remains
nighly impossible.

ZeroHedge explains this phenomenon in their article ["What Happened to
Bitcoin?"](https://www.zerohedge.com/crypto/what-happened-bitcoin):

> At the same time, new technologies were becoming available that vastly
> improved the efficiency and availability of exchange in fiat dollars. They
> included Venmo, Zelle, CashApp, FB payments, and many others besides, in
> addition to smartphone attachments and iPads that enabled any merchant of any
> size to process credit cards. These technologies were completely different
> from Bitcoin because they were permission-based and mediated by financial
> companies. But to users, they seemed great and their presence in the
> marketplace crowded out the use case of Bitcoin at the very time that my
> beloved technology had become an unrecognizable version of itself.

Excluding volatility (already being taken seriously by Stablecoins), we
identified three main problems that prevent the average person from considering
cryptocurrencies as a payment option:

1. **Scalability**: Cryptocurrency transactions are slow compared to
   centralized solutions. For example, credit cards have a practical
confirmation limit of 2 seconds.

1. **Privacy**: Users do not want to reveal their identity for every purchase.
   Financial privacy is a human right, and in the age of big data analysis and
AI, pseudonymity does not provide sufficient privacy.

1. **Ease of Use**: Users prefer not to deal with complex protocols, multiple
   wallets, or extensive security considerations.

I think that Mugraph has a real shot of solving those problems, and that's why
I'm building it.

## Overview

> [!NOTE]
> This overview describes a variant of the blind signatures protocol proposed
> by David Wagner here[2]. For all practical terms, they are equivalent.

In 1983, David Chaum conceived ECash, arguably the first form of
cryptographically secure digital money, in his paper [Blind Signatures for
Untraceable Payments][1]. In it, David Chaum describes a bank (called a Mint)
that emit **Bearer Tokens**.

They are called "bearer" tokens because, like physical cash, possession of the token itself confers the right to redeem its value, and are created through a process called **Blind Signature**.

### Blind Signatures

A Blind Signature is a signature in which the signer is able to verify
attribution (that they in fact signed it at some point), while not revealing
what signed message created that input. Following the usual conventions for
cryptographic examples, we have Alice (the user) and Bob (the Mint). Given
that:

- $G$ is a publicly available generator point on the Ristreto25519 curve,
- Bob has a public/private keypair $k, K$, and $K$ is publicly available.
- Alice has a secret $x$, representing her deposit on Bob.

Then:

1. Alice computes $Y = H(x)$, where $H$ is a function that maps the secret to a
   point on the elliptic curve (hash to curve).
1. Alice generats a random blinding factor $r$.
1. Alice the blinded point $B' &= Y + r \cdot G$:
1. Alice sends $B'$ to Bob.
1. Bob receives $B'$ and computes the blinded signature $C' &= k \cdot B'$.
1. Bob sends $C'$ back to Alice.
1. Alice unblinds the signature by subtracting $r \cdot K$ from $C'$:

$$
\begin{align}
C &= C' - r \cdot K \\
  &= k \cdot B' - r \cdot K \\
  &= k \cdot (Y + r \cdot G) - r \cdot (k \cdot G) \\
  &= k \cdot Y + k \cdot r \cdot G - r \cdot k \cdot G \\
  &= k \cdot Y
\end{align}
$$

To verify the signature, Alice (or any verifier) can check if $C = k \cdot
H(x)$. If this equality holds, it proves that $C$ originated from Bob's private
key $k$, without Bob knowing the original message $x$.

This protocol has been implemented multiple times before, notably in the [
Protocol](https://cashu.space), running on top of the Lightning Network. It is
very cryptographically solid, even 40+ years after it was created, but it also
has some very known flaws:

1. Mints are in total control of issuing tokens, and the only thing barring
   them from practicing fractional reserves is the risk of a Bank Run.
2. Mints don't know which deposit they are redeeming, but they know what asset
   is being transacted and the amounts. Those increase risk of deanonimization
   with data and pattern analysis.

## Protocol

Mugraph differentiates itself from ECash because we use **Zero-Knowledge
Proofs**, specifically ZK-SNARKS, to move the state forward. This means:

1. Instead of Mints, we have **Delegates**. They don't emit tokens
   or control the funds at all, instead they only verify and apply proofs.
2. Users do all operations inside the system, delegates only protect against
   double-spend.
3. Notes themselves can have programs, allowing for arbitrary conditions on
   spending.

### Delegates

The equivalent to Mints in Mugraph are **Delegates**.

1. Verifying *operation proofs* and signing **Blinded Notes**.
1. Signing external transctions on behalf of the user.
1. Emitting **Notes** in response to user deposits.

### Default Programs

#### $F$: Fission

Splits a note into two blinded notes. It is defined as:

$$
F(n, i) \mapsto { n'_o, n'_c }
$$

Where:

- $n$ is the input note to be slit in two
- $i$ is the output amount requested by the operation
- $n'_o$ is a blinded note for the amount $i$
- $n'_c$ is another blinded note for the amount $n_i - i$, where $n_i$ is the
  note amount.

#### $F'$: Fusion

Joins two notes with the same Asset ID and server keys into a single one. It is
defined as:

$F'(n_0, n_1) \mapsto n'$

Where:

- $n_0$ and $n_1$ are the input notes to be fused
- $n'$ is a blinded node for the amount $n_0i + n_1i$

Mugraph (pronounced *"mew-graph"*) is a Layer 2 Network for the Cardano blockchain for untraceable payments with instant finality. It is very simplified in both operations and architecture, meant to be easy to integrate anywhere.

## Glossary

| Symbol | Description                                                             |
|--------|-------------------------------------------------------------------------|
| $G$    | A generator point in the Ristreto25519 curve.                           |
| $n$    | A Note, blindly signed by the Delegate and ready to be used.            |
| $n'$   | A Note with a blinded nullifier to be sent to the Delegate for signing. |

## Further Reading

1. [Roadmap](./support/roadmap.md)
1. [Licensing](./support/licensing.md)

[1]: ./support/reference-material/papers/blind-signatures-for-untraceable-payments.md
[2]: ./support/reference-material/blind-dh.md
