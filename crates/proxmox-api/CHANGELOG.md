# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/cleeryy/jambon/releases/tag/jambon-proxmox-api-v0.1.0) - 2026-06-27

### Added

- production readiness — tests, Docker, Helm, Prometheus, i18n, docs (Tier 6) ([#51](https://github.com/cleeryy/jambon/pull/51))
- advanced resources — LXC, pools, ACL, firewall, QEMU agent (Tier 5) ([#49](https://github.com/cleeryy/jambon/pull/49))
- VM lifecycle commands — create, delete, resize, snapshot, clone ([#48](https://github.com/cleeryy/jambon/pull/48))
- implement backup commands ([#12](https://github.com/cleeryy/jambon/pull/12)) ([#45](https://github.com/cleeryy/jambon/pull/45))
- implement backup commands ([#12](https://github.com/cleeryy/jambon/pull/12))
- implement storage commands ([#11](https://github.com/cleeryy/jambon/pull/11)) ([#44](https://github.com/cleeryy/jambon/pull/44))
- scaffold jambon project with full workspace, CI/CD, and documentation

### Fixed

- resolve CI compilation, fmt, clippy, and doc errors
- resolve CI compilation and lint failures

### Other

- Revert "feat: implement backup commands ([#12](https://github.com/cleeryy/jambon/pull/12))"
- residual fixes from CI pass (imports, error types, re-exports)
