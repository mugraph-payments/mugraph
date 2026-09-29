# Project Close-out Report

## Name of project and project URL

µgraph: Instant, Untraceable Payments in Cardano

- Proposal: https://projectcatalyst.io/funds/12/f12-cardano-use-cases-mvp/graph-instant-untraceable-payments-in-cardano
- Milestones: https://milestones.projectcatalyst.io/projects/1200258
- Code: https://github.com/mugraph-payments/mugraph
- Documentation: https://mugraph.dev

## Project number

1200258 (Fund 12, Cardano Use Cases: MVP)

## Name of project manager

Cainã Costa

## Date project started

12 August 2024

## Date project completed

September 2026

## List of challenge KPIs and how the project addressed them

The Fund 12 category "Cardano Use Cases: MVP" asked projects to take a
prototype to the stage of a minimum viable product (MVP), and to show that it
works on Cardano.

| Challenge goal                                             | How µgraph addressed it                                                                                                                                                                     |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Develop and test the technical feasibility of the solution | The project built a delegate node, a vault validator in Aiken, and a wallet app. It has 489 Rust tests and 14 Aiken tests.                                                                  |
| Show that the innovation works                             | An end-to-end test runs a deposit, a payment between two wallets, a second use of the same notes, and a withdrawal. After each step, it checks that the notes equal the funds in the vault. |
| Show that the MVP is usable on Cardano                     | Deposits and withdrawals are Cardano transactions with a Plutus V3 validator. The node supports the preprod network through Blockfrost. The Docker image starts a node in full mode.        |
| Deploy first on the test networks                          | Version 1.0 is for preprod and preview. The project did not deploy it on the main network.                                                                                                  |
| Privacy and scaling (areas of interest)                    | Blind signatures hide the payer from the delegate. Payments settle off the chain, in one request to the delegate.                                                                           |

## List of project KPIs and how the project addressed them

| Milestone | Planned output                               | Result                                                                                                                                                               |
| --------- | -------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1         | Payments in one node, with private transfers | Done. The node, the simulator and the Docker image. The project replaced the zero-knowledge proofs with blind signatures (change request of December 2025).          |
| 2         | Deposit and withdraw on Cardano              | Done. The Aiken vault validator, the deposit and withdrawal flows, and preprod transactions.                                                                         |
| 3         | Payments between nodes                       | Partly done. The specifications, the messages, the authentication and the state machines exist. The node does not yet check the settlement transaction on the chain. |
| 4         | A mobile wallet                              | Done for Android and iOS builds. QR codes and NFC payments are not in the wallet. Users move notes as text.                                                          |
| 5         | Version 1.0, with documentation              | Done. See the section "Key achievements".                                                                                                                            |

## Key achievements

**An e-cash system on Cardano.** A user deposits ADA into a vault,
pays other users with private notes, and withdraws ADA from the vault. The
delegate can not link a note to the user who got it.

**A security review before the release.** In the final milestone, a review of
the code found problems that let a user make value from nothing. It also found
problems that stopped real-network use. The project fixed each one with a test
that showed the problem first:

- The hash to the group had a public discrete log, so anyone could forge a note.
- The node did not refuse a refresh with an invalid signature.
- One key signed all amounts, so a deposit could give notes of any value.
- A withdrawal did not verify the notes that it burned.
- The vault validator let a depositor take the deposit back alone.
- The node saw the nonce of each new note, so it could link notes.
- The vault address was a reward address, and withdrawals had no Plutus witness.

**Tests that match a real chain.** The mock chain now checks witnesses,
required signers, balance and fees, and it runs the validator. Thus, the
end-to-end test shows that the transactions obey these ledger rules.

**Public documentation.** The protocol specification, the technical
whitepaper, the implementation guide, the user guide, the API reference, and
this report. The documents use the ASD-STE100 writing rules, so readers with
English as a second language can read them.

**Collaboration and engagement.** The code, the plans and the milestone
evidence are public. The project has a Discord server for questions. One
developer did the work.

## Key learnings

1. **Simple designs ship.** The first design used zero-knowledge proofs and
   Hydra channels. Blind signatures gave privacy against the delegate, with
   less code.
2. **Blind signatures have sharp edges.** A small error in the hash to the
   group, or one key for all amounts, breaks the whole system. Review each
   step of the cryptography against a known design, such as Cashu.
3. **Test against the real rules.** The mock chain did not check witnesses or
   run scripts. Thus, the demonstrations passed, but the same transactions
   could not pass on preprod. A test chain must enforce the ledger rules.
4. **Custody is the main trade-off.** The users trust the delegate with their
   funds. The design must say this plainly and work to reduce it.
5. **Plan for time.** The project took longer than the nine months of the
   proposal. One developer carried all the work.

## Next steps for the product or service developed

1. Run a public node on preprod, and publish its address.
2. Get an external security audit of the node, the wallet and the validator.
3. Add keyset IDs, so a delegate can change keys and keep the notes valid.
4. Add a proof of reserves, so users can compare the notes with the vault.
5. Split the delegate key between many operators with a threshold scheme (for example, FROST).
6. Complete the settlement check for payments between nodes.
7. Improve the wallet: encrypted storage, QR codes, and a screen to split notes.
8. After these steps, prepare a release for the main network.

## Final thoughts and comments

µgraph shows that private, instant payments on Cardano need no new chain. A
vault with one small validator and a delegate with blind signatures are
sufficient. The security review in the last milestone made the system much
stronger. It also showed how easy it is to get this type of cryptography wrong.
We thank the Cardano community and Project Catalyst for the support.

## Links to other relevant project sources or documents

- Protocol specification: https://github.com/mugraph-payments/mugraph/blob/main/docs/spec/protocol.md
- Technical whitepaper: https://github.com/mugraph-payments/mugraph/blob/main/docs/whitepaper.md
- Implementation guide: https://github.com/mugraph-payments/mugraph/blob/main/docs/implementation.md
- User guide: https://github.com/mugraph-payments/mugraph/blob/main/docs/user-guide.md
- Presentation: https://github.com/mugraph-payments/mugraph/blob/main/docs/presentation/slides.md
- Presentation (PDF): https://github.com/mugraph-payments/mugraph/releases/download/v1.0.0/mugraph-1.0-slides.pdf
- Release 1.0.0: https://github.com/mugraph-payments/mugraph/releases/tag/v1.0.0

## Link to close-out video

(Add the YouTube link here after the upload.)
