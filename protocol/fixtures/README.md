# Golden protocol v0 frames

These `.bin` files contain complete messages, including the six-byte header.
They are shared by Rust and Godot tests and deliberately include values beyond
float integer precision, unsigned high bits and signed i64 extrema.

| File | Sample fields |
|---|---|
| client_hello.bin | protocol_version=0; player_name=`Viva 🌱` (9 UTF-8 bytes) |
| server_welcome.bin | player_id=7; tick_hz=20; seed=18446744073709551615; tick=42 |
| input.bin | tick=42; move_x=-1; move_y=1 |
| snapshot.bin | tick=42; state_hash=0xfedcba9876543210; entities 2 and 9 below |
| delta.bin | base_tick=42; tick=43; state_hash=0x0123456789abcdef; upsert entity 2 with pos_x=-4294967295; remove 9 |

| Entity field | ID 2 | ID 9 |
|---|---|---|
| owner_id | 7 | absent (tag 0, wire owner 0) |
| pos_x | -4294967296 | 9007199254740993 |
| pos_y | 2147483648 | -9007199254740993 |
| vel_x | -9223372036854775808 | 0 |
| vel_y | 9223372036854775807 | 0 |
| energy | 429496729600 | 0 |

From `server/`, run `cargo test -p vivateria-protocol` to verify every fixture
against independent literal golden bytes and the Rust encoder/decoder.
To intentionally regenerate the binaries in PowerShell:

```powershell
$env:UPDATE_PROTOCOL_FIXTURES = '1'
cargo test -p vivateria-protocol golden_fixtures_and_roundtrips
Remove-Item Env:UPDATE_PROTOCOL_FIXTURES
```

The regeneration test asserts encoder equality with literal golden bytes before
writing, then reads each file back and asserts byte equality again. Changes to
the contract require reviewing the literals and fixtures together.
