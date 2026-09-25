#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/quality-gate-check.sh"

[[ "$(compare_floats 79.11 79.12)" == "-1" ]]
[[ "$(compare_floats 79.12 79.12)" == "0" ]]
[[ "$(compare_floats 79.13 79.12)" == "1" ]]

doc_output=$(cat <<'EOF'
warning: card_game@0.1.0: cargo:warning=tiled-to-shapes: no TSX files
{"reason":"compiler-message","target":{"doc":true},"message":{"level":"warning"}}
EOF
)
[[ "$(printf '%s\n' "$doc_output" | count_rustdoc_warnings)" == "1" ]]
