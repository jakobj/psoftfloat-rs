use crate::{
    soft_float16::{NAN, NEG_INFINITY, NEG_ZERO, POS_INFINITY, POS_ZERO},
    RoundTiesEven, SoftFloat16,
};

impl From<f32> for SoftFloat16 {
    fn from(value: f32) -> Self {
        let bits = value.to_bits();

        // extract fields
        let sign = ((bits & 0x80000000) >> 16) as u16;
        let unbiased_exponent = ((bits & 0x7f800000) >> 23) as i32 - 127;
        let original_significand = bits & 0x7fffff; // without implicit bit

        if unbiased_exponent < -25 {
            // smallest representable number in f16 is 2^{-24} and if unbiased
            // exponent is < -25 we can be sure rounding bit is zero
            if sign == 0 {
                POS_ZERO
            } else {
                NEG_ZERO
            }
        } else if unbiased_exponent == 128 && original_significand != 0 {
            // NAN
            NAN
        } else if unbiased_exponent > 15 {
            // overflow
            if sign == 0 {
                POS_INFINITY
            } else {
                NEG_INFINITY
            }
        } else {
            let (exponent, significand) = if unbiased_exponent < -14 {
                // convert to denormal number
                let original_significand = original_significand | 0x800000; // include implicit bit

                // shift to match exponents of normal f32 and denormal f16
                // shift additionally by 14 to fit 23+1bits into 10bits
                // keep guard, round, sticky bits
                let shift = -(unbiased_exponent + 15); // NOTE 1 2^unbiased_exponent = (1 >> shift) 2^{-15}
                assert!(shift >= 0);
                let sticky_bits = (1 << (shift + 14 - 3 + 1)) - 1;
                let sticky = (original_significand & sticky_bits != 0) as u16;
                let significand = (original_significand >> (shift + 14 - 3)) as u16 | sticky;
                (0, significand)
            } else {
                // adjust exponent
                let exponent = (unbiased_exponent + 15) as u16;

                // adjust significand
                // no need to include implicit bit since we convert a normal f32 to a normal f16 number
                // shift by 13 to fit 23bits into 10bits
                // keep guard, round, sticky bits
                let sticky_bits = (1 << (13 - 3 + 1)) - 1;
                let sticky = (original_significand & sticky_bits != 0) as u16;
                let significand = (original_significand >> (13 - 3)) as u16 | sticky;
                (exponent, significand)
            };

            // rounding (to even)
            let grs = significand & 0x7;
            let significand = significand >> 3;
            let lsb = significand & 1;
            if grs < 0x4 || (grs == 0x4 && lsb == 0) {
                Self::from_bits(sign | exponent << 10 | significand)
            } else {
                // allow significand to overflow into exponent
                Self::from_bits(sign | (exponent << 10 | significand) + 1)
            }
        }
    }
}

impl From<SoftFloat16> for f32 {
    fn from(value: SoftFloat16) -> Self {
        let sign = SoftFloat16::sign(value) as u32;
        let exponent = SoftFloat16::exponent(value) as u32;
        let significand = SoftFloat16::significand(value) as u32;

        if exponent == 0x1F && significand != 0 {
            return f32::NAN;
        } else if exponent == 0x1F && significand == 0 {
            return if sign == 0 {
                f32::INFINITY
            } else {
                f32::NEG_INFINITY
            };
        } else if exponent == 0 && significand == 0 {
            return if sign == 0 { 0.0_f32 } else { -0.0_f32 };
        }

        // handle denormals and implicit bit
        let (exponent, significand) = if exponent == 0 {
            (1, significand)
        } else {
            (exponent, significand | 0x400)
        };

        let mut exponent = exponent + 127 - 15;
        let mut significand = significand << 13;

        while significand & (1 << 23) == 0 && exponent > 1 {
            // realign decimal point if necessary (only happens for denormal
            // numbers)
            significand <<= 1;
            exponent -= 1;
        }

        f32::from_bits(sign << 31 | exponent << 23 | significand & 0x7fffff)
    }
}

impl From<i32> for SoftFloat16 {
    fn from(value: i32) -> Self {
        let bits = value as u32;

        let sign = (bits >> 31) as u16;
        let significand = value.abs() as u32;

        if significand == 0 {
            return if sign == 0 { POS_ZERO } else { NEG_ZERO };
        }

        if significand >= (1 << 16) {
            // overflow
            return if sign == 0 {
                POS_INFINITY
            } else {
                NEG_INFINITY
            };
        }

        let significand = significand as u16;

        // normalize
        let mut exponent = 30;
        let mut significand = significand;
        while significand & (1 << 15) == 0 {
            significand <<= 1;
            exponent -= 1;
        }

        // fit into 11 (significand) + 3 (grs) bits
        let sticky = if significand & 0x7 == 0 { 0 } else { 1 };
        let significand = significand >> (16 - 14) | sticky;

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

        // cut off implicit bit and allow overflow into exponent
        Self::from_bits(sign << 15 | (exponent << 10 | significand & 0x3FF) + rnd)
    }
}

impl From<SoftFloat16> for i32 {
    fn from(value: SoftFloat16) -> Self {
        let value = SoftFloat16::round_ties_even(value);

        let (sign, exponent, significand) = (
            SoftFloat16::sign(value),
            SoftFloat16::exponent(value),
            SoftFloat16::significand(value),
        );

        if exponent == 0x1F {
            // Infinity and NAN are returned as largest-magnitude negative integer
            // http://www.jhauser.us/arithmetic/TestFloat-3/doc/TestFloat-general.html, sec 6.1
            return 1 << 31;
        }

        let unbiased_exponent = (exponent as i16) - 15;

        if unbiased_exponent < 0 {
            // underflow
            return 0;
        }

        let significand = (significand | 0x400) as i32; // include implicit bit

        // shift to make sure lsb has exponent 0
        let shift = unbiased_exponent - 10;
        let significand = if shift < 0 {
            significand >> -shift
        } else {
            significand << shift
        };

        if sign == 0 {
            significand
        } else {
            -significand
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_softfloat16_from_i32() {
        for (x, expected) in [
            (1, 0x3c00),
            (2, 0x4000),
            (64, 0x5400),
            (65503, 0x7bff),
            (65504, 0x7bff),
            (65536, 0x7c00),
        ] {
            let y = SoftFloat16::from(x);
            assert_eq!(SoftFloat16::to_bits(y), expected)
        }
    }

    #[test]
    fn test_i32_from_softfloat16() {
        for (v, expected) in [
            (0x5894, 146),
            (0x609d, 590),
            (0x649d, 1181),
            (0x709d, 9448),
            (0x70E9, 0x2748),
            (0xf09d, -9448),
        ] {
            let x = SoftFloat16::from_bits(v);
            let y = i32::from(x);
            assert_eq!(y, expected)
        }
    }

    #[test]
    fn test_softfloat16_from_f32() {
        for (v, expected) in [
            (0x7FFF0007, 0x7e00),
            (0x337fc010, 0x1),
            (0x331ffffc, 0x1),
            (0x337FFFC4, 0x1),
        ] {
            let x = f32::from_bits(v);
            let y = SoftFloat16::from(x);
            assert_eq!(SoftFloat16::to_bits(y), expected)
        }
    }

    #[test]
    fn test_f32_from_softfloat16() {
        for (v, expected) in [
            (0x97db, 0xbafb6000_u32),
            (0xe850, 0xc50a0000_u32),
            (0x1, 0x33800000),
        ] {
            let x = SoftFloat16::from_bits(v);
            let y = f32::from(x);
            assert_eq!(y.to_bits(), expected)
        }
    }
}
