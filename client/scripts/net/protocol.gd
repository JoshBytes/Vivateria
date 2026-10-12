class_name Protocol
extends RefCounted
## Protocol v0. Fixed-point values stay raw; u64 values use int's exact bit pattern.
## Decoders return null for malformed packets. Invalid encoders return empty bytes.

const WIRE_VERSION: int = 0
const MAX_NAME_BYTES: int = 64
const MAX_ENTITIES: int = 65536
const MAX_FRAME_BYTES: int = 4 * 1024 * 1024
const ENTITY_BYTES: int = 49

enum MessageType { CLIENT_HELLO = 1, SERVER_WELCOME, INPUT, SNAPSHOT, DELTA }


class Reader:
	extends RefCounted
	var data: PackedByteArray
	var offset: int = 6
	var valid: bool = true

	func _init(packet: PackedByteArray) -> void:
		data = packet

	func available(length: int) -> bool:
		if length < 0 or length > data.size() - offset:
			valid = false
		return valid

	func u8() -> int:
		if not available(1):
			return 0
		var value: int = data[offset]
		offset += 1
		return value

	func u16() -> int:
		if not available(2):
			return 0
		var value: int = data.decode_u16(offset)
		offset += 2
		return value

	func u32() -> int:
		if not available(4):
			return 0
		var value: int = data.decode_u32(offset)
		offset += 4
		return value

	func u64() -> int:
		if not available(8):
			return 0
		var value: int = data.decode_u64(offset)
		offset += 8
		return value

	func i64() -> int:
		if not available(8):
			return 0
		var value: int = data.decode_s64(offset)
		offset += 8
		return value


static func decode(packet: PackedByteArray) -> Variant:
	if packet.size() < 6 or packet.size() > MAX_FRAME_BYTES:
		return null
	if packet[0] != 0x56 or packet[1] != 0x49 or packet[2] != 0x56 or packet[3] != 0x50:
		return null
	if packet[4] != WIRE_VERSION:
		return null
	var reader: Reader = Reader.new(packet)
	var message: Variant = null
	match packet[5]:
		MessageType.CLIENT_HELLO:
			message = _read_client_hello(reader)
		MessageType.SERVER_WELCOME:
			message = {
				"type": "server_welcome", "player_id": reader.u32(),
				"tick_hz": reader.u32(), "seed": reader.u64(), "tick": reader.u64(),
			}
		MessageType.INPUT:
			var tick: int = reader.u64()
			var move_x: int = reader.u8()
			var move_y: int = reader.u8()
			move_x = move_x if move_x < 128 else move_x - 256
			move_y = move_y if move_y < 128 else move_y - 256
			if move_x < -1 or move_x > 1 or move_y < -1 or move_y > 1:
				return null
			message = {"type": "input", "tick": tick, "move_x": move_x, "move_y": move_y}
		MessageType.SNAPSHOT:
			var tick: int = reader.u64()
			var state_hash: int = reader.u64()
			var entities: Variant = _read_entities(reader)
			if entities == null:
				return null
			message = {"type": "snapshot", "tick": tick, "state_hash": state_hash, "entities": entities}
		MessageType.DELTA:
			message = _read_delta(reader)
	if not reader.valid or reader.offset != packet.size():
		return null
	return message


static func decode_client_hello(packet: PackedByteArray) -> Variant:
	return _decode_type(packet, MessageType.CLIENT_HELLO)


static func decode_server_welcome(packet: PackedByteArray) -> Variant:
	return _decode_type(packet, MessageType.SERVER_WELCOME)


static func decode_input(packet: PackedByteArray) -> Variant:
	return _decode_type(packet, MessageType.INPUT)


static func decode_snapshot(packet: PackedByteArray) -> Variant:
	return _decode_type(packet, MessageType.SNAPSHOT)


static func decode_delta(packet: PackedByteArray) -> Variant:
	return _decode_type(packet, MessageType.DELTA)


static func _decode_type(packet: PackedByteArray, message_type: int) -> Variant:
	if packet.size() < 6 or packet[5] != message_type:
		return null
	return decode(packet)


static func _read_client_hello(reader: Reader) -> Variant:
	var protocol_version: int = reader.u16()
	var name_length: int = reader.u16()
	if name_length > MAX_NAME_BYTES or not reader.available(name_length):
		return null
	var name_bytes: PackedByteArray = reader.data.slice(reader.offset, reader.offset + name_length)
	if name_bytes.has(0) or not _valid_utf8(name_bytes):
		return null
	reader.offset += name_length
	# Prefix prevents Godot's text decoder from stripping a leading U+FEFF.
	var prefixed: PackedByteArray = PackedByteArray([0x20])
	prefixed.append_array(name_bytes)
	return {"type": "client_hello", "protocol_version": protocol_version, "player_name": prefixed.get_string_from_utf8().substr(1)}


