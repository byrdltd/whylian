# Contributing to whyLIAN

Thanks for your interest in contributing.

## Code of Conduct

Please follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Scope

whyLIAN is a **fork** of [lian-li-linux](https://github.com/sgtaziz/lian-li-linux). Contributions welcome for:

- HydroShift II / AdvanceMode correctness and provisioning
- Bug fixes that benefit fork users without breaking upstream parity
- Documentation, packaging (Arch/AUR), and CI
- Tests for pairing, defaults, and config migration

Generic device support that applies upstream should eventually go to **sgtaziz/lian-li-linux** first; we can cherry-pick or rebase.

## Development setup

```bash
git clone --recursive git@github.com:byrdltd/whyLIAN.git
cd whyLIAN
./scripts/dev-setup.sh
cargo build --release -p lianli-daemon -p lianli-gui
./scripts/install-dev.sh --enable-linger
systemctl --user enable --now lianli-daemon
journalctl --user -u lianli-daemon -f
```

## Testing

```bash
cargo test -p lianli-shared --lib
cargo test -p lianli-devices --lib pairing
cargo fmt --all -- --check
RUSTFLAGS="-D warnings" cargo build --release -p lianli-daemon -p lianli-gui
```

CI runs the same checks on push to `main` (see `.github/workflows/ci.yml`). You can also trigger it manually from GitHub Actions.

Hardware tests need Lian Li RF dongle + HydroShift II; document results in PRs.

## Pull request guidelines

- Keep PRs focused; one logical change per PR when possible
- Update `CHANGELOG.md` for user-visible fork changes
- Match existing Rust style and upstream patterns
- Explain HydroShift / L-Connect behaviour when touching wireless or AIO code

## Security

Do not open public issues for vulnerabilities. See [SECURITY.md](SECURITY.md).

## License

By contributing, you agree your contributions are licensed under [MIT](LICENSE).
Contributors should not expect the maintainers to assume liability for
downstream use of the project; see [DISCLAIMER.md](DISCLAIMER.md).
