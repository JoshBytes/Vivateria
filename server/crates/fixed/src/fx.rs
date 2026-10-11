//! Q32.32 fixed-point number (`Fx`).

use crate::tables::{ATAN_UNIT, SIN_QUARTER};
use core::fmt;
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

const FRAC_BITS: u32 = 32;
const FRAC_MASK: i64 = (1i64 << FRAC_BITS) - 1;
/// Raw value of pi/2 (round(pi/2 * 2^32)); PI and TAU are exact multiples of it.
const FRAC_PI_2_RAW: i64 = 6_746_518_852;
/// Table resolution: 1024 segments, so the lookup phase has 10 bits of index
/// and `32 - 10 = 22` bits of interpolation fraction.
const INTERP_BITS: u32 = 22;
const INTERP_MASK: i64 = (1i64 << INTERP_BITS) - 1;

/// Saturate a wide intermediate into `i64`.
#[inline]
const fn sat(v: i128) -> i64 {
    if v > i64::MAX as i128 {
        i64::MAX
    } else if v < i64::MIN as i128 {
        i64::MIN
    } else {
        v as i64
    }
}

/// Floor of the square root of `n`, using only integer operations.
const fn isqrt_u128(mut n: u128) -> u128 {
    let mut res: u128 = 0;
    let mut bit: u128 = 1u128 << 126;
    while bit > n {
        bit >>= 2;
    }
    while bit != 0 {
        if n >= res + bit {
            n -= res + bit;
            res = (res >> 1) + bit;
        } else {
            res >>= 1;
        }
        bit >>= 2;
    }
    res
}

/// Linear interpolation in a 1025-entry table; `t` is Q32 in `[0, 1]`.
#[inline]
const fn interp(table: &[i64; 1025], t: i64) -> i64 {
    let idx = (t >> INTERP_BITS) as usize;
    if idx >= 1024 {
        return table[1024];
    }
    let frac = t & INTERP_MASK;
    let a = table[idx];
    let b = table[idx + 1];
    a + (((b - a) * frac) >> INTERP_BITS)
}

/// Q32.32 signed fixed-point number stored in an `i64`.
///
/// All operators saturate at [`Fx::MIN`]/[`Fx::MAX`], identically in debug and
/// release builds. `checked_*` and `wrapping_*` variants are also provided.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[repr(transparent)]
pub struct Fx(i64);

impl Fx {
    pub const ZERO: Fx = Fx(0);
    pub const ONE: Fx = Fx(1 << FRAC_BITS);
    pub const HALF: Fx = Fx(1 << (FRAC_BITS - 1));
    pub const MIN: Fx = Fx(i64::MIN);
    pub const MAX: Fx = Fx(i64::MAX);
    /// Smallest positive value (raw 1, 2^-32).
    pub const EPSILON: Fx = Fx(1);
    pub const PI: Fx = Fx(FRAC_PI_2_RAW * 2);
    pub const TAU: Fx = Fx(FRAC_PI_2_RAW * 4);
    pub const FRAC_PI_2: Fx = Fx(FRAC_PI_2_RAW);

    #[inline]
    pub const fn from_raw(raw: i64) -> Fx {
        Fx(raw)
    }

    #[inline]
    pub const fn to_raw(self) -> i64 {
        self.0
    }

    #[inline]
    pub const fn from_int(v: i32) -> Fx {
        Fx((v as i64) << FRAC_BITS)
    }

    /// Floor (toward negative infinity) as an integer. Always fits in `i32`.
    #[inline]
    pub const fn to_int_floor(self) -> i32 {
        (self.0 >> FRAC_BITS) as i32
    }

    /// Round to nearest integer, ties toward positive infinity; clamped to `i32`.
    #[inline]
    pub const fn to_int_round(self) -> i32 {
        let v = (self.0 as i128 + (1i128 << (FRAC_BITS - 1))) >> FRAC_BITS;
        if v > i32::MAX as i128 {
            i32::MAX
        } else {
            v as i32
        }
    }

    /// `num / den` as a fixed-point value (division semantics, see [`Fx::div`]).
    #[inline]
    pub const fn from_ratio(num: i64, den: i64) -> Fx {
        Fx::from_int_i64(num).div(Fx::from_int_i64(den))
    }

    #[inline]
    const fn from_int_i64(v: i64) -> Fx {
        Fx(sat((v as i128) << FRAC_BITS))
    }

