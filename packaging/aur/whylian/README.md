# whylian (AUR)

Release-tagged PKGBUILD for [whyLIAN](https://github.com/byrdltd/whyLIAN).

Pins a GitHub release tag (e.g. `v1.0.4`). Submodule sources are fetched in
`prepare()` — GitHub release tarballs alone are not sufficient for this repo.

Published at <https://aur.archlinux.org/packages/whylian>.

For bleeding-edge `main`, use [`whylian-git`](whylian-git/).

## Install

```bash
yay -S whylian
```

## Publishing a new release

When tagging `vX.Y.Z` on GitHub:

1. Bump `pkgver` in `PKGBUILD`, reset `pkgrel=1`
2. `makepkg --printsrcinfo > .SRCINFO`
3. Push to `ssh://aur@aur.archlinux.org/whylian.git`
