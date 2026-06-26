# Contributing

See [CONTRIBUTING.md](https://github.com/cleeryy/jambon/blob/main/CONTRIBUTING.md) in the repository root.

## Quick Reference

```bash
# Install pre-commit hooks
pre-commit install --install-hooks

# Run all checks
cargo fmt --all -- --check
cargo clippy --workspace --all-features --tests -- -D warnings
cargo test --workspace --all-features

# Commit format
git commit -m "feat: add snapshot management commands"
```
