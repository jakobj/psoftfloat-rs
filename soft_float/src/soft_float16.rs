use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct SoftFloat16(u16);

pub const POS_INFINITY: SoftFloat16 = SoftFloat16(0x7c00);
pub const NEG_INFINITY: SoftFloat16 = SoftFloat16(0xfc00);
pub const NAN: SoftFloat16 = SoftFloat16(0x7e00);
pub const POS_ZERO: SoftFloat16 = SoftFloat16(0x0);
pub const NEG_ZERO: SoftFloat16 = SoftFloat16(0x8000);

impl SoftFloat16 {
    pub fn clz(v: u16) -> u16 {
        if v == 0 {
            16
        } else {
            // branchless binary search
            // compare https://stackoverflow.com/a/10866821
            let mut n = 0;
            let mut v = v;
            let mut s;
            s = ((v & 0xFF00 == 0) as u16) << 3;
            n += s;
            v <<= s;
            s = ((v & 0xF000 == 0) as u16) << 2;
            n += s;
            v <<= s;
            s = ((v & 0xC000 == 0) as u16) << 1;
            n += s;
            v <<= s;
            s = (v & 0x8000 == 0) as u16;
            n += s;
            n
        }
    }

    pub fn from_bits(v: u16) -> Self {
        let v = Self(v);
        // map all possible NANs to a single representation
        if Self::exponent(v) == 0x1F && Self::significand(v) != 0 {
            NAN
        } else {
            v
        }
    }

    pub fn to_bits(v: Self) -> u16 {
        v.0
    }

    pub fn sign(v: Self) -> u16 {
        v.0 >> 15
    }

    pub fn exponent(v: Self) -> u16 {
        (v.0 >> 10) & 0x1F
    }

    pub fn significand(v: Self) -> u16 {
        v.0 & 0x3FF
    }
}

impl fmt::Display for SoftFloat16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f32::from(*self))
        // if self.exponent == 0 {
        //     if self.significand == 0 {
        //         // zero
        //         return write!(f, "{}", (-1.0_f64).powi(self.sign as i32) * 0.0);
        //     } else {
        //         // subnormal
        //         // +- (d0 beta^{-1} + d1 beta^{-2} + ... + d_{p-1} beta^{-p}) beta^{-14}
        //         // = +- ((d0 beta^{p-1} + d1 beta^{p-2} + ... + d_{p-1} beta^0) beta^{-p}) beta^{-14}
        //         // = +- significand beta^{-p-14}
        //         return write!(
        //             f,
        //             "{}",
        //             (-1.0_f64).powi(self.sign as i32)
        //                 * (self.significand as f64)
        //                 * 2_f64.powi(-10 - 14)
        //         );
        //     }
        // }
        // if self.exponent == 0x1F {
        //     if self.significand == 0 {
        //         // infinity
        //         return write!(f, "inf");
        //     } else {
        //         return write!(f, "NaN");
        //     };
        // }

        // let unbiased_exponent = SoftFloat16::unbiased_exponent(self);
        // let significand = self.significand | 0x400; // include the implicit bit
        //                                             // +- (d0 + d1 beta^{-1} + ... + d_{p-1} beta^{-(p-1)}) beta^e
        //                                             // = +- ((d0 beta^{p-1} + d1 beta^{p-2} + ... + d_{p-1} beta^0) beta^{-(p-1)}) beta^e
        //                                             // = +- significand beta^{-(p-1) + e}
        // let value = (-1.0_f64).powi(self.sign as i32)
        //     * (significand as f64)
        //     * 2_f64.powi((-10 + unbiased_exponent) as i32);
        // write!(f, "{}", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_display() {
    //     let x = SoftFloat16::new(0, 14, 659);
    //     assert_eq!(format!("{}", x), "0.82177734375");
    // }

    #[test]
    fn test_all_clz() {
        for i in 0..u16::MAX {
            let x = SoftFloat16::from_bits(i);
            assert_eq!(SoftFloat16::clz(x.0), x.0.leading_zeros() as u16);
        }
    }
}
