# Jambon Documentation

This directory contains the Jambon documentation using mdBook.

## Building

```bash
# Install mdBook
cargo install mdbook

# Build the docs
cd docs
mdbook build

# Serve with hot-reload for development
mdbook serve --open
```

## Structure

```
docs/
├── book.toml        # mdBook configuration
├── src/
│   ├── SUMMARY.md   # Table of contents
│   ├── introduction.md
│   ├── getting-started.md
│   ├── configuration.md
│   ├── architecture.md
│   ├── commands.md
│   ├── admin/
│   │   └── guide.md       # Admin setup guide
│   ├── user/
│   │   └── commands.md    # User command reference
│   └── developer/
│       └── contributing.md # Development guide
└── README.md
```

## Sections

- **Admin Guide** — Discord app setup, Proxmox API token creation, environment
  configuration, Docker and Kubernetes deployment.
- **User Guide** — Complete command reference with examples.
- **Developer Guide** — Architecture overview, development setup, feature flags,
  testing, PR workflow, and release process.
