# Protocol v0 golden test vectors

Each `.hex` file contains one complete WebSocket binary frame encoded as lowercase hexadecimal,
with no spaces. Fixed-point values below give both their mathematical value and raw Q32.32 value.

- `hello.hex`: `Hello { client_name: "Vivatéria" }` (the name is 10 UTF-8 bytes).
- `input.hex`: `Input { tick: 42, move_x: -1, move_y: 1 }`.
- `welcome.hex`: `Welcome { player_id: 7, entity_id: 99, tick_hz: 20, server_tick: 1234 }`.
- `snapshot.hex`: `Snapshot { tick: 50 }` with two entities:
  - `EntityState { id: 99, owner: Some(7), pos: (-1 [raw -4294967296], 0.5 [raw 2147483648]), vel: (0.25 [raw 1073741824], 0 [raw 0]), energy: 5 [raw 21474836480] }`.
  - `EntityState { id: 100, owner: None, pos: (2 [raw 8589934592], -3 [raw -12884901888]), vel: (0 [raw 0], -1 [raw -4294967296]), energy: 1 [raw 4294967296] }`.
- `delta.hex`: `Delta { base_tick: 50, tick: 51 }`, changing `EntityState { id: 99, owner: Some(7), pos: (-0.5 [raw -2147483648], 1 [raw 4294967296]), vel: (0 [raw 0], 0.5 [raw 2147483648]), energy: 4 [raw 17179869184] }`, and removing entity IDs `100` and `200`.
- `disconnect.hex`: `Disconnect { reason: "server shutdown" }`.
