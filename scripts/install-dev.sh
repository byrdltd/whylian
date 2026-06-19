#!/usr/bin/env bash
# Development install: ~/.local/bin + udev rules + user systemd unit.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"$ROOT/scripts/install-local.sh" --user "$@"

echo "==> Installing udev rules (requires sudo for USB access)..."
sudo install -Dm644 "$ROOT/packaging/udev/99-lianli.rules" /etc/udev/rules.d/99-lianli.rules
sudo udevadm control --reload-rules
sudo udevadm trigger

USER_UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
mkdir -p "$USER_UNIT_DIR"
sed "s|ExecStart=/usr/bin/lianli-daemon|ExecStart=${HOME}/.local/bin/lianli-daemon|" \
  "$ROOT/packaging/systemd/lianli-daemon.service" > "$USER_UNIT_DIR/lianli-daemon.service"
echo "==> User systemd unit: $USER_UNIT_DIR/lianli-daemon.service"

# Drop-in override is redundant when unit ExecStart already points at ~/.local/bin
DROPIN="${USER_UNIT_DIR}/lianli-daemon.service.d/override.conf"
if [[ -f "$DROPIN" ]]; then
  rm -f "$DROPIN"
  rmdir "${USER_UNIT_DIR}/lianli-daemon.service.d" 2>/dev/null || true
fi

systemctl --user daemon-reload
echo ""
echo "==> Dev install complete."
echo "    systemctl --user enable --now lianli-daemon"
echo "    journalctl --user -u lianli-daemon -f"
