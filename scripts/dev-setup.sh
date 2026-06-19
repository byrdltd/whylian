#!/usr/bin/env bash
# Install build dependencies for whyLIAN development on Arch / CachyOS.
set -euo pipefail

echo "==> Installing whyLIAN build dependencies..."
sudo pacman -S --needed --noconfirm \
  base-devel rust cargo clang cmake pkg-config nasm \
  libusb ffmpeg fontconfig mesa libxkbcommon wayland libx11 libinput libdrm \
  libjpeg-turbo

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
git submodule update --init --recursive

echo ""
echo "==> Dev setup complete."
echo "    cargo build --release -p lianli-daemon -p lianli-gui"
echo "    ./scripts/install-dev.sh [--enable-linger]"
echo "    systemctl --user enable --now lianli-daemon"