    /// Absolute value; `MIN` saturates to `MAX`.
    #[inline]
    pub const fn abs(self) -> Fx {
        Fx(self.0.saturating_abs())
    }

    /// -ONE, ZERO or ONE.
    #[inline]
    pub const fn signum(self) -> Fx {
        if self.0 > 0 {
            Fx::ONE
        } else if self.0 < 0 {
            Fx(-(1 << FRAC_BITS))
        } else {
            Fx::ZERO
        }
    }

    /// Largest integer value `<= self`.
    #[inline]
    pub const fn floor(self) -> Fx {
        Fx(self.0 & !FRAC_MASK)
    }

    /// Smallest integer value `>= self` (saturating).
    #[inline]
    pub const fn ceil(self) -> Fx {
        let f = self.floor();
        if self.0 & FRAC_MASK == 0 {
            f
        } else {
            Fx(f.0.saturating_add(1 << FRAC_BITS))
        }
    }

    /// `self - floor(self)`, always in `[0, 1)`.
    #[inline]
    pub const fn frac(self) -> Fx {
        Fx(self.0 & FRAC_MASK)
    }

    #[inline]
    pub const fn min(self, other: Fx) -> Fx {
        if self.0 <= other.0 {
            self
        } else {
            other
        }
    }

    #[inline]
    pub const fn max(self, other: Fx) -> Fx {
        if self.0 >= other.0 {
            self
        } else {
            other
        }
    }

    /// Clamp into `[lo, hi]`. Never panics; if `lo > hi` the result is `hi`.
    #[inline]
    pub const fn clamp(self, lo: Fx, hi: Fx) -> Fx {
        self.max(lo).min(hi)
    }

    // ---- add / sub / neg ----

    #[inline]
    pub const fn checked_add(self, rhs: Fx) -> Option<Fx> {
        match self.0.checked_add(rhs.0) {
            Some(v) => Some(Fx(v)),
            None => None,
        }
    }

    #[inline]
    pub const fn checked_sub(self, rhs: Fx) -> Option<Fx> {
        match self.0.checked_sub(rhs.0) {
            Some(v) => Some(Fx(v)),
            None => None,
        }
    }

    #[inline]
    pub const fn wrapping_add(self, rhs: Fx) -> Fx {
        Fx(self.0.wrapping_add(rhs.0))
    }

    #[inline]
    pub const fn wrapping_sub(self, rhs: Fx) -> Fx {
        Fx(self.0.wrapping_sub(rhs.0))
    }

    #[inline]
    pub const fn wrapping_neg(self) -> Fx {
        Fx(self.0.wrapping_neg())
    }

    #[inline]
    pub const fn saturating_add(self, rhs: Fx) -> Fx {
        Fx(self.0.saturating_add(rhs.0))
    }

    #[inline]
    pub const fn saturating_sub(self, rhs: Fx) -> Fx {
        Fx(self.0.saturating_sub(rhs.0))
    }

    // ---- mul ----

    #[inline]
    const fn mul_wide(self, rhs: Fx) -> i128 {
        ((self.0 as i128) * (rhs.0 as i128)) >> FRAC_BITS
    }

    /// Saturating multiply. Uses an `i128` intermediate and an arithmetic shift,
    /// so the result is rounded toward negative infinity.
    #[inline]
    pub const fn mul(self, rhs: Fx) -> Fx {
        Fx(sat(self.mul_wide(rhs)))
    }

    #[inline]
    pub const fn checked_mul(self, rhs: Fx) -> Option<Fx> {
        let v = self.mul_wide(rhs);
        if v > i64::MAX as i128 || v < i64::MIN as i128 {
            None
        } else {
            Some(Fx(v as i64))
        }
    }

    /// Multiply, truncating the result to the low 64 bits on overflow.
    #[inline]
    pub const fn wrapping_mul(self, rhs: Fx) -> Fx {
        Fx(self.mul_wide(rhs) as i64)
    }

    // ---- div ----

    /// Saturating divide, truncating toward zero.
    ///
    /// Division by zero does not panic: it returns [`Fx::MAX`] for a numerator
    /// `>= 0` and [`Fx::MIN`] for a negative numerator.
    #[inline]
    pub const fn div(self, rhs: Fx) -> Fx {
        if rhs.0 == 0 {
            return if self.0 >= 0 { Fx::MAX } else { Fx::MIN };
        }
        Fx(sat(((self.0 as i128) << FRAC_BITS) / (rhs.0 as i128)))
    }

