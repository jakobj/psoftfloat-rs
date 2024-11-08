use crate::{
    soft_float16::{NEG_ZERO, POS_ZERO},
    SoftFloat16,
};

pub trait RoundTiesEven {
    fn round_ties_even(v: Self) -> Self;
}

impl RoundTiesEven for SoftFloat16 {
    fn round_ties_even(v: Self) -> Self {
        let (sign, exponent, significand) =
            (Self::sign(v), Self::exponent(v), Self::significand(v));

        let unbiased_exponent = (exponent as i16) - 15;

        if unbiased_exponent < -1 {
            if sign == 0 {
                POS_ZERO
            } else {
                NEG_ZERO
            }
        } else if unbiased_exponent >= 10 {
            v
        } else {
            let shift = 10 - unbiased_exponent;
            let exponent = (unbiased_exponent + 15) as u16;
            let significand = significand | 1 << 10;

            let integer = significand >> shift;
            let fraction = significand & ((1 << shift) - 1);
            let half = 1 << (shift - 1);

            // rounding (to even)
            let rnd = if fraction < half || fraction == half && integer & 1 == 0 {
                // round down
                0
            } else {
                // round up
                1 << shift
            };

            let significand = integer << shift;

            if significand == 0 && rnd == 0 {
                if sign == 0 {
                    POS_ZERO
                } else {
                    NEG_ZERO
                }
            } else if rnd == 1 << 11 {
                Self::from_bits(sign << 15 | (exponent + 1) << 10)
            } else {
                // cut off implicit bit and allow overflow into exponent
                Self::from_bits(sign << 15 | (exponent << 10 | significand & ((1 << 10) - 1)) + rnd)
            }
        }
    }
}

impl RoundTiesEven for f32 {
    fn round_ties_even(_v: Self) -> Self {
        unreachable!();
    }
}

#[cfg(test)]
mod tests {
    use crate::soft_float16::NAN;

    use super::*;

    #[test]
    fn test_round() {
        for v in [0x47ff] {
            let x = SoftFloat16::from_bits(v);
            let y = SoftFloat16::round_ties_even(x);
            let y_f = SoftFloat16::from(f32::from(x).round_ties_even());
            if y == NAN || y_f == NAN {
                assert!(y == NAN, "{}", v);
                assert!(y_f == NAN, "{}", v);
            } else {
                assert_eq!(
                    SoftFloat16::to_bits(y),
                    SoftFloat16::to_bits(y_f),
                    "\n{}\n  {:016b}\n  {:016b}",
                    v,
                    SoftFloat16::to_bits(y),
                    SoftFloat16::to_bits(y_f),
                );
            }
        }
    }

    #[test]
    #[ignore]
    fn test_all_round() {
        for i in 0..u16::MAX {
            let x = SoftFloat16::from_bits(i);
            let y = SoftFloat16::round_ties_even(x);
            let y_f = SoftFloat16::from(f32::from(x).round_ties_even());
            if y == NAN || y_f == NAN {
                assert!(y == NAN, "{}", i);
                assert!(y_f == NAN, "{}", i);
            } else {
                assert_eq!(
                    SoftFloat16::to_bits(y),
                    SoftFloat16::to_bits(y_f),
                    "\n{}\n  {:016b}\n  {:016b}",
                    i,
                    SoftFloat16::to_bits(y),
                    SoftFloat16::to_bits(y_f),
                );
            }
        }
    }
}
