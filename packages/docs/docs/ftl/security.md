---
sidebar_position: 4
---

# Security

:::caution Needs Work

This section is not well developed yet.

:::

When working with FTL, incoming media is processed through an open socket anyone can access this means anyone could:
- Send their own Video / Audio streams.
- Interfere with the stream.

You should ensure when making an FTL server implementation that you prevent potential attacks on the stream.

### Port Randomisation

- Difficulty: Easy
- Security: Low

This is a pretty simple way to delay attackers but it won't stop potential inteference if an attacker is persistent enough.

When spinning up new UDP sockets for new incoming streams, generate a new random UDP port for each new client, when doing this make sure to use a large port range such as `24000` to `65000`.

### Dynamic IP Whitelist

- Difficulty: Easy
- Security: Medium

As clients connect to the ingest control server save their peer address. Then when a new UDP socket is opened for them, only allow incoming packets from the same peer address.

This may cause issues if the remote client is roaming, but this protocol doesn't support roaming anyways.
