# Close-out Video Script

This script is for the close-out video of Catalyst project 1200258. The video
must show four parts: the challenge and the approach, the progress, a
demonstration, and the next steps. The target length is 4 minutes.

## Before you record

1. Set the screen recorder to 1080p (or 720p).
2. Record the narration with a microphone. Reviewers asked for voice commentary.
3. Start the demonstration with `scripts/demo.sh up`.
4. Open wallet A and wallet B, and complete the setup in each wallet.
5. Send test funds to wallet A with `scripts/demo.sh faucet <address> 200000000`.
6. Open a terminal in the repository for the test run.

After you record, upload the video to YouTube as a public video. Then add
the link to the close-out report.

## Part 1: The challenge and the approach (0:00 to 0:50)

**Screen:** The README of the repository, then the proposal page on Catalyst.

**Narration:**

> Hello. I am Cainã Costa, and this is the close-out video for µgraph, project
> 1200258 in Fund 12, Cardano Use Cases: MVP. The project received 300,000 ADA.
>
> µgraph is a payment layer for Cardano. People do not pay with crypto for
> three reasons: it is slow, it shows all payments to the public, and it is
> hard to use. µgraph gives instant and private payments.
>
> A server, the delegate, holds ADA in a script address, the vault. In
> exchange, it gives users notes. Users pay each other with notes. The
> delegate signs notes with blind signatures, so it can not see who pays whom.

## Part 2: The progress (0:50 to 1:50)

**Screen:** The milestone page, then the list of documents in `docs/`.

**Narration:**

> The project had five milestones. Milestone 1 built the node. We replaced
> the zero-knowledge proofs of the proposal with blind signatures, with a
> change request. Milestone 2 added deposits and withdrawals on Cardano, with
> a validator in Aiken.
>
> Milestone 3 specified payments between nodes. The messages and the state
> machines exist, but the check of the settlement on the chain is not
> complete. Milestone 4 delivered the wallet for Android and iOS. QR codes and
> NFC are not in it yet.
>
> In this last milestone, a security review found serious problems. For
> example, the old hash to the group let anyone forge a note. And one key
> signed all amounts, so a deposit could give notes of any value. We fixed
> each problem with a test, and we released version 1.0 with a
> specification, a whitepaper and guides.

## Part 3: The demonstration (1:50 to 3:30)

**Screen:** The two wallets, the terminal, and the mock chain state.

**Actions and narration:**

1. In wallet A, open Settings, then Deposit. Deposit 100 ADA.

   > Wallet A deposits 100 ADA. The deposit is a normal Cardano transaction to
   > the vault.

2. Run `scripts/demo.sh mine 1`. Wait for the notes in wallet A.

   > After one confirmation, the node signs the notes. The wallet splits the
   > amount into notes of powers of two.

3. In wallet A, select two notes and send them. Copy the envelope.

   > Now wallet A pays wallet B. The payment is a text envelope. It does not
   > touch the chain.

4. In wallet B, import the envelope.

   > Wallet B checks the notes, and swaps them for new notes at the node. Now
   > only wallet B knows them.

5. In wallet C (or wallet B again), import the same envelope.

   > If someone tries to use the same notes again, the node refuses them.

6. In wallet A, withdraw 10 ADA to the funding address.

   > Finally, wallet A withdraws 10 ADA. The node checks that the notes pay
   > for the ADA that leaves the vault, and signs the transaction.

7. In the terminal, run the end-to-end test:
   `cargo test -p mugraph-wallet --test protocol_e2e`.

   > This test runs the same flow on a mock chain that checks signatures and
   > runs the validator. After each step, it checks that the notes equal the
   > ADA in the vault.

## Part 4: The next steps (3:30 to 4:10)

**Screen:** Section 9 of the whitepaper.

**Narration:**

> Version 1.0 is for the test networks. Before the main network, µgraph
> needs an external audit, key rotation, and a proof of reserves. It also
> needs a way to split the delegate key between many operators. The wallet
> needs encrypted storage and QR codes.
>
> (Say here if you plan to ask for more funds, and what the funds are for.
> Catalyst asks for this item.)
>
> All the code and the documents are open source, at
> github.com/mugraph-payments/mugraph. Thank you for your support.
