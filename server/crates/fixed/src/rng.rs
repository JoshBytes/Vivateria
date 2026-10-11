//! PCG32 (PCG-XSH-RR 64/32) deterministic random number generator.

use crate::Fx;

const MULTIPLIER: u64 = 6_364_136_223_846_793_005;

/// PCG32 generator: 64-bit state, 32-bit output. Seeded explicitly; never
/// reads the clock or OS entropy.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    /// Standard PCG32 seeding (`pcg32_srandom_r`).
    pub fn new(seed: u64, stream: u64) -> Pcg32 {
        let mut rng = Pcg32 {
            state: 0,
            inc: (stream << 1) | 1,
        };
        rng.step();
        rng.state = rng.state.wrapping_add(seed);
        rng.step();
        rng
    }

    /// Rebuild a generator from words returned by [`Pcg32::state`].
    /// `inc` is forced odd, as in every valid generator.
    pub fn from_state(state: u64, inc: u64) -> Pcg32 {
        Pcg32 {
            state,
            inc: inc | 1,
        }
    }

    /// Raw `(state, inc)` words, for hashing and serialization.
    pub fn state(&self) -> (u64, u64) {
        (self.state, self.inc)
    }

    #[inline]
    fn step(&mut self) {
        self.state = self.state.wrapping_mul(MULTIPLIER).wrapping_add(self.inc);
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.step();
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Two outputs combined, first output in the high word.
    pub fn next_u64(&mut self) -> u64 {
        let hi = u64::from(self.next_u32());
        let lo = u64::from(self.next_u32());
        (hi << 32) | lo
    }

    /// Unbiased value in `[0, bound)` (rejection sampling). `bound == 0` returns 0
    /// without consuming any output.
    pub fn range_u32(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            return 0;
        }
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let r = self.next_u32();
            if r >= threshold {
                return r % bound;
            }
        }
    }

    /// Unbiased value in the half-open range `[lo, hi)`. If `hi <= lo` returns `lo`.
    pub fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (i64::from(hi) - i64::from(lo)) as u32; // <= 2^32 - 1
        let off = self.range_u32(span);
        (i64::from(lo) + i64::from(off)) as i32
    }

    /// Uniform `Fx` in `[0, 1)` with 32 bits of resolution.
    pub fn next_fx_unit(&mut self) -> Fx {
        Fx::from_raw(i64::from(self.next_u32()))
    }

    /// Uniform `Fx` in `[lo, hi)`. If `hi <= lo` returns `lo`.
    pub fn fx_range(&mut self, lo: Fx, hi: Fx) -> Fx {
        if hi <= lo {
            return lo;
        }
        let span = hi.saturating_sub(lo);
        let u = self.next_fx_unit();
        lo.saturating_add(span * u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_vectors() {
        let mut rng = Pcg32::new(42, 54);
        let want = [
            0xa15c02b7u32,
            0x7b47f409,
            0xba1d3330,
            0x83d2f293,
            0xbfa4784b,
            0xcbed606e,
        ];
        for w in want {
            assert_eq!(rng.next_u32(), w);
        }
    }

    #[test]
    fn next_u64_combines_outputs() {
        let mut a = Pcg32::new(42, 54);
        assert_eq!(a.next_u64(), 0xa15c02b7_7b47f409);
    }

    #[test]
    fn state_roundtrip() {
        let mut a = Pcg32::new(1, 2);
        a.next_u32();
        let (s, i) = a.state();
        let mut b = Pcg32::from_state(s, i);
        assert_eq!(a, b);
        assert_eq!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn streams_differ() {
        let mut a = Pcg32::new(7, 1);
        let mut b = Pcg32::new(7, 2);
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn range_u32_bounds_and_coverage() {
        let mut rng = Pcg32::new(9, 9);
        assert_eq!(rng.range_u32(0), 0);
        assert_eq!(rng.range_u32(1), 0);
        let mut seen = [false; 7];
        for _ in 0..1000 {
            let v = rng.range_u32(7);
            assert!(v < 7);
            seen[v as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
        for _ in 0..1000 {
            assert!(rng.range_u32(u32::MAX) < u32::MAX);
        }
    }

    #[test]
    fn range_i32_bounds() {
        let mut rng = Pcg32::new(3, 4);
        for _ in 0..1000 {
            let v = rng.range_i32(-5, 5);
            assert!((-5..5).contains(&v));
        }
        assert_eq!(rng.range_i32(4, 4), 4);
        assert_eq!(rng.range_i32(4, 1), 4);
        for _ in 0..100 {
            let _ = rng.range_i32(i32::MIN, i32::MAX);
            let v = rng.range_i32(i32::MAX - 1, i32::MAX);
            assert_eq!(v, i32::MAX - 1);
        }
    }

    #[test]
    fn fx_ranges() {
        let mut rng = Pcg32::new(5, 6);
        for _ in 0..1000 {
            let u = rng.next_fx_unit();
            assert!(u >= Fx::ZERO && u < Fx::ONE);
            let v = rng.fx_range(Fx::from_int(-3), Fx::from_int(4));
            assert!(v >= Fx::from_int(-3) && v < Fx::from_int(4));
        }
        assert_eq!(rng.fx_range(Fx::ONE, Fx::ONE), Fx::ONE);
        // full-width range must not overflow/panic
        let _ = rng.fx_range(Fx::MIN, Fx::MAX);
    }

    #[test]
    fn determinism_checksum() {
        fn run() -> u64 {
            let mut rng = Pcg32::new(0xDEADBEEF, 17);
            let mut h: u64 = 0xcbf29ce484222325; // FNV-1a
            for _ in 0..10_000 {
                h ^= u64::from(rng.next_u32());
                h = h.wrapping_mul(0x100000001b3);
            }
            h
        }
        assert_eq!(run(), run());
        assert_eq!(run(), 0xe58b_11f4_393f_07bb);
    }
}
