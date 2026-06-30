# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/cleeryy/jambon/compare/jambon-proxmox-api-v0.1.0...jambon-proxmox-api-v0.1.1) - 2026-06-30

### Fixed

- dashboard cluster-status parsing, error handling in interactions, richer /menu embed
- *(audit)* TaskResponse deserialize, LxcShutdownOptions, missing endpoints
- *(proxmox-api)* send valid JSON body and remove default Content-Type
- *(proxmox-api)* unwrap JSON {"data": ...} envelope in handle_response

### Other

- cargo fmt --all