    /// Returns `None` on division by zero or if the result overflows.
    #[inline]
    pub const fn checked_div(self, rhs: Fx) -> Option<Fx> {
        if rhs.0 == 0 {
            return None;
        }
        let v = ((self.0 as i128) << FRAC_BITS) / (rhs.0 as i128);
        if v > i64::MAX as i128 || v < i64::MIN as i128 {
            None
        } else {
            Some(Fx(v as i64))
        }
    }

    /// Divide, truncating the result to the low 64 bits on overflow.
    /// Division by zero behaves like [`Fx::div`].
    #[inline]
    pub const fn wrapping_div(self, rhs: Fx) -> Fx {
        if rhs.0 == 0 {
            return self.div(rhs);
        }
        Fx((((self.0 as i128) << FRAC_BITS) / (rhs.0 as i128)) as i64)
    }

    /// Remainder with the sign of the dividend. A zero divisor returns `ZERO`.
    #[inline]
    pub const fn rem(self, rhs: Fx) -> Fx {
        if rhs.0 == 0 {
            Fx::ZERO
        } else {
            Fx(self.0.wrapping_rem(rhs.0))
        }
    }

    // ---- sqrt ----

    /// Exact `floor(sqrt(self))` in Q32.32. Negative input returns `ZERO`.
    pub const fn sqrt(self) -> Fx {
        if self.0 <= 0 {
            return Fx::ZERO;
        }
        // sqrt(raw / 2^32) * 2^32 = sqrt(raw * 2^32)
        let n = (self.0 as u128) << FRAC_BITS;
        Fx(isqrt_u128(n) as i64)
    }

    // ---- trig ----

    /// Sine of `(self + phase)` radians, with exact integer range reduction.
    const fn sin_phase(self, phase: i64) -> Fx {
        let tau = Fx::TAU.0 as i128;
        let r = ((self.0 as i128) + (phase as i128)).rem_euclid(tau) as i64;
        let q = r / FRAC_PI_2_RAW;
        let rem = r % FRAC_PI_2_RAW;
        let rising = Fx(Fx::sin_quarter(rem));
        let falling = Fx(Fx::sin_quarter(FRAC_PI_2_RAW - rem));
        match q {
            0 => rising,
            1 => falling,
            2 => Fx(-rising.0),
            _ => Fx(-falling.0),
        }
    }

    /// sin over the first quadrant, `rem` in `[0, pi/2]` (raw).
    const fn sin_quarter(rem: i64) -> i64 {
        let t = (((rem as i128) << FRAC_BITS) / (FRAC_PI_2_RAW as i128)) as i64;
        interp(&SIN_QUARTER, t)
    }

    /// Sine; any input range (reduced modulo TAU with integer math).
    pub const fn sin(self) -> Fx {
        self.sin_phase(0)
    }

    /// Cosine; any input range.
    pub const fn cos(self) -> Fx {
        self.sin_phase(FRAC_PI_2_RAW)
    }

    /// atan for `t` in `[0, 1]` (Q32), returns raw radians.
    const fn atan_unit(t: Fx) -> i64 {
        interp(&ATAN_UNIT, t.0)
    }

    /// Arctangent, result in `[-pi/2, pi/2]`.
    pub const fn atan(self) -> Fx {
        Fx::atan2(self, Fx::ONE)
    }

    /// Four-quadrant arctangent of `y / x`, result in `[-pi, pi]`.
    /// `atan2(0, 0)` is `ZERO`; `atan2(0, x<0)` is `PI`.
    pub const fn atan2(y: Fx, x: Fx) -> Fx {
        if y.0 == 0 && x.0 == 0 {
            return Fx::ZERO;
        }
        let a = y.abs();
        let b = x.abs();
        let mut ang = if a.0 <= b.0 {
            Fx::atan_unit(a.div(b))
        } else {
            FRAC_PI_2_RAW - Fx::atan_unit(b.div(a))
        };
        if x.0 < 0 {
            ang = 2 * FRAC_PI_2_RAW - ang;
        }
        if y.0 < 0 {
            ang = -ang;
        }
        Fx(ang)
    }
}

