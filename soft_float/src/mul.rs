use std::ops::Mul;

use crate::soft_float16::{NAN, NEG_INFINITY, NEG_ZERO, POS_INFINITY, POS_ZERO};
use crate::SoftFloat16;

impl Mul for SoftFloat16 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
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
            // NAN * _ or _ * NAN
            return NAN;
        } else if ((exponent0 == 0x1F && exponent1 == 0) || (exponent0 == 0 && exponent1 == 0x1F))
            && significand0 == 0
            && significand1 == 0
        {
            // oo * 0 or 0 * oo
            return NAN;
        } else if (exponent0 == 0x1F && significand0 == 0)
            || (exponent1 == 0x1F && significand1 == 0)
        {
            // oo * _ or _ * oo
            return if sign0 ^ sign1 == 0 {
                POS_INFINITY
            } else {
                NEG_INFINITY
            };
        } else if (exponent0 == 0 && significand0 == 0) || exponent1 == 0 && significand1 == 0 {
            // 0 * _ or _ * 0
            return if sign0 ^ sign1 == 0 {
                POS_ZERO
            } else {
                NEG_ZERO
            };
        }

        let sign = sign0 ^ sign1;

        // handle denormals and implicit bit
        let (exponent0, significand0) = if exponent0 == 0 {
            // normalize
            let mut exponent = 1_i16;
            let mut significand = significand0;
            assert!(significand != 0);
            while significand & (1 << 10) == 0 {
                significand <<= 1;
                exponent -= 1;
            }
            (exponent, significand)
        } else {
            (exponent0 as i16, significand0 | 0x400)
        };

        let (exponent1, significand1) = if exponent1 == 0 {
            // normalize
            let mut exponent = 1_i16;
            let mut significand = significand1;
            assert!(significand != 0);
            while significand & (1 << 10) == 0 {
                significand <<= 1;
                exponent -= 1;
            }
            (exponent, significand)
        } else {
            (exponent1 as i16, significand1 | 0x400)
        };

        let exponent = (exponent0 + exponent1) as i16 - 15;

        let significand = (significand0 as u32) * (significand1 as u32);
        assert!(significand < (1 << (12 + 10)));
        assert!((significand & (1 << (11 + 10))) != 0 | (significand & (1 << (10 + 10))));

        // keep lowest three bits for guard, round, sticky bits
        let sticky_bits = (1 << (10 - 3)) - 1;
        let sticky = (significand & sticky_bits != 0) as u16;
        let significand = ((significand >> (10 - 3)) as u16) | sticky;

        let (exponent, significand) = if significand & (1 << (11 + 3)) != 0 {
            // realign decimal point
            let sticky = (significand & 1) | sticky;
            let significand = (significand >> 1) | sticky;
            (exponent + 1, significand)
        } else {
            assert!((significand & (1 << (10 + 3))) != 0);
            (exponent, significand)
        };

        let (exponent, significand) = if exponent <= -11 {
            // underflow, since shift would be >= 12, i.e., all bits and guard
            // bit would be zero; will always be rounded to zero
            return if sign == 0 { POS_ZERO } else { NEG_ZERO };
        } else if exponent <= 0 {
            // must convert to denormal number; make exponent representable by
            // shifting significand
            let shift = 1 - exponent;
            let sticky_bits = (1 << shift) - 1;
            let sticky = (significand & sticky_bits != 0) as u16;
            let significand = (significand >> shift) | sticky;
            (1, significand)
        } else if exponent >= 0x1F {
            // overflow
            return if sign == 0 {
                POS_INFINITY
            } else {
                NEG_INFINITY
            };
        } else {
            (exponent as u16, significand)
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

        // cut off implicit bit and allow overflow into exponent
        Self::from_bits(sign << 15 | (exponent << 10 | significand & 0x3FF) + rnd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul() {
        for (v0, v1) in [
            (0x200, 0x200),
            (0x3c04, 0x3c04),
            (513, 5117),
            (1025, 4095),
            (1025, 16383),
            (1057, 14305),
            (15362, 31742),
            (16384, 30721),
            (1, 17537),
            (15362, 31742),
        ] {
            let x0 = SoftFloat16::from_bits(v0);
            let x1 = SoftFloat16::from_bits(v1);
            let y = x0 * x1;
            let y_f = SoftFloat16::from(f32::from(x0) * f32::from(x1));
            assert_eq!(SoftFloat16::to_bits(y), SoftFloat16::to_bits(y_f));
        }
    }

    #[test]
    #[ignore]
    fn test_all_mul() {
        for i in 0..u16::MAX {
            for j in 0..u16::MAX {
                let x0_sf = SoftFloat16::from_bits(i);
                let x1_sf = SoftFloat16::from_bits(j);
                let y_sf = x0_sf * x1_sf;
                let y_f = SoftFloat16::from(f32::from(x0_sf) * f32::from(x1_sf));
                if y_sf == NAN || y_f == NAN {
                    assert!(y_sf == NAN, "{:?}", (i, j));
                    assert!(y_f == NAN, "{:?}", (i, j));
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
