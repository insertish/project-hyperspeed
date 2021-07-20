---
sidebar_position: 1
---

# FTL Protocol

- Version: `0.0.1`
- License: **MIT**
- Crates.io: [crates.io/ftl-protocol](https://crates.io/ftl-protocol)
- Rust Documentation: [docs.rs/ftl-protocol](https://docs.rs/ftl-protocol)

[ftl-protocol](https://gitlab.insrt.uk/insert/project-hyperspeed/-/tree/master/crates/ftl) provides common data structures for working with FTL as well as an optional FTL ingest control server.

### Parsing FTL commands

Once you've isolated incoming FTL commands, you can parse them like so:

```rust
use crate::protocol::FtlCommand;

let command = FtlCommand::from_str(".").unwrap();
assert_eq!(command, FtlCommand::Dot);

let command = FtlCommand::from_str("PING 123").unwrap();
assert_eq!(command, FtlCommand::Ping {
    channel_id: "123".to_string()
});
```

### Creating FTL responses

You can create FTL responses by converting them to a string, every response will automatically be suffixed with `\n`.

```rust
use crate::protocol::FtlResponse;

let resp = FtlResponse::Success;
assert_eq!(resp.to_string(), "200\n".to_string());
```

### Building and verifying FTL handshake

You can quickly construct and verify an incoming FTL handshake.

```rust
use crate::protocol::{FtlCommand, FtlHandshake};

// Start constructing handshake somewhere in your code.
let mut handshake = FtlHandshake::default();

// Example incoming command.
let command = FtlCommand::Attribute {
    key: "ProtocolVersion".to_string(),
    value: "0.9".to_string()
};

// Match attribute and insert it into handshake.
if let FtlCommand::Attribute { key, value } = command {
    handshake.insert(key, value).unwrap();
    // You should handle any errors here,
    // but we know this isn't going to fail.
}

// Once we have the minimum amount of information,
// (see the note under FTL handshakes on the protocol page)
// we can finalise the handshake, this verifies all data is
// correct, such as the protocol version and ensuring if A/V
// streams are enabled that they have all fields present.
let handshake = handshake.finalise().unwrap();
assert_eq!(handshake.protocol_version.1, 9);
```
