extends SceneTree
## Headless protocol test: godot --headless --path client --script res://tests/test_protocol.gd

const P := preload("res://scripts/net/protocol.gd")

var _pass: int = 0
var _fail: int = 0


func _check(cond: bool, label: String) -> void:
	if cond:
		_pass += 1
	else:
		_fail += 1
		print("FAIL: ", label)


func _vec_dir() -> String:
	return ProjectSettings.globalize_path("res://").path_join("../protocol/test-vectors").simplify_path()


func _load(name: String) -> PackedByteArray:
	var path: String = _vec_dir().path_join(name + ".hex")
	var f: FileAccess = FileAccess.open(path, FileAccess.READ)
	if f == null:
		_check(false, "open " + path)
		return PackedByteArray()
	var txt: String = f.get_as_text().strip_edges()
	_check(txt.length() % 2 == 0 and txt.length() > 0, name + " hex length")
	return txt.hex_decode()


func _ent(e: Dictionary, id: int, owner: Variant, px: int, py: int, vx: int, vy: int, en: int, l: String) -> void:
	_check(e["id"] == id, l + " id")
	_check(e["owner"] == owner, l + " owner")
	_check(e["pos"]["x"] == px and e["pos"]["y"] == py, l + " pos")
	_check(e["vel"]["x"] == vx and e["vel"]["y"] == vy, l + " vel")
	_check(e["energy"] == en, l + " energy")


