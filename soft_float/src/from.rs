use crate::{
    soft_float16::{NAN, NEG_INFINITY, NEG_ZERO, POS_INFINITY, POS_ZERO},
    SoftFloat16,
};

impl From<f32> for SoftFloat16 {
    fn from(value: f32) -> Self {
        let bits = value.to_bits();

        // extract fields using corresponding bitmasks
        let original_sign = bits & 0x80000000;
        let original_exponent = bits & 0x7f800000;
        let original_significand = bits & 0x7fffff; // without implicit bit

        let sign = (original_sign >> 16) as u16;

        if original_exponent == 0 {
            // zero or subnormal; largest subnormal in f32 is smaller than smallest subnormal in f16, so we don't need to handle that case separately
            return if sign == 0 { POS_ZERO } else { NEG_ZERO };
        } else if original_exponent == 0x7f800000 {
            if original_significand == 0 {
                // infinity
                return if sign == 0 {
                    POS_INFINITY
                } else {
                    NEG_INFINITY
                };
            } else {
                // nan
                return NAN;
            }
        };

        // adjust exponent
        let unbiased_exponent = (original_exponent >> 23) as i32 - 127;
        if unbiased_exponent < -14 {
            // underflow
            if unbiased_exponent >= -15 - 10 {
                // subnormal representation possible with shift \in [0, 10]
                let original_significand_wib = original_significand | 0x800000; // with implicit bit
                let shift = -(unbiased_exponent + 15); // NOTE 1 2^unbiased_exponent = (1 >> shift) 2^{-15}
                assert!(shift >= 0);
                // shift to match exponents of normal f32 and denormal f16
                // shift additionally by 14 to fit 23+1bits into 10bits
                let significand = ((original_significand_wib >> shift) >> 14) as u16;
                // check (n+1)th bit for rounding where n is the amount of shift
                let rnd_bit = 1 << (shift + 13);
                let lsb = original_significand_wib & (rnd_bit << 1);
                let rnd = original_significand_wib & rnd_bit;
                let rest = original_significand_wib & (rnd_bit - 1);
                if rnd == 0 || (rnd != 0 && rest == 0 && lsb == 0) {
                    return Self::from_bits(sign | 0x0 | significand);
                } else {
                    // allow significand to overflow into exponent
                    return Self::from_bits(sign | (0x0 | significand) + 1);
                }
            } else {
                // zero
                return if sign == 0 { POS_ZERO } else { NEG_ZERO };
            }
        } else if unbiased_exponent > 15 {
            // overflow
            return if sign == 0 {
                POS_INFINITY
            } else {
                NEG_INFINITY
            };
        }
        let exponent = ((unbiased_exponent + 15) << 10) as u16;

        // adjust significand
        // shift by 13 to fit 23bits into 10bits
        let significand = (original_significand >> 13) as u16;
        // check (n+1)th bit for rounding where n is the amount of shift
        let rnd_bit = 1 << 12;
        let lsb = original_significand & (rnd_bit << 1);
        let rnd = original_significand & rnd_bit;
        let rest = original_significand & (rnd_bit - 1);
        if rnd == 0 || (rnd != 0 && rest == 0 && lsb == 0) {
            Self::from_bits(sign | exponent | significand)
        } else {
            // allow significand to overflow into exponent
            Self::from_bits(sign | (exponent | significand) + 1)
        }
    }
}

impl From<SoftFloat16> for f32 {
    fn from(value: SoftFloat16) -> Self {
        todo!()
    }
}
