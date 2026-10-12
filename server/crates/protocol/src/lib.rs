//! Binary protocol v0. See `protocol/v0.md` for the wire contract.

use std::{collections::BTreeSet, fmt};

use vivateria_fixed::Fx;
use vivateria_sim::{Entity, FxVec2, PlayerInput, World};

pub const MAGIC: [u8; 4] = *b"VIVP";
pub const WIRE_VERSION: u8 = 0;
pub const PROTOCOL_VERSION: u16 = 0;
pub const MAX_NAME_BYTES: usize = 64;
pub const MAX_ENTITIES: usize = 65_536;
pub const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;
const ENTITY_BYTES: usize = 49;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientHello {
    pub protocol_version: u16,
    pub player_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerWelcome {
    pub player_id: u32,
    pub tick_hz: u32,
    pub seed: u64,
    pub tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    pub tick: u64,
    pub move_x: i8,
    pub move_y: i8,
}

impl Input {
    /// Identity comes from the authenticated connection, never from the input frame.
    pub fn to_player_input(&self, player: u32) -> PlayerInput {
        PlayerInput {
            player,
            move_x: self.move_x,
            move_y: self.move_y,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityState {
    pub id: u32,
    pub owner_id: Option<u32>,
    pub pos_x: i64,
    pub pos_y: i64,
    pub vel_x: i64,
    pub vel_y: i64,
    pub energy: i64,
}

impl From<&Entity> for EntityState {
    fn from(entity: &Entity) -> Self {
        Self {
            id: entity.id,
            owner_id: entity.owner,
            pos_x: entity.pos.x.to_raw(),
            pos_y: entity.pos.y.to_raw(),
            vel_x: entity.vel.x.to_raw(),
            vel_y: entity.vel.y.to_raw(),
            energy: entity.energy.to_raw(),
        }
    }
}

impl From<&EntityState> for Entity {
    fn from(entity: &EntityState) -> Self {
        Self {
            id: entity.id,
            owner: entity.owner_id,
            pos: FxVec2::new(Fx::from_raw(entity.pos_x), Fx::from_raw(entity.pos_y)),
            vel: FxVec2::new(Fx::from_raw(entity.vel_x), Fx::from_raw(entity.vel_y)),
            energy: Fx::from_raw(entity.energy),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub tick: u64,
    pub state_hash: u64,
    pub entities: Vec<EntityState>,
}

impl From<&World> for Snapshot {
    fn from(world: &World) -> Self {
        Self {
            tick: world.tick,
            state_hash: world.state_hash(),
            entities: world.entities().map(EntityState::from).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delta {
    pub base_tick: u64,
    pub tick: u64,
    pub state_hash: u64,
    pub upserts: Vec<EntityState>,
    pub removed: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    ClientHello(ClientHello),
    ServerWelcome(ServerWelcome),
    Input(Input),
    Snapshot(Snapshot),
    Delta(Delta),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    BadMagic,
    UnsupportedWireVersion(u8),
    UnknownMessageType(u8),
    Truncated,
    TrailingBytes,
    InvalidUtf8,
    NulInPlayerName,
    NameTooLong,
    TooManyEntities,
    FrameTooLarge,
    InvalidMovement,
    InvalidOwnerTag(u8),
    NonzeroAbsentOwner,
    UnorderedIds,
    DuplicateIds,
    ConflictingIds,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "protocol v0 error: {self:?}")
    }
}
impl std::error::Error for DecodeError {}

impl Message {
    /// Canonically encode a valid local message (entity/removed lists are sorted).
    ///
    /// # Panics
    /// Panics for invalid locally constructed messages. Use [`Self::try_encode`]
    /// when fields are supplied by untrusted code. Decoding never calls this method.
    pub fn encode(&self) -> Vec<u8> {
        self.try_encode().expect("invalid local protocol message")
    }

    pub fn try_encode(&self) -> Result<Vec<u8>, DecodeError> {
        let mut out = MAGIC.to_vec();
        out.push(WIRE_VERSION);
        match self {
            Self::ClientHello(hello) => {
                if hello.player_name.len() > MAX_NAME_BYTES {
                    return Err(DecodeError::NameTooLong);
                }
                if hello.player_name.as_bytes().contains(&0) {
                    return Err(DecodeError::NulInPlayerName);
                }
                out.push(1);
                out.extend(hello.protocol_version.to_le_bytes());
                out.extend((hello.player_name.len() as u16).to_le_bytes());
                out.extend(hello.player_name.as_bytes());
            }
            Self::ServerWelcome(welcome) => {
                out.push(2);
                out.extend(welcome.player_id.to_le_bytes());
                out.extend(welcome.tick_hz.to_le_bytes());
                out.extend(welcome.seed.to_le_bytes());
                out.extend(welcome.tick.to_le_bytes());
            }
            Self::Input(input) => {
                if !(-1..=1).contains(&input.move_x) || !(-1..=1).contains(&input.move_y) {
                    return Err(DecodeError::InvalidMovement);
                }
                out.push(3);
                out.extend(input.tick.to_le_bytes());
                out.extend(input.move_x.to_le_bytes());
                out.extend(input.move_y.to_le_bytes());
            }
            Self::Snapshot(snapshot) => {
                out.push(4);
                out.extend(snapshot.tick.to_le_bytes());
                out.extend(snapshot.state_hash.to_le_bytes());
                encode_entities(&mut out, &snapshot.entities)?;
            }
            Self::Delta(delta) => {
                out.push(5);
                out.extend(delta.base_tick.to_le_bytes());
                out.extend(delta.tick.to_le_bytes());
                out.extend(delta.state_hash.to_le_bytes());
                encode_entities(&mut out, &delta.upserts)?;
                if delta.removed.len() > MAX_ENTITIES {
                    return Err(DecodeError::TooManyEntities);
                }
                let mut removed = delta.removed.clone();
                removed.sort_unstable();
                check_order(removed.iter().copied())?;
                let upsert_ids: BTreeSet<u32> = delta.upserts.iter().map(|e| e.id).collect();
                if removed.iter().any(|id| upsert_ids.contains(id)) {
                    return Err(DecodeError::ConflictingIds);
                }
                out.extend((removed.len() as u32).to_le_bytes());
                for id in removed {
                    out.extend(id.to_le_bytes());
                }
            }
        }
        if out.len() > MAX_FRAME_BYTES {
            return Err(DecodeError::FrameTooLarge);
        }
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() > MAX_FRAME_BYTES {
            return Err(DecodeError::FrameTooLarge);
        }
        let mut reader = Reader { bytes, offset: 0 };
        if reader.take::<4>()? != MAGIC {
            return Err(DecodeError::BadMagic);
        }
        let version = reader.u8()?;
        if version != WIRE_VERSION {
            return Err(DecodeError::UnsupportedWireVersion(version));
        }
        let message = match reader.u8()? {
            1 => {
                let protocol_version = reader.u16()?;
                let len = usize::from(reader.u16()?);
                if len > MAX_NAME_BYTES {
                    return Err(DecodeError::NameTooLong);
                }
                let name = reader.slice(len)?;
                let player_name = std::str::from_utf8(name)
                    .map_err(|_| DecodeError::InvalidUtf8)?
                    .to_owned();
                if name.contains(&0) {
                    return Err(DecodeError::NulInPlayerName);
                }
                Self::ClientHello(ClientHello {
                    protocol_version,
                    player_name,
                })
            }
            2 => Self::ServerWelcome(ServerWelcome {
                player_id: reader.u32()?,
                tick_hz: reader.u32()?,
                seed: reader.u64()?,
                tick: reader.u64()?,
            }),
            3 => {
                let tick = reader.u64()?;
                let move_x = i8::from_le_bytes(reader.take()?);
                let move_y = i8::from_le_bytes(reader.take()?);
                if !(-1..=1).contains(&move_x) || !(-1..=1).contains(&move_y) {
                    return Err(DecodeError::InvalidMovement);
                }
                Self::Input(Input {
                    tick,
                    move_x,
                    move_y,
                })
            }
            4 => Self::Snapshot(Snapshot {
                tick: reader.u64()?,
                state_hash: reader.u64()?,
                entities: reader.entities()?,
            }),
            5 => {
                let base_tick = reader.u64()?;
                let tick = reader.u64()?;
                let state_hash = reader.u64()?;
                let upserts = reader.entities()?;
                let count = reader.count(4)?;
                let mut removed = Vec::with_capacity(count);
                for _ in 0..count {
                    removed.push(reader.u32()?);
                }
                check_order(removed.iter().copied())?;
                let upsert_ids: BTreeSet<u32> = upserts.iter().map(|e| e.id).collect();
                if removed.iter().any(|id| upsert_ids.contains(id)) {
                    return Err(DecodeError::ConflictingIds);
                }
                Self::Delta(Delta {
                    base_tick,
                    tick,
                    state_hash,
                    upserts,
                    removed,
                })
            }
            kind => return Err(DecodeError::UnknownMessageType(kind)),
        };
        if reader.offset != bytes.len() {
            return Err(DecodeError::TrailingBytes);
        }
        Ok(message)
    }
}

fn check_order(ids: impl IntoIterator<Item = u32>) -> Result<(), DecodeError> {
    let mut previous = None;
    for id in ids {
        if let Some(prev) = previous {
            if id == prev {
                return Err(DecodeError::DuplicateIds);
            }
            if id < prev {
                return Err(DecodeError::UnorderedIds);
            }
        }
        previous = Some(id);
    }
    Ok(())
}

fn encode_entities(out: &mut Vec<u8>, entities: &[EntityState]) -> Result<(), DecodeError> {
    if entities.len() > MAX_ENTITIES {
        return Err(DecodeError::TooManyEntities);
    }
    let mut sorted: Vec<&EntityState> = entities.iter().collect();
    sorted.sort_unstable_by_key(|e| e.id);
    check_order(sorted.iter().map(|e| e.id))?;
    out.extend((sorted.len() as u32).to_le_bytes());
    for entity in sorted {
        out.extend(entity.id.to_le_bytes());
        out.push(u8::from(entity.owner_id.is_some()));
        out.extend(entity.owner_id.unwrap_or(0).to_le_bytes());
        for raw in [
            entity.pos_x,
            entity.pos_y,
            entity.vel_x,
            entity.vel_y,
            entity.energy,
        ] {
            out.extend(raw.to_le_bytes());
        }
    }
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn slice(&mut self, count: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or(DecodeError::Truncated)?;
        let result = self
            .bytes
            .get(self.offset..end)
            .ok_or(DecodeError::Truncated)?;
        self.offset = end;
        Ok(result)
    }
    fn take<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let mut result = [0; N];
        result.copy_from_slice(self.slice(N)?);
        Ok(result)
    }
    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take::<1>()?[0])
    }
    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.take()?))
    }
    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn i64(&mut self) -> Result<i64, DecodeError> {
        Ok(i64::from_le_bytes(self.take()?))
    }
    fn count(&mut self, item_bytes: usize) -> Result<usize, DecodeError> {
        let count = self.u32()? as usize;
        if count > MAX_ENTITIES {
            return Err(DecodeError::TooManyEntities);
        }
        let need = count
            .checked_mul(item_bytes)
            .ok_or(DecodeError::Truncated)?;
        if need > self.bytes.len() - self.offset {
            return Err(DecodeError::Truncated);
        }
        Ok(count)
    }
    fn entities(&mut self) -> Result<Vec<EntityState>, DecodeError> {
        let count = self.count(ENTITY_BYTES)?;
        let mut entities = Vec::with_capacity(count);
        for _ in 0..count {
            let id = self.u32()?;
            let tag = self.u8()?;
            if tag > 1 {
                return Err(DecodeError::InvalidOwnerTag(tag));
            }
            let owner = self.u32()?;
            if tag == 0 && owner != 0 {
                return Err(DecodeError::NonzeroAbsentOwner);
            }
            entities.push(EntityState {
                id,
                owner_id: (tag == 1).then_some(owner),
                pos_x: self.i64()?,
                pos_y: self.i64()?,
                vel_x: self.i64()?,
                vel_y: self.i64()?,
                energy: self.i64()?,
            });
        }
        check_order(entities.iter().map(|e| e.id))?;
        Ok(entities)
    }
}
