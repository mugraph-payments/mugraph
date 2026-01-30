#!/usr/bin/env bash
set -euo pipefail

# Commit funds to the Mugraph validator, reusing the reference script UTxO deployed
# with scripts/deploy_reference_script.sh. Builds and submits the transaction via
# Blockfrost and cardano-cli (no local node required).
# Requires: cardano-cli in PATH, jq, curl, funding UTxO, reference script UTxO, payment key.
# Usage:
#   scripts/commit_funds.sh \
#     --payment-skey payment.skey \
#     --funding-tx TXHASH \
#     --funding-ix INDEX \
#     --change-addr "$(cat payment.addr)" \
#     --amount 5000000 \
#     --user-pkh <28-byte-hex> \
#     --node-pkh <28-byte-hex> \
#     --reference-tx REF_TXHASH \
#     --reference-ix REF_INDEX \
#     --blockfrost-key PROJECT_ID \
#     [--intent-hash <hex>] \
#     [--network preprod] \
#     [--protocol-params protocol.json]

NETWORK=${CARDANO_NETWORK:-preprod}
AMOUNT=""
INTENT_HASH=""
PROTOCOL_PARAMS=""
BLOCKFROST_KEY="${BLOCKFROST_PROJECT_ID:-}"
FEE=200000

while [[ $# -gt 0 ]]; do
  case "$1" in
  --payment-skey)
    PAYMENT_SKEY="$2"
    shift 2
    ;;
  --funding-tx)
    FUND_TX="$2"
    shift 2
    ;;
  --funding-ix)
    FUND_IX="$2"
    shift 2
    ;;
  --change-addr)
    CHANGE_ADDR="$2"
    shift 2
    ;;
  --amount)
    AMOUNT="$2"
    shift 2
    ;;
  --user-pkh)
    USER_PKH="$2"
    shift 2
    ;;
  --node-pkh)
    NODE_PKH="$2"
    shift 2
    ;;
  --intent-hash)
    INTENT_HASH="$2"
    shift 2
    ;;
  --reference-tx)
    REF_TX="$2"
    shift 2
    ;;
  --reference-ix)
    REF_IX="$2"
    shift 2
    ;;
  --network)
    NETWORK="$2"
    shift 2
    ;;
  --protocol-params)
    PROTOCOL_PARAMS="$2"
    shift 2
    ;;
  --blockfrost-key)
    BLOCKFROST_KEY="$2"
    shift 2
    ;;
  *)
    echo "Unknown arg $1"
    exit 1
    ;;
  esac
done

for v in PAYMENT_SKEY FUND_TX FUND_IX CHANGE_ADDR AMOUNT USER_PKH NODE_PKH REF_TX REF_IX; do
  if [[ -z "${!v:-}" ]]; then
    echo "Missing required arg: --${v,,}"
    exit 1
  fi
done

if [[ ! "$USER_PKH" =~ ^[a-fA-F0-9]{56}$ ]]; then
  echo "user-pkh must be 28-byte hex (56 chars), got '$USER_PKH'"
  exit 1
fi

if [[ ! "$NODE_PKH" =~ ^[a-fA-F0-9]{56}$ ]]; then
  echo "node-pkh must be 28-byte hex (56 chars), got '$NODE_PKH'"
  exit 1
fi

if [[ -n "$INTENT_HASH" ]]; then
  if [[ ! "$INTENT_HASH" =~ ^[a-fA-F0-9]+$ ]]; then
    echo "intent-hash must be hex (or omitted for empty intent)"
    exit 1
  fi
else
  INTENT_HASH=""
fi

if [[ -z "$BLOCKFROST_KEY" ]]; then
  echo "Missing Blockfrost API key: set BLOCKFROST_PROJECT_ID or pass --blockfrost-key"
  exit 1
fi

