# BitCell Documentation

This directory contains the source for the BitCell documentation website, built with [mdBook](https://rust-lang.github.io/mdBook/).

## Building Locally

### Prerequisites

```bash
cargo install mdbook --version 0.4.37
```

### Build

```bash
cd docs/book
mdbook build
```

Output will be in `docs/book/book/`.

### Development Server

```bash
cd docs/book
mdbook serve --open
```

This will start a local server at `http://localhost:3000` with live reload.

## Structure

```
docs/book/
├── book.toml          # mdBook configuration
├── src/
│   ├── SUMMARY.md     # Table of contents
│   ├── introduction.md
│   ├── getting-started/
│   ├── node/
│   ├── wallet/
│   ├── contracts/
│   ├── api/
│   ├── concepts/
│   ├── advanced/
│   ├── development/
│   └── appendix/
└── book/              # Built output (gitignored)
```

## Contributing

To contribute to documentation:

1. Edit markdown files in `src/`
2. Test locally with `mdbook serve`
3. Submit a pull request

### Style Guide

- Use clear, concise language
- Include code examples where appropriate
- Add links to related sections
- Test all commands/code samples
- Use consistent formatting

## Deployment

Documentation is automatically built and deployed to GitHub Pages via `.github/workflows/deploy-docs.yml` when changes are pushed to master.

View live documentation at: https://docs.bitcell.network
