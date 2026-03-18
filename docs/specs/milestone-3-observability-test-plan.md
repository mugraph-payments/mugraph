# Milestone 3 — Observability and Test Plan

Status: Draft
Owners: Platform / Node Runtime / QA
Last updated: 2026-02-26

## 1) Scope

Validate that M3:
1. converges with chain-authoritative settlement,
2. preserves idempotent safety under at-least-once delivery,
3. exposes enough telemetry for operations.

## 2) Required telemetry keys

All critical events:
- `transfer_id`
- `origin_node_id`
- `destination_node_id`
- `protocol_version`
- `state`

Message-scoped events:
- `message_id`
- `message_type`

Chain-scoped events:
- `tx_hash`
- `confirmations_observed`

Retry/security events should add:
- `idempotency_key`
- `attempt_no`
- `error_code`

## 3) Metrics

Prefix: `mugraph_`.

Metric split:
- on-chain convergence metrics (submission/confirmation/reorg)
- off-chain coordination metrics (messages/retries/duplicates/replay)

Core metrics (full names):
- `mugraph_transfers_initiated_total`
- `mugraph_transfers_terminal_total{terminal_state}`
- `mugraph_chain_submission_total{result,provider}`
- `mugraph_chain_confirmation_depth`
- `mugraph_settlement_latency_seconds`
- `mugraph_reorg_events_total{severity}`
- `mugraph_message_send_total{message_type,result}`
- `mugraph_message_retries_total{message_type,reason}`
- `mugraph_duplicate_messages_total{message_type}`
- `mugraph_replay_rejections_total{message_type}`
- `mugraph_idempotency_conflicts_total{operation}`
- `mugraph_stuck_transfers_gauge{state}`

## 4) Traces/logs/audits

Tracing spans (minimum):
- `transfer.create`
- `transfer.message.send`
- `transfer.message.receive`
- `transfer.chain.submit`
- `transfer.chain.confirmation_poll`
- `transfer.state.transition`
- `transfer.reconcile`

Mandatory audit events:
- `transfer.initiated`
- `transfer.notice.accepted`
- `transfer.credited`
- `transfer.confirmed`
- `transfer.invalidated`
- `transfer.replay_rejected`
- `transfer.idempotency_conflict`
- `transfer.manual_override`

## 5) Alerts (minimum)

- stuck transfer in critical states over threshold
- replay rejection spike
- terminal failure ratio breach
- reorg spike impacting active transfers
- protocol-version mismatch growth

## 6) Test strategy

### Unit
- telemetry propagation
- metric emission
- replay/idempotency validators

### Property
- no double credit with duplicate/reordered delivery
- no stale-ack terminal regression

### Integration
- full lifecycle with provider fakes
- crash/restart recovery
- reorg invalidation + reconciliation

### E2E/chaos
- two-node happy path
- destination downtime/recovery
- packet loss/timeouts/clock skew boundaries

## 7) Acceptance matrix

| Req ID | Requirement | Minimum validation |
|---|---|---|
| M3-OBS-01 | lifecycle events include `transfer_id` | unit + integration |
| M3-OBS-02 | message events include `message_id`/`message_type` | handler integration |
| M3-OBS-03 | retries/duplicates are measurable | duplicate/retry scenarios |
| M3-OBS-04 | stuck states are detectable + alerting | forced-stuck + alert test |
| M3-OBS-05 | chain linkage includes `tx_hash` + depth | provider integration |
| M3-OBS-06 | audit timeline reconstructs transfer | audit reconstruction test |
| M3-OBS-07 | replay rejection observable + safe | security + chaos skew |
| M3-OBS-08 | reorg invalidation emits signals + converges | reorg simulation integration |

## 8) CI gates

PR-required:
1. `unit+property`
2. `integration`
3. `compatibility` (protocol changes)

Release-candidate required:
4. `e2e+chaos`

M3 observability readiness requires all `M3-OBS-*` checks passing.
