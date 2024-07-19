<p align="center">
  <picture>
    <source srcset="docs/assets/logo-white.svg" media="(prefers-color-scheme: dark)">
    <img src="docs/assets/logo-dark.svg" alt="µgraph Logo" width="300">
  </picture>
</p>

µgraph is an extremely fast payment network for UTXOs, implemented as a Layer 2 Network on top of Cardano. It is intentionally simplified in both operation and capabilities to fulfill a single use case: payments. Unlike typical Layer 2 networks, no assets can be minted inside µgraph.

Instead, µgraph allows users to hold "vouchers," which are redeemable for funds inside Vaults in their origin Blockchain. These vouchers are files that users can transfer completely off-band through NFCs, QR codes, or even text messages.

µgraph eliminates the need for wallets, 24-word mnemonics, and private keys. It uses strings inside a mobile application, encrypted by the operating system and secured by the user's preferred credential (PIN, FaceID, or TouchID).

Similar to cash transactions, µgraph operates without announcing transactions to the world or notifying a central authority. It brings the privacy of cash to online transactions.

## Motivation

We began by asking: Why are people not using cryptocurrencies for payments? Bitcoin was created to address this specific problem, yet buying groceries with [USDM](https://mehen.io) remains challenging. ZeroHedge explains this phenomenon in their article ["What Happened to Bitcoin?"](https://www.zerohedge.com/crypto/what-happened-bitcoin):

> At the same time, new technologies were becoming available that vastly improved the efficiency and availability of exchange in fiat dollars. They included Venmo, Zelle, CashApp, FB payments, and many others besides, in addition to smartphone attachments and iPads that enabled any merchant of any size to process credit cards. These technologies were completely different from Bitcoin because they were permission-based and mediated by financial companies. But to users, they seemed great and their presence in the marketplace crowded out the use case of Bitcoin at the very time that my beloved technology had become an unrecognizable version of itself.

We identified four main problems that prevent the average person from considering cryptocurrencies as a payment option:

1. **Volatility**: Users desire a stable notion of "value" that is easy to understand and stable enough for short-term planning. They also avoid spending assets they believe will appreciate in value.

1. **Scalability**: Cryptocurrency transactions are slow compared to centralized solutions. For example, credit cards have a practical confirmation limit of 2 seconds.

1. **Privacy**: Users do not want to reveal their identity for every purchase. Financial privacy is a human right, and in the age of big data analysis and AI, pseudonymity does not provide sufficient privacy.

1. **Ease of Use**: Users prefer not to deal with complex protocols, multiple wallets, or extensive security considerations.

Of these problems, only volatility has been seriously addressed through Synthetic Assets and Stablecoins. µgraph aims to solve the remaining issues.

## How µgraph Works

µgraph consists of a network of nodes called mu (pronounced "mew"). Each mu approves transactions on behalf of users, similar to a centralized payment processor. However, unlike centralized processors, mu cannot transact independently; they require approval from a user with sufficient liquidity to cover the transaction.

Users send money to a mu by interacting with a smart contract stored on the blockchain. When users deposit tokens, those tokens become instantly available within the network.

Transactions between users within the same mu are free, instant, and anonymous. The server does not know the identities of the transacting parties.

For transactions between different mu, the mu itself (not the user) sends the transaction. This approach protects all users within the same mu under a shared identity. These transactions incur a fee beyond the cost of the Cardano network itself.

Mu connect to each other through Hydra channels, forming a Constellation. While the scope of Constellations is not covered in this document, they route transactions between sets of mu.

## License

µgraph (and all related projects within the organization) is dual-licensed under the [MIT](./LICENSE) and [Apache 2.0](./LICENSE-APACHE) licenses.

You are free to choose the license that best suits your use case. If neither license meets your needs, please contact us to discuss alternative arrangements.

## Contributing Guidelines

We welcome all contributions that align with the project's goals.

There's only one rule: **do not be an asshole**.

If you are unsure, just talk to us.
