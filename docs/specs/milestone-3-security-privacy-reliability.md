# Milestone 3 — Security, Privacy, Reliability

Status: Draft
Owner: Security/Protocol
Last updated: 2026-02-26

## 1) Objectives

1. Only trusted peers can execute inter-node actions.
2. Tampered/replayed messages are rejected pre-mutation.
3. Duplicate delivery cannot create duplicate side effects.
4. Data exposure is minimized.
5. Failure/recovery remains deterministic and auditable.

## 2) Core controls

Control split:
- off-chain controls protect message integrity/auth/replay
- on-chain controls protect settlement truth and credit eligibility

### AuthN/AuthZ
- Trusted peer registry: `node_id -> allowed kid(s) + validity + revocation`.
- Verify app-layer signatures on all command messages.
- Enforce destination binding (`destination_node_id == local_node_id`).
- Registry format reference: `docs/specs/milestone-3-peer-registry-format.md`.

### Replay/idempotency
For command messages (`transfer_init`, `transfer_notice`, `transfer_ack` when used):
- validate `sent_at` freshness
- validate `expires_at`
- dedupe `message_id`
- enforce idempotency tuple

Tuple:
`(origin_node_id, transfer_id, message_type, idempotency_key)`

### Atomicity
Transition write + idempotency outcome + audit event must commit atomically.

### Admission/overload policy (M3 scope)
- no dedicated rate-limiting requirement in Milestone 3 initial scope
- bounded retries with backoff+jitter still apply for reliability
- overload behavior uses generic retriable/internal failures and recovery paths

### Privacy
- transmit protocol-minimum fields only
- no raw keys/signatures/nonces in normal logs
- redact sensitive references in operational logs

## 3) Reliability rules

- Chain-confirmed truth overrides off-chain ACK disagreement.
- Lost ACK must not block convergence.
- Recovery order: chain -> transfer store -> message history.
- Reorg invalidation path must be deterministic.
- Off-chain success without chain confirmation is never terminal success.
- Default invalidation posture is `held`; `reversed` requires explicit override flow.

Invariants:
1. at most one destination credit per `transfer_id`
2. stale ACK cannot regress terminal confirmed state
3. restart recovery is idempotent

## 4) Failure matrix (minimal)

| Failure | Deterministic behavior | Recovery |
|---|---|---|
| peer/network unavailable | retry notice/status query | converge on recovery or manual review |
| duplicate notice | no-op + deterministic duplicate response | none |
| replay attempt | reject + security audit | none |
| provider transient error | retry submit/poll | bounded retry + reconcile |
| deep reorg invalidation | mark invalidated, halt normal flow | restore canonical or held/reversed/manual |
| crash mid-operation | resume from durable state | startup reconciler |

## 5) Requirement IDs

- `M3-SEC-01` trusted peer authentication
- `M3-SEC-02` signed envelope integrity
- `M3-SEC-03` replay prevention
- `M3-SEC-04` idempotent duplicate handling
- `M3-SEC-05` stale-ack non-regression
- `M3-SEC-PRIV-01` data minimization/redaction
- `M3-SEC-REL-01` deterministic partition recovery
- `M3-SEC-REL-02` provider fault safety
- `M3-SEC-REL-03` reorg invalidation handling

## 6) Verification minimum

- spoofing/invalid signature tests
- replay and idempotency conflict tests
- duplicate-delivery no-double-credit tests
- stale-ack non-regression tests
- partition/provider/reorg/crash recovery tests
- privacy/log-redaction contract tests
