use std::ops::Sub;

use crate::SoftFloat16;

impl Sub for SoftFloat16 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

#[cfg(test)]
mod tests {
    use crate::soft_float16::NAN;

    use super::*;

    #[test]
    #[ignore]
    fn test_all_sub() {
        for i in 0..u16::MAX {
            for j in 0..u16::MAX {
                let x0 = SoftFloat16::from_bits(i);
                let x1 = SoftFloat16::from_bits(j);
                let y = x0 - x1;
                let y_f = SoftFloat16::from(f32::from(x0) - f32::from(x1));
                if y == NAN || y_f == NAN {
                    assert!(y == NAN, "{:?}", (i, j));
                    assert!(y_f == NAN, "{:?}", (i, j));
                } else {
                    assert_eq!(
                        SoftFloat16::to_bits(y),
                        SoftFloat16::to_bits(y_f),
                        "{:?}",
                        (i, j)
                    );
                }
            }
        }
    }
}
