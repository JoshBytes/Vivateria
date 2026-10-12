//! Player inputs and the replayable input log.

use std::fmt;

/// Identifier of a player.
pub type PlayerId = u32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PlayerInput {
    pub player: PlayerId,
    /// Clamped to -1..=1 when applied.
    pub move_x: i8,
    pub move_y: i8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DecodeError {
    BadMagic,
    UnsupportedVersion(u8),
    Truncated,
    TrailingBytes,
    /// Entries must be in non-decreasing tick order.
    Unordered,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::BadMagic => write!(f, "bad input log magic"),
            DecodeError::UnsupportedVersion(v) => write!(f, "unsupported input log version {v}"),
            DecodeError::Truncated => write!(f, "input log truncated"),
            DecodeError::TrailingBytes => write!(f, "trailing bytes after input log"),
            DecodeError::Unordered => write!(f, "input log ticks not ordered"),
        }
    }
}

impl std::error::Error for DecodeError {}

const MAGIC: [u8; 4] = *b"VINP";
const VERSION: u8 = 1;
const HEADER_LEN: usize = 9;
const ENTRY_LEN: usize = 8 + 4 + 1 + 1;

/// Ordered list of `(tick, input)`; order within a tick is log order.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct InputLog {
    entries: Vec<(u64, PlayerInput)>,
}

impl InputLog {
    pub fn new() -> InputLog {
        InputLog::default()
    }

    /// Build from entries; stable-sorted by tick so log order within a tick is kept.
    pub fn from_entries(mut entries: Vec<(u64, PlayerInput)>) -> InputLog {
        entries.sort_by_key(|e| e.0);
        InputLog { entries }
    }

    pub fn entries(&self) -> &[(u64, PlayerInput)] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Append an input; if `tick` is earlier than the last entry the log is re-sorted stably.
    pub fn push(&mut self, tick: u64, input: PlayerInput) {
        let needs_sort = self.entries.last().is_some_and(|l| l.0 > tick);
        self.entries.push((tick, input));
        if needs_sort {
            self.entries.sort_by_key(|e| e.0);
        }
    }

    /// Inputs recorded for `tick`, in log order.
    pub fn inputs_for(&self, tick: u64) -> Vec<PlayerInput> {
        let start = self.entries.partition_point(|e| e.0 < tick);
        self.entries[start..]
            .iter()
            .take_while(|e| e.0 == tick)
            .map(|e| e.1)
            .collect()
    }

    /// Format: magic "VINP", version u8, count u32 LE, then per entry
    /// tick u64 LE, player u32 LE, move_x i8, move_y i8.
    ///
    /// # Panics
    /// Panics if the log holds more than `u32::MAX` entries (format limit).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN + self.entries.len() * ENTRY_LEN);
        out.extend_from_slice(&MAGIC);
        out.push(VERSION);
        out.extend_from_slice(
            &u32::try_from(self.entries.len())
                .expect("input log exceeds u32::MAX entries")
                .to_le_bytes(),
        );
        for (tick, inp) in &self.entries {
            out.extend_from_slice(&tick.to_le_bytes());
            out.extend_from_slice(&inp.player.to_le_bytes());
            out.extend_from_slice(&inp.move_x.to_le_bytes());
            out.extend_from_slice(&inp.move_y.to_le_bytes());
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<InputLog, DecodeError> {
        if bytes.len() >= 4 && bytes[..4] != MAGIC {
            return Err(DecodeError::BadMagic);
        }
        if bytes.len() < HEADER_LEN {
            return Err(DecodeError::Truncated);
        }
        if bytes[4] != VERSION {
            return Err(DecodeError::UnsupportedVersion(bytes[4]));
        }
        let mut cnt = [0u8; 4];
        cnt.copy_from_slice(&bytes[5..9]);
        let count = u32::from_le_bytes(cnt) as usize;
        let body = &bytes[HEADER_LEN..];
        let need = count.checked_mul(ENTRY_LEN).ok_or(DecodeError::Truncated)?;
        if body.len() < need {
            return Err(DecodeError::Truncated);
        }
        if body.len() > need {
            return Err(DecodeError::TrailingBytes);
        }
        let mut entries = Vec::with_capacity(count);
        let mut last = 0u64;
        let (chunks, _) = body.as_chunks::<ENTRY_LEN>();
        for chunk in chunks {
            let mut t = [0u8; 8];
            t.copy_from_slice(&chunk[0..8]);
            let mut p = [0u8; 4];
            p.copy_from_slice(&chunk[8..12]);
            let tick = u64::from_le_bytes(t);
            if tick < last {
                return Err(DecodeError::Unordered);
            }
            last = tick;
            entries.push((
                tick,
                PlayerInput {
                    player: u32::from_le_bytes(p),
                    move_x: i8::from_le_bytes([chunk[12]]),
                    move_y: i8::from_le_bytes([chunk[13]]),
                },
            ));
        }
        Ok(InputLog { entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(player: u32, x: i8, y: i8) -> PlayerInput {
        PlayerInput {
            player,
            move_x: x,
            move_y: y,
        }
    }

    #[test]
    fn roundtrip_and_lookup() {
        let log = InputLog::from_entries(vec![
            (3, inp(1, 1, 0)),
            (3, inp(0, -1, 1)),
            (7, inp(2, 0, 0)),
        ]);
        let back = InputLog::decode(&log.encode()).unwrap();
        assert_eq!(log, back);
        assert_eq!(back.inputs_for(3), vec![inp(1, 1, 0), inp(0, -1, 1)]);
        assert!(back.inputs_for(4).is_empty());
        assert_eq!(back.inputs_for(7).len(), 1);
    }

    #[test]
    fn decode_errors() {
        let enc = InputLog::from_entries(vec![(1, inp(1, 1, 1))]).encode();
        assert_eq!(
            InputLog::decode(&enc[..enc.len() - 1]),
            Err(DecodeError::Truncated)
        );
        let mut bad = enc.clone();
        bad[0] = b'X';
        assert_eq!(InputLog::decode(&bad), Err(DecodeError::BadMagic));
        let mut ver = enc.clone();
        ver[4] = 9;
        assert_eq!(
            InputLog::decode(&ver),
            Err(DecodeError::UnsupportedVersion(9))
        );
        let mut extra = enc;
        extra.push(0);
        assert_eq!(InputLog::decode(&extra), Err(DecodeError::TrailingBytes));
    }
}
