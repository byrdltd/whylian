#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck source=disclaimer-prompt.sh
source "${ROOT}/scripts/disclaimer-prompt.sh"
require_disclaimer_acceptance "$ROOT"

USER_INSTALL=false
ENABLE_LINGER=false

usage() {
  cat <<EOF
Usage: $0 [--user] [--enable-linger]

  --user              Install to ~/.local/bin (recommended for development)
  --enable-linger     Enable loginctl linger (headless fan control; opt-in)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --user) USER_INSTALL=true; shift ;;
    --enable-linger) ENABLE_LINGER=true; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

echo "==> Building release binaries..."
export CARGO_PROFILE_RELEASE_STRIP=symbols
export RUSTFLAGS="-D warnings"
cargo build --release -p lianli-daemon -p lianli-gui

maybe_enable_linger() {
  if ! $ENABLE_LINGER; then
    echo "==> Skipping loginctl enable-linger (pass --enable-linger for headless fan control)."
    return 0
  fi
  if loginctl show-user "$(id -un)" -p Linger --value 2>/dev/null | grep -qx no; then
    echo "==> Enabling systemd linger for $(id -un)..."
    sudo loginctl enable-linger "$(id -un)"
  fi
}

if $USER_INSTALL; then
  DEST="${HOME}/.local/bin"
  echo "==> Installing to $DEST (no sudo)..."
  mkdir -p "$DEST"
  install -Dm755 target/release/lianli-daemon "$DEST/lianli-daemon"
  install -Dm755 target/release/lianli-gui "$DEST/lianli-gui"

  TEMPLATE_DEST="${XDG_DATA_HOME:-$HOME/.local/share}/whylian/templates"
  echo "==> Installing LCD templates to $TEMPLATE_DEST..."
  mkdir -p "$TEMPLATE_DEST"
  cp -a "$ROOT/templates/." "$TEMPLATE_DEST/"

  DROPIN_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/lianli-daemon.service.d"
  mkdir -p "$DROPIN_DIR"
  cat > "$DROPIN_DIR/override.conf" <<EOF
[Service]
ExecStart=
ExecStart=${DEST}/lianli-daemon
EOF
  echo "==> Systemd override written: $DROPIN_DIR/override.conf"
  echo "    Ensure base unit exists (run scripts/install-dev.sh once, or install AUR package)."

  maybe_enable_linger
else
  echo "==> Installing to /usr/bin (requires sudo)..."
  sudo install -Dm755 target/release/lianli-daemon /usr/bin/lianli-daemon
  sudo install -Dm755 target/release/lianli-gui /usr/bin/lianli-gui

  if [[ -f packaging/udev/99-lianli.rules ]]; then
    echo "==> Installing udev rules..."
    sudo install -Dm644 packaging/udev/99-lianli.rules /etc/udev/rules.d/99-lianli.rules
    sudo udevadm control --reload-rules
    sudo udevadm trigger
  fi

  if [[ -f packaging/systemd/lianli-daemon.service ]]; then
    echo "==> Installing systemd user unit..."
    sudo install -Dm644 packaging/systemd/lianli-daemon.service \
      /usr/lib/systemd/user/lianli-daemon.service
  fi

  echo "==> Installing LCD templates to /usr/share/whylian/templates..."
  sudo install -dm755 /usr/share/whylian/templates
  sudo cp -a "$ROOT/templates/." /usr/share/whylian/templates/

  maybe_enable_linger

  DROPIN="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/lianli-daemon.service.d/override.conf"
  if [[ -f "$DROPIN" ]]; then
    echo "==> Removing systemd override: $DROPIN"
    rm -f "$DROPIN"
    rmdir "${DROPIN%/*}" 2>/dev/null || true
  fi
fi

systemctl --user daemon-reload 2>/dev/null || true
if systemctl --user is-active lianli-daemon &>/dev/null; then
  echo "==> Restarting lianli-daemon..."
  systemctl --user restart lianli-daemon
fi

echo "==> Done."
ls -la "${DEST:-/usr/bin/lianli-daemon}" "${DEST:-/usr/bin}/lianli-gui" 2>/dev/null || ls -la /usr/bin/lianli-daemon /usr/bin/lianli-gui
