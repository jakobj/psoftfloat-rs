use std::ops::Add;

use crate::soft_float16::{NAN, NEG_INFINITY, NEG_ZERO, POS_INFINITY, POS_ZERO};
use crate::SoftFloat16;

impl Add for SoftFloat16 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (NAN, _) => return NAN,
            (_, NAN) => return NAN,
            (POS_ZERO, POS_ZERO) => return POS_ZERO,
            (POS_ZERO, NEG_ZERO) => return POS_ZERO,
            (NEG_ZERO, POS_ZERO) => return POS_ZERO,
            (NEG_ZERO, NEG_ZERO) => return NEG_ZERO,
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

        let shift = exponent0 - exponent1;

        // if shifting operation would throw out all bits (even beyond guard,
        // round, sticky) it's like adding zero, so we can just return early
        if shift >= 13 {
            return Self::from_bits(sign0 << 15 | exponent0 << 10 | significand0 & 0x3FF);
        }

        // insert guard, round, sticky bits
        let significand0 = significand0 << 3;
        let significand1 = significand1 << 3;
        let sticky_bits = (1 << (shift + 3 - 2)) - 1;
        let sticky = if significand1 & sticky_bits == 0 {
            0
        } else {
            1
        };

        // align decimal point of second number
        let significand1 = (significand1 >> shift) | sticky;

        // if signs are equal add significands, otherwise subtract
        let (sign, exponent, significand) = if sign0 == sign1 {
            let (sign, exponent, significand) = (sign0, exponent0, significand0 + significand1);
            if significand & (1 << (11 + 3)) == 0 {
                (sign, exponent, significand)
            } else {
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
            }
        } else {
            // always subtract smaller from larger significand and pick
            // corresponding sign
            let (sign, mut exponent, mut significand) = if significand0 >= significand1 {
                (sign0, exponent0, significand0 - significand1)
            } else {
                (sign1, exponent0, significand1 - significand0)
            };
            while significand & (1 << (10 + 3)) == 0 && exponent > 1 {
                // realign decimal point
                significand <<= 1;
                exponent -= 1;
            }
            (sign, exponent, significand)
        };

        // rounding
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

        if exponent == 1 && significand < 0x400 {
            // denormal number
            Self::from_bits(sign << 15 | significand + rnd)
        } else {
            // normal number
            Self::from_bits(sign << 15 | (exponent << 10 | significand & 0x3FF) + rnd)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        for ((v0, v1), expected) in [
            ((0x87FF, 0xE850), 0xE850),
            ((0x0000, 0x857F), 0x857F),
            ((0x74FB, 0xE879), 0x746C),
            ((0x7978, 0x0001), 0x7978),
            ((0x0000, 0x0000), 0x0000),
            ((0xC19A, 0xCFEB), 0xD04F),
            ((0x0200, 0x0200), 0x0400),
            ((0x0301, 0x0101), 0x0402),
            ((30721, 30721), 31744),
            ((1025, 34816), 33791),
            ((32768, 0), 0),
            ((32769, 1), 0),
        ] {
            let x0 = SoftFloat16::from_bits(v0);
            let x1 = SoftFloat16::from_bits(v1);
            let y = x0 + x1;
            assert_eq!(SoftFloat16::to_bits(y), expected);
        }
    }

    #[test]
    #[ignore]
    fn test_all_add() {
        for i in 0..u16::MAX {
            for j in 0..u16::MAX {
                let x0_sf = SoftFloat16::from_bits(i);
                let x1_sf = SoftFloat16::from_bits(j);
                let y_sf = x0_sf + x1_sf;
                let y_f = SoftFloat16::from(f32::from(x0_sf) + f32::from(x1_sf));
                if y_sf == NAN || y_f == NAN {
                    assert!(y_sf == NAN);
                    assert!(y_f == NAN);
                } else {
                    assert_eq!(
                        SoftFloat16::to_bits(y_sf),
                        SoftFloat16::to_bits(y_f),
                        "{:?}",
                        (i, j)
                    );
                }
            }
        }
    }
}
