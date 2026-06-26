# Contributing to Jambon

Thank you for your interest in contributing! This document provides guidelines to help you get started.

## Getting Started

1. **Fork** the repository on GitHub.
2. **Clone** your fork: `git clone git@github.com:your-username/jambon.git`
3. **Install prerequisites**: Rust 1.81+, pre-commit.
4. **Install pre-commit hooks**: `pre-commit install --install-hooks`
5. **Set up environment**: Copy `.env.example` to `.env` and fill in your credentials.
6. **Build**: `cargo build`

## Development Workflow

1. Create a feature branch: `git checkout -b feat/my-feature`
2. Make your changes.
3. Ensure all checks pass:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-features --tests -- -D warnings
   cargo test --workspace --all-features
   ```
4. Commit using [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat: add support for VM snapshots`
   - `fix: handle timeout during VM shutdown`
   - `docs: update command examples`
   - `ci: speed up cargo caching`
5. Push and open a Pull Request.

## Code Style

- Follow `rustfmt` (120 columns, 4-space tabs).
- Use `clippy` with workspace lints (no warnings).
- Use `thiserror` for error types, not `anyhow` in library crates.
- Document all public API items with doc comments (`///`).
- Keep functions focused and small — extract helpers as needed.

## Project Structure

```
crates/
├── proxmox-api/    # Proxmox VE REST API client (no Discord dependency)
├── bot-core/       # Framework integration, config, error handling
├── bot-commands/   # Slash command handlers
└── bot-bin/        # Binary entrypoint
```

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/) to automate changelog generation and semantic versioning via `release-plz`.

| Prefix   | Usage                              |
|----------|------------------------------------|
| `feat:`  | A new feature                      |
| `fix:`   | A bug fix                          |
| `docs:`  | Documentation changes              |
| `refactor:` | Code refactoring               |
| `test:`  | Adding or updating tests           |
| `ci:`    | CI/CD changes                      |
| `chore:` | Maintenance tasks (deps, config)   |

## Pull Request Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-features --tests -- -D warnings` passes
- [ ] `cargo test --workspace --all-features` passes
- [ ] New code includes tests
- [ ] Documentation updated if applicable

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
