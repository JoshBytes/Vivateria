extends SceneTree
## Run: godot --headless --path client --script res://tests/test_protocol.gd

const Codec = preload("res://scripts/net/protocol.gd")
const MIN_I64: int = -9223372036854775807 - 1
const MAX_I64: int = 9223372036854775807
var failures: int = 0
var checks: int = 0


func _initialize() -> void:
	_test_client_boundaries()
	var fixtures: Dictionary = {}
	for name: String in ["client_hello", "server_welcome", "input", "snapshot", "delta"]:
		var path: String = ProjectSettings.globalize_path("res://../protocol/fixtures/" + name + ".bin")
		_check(FileAccess.file_exists(path), "fixture exists: " + name)
		var data: PackedByteArray = FileAccess.get_file_as_bytes(path) if FileAccess.file_exists(path) else PackedByteArray()
		fixtures[name] = data
	if failures > 0:
		_finish()
		return
	_test_fixtures(fixtures)
	_test_malformed(fixtures)
	_finish()


func _test_fixtures(fixtures: Dictionary) -> void:
	var entity: Dictionary = {
		"id": 2, "owner_id": 7,
		"pos_x": -4294967296, "pos_y": 2147483648,
		"vel_x": MIN_I64, "vel_y": MAX_I64, "energy": 429496729600,
	}
	var second_entity: Dictionary = {
		"id": 9, "owner_id": null,
		"pos_x": 9007199254740993, "pos_y": -9007199254740993,
		"vel_x": 0, "vel_y": 0, "energy": 0,
	}
	var delta_entity: Dictionary = entity.duplicate()
	delta_entity["pos_x"] = -4294967295
	var expected: Dictionary = {
		"client_hello": {"type": "client_hello", "protocol_version": 0, "player_name": "Viva 🌱"},
		# u64::MAX and 0xfedcba9876543210 retain all bits in signed Godot ints.
		"server_welcome": {"type": "server_welcome", "player_id": 7, "tick_hz": 20, "seed": -1, "tick": 42},
		"input": {"type": "input", "tick": 42, "move_x": -1, "move_y": 1},
		"snapshot": {"type": "snapshot", "tick": 42, "state_hash": -81985529216486896, "entities": [entity, second_entity]},
		"delta": {"type": "delta", "base_tick": 42, "tick": 43, "state_hash": 81985529216486895, "upserts": [delta_entity], "removed": [9]},
	}
	for name: String in expected:
		_check(Codec.decode(fixtures[name]) == expected[name], "fixture expected values: " + name)
	_check(Codec.decode_client_hello(fixtures["client_hello"]) == expected["client_hello"], "typed hello decoder")
	_check(Codec.decode_server_welcome(fixtures["server_welcome"]) == expected["server_welcome"], "typed welcome decoder")
	_check(Codec.decode_input(fixtures["input"]) == expected["input"], "typed input decoder")
	_check(Codec.decode_snapshot(fixtures["snapshot"]) == expected["snapshot"], "typed snapshot decoder")
	_check(Codec.decode_delta(fixtures["delta"]) == expected["delta"], "typed delta decoder")
	_check(Codec.decode_input(fixtures["client_hello"]) == null, "typed decoder rejects wrong message")
	_check(Codec.encode_client_hello(0, "Viva 🌱") == fixtures["client_hello"], "hello encode byte-identical to Rust")
	_check(Codec.encode_input(42, -1, 1) == fixtures["input"], "input encode byte-identical to Rust")


