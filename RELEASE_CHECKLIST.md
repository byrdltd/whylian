# Release Readiness Checklist

Use this checklist before tagging a whyLIAN release.

## Pre-release

- [ ] `CHANGELOG.md` updated under `[Unreleased]` → new version section
- [ ] `ROADMAP.md` “Done” / “Next” reflects shipped scope
- [ ] `cargo fmt --all -- --check` passes locally
- [ ] `RUSTFLAGS="-D warnings" cargo test -p lianli-shared --lib` passes
- [ ] `RUSTFLAGS="-D warnings" cargo test -p lianli-devices --lib pairing` passes
- [ ] `RUSTFLAGS="-D warnings" cargo build --release -p lianli-daemon -p lianli-gui` passes
- [ ] CI green on `main` ([Actions](https://github.com/byrdltd/whyLIAN/actions)) — push to `main` or **Run workflow** on the CI workflow

## HydroShift II smoke test (when touching AIO / wireless / install)

- [ ] `lianli-daemon` discovers WaterBlock2 + TLV2 radiator cluster
- [ ] Fan PWM responds to coolant temperature (not stuck at max RPM)
- [ ] `wireless theme mode engaged` in journal after start
- [ ] USB LCD streams with bundled `cooler` template (if hardware present)
- [ ] `loginctl show-user $USER -p Linger` → `yes` on headless setups

## Tag and publish

```bash
git checkout main && git pull origin main
git tag -a vX.Y.Z -m "vX.Y.Z: short summary."
git push origin main   # if CHANGELOG commit pending
git push origin vX.Y.Z
```

- [ ] Tag matches `CHANGELOG` section (e.g. `v1.0.4` → `## [1.0.4]`)
- [ ] GitHub Release created (automatic on tag push via `.github/workflows/release.yml`, or manual `gh release create`)
- [ ] User-facing install paths surface [DISCLAIMER.md](DISCLAIMER.md) (README, install script, AUR hook)

## Post-release

- [ ] Verify [latest release](https://github.com/byrdltd/whyLIAN/releases/latest) page
- [ ] Bump `packaging/aur/whylian` (`pkgver` + `pkgrel=1`) and push to AUR
- [ ] If only `main` moved: bump `whylian-git` `pkgrel` and push to AUR
- [ ] Optional: `gh workflow run refresh-release-notes.yml -f tag=vX.Y.Z`
- [ ] Optional: upstream PR for generic pairing changes

## AUR quick reference

| Package | Install | Publish target |
|---------|---------|----------------|
| `whylian` | `yay -S whylian` | `ssh://aur@aur.archlinux.org/whylian.git` |
| `whylian-git` | `yay -S whylian-git` | `ssh://aur@aur.archlinux.org/whylian-git.git` |

Details: [packaging/aur/README.md](packaging/aur/README.md)

## Notes

- whyLIAN releases are **source-first** (`./install.sh`); no prebuilt Linux binaries are required for v1.
- Do not run L-Connect 3 and `lianli-daemon` simultaneously on the same RF dongle (dual-boot).
