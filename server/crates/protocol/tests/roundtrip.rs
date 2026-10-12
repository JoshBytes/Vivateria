use vivateria_fixed::{Fx, Pcg32};
use vivateria_protocol::{ClientMsg, DecodeError, EncodeError, EntityState, ServerMsg, Vec2};

fn entity(id: u32, owner: Option<u32>, values: [i64; 5]) -> EntityState {
    EntityState {
        id,
        owner,
        pos: Vec2::new(Fx::from_raw(values[0]), Fx::from_raw(values[1])),
        vel: Vec2::new(Fx::from_raw(values[2]), Fx::from_raw(values[3])),
        energy: Fx::from_raw(values[4]),
    }
}

#[test]
fn every_client_variant_round_trips_with_edges() {
    let messages = [
        ClientMsg::Hello {
            client_name: String::new(),
        },
        ClientMsg::Hello {
            client_name: "x".repeat(256),
        },
        ClientMsg::Input {
            tick: u64::MAX,
            move_x: -1,
            move_y: 1,
        },
    ];
    for message in messages {
        assert_eq!(ClientMsg::decode(&message.encode().unwrap()), Ok(message));
    }
}

#[test]
fn every_server_variant_round_trips_with_edges() {
    let messages = [
        ServerMsg::Welcome {
            player_id: u32::MAX,
            entity_id: 0,
            tick_hz: 20,
            server_tick: u64::MAX,
        },
        ServerMsg::Snapshot {
            tick: 0,
            entities: Vec::new(),
        },
        ServerMsg::Snapshot {
            tick: 1,
            entities: vec![
                entity(1, None, [i64::MIN, i64::MAX, -1, 0, i64::MIN]),
                entity(2, Some(u32::MAX), [i64::MAX, i64::MIN, 1, -1, i64::MAX]),
            ],
        },
        ServerMsg::Delta {
            base_tick: 4,
            tick: 5,
            changed: Vec::new(),
            removed: Vec::new(),
        },
        ServerMsg::Disconnect {
            reason: "é".repeat(128),
        },
    ];
    for message in messages {
        assert_eq!(ServerMsg::decode(&message.encode().unwrap()), Ok(message));
    }
}

fn hello_frame() -> Vec<u8> {
    ClientMsg::Hello {
        client_name: "ok".into(),
    }
    .encode()
    .unwrap()
}

#[test]
fn encode_reports_invalid_values() {
    assert_eq!(
        ClientMsg::Hello {
            client_name: "x".repeat(257)
        }
        .encode(),
        Err(EncodeError::StringTooLong {
            length: 257,
            max: 256
        })
    );
    assert_eq!(
        ClientMsg::Input {
            tick: 0,
            move_x: -2,
            move_y: 0
        }
        .encode(),
        Err(EncodeError::AxisOutOfRange {
            axis: "move_x",
            value: -2
        })
    );
    assert_eq!(
        ServerMsg::Delta {
            base_tick: 0,
            tick: 1,
            changed: Vec::new(),
            removed: vec![0; usize::from(u16::MAX) + 1]
        }
        .encode(),
        Err(EncodeError::ListTooLong {
            field: "removed",
            length: usize::from(u16::MAX) + 1,
            max: usize::from(u16::MAX)
        })
    );
}

#[test]
fn rejects_bad_header_type_truncation_and_trailing_data() {
    let valid = hello_frame();
    let mut wrong_magic = valid.clone();
    wrong_magic[0] = 0;
    assert_eq!(
        ClientMsg::decode(&wrong_magic),
        Err(DecodeError::WrongMagic)
    );

    let mut version = valid.clone();
    version[4] = 1;
    assert_eq!(
        ClientMsg::decode(&version),
        Err(DecodeError::UnknownVersion(1))
    );

    let mut unknown = valid.clone();
    unknown[6] = 0x7f;
    assert_eq!(
        ClientMsg::decode(&unknown),
        Err(DecodeError::UnknownMessageType(0x7f))
    );

    assert_eq!(
        ClientMsg::decode(&valid[..valid.len() - 1]),
        Err(DecodeError::Truncated)
    );
    let mut trailing = valid;
    trailing.push(0);
    assert_eq!(
        ClientMsg::decode(&trailing),
        Err(DecodeError::TrailingBytes(1))
    );
}

#[test]
fn rejects_messages_from_the_wrong_direction() {
    let server_frame = ServerMsg::Welcome {
        player_id: 1,
        entity_id: 2,
        tick_hz: 20,
        server_tick: 3,
    }
    .encode()
    .unwrap();
    assert_eq!(
        ClientMsg::decode(&server_frame),
        Err(DecodeError::UnknownMessageType(0x81))
    );

    let client_frame = hello_frame();
    assert_eq!(
        ServerMsg::decode(&client_frame),
        Err(DecodeError::UnknownMessageType(0x01))
    );
}

#[test]
fn rejects_invalid_options_inputs_and_strings() {
    let mut option = ServerMsg::Snapshot {
        tick: 0,
        entities: vec![entity(1, None, [0; 5])],
    }
    .encode()
    .unwrap();
    option[21] = 2;
    assert_eq!(
        ServerMsg::decode(&option),
        Err(DecodeError::InvalidOptionTag(2))
    );

    let mut input = ClientMsg::Input {
        tick: 0,
        move_x: 0,
        move_y: 0,
    }
    .encode()
    .unwrap();
    input[15] = 2;
    assert_eq!(
        ClientMsg::decode(&input),
        Err(DecodeError::InputOutOfRange {
            axis: "move_x",
            value: 2
        })
    );

    input[15] = 0;
    input[16] = 0x80;
    assert_eq!(
        ClientMsg::decode(&input),
        Err(DecodeError::InputOutOfRange {
            axis: "move_y",
            value: -128
        })
    );

    let invalid_utf8 = [b'V', b'I', b'V', b'A', 0, 0, 1, 1, 0, 0xff];
    assert_eq!(
        ClientMsg::decode(&invalid_utf8),
        Err(DecodeError::InvalidUtf8)
    );

    let overlong = [b'V', b'I', b'V', b'A', 0, 0, 1, 1, 1];
    assert_eq!(
        ClientMsg::decode(&overlong),
        Err(DecodeError::StringTooLong(257))
    );
}