func _test_malformed(fixtures: Dictionary) -> void:
	for name: String in fixtures:
		var packet: PackedByteArray = fixtures[name]
		for length: int in range(packet.size()):
			_check(Codec.decode(packet.slice(0, length)) == null, "truncation: %s at %d" % [name, length])
		var trailing: PackedByteArray = packet.duplicate()
		trailing.append(0)
		_check(Codec.decode(trailing) == null, "trailing byte: " + name)
		for offset: int in range(6):
			var bad_header: PackedByteArray = packet.duplicate()
			bad_header[offset] = 255
			_check(Codec.decode(bad_header) == null, "bad header: %s at %d" % [name, offset])
	var bad_input: PackedByteArray = fixtures["input"].duplicate()
	bad_input[14] = 2
	_check(Codec.decode(bad_input) == null, "invalid movement")
	bad_input[14] = 254
	_check(Codec.decode(bad_input) == null, "invalid negative movement")
	var bad_name: PackedByteArray = fixtures["client_hello"].duplicate()
	bad_name.encode_u16(8, 65)
	_check(Codec.decode(bad_name) == null, "oversized name")
	var invalid_utf8: Array[PackedByteArray] = [
		PackedByteArray([0xc0, 0x80]), PackedByteArray([0xed, 0xa0, 0x80]),
		PackedByteArray([0xf4, 0x90, 0x80, 0x80]), PackedByteArray([0x80]),
		PackedByteArray([0xe2, 0x82]), PackedByteArray([0xe2, 0x28, 0xa1]),
	]
	for invalid_name: PackedByteArray in invalid_utf8:
		bad_name = fixtures["client_hello"].slice(0, 10)
		bad_name.encode_u16(8, invalid_name.size())
		bad_name.append_array(invalid_name)
		_check(Codec.decode(bad_name) == null, "invalid UTF-8: " + invalid_name.hex_encode())
	# Godot Strings cannot retain U+0000; reject it before text conversion.
	for invalid_name: PackedByteArray in [PackedByteArray([0]), PackedByteArray([97, 0, 98])]:
		bad_name = fixtures["client_hello"].slice(0, 10)
		bad_name.encode_u16(8, invalid_name.size())
		bad_name.append_array(invalid_name)
		_check(Codec.decode(bad_name) == null, "NUL name rejected without conversion")
	var bad_snapshot: PackedByteArray = fixtures["snapshot"].duplicate()
	bad_snapshot.encode_u32(22, 65537)
	_check(Codec.decode(bad_snapshot) == null, "oversized snapshot count")
	bad_snapshot.encode_u32(22, 0xffffffff)
	_check(Codec.decode(bad_snapshot) == null, "hostile snapshot count")
	bad_snapshot = fixtures["snapshot"].duplicate()
	bad_snapshot[30] = 2
	_check(Codec.decode(bad_snapshot) == null, "invalid owner tag")
	bad_snapshot = fixtures["snapshot"].duplicate()
	bad_snapshot[30] = 0
	_check(Codec.decode(bad_snapshot) == null, "nonzero owner with none tag")
	bad_snapshot = fixtures["snapshot"].duplicate()
	bad_snapshot.encode_u32(75, 2)
	_check(Codec.decode(bad_snapshot) == null, "duplicate entity IDs")
	bad_snapshot.encode_u32(75, 1)
	_check(Codec.decode(bad_snapshot) == null, "unsorted entity IDs")
	var bad_delta: PackedByteArray = fixtures["delta"].duplicate()
	bad_delta.encode_u32(83, 65537)
	_check(Codec.decode(bad_delta) == null, "oversized removed count")
	bad_delta = fixtures["delta"].duplicate()
	bad_delta.encode_u32(87, 2)
	_check(Codec.decode(bad_delta) == null, "upsert and removed overlap")
	bad_delta = fixtures["delta"].duplicate()
	bad_delta.encode_u32(83, 2)
	bad_delta.resize(95)
	bad_delta.encode_u32(91, 9)
	_check(Codec.decode(bad_delta) == null, "duplicate removed IDs")
	bad_delta.encode_u32(91, 1)
	_check(Codec.decode(bad_delta) == null, "unsorted removed IDs")
	var oversized: PackedByteArray = PackedByteArray()
	oversized.resize(Codec.MAX_FRAME_BYTES + 1)
	_check(Codec.decode(oversized) == null, "oversized frame")
	# A deterministic byte corpus exercises arbitrary packet contents without RNG.
	for length: int in range(96):
		var junk: PackedByteArray = PackedByteArray()
		junk.resize(length)
		for offset: int in range(length):
			junk[offset] = (length * 17 + offset * 31) & 0xff
		_check(Codec.decode(junk) == null, "arbitrary bytes: %d" % length)


func _test_client_boundaries() -> void:
	_check(Codec.encode_client_hello(-1, "name").is_empty(), "negative protocol version")
	_check(Codec.encode_client_hello(65536, "name").is_empty(), "oversized protocol version")
	_check(Codec.encode_client_hello(0, "a".repeat(65)).is_empty(), "65-byte name rejected")
	_check(Codec.encode_client_hello(0, "🌱".repeat(17)).is_empty(), "name limit counts UTF-8 bytes")
	for name: String in ["", "a".repeat(64), "🌱".repeat(16), "é水🌱", String.chr(0xfeff) + "name"]:
		_check(Codec.decode_client_hello(Codec.encode_client_hello(65535, name)) == {
			"type": "client_hello", "protocol_version": 65535, "player_name": name,
		}, "name round trip: %d bytes" % name.to_utf8_buffer().size())
	_check(Codec.encode_input(42, -2, 0).is_empty(), "invalid input x encoder")
	_check(Codec.encode_input(42, 0, 2).is_empty(), "invalid input y encoder")
	for tick: int in [0, MAX_I64, MIN_I64, -1]:
		for move_x: int in range(-1, 2):
			for move_y: int in range(-1, 2):
				_check(Codec.decode_input(Codec.encode_input(tick, move_x, move_y)) == {
					"type": "input", "tick": tick, "move_x": move_x, "move_y": move_y,
				}, "input boundary bit-pattern round trip")


func _check(condition: bool, description: String) -> void:
	checks += 1
	if not condition:
		failures += 1
		printerr("FAIL: " + description)


func _finish() -> void:
	if failures == 0:
		print("Protocol tests passed (%d checks)." % checks)
	else:
		printerr("Protocol tests failed: %d of %d checks." % [failures, checks])
	quit(0 if failures == 0 else 1)
