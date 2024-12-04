use std::cmp;
use std::ops::Add;

use crate::soft_float16::{NAN, NEG_INFINITY, NEG_ZERO, POS_INFINITY, POS_ZERO};
use crate::SoftFloat16;

impl Add for SoftFloat16 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
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

        if (exponent0 == 0x1F && significand0 != 0) || (exponent1 == 0x1F && significand1 != 0) {
            // NAN + _ or _ + NAN
            return NAN;
        } else if (exponent0 == 0 && significand0 == 0) & (exponent1 == 0 && significand1 == 0) {
            // 0 + 0
            return if sign0 & sign1 == 0 {
                POS_ZERO
            } else {
                NEG_ZERO
            };
        } else if exponent0 == 0 && significand0 == 0 {
            // 0 + _
            return other;
        } else if exponent1 == 0 && significand1 == 0 {
            // _ + 0
            return self;
        } else if (exponent0 == 0x1F && significand0 == 0)
            && (exponent1 == 0x1F && significand1 == 0)
        {
            // oo + oo
            return if sign0 == sign1 { self } else { NAN };
        } else if exponent0 == 0x1F && significand0 == 0 {
            // oo + _
            return self;
        } else if exponent1 == 0x1F && significand1 == 0 {
            // _ + oo
            return other;
        };

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
        let sticky_bits = (1 << shift) - 1;
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
                // realign decimal point
                let sticky = (significand & 1) | sticky;
                let significand = (significand >> 1) | sticky;
                (sign, exponent + 1, significand)
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
                // (try to) realign decimal point
                significand <<= 1;
                exponent -= 1;
            }

            if significand & (1 << (10 + 3)) == 0 && exponent > 1 {
                // continue to (try to) realign decimal point if several leading
                // digits have been canceled; this can only happen for
                // - two normal numbers; cancellation occurs -> implicit bits
                // need to be lined up -> shift must be 0 or 1 -> at most the
                // guard bit is nonzero
                // - two denormal numbers; have same exponent, so GRS bits are
                // zero
                // in both cases we don't need to worry about sticky bit being
                // shifted back into significand
                assert!(SoftFloat16::exponent(self) != 0);
                assert!(SoftFloat16::exponent(other) != 0);
                assert!(shift <= 1);

                // (11 + 3)th bit should be one, with a 16bit significand, we
                // thus want 2 leading zeros
                let clz = SoftFloat16::clz(significand);
                assert!(clz > 2);
                let shift = cmp::min(exponent - 1, clz - 2);
                significand <<= shift;
                exponent -= shift;
            }

            (sign, exponent, significand)
        };

        assert!(exponent > 0);
        if exponent >= 0x1F {
            // overflow
            return if sign == 0 {
                POS_INFINITY
            } else {
                NEG_INFINITY
            };
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

        // cut off implicit bit and allow into exponent
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
            if y == NAN || y_f == NAN {
                assert!(y == NAN, "{:?}", (v0, v1));
                assert!(y_f == NAN, "{:?}", (v0, v1));
            } else {
                assert_eq!(
                    SoftFloat16::to_bits(y),
                    SoftFloat16::to_bits(y_f),
                    "\n{:?}\n  {:016b}\n  {:016b}",
                    (v0, v1),
                    SoftFloat16::to_bits(y),
                    SoftFloat16::to_bits(y_f),
                );
            }
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
