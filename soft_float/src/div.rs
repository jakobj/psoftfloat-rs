use std::ops::Div;

use crate::soft_float16::{NAN, NEG_INFINITY, NEG_ZERO, POS_INFINITY, POS_ZERO};
use crate::SoftFloat16;

impl Div for SoftFloat16 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        // TODO handle NANs etc

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

        let sign = sign0 ^ sign1;

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

        let exponent = ((exponent0 - exponent1) as i16 + 15) as u16;
        // TODO assert exponent range

        // TODO large shift seems unnecessary, maybe we can get away with a shift of 3?
        let significand = ((significand0 as u32) << (10 + 3)) / (significand1 as u32);
        // if non-zero digits appear after the ones we have calculated, then z =
        // x/y => x = y * z will not hold, but rather x > y * z; we use this as
        // an easy way to determine the sticky bit
        let sticky = if (significand0 as u32) - ((significand1 as u32) * significand) == 0 { 0 } else { 1 };
        let significand = significand | sticky;
        println!("{:09b}|{:024b} <- after div", exponent, significand);
        // TODO normalize
        let (exponent, significand) = if significand & (1 << (11 + 3)) != 0 {
            let mut exponent = exponent;
            let mut significand = significand;
            let mut sticky;
            // // decimal point too far right
            // let exponent = exponent + 1;
            // let sticky = (significand & 1 != 0) as u32;
            // let significand = (significand >> 1) | sticky;
            while significand > (0x800 << 3) {
                sticky = significand & 1;
                significand = (significand >> 1) | sticky;
                exponent += 1;
            }
            (exponent, significand)
        } else {
            // decimal point too far left
            let mut exponent = exponent;
            let mut significand = significand;
            // TODO do we ever shift more than one place here?
            while significand & (1 << (10 + 3)) == 0 && exponent > 1 {
                significand <<= 1;
                exponent -= 1;
            }
            (exponent, significand)
        };
        println!("{:09b}|{:024b} <- after shift", exponent, significand);

        let significand = significand as u16;

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
            // ((0x3c10, 0x3410), 0x4400),
            // ((0x24ff, 0x75f), 0x596C),
            (0x24ff, 0x11),
            // (0x400, 0x7ff),
            // ((0x07ff, 0x400), 0x3fff),
            // (0x07ff, 0x350),
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

}
