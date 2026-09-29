# µgraph Protocol Specification

Version 1.0. This document specifies the µgraph protocol as the code in this
repository implements it. When this document and the code do not agree, the
code is the reference, and the difference is an error in this document.

The key words MUST, MUST NOT, SHOULD and MAY have the meanings in RFC 2119.

## 1. Scope

µgraph is a payment layer for Cardano. A server, the **delegate**, holds
Cardano funds in a script address, the **vault**. In exchange, the delegate
gives **notes** to users. A note is a bearer token: it has a value in one
Cardano asset, and the holder of the note can spend it.

The delegate signs notes with blind signatures. Thus, the delegate can check
that a note is valid. But it can not link a note that it receives to the note
that it signed. The delegate sees the asset and the amount of each note. It
does not see who holds the note.

This document specifies these parts of the protocol:

- The cryptography: hashes, keys, blind signatures and proofs (sections 4 to 6)
- The notes and their wire format (sections 7 and 8)
- The operations: keys, refresh, deposit, withdrawal and transfer (section 9)
- The vault validator on Cardano (section 10)
- The invariants and the security properties (sections 11 and 12)

Section 13 describes the cross-node extension. Section 14 describes the
changes from version 0.

## 2. Terms

| Term             | Meaning                                                                                             |
| ---------------- | --------------------------------------------------------------------------------------------------- |
| Delegate         | The server that holds the vault funds and signs notes. The code calls it the node.                  |
| Vault            | The Cardano script address that holds the funds for the notes of one delegate.                      |
| Note             | A bearer token for an amount of one asset, signed by one delegate.                                  |
| Asset            | A Cardano asset: a policy ID and an asset name. ADA has the zero policy ID and an empty name.       |
| Denomination     | A note amount. Each amount is a power of two, from 2^0 to 2^63.                                     |
| Master key       | The delegate's long-term secret key. All other delegate keys come from it.                          |
| Denomination key | The key that signs notes of one asset and one denomination.                                         |
| Keyset           | The 64 denomination public keys of one asset.                                                       |
| Refresh          | An operation that spends notes and makes new notes of the same total value.                         |
| Atom             | One input note or one output note in a refresh.                                                     |
| Wallet           | The client software of a user.                                                                      |
| Unit             | The name of an asset in the Cardano provider API: "lovelace", or the policy ID and the name in hex. |

## 3. System model

### 3.1 Parties

The protocol has three parties:

- **Wallets** keep notes, make blinded outputs, and send notes to other wallets.
- **The delegate** keeps the vault and a set of spent notes. It signs outputs, and it signs withdrawal transactions.
- **Cardano** keeps the vault funds. The vault validator lets only the delegate spend them.

### 3.2 Trust

The protocol is custodial. The users trust the delegate for these items:

- The delegate holds the vault funds and does not spend them outside a withdrawal.
- The delegate does not sign notes that no deposit pays for.
- The delegate processes refresh and withdrawal requests.

The users do not trust the delegate with their privacy. The blind signatures
make sure that the delegate can not link a spent note to the refresh or deposit
that made it.

The delegate does not trust the users. Each rule in this document that starts
with "the delegate MUST" protects the vault from users that do not obey the
protocol.

## 4. Notation and primitives

### 4.1 Group

The protocol uses the Ristretto255 group over Curve25519.

- `G` is the Ristretto base point.
- `ℓ` is the order of the group.
- Scalars are integers modulo `ℓ`. Points are elements of the group.
- A point encodes to 32 bytes (compressed Ristretto). A scalar encodes to 32 bytes, little-endian.
- A 32-byte string becomes a scalar through a reduction modulo `ℓ`.

### 4.2 Hash functions

- `BLAKE3(x)` is BLAKE3 with a 32-byte output.
- `BLAKE3_XOF(x, n)` is the first `n` bytes of the BLAKE3 extendable output.
- `len64(x)` is the length of `x` in bytes, as an 8-byte little-endian integer.
- `‖` is the concatenation of byte strings.

The protocol uses these domain tags:

