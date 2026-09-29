---
marp: true
theme: default
paginate: true
title: µgraph 1.0
---

# µgraph 1.0

Private, instant payments on Cardano

Project Catalyst, Fund 12, project 1200258

---

## The problem

Few people pay for goods with crypto. Three problems stop them:

- **Speed.** A card payment takes about 2 seconds. A Cardano block takes about 20.
- **Privacy.** A payment on a public chain shows the payer, the payee and the amount.
- **Ease of use.** Users do not want to manage keys, channels and protocols.

---

## The idea: e-cash on Cardano

- A server, the **delegate**, holds ADA in a script address, the **vault**.
- In exchange, it gives users **notes**: bearer tokens with a value.
- Users pay each other with notes, off the chain.
- The delegate signs notes with **blind signatures**. It can not see who pays whom.

---

## A payment

1. **Deposit.** Alice sends 100 ADA to the vault. The delegate signs her notes.
2. **Pay.** Alice gives notes to Bob, as a text message.
3. **Refresh.** Bob swaps the notes for new notes at the delegate.
4. **Withdraw.** Bob burns notes. The delegate signs a transaction that pays him.

Steps 2 and 3 do not touch the chain.

---

## Blind signatures

Alice wants a signature on `x` from the delegate (secret key `k`, public key `K`):

- Alice sends `B' = hash_to_curve(x) + r·G`, with a random `r`.
- The delegate sends back `C' = k·B'`, and a proof that it used `k` (DLEQ).
- Alice calculates `C = C' − r·K = k·hash_to_curve(x)`.

The delegate sees `B'` only. `B'` gives no information about `x`.

Group: Ristretto255. Hash: BLAKE3.

---

## One key for each amount

- A blinded note hides its amount.
- With one key for all notes, a user can ask for a note of 1,000 ADA in a deposit of 1 ADA.
- Thus, the delegate uses **one key for each asset and each power of two**.
- The key fixes the amount. A note of 1,000 with the key for 1 is not valid.
- All notes of one asset and amount look the same: a large anonymity set.

---

## The vault on Cardano

- A Plutus V3 validator in Aiken, 270 bytes.
- **Rule:** only the delegate can spend a vault UTxO.
- **Deposit:** the notes must equal the value of the deposit UTxO, for each asset.
- **Withdrawal:** notes minus change must equal the value that leaves the vault, fee included.

---

## Privacy and trust

The delegate sees:

- the asset and the amount of each note
- the deposits and the withdrawals (they are on the chain).

The delegate does not see which spent note comes from which request.

The users trust the delegate with custody. The delegate could sign notes that no deposit pays for.

---

## What we built

| Part      | Content                                            |
| --------- | -------------------------------------------------- |
| Node      | Rust, HTTP and JSON, redb, Blockfrost and Maestro  |
| Validator | Aiken, Plutus V3                                   |
| Wallet    | Tauri and React, Android and iOS builds            |
| Tests     | 485 Rust tests, 14 Aiken tests, an end-to-end test |
| Tools     | Docker image, mock chain, simulator                |
| Documents | Specification, whitepaper, guides, API reference   |

---

## The security review of 1.0

We found and fixed, each with a test first:

- A hash to the group with a public discrete log: anyone could forge notes.
- One key for all amounts: a deposit could give notes of any value.
- Withdrawals that did not verify the notes that they burned.
- A validator that let a depositor take the deposit back alone.
- A refresh that showed the nonces of new notes to the delegate.
- A vault address of the wrong type, and withdrawals with no Plutus witness.

---

## Costs on the chain

| Item                          | Value                                      |
| ----------------------------- | ------------------------------------------ |
| Validator cost for each spend | 9.2 million CPU steps, 28,000 memory units |
| Withdrawal with 1 vault input | 688 bytes, 0.204 ADA fee                   |
| Each extra vault input        | 0.011 ADA                                  |
| Payment between users         | No transaction                             |

---

## Limits and next steps

- Version 1.0 is for the test networks.
- An external security audit.
- Keyset IDs, so a delegate can change keys.
- A proof of reserves.
- A delegate key that many operators share (for example, FROST).
- The settlement check for payments between nodes.
- A better wallet: encrypted storage, QR codes, a screen to split notes.

---

# Thank you

- Code: github.com/mugraph-payments/mugraph
- Documents: mugraph.dev
- Specification: `docs/spec/protocol.md`
- Whitepaper: `docs/whitepaper.md`