func _init() -> void:
	var hello: PackedByteArray = _load("hello")
	var input: PackedByteArray = _load("input")
	var welcome: PackedByteArray = _load("welcome")
	var snap: PackedByteArray = _load("snapshot")
	var delta: PackedByteArray = _load("delta")
	var disc: PackedByteArray = _load("disconnect")

	# Client messages
	var d: Dictionary = P.decode_client(hello)
	_check(d["ok"] and d["type"] == "hello" and d["client_name"] == "Vivatéria", "hello decode")
	_check(P.encode_hello("Vivatéria") == hello, "hello encode")
	d = P.decode_client(input)
	_check(d["ok"] and d["type"] == "input" and d["tick"] == 42 and d["move_x"] == -1 and d["move_y"] == 1, "input decode")
	_check(P.encode_input(42, -1, 1) == input, "input encode")
	_check(P.encode_input(0, 2, 0).is_empty(), "input encode rejects axis")

	# Welcome
	d = P.decode_server(welcome)
	_check(d["ok"] and d["type"] == "welcome" and d["player_id"] == 7 and d["entity_id"] == 99
			and d["tick_hz"] == 20 and d["server_tick"] == 1234, "welcome decode")

	# Snapshot
	d = P.decode_server(snap)
	_check(d["ok"] and d["type"] == "snapshot" and d["tick"] == 50 and d["entities"].size() == 2, "snapshot decode")
	if d["ok"] and d["entities"].size() == 2:
		_ent(d["entities"][0], 99, 7, -4294967296, 2147483648, 1073741824, 0, 21474836480, "snap e0")
		_ent(d["entities"][1], 100, null, 8589934592, -12884901888, 0, -4294967296, 4294967296, "snap e1")
		_check(P.fx_to_float(d["entities"][0]["pos"]["x"]) == -1.0, "fx_to_float")

	# Delta
	d = P.decode_server(delta)
	_check(d["ok"] and d["type"] == "delta" and d["base_tick"] == 50 and d["tick"] == 51
			and d["changed"].size() == 1 and d["removed"] == [100, 200], "delta decode")
	if d["ok"] and d["changed"].size() == 1:
		_ent(d["changed"][0], 99, 7, -2147483648, 4294967296, 0, 2147483648, 17179869184, "delta e0")

	# Disconnect
	d = P.decode_server(disc)
	_check(d["ok"] and d["type"] == "disconnect" and d["reason"] == "server shutdown", "disconnect decode")

	# Truncation at every length, trailing bytes, flipped magic, bad version, unknown type
	var server_vecs: Dictionary = {"welcome": welcome, "snapshot": snap, "delta": delta, "disconnect": disc}
	for k: String in server_vecs:
		var v: PackedByteArray = server_vecs[k]
		for n in v.size():
			_check(not P.decode_server(v.slice(0, n))["ok"], k + " truncated " + str(n))
		var t: PackedByteArray = v.duplicate()
		t.append(0)
		_check(not P.decode_server(t)["ok"], k + " trailing")
		var m: PackedByteArray = v.duplicate()
		m[0] = 0x58
		_check(not P.decode_server(m)["ok"], k + " bad magic")
		var ver: PackedByteArray = v.duplicate()
		ver[4] = 1
		_check(not P.decode_server(ver)["ok"], k + " bad version")
		var ty: PackedByteArray = v.duplicate()
		ty[6] = 0x7F
		_check(not P.decode_server(ty)["ok"], k + " unknown type")
	var client_vecs: Dictionary = {"hello": hello, "input": input}
	for k: String in client_vecs:
		var v: PackedByteArray = client_vecs[k]
		for n in v.size():
			_check(not P.decode_client(v.slice(0, n))["ok"], k + " truncated " + str(n))
		var t: PackedByteArray = v.duplicate()
		t.append(0)
		_check(not P.decode_client(t)["ok"], k + " trailing")
		var m: PackedByteArray = v.duplicate()
		m[1] = 0
		_check(not P.decode_client(m)["ok"], k + " bad magic")
		var cv: PackedByteArray = v.duplicate()
		cv[4] = 1
		_check(not P.decode_client(cv)["ok"], k + " bad version")
		var ct: PackedByteArray = v.duplicate()
		ct[6] = 0x7F
		_check(not P.decode_client(ct)["ok"], k + " unknown type")
	# Tick with high bit set (>= 2^63) must be rejected
	var hb: PackedByteArray = input.duplicate()
	hb[14] = 0x80
	_check(not P.decode_client(hb)["ok"], "input tick high bit")
	var hw: PackedByteArray = welcome.duplicate()
	hw[hw.size() - 1] = 0x80
	_check(not P.decode_server(hw)["ok"], "welcome server_tick high bit")
	# Wrong direction
	_check(not P.decode_server(hello)["ok"], "server rejects hello type")
	_check(not P.decode_client(welcome)["ok"], "client rejects welcome type")

	# Bad option tag (header 7 + tick 8 + count 2 + id 4 = offset 21)
	var bo: PackedByteArray = snap.duplicate()
	bo[21] = 2
	_check(not P.decode_server(bo)["ok"], "bad option tag")
	# Input axis out of range
	var ia: PackedByteArray = input.duplicate()
	ia[15] = 2
	_check(not P.decode_client(ia)["ok"], "input axis out of range")
	# Invalid UTF-8
	var bu: PackedByteArray = disc.duplicate()
	bu[9] = 0xFF
	_check(not P.decode_server(bu)["ok"], "invalid utf8")
	var bu2: PackedByteArray = P.encode_hello("ab")
	bu2[9] = 0xE2
	bu2[10] = 0x82
	_check(not P.decode_client(bu2)["ok"], "truncated utf8 sequence")
	# Over-long string
	var long: PackedByteArray = P.encode_hello("a")
	long.resize(9 + 257)
	long.encode_u16(7, 257)
	for i in 257:
		long[9 + i] = 0x61
	_check(not P.decode_client(long)["ok"], "string over 256")
	_check(P.encode_hello("a".repeat(257)).is_empty(), "encode rejects long string")
	# Huge list count with no data must fail fast
	var hc: PackedByteArray = snap.slice(0, 17)
	hc.encode_u16(15, 65535)
	_check(not P.decode_server(hc)["ok"], "huge count")

	print("protocol tests: ", _pass, " passed, ", _fail, " failed")
	quit(1 if _fail > 0 else 0)
