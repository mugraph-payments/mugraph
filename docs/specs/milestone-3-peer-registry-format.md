# Milestone 3 — Trusted Peer Registry Format

Status: Draft

This document defines the trusted peer registry file used for cross-node authentication.

## File format

- Format: JSON
- Config key / CLI flag: `xnode_peer_registry_file` / `--xnode-peer-registry-file`
- Env var: `XNODE_PEER_REGISTRY_FILE`

## Schema

```json
{
  "peers": [
    {
      "node_id": "node://alpha",
      "endpoint": "https://alpha.example/rpc",
      "auth_alg": "Ed25519",
      "kid": "alpha-k1",
      "public_key_hex": "<32-byte hex>",
      "revoked": false
    }
  ]
}
```

## Validation rules

1. `auth_alg` MUST be `Ed25519`.
2. `endpoint` MUST be a valid URL.
3. `public_key_hex` MUST decode to exactly 32 bytes.
4. `(node_id, kid)` pair MUST be unique.

## Key role separation

- Peer registry keys (`auth_alg=Ed25519`) are for **inter-node envelope auth only**.
- BDHKE/blind-signing keys are separate and not loaded from this registry.

## Rotation expectations

- Rotation is modeled by adding a new `kid` row for the same `node_id`.
- Revocation is modeled by setting `revoked=true` on old keys.
- During rollout, old/new keys may coexist.
