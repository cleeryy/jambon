# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/cleeryy/jambon/releases/tag/jambon-bot-core-v0.1.0) - 2026-06-27

### Added

- implement scheduling and automation (Tier 4) ([#50](https://github.com/cleeryy/jambon/pull/50))
- implement audit log for destructive operations ([#13](https://github.com/cleeryy/jambon/pull/13)) ([#46](https://github.com/cleeryy/jambon/pull/46))
- implement health monitor with Discord alerts ([#10](https://github.com/cleeryy/jambon/pull/10)) ([#43](https://github.com/cleeryy/jambon/pull/43))
- *(p1)* implement P1 priority features ([#42](https://github.com/cleeryy/jambon/pull/42))
- scaffold jambon project with full workspace, CI/CD, and documentation

### Fixed

- resolve CI compilation, fmt, clippy, and doc errors

### Other

- residual fixes from CI pass (imports, error types, re-exports)
