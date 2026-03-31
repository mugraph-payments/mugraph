# Milestone 3 — Inter-Node Protocol Contract

Status: Draft
Owner: Protocol

## 1) Model

- Chain evidence is authoritative.
- Transport is at-least-once.
- Handlers must be idempotent.
- Transport is the existing `/rpc` path using `cross_node_transfer_*` methods (no separate route/protocol).
- Off-chain messages carry chain hints/proofs; they do not replace chain verification.

## 2) Message set

Required:

1. `transfer_init`
2. `transfer_notice`
3. `transfer_status_query`
4. `transfer_status`

Optional: 5. `transfer_ack` (advisory only)

Legacy/non-required types may be accepted for compatibility telemetry; otherwise reject with `UNSUPPORTED_MESSAGE_TYPE` (no state mutation).

## 3) Canonical envelope

This is the logical inter-node message envelope carried inside existing `/rpc` request/response payloads.

```json
{
  "m": "xnode",
  "version": "3.0",
  "message_type": "transfer_notice",
  "message_id": "...",
  "transfer_id": "...",
  "idempotency_key": "nodeA::tr...::transfer_notice::v1",
  "correlation_id": "...",
  "origin_node_id": "node://source",
  "destination_node_id": "node://dest",
  "sent_at": "2026-02-25T16:30:00Z",
  "expires_at": "2026-02-25T16:35:00Z",
  "payload": {},
  "auth": { "alg": "Ed25519", "kid": "k1", "sig": "..." }
}
```

Rules:

- `m == "xnode"`
- `version` is `major.minor` (current `3.0`)
- signed canonical envelope with domain separation
- command messages require valid `sent_at` + `expires_at`
- if `transfer_ack` is used, treat it as command envelope

## 4) Minimal payload requirements

### `transfer_init`

- `asset`
- `amount` (positive integer string)
- `destination_account_ref`
- `source_intent_hash`

### `transfer_notice`

- `notice_stage` (`submitted|confirmed|finalized`)
- `tx_hash`
- `confirmations` (required for `confirmed|finalized`)

Receiver rule: `transfer_notice` is advisory input; destination must verify `tx_hash` on-chain before crediting.

### `transfer_status_query`

- `query_type` (`current|history`)

### `transfer_status`

- `source_state`
- `destination_state`
- `settlement_state`
- `chain_state`
- `credit_state`
- `tx_hash` (if known)
- `confirmations_observed`
- `updated_at`

### `transfer_ack` (optional)

- `ack_for_message_id`
- `ack_status` (`processed|duplicate|deferred|rejected`)
- `ack_at`

## 5) Delivery and dedupe

- Dedupe/idempotency tuple:
  `(origin_node_id, transfer_id, message_type, idempotency_key)`
- Same tuple => same semantic result.
- Best-effort ordering only; causal guards enforced by receiver.

## 6) Replay/idempotency controls

For command messages:

1. verify signature + trusted `kid`
2. check `sent_at` freshness window
3. check `expires_at`
4. dedupe `message_id`
5. enforce idempotency tuple consistency

Replay/conflict => deterministic rejection + security audit event.

## 7) Canonical errors

- `UNSUPPORTED_VERSION`
- `UNSUPPORTED_MESSAGE_TYPE`
- `SCHEMA_VALIDATION_FAILED`
- `INVALID_SIGNATURE`
- `UNKNOWN_KEY_ID`
- `AUTHZ_DENIED`
- `REPLAY_DETECTED`
- `IDEMPOTENCY_CONFLICT`
- `TRANSFER_NOT_FOUND`
- `TRANSFER_STATE_CONFLICT`
- `INTERNAL_ERROR`

## 8) Version negotiation

- Same major: accept, negotiate minor.
- Unsupported major: reject with `UNSUPPORTED_VERSION` + supported versions.
- No silent major downgrade.

## 9) References

- `milestone-3-glossary-and-defaults.md`
- `milestone-3-cross-node-payments.md`
- `milestone-3-security-privacy-reliability.md`
