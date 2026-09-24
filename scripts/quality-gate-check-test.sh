#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/quality-gate-check.sh"

[[ "$(compare_floats 79.11 79.12)" == "-1" ]]
[[ "$(compare_floats 79.12 79.12)" == "0" ]]
[[ "$(compare_floats 79.13 79.12)" == "1" ]]
