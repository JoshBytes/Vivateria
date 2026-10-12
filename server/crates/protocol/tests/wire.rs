use std::{fs, path::PathBuf};

use vivateria_protocol::{
    ClientHello, DecodeError, Delta, EntityState, Input, Message, ServerWelcome, Snapshot,
    MAX_ENTITIES, MAX_FRAME_BYTES,
};
use vivateria_sim::World;

fn entity(id: u32) -> EntityState {
    EntityState {
        id,
        owner_id: Some(7),
        pos_x: -4_294_967_296,
        pos_y: 2_147_483_648,
        vel_x: i64::MIN,
        vel_y: i64::MAX,
        energy: 429_496_729_600,
    }
}

fn samples() -> Vec<(&'static str, Message, &'static str)> {
    let mut npc = entity(9);
    npc.owner_id = None;
    npc.pos_x = 9_007_199_254_740_993;
    npc.pos_y = -9_007_199_254_740_993;
    npc.vel_x = 0;
    npc.vel_y = 0;
    npc.energy = 0;
    let mut changed = entity(2);
    changed.pos_x += 1;
    vec![
        ("client_hello", Message::ClientHello(ClientHello { protocol_version: 0, player_name: "Viva 🌱".into() }), "564956500001000009005669766120f09f8cb1"),
        ("server_welcome", Message::ServerWelcome(ServerWelcome { player_id: 7, tick_hz: 20, seed: u64::MAX, tick: 42 }), "5649565000020700000014000000ffffffffffffffff2a00000000000000"),
        ("input", Message::Input(Input { tick: 42, move_x: -1, move_y: 1 }), "5649565000032a00000000000000ff01"),
        ("snapshot", Message::Snapshot(Snapshot { tick: 42, state_hash: 0xfedc_ba98_7654_3210, entities: vec![entity(2), npc] }), "5649565000042a000000000000001032547698badcfe0200000002000000010700000000000000ffffffff00000080000000000000000000000080ffffffffffffff7f00000000640000000900000000000000000100000000002000ffffffffffffdfff000000000000000000000000000000000000000000000000"),
        ("delta", Message::Delta(Delta { base_tick: 42, tick: 43, state_hash: 0x0123_4567_89ab_cdef, upserts: vec![changed], removed: vec![9] }), "5649565000052a000000000000002b00000000000000efcdab89674523010100000002000000010700000001000000ffffffff00000080000000000000000000000080ffffffffffffff7f00000000640000000100000009000000"),
    ]
}

#[test]
fn golden_fixtures_and_roundtrips() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../protocol/fixtures");
    for (name, message, golden_hex) in samples() {
        let golden: Vec<u8> = golden_hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        let encoded = message.encode();
        assert_eq!(encoded, golden, "independent golden bytes for {name}");
        let path = fixture_dir.join(format!("{name}.bin"));
        if std::env::var_os("UPDATE_PROTOCOL_FIXTURES").is_some() {
            fs::create_dir_all(&fixture_dir).unwrap();
            fs::write(&path, &golden).unwrap();
        }
        assert_eq!(
            fs::read(path).unwrap(),
            encoded,
            "checked-in fixture {name}"
        );
        assert_eq!(Message::decode(&encoded).unwrap(), message);
    }
}

#[test]
fn every_truncation_and_trailing_bytes_is_rejected() {
    for (name, message, _) in samples() {
        let mut encoded = message.encode();
        for len in 0..encoded.len() {
            assert_eq!(
                Message::decode(&encoded[..len]),
                Err(DecodeError::Truncated),
                "{name} cut at {len}"
            );
        }
        encoded.push(0);
        assert_eq!(Message::decode(&encoded), Err(DecodeError::TrailingBytes));
    }
}

#[test]
fn header_errors() {
    let bytes = samples()[2].1.encode();
    let mut bad = bytes.clone();
    bad[0] = b'X';
    assert_eq!(Message::decode(&bad), Err(DecodeError::BadMagic));
    let mut bad = bytes.clone();
    bad[4] = 1;
    assert_eq!(
        Message::decode(&bad),
        Err(DecodeError::UnsupportedWireVersion(1))
    );
    for kind in [0, 6, 255] {
        let mut bad = bytes.clone();
        bad[5] = kind;
        assert_eq!(
            Message::decode(&bad),
            Err(DecodeError::UnknownMessageType(kind))
        );
    }
}

