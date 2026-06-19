# Changelog

All notable changes to **whyLIAN** are documented here.

## [1.0.4] — 2026-06-19

First public release of **whyLIAN** — a HydroShift II AdvanceMode fork of
[lian-li-linux](https://github.com/sgtaziz/lian-li-linux).

### Added

- HydroShift II AdvanceMode radiator pairing and auto-provisioning
- Coolant fan curve, theme index 4, USB LCD + bundled `cooler` template
- Root `install.sh`, governance docs, GitHub CI, AUR packages (`whylian`, `whylian-git`)
- Strengthened [DISCLAIMER.md](DISCLAIMER.md) with install-time acceptance
- Dev workflow: `scripts/dev-setup.sh`, `scripts/install-dev.sh`
- TURZX detection: Lancool 207 (`0xACD1`) and Universal Screen (`0xACE1`) desktop PIDs

### Changed

- systemd user unit: `Restart=always`, headless-friendly
- Linger opt-in (`--enable-linger`); no longer auto-enabled by install scripts
- Wireless: shared controller handle, atomic GetDev, discovery gating
- AUR builds use Arch `rust`/`cargo` (no rustup toolchain file)

### Fixed

- Wireless USB reliability and stale TX handle after reconnect
- AIO controller: fan PWM continues when wireless theme switch retries
- Resume: wireless recovery before daemon restart
- AUR install: remove orphan templates from prior `./install.sh`

### Removed

- **evdi / desktop-display:** `lianli-evdi` crate, virtual-monitor support, udev
  `modprobe evdi` rules — HydroShift II AdvanceMode (fan/pump/LCD) is unchanged
