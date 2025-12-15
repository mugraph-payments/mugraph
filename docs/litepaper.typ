#set page(
  paper: "a4",
  margin: (x: 1.15in, y: 1.1in),
)

#set text(
  size: 11pt,
)

#set heading(numbering: "1.")
#set par(justify: true, leading: 0.75em)

#let title = "Mugraph Litepaper"
#let subtitle = "Instant, untraceable payments for Cardano"
#let version = "Draft"
#let date = "2025-12-15"
#let repo = "https://github.com/mugraph-payments/mugraph"

#align(center)[
  #v(1.25em)
  #text(size: 28pt, weight: "bold")[#title]
  #v(0.35em)
  #text(size: 13pt)[#emph[#subtitle]]
  #v(1.25em)
  #text(size: 10pt)[#version · #date]
  #v(0.75em)
  #text(size: 10pt)[Repository: #link(repo)[#repo]]
  #v(2.0em)
  #text(size: 9pt)[
    This document is a high-level description of Mugraph’s goals and current
    prototype. It is not an audited specification and must not be used as the
    basis for production deployments.
  ]
]

#pagebreak()

= Abstract

Mugraph (pronounced _“mew-graph”_) is a Layer 2 payment network being built for
Cardano with the goal of making everyday payments feel as fast as centralized
apps while improving financial privacy.

The core idea is to introduce _delegates_: service providers that escrow funds in
a Layer 1 vault and issue short-lived, bearer-like payment instruments for
off-chain transfers. Delegates must be unable to spend user funds unilaterally,
and—critically—should not be able to link senders to receivers inside a delegate
group.

This litepaper explains the intended architecture, the protocol shape that the
current repository prototypes, and the roadmap to a Cardano-integrated design.

= Goals

- *Instant UX:* off-chain payments with sub-second confirmation.
- *Untraceable within a group:* transactions between users of the same delegate
  should not reveal sender↔receiver links.
- *Group concealing:* payments to outsiders should appear as coming from a shared
  delegate identity.
- *Multi-asset:* support ADA and native assets.
- *Practical operations:* simple node software and wallet flows.

= Non-goals (for this document)

- A final cryptographic specification.
- A full Cardano on-chain design (vaults, withdrawals, dispute flows).
- Production security guarantees; the current implementation is a prototype.

#pagebreak()

#outline(title: "Contents")

#pagebreak()

= Problem Statement

Payments are a “solved problem” in the fiat world: confirmation is near-instant
and user experience is polished. Public blockchains, meanwhile, trade off speed
and privacy for decentralization: settlement is slower, and transaction graphs
are highly analyzable.

Mugraph’s starting point is a pragmatic question: _how do we get credit-card
speed, with stronger privacy, while remaining compatible with Cardano’s Layer 1
settlement?_

= High-level Architecture

Mugraph is organized around three roles:

- *Users* hold and transfer off-chain payment instruments (notes).
- *Delegates* validate spends, prevent double-spends, and (eventually) interface
  with Layer 1 vaults.
- *Cardano Layer 1* provides auditable custody (vault smart contracts) and final
  settlement to and from delegates.

At a high level:

1. A user deposits assets into a delegate-controlled vault on Cardano (planned).
2. The delegate issues off-chain notes representing claims on the vault.
3. Users transfer notes off-chain; the delegate enforces uniqueness (no
   double-spend) during re-issuance.
4. Users can withdraw by burning notes and redeeming from the vault (planned).

= Core Objects (Prototype)

The current codebase prototypes an e-cash-like flow around two objects:

- *Note:* a signed claim on `(delegate, asset, amount, nonce)`.
- *Refresh transaction:* consumes existing notes and creates new notes (split,
  merge, pay-with-change).

In the repository, these live in `mugraph-core`:

```text
Note {
  amount: u64,
  delegate: PublicKey,
  asset_id: Hash,
  nonce: Hash,
  signature: Signature,
}
```

```text
Refresh {
  input_mask: BitSet32,          // marks which atoms are inputs
  atoms: Vec<Atom>,              // inputs + outputs
  asset_ids: Vec<Hash>,          // dictionary for atom asset_id indices
  signatures: Vec<Signature>,    // signatures for input atoms
}
```

This form makes it easy to prototype correctness and double-spend prevention.
Future iterations are expected to replace explicit amounts with commitments and
zero-knowledge proofs (see Roadmap).

= Protocol Sketch

This section describes the intended “note re-issuance” flow. The prototype
implements the same shape, with simplified cryptography.

== Setup

Each delegate runs a node with a long-term keypair:

