# Project Milestones

> [!WARNING]  
> This is a just a draft roadmap, and it is not final. It is subject to change, either because requirements change or because we learn something new.

```mermaid
gantt
    title Project Milestones
    dateFormat YYYY-MM-DD

    section Infrastructure
    Infrastructure Development :active, m1, 2024-08-01, 2024-10-31

    section Asset Management
    Asset Bridging             :m2, 2024-11-01, 2024-12-31

    section Payments
    Cross-node Payments        :m3, 2025-01-01, 2025-03-31

    section User Interface
    Mobile Wallet Development  :m4, 2025-04-01, 2025-04-30

    section Documentation & Release
    Protocol and Wallet Preparation, Version 1.0 Release :m5, 2025-05-01, 2025-05-31
```

This document outlines the key milestones necessary to complete the project successfully.

## Milestone 1: Infrastructure Development

The first milestone focuses on developing the infrastructure for processing payments, including the Node Software and Zero-Knowledge Proofs. Upon completion, the project will enable trustless payments between users within the same Node. This milestone does not include Vaults or the connection to the Cardano blockchain.

Expected deliverables:

1. Zero-Knowledge Circuits for generating proofs for transactions between users and nodes.
2. An installable node Docker image for nodes to blindly approve transactions with valid Zero-Knowledge Proofs.
3. A runnable specification of the protocol to facilitate the development of alternative clients and nodes.

## Milestone 2: Asset Bridging

The second milestone enables the "bridging of assets" between the Cardano Network and the µgraph.

This milestone includes Plutus Smart Contracts for:

1. Depositing ADA and Native Cardano assets into a Vault on the Cardano blockchain, allowing users to mint those funds in running network nodes.
2. Withdrawing ADA and Cardano assets with the Vault's acknowledgment.

## Milestone 3: Cross-node Payments

The third milestone implements cross-node payments, similar to the Lightning Network. This feature allows transaction confirmation without reaching the Cardano Blockchain, regardless of the participating nodes. It achieves this by "routing" transactions using Hydra Channels between nodes.

This milestone includes the mechanism to make these transfers, built as an extension to the protocol.

## Milestone 4: Mobile Wallet Development

Milestone 4 focuses on creating the Mobile Wallet for interacting with the network.

After the wallet release, users will be able to:

1. Send and receive funds via QR Codes or messages.
2. Pay and receive payments using a process similar to Google/Apple Pay (via NFC).

## Milestone 5: Protocol and Wallet Preparation

Milestone 5 prepares the protocol and wallet for release. This includes:

1. Protocol Specifications
2. Implementation Documentation
3. User Documentation
4. Reference Material
5. Technical Whitepapers
6. Presentation Material

## Final Milestone: Version 1.0 Release

The final milestone for this project is the release of Version 1.0 of the protocol to the global Cardano community and addressing any potential issues before launch.

The outputs for this final milestone include:

1. A project post-mortem detailing the project's progress and outlining next steps.
2. A video showcasing the project's accomplishments, targeting both project backers and new users.
3. A working product ready for developer adoption and mass user implementation.

**Important**: This milestone signifies the protocol's readiness for Testnet, not Mainnet. Achieving production-ready status requires additional steps, including user feedback and scrutiny, which may extend beyond the project's completion.