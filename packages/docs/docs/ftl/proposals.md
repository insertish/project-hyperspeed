---
sidebar_position: 5
---

# Proposals

Below are areas of the FTL protocol that could be improved and a discussion into how they could be solved.

:::caution

These are all the author's opinion and should not be taken as part of the FTL protocol specification.

:::

### Semver for Protocol Version

- Backwards compatible.
- Affects server code.

Instead of the `{major}.{minor}` format for the protocol version, we could instead adhere to semver, this is available in most languages [including Rust](https://docs.rs/semver/1.0.3/semver/).

Clients do not need to change much meanwhile servers can start parsing existing version strings as semvers without breaking backwards compatibility.

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Semver checker](https://jubianchi.github.io/semver-check/#/)

### Signed UDP Packets

- Backwards compatible.
- Affects client + server code.

For security, we could consider SRTP but as all streams are public anyways this is probably easier to implement. To ensure that UDP packets are coming from who we think is sending them we could include a small hash in each packet. For example, we could establish a secret key when negotiating with the ingest control server, this key is then used to hash the `sequence_number` and `timestamp` or any other combination of values in the header, the resulting hash can be stored in the `extensions` part of the header, see [Header::extensions](https://docs.rs/rtp/0.3.0/rtp/header/struct.Header.html#structfield.extensions).

The only downside to this approach is that we need to modify existing client implementations to support this, but it could be implemented as an optional switch, for example:

```yaml
Client: ProtocolVersion: 0.1-secure\n
     C: SignedPackets: true\n
     C: [..]
```

This would be backwards compatible while allowing new clients to increase their transmission security.

### Roaming

- Backwards compatible.
- Affects client + server code.

The FTL protocol could be modified to not automatically clean up after disconnect, instead allow clients to roam between networks and continue sending data.

Ideally, there would be a timeout of about 10 seconds to resume and clients could connect with proof:

```yaml
Client: HMAC\n
Server: HMAC {..}\n
Client: RESUME {Channel ID} {Hashed Key}\n
Server: 200\n
```

Similarly to signed packets, we may want to add a flag that this is enabled:

```yaml
Client: ProtocolVersion: 0.1-roaming\n
     C: RoamingEnabled: true\n
     C: [..]
```

This is just so that old protocol version clients stop streaming immediately when the connection drops.

### Feature Flags

- Backwards compatible.
- Affects client + server code.

Building on top of [Signed UDP Packets](#signed-udp-packets) and [Roaming](#roaming), we could instead have a single attribute for specifying additional protocol features which the client supports.

```yaml
Client: ProtocolVersion: 0.1-features\n
     C: Features: SignedPackets Roaming\n
     C: [..]
```

Features are separated by a single space.
