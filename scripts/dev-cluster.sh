#!/usr/bin/env bash
set -euo pipefail

SESSION="mugraph-dev"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

NODE1_PORT=9999
NODE2_PORT=9998
NODE1_ADDR="127.0.0.1:${NODE1_PORT}"
NODE2_ADDR="127.0.0.1:${NODE2_PORT}"

# Per-node data directories so they don't share the redb database
NODE1_DIR="$ROOT/.dev/node-alpha"
NODE2_DIR="$ROOT/.dev/node-beta"
mkdir -p "$NODE1_DIR" "$NODE2_DIR"

# Kill any previous session with the same name
tmux kill-session -t "$SESSION" 2>/dev/null || true

# Build first so all three panes don't race on compilation
echo "Building workspace (release)..."
cargo build --release --manifest-path "$ROOT/Cargo.toml" -p mugraph-node -p mugraph-simulator

NODE_BIN="$ROOT/target/release/mugraph-node"
SIM_BIN="$ROOT/target/release/mugraph-simulator"

# Create session with the simulator in the main (left) pane.
# It waits for the nodes to come up before connecting.
tmux new-session -d -s "$SESSION" -n "cluster" \
  "sleep 2 && cd '$ROOT' && '$SIM_BIN' --node-url http://$NODE1_ADDR --node-url http://$NODE2_ADDR; read"

# Right side: split into two stacked panes for the nodes
tmux split-window -h -t "$SESSION:cluster.0" \
  "cd '$NODE1_DIR' && MUGRAPH_DB_PATH='$NODE1_DIR/db.redb' '$NODE_BIN' server --addr $NODE1_ADDR --dev-mode --seed 1 --xnode-node-id node://alpha; read"

tmux split-window -v -t "$SESSION:cluster.1" \
  "cd '$NODE2_DIR' && MUGRAPH_DB_PATH='$NODE2_DIR/db.redb' '$NODE_BIN' server --addr $NODE2_ADDR --dev-mode --seed 2 --xnode-node-id node://beta; read"

# Pane 0 (simulator) gets ~65% width, panes 1+2 (nodes) share the right
tmux select-layout -t "$SESSION:cluster" main-vertical
tmux select-pane -t "$SESSION:cluster.0"

tmux attach-session -t "$SESSION"
