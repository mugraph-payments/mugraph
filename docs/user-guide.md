# µgraph Wallet User Guide

This guide tells you how to use the µgraph wallet. The wallet keeps private
payment notes for Cardano assets. You can send notes to other persons, put
ADA into notes, and take ADA out of notes.

## 1. Before you start

### 1.1 How the wallet works

A note is like a banknote. It has a value in one asset, for example ADA. A
server, the node, holds the ADA for all notes in a Cardano script address,
the vault.

- A **deposit** puts ADA into the vault and gives you notes of the same value.
- A **send** gives notes to another person.
- A **withdrawal** burns notes and pays their ADA to a Cardano address.

The node can not see who holds a note. It sees only the asset and the amount.

All amounts in the wallet are in lovelace. One ADA is 1,000,000 lovelace.

### 1.2 What you need

- The URL of a µgraph node.
- A Blockfrost project key for the Cardano network that you use.
- Test ADA on that network, for a deposit.

> **Note:** Version 1.0 is for the Cardano test networks (preprod and preview).
> Do not use it with real funds.

## 2. Install the wallet

### 2.1 Android

1. Open the release page of the project on GitHub.
2. Download the Android package (`.apk`).
3. Open the package on your phone.
4. If Android asks, let your phone install apps from this source.
5. Install the app.

### 2.2 Desktop

The desktop app has no release package in version 1.0. A developer can start
it from the source code (see `docs/implementation.md`, section 3.5).

## 3. Set up the wallet

When you open the wallet for the first time, it shows the setup screen.

1. In **Wallet label**, enter a name for this wallet.
2. In **Node URLs**, enter the node URL for **Mainnet**, **Preprod** and **Preview**.
3. In **Cardano provider**, select **Blockfrost**.
4. In **API key**, enter your Blockfrost key.
5. Optional: enter a different provider URL. Leave it empty for the public Blockfrost service.
6. Complete the setup.

The wallet makes its keys on your device. Then it connects to the node on each
network and saves the node keys. The wallet starts on the preprod network.

> **Note:** Withdrawals need Blockfrost. The Maestro client in version 1.0
> can not read the protocol parameters that a withdrawal needs.

## 4. The screens

| Screen   | Content                                                                   |
| -------- | ------------------------------------------------------------------------- |
| Home     | Your balance, the actions to pay and to receive, and your recent activity |
| Assets   | Your notes, with their amounts and their status                           |
| Activity | A list of your deposits, sends, refreshes and withdrawals                 |
| Settings | The network, the node, the deposit and withdrawal screens, and sync       |

A note has one of these states:

| State       | Meaning                                                       |
| ----------- | ------------------------------------------------------------- |
| Available   | You can spend the note.                                       |
| Spent       | You gave the note away, or the node accepted it.              |
| Quarantined | The wallet could not verify the note, or the node refused it. |

The wallet syncs with the node every 30 seconds.

## 5. Deposit ADA

A deposit sends ADA from your funding address to the vault. Then the node
gives you notes of the same value.

### 5.1 Get test ADA

1. Open **Settings**.
2. Select **Deposit**.
3. Copy the **Cardano funding address**.
4. Send test ADA to this address, for example from the Cardano testnet faucet.

Keep at least 7 ADA in the funding address after a deposit. The wallet uses
5 ADA of it as collateral for withdrawals, and it needs ADA for the fees.

### 5.2 Make a deposit

1. Open **Settings**.
2. Select **Deposit**.
3. Select a **Funding UTxO**.
4. In **Note denominations (lovelace)**, enter the amount of the deposit.
5. Optional: select **Add denomination** to add more amounts.
6. Select **Submit deposit**.

The wallet splits each amount into notes of powers of two. The deposit fee is
0.2 ADA.

The node gives the notes after the deposit has enough confirmations on the
chain. On preprod, this takes some minutes. Until then, the deposit screen
shows that the wallet sent the deposit. The wallet asks the node again at each sync.
The notes then show on the Assets screen.

> **Caution:** Do not delete the wallet data before the notes arrive. The
> wallet keeps the secret data for the notes of the deposit. Without it, you
> can not get the notes, and the ADA stays in the vault.

## 6. Receive notes

Another person sends you notes as a text envelope.

1. Get the envelope from the sender, for example in a message.
2. On the Home screen, select the action to receive.
3. Paste the envelope.
4. Select **Import notes**.

The wallet verifies each note with the node keys. Then it asks the node for
new notes of the same value. After this step, only you know the new notes.

If the node refuses a note, the wallet puts it in quarantine. The node refuses
a note that someone spent before. For example, the sender spent it before your
import.

> **Caution:** Import notes as soon as you get them. Until your import, the
> sender can still spend the notes.

You can also make a receive request, to tell the sender what you want:

1. On the Home screen, select the action to receive.
2. Select an **Asset**, and enter the **Amount** and a **Label**.
3. Select **Generate request**.
4. Select **Copy envelope**, and send the text to the payer.

## 7. Send notes

1. On the Home screen, select the action to pay.
2. Select the notes to send. The screen shows their total.
3. Select **Send selected notes**.
4. Select **Copy envelope**.
5. Send the text to the receiver.

The wallet marks the notes as spent when it makes the envelope. Each note has
a value that is a power of two. Version 1.0 has no screen to split notes, so
you can send only whole notes. Select notes that add up to the amount, or to a
little more.

> **Caution:** Send each envelope to one person only. The receiver that
> imports it first gets the value.

## 8. Withdraw ADA

A withdrawal burns notes and pays ADA from the vault to a Cardano address.

1. Open **Settings**.
2. Select **Withdraw**.
3. In **Asset**, select ADA.
4. In **Amount (lovelace)**, enter the amount.
5. In **Destination address**, enter the Cardano address.
6. Select **Submit withdrawal**.

The notes pay the amount and the transaction fee. The fee is about 0.2 ADA.
If your notes have more value than necessary, the node gives you new notes
for the difference.

The withdrawal needs a UTxO of at least 5 ADA at your funding address, for
collateral. The chain takes the collateral only if the vault script fails.

## 9. Change the network

1. Open **Settings**.
2. In **Network**, select **Mainnet**, **Preprod** or **Preview**.

Each network has its own notes and its own node keys.

## 10. Safety

> **Warning:** The wallet keeps your keys and notes on your device without
> encryption. A person with access to your device can take your notes. Do not
> keep large values in the wallet.

> **Caution:** Notes are like cash. If you lose your device or delete the
> app data, you lose the notes. Version 1.0 has no backup function.

The attention banner shows when the wallet has quarantined notes, or secret
data from a request that did not complete. Version 1.0 shows the banner, but
it has no buttons to retry or discard these items.

## 11. Problems and solutions

| Problem                                         | Possible cause                                         | Solution                                                         |
| ----------------------------------------------- | ------------------------------------------------------ | ---------------------------------------------------------------- |
| The deposit shows as sent, but no notes arrive. | The deposit does not have enough confirmations yet.    | Wait some minutes. The wallet asks again at each sync.           |
| The setup fails.                                | The node URL is not correct, or the node does not run. | Make sure that the node URL is correct. Then do the setup again. |
| An import puts the notes in quarantine.         | The sender or another person spent the notes first.    | Ask the sender for other notes.                                  |
| A withdrawal fails with a collateral error.     | The funding address has no UTxO of 5 ADA or more.      | Send at least 5 ADA to the funding address.                      |
| A withdrawal fails with a provider error.       | The wallet uses Maestro.                               | Configure the wallet with Blockfrost.                            |
| The node refuses all notes.                     | The node has a new master key.                         | Ask the node operator. Notes from the old key are not valid.     |
