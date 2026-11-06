---
title: Contributing
---

# Contributing

## Check, lint, test

```bash
cargo check -p oxidizer-firewall
cargo clippy -p oxidizer-firewall --all-targets -- -D warnings
cargo fmt
cargo test -p oxidizer-firewall
```

Unit tests cover the parts that do not need root, such as reading the interface from `ip route` output and reading the job id from `at` output. Nothing in the test suite calls `nft`.

## Pull requests

- Never test against a live or remote host from an automated environment. See [Safety](./safety.md).
- Say clearly in the description if a change makes the firewall more permissive, such as a new open port, a removed block or a disabled hook.
- Treat any change to `countdown.rs` as safety-critical and ask for human review.

## These docs

The site lives in `website/` and is built with Docusaurus.

```bash
cd website
pnpm install
pnpm start
```

`pnpm build` writes the static site to `website/build`. Pushes to `main` that touch `website/` deploy it to GitHub Pages.