| Name         | Value               | Use               |
| ------------ | ------------------- | ----------------- |
| `HTC_SEP`    | `mugraph_v1_htc`    | Hash to the group |
| `DLEQ_SEP`   | `mugraph_v0_dleq`   | DLEQ challenge    |
| `KEYSET_SEP` | `mugraph_v1_keyset` | Denomination keys |

### 4.3 Hash to a scalar

For a domain tag `D` and byte strings `x_1 … x_n`:

```
hash_to_scalar(D, [x_1 … x_n]) =
    BLAKE3(D ‖ len64(x_1) ‖ x_1 ‖ … ‖ len64(x_n) ‖ x_n) mod ℓ
```

The domain tag has no length prefix.

### 4.4 Hash to the group

```
hash_to_curve(m) = Ristretto.from_uniform_bytes(
    BLAKE3_XOF(HTC_SEP ‖ len64(m) ‖ m, 64))
```

`from_uniform_bytes` is the Ristretto map from 64 uniform bytes to a point. No
party knows the discrete log of the result relative to `G`. The security of
the blind signatures depends on this property (see section 12.1).

## 5. Keys

### 5.1 Master key

The delegate has one master secret key `k_M` (a scalar) and its public key
`K_M = k_M · G`. The delegate publishes `K_M`. A note names its delegate by
`K_M`.

### 5.2 Denomination keys

For an asset `A` and an amount `2^d`, the denomination secret key is:

```
k(A, d) = hash_to_scalar(KEYSET_SEP, [ enc(k_M), asset_bytes(A), [d] ])
K(A, d) = k(A, d) · G
```

- `enc(k_M)` is the 32-byte canonical encoding of the master scalar.
- `asset_bytes(A)` is the 64-byte asset encoding (section 7.2).
- `[d]` is one byte with the value `d`, from 0 to 63.

An amount that is not a power of two has no key. The delegate MUST refuse
such an amount in each operation.

### 5.3 Keysets

The keyset of an asset `A` is the list `[K(A, 0), …, K(A, 63)]`. The key at
index `d` signs notes of amount `2^d`. A wallet gets a keyset with the `keys`
request (section 9.1).

Because the key depends on the asset and the amount, a signature is valid only
for one asset and one amount. The delegate can not see the amount in a blinded
note, but the key that it uses fixes the amount.

## 6. Blind signatures

The protocol uses Blind Diffie-Hellman Key Exchange (BDHKE) with a
Chaum-Pedersen proof of discrete log equality (DLEQ). In this section, `k` is
a denomination secret key and `K = k · G`.

### 6.1 Blind

The wallet has a message `x` (the note commitment, section 7.3). The wallet:

1. Calculates `Y = hash_to_curve(x)`.
2. Selects a random nonzero scalar `r`, the blinding factor.
3. Calculates `B' = Y + r · G`.
4. Sends `B'` to the delegate.

### 6.2 Sign

The delegate calculates `C' = k · B'`. Then it makes a DLEQ proof that
`log_G(K) = log_B'(C')`:

1. Select a random scalar `n`.
2. Calculate `R_1 = n · G` and `R_2 = n · B'`.
3. Calculate `e = hash_to_scalar(DLEQ_SEP, [G, B', K, C', R_1, R_2])`, with each point encoded in 32 bytes.
4. Calculate `z = n + e · k`.

The blind signature is the pair `(C', (e, z))`.

### 6.3 Check the proof

The wallet checks the proof before it unblinds:

1. Calculate `R_1 = z · G − e · K` and `R_2 = z · B' − e · C'`.
2. Calculate `e' = hash_to_scalar(DLEQ_SEP, [G, B', K, C', R_1, R_2])`.
3. Accept the proof only if `e' = e`.

### 6.4 Unblind

The wallet calculates `C = C' − r · K`. The note signature is `C`. Because
`C' = k · (Y + r · G)`, the result is `C = k · Y`. The wallet MUST refuse a
blinding factor of zero.

### 6.5 Verify as the delegate

