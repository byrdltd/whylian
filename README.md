<p align="center">
  <img src="assets/icons/icon.svg" width="128" height="128" alt="whyLIAN">
</p>

<h1 align="center">whyLIAN</h1>

<p align="center">
  <i>Lian Li device control for Linux — HydroShift II AdvanceMode fork of</i>
  <a href="https://github.com/sgtaziz/lian-li-linux">lian-li-linux</a>
</p>

<p align="center">
  <a href="https://github.com/byrdltd/whyLIAN">GitHub</a> ·
  <a href="PRODUCT_CONTEXT.md">Product context</a> ·
  <a href="ROADMAP.md">Roadmap</a> ·
  <a href="CHANGELOG.md">Changelog</a>
</p>

<p align="center">
  <strong>Current release:</strong> <a href="https://github.com/byrdltd/whyLIAN/releases/tag/v1.0.4">v1.0.4</a>
  · AUR: <code>whylian</code> / <code>whylian-git</code> (Arch package names are lowercase)
</p>

## About

**whyLIAN** keeps HydroShift II LCD-S/C **AdvanceMode** setups working on Linux
without L-Connect 3: radiator fan PWM, pump control, wireless RGB, USB LCD streaming,
and coolant-based fan curves — with sensible defaults on first discovery.

Fork of [lian-li-linux](https://github.com/sgtaziz/lian-li-linux) (MIT). Upstream
credit and submodule licenses: [NOTICE](NOTICE).

> ### Read this before installing
>
> whyLIAN controls **fans, pumps, and cooling hardware** via reverse-engineered
> protocols. It is **not** official Lian Li software. **You** are responsible for
> temperatures, hardware safety, and legal compliance — not the authors or
> packagers.
>
> By installing or running whyLIAN you accept **[DISCLAIMER.md](DISCLAIMER.md)**
> in full, including **no warranty** and **no liability** for hardware damage,
> overheating, or data loss.
>
> Summary: keep the daemon running on headless systems (`loginctl enable-linger`),
> monitor coolant/CPU temps after config changes, never run L-Connect 3 and
> whyLIAN on the same RF dongle at once.

## Documentation

| Doc | Purpose |
|-----|---------|
| [PRODUCT_CONTEXT.md](PRODUCT_CONTEXT.md) | Problem, architecture, scope |
| [ROADMAP.md](ROADMAP.md) | Done / planned / research items |
| [CHANGELOG.md](CHANGELOG.md) | Fork release notes |
| [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) | Pre-tag verification steps |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Dev setup, PR guidelines |
| [DISCLAIMER.md](DISCLAIMER.md) | **Required reading** — liability, cooling risk, acceptance |
| [SECURITY.md](SECURITY.md) | Vulnerability reporting |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Community standards |

## Fork highlights

| Feature | whyLIAN | Upstream default |
|---------|---------|------------------|
| TLV2 radiator pairing (WaterBlock2 `fan_count=0`) | Auto | Manual TLV2 fan group |
| Coolant fan/pump curve | Auto `Coolant` curve | Constant ~50% PWM |
| AdvanceMode wireless theme | Index `4` | `0` |
| HydroShift II USB LCD | Auto + `cooler` template | Manual `lcds` entry |
| Resume | Wireless recover + restart | Restart only |

Implementation: `crates/lianli-devices/src/wireless/pairing.rs`, `crates/lianli-shared/src/hydroshift.rs`.

**Not implemented** (Windows USB RE required): LCD surround ring RGB, USB-mode fan/pump,
firmware updates. Wireless pump-head RGB (24 LEDs) works via the existing RGB controller.

## Install (Arch / CachyOS)

**Development (recommended):**

```bash
git clone --recursive https://github.com/byrdltd/whyLIAN.git
cd whyLIAN
./scripts/dev-setup.sh
./scripts/install-dev.sh --enable-linger   # ~/.local/bin + udev + user systemd unit
systemctl --user enable --now lianli-daemon
```

**Production:**

```bash
yay -S whylian
```

**AUR:** `yay -S whylian` (release tag) or `yay -S whylian-git` (main branch) — see [packaging/aur/](packaging/aur/).

Dependencies: `libusb`, `ffmpeg`, and libraries listed in the PKGBUILD.
Replaces `lianli-linux`, `lianli-linux-git`, and `whylian-git` if previously installed.

---

## Supported Devices

### HID

| Device | Fan Control | RGB | LCD | Pump | Tested |
|--------|:-----------:|:---:|:---:|:----:|:------:|
| UNI FAN SL / AL / SL Infinity / SL V2 / AL V2 | 4 groups | Yes | - | - | Yes |
| UNI FAN TL Controller | 4 ports | Yes | - | - | Yes |
| UNI FAN TL LCD | 4 ports | Yes | 400x400 | - | Yes |
| Galahad II Trinity AIO | Yes | Yes | - | Yes | Yes |
| HydroShift LCD AIO | Yes | Yes | 480x480 | Yes | Yes |
| Galahad II LCD / Vision AIO | Yes | Yes | 480x480 | Yes | Yes |

\* Galahad II LCD / Vision uses the same driver as HydroShift LCD AIO.

### Wireless (via TX/RX dongle)

| Device | Fan Control | RGB | LCD | Pump | Tested |
|--------|:-----------:|:---:|:---:|:----:|:------:|
| UNI FAN TL V2 (LCD / LED) | Yes | Yes | 400x400 | - | Yes |
| UNI FAN SL V3 (LCD / LED) | Yes | Yes | 400x400 | - | Yes |
| UNI FAN SL-INF | Yes | Yes | - | - | Yes |
| UNI FAN CL / RL120 | Yes | Yes | - | - | - |
| HydroShift II LCD-C (Wireless) | Yes | Yes | - | Yes | Yes |
| HydroShift II LCD-S (Wireless) | Yes | Yes | - | Yes | Yes |
| Strimer Plus Wireless | - | Yes | - | - | Yes |
| Lancool 217 Wireless | - | Yes | - | - | - |
| Lancool V150 Wireless | Yes | Yes | - | - | - |
| Universal Screen 8.8" Wireless | - | Yes | - | - | - |

Both V1 (VID 0x0416) and V2 (VID 0x1A86) wireless dongles are supported. Binding devices is supported through the GUI.

> **HydroShift II AdvanceMode** (WaterBlock2 + external TLV2 radiator cluster) is tested on one workstation configuration. Other wireless AIO / radiator combinations are not guaranteed.

> **Note:** Wireless devices with LCDs still need to be plugged in via USB to control the LCD. LCD cannot be controlled through wireless dongle alone.

### USB (Standalone LCD)

| Device | LCD | Tested | Notes |
|--------|:---:|:------:|-------|
| HydroShift II LCD Circle | 480x480 | Yes | |
| HydroShift II LCD Square | 480x480 | Yes | |
| Lancool 207 Digital | 1472x720 | Yes | |
| Universal Screen 8.8" | 1920x480 | Yes | |
| Universal Screen 8.8" LED Ring | - | Yes | RGB control supported |

Devices stuck in desktop/display mode are detected and can be switched back to LCD mode via the GUI.

### Display mode switching

HydroShift II, Lancool 207 Digital, and Universal Screen 8.8" can run in **LCD mode**
(custom content via the daemon) or **desktop/display mode** (Windows-only native monitor).
On Linux, whyLIAN detects devices stuck in display mode and switches them back to LCD mode
via the GUI. **Virtual-monitor desktop mode is not supported** — there is no evdi dependency
and no `modprobe evdi` in packaging.

### Other

| Device | RGB | Tested |
|--------|:---:|:------:|
| Strimer Plus (wired) | Yes | - |

If you've tested a device that isn't marked as tested above, please [open an issue or PR](https://github.com/byrdltd/whyLIAN/issues) to update this table.

## Architecture

```
lianli-daemon          User service - fan control loop + LCD streaming
  lianli-devices       HID/USB device drivers
  lianli-transport     USB bulk transport (wireless protocol, display streaming)
  lianli-media         Image/video/GIF encoding, sensor gauge rendering
  lianli-shared        IPC types, config schema, device IDs

lianli-gui             Slint desktop app - connects to daemon via Unix socket
```

The daemon runs as a user systemd service. USB access is granted via udev rules (no root required).
The GUI connects over `$XDG_RUNTIME_DIR/lianli-daemon.sock`.

## Installing

### Arch Linux (AUR)

```bash
yay -S whylian
```

Or with any AUR helper (`paru`, `trizen`, etc.). Installs binaries, udev rules, systemd user service, desktop entry, and icons. Globally enables `lianli-daemon.service` for new users.

For headless fan control: `sudo loginctl enable-linger $USER`.

### From Source

1) Clone the repo and submodules:
```bash
git clone --recurse-submodules https://github.com/byrdltd/whyLIAN.git && cd whyLIAN
```
> If you already cloned without `--recurse-submodules`, run: `git submodule update --init --recursive`

2) Install dependencies:
- **Rust** (stable, 1.75+)
- **ffmpeg** and **ffprobe** in `PATH` (for video/GIF decoding)
- **System libraries:**

