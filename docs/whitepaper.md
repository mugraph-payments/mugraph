# µgraph: Private, Instant Payments on Cardano

Technical whitepaper, version 1.0.

## Abstract

µgraph is a payment layer for Cardano. A server, the delegate, holds Cardano
funds in a script address, the vault. In exchange, it gives users bearer
tokens, the notes. Users pay each other with notes, and a payment settles in
one request to the delegate, not in a block. The delegate signs notes with
blind signatures, so it can not link the notes that it sees to their users.

This paper describes the problem, the design, and the reasons for each
design choice. It also describes the security and privacy properties, the
costs on the chain, and the limits of version 1.0. The protocol specification
(`docs/spec/protocol.md`) gives the exact rules.

## 1. The problem

Few people use cryptocurrencies to buy goods. We think that three problems
stop them:

1. **Speed.** A card payment confirms in about two seconds. A block on
   Cardano takes about 20 seconds, and a merchant must wait for more blocks to
   be sure.
2. **Privacy.** Each payment on a public chain shows the payer, the payee and
   the amount to all observers. Pseudonyms do not protect users from modern
   data analysis.
3. **Ease of use.** Users do not want to manage many keys, channels, or
   protocols to buy a coffee.

Price volatility is also a problem, but stablecoins on Cardano already
address it. µgraph works with any Cardano asset, stablecoins included.

## 2. Design goals

The design has these goals, in this order:

1. **No value from nothing.** No user can make notes that the vault does not
   pay for.
2. **Privacy for payers.** The delegate can not link a payment to the user
   that made it.
3. **Instant settlement.** A payment settles when the delegate replies.
4. **Simple wallets.** A wallet needs no channel state, no chain monitor, and
   no zero-knowledge prover.
5. **Standard Cardano.** The vault is a normal script address. Deposits and
   withdrawals are normal Cardano transactions.

The design accepts one important cost: the users trust the delegate with the
custody of their funds (section 7.4).

## 3. Background

### 3.1 Chaumian e-cash

David Chaum described e-cash in 1982. A bank signs a coin with a blind
signature. The bank can later verify its signature on the coin, but it can not
see which signature request made the coin. Thus, the bank knows that a coin is
valid but does not know who spent it.

### 3.2 Blind Diffie-Hellman key exchange

David Wagner described a blind signature from Diffie-Hellman in 1996, as an
alternative to Chaum's RSA method. The signer multiplies a point by its secret
key. The user adds and later removes a random point. Cashu, an e-cash system
for Bitcoin, uses this method. µgraph uses the same method, with the proofs
and the denomination keys that Cashu also uses.

### 3.3 Layer 2 systems

Payment channels (the Lightning Network) and Hydra heads move payments off the
chain. They give speed, but each channel or head needs locked funds, online
parties, and state that the parties must monitor. They do not hide the payer
from the other parties of the channel.

The original µgraph proposal planned to route payments through Hydra channels
and to prove transactions with zero-knowledge proofs. During the project, the
design removed the zero-knowledge component and used blind signatures, with a
change request to Project Catalyst. Blind signatures hide the payer from the
delegate, and the wallet needs no prover.

## 4. System overview

### 4.1 Parties

- **The wallet** keeps notes, makes blinded requests, and sends notes to other wallets.
- **The delegate** keeps a set of spent notes, signs new notes, and signs withdrawal transactions.
- **Cardano** keeps the funds in the vault.

### 4.2 Life of a payment

1. **Deposit.** Alice sends 100 ADA to the vault. The deposit datum names her
   blinded notes. The delegate checks the deposit on the chain and signs the
   notes.
2. **Payment.** Alice gives notes worth 30 ADA to Bob, for example as text in a
   message. Bob verifies the notes with the delegate's public keys.
3. **Refresh.** Bob sends the notes to the delegate and gets new notes. The
   delegate marks the old notes as spent. From now on, only Bob knows the new
   notes.
4. **Withdrawal.** Bob burns notes, and the delegate signs a Cardano
   transaction that pays Bob from the vault.

Steps 2 and 3 do not touch the chain. A refresh is one request to the
delegate, with no block to wait for. The payment is final when the reply
arrives.

## 5. Cryptography

### 5.1 Group and hashes

