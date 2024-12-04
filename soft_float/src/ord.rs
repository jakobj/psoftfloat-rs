use std::cmp::Ordering;

use crate::SoftFloat16;

impl PartialOrd for SoftFloat16 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let (sign0, exponent0, significand0) = (
            Self::sign(*self),
            Self::exponent(*self),
            Self::significand(*self),
        );
        let (sign1, exponent1, significand1) = (
            Self::sign(*other),
            Self::exponent(*other),
            Self::significand(*other),
        );

        if (exponent0 == 0x1F && significand0 != 0) || (exponent1 == 0x1F && significand1 != 0) {
            // NAN == NAN
            None
        } else if (exponent0 == 0 && significand0 == 0) && (exponent1 == 0 && significand1 == 0) {
            // 0 == 0
            Some(Ordering::Equal)
        } else if sign0 == 0 && sign1 == 1 {
            Some(Ordering::Greater)
        } else if sign0 == 1 && sign1 == 0 {
            Some(Ordering::Less)
        } else {
            assert!(sign0 == sign1);
            let self_bits_abs = SoftFloat16::to_bits(*self) & 0x7fff;
            let other_bits_abs = SoftFloat16::to_bits(*other) & 0x7fff;
            if sign0 == 0 {
                self_bits_abs.partial_cmp(&other_bits_abs)
            } else {
                other_bits_abs.partial_cmp(&self_bits_abs)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_all_le() {
        for i in 0..u16::MAX {
            for j in 0..u16::MAX {
                let x0 = SoftFloat16::from_bits(i);
                let x1 = SoftFloat16::from_bits(j);
                let y = x0 <= x1;
                let y_f = f32::from(x0) <= f32::from(x1);
                assert_eq!(y, y_f, "{:?}", (i, j));
            }
        }
    }

    #[test]
    #[ignore]
    fn test_all_lt() {
        for i in 0..u16::MAX {
            for j in 0..u16::MAX {
                let x0 = SoftFloat16::from_bits(i);
                let x1 = SoftFloat16::from_bits(j);
                let y = x0 < x1;
                let y_f = f32::from(x0) < f32::from(x1);
                assert_eq!(y, y_f, "{:?}", (i, j));
            }
        }
    }
}
