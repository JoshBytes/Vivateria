# Vivateria Protocol v0

This document is the normative specification for protocol version 0. The protocol is a
hand-rolled binary format carried in WebSocket **binary** frames. Each WebSocket frame contains
exactly one complete protocol message; messages must not be split across frames or combined.

## Frame

| Offset | Type | Value |
|---:|---|---|
| 0 | 4 bytes | ASCII magic `VIVA` (`56 49 56 41`) |
| 4 | `u16` | Protocol version, exactly `0` |
| 6 | `u8` | Message type |
| 7 | bytes | Message payload, as specified below |

All multi-byte integers are little-endian. Offsets in the message tables are relative to the
start of the payload and fields appear without padding.

## Primitive encodings

| Type | Encoding |
|---|---|
| `u8`, `i8` | One unsigned or two's-complement signed byte |
| `u16`, `u32`, `u64`, `i64` | Little-endian integer of the stated width; signed values use two's complement |
| `bool` | Reserved for future protocol versions; when introduced, a `u8`: `0` is false and `1` is true; no other value is valid |
| `String` | A `u16` byte length followed by exactly that many UTF-8 bytes; length is at most 256 bytes |
| `Option<T>` | A `u8` tag: `0` means no value and has no following bytes; `1` means one encoded `T` follows |
| `List<T>` | A `u16` item count followed by exactly that many encoded `T` values |
| `Fx` | An `i64` containing the raw Q32.32 value; its exact mathematical value is `raw / 2^32` |
| `Vec2` | `x: Fx`, then `y: Fx` |

Lengths and counts describe bytes or items, respectively, not Unicode scalar values. Encoders
must not emit a string longer than 256 UTF-8 bytes or a list longer than 65,535 items.
Protocol v0 has no `bool` fields.

## Client to server

### `0x01` — `Hello`

| Field | Type | Meaning |
|---|---|---|
| `client_name` | `String` | Client display name |

### `0x02` — `Input`

| Field | Type | Meaning |
|---|---|---|
| `tick` | `u64` | Simulation tick to which the input applies |
| `move_x` | `i8` | Horizontal movement, exactly one of `-1`, `0`, `1` |
| `move_y` | `i8` | Vertical movement, exactly one of `-1`, `0`, `1` |

## Server to client

### `0x81` — `Welcome`

| Field | Type | Meaning |
|---|---|---|
| `player_id` | `u32` | Assigned player identifier |
| `entity_id` | `u32` | Entity controlled by that player |
| `tick_hz` | `u8` | Authoritative simulation ticks per second |
| `server_tick` | `u64` | Current authoritative tick |

### `0x82` — `Snapshot`

| Field | Type | Meaning |
|---|---|---|
| `tick` | `u64` | Tick represented by the state |
| `entities` | `List<EntityState>` | Complete entity state |

### `0x83` — `Delta`

| Field | Type | Meaning |
|---|---|---|
| `base_tick` | `u64` | Tick of the state to which this delta applies |
| `tick` | `u64` | Tick produced after applying this delta |
| `changed` | `List<EntityState>` | Added or changed entities |
| `removed` | `List<u32>` | IDs of removed entities |

### `0x84` — `Disconnect`

| Field | Type | Meaning |
|---|---|---|
| `reason` | `String` | Human-readable reason |

## Compound type: `EntityState`

| Field | Type | Meaning |
|---|---|---|
| `id` | `u32` | Entity identifier |
| `owner` | `Option<u32>` | Owning player ID, or none |
| `pos` | `Vec2` | Authoritative position |
| `vel` | `Vec2` | Authoritative velocity |
| `energy` | `Fx` | Authoritative energy |

## Decoding requirements

A decoder must reject the entire frame if any of the following is true: the magic is not
`VIVA`; the version is not `0`; the message type is unknown for the receiving direction; any
field is truncated; any bytes remain after the specified message; an option tag is not `0` or `1`;
either input axis is outside `-1..=1`; a string is invalid UTF-8; or a string's byte length exceeds
256. A decoder must reject a string length greater than 256 before reading the string bytes.
Decoders must validate lengths and counts against the bytes remaining before allocating or
iterating, and must fail safely rather than panic on arbitrary input.

Protocol v0 contains no human-only actions. Stage advancement, opening the editor or evolving,
and forming or breaking alliances must never be exposed through the Agent API.
