class_name Protocol
extends RefCounted
## Vivateria protocol v0 codec (serialization only; see protocol/SCHEMA.md).
## Fx values stay raw Q32.32 ints. fx_to_float is for rendering only.

const MAGIC: PackedByteArray = [0x56, 0x49, 0x56, 0x41]
const VERSION: int = 0
const MAX_STRING_BYTES: int = 256
## Minimum encoded size of one EntityState (id 4 + option tag 1 + 4 x Fx 32 + energy 8).
const MIN_ENTITY_BYTES: int = 45

const MSG_HELLO: int = 0x01
const MSG_INPUT: int = 0x02
const MSG_WELCOME: int = 0x81
const MSG_SNAPSHOT: int = 0x82
const MSG_DELTA: int = 0x83
const MSG_DISCONNECT: int = 0x84


## Bounds-checked cursor. Once `failed` is set every read returns 0 / "".
class Reader extends RefCounted:
	const MAX_STRING_BYTES: int = 256
	const MIN_ENTITY_BYTES: int = 45

	var buf: PackedByteArray
	var pos: int = 0
	var failed: bool = false
	var error: String = ""

	func _init(b: PackedByteArray) -> void:
		buf = b

	func fail(msg: String) -> void:
		if not failed:
			failed = true
			error = msg

	func remaining() -> int:
		return buf.size() - pos

	func need(n: int, what: String) -> bool:
		if failed:
			return false
		if n < 0 or remaining() < n:
			fail("truncated " + what)
			return false
		return true

	func u8(what: String) -> int:
		if not need(1, what):
			return 0
		var v: int = buf[pos]
		pos += 1
		return v

	func i8(what: String) -> int:
		if not need(1, what):
			return 0
		var v: int = buf.decode_s8(pos)
		pos += 1
		return v

	func u16(what: String) -> int:
		if not need(2, what):
			return 0
		var v: int = buf.decode_u16(pos)
		pos += 2
		return v

	func u32(what: String) -> int:
		if not need(4, what):
			return 0
		var v: int = buf.decode_u32(pos)
		pos += 4
		return v

	func u64(what: String) -> int:
		if not need(8, what):
			return 0
		var v: int = buf.decode_u64(pos)
		pos += 8
		return v

	func s64(what: String) -> int:
		if not need(8, what):
			return 0
		var v: int = buf.decode_s64(pos)
		pos += 8
		return v

	static func utf8_ok(b: PackedByteArray) -> bool:
		var i: int = 0
		var n: int = b.size()
		while i < n:
			var c: int = b[i]
			if c < 0x80:
				i += 1
				continue
			var extra: int = 0
			var minv: int = 0
			var cp: int = 0
			if c >= 0xC2 and c <= 0xDF:
				extra = 1
				minv = 0x80
				cp = c & 0x1F
			elif c >= 0xE0 and c <= 0xEF:
				extra = 2
				minv = 0x800
				cp = c & 0x0F
			elif c >= 0xF0 and c <= 0xF4:
				extra = 3
				minv = 0x10000
				cp = c & 0x07
			else:
				return false
			if i + extra >= n:
				return false
			for k in range(1, extra + 1):
				var cc: int = b[i + k]
				if (cc & 0xC0) != 0x80:
					return false
				cp = (cp << 6) | (cc & 0x3F)
			if cp < minv or cp > 0x10FFFF or (cp >= 0xD800 and cp <= 0xDFFF):
				return false
			i += extra + 1
		return true

	func string(what: String) -> String:
		var n: int = u16(what + " length")
		if failed:
			return ""
		if n > MAX_STRING_BYTES:
			fail("string too long: " + what)
			return ""
		if not need(n, what):
			return ""
		var slice: PackedByteArray = buf.slice(pos, pos + n)
		pos += n
		if not utf8_ok(slice):
			fail("invalid utf-8: " + what)
			return ""
		return slice.get_string_from_utf8()

	func vec2(what: String) -> Dictionary:
		var x: int = s64(what + ".x")
		var y: int = s64(what + ".y")
		return {"x": x, "y": y}

	func entity() -> Dictionary:
		var id: int = u32("entity id")
		var tag: int = u8("owner tag")
		var owner: Variant = null
		if not failed:
			if tag == 1:
				owner = u32("owner")
			elif tag != 0:
				fail("invalid option tag")
		var pos_v: Dictionary = vec2("pos")
		var vel_v: Dictionary = vec2("vel")
		var energy: int = s64("energy")
		return {"id": id, "owner": owner, "pos": pos_v, "vel": vel_v, "energy": energy}

	func entity_list(what: String) -> Array:
		var out: Array = []
		var n: int = u16(what + " count")
		if not failed and n * MIN_ENTITY_BYTES > remaining():
			fail("truncated " + what)
			return out
		for i in n:
			if failed:
				break
			out.append(entity())
		return out


