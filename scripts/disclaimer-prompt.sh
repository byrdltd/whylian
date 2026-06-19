#!/usr/bin/env bash
# Sourced by install-local.sh — do not execute directly.
require_disclaimer_acceptance() {
  local root="${1:?}"
  local disclaimer="${root}/DISCLAIMER.md"

  if [[ "${WHYLIAN_SKIP_DISCLAIMER:-}" == "1" ]]; then
    echo "==> WHYLIAN_SKIP_DISCLAIMER=1 (automation/CI only)"
    return 0
  fi

  if [[ "${WHYLIAN_ACCEPT_DISCLAIMER:-}" == "1" ]]; then
    echo "==> WHYLIAN_ACCEPT_DISCLAIMER=1"
    return 0
  fi

  echo ""
  echo "=============================================="
  echo "DISCLAIMER — you must accept before installing"
  echo "=============================================="
  echo ""
  echo "whyLIAN controls fans, pumps, RGB, and LCD hardware via"
  echo "reverse-engineered protocols. It is NOT official Lian Li software."
  echo ""
  echo "YOU assume ALL risk of overheating, hardware damage, or data loss."
  echo "Authors, maintainers, and packagers accept NO liability."
  echo ""
  echo "Full text: ${disclaimer}"
  echo "Also: https://github.com/byrdltd/whyLIAN/blob/main/DISCLAIMER.md"
  echo ""
  read -r -p "Type I ACCEPT to continue: " _whylian_ans
  if [[ "${_whylian_ans}" != "I ACCEPT" ]]; then
    echo "Installation aborted — disclaimer not accepted."
    exit 1
  fi
  echo ""
}
