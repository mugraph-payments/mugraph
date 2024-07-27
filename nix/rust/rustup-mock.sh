#!/usr/bin/env bash

set -e

if ! which rustc &>/dev/null; then
	echo "[error]: Rust is not available on PATH."
	exit 1
fi

CMD="${1:-none}"

case "$CMD" in
toolchain)
	echo "risc0"
	;;
+risc0)
	which rustc
	;;
*)
	echo "[rustup-mock] $CMD: unknown command, ignoring."
	exit 0
	;;
esac
