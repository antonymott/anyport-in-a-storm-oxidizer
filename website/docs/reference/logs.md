---
title: Logs
---

# Logs

Dropped and rejected traffic is logged by the kernel with a fixed prefix. Each logging rule is rate-limited so a flood cannot fill the journal.

| Prefix | Table | Logged when | Limit |
|---|---|---|---|
| `GEOBLOCK-DROP-V4: ` | `netdev filter` | IPv4 from a blocked country is dropped | 5 per minute |
| `GEOBLOCK-DROP-V6: ` | `netdev filter` | IPv6 from a blocked country is dropped | 5 per minute |
| `REDIS-LOOPBACK-BLOCK: ` | `inet filter` | A user other than `app-runner` tries Redis on loopback | none |
| `USER-BLOCK-APP-RUNNER: ` | `inet filter` | `app-runner` tries a new connection other than Redis | 5 per minute |
| `USER-BLOCK-GIT-DEPLOY: ` | `inet filter` | `git-deploy` tries any new connection | 5 per minute |

The GeoIP drop chains also keep a packet counter, so `nft list table netdev filter` shows the full count even when the log is limited.

## Reading the logs

```bash
journalctl -fk | grep -E 'GEOBLOCK-DROP|USER-BLOCK'
journalctl --since '1 hour ago' -k | grep -E 'GEOBLOCK|USER' | wc -l
```

The summary at the end of each run also prints a script that lists the top 10 blocked IPv4 addresses with their country.
