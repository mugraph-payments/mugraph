<p align="center">
  <picture>
    <source srcset="docs/assets/logo-white.svg" media="(prefers-color-scheme: dark)">
    <img src="docs/assets/logo-dark.svg" alt="Mugraph Logo" width="300">
  </picture>
</p>

Mugraph (pronounced *"mew-graph"*) is a Layer 2 Network for the Cardano blochchain, with anonymous payments and instant finality.

It is not a blockchain but a payment network, where **Delegators** hold and transact funds (inside a smart contract vault) on behalf of users. Delegators can connect between each other as **Galaxies** and route payments directly via [Hydra Heads](https://hydra.family), and do not know anything about the users transacting in their vault, nor do they control any of the funds deposited there.

It is intentionally simplified in design and operation to fix only one problem: payments.

## Motivation

> Why are people not using cryptocurrencies for payments?

This is the question we wanted to address when Mugraph started. Bitcoin was creted to address this specific problem, yet [The Lightning Network](https://lightning.network) is not widely supported even in countries which adopted Bitcoin as legal tender.

Even if Cardano has much more advanced technology than Bitcoin does (like eUTXO or Smart Contracts), buying groceries with [USDM](https://mehen.io) remains nighly impossible.

ZeroHedge explains this phenomenon in their article ["What Happened to Bitcoin?"](https://www.zerohedge.com/crypto/what-happened-bitcoin):

> At the same time, new technologies were becoming available that vastly improved the efficiency and availability of exchange in fiat dollars. They included Venmo, Zelle, CashApp, FB payments, and many others besides, in addition to smartphone attachments and iPads that enabled any merchant of any size to process credit cards. These technologies were completely different from Bitcoin because they were permission-based and mediated by financial companies. But to users, they seemed great and their presence in the marketplace crowded out the use case of Bitcoin at the very time that my beloved technology had become an unrecognizable version of itself.

Excluding volatility (already being taken seriously by Stablecoins), we identified three main problems that prevent the average person from considering cryptocurrencies as a payment option:

1. **Scalability**: Cryptocurrency transactions are slow compared to centralized solutions. For example, credit cards have a practical confirmation limit of 2 seconds.

1. **Privacy**: Users do not want to reveal their identity for every purchase. Financial privacy is a human right, and in the age of big data analysis and AI, pseudonymity does not provide sufficient privacy.

1. **Ease of Use**: Users prefer not to deal with complex protocols, multiple wallets, or extensive security considerations.

Solving those reasons is why we created Mugraph.

## License

Mugraph (and all related projects within the organization) is dual-licensed under the [MIT](./LICENSE) and [Apache 2.0](./LICENSE-APACHE) licenses.

You are free to choose the license that best suits your use case. If neither license meets your needs, please contact us to discuss alternative arrangements.

## Contributing Guidelines

We welcome all contributions that align with the project's goals.

There's only one rule: **do not be an asshole**.

If you are unsure, just talk to us.
