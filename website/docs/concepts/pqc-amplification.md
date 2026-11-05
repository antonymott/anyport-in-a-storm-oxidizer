---
title: PQC handshake amplification
---

# PQC handshake amplification

Post-quantum signatures and key encapsulation are much larger than the classical algorithms they replace. ML-DSA and SLH-DSA signatures run to kilobytes and ML-KEM public keys and ciphertexts are around a kilobyte each, against tens of bytes for ECDSA or X25519. Every post-quantum handshake therefore moves more bytes and costs the server more work to parse and verify.

That changes the economics of a flood. A small packet from an attacker can make a server allocate state, read a large message and run an expensive verification. The later in the network stack a packet is dropped, the more of that cost the server has already paid.

## Why drop early

A packet that reaches the classic `PREROUTING` chain has already been through buffer allocation and will next get a conntrack entry and a routing lookup. A packet that reaches an application has also cost a socket and the start of a handshake.

`oxidizer` drops traffic from blocked address ranges at the netdev ingress hook instead. At that point the kernel has allocated the packet buffer and nothing else. No conntrack entry exists, no route has been looked up and no TLS or post-quantum code has run.

## What it does not do

- It does not inspect handshakes or recognise post-quantum traffic. It blocks by source address range.
- It does not drop packets before the kernel allocates the packet buffer. That needs native or zero-copy XDP, which most cloud instances cannot run. See [The ingress hook](./ingress-hook.md).
- It does not rate-limit per source. The rate limits in the main firewall are global per rule. See [Ruleset](../reference/ruleset.md).
