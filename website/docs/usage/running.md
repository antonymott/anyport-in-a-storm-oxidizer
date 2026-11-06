---
title: Running fw
---

# Running fw

`fw` has two modes. Always use test mode first on a host, and over a connection you can afford to lose for a minute.

## Test mode

```bash
sudo ./target/release/fw --test
```

`-t` and `flushinoneminute` are accepted as aliases for `--test`.

1. Before anything else, `fw` queues `nft flush ruleset` with `at` to run in one minute.
2. It builds the new ruleset as described in the [Introduction](../intro.md).
3. It reads `SSH_CONNECTION` and uses `ss` to check that your own SSH session is still established. If it is not, it flushes the ruleset at once, removes every pending `at` job and exits with status 1. On a console session with no `SSH_CONNECTION` this check is skipped.
4. It prints the summary and starts a countdown bar.
5. **Press any key** before the bar reaches 100% to cancel the `at` job and keep the new rules.
6. **Do nothing** and the `at` job flushes the ruleset at 100%. The host is then running with **no firewall at all** until you run `fw` again.

If you lose access during the countdown, wait. The flush restores access within the minute.

The SSH check only proves that your existing session survived. It cannot prove that new connections will be accepted. Open a second SSH session before pressing a key.

## Production mode

```bash
sudo ./target/release/fw
```

Production mode applies the rules with no safety timer, no SSH check and no countdown. Use it only with a ruleset you have already confirmed in test mode.

## Turning off GeoIP blocking

```bash
sudo ./target/release/fw --test --geoipblock=false
```

This skips the country checklist and the ingress table and builds only the main `inet` firewall. `--geoipblock` accepts `true`, `false`, `1`, `0`, `yes`, `no`, `on` and `off`.

## Keeping the rules after a reboot

`fw` changes the running ruleset only. To keep it across reboots, save it once you are happy with it:

```bash
sudo nft list ruleset > /etc/nftables.conf
sudo systemctl enable nftables
sudo systemctl restart nftables
```

## Exit status

| Status | Meaning |
|---|---|
| `0` | The ruleset was applied |
| `1` | A planned stop (unknown host, lost SSH session and rollback), a bad argument or a failure part way through |

On a failure `fw` prints the step it was on, for example `FIREWALL SCRIPT CRASHED AT STEP: 'Assembling Layer-3 Inet Ruleset'`. A failure after the flush can leave the host with a partial ruleset. In test mode the safety timer still reverts it.

## Verifying the result

The summary prints these commands. Run the `nmap` scans from another machine.

```bash
nft list ruleset
nft list table netdev filter
nft list table inet filter

nmap -sS -p 21,22,23,80,443,3306,8080,3000,3001,9010,7909 <server-ip>
nmap -sU -p 53,123,3000,3001 <server-ip>
```

Only 80, 443 and the SSH port 7909 should show open, plus 9010 on radon.
