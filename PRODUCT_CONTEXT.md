# whyLIAN Product Context

## Mission

whyLIAN is a maintained Linux fork of [lian-li-linux](https://github.com/sgtaziz/lian-li-linux)
focused on making **HydroShift II LCD-S/C AdvanceMode** setups work out of the box
without Windows L-Connect 3 — fan PWM, pump control, wireless RGB, and USB LCD streaming.

## Problem statement

HydroShift II AdvanceMode splits responsibilities across two wireless records on the
same RF dongle:

| Device | Role | `fan_count` in discovery |
|--------|------|--------------------------|
| WaterBlock2 (AIO head) | Pump, block RGB, LCD theme | `0` |
| Tlv2Led (radiator cluster) | 3× radiator fans + fan RGB | `3` |

Upstream `lian-li-linux` treated the AIO as having no fans and sent `[0,0,0,0]` PWM to
WaterBlock2. Windows L-Connect routes radiator PWM to the TLV2 MAC instead. Without that
routing, the AIO fail-safe drives fans to maximum RPM.

## what whyLIAN adds

1. **Radiator pairing** — auto-link WaterBlock2 ↔ TLV2 on the same master dongle
2. **PWM routing** — AIO fan slots target the paired TLV2 cluster
3. **AdvanceMode defaults** — wireless theme index `4`, coolant-based fan/pump curve
4. **USB LCD provisioning** — auto-config for `1cbe:a034` with bundled `cooler` template
5. **Ops hardening** — systemd linger, resume wireless recovery, packaged templates

## Non-negotiable principles

1. **Upstream respect** — credit lian-li-linux; contribute generic fixes upstream when possible
2. **No telemetry** — same as upstream: local config, no phoning home
3. **Fail-safe awareness** — daemon must stay running (linger) or fans revert to full speed
4. **Honest scope** — document what requires Windows protocol RE (USB ring RGB, firmware)

## Out of scope (v1)

- LCD surround ring RGB over USB (`lcd207.dll` protocol)
- USB-mode fan/pump PWM (RF path is sufficient for AdvanceMode)
- Firmware update tooling
- Full L-Connect GUI parity
- Virtual-monitor / evdi desktop-display on Linux

See [ROADMAP.md](ROADMAP.md) for planned work.