impl fmt::Display for Fx {
    /// Decimal with 9 fractional digits (truncated toward zero), integer math only.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mag = self.0.unsigned_abs();
        let int = mag >> FRAC_BITS;
        let frac = ((mag & (FRAC_MASK as u64)) * 1_000_000_000) >> FRAC_BITS;
        let sign = if self.0 < 0 { "-" } else { "" };
        write!(f, "{sign}{int}.{frac:09}")
    }
}

macro_rules! binop {
    ($tr:ident, $m:ident, $atr:ident, $am:ident) => {
        impl $tr for Fx {
            type Output = Fx;
            #[inline]
            fn $m(self, rhs: Fx) -> Fx {
                Fx::$m(self, rhs)
            }
        }
        impl $atr for Fx {
            #[inline]
            fn $am(&mut self, rhs: Fx) {
                *self = Fx::$m(*self, rhs);
            }
        }
    };
}

impl Fx {
    #[inline]
    const fn add(self, rhs: Fx) -> Fx {
        self.saturating_add(rhs)
    }
    #[inline]
    const fn sub(self, rhs: Fx) -> Fx {
        self.saturating_sub(rhs)
    }
}

binop!(Add, add, AddAssign, add_assign);
binop!(Sub, sub, SubAssign, sub_assign);
binop!(Mul, mul, MulAssign, mul_assign);
binop!(Div, div, DivAssign, div_assign);
binop!(Rem, rem, RemAssign, rem_assign);

impl Neg for Fx {
    type Output = Fx;
    /// Saturating negation (`-MIN == MAX`).
    #[inline]
    fn neg(self) -> Fx {
        Fx(self.0.saturating_neg())
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_types, clippy::float_arithmetic)]
mod tests {
    use super::*;

    fn fx(n: i32) -> Fx {
        Fx::from_int(n)
    }

    fn to_f64(v: Fx) -> f64 {
        v.0 as f64 / 4294967296.0
    }

    #[test]
    fn constants() {
        assert_eq!(Fx::ONE.to_raw(), 1 << 32);
        assert_eq!(Fx::HALF + Fx::HALF, Fx::ONE);
        assert_eq!(Fx::EPSILON.to_raw(), 1);
        assert_eq!(Fx::PI, Fx::FRAC_PI_2 * fx(2));
        assert_eq!(Fx::TAU, Fx::PI * fx(2));
        assert!((to_f64(Fx::PI) - core::f64::consts::PI).abs() < 1e-9);
        assert!((to_f64(Fx::TAU) - core::f64::consts::TAU).abs() < 1e-9);
        assert!((to_f64(Fx::FRAC_PI_2) - core::f64::consts::FRAC_PI_2).abs() < 1e-9);
    }

    #[test]
    fn conversions() {
        assert_eq!(fx(7).to_int_floor(), 7);
        assert_eq!(fx(-7).to_int_floor(), -7);
        assert_eq!((fx(-1) + Fx::HALF).to_int_floor(), -1);
        assert_eq!(Fx::HALF.to_int_round(), 1);
        assert_eq!((Fx::HALF - Fx::EPSILON).to_int_round(), 0);
        assert_eq!((-Fx::HALF).to_int_round(), 0);
        assert_eq!(fx(-3).to_int_round(), -3);
        assert_eq!(Fx::MAX.to_int_round(), i32::MAX);
        assert_eq!(Fx::MIN.to_int_round(), i32::MIN);
        assert_eq!(Fx::MIN.to_int_floor(), i32::MIN);
        assert_eq!(fx(i32::MAX).to_int_floor(), i32::MAX);
        assert_eq!(fx(i32::MIN).to_int_floor(), i32::MIN);
        assert_eq!(Fx::from_ratio(1, 2), Fx::HALF);
        assert_eq!(Fx::from_ratio(-3, 2).to_raw(), -(3 << 31));
        assert_eq!(Fx::from_ratio(1, 0), Fx::MAX);
        assert_eq!(Fx::from_ratio(-1, 0), Fx::MIN);
    }

