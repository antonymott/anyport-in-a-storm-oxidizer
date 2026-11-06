---
title: The ingress hook
---

# The ingress hook

`oxidizer` binds its first table to the nftables `netdev` family with an `ingress` hook on the host's interface:

```text
type filter hook ingress device "eth0" priority 0; policy accept;
```

This is a netfilter feature (`NF_NETDEV_INGRESS`). It is not eBPF and it is not XDP.

## Packet path

Where each filtering point sits, from the wire inwards:

1. NIC hardware DMA into the ring buffer
2. **Tier 0**, `XDP_ZEROCOPY` / AF_XDP: before the packet buffer exists, straight to user space
3. **Tier 1**, `XDP_DRV` native XDP: before the packet buffer exists, inside the driver
4. The kernel allocates the packet buffer (`sk_buff`)
5. **Tier 2**, `XDP_SKB` generic XDP: software fallback after allocation
6. **`oxidizer`**: nftables netdev ingress hook, the same early window as step 5
7. Classic `PREROUTING` and conntrack, where most firewalls filter today
8. Routing lookup
9. Local delivery to a socket

## Timing and availability

- **Timing:** the ingress hook runs at Tier 2 timing. The packet buffer already exists but conntrack, routing and the application have not been touched.
- **Availability:** the hook needs no driver, hypervisor or NIC cooperation. It works the same on bare metal and on the most restrictive cloud instances, where XDP is not available at any level.

## Cloud availability of XDP

This table is about eBPF/XDP hooks, not `oxidizer`. It shows why an ingress-hook tool is useful: most instances cannot run the faster XDP modes at all. Figures are estimates.

| Tier | Mechanism | Est. share | Typical hosts |
|---|---|---|---|
| Tier 3 | No eBPF/XDP hooks | ~75% | AWS EC2 and Lightsail, GCP, Azure, Linode and other shared VPS |
| Tier 2 | `XDP_SKB` generic mode | ~24.5% | Hetzner Cloud, DigitalOcean, Vultr, Scaleway, OVHcloud VPS |
| Tier 1 | `XDP_DRV` native mode | ~0.49% | Dedicated bare metal with an XDP-capable driver |
| Tier 0 | `XDP_ZEROCOPY` | &lt; 0.01% | Custom bare metal with enterprise NICs such as Intel ICE or Mellanox ConnectX-5+ |

The full table with instance counts is in the [README](https://github.com/antonymott/anyport-in-a-storm-oxidizer#global-cloud-instances-by-ebpfxdp-reflex-layer-accessibility).
