# Milestone 3 — Cross-Node Payments Spec

Status: Draft

## 0) Core rule

**Blockchain-confirmed settlement is the source of truth.**

Inter-node messages are coordination signals (latency/recovery), not final truth.

## 1) Scope

Milestone 3 defines:
- cross-node transfer lifecycle
- chain-driven settlement/credit rules
- minimal persistence and recovery model
- compatibility behavior for mixed peers
- clear split of on-chain and off-chain responsibilities

Non-goals:
- exactly-once network delivery
- ACK-as-finality
- `/rpc` envelope redesign

## 2) Invariants

1. `transfer_id` is globally unique and immutable.
2. Source debit happens at most once per transfer.
3. Destination credit happens at most once per transfer.
4. Off-chain ACK disagreement cannot override chain-confirmed settlement.
5. Terminal failure/manual transitions are auditable.

## 3) Protocol shape (summary)

Required wire messages:
- `transfer_init`
- `transfer_notice`
- `transfer_status_query`
- `transfer_status`

Optional optimization:
- `transfer_ack`

See `milestone-3-inter-node-protocol-messages.md` for envelope and validation rules.
All inter-node traffic is transported through existing `/rpc` (`cross_node_transfer_*` methods); there is no separate transport surface.

## 4) Two-lane execution model (on-chain + off-chain)

Every transfer has two lanes that must converge:

1. **On-chain lane (authoritative)**
   - submit settlement transaction
   - observe canonical inclusion and confirmation depth
   - detect invalidation on deep reorg

2. **Off-chain lane (coordination)**
   - exchange `transfer_*` messages over `/rpc`
   - dedupe, replay-check, and persist progress hints
   - drive retries/status queries when messages are lost

Convergence rule:
- off-chain lane may accelerate completion, but cannot finalize against chain evidence.

Minimal handoff by phase:
- `transfer_init` accepted -> creates transfer row (off-chain start)
- transaction submitted -> binds `tx_hash` (on-chain start)
- `transfer_notice` delivered -> destination starts chain observation earlier
- destination credit applied -> only after chain threshold
- terminal success -> only after chain confirmation policy

## 5) Canonical on-chain settlement artifact (design freeze)

A cross-node transfer is represented on-chain by a **single settlement transaction** identified by `tx_hash`.

Required transaction evidence (normative):
1. transaction is on the expected Cardano network.
2. transaction includes metadata label `673` with fields:
   - `transfer_id`
   - `source_node_id`
   - `destination_node_id`
   - `asset`
   - `amount`
3. transaction includes at least one output controlled by destination node settlement address/policy.
4. one output must match (`asset`, `amount`) for the transfer intent.

Required persisted settlement evidence in `CROSS_NODE_TRANSFERS`:
- `tx_hash`
- `network`
- `settlement_output_index`
- `asset`
- `amount`
- `confirmations_observed`
- `canonical_inclusion` (bool)
- `observed_block_height`
- `last_canonical_height_checked`

### 5.1 Destination verification algorithm (deterministic)

On `transfer_notice(tx_hash, transfer_id, ...)`, destination MUST:
1. fetch tx by `tx_hash` from provider.
2. fail soft (retry path) if tx not yet visible.
3. verify metadata label `673` exists and matches transfer tuple.
4. verify destination-controlled output exists with required `asset` + `amount`.
5. bind `settlement_output_index` and set `chain_state=submitted|confirming`.
6. advance to credit eligibility only when confirmations `>= credit_target`.

If any hard check fails (metadata mismatch, wrong destination, amount mismatch), mark deterministic failure (`TRANSFER_STATE_CONFLICT` / `SCHEMA_VALIDATION_FAILED`) and do not credit.

### 5.2 Reorg invalidation criteria (explicit)

- **Shallow reorg** (`depth <= reorg_tolerance`): remain in `confirming`, continue polling.
- **Deep reorg** (`depth > reorg_tolerance`) OR tx disappears from canonical chain for more than `reorg_tolerance` consecutive checks: set `chain_state=invalidated`.
- If tx returns canonically with matching evidence, transition `invalidated -> confirming`.
- If not recoverable within retry budget/SLA, transition to `ManualReview` (destination may `held` or `reversed` per policy).

## 6) Lifecycle (simplified)

### Source
`Requested -> Submitted -> Confirming -> Confirmed`

Exceptional path:
`Confirming|Confirmed -> Invalidated -> (Confirming|ManualReview)`

### Destination
`NoticeReceived -> ChainObserved -> CreditEligible -> Credited`

Exceptional path:
`Credited -> Invalidated -> (Held/Reversed|ManualReview)`

Rules:
- crediting is chain-gated (`credit_target`)
- final success is chain-gated (`finality_target`)
- duplicates are idempotent no-ops

## 7) Persistence

Primary table:
- `CROSS_NODE_TRANSFERS` (authoritative transfer row)

Support tables:
- `CROSS_NODE_MESSAGES` (dedupe + transport diagnostics)
- `IDEMPOTENCY_KEYS`
- `TRANSFER_AUDIT_LOG`

Canonical enums:
- `chain_state`: `unknown|submitted|confirming|confirmed|invalidated`
- `credit_state`: `none|eligible|credited|held|reversed`

## 8) Recovery policy

Recovery order:
1. chain evidence
2. durable transfer row
3. message history (diagnostics)

Implications:
- lost ACK does not block convergence
- lost notice recovered by retry/status query
- deep reorg follows invalidated path deterministically

## 9) Compatibility

- Existing RPC methods remain (`cross_node_transfer_*`).
- Older peers: `unsupported_method:cross_node_transfer_*`.
- Unsupported legacy message types: `UNSUPPORTED_MESSAGE_TYPE` with no state mutation.

## 10) Normative references

- Glossary/defaults: `milestone-3-glossary-and-defaults.md`
- Protocol contract: `milestone-3-inter-node-protocol-messages.md`
- Security/reliability: `milestone-3-security-privacy-reliability.md`
- Observability/tests: `milestone-3-observability-test-plan.md`
- Implementation backlog: `milestone-3-implementation-backlog.md`

## 11) Policy decisions (resolved for M3 baseline)

Resolved defaults are defined in `milestone-3-glossary-and-defaults.md`:
- network profiles for `credit_target`, `finality_target`, `reorg_tolerance`
- destination invalidation default action = `held`
- manual-review closure quorum = `2-of-3` (`ops`, `security`, `risk/product`)

Any change to these values is an operational tuning change and must be recorded in release notes/runbooks.