    #[test]
    fn floor_ceil_frac_abs_signum() {
        let v = Fx::from_ratio(-5, 4); // -1.25
        assert_eq!(v.floor(), fx(-2));
        assert_eq!(v.ceil(), fx(-1));
        assert_eq!(v.frac(), Fx::from_ratio(3, 4));
        assert_eq!(v.floor() + v.frac(), v);
        assert_eq!(fx(3).ceil(), fx(3));
        assert_eq!(fx(3).floor(), fx(3));
        assert_eq!(v.abs(), Fx::from_ratio(5, 4));
        assert_eq!(Fx::MIN.abs(), Fx::MAX);
        assert_eq!(v.signum(), fx(-1));
        assert_eq!(fx(9).signum(), Fx::ONE);
        assert_eq!(Fx::ZERO.signum(), Fx::ZERO);
        assert_eq!(Fx::MAX.ceil(), Fx::MAX);
    }

    #[test]
    fn min_max_clamp() {
        assert_eq!(fx(1).min(fx(2)), fx(1));
        assert_eq!(fx(1).max(fx(2)), fx(2));
        assert_eq!(fx(5).clamp(fx(0), fx(3)), fx(3));
        assert_eq!(fx(-5).clamp(fx(0), fx(3)), fx(0));
        assert_eq!(fx(2).clamp(fx(0), fx(3)), fx(2));
        assert_eq!(fx(2).clamp(fx(3), fx(0)), fx(0)); // inverted bounds: no panic
    }

    #[test]
    fn arithmetic_identities() {
        let a = Fx::from_ratio(7, 3);
        let b = Fx::from_ratio(-11, 5);
        assert_eq!(a + Fx::ZERO, a);
        assert_eq!(a * Fx::ONE, a);
        assert_eq!(a * Fx::ZERO, Fx::ZERO);
        assert_eq!(a / Fx::ONE, a);
        assert_eq!(a - a, Fx::ZERO);
        assert_eq!(a + b, b + a);
        assert_eq!(a * b, b * a);
        assert_eq!((a + b) - b, a);
        assert_eq!(-(-a), a);
        assert_eq!(fx(6) / fx(3), fx(2));
        assert_eq!(fx(-6) / fx(3), fx(-2));
        assert_eq!(fx(7) % fx(3), fx(1));
        assert_eq!(fx(-7) % fx(3), fx(-1));
        assert_eq!(fx(7) % Fx::ZERO, Fx::ZERO);
        assert_eq!(Fx::MIN % fx(-1), Fx::ZERO);
        let mut c = fx(10);
        c += fx(2);
        c -= fx(4);
        c *= fx(3);
        c /= fx(2);
        c %= fx(8);
        assert_eq!(c, fx(4));
    }

    #[test]
    fn mul_rounds_toward_neg_infinity() {
        let e = Fx::EPSILON;
        // e * HALF = 2^-33 -> floor -> 0
        assert_eq!(e * Fx::HALF, Fx::ZERO);
        // -e * HALF = -2^-33 -> floor -> -e
        assert_eq!((-e) * Fx::HALF, -e);
        assert_eq!(fx(-1) * Fx::HALF, Fx::from_ratio(-1, 2));
        assert_eq!(Fx::HALF * Fx::HALF, Fx::from_ratio(1, 4));
        assert_eq!(fx(-2) * fx(-3), fx(6));
    }

    #[test]
    fn div_truncates_toward_zero() {
        let third = Fx::from_ratio(1, 3);
        assert_eq!(third.to_raw(), (1i64 << 32) / 3);
        assert_eq!(Fx::from_ratio(-1, 3).to_raw(), -((1i64 << 32) / 3));
        assert_eq!(Fx::EPSILON / fx(2), Fx::ZERO);
        assert_eq!((-Fx::EPSILON) / fx(2), Fx::ZERO);
    }

    #[test]
    fn saturation() {
        assert_eq!(Fx::MAX + Fx::EPSILON, Fx::MAX);
        assert_eq!(Fx::MAX + Fx::MAX, Fx::MAX);
        assert_eq!(Fx::MIN - Fx::EPSILON, Fx::MIN);
        assert_eq!(Fx::MIN + Fx::MIN, Fx::MIN);
        assert_eq!(-Fx::MIN, Fx::MAX);
        assert_eq!(Fx::MAX * fx(2), Fx::MAX);
        assert_eq!(Fx::MIN * fx(2), Fx::MIN);
        assert_eq!(Fx::MIN * fx(-2), Fx::MAX);
        assert_eq!(Fx::MIN * Fx::MIN, Fx::MAX);
        assert_eq!(Fx::MAX / Fx::HALF, Fx::MAX);
        assert_eq!(Fx::MIN / Fx::HALF, Fx::MIN);
        assert_eq!(Fx::MIN / fx(-1), Fx::MAX);
        assert_eq!(fx(i32::MAX) + fx(i32::MAX), Fx::MAX);
    }

