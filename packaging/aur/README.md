# AUR packaging (whylian)

Canonical PKGBUILDs live in this directory. Mirror changes to the AUR git repos over SSH.

| Package | Tracks | AUR |
|---------|--------|-----|
| [`whylian`](whylian/) | GitHub release tag (`v1.0.4`, …) | https://aur.archlinux.org/packages/whylian |
| [`whylian-git`](whylian-git/) | `main` branch | https://aur.archlinux.org/packages/whylian-git |

```bash
yay -S whylian        # stable release
yay -S whylian-git    # bleeding edge
```

The packages **conflict** — install one or the other.

## Why stable uses `git#tag=`, not a tarball

GitHub archive tarballs do not include `vendor/` submodules (HDiffPatch, tinyuz). Both PKGBUILDs clone git and run `git submodule update --init --recursive` in `prepare()`.

**AUR builds use Arch `rust`/`cargo`**, not rustup. `rust-toolchain.toml` is renamed during `makepkg` so `yay` does not require `rustup default stable`.

## Publishing

**Install on Arch:** always `yay -S whylian` (or `paru`).

### New GitHub release → bump `whylian`

```bash
cd packaging/aur/whylian
# edit pkgver=, reset pkgrel=1
makepkg --printsrcinfo > .SRCINFO
git clone ssh://aur@aur.archlinux.org/whylian.git /tmp/aur-whylian
cp PKGBUILD .SRCINFO whylian.install /tmp/aur-whylian/
cd /tmp/aur-whylian && git add -A && git commit -m "whylian X.Y.Z-1" && git push
```

### Packaging-only change on `main` → bump `whylian-git` pkgrel

```bash
cd packaging/aur/whylian-git
makepkg -od   # refreshes pkgver() in PKGBUILD
makepkg --printsrcinfo > .SRCINFO
# push to ssh://aur@aur.archlinux.org/whylian-git.git
```

See [RELEASE_CHECKLIST.md](../RELEASE_CHECKLIST.md) for the full release flow.
