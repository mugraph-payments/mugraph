#!/usr/bin/env bash
set -euo pipefail

# Two-wallet demo orchestrator. Boots a mock Cardano chain and one
# mugraph node, then guides the operator through launching wallet A
# and wallet B with isolated data dirs. See docs/specs/demo.md.

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SESSION="mugraph-demo"
MOCK_ADDR="127.0.0.1:8090"
NODE_ADDR="127.0.0.1:9999"
WALLET_A_DIR="$ROOT/.demo/wallet-a"
WALLET_B_DIR="$ROOT/.demo/wallet-b"
NODE_DIR="$ROOT/.demo/node"

mock_url() { echo "http://${MOCK_ADDR}"; }
node_url() { echo "http://${NODE_ADDR}"; }

usage() {
  cat <<EOF
demo.sh — two-wallet Mugraph demo orchestrator

Usage:
  demo.sh up                    boot mock-chain + node in tmux
  demo.sh down                  kill the demo tmux session
  demo.sh wallet <a|b>          print the launch command for wallet A or B
  demo.sh faucet <addr> <lovelace>  mint a UTxO at addr
  demo.sh mine [count]          mine count blocks (default 1)
  demo.sh state                 dump the mock chain's admin state
  demo.sh utxos <addr>          list live UTxOs at addr
  demo.sh reset                 wipe mock chain state
  demo.sh logs                  attach to the tmux session

Environment:
  MOCK_ADDR  override mock-chain bind addr (default $MOCK_ADDR)
  NODE_ADDR  override node bind addr (default $NODE_ADDR)
EOF
}

require_running() {
  if ! curl -sSf "$(mock_url)/admin/state" >/dev/null 2>&1; then
    echo "mock-chain is not running at $(mock_url) — run: $0 up" >&2
    exit 1
  fi
}

cmd_up() {
  mkdir -p "$WALLET_A_DIR" "$WALLET_B_DIR" "$NODE_DIR"

  echo "Building demo binaries..."
  cargo build --release \
    --manifest-path "$ROOT/Cargo.toml" \
    -p mock-chain -p mugraph-node

  local mock_bin="$ROOT/target/release/mock-chain"
  local node_bin="$ROOT/target/release/mugraph-node"

  tmux kill-session -t "$SESSION" 2>/dev/null || true

  tmux new-session -d -s "$SESSION" -n "demo" \
    "'$mock_bin' --addr $MOCK_ADDR; read"

  # Give the mock a moment to bind before the node tries to query it.
  tmux split-window -h -t "$SESSION:demo.0" \
    "sleep 1 && cd '$NODE_DIR' && MUGRAPH_DB_PATH='$NODE_DIR/db.redb' '$node_bin' server \
      --addr $NODE_ADDR \
      --cardano-network preprod \
      --cardano-provider blockfrost \
      --cardano-provider-url $(mock_url) \
      --cardano-api-key demo \
      --deposit-confirm-depth 1 \
      --seed 7; read"

  tmux select-layout -t "$SESSION:demo" even-horizontal
  tmux select-pane -t "$SESSION:demo.0"

  cat <<EOF
demo cluster booted.

Mock chain:    $(mock_url)
Mugraph node:  $(node_url)
Tmux session:  $SESSION   (attach with: $0 logs)

Next steps:
  $0 wallet a       # prints the launch command for wallet A
  $0 wallet b       # prints the launch command for wallet B

Then in each wallet, run guided setup with:
  Node URL (all 3 networks): $(node_url)
  Provider: blockfrost
  API key:  demo
  Provider URL override: $(mock_url)

Stop with: $0 down
EOF
}

cmd_down() {
  tmux kill-session -t "$SESSION" 2>/dev/null || true
  echo "demo session stopped."
}

cmd_wallet() {
  local which="${1:-}"
  local dir
  case "$which" in
    a|A) dir="$WALLET_A_DIR" ;;
    b|B) dir="$WALLET_B_DIR" ;;
    *) echo "usage: $0 wallet <a|b>" >&2; exit 2 ;;
  esac
  mkdir -p "$dir"
  cat <<EOF
Run wallet ${which^^} in a fresh terminal:

  MUGRAPH_WALLET_DATA_DIR='$dir' \\
    cargo tauri dev --manifest-path '$ROOT/wallet/src-tauri/Cargo.toml'

Or if you've already built the binary:

  MUGRAPH_WALLET_DATA_DIR='$dir' '$ROOT/target/debug/mugraph-wallet'
EOF
}

cmd_faucet() {
  require_running
  local addr="${1:-}"
  local lovelace="${2:-}"
  if [[ -z "$addr" || -z "$lovelace" ]]; then
    echo "usage: $0 faucet <address> <lovelace>" >&2
    exit 2
  fi
  curl -sSf -X POST -H "content-type: application/json" \
    -d "{\"address\":\"$addr\",\"lovelace\":$lovelace}" \
    "$(mock_url)/admin/faucet"
  echo
}

cmd_mine() {
  require_running
  local count="${1:-1}"
  curl -sSf -X POST -H "content-type: application/json" \
    -d "{\"count\":$count}" \
    "$(mock_url)/admin/mine"
  echo
}

cmd_state() {
  require_running
  curl -sSf "$(mock_url)/admin/state"
  echo
}

cmd_utxos() {
  require_running
  local addr="${1:-}"
  if [[ -z "$addr" ]]; then
    echo "usage: $0 utxos <address>" >&2
    exit 2
  fi
  curl -sSfG "$(mock_url)/addresses/$addr/utxos"
  echo
}

cmd_reset() {
  require_running
  curl -sSf -X POST "$(mock_url)/admin/reset"
  echo "mock chain reset."
}

cmd_logs() {
  tmux attach-session -t "$SESSION"
}

main() {
  local sub="${1:-}"
  shift || true
  case "$sub" in
    up) cmd_up "$@" ;;
    down) cmd_down "$@" ;;
    wallet) cmd_wallet "$@" ;;
    faucet) cmd_faucet "$@" ;;
    mine) cmd_mine "$@" ;;
    state) cmd_state "$@" ;;
    utxos) cmd_utxos "$@" ;;
    reset) cmd_reset "$@" ;;
    logs) cmd_logs "$@" ;;
    -h|--help|help|"") usage ;;
    *) echo "unknown command: $sub" >&2; usage; exit 2 ;;
  esac
}

main "$@"
