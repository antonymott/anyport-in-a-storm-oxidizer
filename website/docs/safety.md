---
title: Safety
---

# Safety

`fw` runs as root and replaces every firewall rule on the host. Treat each run as if it were on a remote server with no console access.

## How it can fail

- **Fail open.** The host is left with fewer rules than intended, or none. This happens briefly during every run, between the flush and the new rules loading. It also happens after a test-mode countdown runs out, after an SSH rollback and when a run fails part way through.
- **Fail closed.** The new rules block your own access. Over SSH with no console this can lock you out of the host for good, unless the safety timer is armed.

## The safety timer

In test mode `fw` queues `nft flush ruleset` with `at` before it changes anything. If you lose access, wait and the flush restores it within one minute. Pressing a key during the countdown cancels the job and keeps the rules.

The timer is the only protection against a lockout in a remote session. Do not shorten it, remove it or work around it without explicit sign-off from a maintainer. Any change to `countdown.rs` is safety-critical.

Production mode has no timer. Run it only with a ruleset you have already confirmed in test mode on the same host.

## The SSH check

After the new rules load in test mode, `fw` checks that your current SSH session still appears in `ss` as established. If it does not, it flushes the ruleset and removes every pending `at` job at once.

This only proves that an existing session survived. It cannot prove that a new SSH connection will get through. Keep a second session open and test a fresh login before pressing a key.

## Checklist before a run

1. Run in test mode first.
2. Make sure `atd` is running (`systemctl status atd`).
3. Keep a second SSH session open.
4. Know how to reach the host's console in case both sessions drop.
5. After pressing a key, open a new SSH connection to confirm you can still log in.
6. Save the ruleset to `/etc/nftables.conf` only once you are sure.

## Rules for automated agents

`AGENTS.md` in the repository sets the rules for AI coding agents. In short:

1. Never run `fw`, `nft` or any rule-applying command against a live or remote host. Only `cargo build`, `check`, `test` and `clippy` are allowed.
2. Never remove, shorten or bypass the safety timer in `countdown.rs` without human sign-off.
3. Never make the firewall more permissive silently. Say so in the pull request.