#[test]
fn rejects_impossible_list_counts_before_allocating() {
    let snapshot = [
        b'V', b'I', b'V', b'A', 0, 0, 0x82, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
    ];
    assert_eq!(
        ServerMsg::decode(&snapshot),
        Err(DecodeError::CountExceedsRemaining {
            count: 1,
            item_size: 45
        })
    );

    let mut delta = ServerMsg::Delta {
        base_tick: 0,
        tick: 1,
        changed: Vec::new(),
        removed: vec![42],
    }
    .encode()
    .unwrap();
    delta[25..27].copy_from_slice(&2_u16.to_le_bytes());
    assert_eq!(
        ServerMsg::decode(&delta),
        Err(DecodeError::CountExceedsRemaining {
            count: 2,
            item_size: 4
        })
    );
}

#[test]
fn arbitrary_bytes_and_valid_frame_truncations_never_panic() {
    let mut rng = Pcg32::new(0x5649_5641, 0);
    for _ in 0..10_000 {
        let len = rng.range_u32(512) as usize;
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len {
            bytes.push(rng.next_u32() as u8);
        }
        let _ = ClientMsg::decode(&bytes);
        let _ = ServerMsg::decode(&bytes);
    }

    let frames = [
        hello_frame(),
        ServerMsg::Delta {
            base_tick: 9,
            tick: 10,
            changed: vec![entity(7, Some(3), [1, -1, i64::MIN, i64::MAX, 0])],
            removed: vec![1, 2, 3],
        }
        .encode()
        .unwrap(),
    ];
    for frame in frames {
        for end in 0..frame.len() {
            let _ = ClientMsg::decode(&frame[..end]);
            let _ = ServerMsg::decode(&frame[..end]);
        }
    }
}

fn sample_messages() -> [(ServerOrClient, &'static str); 6] {
    [
        (
            ServerOrClient::Client(ClientMsg::Hello {
                client_name: "Vivatéria".into(),
            }),
            include_str!("../../../../protocol/test-vectors/hello.hex"),
        ),
        (
            ServerOrClient::Client(ClientMsg::Input {
                tick: 42,
                move_x: -1,
                move_y: 1,
            }),
            include_str!("../../../../protocol/test-vectors/input.hex"),
        ),
        (
            ServerOrClient::Server(ServerMsg::Welcome {
                player_id: 7,
                entity_id: 99,
                tick_hz: 20,
                server_tick: 1_234,
            }),
            include_str!("../../../../protocol/test-vectors/welcome.hex"),
        ),
        (
            ServerOrClient::Server(ServerMsg::Snapshot {
                tick: 50,
                entities: vec![
                    entity(
                        99,
                        Some(7),
                        [-(1_i64 << 32), 1_i64 << 31, 1_i64 << 30, 0, 5_i64 << 32],
                    ),
                    entity(
                        100,
                        None,
                        [2_i64 << 32, -(3_i64 << 32), 0, -(1_i64 << 32), 1_i64 << 32],
                    ),
                ],
            }),
            include_str!("../../../../protocol/test-vectors/snapshot.hex"),
        ),
        (
            ServerOrClient::Server(ServerMsg::Delta {
                base_tick: 50,
                tick: 51,
                changed: vec![entity(
                    99,
                    Some(7),
                    [-(1_i64 << 31), 1_i64 << 32, 0, 1_i64 << 31, 4_i64 << 32],
                )],
                removed: vec![100, 200],
            }),
            include_str!("../../../../protocol/test-vectors/delta.hex"),
        ),
        (
            ServerOrClient::Server(ServerMsg::Disconnect {
                reason: "server shutdown".into(),
            }),
            include_str!("../../../../protocol/test-vectors/disconnect.hex"),
        ),
    ]
}

enum ServerOrClient {
    Client(ClientMsg),
    Server(ServerMsg),
}

#[test]
fn encodings_match_golden_vectors_exactly() {
    for (message, expected) in sample_messages() {
        let bytes = match &message {
            ServerOrClient::Client(message) => message.encode().unwrap(),
            ServerOrClient::Server(message) => message.encode().unwrap(),
        };
        assert_eq!(hex(&bytes), expected.trim());

        let golden_bytes = decode_hex(expected);
        assert_eq!(golden_bytes, bytes);
        match message {
            ServerOrClient::Client(expected_message) => {
                assert_eq!(ClientMsg::decode(&golden_bytes), Ok(expected_message));
            }
            ServerOrClient::Server(expected_message) => {
                assert_eq!(ServerMsg::decode(&golden_bytes), Ok(expected_message));
            }
        }
    }
}

fn decode_hex(text: &str) -> Vec<u8> {
    let text = text.trim();
    assert!(!text.is_empty());
    assert!(text.len().is_multiple_of(2));
    assert!(text
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let high = hex_digit(pair[0]);
            let low = hex_digit(pair[1]);
            (high << 4) | low
        })
        .collect()
}

fn hex_digit(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => unreachable!("validated lowercase hexadecimal digit"),
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}
