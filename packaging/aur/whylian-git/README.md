# whylian-git (AUR)

Upstream-tracking PKGBUILD for [whyLIAN](https://github.com/byrdltd/whyLIAN).

Canonical copy: `packaging/aur/whylian-git/` in the whyLIAN repository.
Published at <https://aur.archlinux.org/packages/whylian-git>.

## Install

```bash
yay -S whylian-git    # latest main (pkgver tracks tag + commits)
yay -S whylian        # pinned release tag (e.g. v1.0.4)
```

## Publishing a new revision

```bash
cd packaging/aur/whylian-git
makepkg --printsrcinfo > .SRCINFO

git clone ssh://aur@aur.archlinux.org/whylian-git.git /tmp/aur-whylian-git
cp PKGBUILD .SRCINFO whylian.install /tmp/aur-whylian-git/
cd /tmp/aur-whylian-git
git add PKGBUILD .SRCINFO whylian.install
git commit -m "Update to <pkgver>-<pkgrel>"
git push
```

Bump `pkgrel` when only packaging changes; `pkgver()` tracks `main` at build time.