#[test]
fn movement_name_and_frame_limits() {
    let input = samples()[2].1.encode();
    for axis in [14, 15] {
        for value in [2, 127, 128, 254] {
            let mut bad = input.clone();
            bad[axis] = value;
            assert_eq!(Message::decode(&bad), Err(DecodeError::InvalidMovement));
        }
    }
    for x in -1..=1 {
        for y in -1..=1 {
            let message = Message::Input(Input {
                tick: u64::MAX,
                move_x: x,
                move_y: y,
            });
            assert_eq!(Message::decode(&message.encode()), Ok(message));
        }
    }
    let mut hello = samples()[0].1.encode();
    hello[10] = 255;
    assert_eq!(Message::decode(&hello), Err(DecodeError::InvalidUtf8));
    let mut hello = samples()[0].1.encode();
    hello[8..10].copy_from_slice(&65u16.to_le_bytes());
    assert_eq!(Message::decode(&hello), Err(DecodeError::NameTooLong));
    for name in [String::new(), "🌱".repeat(16)] {
        let message = Message::ClientHello(ClientHello {
            protocol_version: u16::MAX,
            player_name: name,
        });
        assert_eq!(Message::decode(&message.encode()), Ok(message));
    }
    let oversized = Message::ClientHello(ClientHello {
        protocol_version: 0,
        player_name: "🌱".repeat(17),
    });
    assert_eq!(oversized.try_encode(), Err(DecodeError::NameTooLong));
    assert_eq!(
        Message::decode(&vec![0; MAX_FRAME_BYTES + 1]),
        Err(DecodeError::FrameTooLarge)
    );
    assert_eq!(
        Message::Input(Input {
            tick: 0,
            move_x: 2,
            move_y: 0
        })
        .try_encode(),
        Err(DecodeError::InvalidMovement)
    );
}

#[test]
fn names_reject_nul_and_preserve_other_utf8() {
    for name in ["\0", "Viva\0🌱", "Viva\0"] {
        let message = Message::ClientHello(ClientHello {
            protocol_version: 0,
            player_name: name.into(),
        });
        assert_eq!(message.try_encode(), Err(DecodeError::NulInPlayerName));
        let mut bytes = b"VIVP\0\x01\0\0".to_vec();
        bytes.extend((name.len() as u16).to_le_bytes());
        bytes.extend(name.as_bytes());
        assert_eq!(Message::decode(&bytes), Err(DecodeError::NulInPlayerName));
    }
    for name in ["\u{feff}Viva 🌱", "\u{feff}", "é\n🌱"] {
        let message = Message::ClientHello(ClientHello {
            protocol_version: 0,
            player_name: name.into(),
        });
        assert_eq!(Message::decode(&message.encode()), Ok(message));
    }
}

