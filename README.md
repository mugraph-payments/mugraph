# µgraph: Universal, Instant, Untraceable Payments

µgraph is an extremely fast payment network for UTXOs, implemented as a Layer 2 Network on top of Cardano. It is an intentionally simplified both in operation and capabilities, to fulfill a single use-case: payments. Unlike normal Layer 2s, no assets can be minted inside the network.

Instead, it allows users to hold "vouchers", redeemable for funds inside Vaults in their origin Blockchain. Those vouchers are just files, transferrable between users completely off-band via NFCs, QR codes, or even text messages.

There are no wallets, no 24-word mnemonics, no private keys to protect. Just some strings inside a mobile application, encrypted by the operating system, and secured by the user's credential of choice (Pin, FaceID, TouchID).

When you spend cash, you don't announce it to the world, and you don't notify the Central Bank. µgraph is cash, but online.

## Documentation

1. [Motivation](./docs/motivation.md)
1. [Introduction](./docs/introduction.md)
1. [Specification](./docs/spec.md)

## License

µgraph (and all related projects inside the organization) is dual licensed under the [MIT](./LICENSE) and [Apache 2.0](./LICENSE.apache2) licenses. You are free to choose which one of the two choose your use-case the best, or please contact me if you need any form of expecial exceptions.

## Contributing Guidelines

All contributions are welcome, as long as they align with the goal of the project. If you are not sure whether or not what you want to implement is aligned with the goals of the project, just ask!

Don't be an asshole to anyone inside and out of the project and you'll be fine.