static func _read_entities(reader: Reader) -> Variant:
	var count: int = reader.u32()
	if count > MAX_ENTITIES or not reader.available(count * ENTITY_BYTES):
		return null
	var entities: Array[Dictionary] = []
	var previous_id: int = -1
	for index: int in range(count):
		var entity_id: int = reader.u32()
		var owner_tag: int = reader.u8()
		var owner: int = reader.u32()
		if entity_id <= previous_id or owner_tag > 1 or (owner_tag == 0 and owner != 0):
			return null
		previous_id = entity_id
		entities.append({
			"id": entity_id, "owner_id": owner if owner_tag == 1 else null,
			"pos_x": reader.i64(), "pos_y": reader.i64(),
			"vel_x": reader.i64(), "vel_y": reader.i64(), "energy": reader.i64(),
		})
	return entities


static func _read_delta(reader: Reader) -> Variant:
	var base_tick: int = reader.u64()
	var tick: int = reader.u64()
	var state_hash: int = reader.u64()
	var upserts: Variant = _read_entities(reader)
	if upserts == null:
		return null
	var count: int = reader.u32()
	if count > MAX_ENTITIES or not reader.available(count * 4):
		return null
	var removed: Array[int] = []
	var previous_id: int = -1
	var upsert_index: int = 0
	for index: int in range(count):
		var entity_id: int = reader.u32()
		if entity_id <= previous_id:
			return null
		previous_id = entity_id
		while upsert_index < upserts.size() and upserts[upsert_index]["id"] < entity_id:
			upsert_index += 1
		if upsert_index < upserts.size() and upserts[upsert_index]["id"] == entity_id:
			return null
		removed.append(entity_id)
	return {"type": "delta", "base_tick": base_tick, "tick": tick, "state_hash": state_hash, "upserts": upserts, "removed": removed}


static func encode_client_hello(protocol_version: int, player_name: String) -> PackedByteArray:
	var name_bytes: PackedByteArray = player_name.to_utf8_buffer()
	if protocol_version < 0 or protocol_version > 65535 or name_bytes.size() > MAX_NAME_BYTES or name_bytes.has(0):
		return PackedByteArray()
	var packet: PackedByteArray = _header(MessageType.CLIENT_HELLO, 4 + name_bytes.size())
	packet.encode_u16(6, protocol_version)
	packet.encode_u16(8, name_bytes.size())
	for index: int in range(name_bytes.size()):
		packet[10 + index] = name_bytes[index]
	return packet


static func encode_input(tick: int, move_x: int, move_y: int) -> PackedByteArray:
	if move_x < -1 or move_x > 1 or move_y < -1 or move_y > 1:
		return PackedByteArray()
	var packet: PackedByteArray = _header(MessageType.INPUT, 10)
	packet.encode_u64(6, tick)
	packet[14] = move_x & 0xff
	packet[15] = move_y & 0xff
	return packet


static func _header(message_type: int, payload_size: int) -> PackedByteArray:
	var packet: PackedByteArray = PackedByteArray([0x56, 0x49, 0x56, 0x50, WIRE_VERSION, message_type])
	packet.resize(6 + payload_size)
	return packet


static func _valid_utf8(data: PackedByteArray) -> bool:
	# Validate before conversion so malformed network bytes never trigger engine errors.
	var offset: int = 0
	while offset < data.size():
		var first: int = data[offset]
		if first < 0x80:
			offset += 1
			continue
		var length: int = 0
		var second_min: int = 0x80
		var second_max: int = 0xbf
		if first >= 0xc2 and first <= 0xdf:
			length = 2
		elif first >= 0xe0 and first <= 0xef:
			length = 3
			if first == 0xe0:
				second_min = 0xa0
			elif first == 0xed:
				second_max = 0x9f
		elif first >= 0xf0 and first <= 0xf4:
			length = 4
			if first == 0xf0:
				second_min = 0x90
			elif first == 0xf4:
				second_max = 0x8f
		else:
			return false
		if length > data.size() - offset:
			return false
		if data[offset + 1] < second_min or data[offset + 1] > second_max:
			return false
		for index: int in range(2, length):
			if data[offset + index] < 0x80 or data[offset + index] > 0xbf:
				return false
		offset += length
	return true
