# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/cleeryy/jambon/compare/jambon-bot-commands-v0.1.0...jambon-bot-commands-v0.1.1) - 2026-06-30

### Added

- autocomplete for commands + interactive /menu ([#57](https://github.com/cleeryy/jambon/pull/57))
- extract public embed/component builders into interactions.rs ([#56](https://github.com/cleeryy/jambon/pull/56))
- add node_utils module with try_for_each_node helper

### Fixed

- dashboard cluster-status parsing, error handling in interactions, richer /menu embed
- *(storage)* use try_for_each_node to skip unreachable nodes

### Other

- cargo fmt --all