The delegate verifies a note signature with its secret key: the signature is
valid if `C = k · hash_to_curve(x)`. Only the delegate can do this check.

### 6.6 Verify as a holder

A holder can not calculate `k · Y`. Thus, a note carries its DLEQ proof and
its blinding factor `r`. A holder verifies the note with the public key `K`:

1. Calculate `Y = hash_to_curve(x)`.
2. Calculate `B' = Y + r · G` and `C' = C + r · K`.
3. Check the DLEQ proof `(e, z)` for `(K, B', C')` (section 6.3).

The proof shows that the key holder of `K` signed `B'`. No party except the
wallet that made `B'` knows a discrete log of `B' − r · G` relative to `G`.
Thus, a forger can not make a valid `(C, r, e, z)` for a message that the
delegate did not sign.

## 7. Notes

### 7.1 Fields

| Field        | Type          | Meaning                                                            |
| ------------ | ------------- | ------------------------------------------------------------------ |
| `amount`     | u64           | The value, in the smallest unit of the asset. A power of two.      |
| `delegate`   | 32 bytes      | The master public key `K_M` of the delegate.                       |
| `policy_id`  | 28 bytes      | The Cardano policy ID of the asset.                                |
| `asset_name` | 0 to 32 bytes | The Cardano asset name.                                            |
| `nonce`      | 32 bytes      | A random value that the wallet selects. It makes each note unique. |
| `signature`  | 32 bytes      | The unblinded signature `C`, as a point.                           |
| `dleq`       | optional      | The proof `e`, `z` and the blinding factor `r`, for holder checks. |

### 7.2 Asset encoding

`asset_bytes(A)` is 64 bytes:

| Bytes    | Content                                                   |
| -------- | --------------------------------------------------------- |
| 0 to 27  | The policy ID                                             |
| 28 to 31 | The length of the asset name, as a u32, little-endian     |
| 32 to 63 | The asset name, with zero bytes after it to fill 32 bytes |

### 7.3 Commitment

The commitment of a note is the message that the delegate signs:

```
x = BLAKE3( delegate ‖ asset_bytes(A) ‖ u64_le(amount) ‖ nonce )
```

The input is 136 bytes. The commitment has no domain tag.

### 7.4 Validity

A note is valid if all these conditions are true:

- The amount is a power of two.
- The signature is valid for the commitment with `k(A, d)`, where `2^d` is the amount.
- The signature is not in the delegate's spent set.

## 8. Wire format

### 8.1 Transport

A delegate serves HTTP. `GET /health` returns the text `OK`. `POST /rpc`
takes one request as JSON and returns one response as JSON, with the HTTP
status 200.

A request is `{"m": <method>, "p": <payload>}`. A response is
`{"m": <method>, "r": <result>}`. An error response is
`{"m": "error", "r": {"reason": <text>}}`.

### 8.2 Encodings

- Byte strings (keys, points, hashes, policy IDs) are lowercase hex strings.
- An asset name is a JSON string with the UTF-8 text of the name.
- Integers are JSON numbers.

### 8.3 Types

**Note.** An object with the keys `amount`, `delegate`, `policy_id`,
`asset_name`, `nonce`, `signature` and `dleq`. The value of `dleq` is null, or
`{"e": <hex>, "z": <hex>, "r": <hex>}`.

**BlindSignature.** `{"c": <point C'>, "p": {"e": <hex>, "z": <hex>}}`.

**BlindedOutput.** `{"policy_id", "asset_name", "amount", "point"}`. The
`point` is `B'`.

**Atom.** `{"delegate", "asset_id", "amount", "nonce", "signature"}`.
`asset_id` is an index in the refresh asset list. `signature` is an index in
the refresh signature list for an input, and null for an output.

**Refresh.** An object with these keys:

| Key  | Type           | Meaning                                                |
| ---- | -------------- | ------------------------------------------------------ |
| `m`  | u32            | A bit mask. Bit `i` is 1 if atom `i` is an input.      |
| `a`  | list of Atom   | The atoms.                                             |
| `a_` | list of Asset  | The assets that the atoms refer to.                    |
| `s`  | list of points | The signatures of the input notes.                     |
| `b`  | list of points | The blinded points `B'` of the outputs, in atom order. |

