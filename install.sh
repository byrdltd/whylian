#!/usr/bin/env bash
# whyLIAN — install wrapper (see scripts/install-local.sh)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=============================================="
echo "whyLIAN Installation"
echo "Lian Li device control for Linux (HydroShift II)"
echo "=============================================="
echo ""
echo "Upstream base: lian-li-linux (MIT)"
echo "Fork focus: HydroShift II AdvanceMode on Linux"
echo ""
echo "DISCLAIMER: https://github.com/byrdltd/whyLIAN/blob/main/DISCLAIMER.md"
echo "You must accept before install (type I ACCEPT when prompted)."
echo ""

exec "$ROOT/scripts/install-local.sh" "$@"
