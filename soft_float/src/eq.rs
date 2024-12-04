use crate::SoftFloat16;

impl PartialEq for SoftFloat16 {
    fn eq(&self, other: &Self) -> bool {
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
            false
        } else if exponent0 == 0 && significand0 == 0 && exponent1 == 0 && significand1 == 0 {
            true
        } else {
            if sign0 == sign1 && exponent0 == exponent1 && significand0 == significand1 {
                true
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_all_eq() {
        for i in 0..u16::MAX {
            for j in 0..u16::MAX {
                let x0 = SoftFloat16::from_bits(i);
                let x1 = SoftFloat16::from_bits(j);
                let y = x0 == x1;
                let y_f = f32::from(x0) == f32::from(x1);
                assert_eq!(y, y_f, "{:?}", (i, j));
            }
        }
    }
}
