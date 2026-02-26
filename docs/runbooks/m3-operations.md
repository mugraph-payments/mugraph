# M3 Operations Runbook

Status: Draft

This runbook covers cross-node M3 operational handling for invalidation, manual-review, and rollback.

## 1) Invalidation response

Trigger signals:
- `chain_state=invalidated`
- `mugraph_m3_reorg_events_total{severity="deep"}` increases
- audit entries: `transfer.invalidated` or `reconciler.manual_review`

Operator actions:
1. Confirm canonical chain status from provider and compare `tx_hash`.
2. Inspect transfer timeline (`TRANSFER_AUDIT_LOG`) for replay/idempotency errors.
3. Keep destination in `held` until chain truth is stable.
4. If canonical recovery occurs, transition back to `confirming`/`confirmed` via reconciler.

## 2) Manual-review handling

Manual review entry conditions:
- retry exhaustion for `transfer_notice` / `transfer_status_query`
- invalidated transfer with `credit_state=held`

Checklist:
1. Verify transfer tuple (`transfer_id`, `origin_node_id`, `destination_node_id`).
2. Reconstruct timeline from audit events in chronological order.
3. Verify destination credit side effects (must be <= 1 credit per `transfer_id`).
4. Choose disposition:
   - keep `held` pending more evidence
   - mark `reversed` under explicit override
   - mark recovered path when chain reconverges

## 3) Rollback guidance

If regression is introduced in M3 handlers:
1. Pause outbound coordination workers/reconciler if required.
2. Preserve DB state; do not wipe `CROSS_NODE_*` or `TRANSFER_AUDIT_LOG`.
3. Roll back application version.
4. Run reconciler once after rollback and confirm:
   - no duplicate credits
   - no replay acceptance
   - no stale-ack terminal regressions
5. Capture incident note with affected `transfer_id`s and event timeline.

## 4) Minimal query snippets

Examples (conceptual):
- Fetch transfer row by `transfer_id` from `CROSS_NODE_TRANSFERS`
- Fetch message history from `CROSS_NODE_MESSAGES`
- Reconstruct ordered timeline from `TRANSFER_AUDIT_LOG`

## 5) Related specs

- `docs/specs/milestone-3-cross-node-payments.md`
- `docs/specs/milestone-3-security-privacy-reliability.md`
- `docs/specs/milestone-3-observability-test-plan.md`