### 8.4 Methods

| Method       | Payload                             | Result                                                                               |
| ------------ | ----------------------------------- | ------------------------------------------------------------------------------------ |
| `public_key` | none                                | `delegate_pk`, `cardano_script_address`, `cardano_payment_vk`, `cardano_script_cbor` |
| `keys`       | `policy_id`, `asset_name`           | `keys`: the keyset (64 points)                                                       |
| `refresh`    | Refresh                             | `s`: a list of BlindSignature, one for each output                                   |
| `deposit`    | DepositRequest (section 9.3)        | `s`: a list of BlindSignature, and `deposit_ref`                                     |
| `withdraw`   | WithdrawRequest (section 9.4)       | `signed_tx_cbor`, `tx_hash`, and `s`: the change signatures                          |
| `emit`       | `policy_id`, `asset_name`, `amount` | A Note. Only in dev mode (section 9.6).                                              |

Section 13 lists the cross-node methods.

## 9. Operations

### 9.1 Keys and delegate information

A wallet gets the delegate information with `public_key`. The result has the
master public key, the vault address, the delegate's Cardano payment key and
the vault validator. A wallet gets a keyset with `keys`.

The keyset of an asset does not change while the master key does not change.
A wallet MAY keep keysets.

### 9.2 Refresh

A refresh spends input notes and makes output notes of the same value. A
wallet uses a refresh to split notes, to merge notes, and to take sole control
of notes that it received.

#### 9.2.1 Wallet procedure

1. Select the input notes.
2. Split each output amount into denominations. Each denomination becomes one output atom.
3. Select a new random nonce for each output note.
4. Blind the commitment of each output (section 6.1).
5. Save the nonces and the blinding factors.
6. Set the nonce of each output atom to 32 zero bytes.
7. Send the refresh.
8. For each output, check the proof and unblind the signature (sections 6.3 and 6.4).
9. Keep the new notes, and mark the input notes as spent.

Step 5 lets the wallet recover the output notes if the response does not
arrive. Step 6 keeps the nonces secret (see section 12.2).

#### 9.2.2 Delegate rules

The delegate MUST refuse a refresh unless all these conditions are true:

1. The refresh has 32 atoms or less.
2. Each atom amount is a power of two.
3. Each `asset_id` is an index in `a_`.
4. Each input atom has a signature index in `s`.
5. For each asset, the sum of the input amounts equals the sum of the output amounts.
6. The list `b` has one point for each output atom.
7. Each atom names this delegate.
8. Each output atom has a nonce of 32 zero bytes.
9. Each input signature is not zero, and is not in the spent set.
10. Each input signature is valid for its commitment with `k(A, d)` (section 6.5).

If the refresh is valid, the delegate MUST do these steps in one database
transaction:

- Add each input signature to the spent set.
- Sign each blinded point `b_j` with `k(A, d)` of output atom `j`.

If one input fails, the delegate MUST NOT add any input to the spent set.

### 9.3 Deposit

A deposit locks funds in the vault and gives notes of the same value.

#### 9.3.1 Vault datum

Each vault UTxO has an inline datum. The datum is a Plutus constructor 0 with
three byte strings:

| Field              | Size     | Content                                             |
| ------------------ | -------- | --------------------------------------------------- |
| `user_pubkey_hash` | 28 bytes | `BLAKE2b-224` of the depositor's Ed25519 public key |
| `node_pubkey_hash` | 28 bytes | `BLAKE2b-224` of the delegate's Cardano payment key |
| `intent_hash`      | 32 bytes | `BLAKE2b-256` of the intent payload (section 9.3.2) |

#### 9.3.2 Intent payload

The intent payload is the JSON text of this object, with no spaces and with
the keys in this order:

```json
{
  "outputs": [{ "policy_id": "<hex>", "asset_name": "<hex>", "amount": 1024, "point": "<hex>" }],
  "delegate_pk": "<hex>",
  "script_address": "<bech32>",
  "nonce": 1727600000,
  "network": "preprod"
}
```

