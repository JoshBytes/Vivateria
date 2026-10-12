//! Minimal fixed-point 2D vector.

use std::ops::{Add, Sub};

use vivateria_fixed::Fx;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct FxVec2 {
    pub x: Fx,
    pub y: Fx,
}

impl Add for FxVec2 {
    type Output = FxVec2;
    fn add(self, o: FxVec2) -> FxVec2 {
        FxVec2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for FxVec2 {
    type Output = FxVec2;
    fn sub(self, o: FxVec2) -> FxVec2 {
        FxVec2::new(self.x - o.x, self.y - o.y)
    }
}

impl FxVec2 {
    pub const ZERO: FxVec2 = FxVec2 {
        x: Fx::ZERO,
        y: Fx::ZERO,
    };

    pub const fn new(x: Fx, y: Fx) -> FxVec2 {
        FxVec2 { x, y }
    }

    pub fn scale(self, k: Fx) -> FxVec2 {
        FxVec2::new(self.x * k, self.y * k)
    }

    pub fn length_sq(self) -> Fx {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> Fx {
        self.length_sq().sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_3_4_5() {
        let v = FxVec2::new(Fx::from_int(3), Fx::from_int(4));
        assert_eq!(v.length(), Fx::from_int(5));
        assert_eq!((v + v) - v, v);
        assert_eq!(v.scale(Fx::HALF).x, Fx::from_ratio(3, 2));
    }
}
