<p align="center">
  <picture>
    <source srcset="assets/logo-white.svg" media="(prefers-color-scheme: dark)">
    <img src="assets/logo-dark.svg" alt="Mugraph Logo" width="300">
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

Mugraph is a layer 2 for Cardano that handles private, instant payments in a
custodial manner. In a nutshell, a server/node (called a **Delegate**) holds
funds in behalf of users and in exchange issues Bearer Tokens to those same
users.

Those tokens (called Notes) can be exchanged between users, instantaneously and
even off-line (with some safeguards which we'll discuss later). Through a
mechanism called **Blind Signatures**, those payments are private, in the sense
that no one (not even the Delegate) knows any information about them beyond the
asset being transacted and the amount.

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

## Notes and Transactions

A Note is a simple string of bytes. It can be consumed to generate new notes,
as part of a transaction. When serialized over the wire it looks like this JSON
object:

```json
{
  "amount": 42,
  "delegate": "1111111111111111111111111111111111111111111111111111111111111111",
  "asset_id": {
    "policy_id": "22222222222222222222222222222222222222222222222222222222",
    "asset_name": "USD"
  },
  "nonce": "3333333333333333333333333333333333333333333333333333333333333333",
  "signature": "4444444444444444444444444444444444444444444444444444444444444444",
  "dleq": {
    "e": "5555555555555555555555555555555555555555555555555555555555555555",
    "z": "6666666666666666666666666666666666666666666666666666666666666666",
    "r": "7777777777777777777777777777777777777777777777777777777777777777"
  }
}
```

Notes sent to the Delegate are considered consumed, and can not be used again.
They can also be sent between users, without ever touching the Delegate (or the
Internet, for that matter). Doing so will not prevent against double-spending,
but is a valid option when users trust each other.

## Cryptography (Blind Signatures, DLEQ Proofs)

Notes are signed, to ensure their provenance from a specific delegate, however
they use **Blind Signatures**. Unlike normal signatures, blind signatures go
through a process of blinding and unblinding, in which the message the Delegate
signs is not the same one it verifies, even if they are signed and verified
with the same key.

Notes signed using this scheme are untraceable, in the sense that the delegate
has no way to know which message it signed that created the note, even if it
can verify the signature from the note is valid using it's own key.

The specific blind signature protocol used on Mugraph is called Blind
Diffie-Hellman Key Exchange (BDHKE).

### Blind Diffie-Hellman Key Exchange (BDHKE)

Blind Diffie-Hellman Key Exchange (BDHKE) protocol is a cryptographic method
that allows two parties to establish a shared secret key without revealing
their individual private keys. BDHKE is used on Mugraph to guarantee group
concealing, making the server oblivious to the identity of the note owner,
severing the connection between inputs from one transaction to the next. The
BDHKE protocol was described in a 1996 cypherpunk mailing list post by David
Wagner. It was devised as an alternative to RSA blinding to circumvent the
now-expired patent by David Chaum.

#### Overview

BDHKE is an extension of the traditional Diffie-Hellman key exchange,
incorporating a blinding factor to make it blind (meaning the message can be
verified without knowning which original message generated it). The protocol
involves two main parties:

1. Alice: The user who initiates the key exchange and wants to obtain a blindly
   signed value.
2. Bob: The Delegate, who performs the blind signing operation.

The goal is for Bob to sign Alice's message without knowing its content, while
Alice can later prove that the signature came from Bob.

#### Protocol Steps

##### 1. Initial Setup

Alice and Bob agree on a common elliptic curve group with a generator point
$G$.

##### 2. Key Generation

Alice generates a private key $a$ and computes the corresponding public key
$A$:

$$
\begin{aligned}
A &= a \cdot G
\end{aligned}
$$

Alice sends $A$ to Bob.

Bob generates a private key $k$ and computes the corresponding public key $K$:

$$
\begin{aligned}
K &= k \cdot G
\end{aligned}
$$

Bob makes $K$ publicly available.

##### 3. Blinding

Alice performs the following steps:

1. Choose a secret message $x$. In ecash protocols, this message is remembered
   to prevent double spending.
2. Compute $Y = H(x)$, where $H$ is a function that maps the secret to a point
   on the elliptic curve (hash to curve).
3. Generate a random blinding factor $r$.
4. Compute the blinded point $B'$:

$$
\begin{aligned}
B' &= Y + r \cdot G
\end{aligned}
$$

Alice then sends $B'$ to Bob.

##### 4. Signing

Bob receives $B'$ and computes the blinded signature $C'$:

$$
\begin{aligned}
C' &= k \cdot B'
\end{aligned}
$$

Bob sends $C'$ back to Alice.

##### 5. Unblinding

Alice unblinds the signature by subtracting $r \cdot K$ from $C'$:

$$
\begin{aligned}
C &= C' - r \cdot K \\
  &= k \cdot B' - r \cdot K \\
  &= k \cdot (Y + r \cdot G) - r \cdot (k \cdot G) \\
  &= k \cdot Y + k \cdot r \cdot G - r \cdot k \cdot G \\
  &= k \cdot Y
\end{aligned}
$$

##### 6. Verification

To verify the signature, Alice (or any verifier) can check if:

$$C = K \cdot H(x)$$

If this equality holds, it proves that $C$ originated from Bob's private key $k$, without Bob knowing the original message $x$.

### Discrete Log Equality Proof (DLEQ Proof)

To prevent potential attacks where Bob might not correctly generate $C'$, an additional step is included:

Bob provides a Discrete Log Equality Proof (DLEQ) to demonstrate that the $k$ in $K = k \cdot G$ is the same $k$ used in $C' = k \cdot B'$. This proof can be implemented using a Schnorr signature as follows:

1. Bob generates a random nonce $r$.
2. Bob computes:

$$
\begin{aligned}
R_1 &= r \cdot G \\
R_2 &= r \cdot B' \\
e &= \text{hash}(R_1, R_2, K, C') \\
s &= r + e \cdot k
\end{aligned}
$$

3. Bob sends $e$ and $s$ to Alice.
4. Alice verifies the proof by checking:

$$
\begin{aligned}
R_1 &= s \cdot G - e \cdot K \\
R_2 &= s \cdot B' - e \cdot C' \\
e &= \text{hash}(R_1, R_2, K, C')
\end{aligned}
$$

If the verification passes, Alice can be confident that Bob correctly generated $C'$.

## Running

### With Docker

TODO.

## Developing

All of the core development team uses [Nix](https://nixos.org) to set up the development environment, so changes in the environment setup are more likely to appear there first. With that being said, Mugraph uses stable Rust, which you can install with [rustup](https://rustup.rs):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then, you can build the application using `cargo build`, as expected.

### With Nix

First, you will need [Nix](https://nixos.org) installed, which you can do with the [Determinate Systems Nix Installer](https://github.com/DeterminateSystems/nix-installer), like so:

```sh
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Then, you can run this command to spawn a development shell:

```sh
nix develop
```

You can also install [direnv](https://direnv.net/) do do this automatically when you `cd` to the folder. You can now build the application using Cargo:

```sh
cargo build
```

## Licensing

### Software

Mugraph, as well as all projects under the `mugraph-payments` is dual-licensed under the [MIT](./LICENSE) and [Apache 2.0](./LICENSE-APACHE) licenses.

This should cover most possible uses for this software, but if you need an exception for any reason, please do get in touch.

### Logo

The project logo uses the [Berkeley Mono Typeface](https://berkeleygraphics.com/), under a [Developer License](https://cdn.berkeleygraphics.com/static/legal/licenses/developer-license.pdf).

All graphics we create are also licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/?ref=chooser-v1). It only requires attribution, but if this license is a problem for your use-case, get in touch.
