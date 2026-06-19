# whyLIAN — Disclaimer and Acceptable-Use Statement

> **In plain language:** whyLIAN controls fans, pumps, RGB, and LCD hardware on
> Linux using **reverse-engineered** protocols. It is not made or endorsed by
> Lian Li. **You** — not the authors, maintainers, or packagers — are
> responsible for your hardware, cooling, and legal compliance. Read this
> before installing or running the daemon.

whyLIAN is a fork of [lian-li-linux](https://github.com/sgtaziz/lian-li-linux)
with HydroShift II AdvanceMode improvements. It is **experimental systems
software**, not a certified cooling or safety product.

---

## 1. Scope of the project

whyLIAN is intended for:

- Personal workstations where you own the Lian Li hardware
- Running fan/pump/RGB/LCD control on Linux without L-Connect 3
- Self-hosted, local-only device control (no cloud dependency)

whyLIAN is **not** intended for, and the authors do not condone using it to:

- Operate cooling on hardware you do not own or lack permission to configure
- Disable or bypass manufacturer safety limits in ways that risk overheating
- Run alongside L-Connect 3 on the **same** RF dongle (dual-boot conflict risk)
- Represent the tool as official Lian Li software or L-Connect parity where
  features are explicitly unimplemented (USB ring RGB, firmware update, etc.)
- Redistribute commercially without complying with the [MIT license](LICENSE)

---

## 2. Hardware and cooling risks

Incorrect fan or pump control can cause **overheating**, **thermal throttling**,
**component damage**, or **data loss**. You acknowledge that:

- Protocols were reverse-engineered; behaviour may differ from Windows L-Connect
- Misconfiguration, bugs, USB/RF errors, or daemon crashes can affect cooling
- Many devices enter a **fail-safe maximum fan speed** when the daemon stops —
  monitor temperatures after any change
- On headless systems you should enable `loginctl enable-linger` so the daemon
  keeps running without a graphical login

**You** are solely responsible for monitoring CPU/GPU/coolant temperatures and
ensuring adequate cooling at all times.

---

## 3. Legal responsibility is yours

By downloading, building, installing, packaging, or running whyLIAN you agree
that **you are the operator** and the **legally responsible party**.

The authors, maintainers, contributors, distributors, and AUR packagers of
whyLIAN:

- Make **no warranty** of merchantability, fitness for a particular purpose,
  non-infringement, or error-free operation
- Do **not** guarantee compatibility with any specific Lian Li product,
  firmware revision, or Linux distribution
- Accept **no liability** for any direct, indirect, incidental, special,
  consequential, or punitive damages, including but not limited to:
  - Hardware damage or premature wear
  - Lost data or system downtime
  - Fire, electrical, or property damage arising from inadequate cooling
  - Fines, regulatory action, or third-party claims related to your use

The full MIT warranty disclaimer is in [LICENSE](LICENSE). This document
supplements — and does not replace — that license.

---

## 4. Indemnification

To the fullest extent permitted by applicable law, you agree to **indemnify,
defend, and hold harmless** the authors, maintainers, contributors,
distributors, and packagers of whyLIAN from any claim, demand, loss, liability,
or expense (including reasonable attorneys' fees) arising out of or related to:

- Your installation, configuration, or use of whyLIAN
- Your failure to maintain adequate cooling or monitor hardware temperatures
- Your redistribution or modification of the software
- Any dispute between you and Lian Li or other third parties

---

## 5. Installation and packaging

- Running `./install.sh` or `./install.sh --user` requires explicit acceptance
  (type `I ACCEPT`) unless `WHYLIAN_ACCEPT_DISCLAIMER=1` is set for automation
- `WHYLIAN_SKIP_DISCLAIMER=1` is reserved for CI — **not** for skipping consent
  on a personal machine
- AUR/binary packages install [DISCLAIMER.md](DISCLAIMER.md) under
  `/usr/share/doc/whylian/`; installing the package constitutes acceptance
- Preserve `LICENSE`, `NOTICE`, and this file in any redistribution

---

## 6. Upstream and trademarks

Core protocol work builds on the [lian-li-linux](https://github.com/sgtaziz/lian-li-linux)
community's reverse-engineering of L-Connect behaviour. Fork-specific changes
are documented in [PRODUCT_CONTEXT.md](PRODUCT_CONTEXT.md).

**Lian Li**, **L-Connect**, and related names are trademarks of their respective
owners. whyLIAN is an independent project and is **not affiliated with,
sponsored by, or endorsed by** Lian Li.

---

## 7. Acceptance

By using whyLIAN you confirm that:

1. You have read and understood this disclaimer
2. You accept all hardware, cooling, and legal risks of running reverse-engineered
   device control software
3. You will not hold the project authors, maintainers, or packagers liable for
   damage or loss arising from your use
4. You agree to the indemnification terms in section 4

If you cannot agree, **do not install or run** whyLIAN.

---

*Questions? Open an issue at <https://github.com/byrdltd/whyLIAN/issues>.*
