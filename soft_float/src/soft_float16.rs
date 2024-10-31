use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct SoftFloat16(u16);

pub const POS_INFINITY: SoftFloat16 = SoftFloat16(0x7c00);
pub const NEG_INFINITY: SoftFloat16 = SoftFloat16(0xfc00);
pub const NAN: SoftFloat16 = SoftFloat16(0x7e00);
pub const POS_ZERO: SoftFloat16 = SoftFloat16(0x0);
pub const NEG_ZERO: SoftFloat16 = SoftFloat16(0x8000);

impl SoftFloat16 {
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

impl Div for SoftFloat16 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        todo!()
    }
}

impl Neg for SoftFloat16 {
    type Output = Self;

    fn neg(self) -> Self {
        if self == NAN {
            NAN
        } else {
            Self(self.0 ^ (1 << 15))
        }
    }
}

impl Sub for SoftFloat16 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

fn decompose_f32(v: f32) -> String {
    let bits = v.to_bits();
    let sign = (bits >> 31) as u16;
    let exponent = (bits >> 23) & 0xFF;
    let significand = bits & 0x7FFFFF;
    format!("{:01b}|{:08b}|{:023b}", sign, exponent, significand)
}

fn decompose_soft_float16(v: SoftFloat16) -> String {
    format!(
        "{:01b}|{:05b}|{:010b}",
        SoftFloat16::sign(v),
        SoftFloat16::exponent(v),
        SoftFloat16::significand(v)
    )
}

// impl Convert<f32> for SoftFloat16 {
//     fn convert(value: f32) -> Self {
//         let bits = value.to_bits();

//         // extract fields using corresponding bitmasks
//         let original_sign = bits & 0x80000000;
//         let original_exponent = bits & 0x7f800000;
//         let original_significand = bits & 0x7fffff; // without implicit bit

//         let sign = (original_sign >> 16) as u16;

//         // TODO create one big if else to make sure all cases are covered
//         if original_exponent == 0 {
//             // zero or subnormal; largest subnormal in f32 is smaller than smallest subnormal in f16, so we don't need to handle that case separately
//             return Self(sign | 0x0 | 0x0);
//         } else if original_exponent == 0x7f800000 {
//             if original_significand == 0 {
//                 // infinity
//                 return Self(sign | 0x7c00 | 0x0);
//             } else {
//                 // nan, using msb of significand
//                 return Self(sign | 0x7c00 | 1 << 9);
//             }
//         };

//         // adjust exponent
//         let unbiased_exponent = (original_exponent >> 23) as i32 - 127;
//         if unbiased_exponent < -14 {
//             // underflow
//             if unbiased_exponent >= -15 - 10 {
//                 // subnormal representation possible with shift \in [0, 10]
//                 let original_significand_wib = original_significand | 0x800000; // with implicit bit
//                 let shift = -(unbiased_exponent + 15); // NOTE 1 2^unbiased_exponent = (1 >> shift) 2^{-15}
//                 assert!(shift >= 0);
//                 // shift to match exponents of normal f32 and denormal f16
//                 // shift additionally by 14 to fit 23+1bits into 10bits
//                 let significand = ((original_significand_wib >> shift) >> 14) as u16;
//                 // check (n+1)th bit for rounding where n is the amount of shift
//                 let rnd_bit = 1 << (shift + 13);
//                 let lsb = original_significand_wib & (rnd_bit << 1);
//                 let rnd = original_significand_wib & rnd_bit;
//                 let rest = original_significand_wib & (rnd_bit - 1);
//                 if rnd == 0 || (rnd != 0 && rest == 0 && lsb == 0) {
//                     return Self(sign | 0x0 | significand);
//                 } else {
//                     // allow significand to overflow into exponent
//                     return Self(sign | (0x0 | significand) + 1);
//                 }
//             } else {
//                 // zero
//                 return Self(sign | 0x0 | 0x0);
//             }
//         } else if unbiased_exponent > 15 {
//             // overflow
//             return Self(sign | 0x1F << 10 | 0);
//         }
//         let exponent = ((unbiased_exponent + 15) << 10) as u16;

