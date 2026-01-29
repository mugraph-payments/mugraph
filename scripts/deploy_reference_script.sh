#!/usr/bin/env bash
set -euo pipefail

# Deploy the Aiken validator as a reference script UTxO using Blockfrost.
# Requires: cardano-cli in PATH, jq, curl, funding UTxO, payment keys.
# Network: set via --network mainnet|preprod|preview or CARDANO_NETWORK env.
# Usage:
#   scripts/deploy_reference_script.sh \
#     --payment-skey payment.skey \
#     --payment-vkey payment.vkey \
#     --funding-tx TXHASH \
#     --funding-ix INDEX \
#     --change-addr payment.addr \
#     --out-addr payment.addr \
#     --blockfrost-key PROJECT_ID \
#     [--min-ada 3000000] \
#     [--network preprod] \
#     [--protocol-params protocol.json]

NETWORK=${CARDANO_NETWORK:-preprod}
MIN_ADA=3000000
PROTOCOL_PARAMS=""
BLOCKFROST_KEY="${BLOCKFROST_PROJECT_ID:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
  --payment-skey)
    PAYMENT_SKEY="$2"
    shift 2
    ;;
  --payment-vkey)
    PAYMENT_VKEY="$2"
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
  --out-addr)
    OUT_ADDR="$2"
    shift 2
    ;;
  --min-ada)
    MIN_ADA="$2"
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

for v in PAYMENT_SKEY PAYMENT_VKEY FUND_TX FUND_IX CHANGE_ADDR OUT_ADDR; do
  if [[ -z "${!v:-}" ]]; then
    echo "Missing required arg: --${v,,}"
    exit 1
  fi
done

# Validate funding transaction hash format
if [[ "$FUND_TX" == addr* ]] || [[ "$FUND_TX" == "addr1"* ]] || [[ "$FUND_TX" == "addr_test1"* ]]; then
  echo "Error: --funding-tx looks like a Cardano address, not a transaction hash"
  echo "       You provided: $FUND_TX"
  echo "       Transaction hashes look like: abc123def456... (64 hex characters)"
  echo ""
  echo "       To find a UTxO to use as funding:"
  echo "       1. Look up your address UTxOs on a Cardano explorer"
  echo "       2. Or use: cardano-cli query utxo --address <your_address>"
  exit 1
fi

# Transaction hashes should be 64 hex characters
if [[ ! "$FUND_TX" =~ ^[a-fA-F0-9]{64}$ ]]; then
  echo "Warning: --funding-tx doesn't look like a valid Cardano transaction hash"
  echo "         Expected: 64 hexadecimal characters"
  echo "         Got: $FUND_TX (length: ${#FUND_TX})"
fi

if [[ -z "$BLOCKFROST_KEY" ]]; then
  echo "Missing Blockfrost API key: set BLOCKFROST_PROJECT_ID or pass --blockfrost-key"
  exit 1
fi

# Validate Blockfrost key matches network
key_prefix=$(echo "$BLOCKFROST_KEY" | cut -d'_' -f1)
case "$NETWORK" in
mainnet)
  if [[ "$key_prefix" != "mainnet" ]]; then
    echo "WARNING: Using --network=mainnet but Blockfrost key starts with '$key_prefix'"
    echo "         Blockfrost project IDs should match the network: mainnet_... for mainnet"
  fi
  ;;
preprod)
  if [[ "$key_prefix" != "preprod" ]]; then
    echo "WARNING: Using --network=preprod but Blockfrost key starts with '$key_prefix'"
    echo "         Blockfrost project IDs should match the network: preprod_... for preprod"
  fi
  ;;
preview)
  if [[ "$key_prefix" != "preview" ]]; then
    echo "WARNING: Using --network=preview but Blockfrost key starts with '$key_prefix'"
    echo "         Blockfrost project IDs should match the network: preview_... for preview"
  fi
  ;;
esac

case "$NETWORK" in
mainnet)
  MAGIC_FLAG=(--mainnet)
  BLOCKFROST_URL="https://cardano-mainnet.blockfrost.io/api/v0"
  ;;