#[test]
fn owners_ordering_and_count_validation() {
    let snapshot = samples()[3].1.encode();
    let mut bad = snapshot.clone();
    bad[30] = 2;
    assert_eq!(Message::decode(&bad), Err(DecodeError::InvalidOwnerTag(2)));
    let mut bad = snapshot.clone();
    bad[30] = 0;
    assert_eq!(Message::decode(&bad), Err(DecodeError::NonzeroAbsentOwner));
    let mut bad = snapshot.clone();
    bad[75..79].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(Message::decode(&bad), Err(DecodeError::DuplicateIds));
    let mut bad = snapshot.clone();
    bad[75..79].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(Message::decode(&bad), Err(DecodeError::UnorderedIds));
    let mut bad = snapshot;
    bad[22..26].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(Message::decode(&bad), Err(DecodeError::TooManyEntities));
    let mut delta = samples()[4].1.encode();
    let last = delta.len() - 4;
    delta[last..].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(Message::decode(&delta), Err(DecodeError::ConflictingIds));
    let mut delta = samples()[4].1.encode();
    let count_offset = delta.len() - 8;
    delta[count_offset..count_offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(Message::decode(&delta), Err(DecodeError::TooManyEntities));
    let mut delta = Message::Delta(Delta {
        base_tick: 0,
        tick: 1,
        state_hash: 0,
        upserts: vec![],
        removed: vec![2, 9],
    })
    .encode();
    let last = delta.len() - 4;
    delta[last..].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(Message::decode(&delta), Err(DecodeError::DuplicateIds));
    delta[last..].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(Message::decode(&delta), Err(DecodeError::UnorderedIds));
}

#[test]
fn encoder_canonicalizes_and_rejects_ambiguous_lists() {
    let sorted = Message::Snapshot(Snapshot {
        tick: 0,
        state_hash: 0,
        entities: vec![entity(2), entity(9)],
    });
    let reversed = Message::Snapshot(Snapshot {
        tick: 0,
        state_hash: 0,
        entities: vec![entity(9), entity(2)],
    });
    assert_eq!(sorted.encode(), reversed.encode());
    let duplicate = Message::Snapshot(Snapshot {
        tick: 0,
        state_hash: 0,
        entities: vec![entity(2), entity(2)],
    });
    assert_eq!(duplicate.try_encode(), Err(DecodeError::DuplicateIds));
    let duplicate = Message::Delta(Delta {
        base_tick: 0,
        tick: 1,
        state_hash: 0,
        upserts: vec![],
        removed: vec![2, 2],
    });
    assert_eq!(duplicate.try_encode(), Err(DecodeError::DuplicateIds));
    let conflicting = Message::Delta(Delta {
        base_tick: 0,
        tick: 1,
        state_hash: 0,
        upserts: vec![entity(2)],
        removed: vec![2],
    });
    assert_eq!(conflicting.try_encode(), Err(DecodeError::ConflictingIds));
    let sorted = Message::Delta(Delta {
        base_tick: 0,
        tick: 1,
        state_hash: 0,
        upserts: vec![entity(2), entity(9)],
        removed: vec![10, 12],
    });
    let reversed = Message::Delta(Delta {
        base_tick: 0,
        tick: 1,
        state_hash: 0,
        upserts: vec![entity(9), entity(2)],
        removed: vec![12, 10],
    });
    assert_eq!(sorted.encode(), reversed.encode());
    assert_eq!(Message::decode(&sorted.encode()), Ok(sorted));
    let oversized = Message::Snapshot(Snapshot {
        tick: 0,
        state_hash: 0,
        entities: vec![entity(2); MAX_ENTITIES + 1],
    });
    assert_eq!(oversized.try_encode(), Err(DecodeError::TooManyEntities));
}

#[test]
fn empty_collections_and_max_count() {
    for message in [
        Message::Snapshot(Snapshot {
            tick: 0,
            state_hash: u64::MAX,
            entities: vec![],
        }),
        Message::Delta(Delta {
            base_tick: u64::MAX,
            tick: 0,
            state_hash: 0,
            upserts: vec![],
            removed: vec![],
        }),
    ] {
        assert_eq!(Message::decode(&message.encode()), Ok(message));
    }
    let message = Message::Snapshot(Snapshot {
        tick: 0,
        state_hash: 0,
        entities: (0..MAX_ENTITIES as u32).map(entity).collect(),
    });
    assert_eq!(Message::decode(&message.encode()), Ok(message));
}

#[test]
fn sim_conversion_preserves_exact_raw_state() {
    let mut world = World::new(u64::MAX);
    world.step(&[]);
    let snapshot = Snapshot::from(&world);
    assert_eq!(snapshot.tick, world.tick);
    assert_eq!(snapshot.state_hash, world.state_hash());
    for (wire, simulation) in snapshot.entities.iter().zip(world.entities()) {
        assert_eq!(vivateria_sim::Entity::from(wire), *simulation);
    }
    let extremes = entity(2);
    assert_eq!(
        EntityState::from(&vivateria_sim::Entity::from(&extremes)),
        extremes
    );
    let input = Input {
        tick: 42,
        move_x: -1,
        move_y: 1,
    }
    .to_player_input(7);
    assert_eq!(input.player, 7);
    assert_eq!((input.move_x, input.move_y), (-1, 1));
}

#[test]
fn arbitrary_short_frames_never_panic() {
    let mut state = 1u64;
    for len in 0..512 {
        let bytes: Vec<u8> = (0..len)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                (state >> 32) as u8
            })
            .collect();
        let _ = Message::decode(&bytes);
    }
}
