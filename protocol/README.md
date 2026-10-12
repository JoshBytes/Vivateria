# protocol

The [v0 wire contract](v0.md) defines the local human client protocol for M1.
The Rust codec lives in `server/crates/protocol`; the Godot codec lives in
`client/scripts/net/protocol.gd`. Shared binary golden frames are in
[`fixtures/`](fixtures/README.md).

This task provides schemas and codecs. WebSocket transport is M1-05; the
perception-limited Agent API is a later, separate protocol.
