---
title: Code map
---

# Code map

The workspace has one crate, `oxidizer-firewall`, in `crates/firewall`. `src/main.rs` parses arguments and calls `run` in `src/lib.rs`, which does all the work. Keeping the logic in the library lets it be tested and reused.

| Module | Responsibility |
|---|---|
| `lib.rs` | `run`, the ordered provisioning steps, and step tracking for error messages |
| `main.rs` | Argument parsing, the panic hook and exit codes |
| `config.rs` | Ports, paths, the safety timer length and trusted addresses |
| `network.rs` | Host detection from the hostname and interface detection from the default route |
| `selector.rs` | The interactive country checklist and the list of countries it offers |
| `geoip.rs` | Writing and loading the `netdev filter` ingress table |
| `firewall.rs` | Flushing the ruleset and building the `inet filter` table |
| `session.rs` | Checking the operator's SSH session survived the reload |
| `countdown.rs` | The `at` safety flush and the countdown bar |
| `diagnostics.rs` | The summary and verification commands printed after a run |
| `logging.rs` | Coloured console output |

## API docs

Every public item has a doc comment. Build and open the Rust API docs locally with:

```bash
cargo doc -p oxidizer-firewall --no-deps --open
```
