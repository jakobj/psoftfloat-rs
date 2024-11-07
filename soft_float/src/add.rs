use std::ops::Add;

use crate::soft_float16::{NAN, NEG_INFINITY, NEG_ZERO, POS_INFINITY, POS_ZERO};
use crate::SoftFloat16;

impl Add for SoftFloat16 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (NAN, _) => return NAN,
            (_, NAN) => return NAN,
            (POS_ZERO, NEG_ZERO) => return POS_ZERO,
            (POS_ZERO, _) => return other,
            (NEG_ZERO, _) => return other,
            (_, POS_ZERO) => return self,
            (_, NEG_ZERO) => return self,
            (POS_INFINITY, NEG_INFINITY) => return NAN,
            (NEG_INFINITY, POS_INFINITY) => return NAN,
            (POS_INFINITY, _) => return POS_INFINITY,
            (NEG_INFINITY, _) => return NEG_INFINITY,
            (_, POS_INFINITY) => return POS_INFINITY,
            (_, NEG_INFINITY) => return NEG_INFINITY,
            _ => (),
        };

        let (sign0, exponent0, significand0) = (
            Self::sign(self),
            Self::exponent(self),
            Self::significand(self),
        );
        let (sign1, exponent1, significand1) = (
            Self::sign(other),
            Self::exponent(other),
            Self::significand(other),
        );

        // numbers only differ in sign, so result is POS_ZERO
        if sign0 != sign1 && exponent0 == exponent1 && significand0 == significand1 {
            return POS_ZERO;
        }

        // handle denormals and implicit bit
        let (exponent0, significand0) = if exponent0 == 0 {
            (1, significand0)
        } else {
            (exponent0, significand0 | 0x400)
        };

        let (exponent1, significand1) = if exponent1 == 0 {
            (1, significand1)
        } else {
            (exponent1, significand1 | 0x400)
        };

        // make sure that first number has larger or equal exponent to make
        // subsequent logic easier
        let (sign0, exponent0, significand0, sign1, exponent1, significand1) =
            if exponent0 >= exponent1 {
                (
                    sign0,
                    exponent0,
                    significand0,
                    sign1,
                    exponent1,
                    significand1,
                )
            } else {
                (
                    sign1,
                    exponent1,
                    significand1,
                    sign0,
                    exponent0,
                    significand0,
                )
            };

        let shift = (exponent0 - exponent1) as u16;

        // insert guard, round, sticky bits
        let significand0 = significand0 << 3;
        let significand1 = significand1 << 3;
        let sticky_bits = (1 << (shift + 1)) - 1;
        let sticky = ((significand1 & sticky_bits) != 0) as u16;

        // align decimal point of second number
        let significand1 = if shift < 16 {
            (significand1 >> shift) | sticky
        } else {
            0
        };

        // if signs are equal add significands, otherwise subtract
        let (sign, exponent, significand) = if sign0 == sign1 {
            let (sign, exponent, significand) = (sign0, exponent0, significand0 + significand1);
            assert!(significand < (1 << (12 + 3)));

            if significand & (1 << (11 + 3)) != 0 {
                // need to realign decimal point
                let significand = (significand >> 1) | sticky;
                let exponent = exponent + 1;
                if exponent >= 0x1F {
                    // overflow
                    return if sign == 0 {
                        POS_INFINITY
                    } else {
                        NEG_INFINITY
                    };
                }
                (sign, exponent, significand)
            } else {
                (sign, exponent, significand)
            }
        } else {
            // always subtract smaller from larger significand and pick
            // corresponding sign
            let (sign, mut exponent, mut significand) = if significand0 >= significand1 {
                (sign0, exponent0, significand0 - significand1)
            } else {
                (sign1, exponent0, significand1 - significand0)
            };
            assert!(significand < (1 << (11 + 3)));

            if significand & (1 << (10 + 3)) == 0 && exponent > 1 {
                // need to realign decimal point
                significand <<= 1;
                exponent -= 1;
            }

            if significand & (1 << (10 + 3)) == 0 && exponent > 1 {
                // continue to realign decimal point if several leading digits
                // have been canceled; this can only happen for two normal
                // numbers; cancellation occurs -> implicit bits need to be
                // lined up -> shift must be 0 or 1 -> at most the guard bit is
                // nonzero, so we don't need to worry about round and sticky
                // bits
                assert!(SoftFloat16::exponent(self) != 0);
                assert!(SoftFloat16::exponent(other) != 0);
                assert!(shift <= 1);
                while significand & (1 << (10 + 3)) == 0 && exponent > 1 {
                    significand <<= 1;
                    exponent -= 1;
                }
            }

            (sign, exponent, significand)
        };

        // rounding (to even)
        let grs = significand & 0x7;
        let significand = significand >> 3;
        let lsb = significand & 1;
        let rnd = if grs < 0x4 || (grs == 0x4 && lsb == 0) {
            // round down
            0
        } else {
            // round up
            1
        };

        // denormals have exponent 0
        let exponent = if exponent == 1 && significand < 0x400 {
            0
        } else {
            exponent
        };

        Self::from_bits(sign << 15 | (exponent << 10 | significand & 0x3FF) + rnd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        for (v0, v1) in [
            (4143, 5311),
            (0x77, 0x15),
            (0x14bf, 0x142f),
            (0x87FF, 0xE850),
            (0x0000, 0x857F),
            (0x74FB, 0xE879),
            (0x7978, 0x0001),
            (0x0000, 0x0000),
            (0xC19A, 0xCFEB),
            (0x0200, 0x0200),
            (0x0301, 0x0101),
            (30721, 30721),
            (1025, 34816),
            (32768, 0),
            (32769, 1),
        ] {
            let x0 = SoftFloat16::from_bits(v0);
            let x1 = SoftFloat16::from_bits(v1);
            let y = x0 + x1;
            let y_f = SoftFloat16::from(f32::from(x0) + f32::from(x1));
            assert_eq!(SoftFloat16::to_bits(y), SoftFloat16::to_bits(y_f));
        }
    }

    #[test]
    #[ignore]
    fn test_all_add() {
        for i in 0..u16::MAX {
            for j in 0..u16::MAX {
                let x0 = SoftFloat16::from_bits(i);
                let x1 = SoftFloat16::from_bits(j);
                let y = x0 + x1;
                let y_f = SoftFloat16::from(f32::from(x0) + f32::from(x1));
                if y == NAN || y_f == NAN {
                    assert!(y == NAN, "{:?}", (i, j));
                    assert!(y_f == NAN, "{:?}", (i, j));
                } else {
                    assert_eq!(
                        SoftFloat16::to_bits(y),
                        SoftFloat16::to_bits(y_f),
                        "{:?}",
                        (i, j)
                    );
                }
            }
        }
    }
}