key_prefix=$(echo "$BLOCKFROST_KEY" | cut -d'_' -f1)
case "$NETWORK" in
mainnet)
  if [[ "$key_prefix" != "mainnet" ]]; then
    echo "WARNING: --network=mainnet but Blockfrost key starts with '$key_prefix'"
  fi
  MAGIC_FLAG=(--mainnet)
  BLOCKFROST_URL="https://cardano-mainnet.blockfrost.io/api/v0"
  ;;
preprod)
  if [[ "$key_prefix" != "preprod" ]]; then
    echo "WARNING: --network=preprod but Blockfrost key starts with '$key_prefix'"
  fi
  MAGIC_FLAG=(--testnet-magic 1)
  BLOCKFROST_URL="https://cardano-preprod.blockfrost.io/api/v0"
  ;;
preview)
  if [[ "$key_prefix" != "preview" ]]; then
    echo "WARNING: --network=preview but Blockfrost key starts with '$key_prefix'"
  fi
  MAGIC_FLAG=(--testnet-magic 2)
  BLOCKFROST_URL="https://cardano-preview.blockfrost.io/api/v0"
  ;;
*)
  echo "Unsupported network: $NETWORK (use mainnet|preprod|preview)"
  exit 1
  ;;
esac

blockfrost_get() {
  local endpoint="$1"
  curl -s -H "project_id: $BLOCKFROST_KEY" "$BLOCKFROST_URL$endpoint"
}

blockfrost_post() {
  local endpoint="$1"
  local data="$2"
  curl -s -X POST -H "project_id: $BLOCKFROST_KEY" -H "Content-Type: application/cbor" --data-binary @"$data" "$BLOCKFROST_URL$endpoint"
}

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
VALIDATOR_DIR="$ROOT_DIR/validator"
CBOR_HEX="$VALIDATOR_DIR/plutus.json"

if [[ ! -f "$CBOR_HEX" ]]; then
  echo "Building validator..."
  (cd "$VALIDATOR_DIR" && aiken build)
fi

SCRIPT_CBOR_HEX=$(jq -r '.validators[0].compiledCode' "$CBOR_HEX")
SCRIPT_FILE=$(mktemp)
SCRIPT_JSON=$(mktemp)
printf "%s" "$SCRIPT_CBOR_HEX" | xxd -r -p >"$SCRIPT_FILE"
cat >"$SCRIPT_JSON" <<JSON
{"type": "PlutusScriptV2", "description": "aiken validator", "cborHex": "$SCRIPT_CBOR_HEX"}
JSON

SCRIPT_ADDR=$(cardano-cli address build --payment-script-file "$SCRIPT_JSON" "${MAGIC_FLAG[@]}")
echo "Script address: $SCRIPT_ADDR"

REF_UTXOS=$(blockfrost_get "/txs/${REF_TX}/utxos")
REF_OUT=$(echo "$REF_UTXOS" | jq -r --arg ix "$REF_IX" '.outputs[$ix | tonumber]')
if [[ -z "$REF_OUT" || "$REF_OUT" == "null" ]]; then
  echo "Error: reference UTxO not found at ${REF_TX}#$REF_IX"
  exit 1
fi

REF_SCRIPT_HASH=$(echo "$REF_OUT" | jq -r '.reference_script_hash // empty')
if [[ -z "$REF_SCRIPT_HASH" ]]; then
  echo "Error: provided reference UTxO does not contain a reference script (hash missing)"
  exit 1
fi
echo "Reference script hash: $REF_SCRIPT_HASH"

TX_UTXOS=$(blockfrost_get "/txs/${FUND_TX}/utxos")
FUND_UTXO=$(echo "$TX_UTXOS" | jq -r --arg ix "$FUND_IX" '.outputs[$ix | tonumber]')
if [[ -z "$FUND_UTXO" || "$FUND_UTXO" == "null" ]]; then
  echo "Error: Failed to fetch funding UTxO (tx: $FUND_TX, index: $FUND_IX)"
  exit 1
fi

