# Milestone 3 — Glossary and Defaults

Status: Draft

Settlement model: chain-confirmed evidence is authoritative; inter-node messages are coordination signals.
Execution model: each transfer has an on-chain lane (truth) and an off-chain lane (coordination).

## 1) Canonical naming

### 1.1 Wire message types

Required:
- `transfer_init`
- `transfer_notice`
- `transfer_status_query`
- `transfer_status`

Optional:
- `transfer_ack`

Legacy/non-required:
- `transfer_decision`
- `transfer_retry_scheduled`
- `transfer_failed`
- `transfer_cancel`

### 1.2 RPC -> protocol mapping

Transport rule: inter-node messages are carried via existing `/rpc` methods only (no separate endpoint).

- `cross_node_transfer_create` -> `transfer_init`
- `cross_node_transfer_notify` -> `transfer_notice`
- `cross_node_transfer_ack` -> `transfer_ack`
- `cross_node_transfer_status` -> `transfer_status`

(`transfer_notify` is non-canonical; use `transfer_notice`.)

### 1.3 Envelope fields

Required core fields:
- `version` (`major.minor`, current `3.0`)
- `message_type`, `message_id`, `transfer_id`
- `idempotency_key`, `correlation_id`
- `origin_node_id`, `destination_node_id`
- `sent_at`

Command-only field:
- `expires_at` (`transfer_init`, `transfer_notice`, and `transfer_ack` when used)

## 2) Default policies (configurable)

| Policy | Default |
|---|---|
| max clock skew (`sent_at`) | ±300s |
| max command expiry horizon | 15m |
| message dedupe retention | >= 7d |
| nonce retention | >= 24h |
| max notice retries | 12 |
| retry strategy | exponential backoff + jitter |
| idempotency tuple | (`origin_node_id`,`transfer_id`,`message_type`,`idempotency_key`) |
| audit hot retention | >= 90d |
| audit cold retention | >= 1y |
| raw payload retention | 30d (encrypted at rest) |

If another M3 doc conflicts on defaults, this file wins.

## 3) Status mapping contract

`transfer_status` MUST expose:
- `source_state`
- `destination_state`
- `settlement_state`
- `chain_state`
- `credit_state`

Canonical external `settlement_state`:
- `not_submitted`
- `submitted`
- `confirming`
- `confirmed`
- `invalidated`
- `manual_review`

Canonical status sub-enums:
- `chain_state`: `unknown|submitted|confirming|confirmed|invalidated`
- `credit_state`: `none|eligible|credited|held|reversed`

Internal states may be richer but must map deterministically.

## 4) Requirement ID namespaces

- `M3-PROTO-*` protocol
- `M3-SEC-*` security/privacy/reliability
- `M3-OBS-*` observability/testing
- `M3-REL-*` rollout/release
