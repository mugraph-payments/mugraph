# Changelog

## 1.0.0

Version 1.0.0 is the first complete release of µgraph, for the Cardano test
networks. It is not compatible with version 0.1. Notes, keys and vault
addresses from version 0.1 do not work with version 1.0.0.

### Security

- The hash to the group now has an unknown discrete log. Before this change,
  any person with the public key of a delegate could forge notes.
- The node now refuses a refresh input with an invalid signature.
- The delegate signs each note with a key for its asset and its amount. Before
  this change, a deposit or a refresh could give notes of any value.
- A refresh output now hides its nonce from the node. Before this change, the
  node could link each note to the refresh that made it.
- A deposit now gives notes of the exact value of the deposit UTxO.
- A withdrawal now verifies the notes that it burns. The notes minus the
  change must equal the value that leaves the vault.
- The vault validator now requires the signature of the node. Before this
  change, a depositor could take the deposit back alone.
- The wallet no longer sends blinding factors to the node in a withdrawal.
- `mugraph-node generate-key` now prints the secret key, and it uses a key
  with full entropy.

### Added

- Denomination keys, keysets, and the `keys` RPC method.
- Full Plutus spends for withdrawals: the validator, redeemers, collateral,
  the script data hash, and fees from the protocol parameters.
- Pending deposit claims in the wallet. The wallet tries each claim again
  until the deposit has enough confirmations.
- Ledger checks in the mock chain: witnesses, required signers, redeemers,
  balance, fees, and the validator.
- An end-to-end test with the mock chain, a node and three wallets.
- The protocol specification, the technical whitepaper, the implementation
  guide, the user guide, the API reference, and a slide deck.

### Changed

- Note amounts are powers of two.
- `DepositRequest.outputs` is a list of `BlindedOutput`, with the asset and
  the amount of each output.
- `WithdrawRequest.notes` is a list of `Note`, and `change_outputs` is a list
  of `BlindedOutput`.
- The `public_key` result has the new field `cardano_script_cbor`.
- The vault address is an enterprise script address (header `0x70`).
- The node takes the validator from the blueprint in its binary.
- The Docker image starts the `server` command.

### Removed

- The label 1914 metadata check on withdrawals.
- The deposit record check on withdrawal inputs. Any vault UTxO of the node can
  pay for a withdrawal.

### Known limitations

- The protocol has no keyset IDs and no key rotation.
- The node has no proof of reserves.
- The node does not check the settlement of payments between nodes.
- The wallet keeps its data without encryption, and it has no QR codes and no
  screen to split notes.
- Withdrawals need the Blockfrost provider.

### Upgrade from 0.1

Start the node with a new database and a key from `generate-key`. Configure
each wallet again. Funds at the version 0.1 vault address stay under the
version 0.1 validator.
