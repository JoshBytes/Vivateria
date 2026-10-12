//! Vivateria protocol v0 binary messages.

use core::fmt;
use vivateria_fixed::Fx;

const MAGIC: &[u8; 4] = b"VIVA";
const VERSION: u16 = 0;
const HEADER_LEN: usize = 7;
const MAX_STRING_BYTES: usize = 256;
const MIN_ENTITY_BYTES: usize = 45;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vec2 {
    pub x: Fx,
    pub y: Fx,
}

impl Vec2 {
    pub const fn new(x: Fx, y: Fx) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityState {
    pub id: u32,
    pub owner: Option<u32>,
    pub pos: Vec2,
    pub vel: Vec2,
    pub energy: Fx,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientMsg {
    Hello { client_name: String },
    Input { tick: u64, move_x: i8, move_y: i8 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerMsg {
    Welcome {
        player_id: u32,
        entity_id: u32,
        tick_hz: u8,
        server_tick: u64,
    },
    Snapshot {
        tick: u64,
        entities: Vec<EntityState>,
    },
    Delta {
        base_tick: u64,
        tick: u64,
        changed: Vec<EntityState>,
        removed: Vec<u32>,
    },
    Disconnect {
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    Truncated,
    WrongMagic,
    UnknownVersion(u16),
    UnknownMessageType(u8),
    TrailingBytes(usize),
    InvalidOptionTag(u8),
    InputOutOfRange { axis: &'static str, value: i8 },
    InvalidUtf8,
    StringTooLong(usize),
    CountExceedsRemaining { count: u16, item_size: usize },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DecodeError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncodeError {
    StringTooLong {
        length: usize,
        max: usize,
    },
    AxisOutOfRange {
        axis: &'static str,
        value: i8,
    },
    ListTooLong {
        field: &'static str,
        length: usize,
        max: usize,
    },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for EncodeError {}

impl ClientMsg {
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = header(match self {
            Self::Hello { .. } => 0x01,
            Self::Input { .. } => 0x02,
        });
        match self {
            Self::Hello { client_name } => {
                put_string(&mut out, client_name)?;
            }
            Self::Input {
                tick,
                move_x,
                move_y,
            } => {
                validate_encode_axis("move_x", *move_x)?;
                validate_encode_axis("move_y", *move_y)?;
                put_u64(&mut out, *tick);
                out.push(*move_x as u8);
                out.push(*move_y as u8);
            }
        }
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = Reader::new(bytes);
        let ty = reader.header()?;
        let message = match ty {
            0x01 => Self::Hello {
                client_name: reader.string()?,
            },
            0x02 => {
                let tick = reader.u64()?;
                let move_x = reader.i8()?;
                let move_y = reader.i8()?;
                validate_axis("move_x", move_x)?;
                validate_axis("move_y", move_y)?;
                Self::Input {
                    tick,
                    move_x,
                    move_y,
                }
            }
            other => return Err(DecodeError::UnknownMessageType(other)),
        };
        reader.finish(message)
    }
}

impl ServerMsg {
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut out = header(match self {
            Self::Welcome { .. } => 0x81,
            Self::Snapshot { .. } => 0x82,
            Self::Delta { .. } => 0x83,
            Self::Disconnect { .. } => 0x84,
        });
        match self {
            Self::Welcome {
                player_id,
                entity_id,
                tick_hz,
                server_tick,
            } => {
                put_u32(&mut out, *player_id);
                put_u32(&mut out, *entity_id);
                out.push(*tick_hz);
                put_u64(&mut out, *server_tick);
            }
            Self::Snapshot { tick, entities } => {
                put_u64(&mut out, *tick);
                put_entities(&mut out, "entities", entities)?;
            }
            Self::Delta {
                base_tick,
                tick,
                changed,
                removed,
            } => {
                put_u64(&mut out, *base_tick);
                put_u64(&mut out, *tick);
                put_entities(&mut out, "changed", changed)?;
                put_u32_list(&mut out, "removed", removed)?;
            }
            Self::Disconnect { reason } => {
                put_string(&mut out, reason)?;
            }
        }
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = Reader::new(bytes);
        let ty = reader.header()?;
        let message = match ty {
            0x81 => Self::Welcome {
                player_id: reader.u32()?,
                entity_id: reader.u32()?,
                tick_hz: reader.u8()?,
                server_tick: reader.u64()?,
            },
            0x82 => Self::Snapshot {
                tick: reader.u64()?,
                entities: reader.entities()?,
            },
            0x83 => Self::Delta {
                base_tick: reader.u64()?,
                tick: reader.u64()?,
                changed: reader.entities()?,
                removed: reader.u32_list()?,
            },
            0x84 => Self::Disconnect {
                reason: reader.string()?,
            },
            other => return Err(DecodeError::UnknownMessageType(other)),
        };
        reader.finish(message)
    }
}

fn validate_axis(axis: &'static str, value: i8) -> Result<(), DecodeError> {
    if (-1..=1).contains(&value) {
        Ok(())
    } else {
        Err(DecodeError::InputOutOfRange { axis, value })
    }
}

fn validate_encode_axis(axis: &'static str, value: i8) -> Result<(), EncodeError> {
    if (-1..=1).contains(&value) {
        Ok(())
    } else {
        Err(EncodeError::AxisOutOfRange { axis, value })
    }
}

fn header(message_type: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN);
    out.extend_from_slice(MAGIC);
    put_u16(&mut out, VERSION);
    out.push(message_type);
    out
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_i64(out: &mut Vec<u8>, value: i64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), EncodeError> {
    let bytes = value.as_bytes();
    if bytes.len() > MAX_STRING_BYTES {
        return Err(EncodeError::StringTooLong {
            length: bytes.len(),
            max: MAX_STRING_BYTES,
        });
    }
    let len = u16::try_from(bytes.len()).expect("maximum string length fits in u16");
    put_u16(out, len);
    out.extend_from_slice(bytes);
    Ok(())
}

fn put_entities(
    out: &mut Vec<u8>,
    field: &'static str,
    values: &[EntityState],
) -> Result<(), EncodeError> {
    let count = list_count(field, values.len())?;
    put_u16(out, count);
    for value in values {
        put_entity(out, value);
    }
    Ok(())
}

fn put_u32_list(out: &mut Vec<u8>, field: &'static str, values: &[u32]) -> Result<(), EncodeError> {
    let count = list_count(field, values.len())?;
    put_u16(out, count);
    for value in values {
        put_u32(out, *value);
    }
    Ok(())
}

fn list_count(field: &'static str, length: usize) -> Result<u16, EncodeError> {
    u16::try_from(length).map_err(|_| EncodeError::ListTooLong {
        field,
        length,
        max: usize::from(u16::MAX),
    })
}

fn put_entity(out: &mut Vec<u8>, value: &EntityState) {
    put_u32(out, value.id);
    match value.owner {
        None => out.push(0),
        Some(owner) => {
            out.push(1);
            put_u32(out, owner);
        }
    }
    put_vec2(out, value.pos);
    put_vec2(out, value.vel);
    put_i64(out, value.energy.to_raw());
}

fn put_vec2(out: &mut Vec<u8>, value: Vec2) {
    put_i64(out, value.x.to_raw());
    put_i64(out, value.y.to_raw());
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn header(&mut self) -> Result<u8, DecodeError> {
        let magic = self.take(4)?;
        if magic != MAGIC {
            return Err(DecodeError::WrongMagic);
        }
        let version = self.u16()?;
        if version != VERSION {
            return Err(DecodeError::UnknownVersion(version));
        }
        self.u8()
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let end = self.pos.checked_add(len).ok_or(DecodeError::Truncated)?;
        let value = self
            .bytes
            .get(self.pos..end)
            .ok_or(DecodeError::Truncated)?;
        self.pos = end;
        Ok(value)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        self.take(N)?.try_into().map_err(|_| DecodeError::Truncated)
    }

    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.array::<1>()?[0])
    }

    fn i8(&mut self) -> Result<i8, DecodeError> {
        Ok(i8::from_le_bytes(self.array()?))
    }

    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn i64(&mut self) -> Result<i64, DecodeError> {
        Ok(i64::from_le_bytes(self.array()?))
    }

    fn string(&mut self) -> Result<String, DecodeError> {
        let len = usize::from(self.u16()?);
        if len > MAX_STRING_BYTES {
            return Err(DecodeError::StringTooLong(len));
        }
        let bytes = self.take(len)?;
        let text = core::str::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)?;
        Ok(text.to_owned())
    }

    fn option_u32(&mut self) -> Result<Option<u32>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.u32()?)),
            other => Err(DecodeError::InvalidOptionTag(other)),
        }
    }

    fn vec2(&mut self) -> Result<Vec2, DecodeError> {
        Ok(Vec2::new(
            Fx::from_raw(self.i64()?),
            Fx::from_raw(self.i64()?),
        ))
    }

    fn entity(&mut self) -> Result<EntityState, DecodeError> {
        Ok(EntityState {
            id: self.u32()?,
            owner: self.option_u32()?,
            pos: self.vec2()?,
            vel: self.vec2()?,
            energy: Fx::from_raw(self.i64()?),
        })
    }

    fn entities(&mut self) -> Result<Vec<EntityState>, DecodeError> {
        let count = self.u16()?;
        if usize::from(count) > self.remaining() / MIN_ENTITY_BYTES {
            return Err(DecodeError::CountExceedsRemaining {
                count,
                item_size: MIN_ENTITY_BYTES,
            });
        }
        let mut values = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            values.push(self.entity()?);
        }
        Ok(values)
    }

    fn u32_list(&mut self) -> Result<Vec<u32>, DecodeError> {
        let count = self.u16()?;
        if usize::from(count) > self.remaining() / 4 {
            return Err(DecodeError::CountExceedsRemaining {
                count,
                item_size: 4,
            });
        }
        let mut values = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            values.push(self.u32()?);
        }
        Ok(values)
    }

    fn finish<T>(&self, value: T) -> Result<T, DecodeError> {
        let remaining = self.remaining();
        if remaining == 0 {
            Ok(value)
        } else {
            Err(DecodeError::TrailingBytes(remaining))
        }
    }
}
