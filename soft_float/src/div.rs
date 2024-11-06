use std::ops::Div;

use crate::soft_float16::{NAN, NEG_INFINITY, NEG_ZERO, POS_INFINITY, POS_ZERO};
use crate::SoftFloat16;

impl Div for SoftFloat16 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (NAN, _) => return NAN,
            (_, NAN) => return NAN,
            (POS_ZERO, POS_ZERO) => return NAN,
            (POS_ZERO, NEG_ZERO) => return NAN,
            (POS_ZERO, POS_INFINITY) => return POS_ZERO,
            (POS_ZERO, NEG_INFINITY) => return NEG_ZERO,
            (NEG_ZERO, POS_ZERO) => return NAN,
            (NEG_ZERO, NEG_ZERO) => return NAN,
            (NEG_ZERO, POS_INFINITY) => return NEG_ZERO,
            (NEG_ZERO, NEG_INFINITY) => return POS_ZERO,
            _ => (),
        }

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

        match (self, other) {
            (POS_ZERO, _) => return if sign1 == 0 { POS_ZERO } else { NEG_ZERO },
            (NEG_ZERO, _) => return if sign1 == 0 { NEG_ZERO } else { POS_ZERO },
            (_, POS_ZERO) => return if sign0 == 0 { POS_INFINITY } else { NEG_INFINITY },
            (_, NEG_ZERO) => return if sign0 == 0 { NEG_INFINITY } else { POS_INFINITY },
            _ => (),
        }

        let sign = sign0 ^ sign1;

        println!("{}|{:024b} <- before normalize 0", exponent0, significand0);
        println!("{}|{:024b} <- before normalize 1", exponent1, significand1);

        // handle denormals and implicit bit
        let (exponent0, significand0) = if exponent0 == 0 {
            // normalize
            let mut exponent = 1_i16;
            let mut significand = significand0;
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
            while significand & (1 << 10) == 0 {
                significand <<= 1;
                exponent -= 1;
            }
            (exponent, significand)
        } else {
            (exponent1 as i16, significand1 | 0x400)
        };
        println!("{:02}|{:024b} <- after normalize 0", exponent0, significand0);
        println!("{:02}|{:024b} <- after normalize 1", exponent1, significand1);

        let exponent = (exponent0 - exponent1) + 15;
        // TODO assert exponent range

        // generate quotient one bit at a time using long division
        let mut x = significand0;
        let y = significand1;
        let mut r = 0;
        for _ in 0..(10 + 3) {
            if x >= y {
                x = x - y;
                r = r | 1;
            }
            x <<= 1;
            r <<= 1;
        }
        let sticky = (x != 0) as u16;
        let significand = r | sticky;
        assert!(significand < (0x800 << 3));
        assert!(significand >= (0x100 << 3));

        println!("{:02}|{:024b} <- after div", exponent, significand);

        assert!(exponent > 0);
        let (exponent, significand) = if significand & (0x400 << 3) == 0 {
            ((exponent - 1) as u16, significand << 1)
        } else {
            (exponent as u16, significand)
        };

        println!("{:02}|{:024b} <- after shift", exponent, significand);

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
    fn test_div() {
        for (v0, v1) in [
            (0x3c10, 0x3410),
            (0x24ff, 0x75f),
            (0x24ff, 0x11),
            (0x400, 0x7ff),
            (0x07ff, 0x400),
            (0x07ff, 0x350),
            (0x1, 0x3),
            (0x8, 0xab8),
        ] {
            let x0 = SoftFloat16::from_bits(v0);
            let x1 = SoftFloat16::from_bits(v1);
            let y_sf = x0 / x1;
            let y_f = SoftFloat16::from(f32::from(x0) / f32::from(x1));
            println!("{:017b} <- sf", SoftFloat16::to_bits(y_sf));
            println!("{:017b} <-  f", SoftFloat16::to_bits(y_f));
            assert_eq!(SoftFloat16::to_bits(y_sf), SoftFloat16::to_bits(y_f));
        }
    }

    #[test]
    #[ignore]
    fn test_all_div() {
        for i in 0..u16::MAX {
            for j in 0..u16::MAX {
                let x0_sf = SoftFloat16::from_bits(i);
                let x1_sf = SoftFloat16::from_bits(j);
                let y_sf = x0_sf / x1_sf;
                let y_f = SoftFloat16::from(f32::from(x0_sf) / f32::from(x1_sf));
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