µgraph uses the Ristretto255 group and the BLAKE3 hash. Ristretto gives a group
of prime order with no cofactor problems, and fast, widely used code
(`curve25519-dalek`). BLAKE3 is fast and has an extendable output. Each use of
a hash has its own domain tag.

### 5.2 Hash to the group

A blind signature signs a point `Y` that comes from the message. No party may
know the discrete log of `Y`. If a party knows `y` with `Y = y · G`, it can
calculate the signature `k · Y = y · K` from the public key `K` alone.

Version 0 of µgraph made this error: it calculated `Y = H(m) · G`. Version 1.0
maps 64 bytes of BLAKE3 output to the group with the Ristretto
`from_uniform_bytes` map. Nobody knows the discrete log of the result.

### 5.3 Blind signatures

For a message `x` and a key `k`:

1. The wallet calculates `Y = hash_to_curve(x)`, selects a random `r`, and sends `B' = Y + r · G`.
2. The delegate returns `C' = k · B'`.
3. The wallet calculates `C = C' − r · K = k · Y`.

The delegate sees only `B'`. For each possible message, a value of `r` exists
that gives the same `B'`. Thus, `B'` gives no information about `x`.

### 5.4 Proofs of correct signing

The delegate also returns a DLEQ proof that it used the same `k` for `C'` as
for its public key `K`. The proof protects the wallet from a delegate that
signs with a different key for each user, to mark the users.

The proof also lets a holder verify a note. The note carries the blinding
factor `r` and the proof. A holder recalculates `B'` and `C'` from the note and
checks the proof. Only the delegate can verify `C = k · Y` directly, so this
is the only way for a user to check a note offline.

### 5.5 Denomination keys

A blinded point hides the amount of the note. Assume that one key signs all
notes. Then a user can blind a note for 1,000 ADA and ask for a signature in a
deposit of 1 ADA. The delegate does not see the difference.

Thus, the delegate uses a different key for each asset and each amount. The
amounts are powers of two, so each asset has 64 keys. The delegate derives
each key from its master key with a hash, so it stores only one secret. The
request shows the amount of each output. The key that the delegate uses fixes
that amount. A note for 1,000 ADA with a signature from the key for 1 ADA is
not valid.

Powers of two also make the anonymity sets large. Each note of 2^20 lovelace
of ADA looks the same as all other notes of 2^20 lovelace of ADA.

## 6. Operations

### 6.1 Refresh

A refresh spends notes and makes new notes of the same value. It is the one
operation for splits, merges, and payments. The delegate checks the value of
each asset, verifies each input, and signs each blinded output with the key
for its amount. It does all these steps in one database transaction, so a
failed refresh changes nothing.

The wallet sends each output with a nonce of zero. The real nonce stays in the
wallet. Version 0 sent the real nonce. Thus, a version 0 delegate saw the full
note when it signed it, and it could link the note when a user spent it. The
blinding then gave no privacy.

### 6.2 Deposit

A deposit is a Cardano transaction that pays the vault. Its inline datum has
three hashes: the depositor's key, the delegate's key, and the intent. The
intent lists the blinded outputs with their assets and amounts. The depositor
also signs the intent with CIP-8.

The delegate checks the transaction on the chain and waits for enough blocks.
Then it checks that the outputs add up to the exact value of the UTxO, for
each asset. After that, it signs the outputs.

### 6.3 Withdrawal

A withdrawal is a Cardano transaction that the wallet builds and the delegate
signs. It spends vault UTxOs and pays the user. The rest goes back to the vault
with a datum that names the delegate.

The delegate verifies the notes that the withdrawal burns. Then it checks the
main rule of the withdrawal: the notes minus the change notes must equal the
value that leaves the vault. The value that leaves the vault includes the
fee, so the notes pay the fee.

The vault validator lets only the delegate spend vault UTxOs. The wallet adds
the validator, the redeemers, and collateral from its own address. The
delegate adds its witness and submits the transaction.

### 6.4 Payments between users

A payment between users does not need the delegate at first. The payer gives
the notes to the payee. The payee verifies them offline with the DLEQ proofs.
Then the payee refreshes them as soon as it can. Before that refresh, the payer
still knows the notes and can spend them first. Thus, a payee that does not
trust the payer must refresh before it gives goods.

## 7. Security and privacy

### 7.1 Threat model

We consider three types of attackers:

- A user who wants to make or spend value that is not theirs
- A delegate who wants to link payments to users
- An outside observer of the chain and of the network