    #[test]
    fn div_by_zero() {
        assert_eq!(fx(5) / Fx::ZERO, Fx::MAX);
        assert_eq!(Fx::ZERO / Fx::ZERO, Fx::MAX);
        assert_eq!(fx(-5) / Fx::ZERO, Fx::MIN);
        assert_eq!(fx(5).checked_div(Fx::ZERO), None);
        assert_eq!(fx(6).checked_div(fx(3)), Some(fx(2)));
        assert_eq!(Fx::MAX.checked_div(Fx::HALF), None);
        assert_eq!(fx(5).wrapping_div(Fx::ZERO), Fx::MAX);
    }

    #[test]
    fn checked_and_wrapping() {
        assert_eq!(Fx::MAX.checked_add(Fx::EPSILON), None);
        assert_eq!(Fx::MIN.checked_sub(Fx::EPSILON), None);
        assert_eq!(Fx::MAX.checked_mul(fx(2)), None);
        assert_eq!(fx(2).checked_add(fx(3)), Some(fx(5)));
        assert_eq!(fx(2).checked_sub(fx(3)), Some(fx(-1)));
        assert_eq!(fx(2).checked_mul(fx(3)), Some(fx(6)));
        assert_eq!(Fx::MAX.wrapping_add(Fx::EPSILON), Fx::MIN);
        assert_eq!(Fx::MIN.wrapping_sub(Fx::EPSILON), Fx::MAX);
        assert_eq!(Fx::MIN.wrapping_neg(), Fx::MIN);
        // MAX * 2 = (2^63 - 1) * 2 / 2^32 * ... computed in i128, then truncated
        let wide = ((i64::MAX as i128) * ((2i128) << 32)) >> 32;
        assert_eq!(Fx::MAX.wrapping_mul(fx(2)).to_raw(), wide as i64);
        assert_eq!(fx(3).wrapping_mul(fx(4)), fx(12));
        assert_eq!(Fx::MAX.wrapping_div(Fx::HALF).to_raw(), -2);
    }

    #[test]
    fn sqrt_exact() {
        assert_eq!(Fx::ZERO.sqrt(), Fx::ZERO);
        assert_eq!(fx(-4).sqrt(), Fx::ZERO);
        assert_eq!(Fx::MIN.sqrt(), Fx::ZERO);
        assert_eq!(fx(1).sqrt(), fx(1));
        assert_eq!(fx(4).sqrt(), fx(2));
        assert_eq!(fx(144).sqrt(), fx(12));
        assert_eq!(Fx::from_ratio(1, 4).sqrt(), Fx::HALF);
        // floor(sqrt(2) * 2^32) = 6074000999
        assert_eq!(fx(2).sqrt().to_raw(), 6_074_000_999);
        // floor property over a sweep: r^2 <= n*2^32 < (r+1)^2
        let mut raw: i64 = 1;
        while raw < i64::MAX / 3 {
            let r = Fx::from_raw(raw).sqrt().to_raw() as u128;
            let n = (raw as u128) << 32;
            assert!(r * r <= n && n < (r + 1) * (r + 1), "raw {raw}");
            raw = raw * 3 + 1;
        }
        let r = Fx::MAX.sqrt().to_raw() as u128;
        let n = (i64::MAX as u128) << 32;
        assert!(r * r <= n && n < (r + 1) * (r + 1));
    }

    #[test]
    fn sin_cos_accuracy() {
        let mut worst: f64 = 0.0;
        let mut raw: i64 = -(12 << 32);
        while raw <= (12 << 32) {
            let x = Fx::from_raw(raw);
            let xf = to_f64(x);
            worst = worst.max((to_f64(x.sin()) - xf.sin()).abs());
            worst = worst.max((to_f64(x.cos()) - xf.cos()).abs());
            raw += 1_234_567; // ~0.00029 rad
        }
        assert!(worst <= 1e-6, "worst sin/cos error {worst}");
    }