```bash
# Arch
sudo pacman -S hidapi libusb ffmpeg fontconfig mesa libxkbcommon wayland libx11 libinput libdrm clang cmake pkg-config nasm rust
```

3) Build:
```bash
cargo build --release
```

Binaries: `target/release/lianli-daemon` and `target/release/lianli-gui`

4) Install udev rules (required for USB access without root):
```bash
sudo cp packaging/udev/99-lianli.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

5) Install and start the daemon:

```bash
./scripts/install-dev.sh --enable-linger
systemctl --user enable --now lianli-daemon
```

Or use `./install.sh` for a system-wide install to `/usr/bin`.

A default config is created automatically at `~/.config/lianli/config.json` on first run.

6) Install desktop entry and icons:
```bash
# Install icons
for size in 32x32 128x128 256x256 scalable; do mkdir -p ~/.local/share/icons/hicolor/$size/apps; done
cp assets/icons/32x32.png ~/.local/share/icons/hicolor/32x32/apps/com.sgtaziz.lianlilinux.png
cp assets/icons/128x128.png ~/.local/share/icons/hicolor/128x128/apps/com.sgtaziz.lianlilinux.png
cp assets/icons/128x128@2x.png ~/.local/share/icons/hicolor/256x256/apps/com.sgtaziz.lianlilinux.png
cp assets/icons/icon.svg ~/.local/share/icons/hicolor/scalable/apps/com.sgtaziz.lianlilinux.svg