### 7.2 Value

A user can not make a note without the key for its asset and amount, because
the discrete log of each signed point is unknown. A user can not spend a note
two times, because the delegate keeps the spent signatures. A deposit gives
the same value as it locks, and a withdrawal pays the same value as it burns.
The delegate checks the value of each asset in both. A user can not take funds
from the vault on the chain, because the validator needs the delegate's
signature.

The end-to-end test runs a deposit, a transfer, a second use of the same
notes, and a withdrawal. It uses a mock chain that checks witnesses and runs
the validator. After each step, the test checks that the unspent notes equal
the value in the vault.

### 7.3 Privacy

The delegate sees the asset and amount of each note that it signs, and each
note that it receives. It does not see the link between them. An attacker that
controls the delegate needs other data to link a note to a user. Examples are
the time of the requests and the network address of the user.
Denominations of powers of two make many notes look the same.

Deposits and withdrawals are public Cardano transactions. An observer can link
a deposit and a withdrawal to their Cardano addresses. A user who wants
privacy should not deposit and withdraw the same amounts at about the same
time.

### 7.4 Custody

The delegate holds the funds. It can refuse to serve a user, and it can make
notes that no deposit pays for, because it has the keys. Nobody outside the
delegate can count the notes, because they are blind. A leak of the master key
lets an attacker make notes and empty the vault.

These are the costs of the design. Section 9 describes ways to reduce them.

## 8. Costs

### 8.1 On the chain

We measured these values with the preprod protocol parameters:

| Item                          | Value                                               |
| ----------------------------- | --------------------------------------------------- |
| Validator size                | 270 bytes                                           |
| Validator cost for each spend | about 9.2 million CPU steps and 28,000 memory units |
| Declared units for each spend | 40 million CPU steps and 100,000 memory units       |
| Withdrawal with 1 vault input | 688 bytes, 0.204 ADA fee                            |
| Each extra vault input        | 54 bytes, 0.011 ADA fee                             |
| Deposit fee (wallet default)  | 0.2 ADA                                             |

The transaction carries the validator in its witness set, so it does not need
a reference script.

### 8.2 Off the chain

A refresh costs the delegate one scalar multiplication and one proof for each
output, and one key derivation and one multiplication for each input. The
spent set holds 32 bytes for each spent note. Payments between users need no
work from the delegate until the refresh.

## 9. Limits and future work

Version 1.0 runs on the Cardano testnets. These items stop it from use on the
main network:

- **Key rotation.** The protocol has no keyset IDs. A new master key makes all
  notes invalid. Keyset IDs with an expiry time would let a delegate rotate
  keys.
- **Proof of reserves.** The users can not check that the notes and the vault
  agree. A delegate could publish the total of each denomination that it
  signed and spent, and let users compare it with the vault.
- **Shared custody.** A threshold signature scheme, for example FROST, could
  split the master key between many operators. Then no single operator could
  make notes or spend the vault.
- **Cross-node payments.** The messages and state machines for payments
  between delegates exist. The check of the settlement transaction on the
  chain does not.
- **Wallet safety.** The wallet keeps its keys without encryption and has no
  QR codes. A mobile release needs encrypted storage and QR transfers.
- **Audit.** The code and the validator need an external security audit.

## 10. Conclusion

µgraph shows that Cardano can support private, instant payments with a small
and simple design. Blind signatures move the payments off the chain and hide
the payer from the delegate. A vault with one validator keeps the funds on the
chain, in a normal script address. The main open problem is custody: the users
trust one delegate. The work in section 9 reduces that trust.

## References

1. D. Chaum. "Blind Signatures for Untraceable Payments." Advances in Cryptology, 1982.
2. D. Wagner. "Chaum's Blinding Algorithm with Diffie-Hellman." Cypherpunks mailing list, 1996.
3. Cashu. "NUT-00: Notation and Models", "NUT-02: Keysets", "NUT-12: DLEQ proofs." https://github.com/cashubtc/nuts
4. H. de Valence et al. "The Ristretto Group." https://ristretto.group
5. D. Chaum and T. Pedersen. "Wallet Databases with Observers." CRYPTO 1992.
6. CIP-8: Message Signing. https://cips.cardano.org/cip/CIP-0008
7. The µgraph protocol specification. `docs/spec/protocol.md`