preprod)
  MAGIC_FLAG=(--testnet-magic 1)
  BLOCKFROST_URL="https://cardano-preprod.blockfrost.io/api/v0"
  ;;
preview)
  MAGIC_FLAG=(--testnet-magic 2)
  BLOCKFROST_URL="https://cardano-preview.blockfrost.io/api/v0"
  ;;
*)
  echo "Unsupported network: $NETWORK (use mainnet|preprod|preview)"
  exit 1
  ;;
esac

# Helper function for Blockfrost API calls
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
SCRIPT_FILE=$(mktemp) # raw cbor for inlining
SCRIPT_JSON=$(mktemp) # JSON wrapper for cardano-cli hash/address
printf "%s" "$SCRIPT_CBOR_HEX" | xxd -r -p >"$SCRIPT_FILE"
cat >"$SCRIPT_JSON" <<JSON
{"type": "PlutusScriptV2", "description": "aiken validator", "cborHex": "$SCRIPT_CBOR_HEX"}
JSON

SCRIPT_HASH=$(cardano-cli conway transaction policyid --script-file "$SCRIPT_JSON")
SCRIPT_ADDR=$(cardano-cli address build --payment-script-file "$SCRIPT_JSON" "${MAGIC_FLAG[@]}")

echo "Script hash:   $SCRIPT_HASH"
echo "Script address: $SCRIPT_ADDR"

txbody=$(mktemp)
txsigned=$(mktemp)
ppfile=""
trap 'rm -f "$SCRIPT_FILE" "$SCRIPT_JSON" "$txbody" "$txsigned" "${ppfile:-}"' EXIT

# Fetch protocol parameters from Blockfrost if not provided
if [[ -z "$PROTOCOL_PARAMS" ]]; then
  echo "Fetching protocol parameters from Blockfrost..."
  ppfile=$(mktemp)
  PROTOCOL_PARAMS="$ppfile"
  blockfrost_get "/epochs/latest/parameters" >"$PROTOCOL_PARAMS"
fi

# Get current slot for TTL
LATEST_BLOCK=$(blockfrost_get "/blocks/latest")
CURRENT_SLOT=$(echo "$LATEST_BLOCK" | jq -r '.slot')
if [[ -z "$CURRENT_SLOT" ]] || [[ "$CURRENT_SLOT" == "null" ]]; then
  echo "Error: Failed to fetch latest block from Blockfrost"
  echo "Response: $LATEST_BLOCK"
  if echo "$LATEST_BLOCK" | jq -e '.status_code == 403' >/dev/null 2>&1; then
    echo ""
    echo "HINT: Blockfrost returned 403 Forbidden - network token mismatch."
    echo "      Your Blockfrost project ID is for a different network than --network=$NETWORK"
    echo "      Make sure your BLOCKFROST_PROJECT_ID matches the network you're using."
  fi
  exit 1
fi
TTL=$((CURRENT_SLOT + 3600)) # Valid for 1 hour

# Get funding UTxO details (output of the funding transaction at the given index)
TX_UTXOS=$(blockfrost_get "/txs/${FUND_TX}/utxos")
FUND_UTXO=$(echo "$TX_UTXOS" | jq -r --arg ix "$FUND_IX" '.outputs[$ix | tonumber]')
if [[ -z "$FUND_UTXO" ]] || [[ "$FUND_UTXO" == "null" ]]; then
  echo "Error: Failed to fetch funding UTxO (tx: $FUND_TX, index: $FUND_IX)"
  echo "Response: $TX_UTXOS"
  if echo "$TX_UTXOS" | jq -e '.status_code == 404' >/dev/null 2>&1; then
    echo ""
    echo "HINT: Blockfrost returned 404 Not Found - the transaction doesn't exist on this network."
    echo "      Make sure:"
    echo "      1. The transaction hash is correct"
    echo "      2. The transaction exists on the $NETWORK network"
    echo "      3. You're using the correct --network flag (current: $NETWORK)"
  fi
  exit 1