    #[test]
    fn sin_cos_special_and_huge() {
        assert_eq!(Fx::ZERO.sin(), Fx::ZERO);
        assert_eq!(Fx::ZERO.cos(), Fx::ONE);
        assert_eq!(Fx::FRAC_PI_2.sin(), Fx::ONE);
        assert!(Fx::PI.sin().abs() < Fx::from_raw(8));
        assert_eq!(Fx::PI.cos().to_raw(), -(1 << 32));
        // extreme inputs must not panic and must stay in [-1, 1]
        for x in [Fx::MAX, Fx::MIN, Fx::from_raw(i64::MAX - 1)] {
            assert!(x.sin().abs() <= Fx::ONE);
            assert!(x.cos().abs() <= Fx::ONE);
        }
        // periodicity
        let x = Fx::from_ratio(7, 10);
        assert_eq!((x + Fx::TAU).sin(), x.sin());
        assert_eq!((x - Fx::TAU).cos(), x.cos());
    }

    #[test]
    fn sin_squared_plus_cos_squared() {
        let mut raw: i64 = -(7 << 32);
        while raw <= (7 << 32) {
            let x = Fx::from_raw(raw);
            let s = x.sin();
            let c = x.cos();
            let sum = to_f64(s * s + c * c);
            assert!((sum - 1.0).abs() < 4e-6, "x raw {raw}: {sum}");
            raw += 98_765_431;
        }
    }

    #[test]
    fn atan2_accuracy() {
        let mut worst: f64 = 0.0;
        for yi in -40..=40 {
            for xi in -40..=40 {
                let y = Fx::from_ratio(yi, 8);
                let x = Fx::from_ratio(xi, 8);
                if yi == 0 && xi == 0 {
                    assert_eq!(Fx::atan2(y, x), Fx::ZERO);
                    continue;
                }
                let got = to_f64(Fx::atan2(y, x));
                let want = to_f64(y).atan2(to_f64(x));
                worst = worst.max((got - want).abs());
            }
        }
        // extreme ratios
        for (y, x) in [(1i64, 1_000_000i64), (1_000_000, 1), (-1, -1_000_000)] {
            let (yf, xf) = (Fx::from_int(y as i32), Fx::from_int(x as i32));
            let got = to_f64(Fx::atan2(yf, xf));
            let want = (y as f64).atan2(x as f64);
            worst = worst.max((got - want).abs());
        }
        assert!(worst <= 1e-5, "worst atan2 error {worst}");
    }

    #[test]
    fn atan_values() {
        assert_eq!(Fx::ZERO.atan(), Fx::ZERO);
        let one = to_f64(Fx::ONE.atan());
        assert!((one - core::f64::consts::FRAC_PI_4).abs() < 1e-5);
        let neg = to_f64(fx(-1).atan());
        assert!((neg + core::f64::consts::FRAC_PI_4).abs() < 1e-5);
        assert!((to_f64(Fx::MAX.atan()) - core::f64::consts::FRAC_PI_2).abs() < 1e-5);
        assert_eq!(Fx::atan2(Fx::ZERO, fx(-1)), Fx::PI);
        assert_eq!(Fx::atan2(fx(1), Fx::ZERO), Fx::FRAC_PI_2);
        assert_eq!(Fx::atan2(fx(-1), Fx::ZERO), -Fx::FRAC_PI_2);
    }

    #[test]
    fn table_accuracy() {
        use crate::tables::{ATAN_UNIT, SIN_QUARTER};
        let scale = 4294967296.0f64;
        for i in 0..=1024usize {
            let t = i as f64 / 1024.0;
            let s = SIN_QUARTER[i] as f64 / scale;
            let a = ATAN_UNIT[i] as f64 / scale;
            assert!((s - (t * core::f64::consts::FRAC_PI_2).sin()).abs() < 1e-9);
            assert!((a - t.atan()).abs() < 1e-9);
        }
    }

    #[test]
    fn display() {
        assert_eq!(Fx::ZERO.to_string(), "0.000000000");
        assert_eq!(fx(3).to_string(), "3.000000000");
        assert_eq!(Fx::HALF.to_string(), "0.500000000");
        assert_eq!(fx(-2).to_string(), "-2.000000000");
        assert_eq!(Fx::from_ratio(-5, 4).to_string(), "-1.250000000");
        assert_eq!(Fx::from_ratio(1, 3).to_string(), "0.333333333");
        assert_eq!(Fx::MIN.to_string(), "-2147483648.000000000");
    }
}