FUND_AMOUNT=$(echo "$FUND_UTXO" | jq -r '.amount[] | select(.unit=="lovelace") | .quantity')
if [[ -z "$FUND_AMOUNT" || "$FUND_AMOUNT" == "null" ]]; then
  echo "Error: Failed to parse funding amount"
  exit 1
fi

echo "Funding UTxO amount: $FUND_AMOUNT lovelace"

LATEST_BLOCK=$(blockfrost_get "/blocks/latest")
CURRENT_SLOT=$(echo "$LATEST_BLOCK" | jq -r '.slot')
if [[ -z "$CURRENT_SLOT" || "$CURRENT_SLOT" == "null" ]]; then
  echo "Error: Failed to fetch latest block from Blockfrost"
  exit 1
fi
TTL=$((CURRENT_SLOT + 3600))
echo "Current slot: $CURRENT_SLOT, TTL: $TTL"

txbody=$(mktemp)
txsigned=$(mktemp)
datumfile=$(mktemp)
trap 'rm -f "$SCRIPT_FILE" "$SCRIPT_JSON" "$txbody" "$txsigned" "$datumfile" "${ppfile:-}"' EXIT

cat >"$datumfile" <<JSON
{
  "constructor": 0,
  "fields": [
    {"bytes": "$USER_PKH"},
    {"bytes": "$NODE_PKH"},
    {"bytes": "$INTENT_HASH"}
  ]
}
JSON

if [[ -z "$PROTOCOL_PARAMS" ]]; then
  echo "Fetching protocol parameters from Blockfrost..."
  ppfile=$(mktemp)
  PROTOCOL_PARAMS="$ppfile"
  blockfrost_get "/epochs/latest/parameters" >"$PROTOCOL_PARAMS"
fi

CHANGE_AMOUNT=$((FUND_AMOUNT - AMOUNT - FEE))
if [[ $CHANGE_AMOUNT -lt 0 ]]; then
  echo "Error: Insufficient funds. Need at least $((AMOUNT + FEE)) lovelace, have $FUND_AMOUNT"
  exit 1
fi

echo "Building raw transaction..."
echo "  Commitment: $AMOUNT lovelace to script"
echo "  Fee:        $FEE lovelace"
echo "  Change:     $CHANGE_AMOUNT lovelace"

cardano-cli conway transaction build-raw \
  --tx-in "${FUND_TX}#${FUND_IX}" \
  --tx-out "${SCRIPT_ADDR}+${AMOUNT}" \
  --tx-out-inline-datum-file "$datumfile" \
  --read-only-tx-in-reference "${REF_TX}#${REF_IX}" \
  --tx-out "${CHANGE_ADDR}+${CHANGE_AMOUNT}" \
  --invalid-hereafter $TTL \
  --fee $FEE \
  --out-file "$txbody"

cardano-cli conway transaction sign \
  --tx-body-file "$txbody" \
  --signing-key-file "$PAYMENT_SKEY" \
  "${MAGIC_FLAG[@]}" \
  --out-file "$txsigned"

echo "Submitting transaction to Blockfrost..."
TX_CBOR_HEX=$(jq -r '.cborHex' "$txsigned")
TX_CBOR_BIN=$(mktemp)
printf "%s" "$TX_CBOR_HEX" | xxd -r -p >"$TX_CBOR_BIN"
trap 'rm -f "$SCRIPT_FILE" "$SCRIPT_JSON" "$txbody" "$txsigned" "$TX_CBOR_BIN" "$datumfile" "${ppfile:-}"' EXIT

SUBMIT_RESPONSE=$(blockfrost_post "/tx/submit" "$TX_CBOR_BIN")

if [[ "$SUBMIT_RESPONSE" =~ ^[a-fA-F0-9]{64}$ ]]; then
  THASH="$SUBMIT_RESPONSE"
  echo "Transaction submitted successfully!"
  echo "Committed script UTxO: ${THASH}#0"
  echo "Change UTxO: ${THASH}#1"
else
  echo "Error submitting transaction:"
  echo "$SUBMIT_RESPONSE" | jq '.'
  exit 1
fi