# Install desktop entry
cp packaging/desktop/com.sgtaziz.lianlilinux.desktop ~/.local/share/applications/
update-desktop-database ~/.local/share/applications/
```

### With Docker

```bash
./docker/build.sh
```

The script builds the image (matching your UID/GID), mounts a cargo cache, and runs `cargo build --release`. Artifacts land in `target/release/` on the host. Then follow steps 4-6 from "From Source" above.

## Configuration

The daemon reads `~/.config/lianli/config.json`. The GUI edits this file via the daemon's IPC socket.

### LCD Streaming

Each LCD entry specifies a target device (by serial), media type, and orientation:

| Type | Description |
|------|-------------|
| `image` | Static image (JPEG, PNG, BMP, GIF) |
| `video` | Video file (decoded frame-by-frame via ffmpeg) |
| `gif` | Animated GIF |
| `color` | Solid RGB color |
| `sensor` | Live sensor gauge (CPU temp, GPU temp, etc.) |

### Fan Curves

Fan curves map a temperature source (any shell command) to a speed percentage.
Points are linearly interpolated; temperatures outside the curve range clamp to the nearest point's speed.

### Fan Speed Modes

| Mode | Description |
|------|-------------|
| `0` | Off (0% PWM) |
| `"curve-name"` | Follow a named fan curve |
| `1-255` | Constant PWM duty (1=0.4%, 128=50%, 255=100%) |
| `"__mb_sync__"` | Mirror motherboard PWM signal (hardware passthrough) |

## Troubleshooting

**Daemon won't start / no devices found:**
```bash
# Check udev rules are loaded
sudo udevadm test /sys/bus/usb/devices/<your-device>

# Check daemon logs
journalctl --user -u lianli-daemon -f
```

**GUI says "Daemon offline":**
```bash
# Verify daemon is running
systemctl --user status lianli-daemon

# Check socket exists
ls -la $XDG_RUNTIME_DIR/lianli-daemon.sock
```

**Permission denied on USB device:**
```bash
# Re-trigger udev after plugging in device
sudo udevadm trigger
```

## License

MIT. See [LICENSE](LICENSE).

This project is not affiliated with Lian Li Industrial Co., Ltd.
Protocol information was obtained through reverse engineering for interoperability purposes.