The payload binds the deposit to its blinded outputs and their amounts. It
does not include the UTxO reference, because the datum must contain the
intent hash before the transaction exists.

#### 9.3.3 Wallet procedure

1. Split the deposit amount into denominations.
2. Make a note, a nonce and a blinded point for each denomination.
3. Save the nonces and the blinding factors.
4. Make the intent payload.
5. Sign the payload as a CIP-8 COSE_Sign1 message with the wallet's Ed25519 key.
6. Make a Cardano transaction that pays the amount to the vault with the datum.
7. Sign the transaction with the key of the funding address.
8. Submit the transaction.
9. Send the deposit request.
10. Check the proofs and unblind the signatures.

The deposit request has these fields:

| Field       | Content                                                             |
| ----------- | ------------------------------------------------------------------- |
| `utxo`      | `{"tx_hash", "index"}` of the vault UTxO                            |
| `outputs`   | The list of BlindedOutput                                           |
| `message`   | JSON text with the key `user_pubkey`: the Ed25519 public key in hex |
| `signature` | The COSE_Sign1 bytes                                                |
| `nonce`     | The payload nonce                                                   |
| `network`   | The Cardano network name                                            |

#### 9.3.4 Delegate rules

The delegate MUST refuse a deposit unless all these conditions are true:

1. The COSE_Sign1 signature uses EdDSA, and it is valid for `user_pubkey`.
2. The signed payload is equal to the intent payload that the delegate makes from the request.
3. The UTxO is at the vault address.
4. The UTxO has at least the configured number of confirmations.
5. The datum `user_pubkey_hash` is the hash of `user_pubkey`.
6. The datum `node_pubkey_hash` is the hash of the delegate's payment key.
7. The datum `intent_hash` is the hash of the intent payload.
8. There is at least one output, and each output amount is a power of two.
9. For each unit, the sum of the output amounts equals the quantity in the UTxO.
10. The UTxO holds at least the minimum deposit in lovelace.
11. The delegate did not accept a deposit for this UTxO before.

If the deposit is valid, the delegate signs each output with `k(A, d)` of the
output, and records the UTxO reference. The result `deposit_ref` is
`<tx_hash>:<index>`.

### 9.4 Withdrawal

A withdrawal burns notes and pays their value from the vault to a Cardano
address.

#### 9.4.1 Transaction

The wallet makes a Cardano transaction with these parts:

- The inputs are vault UTxOs only.
- One output pays the withdrawal amount to the destination address.
- An optional output returns the rest to the vault. It MUST have a vault datum that names the delegate.
- `required_signers` includes the delegate's payment key hash.
- The witness set has the vault validator and one spend redeemer for each input. The redeemer data is the Plutus unit value (constructor 0 with no fields).
- The transaction has collateral from a key address of the wallet, a collateral return and a total collateral.
- The body has the script data hash of the redeemers and the PlutusV3 cost model.

The notes pay for the payout and for the transaction fee. The wallet signs for
the collateral. The delegate adds its own witness.

The wallet MUST send each note without its DLEQ proof. The proof has the
blinding factor `r`. With `r`, the delegate can calculate `B'` and link the
note to the request that made it (see section 12.2).

#### 9.4.2 Request

| Field            | Content                                                    |
| ---------------- | ---------------------------------------------------------- |
| `notes`          | The notes to burn, as Note objects with `dleq` set to null |
| `change_outputs` | A list of BlindedOutput for the change                     |
| `tx_cbor`        | The transaction, in hex                                    |
| `tx_hash`        | The BLAKE2b-256 hash of the transaction body, in hex       |

#### 9.4.3 Delegate rules

The delegate MUST refuse a withdrawal unless all these conditions are true:

1. The delegate did not complete a withdrawal with this hash before.
2. The transaction is not larger than the size limit, and the fee is not more than the fee limit.
3. The hash of the transaction body is `tx_hash`.
4. There is at least one note, and no note occurs two times.
5. Each note names this delegate, and its signature is valid with `k(A, d)` (section 6.5).
6. Each change output amount is a power of two.
7. Each input is a vault UTxO with a vault datum that names this delegate. No input occurs two times.
8. Each vkey and bootstrap witness in the transaction is valid.
9. `required_signers` includes the delegate's payment key hash.
10. For each unit, the inputs equal the outputs plus the fee.
11. Each output is on the delegate's network.
12. Each output to the vault has a vault datum that names this delegate.
13. For each unit, the notes minus the change outputs equal the vault inputs minus the vault outputs.

Rule 13 is the value rule of the withdrawal. The fee leaves the vault, so the
notes pay for it.

If the withdrawal is valid, the delegate does these steps:

1. Add its vkey witness to the transaction.
2. Sign each change output with `k(A, d)` of the output.
3. In one database transaction, add each note signature to the spent set, and record the withdrawal as pending.
4. Submit the transaction.
5. Record the withdrawal as completed, or as failed if the submission fails.

The delegate does not remove notes from the spent set after a failed
submission, because the transaction can still reach the chain. A wallet can
send the same request again after a failure. The delegate then submits the
transaction again and does not burn the notes a second time.

### 9.5 Transfer between users

A user pays another user with notes:

1. The sender gives the notes to the receiver, for example as JSON text.
2. The receiver gets the keyset of each asset.
3. The receiver verifies each note (section 6.6).
4. The receiver refreshes each note into a new note of the same amount.

The transfer does not need the delegate until step 4. Before step 4, the sender
still knows the notes and can spend them. The first refresh or withdrawal that
the delegate accepts spends the notes. A receiver SHOULD do step 4 at once.

Sometimes the sender has no notes that add up to the payment amount. Then the
sender first makes notes of the correct amounts with a refresh.

### 9.6 Dev mode

A delegate in dev mode accepts `emit`, which makes a note without a deposit.
Dev mode is only for tests and demonstrations. A delegate on a public network
MUST NOT use dev mode.

## 10. Vault validator

The vault validator is an Aiken validator for Plutus V3. It has no parameters.
The blueprint `validator/plutus.json` has its compiled code and its hash.

The validator accepts a spend if all these conditions are true:

1. The UTxO has a datum.
2. `user_pubkey_hash` and `node_pubkey_hash` are 28 bytes each.
3. `node_pubkey_hash` is in the `extra_signatories` of the transaction.

The redeemer MUST be constructor 0 with no fields. The validator does not use
the redeemer for other checks. It does not check the outputs. The delegate
checks the outputs (section 9.4.3) before it signs.

The vault address is a Shelley enterprise address with the validator hash as
its payment credential (address type 7).

## 11. Invariants

The protocol keeps these invariants:

1. **Reserve.** For each asset, the unspent notes equal the value in the vault. Deposits that the delegate did not accept are not part of this value.
2. **No new value.** A refresh keeps the value of each asset. A deposit makes notes of the same value as the funds that it locks. A withdrawal pays out the same value as the notes that it burns.
3. **One spend.** The delegate accepts each note signature one time only.
4. **Custody.** Only a transaction that the delegate signed can spend a vault UTxO.

Invariant 1 follows from invariants 2 to 4. The end-to-end test in
`wallet/src-tauri/tests/protocol_e2e.rs` checks invariant 1 after a deposit, a
transfer, a second import of the same notes, and a withdrawal.

## 12. Security

### 12.1 Unforgeability

A valid note signature needs `k(A, d)`. The discrete log of
`hash_to_curve(x)` is unknown, so no party can calculate `k(A, d) · Y` from
public data. A user who does not have a note can not make one.

### 12.2 Unlinkability

The delegate sees these items:

- At a refresh: the input notes (with their nonces), and the asset and amount of each output
- At a deposit: the funding transaction, and the asset and amount of each output
- At a withdrawal: the notes, and the destination address

