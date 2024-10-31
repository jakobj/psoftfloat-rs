use std::ops::Neg;

use crate::soft_float16::NAN;
use crate::SoftFloat16;

impl Neg for SoftFloat16 {
    type Output = Self;

    fn neg(self) -> Self {
        if self == NAN {
            NAN
        } else {
            Self::from_bits(Self::to_bits(self) ^ (1 << 15))
        }
    }
}
