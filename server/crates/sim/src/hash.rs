//! Hand-written FNV-1a 64-bit hash over explicit byte encodings.

const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// Streaming FNV-1a 64-bit hasher. Feed it canonical little-endian bytes only.
#[derive(Clone, Debug)]
pub struct Fnv1a64(u64);

impl Default for Fnv1a64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Fnv1a64 {
    pub const fn new() -> Self {
        Fnv1a64(OFFSET_BASIS)
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(PRIME);
        }
    }

    pub fn write_u8(&mut self, v: u8) {
        self.write(&[v]);
    }

    pub fn write_u32(&mut self, v: u32) {
        self.write(&v.to_le_bytes());
    }

    pub fn write_u64(&mut self, v: u64) {
        self.write(&v.to_le_bytes());
    }

    pub fn write_i64(&mut self, v: i64) {
        self.write(&v.to_le_bytes());
    }

    pub const fn finish(&self) -> u64 {
        self.0
    }
}

/// One-shot FNV-1a 64 of a byte slice.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = Fnv1a64::new();
    h.write(bytes);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn streaming_matches_one_shot() {
        let mut h = Fnv1a64::new();
        h.write(b"foo");
        h.write(b"bar");
        assert_eq!(h.finish(), fnv1a64(b"foobar"));
    }
}