fi
FUND_AMOUNT=$(echo "$FUND_UTXO" | jq -r '.amount[0].quantity')
if [[ -z "$FUND_AMOUNT" ]] || [[ "$FUND_AMOUNT" == "null" ]]; then
  echo "Error: Failed to parse funding amount"
  echo "UTxO: $FUND_UTXO"
  exit 1
fi

echo "Current slot: $CURRENT_SLOT, TTL: $TTL"
echo "Funding UTxO amount: $FUND_AMOUNT lovelace"

# Calculate fee (set to 0.2 ADA as safe default, will be calculated properly)
FEE=200000
CHANGE_AMOUNT=$((FUND_AMOUNT - MIN_ADA - FEE))

if [[ $CHANGE_AMOUNT -lt 0 ]]; then
  echo "Error: Insufficient funds. Need at least $((MIN_ADA + FEE)) lovelace, have $FUND_AMOUNT lovelace"
  exit 1
fi

echo "Building raw transaction..."
echo "  Fee: $FEE lovelace"
echo "  Change: $CHANGE_AMOUNT lovelace"

# Build raw transaction
cardano-cli conway transaction build-raw \
  --tx-in "${FUND_TX}#${FUND_IX}" \
  --tx-out "${OUT_ADDR}+${MIN_ADA}" \
  --tx-out-reference-script-file "$SCRIPT_JSON" \
  --tx-out "${CHANGE_ADDR}+${CHANGE_AMOUNT}" \
  --invalid-hereafter $TTL \
  --fee $FEE \
  --out-file "$txbody"

# Calculate exact fee
# echo "Calculating transaction fee..."
# FEE=$(cardano-cli conway transaction calculate-min-fee \
#   --tx-body-file "$txbody" \
#   --witness-count 1 \
#   --protocol-params-file "$PROTOCOL_PARAMS" \
#   | cut -d' ' -f1)

echo "Exact fee: $FEE lovelace"

# Recalculate change with exact fee
CHANGE_AMOUNT=$((FUND_AMOUNT - MIN_ADA - FEE))

if [[ $CHANGE_AMOUNT -lt 0 ]]; then
  echo "Error: Insufficient funds. Need at least $((MIN_ADA + FEE)) lovelace, have $FUND_AMOUNT lovelace"
  exit 1
fi

# Rebuild with exact fee
cardano-cli conway transaction build-raw \
  --tx-in "${FUND_TX}#${FUND_IX}" \
  --tx-out "${OUT_ADDR}+${MIN_ADA}" \
  --tx-out-reference-script-file "$SCRIPT_JSON" \
  --tx-out "${CHANGE_ADDR}+${CHANGE_AMOUNT}" \
  --invalid-hereafter $TTL \
  --fee $FEE \
  --out-file "$txbody"

cardano-cli conway transaction sign \
  --tx-body-file "$txbody" \
  --signing-key-file "$PAYMENT_SKEY" \
  "${MAGIC_FLAG[@]}" \
  --out-file "$txsigned"

# Submit via Blockfrost
echo "Submitting transaction to Blockfrost..."

# Convert signed transaction from text envelope to binary CBOR
TX_CBOR_HEX=$(jq -r '.cborHex' "$txsigned")
TX_CBOR_BIN=$(mktemp)
printf "%s" "$TX_CBOR_HEX" | xxd -r -p > "$TX_CBOR_BIN"
trap 'rm -f "$SCRIPT_FILE" "$SCRIPT_JSON" "$txbody" "$txsigned" "$TX_CBOR_BIN" "${ppfile:-}"' EXIT

SUBMIT_RESPONSE=$(blockfrost_post "/tx/submit" "$TX_CBOR_BIN")

# Blockfrost returns the tx hash directly as a string on success, or JSON on error
if [[ "$SUBMIT_RESPONSE" =~ ^[a-fA-F0-9]{64}$ ]]; then
  THASH="$SUBMIT_RESPONSE"
  echo "Transaction submitted successfully!"
  echo "Reference script UTxO: ${THASH}#0"
else
  echo "Error submitting transaction:"
  echo "$SUBMIT_RESPONSE" | jq '.'
  exit 1
fi