The delegate does not see the nonce or the commitment of an output when it
signs it. The blinding factor `r` is random, so `B'` has no relation to the
commitment that the delegate sees later. Thus, the delegate can not link a
spent note to the operation that made it.

A note carries its blinding factor `r` for holder checks. With `r`, the
delegate can calculate `B'` and link the note to the request that made it.
Thus, a wallet sends a note to the delegate without its proof, and the
delegate never sees `r`.

The anonymity set of a note is the set of notes of the same asset and
denomination. Deposits and withdrawals are on the chain, so an observer can
link them to Cardano addresses.

### 12.3 Double spend

The spent set stops a second spend at the delegate. A transfer between users
(section 9.5) does not protect the receiver until the receiver refreshes the
notes.

### 12.4 Custody risk

The delegate can sign notes that no deposit pays for, because it has the
keys. Nobody outside the delegate can count the unspent notes, because the
notes are blind. Thus, the users trust the delegate for the reserve
invariant. A leak of the master key lets an attacker make notes and empty the
vault.

### 12.5 Known limitations

- The protocol has no key rotation and no keyset IDs. A new master key makes all notes invalid.
- The `refresh` method has no authentication and no rate limit.
- The delegate makes no proof of reserves.
- The wallet keeps its keys and notes without encryption.
- The wallet does not show or read QR codes. Users move notes as JSON text.
- The cross-node settlement check (section 13) is not complete.

## 13. Cross-node extension

The cross-node extension moves value between two delegates. The Milestone 3
specifications in `docs/specs/` describe it:

- `milestone-3-cross-node-payments.md`: the model, the settlement and the state machines
- `milestone-3-inter-node-protocol-messages.md`: the messages and their authentication
- `milestone-3-peer-registry-format.md`: the file of trusted peers

The RPC methods are `cross_node_transfer_create`, `cross_node_transfer_notify`,
`cross_node_transfer_status` and `cross_node_transfer_ack`. Each message has
an Ed25519 signature from a key in the peer registry.

The node has the message types, the authentication, the idempotency rules and
the state machines. It does not yet check the settlement transaction on the
chain (metadata label 673). It does not yet send retries to peers. Thus, a
node MUST NOT credit a cross-node transfer on a public network with version
1.0.

## 14. Changes from version 0

Version 1.0 is not compatible with version 0. Notes from version 0 are not
valid in version 1.0.

| Area                | Version 0                              | Version 1.0                                              |
| ------------------- | -------------------------------------- | -------------------------------------------------------- |
| Hash to the group   | `G · H(m)`, with a public discrete log | Ristretto map of a 64-byte BLAKE3 output                 |
| Signing keys        | One key for all notes                  | One key for each asset and denomination                  |
| Note amounts        | Any amount                             | Powers of two                                            |
| Holder verification | With the public key only               | With the DLEQ proof and the blinding factor              |
| Refresh outputs     | Nonce in clear, blinding optional      | Nonce hidden, blinding required                          |
| Deposit outputs     | Only the blinded point                 | Asset, amount and blinded point, equal to the UTxO value |
| Withdrawal notes    | Not verified                           | Verified, and equal to the vault outflow                 |
| Vault spend         | The depositor signs                    | The delegate signs                                       |
| Vault address       | Header 0xF0 (a reward address)         | Header 0x70 (an enterprise script address)               |

## 15. Constants

| Name                                | Value                              |
| ----------------------------------- | ---------------------------------- |
| Group                               | Ristretto255                       |
| Denominations for each asset        | 64 (2^0 to 2^63)                   |
| Atoms in a refresh                  | 32 or less                         |
| Commitment input                    | 136 bytes                          |
| Asset encoding                      | 64 bytes                           |
| Policy ID                           | 28 bytes                           |
| Asset name                          | 32 bytes or less                   |
| Default confirmations for a deposit | 15 blocks                          |
| Default minimum deposit             | 1,000,000 lovelace                 |
| Default maximum transaction size    | 16,384 bytes                       |
| Default maximum withdrawal fee      | 2,000,000 lovelace, plus 5%        |
| Validator                           | Plutus V3, `validator/plutus.json` |