- secret key `sk_d`
- public key `pk_d`

The delegate also maintains a database of spent note identifiers to prevent
double-spending.

== Mint (Deposit → Notes)

Planned flow:

1. User deposits assets into a delegate vault on Cardano.
2. User proves the deposit (e.g., with a chain proof / Mithril-style proof of
   inclusion).
3. Delegate issues notes to the user equal to the deposited value.

Prototype shortcut:

- The node exposes an `emit` RPC that issues notes directly for simulation.

== Pay / Refresh (Notes → New Notes)

To pay someone, a user forms a balanced refresh transaction:

- Inputs: one or more notes currently owned by the sender.
- Outputs: one note for the recipient plus a change note back to the sender (or
  more outputs for splitting).

The delegate verifies:

- Each input signature is valid under `pk_d`.
- Each input has not been spent before.
- The transaction is balanced per asset (sum(inputs) = sum(outputs)).

If valid, the delegate marks inputs as spent and returns signatures for each
output, which materialize into new notes.

== Why re-issuance?

Re-issuance turns “bearer” notes into fresh notes with new nonces. With proper
blinding (next section), this can break linkability between what went in and
what came out, while still enforcing that value is conserved and double-spends
are rejected.

= Privacy Design (Intended)

Mugraph aims for two main privacy properties:

- *Untraceable within a group:* a delegate should not be able to link the sender
  of a payment to the receiver based on protocol messages alone.
- *Group concealing:* on-chain interactions can be aggregated so that outside
  observers see “the delegate” rather than individual users.

== Blind issuance / blind re-issuance

The design notes in `support/` describe using a blind-signing construction often
called _Blind Diffie–Hellman Key Exchange (BDHKE)_.

Informally:

1. The wallet commits to a secret message `m` (e.g., note preimage).
2. The wallet blinds `m`, sending a blinded point `B'` to the delegate.
3. The delegate signs `B'` and returns `C'` (plus a proof, see below).
4. The wallet unblinds, obtaining a signature `C` on `m` that the delegate
   cannot link to `B'`.

== Correctness proofs (DLEQ)

To prevent malformed signing responses, the delegate can attach a
_discrete-log equality proof (DLEQ)_ showing the same secret key was used
consistently (i.e., `pk_d = sk_d · G` and `C' = sk_d · B'`).

== Amount privacy (commitments + ZK)

The longer-term design (see `support/workflow.md`) anticipates hiding amounts
from delegates by using value commitments and aggregated range proofs, while
still proving per-asset balance.

= Trust Model and Security Considerations

- *Delegate availability:* users need a delegate online to refresh/transfer
  notes. Delegates can censor, rate-limit, or go offline.
- *Double-spend prevention:* delegates must store and enforce “spent” status for
  note identifiers; this is a core security responsibility.
- *Custody and vault safety (planned):* delegates should escrow in auditable
  on-chain vaults and must not be able to spend user funds without
  authorization.
- *Metadata:* network-level metadata (IP addresses, timing, routing) can leak
  information unless mitigated at higher layers.
- *Prototype status:* the current repository is not audited and does not yet
  implement the full set of planned cryptographic proofs.

= Implementation Status (This Repository)

The current workspace contains:

- `mugraph-core`: data types and cryptographic utilities used by the protocol
  prototype.
- `mugraph-node`: a delegate node with a minimal HTTP RPC (`/rpc`) implementing
  `emit`, `refresh`, and `public_key`.
- `mugraph-simulator`: a terminal UI that spawns wallets and repeatedly performs
  refresh-based payments against a node to exercise correctness and throughput.

Run the prototype locally (two terminals):

```bash
cargo run -p mugraph-node -- server --addr 127.0.0.1:9999
```

```bash
cargo run -p mugraph-simulator -- --node-url http://127.0.0.1:9999
```

= Roadmap (Short)

The repository’s `support/roadmap.md` outlines milestones. In simplified form:

1. *Intra-node payments:* correctness, proofs, and a stable node/wallet protocol.
2. *Asset bridging:* Cardano vault contracts for deposits and withdrawals.
3. *Cross-node payments:* routing between delegates (e.g., Hydra-based channels).
4. *Wallet UX:* mobile wallet and merchant flows (QR/NFC).
5. *Specifications:* formal protocol docs, audits, and testnet readiness.

= References

- Project repository: #link(repo)[#repo]
- Concept note: `support/concepts/blind-diffie-hellman.md`
- Protocol workflow draft: `support/workflow.md`
- Roadmap draft: `support/roadmap.md`
