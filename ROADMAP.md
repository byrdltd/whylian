# whyLIAN Roadmap

## Done (v1.0.4)

- [x] WaterBlock2 ↔ TLV2 radiator auto-pairing and PWM routing
- [x] Coolant temperature fan/pump curve defaults
- [x] AdvanceMode wireless theme index 4
- [x] USB HydroShift II LCD auto-config + local template install
- [x] LCD config binds `sensor_source_2` to wireless coolant when AIO known
- [x] Systemd linger + resume wireless recovery
- [x] Removed evdi / desktop-display virtual monitor support (HydroShift II path needs no evdi)
- [x] Dev workflow: `dev-setup.sh`, `install-dev.sh`
- [x] Arch packaging skeleton (`whylian-git`)
- [x] GitHub CI (fmt, test, `-D warnings` build; manual dispatch supported)
- [x] Release checklist + tag-triggered release workflow
- [x] AUR packages `whylian` (release) and `whylian-git` (main) published

## Next

- [ ] Upstream PR for generic `pairing.rs` (if accepted by lian-li-linux)
- [ ] GUI: surface coolant sensor in AIO page by default
- [ ] Cooler template widgets: native `wireless_coolant` sources (requires template sha256 bump)

## Research required

- [ ] LCD surround ring RGB over USB (Windows `lcd207.dll` / WinUSB capture)
- [ ] `switchAioLcdWirelessMode` full parity with L-Connect AdvanceMode init
- [ ] Sleep/resume PWM dropout — long-run soak testing
- [ ] Firmware update protocol (low priority; Windows-only today)

## Non-goals

- Reimplementing all of L-Connect 3
- Supporting motherboard PWM headers for HydroShift fans (hardware uses RF only)
- Virtual-monitor / evdi desktop-display mode on Linux