//         // adjust significand
//         // shift by 13 to fit 23bits into 10bits
//         let significand = (original_significand >> 13) as u16;
//         // check (n+1)th bit for rounding where n is the amount of shift
//         let rnd_bit = 1 << 12;
//         let lsb = original_significand & (rnd_bit << 1);
//         let rnd = original_significand & rnd_bit;
//         let rest = original_significand & (rnd_bit - 1);
//         if rnd == 0 || (rnd != 0 && rest == 0 && lsb == 0) {
//             Self(sign | exponent | significand)
//         } else {
//             // allow significand to overflow into exponent
//             Self(sign | (exponent | significand) + 1)
//         }
//     }
// }

// impl Convert<f64> for SoftFloat16 {
//     fn convert(value: f64) -> Self {
//         Self::convert(value as f32)
//     }
// }

// impl Convert<SoftFloat16> for SoftFloat16 {
//     fn convert(value: SoftFloat16) -> Self {
//         value
//     }
// }

// impl Float for SoftFloat16 {
//     fn abs(self) -> Self {
//         todo!()
//     }
//     fn exp(self) -> Self {
//         todo!()
//     }
//     fn powf(self, n: Self) -> Self {
//         todo!()
//     }
//     fn powi(self, n: i32) -> Self {
//         todo!()
//     }
//     fn sqrt(self) -> Self {
//         todo!()
//     }
// }

impl fmt::Display for SoftFloat16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!();
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

// impl fmt::LowerHex for SoftFloat16 {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         fmt::LowerHex::fmt(&SoftFloat16::to_bits(*self), f)
//     }
// }

impl From<f32> for SoftFloat16 {
    fn from(value: f32) -> Self {
        todo!()
    }
}

impl From<SoftFloat16> for f32 {
    fn from(value: SoftFloat16) -> Self {
        todo!()
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

    // #[test]
    // fn test_from_f32() {
    //     for (v, expected) in [
    //         (0xBFFFFFCF, 0xc000),
    //         (0x7FFF0007, 0x7e00),
    //         (0xBFFC1000, 0xbfe0),
    //         (0x7F800000, 0x7c00),
    //         (0x00000002, 0x0000),
    //         (0x33FFFFFF, 0x0002),
    //         (0x34000000, 0x0002),
    //         (0xB4FF8003, 0x8008),
    //         (0x387fe000, 0x0400),
    //         (0x33ffe000, 0x0002),
    //         (0x33bfff7e, 0x0001),
    //         (0x387fc000, 0x03ff),
    //         (0x38000000, 0x0200),
    //         (0x33000001, 0x0001),
    //         (0x477749c0, 0x7bba),
    //     ] {
    //         let x = SoftFloat16::convert(f32::from_bits(v));
    //         assert_eq!(SoftFloat16::to_bits(x), expected);
    //     }
    // }

    // #[test]
    // fn test_all_from_f32() {
    //     for i in 0..u32::MAX {
    //         let v = f32::from_bits(i);
    //         let x = SoftFloat16::convert(v);
    //         let y = Float16::convert(v);
    //         if x == NAN {
    //             assert!(Float16::is_nan(y));
    //         } else {
    //             assert_eq!(SoftFloat16::to_bits(x), Float16::to_bits(y));
    //         }
    //     }
    // }

    // #[test]
    // fn test_all_neg() {
    //     for i in 0..u16::MAX {
    //         let x_sf = SoftFloat16::from_bits(i);
    //         let y_sf = -x_sf;
    //         let x_f = Float16::from_bits(i);
    //         let y_f = -x_f;
    //         if y_sf == NAN {
    //             assert!(Float16::is_nan(y_f));
    //         } else {
    //             assert_eq!(SoftFloat16::to_bits(y_sf), Float16::to_bits(y_f),);
    //         }
    //     }
    // }

    // #[test]
    // fn test_all_sub() {
    //     for i in 0..u16::MAX {
    //         for j in 0..u16::MAX {
    //             let x0_sf = SoftFloat16::from_bits(i);
    //             let x1_sf = SoftFloat16::from_bits(j);
    //             let y_sf = x0_sf - x1_sf;
    //             let x0_f = Float16::from_bits(i);
    //             let x1_f = Float16::from_bits(j);
    //             let y_f = x0_f - x1_f;
    //             if y_sf == NAN {
    //                 assert!(Float16::is_nan(y_f));
    //             } else {
    //                 assert_eq!(SoftFloat16::to_bits(y_sf), Float16::to_bits(y_f),);
    //             }
    //         }
    //     }
    // }
}
