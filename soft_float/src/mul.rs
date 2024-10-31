use std::ops::Mul;

use crate::soft_float16::{NAN, NEG_INFINITY, NEG_ZERO, POS_INFINITY, POS_ZERO};
use crate::SoftFloat16;

impl Mul for SoftFloat16 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (NAN, _) => return NAN,
            (_, NAN) => return NAN,
            (POS_ZERO, POS_INFINITY) => return NAN,
            (NEG_ZERO, POS_INFINITY) => return NAN,
            (POS_ZERO, NEG_INFINITY) => return NAN,
            (NEG_ZERO, NEG_INFINITY) => return NAN,
            (POS_INFINITY, POS_ZERO) => return NAN,
            (POS_INFINITY, NEG_ZERO) => return NAN,
            (NEG_INFINITY, POS_ZERO) => return NAN,
            (NEG_INFINITY, NEG_ZERO) => return NAN,
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

        match (self, other) {
            (POS_INFINITY, _) => {
                return if sign1 == 0 {
                    POS_INFINITY
                } else {
                    NEG_INFINITY
                }
            }
            (NEG_INFINITY, _) => {
                return if sign1 == 0 {
                    NEG_INFINITY
                } else {
                    POS_INFINITY
                }
            }
            (_, POS_INFINITY) => {
                return if sign0 == 0 {
                    POS_INFINITY
                } else {
                    NEG_INFINITY
                }
            }
            (_, NEG_INFINITY) => {
                return if sign0 == 0 {
                    NEG_INFINITY
                } else {
                    POS_INFINITY
                }
            }
            (POS_ZERO, _) => return if sign1 == 0 { POS_ZERO } else { NEG_ZERO },
            (NEG_ZERO, _) => return if sign1 == 0 { NEG_ZERO } else { POS_ZERO },
            (_, POS_ZERO) => return if sign0 == 0 { POS_ZERO } else { NEG_ZERO },
            (_, NEG_ZERO) => return if sign0 == 0 { NEG_ZERO } else { POS_ZERO },
            _ => (),
        };

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

        let exponent = (exponent0 + exponent1) as i16 - 15;
        assert!(exponent >= -15);
        assert!(exponent <= 60);

        let significand = (significand0 as u32) * (significand1 as u32);
        assert!(significand < (1 << (12 + 10)));

        // (try to) normalize
        let (exponent, significand) = if significand & (1 << (11 + 10)) != 0 {
            // decimal point too far right
            let exponent = exponent + 1;
            let sticky = (significand & 1 != 0) as u32;
            let significand = (significand >> 1) | sticky;
            (exponent, significand)
        } else {
            // decimal point too far left
            let mut exponent = exponent;
            let mut significand = significand;
            while significand & (1 << (10 + 10)) == 0 && exponent > 1 {
                significand <<= 1;
                exponent -= 1;
            }
            (exponent, significand)
        };

        if exponent >= 0x1F {
            // overflow
            return if sign == 0 {
                POS_INFINITY
            } else {
                NEG_INFINITY
            };
        }

        // keep lowest three bits for guard, round, sticky bits
        let sticky_bits = (1 << (10 - 2)) - 1;
        let sticky = (significand & sticky_bits != 0) as u16;
        let significand = ((significand >> 7) as u16) | sticky;

        let (exponent, significand) = if exponent < -11 {
            // underflow
            return Self::from_bits(sign << 15);
        } else if exponent <= 0 {
            // make exponent representable by shifting significand
            let shift = 1 - exponent;
            let sticky_bits = (1 << (shift + 3 - 2)) - 1;
            let sticky = (significand & sticky_bits != 0) as u16;
            let significand = (significand >> shift) | sticky;
            (1, significand)
        } else {
            (exponent as u16, significand)
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

    // #[test]
    // fn test_mul() {
    //     for ((v0, v1), expected) in [
    //         // ((0x200, 0x200), 0x0),
    //         // ((0x3c04, 0x3c04), 0x3C08),
    //         // ((513, 5117), 1),
    //         // ((1025, 4095), 1),
    //         // ((1025, 16383), 2048),
    //         // ((1057, 14305), 521),
    //         // ((15362, 31742), 31744),
    //         ((16384, 30721), 31744),
    //     ] {
    //         let x0 = SoftFloat16::from_bits(v0);
    //         let x1 = SoftFloat16::from_bits(v1);
    //         println!("{:016b} x0", x0.0);
    //         println!("{:016b} x1", x1.0);

    //         let y = x0 * x1;

    //         println!("{:016b} y", y.0);
    //         println!("{:016b} expected", expected);

    //         assert_eq!(SoftFloat16::to_bits(y), expected);
    //     }
    // }

    // #[test]
    // fn test_all_mul() {
    //     for i in 0..u16::MAX {
    //         if i % 1000 == 0 {
    //             println!("{}", i);
    //         }
    //         for j in 0..u16::MAX {
    //             let x0_sf = SoftFloat16::from_bits(i);
    //             let x1_sf = SoftFloat16::from_bits(j);
    //             let y_sf = x0_sf * x1_sf;
    //             let x0_f = Float16::from_bits(i);
    //             let x1_f = Float16::from_bits(j);
    //             let y_f = x0_f * x1_f;
    //             if y_sf == NAN {
    //                 assert!(Float16::is_nan(y_f));
    //             } else {
    //                 // println!("{:016b} x0", x0_sf.0);
    //                 // println!("{:016b} x1", x1_sf.0);

    //                 // println!("{:016b} y", y_sf.0);
    //                 // println!("{:016b} expected\n", y_f.0);
    //                 assert_eq!(
    //                     SoftFloat16::to_bits(y_sf),
    //                     Float16::to_bits(y_f),
    //                     "{:?}",
    //                     (i, j)
    //                 );
    //             }
    //         }
    //     }
    // }
}