static func fx_to_float(raw: int) -> float:
	return float(raw) / 4294967296.0


static func is_valid_utf8(b: PackedByteArray) -> bool:
	return Reader.utf8_ok(b)


static func _header(r: Reader) -> int:
	if not r.need(7, "header"):
		return -1
	if r.buf.slice(0, 4) != MAGIC:
		r.fail("bad magic")
		return -1
	r.pos = 4
	var ver: int = r.u16("version")
	if ver != VERSION:
		r.fail("unsupported version")
		return -1
	return r.u8("type")


static func _finish(r: Reader, result: Dictionary) -> Dictionary:
	if r.failed:
		return {"ok": false, "error": r.error}
	if r.remaining() != 0:
		return {"ok": false, "error": "trailing bytes"}
	result["ok"] = true
	return result


static func decode_server(bytes: PackedByteArray) -> Dictionary:
	var r: Reader = Reader.new(bytes)
	var t: int = _header(r)
	if r.failed:
		return {"ok": false, "error": r.error}
	match t:
		MSG_WELCOME:
			var pid: int = r.u32("player_id")
			var eid: int = r.u32("entity_id")
			var hz: int = r.u8("tick_hz")
			var st: int = r.u64("server_tick")
			return _finish(r, {"type": "welcome", "player_id": pid, "entity_id": eid,
					"tick_hz": hz, "server_tick": st})
		MSG_SNAPSHOT:
			var tick: int = r.u64("tick")
			var ents: Array = r.entity_list("entities")
			return _finish(r, {"type": "snapshot", "tick": tick, "entities": ents})
		MSG_DELTA:
			var base: int = r.u64("base_tick")
			var tick2: int = r.u64("tick")
			var changed: Array = r.entity_list("changed")
			var removed: Array = []
			var n: int = r.u16("removed count")
			if not r.failed and n * 4 > r.remaining():
				r.fail("truncated removed")
			for i in n:
				if r.failed:
					break
				removed.append(r.u32("removed id"))
			return _finish(r, {"type": "delta", "base_tick": base, "tick": tick2,
					"changed": changed, "removed": removed})
		MSG_DISCONNECT:
			var reason: String = r.string("reason")
			return _finish(r, {"type": "disconnect", "reason": reason})
		_:
			return {"ok": false, "error": "unknown message type"}


static func decode_client(bytes: PackedByteArray) -> Dictionary:
	var r: Reader = Reader.new(bytes)
	var t: int = _header(r)
	if r.failed:
		return {"ok": false, "error": r.error}
	match t:
		MSG_HELLO:
			var name: String = r.string("client_name")
			return _finish(r, {"type": "hello", "client_name": name})
		MSG_INPUT:
			var tick: int = r.u64("tick")
			var mx: int = r.i8("move_x")
			var my: int = r.i8("move_y")
			if not r.failed and (mx < -1 or mx > 1 or my < -1 or my > 1):
				r.fail("input axis out of range")
			return _finish(r, {"type": "input", "tick": tick, "move_x": mx, "move_y": my})
		_:
			return {"ok": false, "error": "unknown message type"}


static func _new_frame(type_id: int, total: int) -> PackedByteArray:
	var b: PackedByteArray = PackedByteArray()
	b.resize(total)
	for i in 4:
		b[i] = MAGIC[i]
	b.encode_u16(4, VERSION)
	b[6] = type_id
	return b


## Returns an empty array if the name exceeds 256 UTF-8 bytes.
static func encode_hello(name: String) -> PackedByteArray:
	var s: PackedByteArray = name.to_utf8_buffer()
	if s.size() > MAX_STRING_BYTES:
		return PackedByteArray()
	var b: PackedByteArray = _new_frame(MSG_HELLO, 9 + s.size())
	b.encode_u16(7, s.size())
	for i in s.size():
		b[9 + i] = s[i]
	return b


## Returns an empty array if an axis is outside -1..1.
static func encode_input(tick: int, move_x: int, move_y: int) -> PackedByteArray:
	if move_x < -1 or move_x > 1 or move_y < -1 or move_y > 1:
		return PackedByteArray()
	var b: PackedByteArray = _new_frame(MSG_INPUT, 17)
	b.encode_u64(7, tick)
	b.encode_s8(15, move_x)
	b.encode_s8(16, move_y)
	return b
