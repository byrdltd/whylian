# whyLIAN vX.Y.Z

**Release date:** YYYY-MM-DD

## Highlights

- One or two bullets for the most important fork-specific changes.

## Install (Arch / CachyOS)

**Production (recommended):**

```bash
yay -S whylian
systemctl --user enable --now lianli-daemon
sudo loginctl enable-linger $USER   # headless / no GUI session
```

**From source:**

```bash
git clone --recursive https://github.com/byrdltd/whyLIAN.git
cd whyLIAN
./scripts/dev-setup.sh
./scripts/install-dev.sh --enable-linger
systemctl --user enable --now lianli-daemon
```

User-local install (no `/usr/bin`): `./install.sh --user`

## Upgrade notes

- Restart daemon after upgrade: `systemctl --user restart lianli-daemon`
- Any config migration steps, if applicable.

## Security / hardware notes

- Read [DISCLAIMER.md](../DISCLAIMER.md) — daemon must stay running or fans may revert to fail-safe max RPM.

## Upstream

Fork of [lian-li-linux](https://github.com/sgtaziz/lian-li-linux) (MIT). See [NOTICE](../NOTICE).
