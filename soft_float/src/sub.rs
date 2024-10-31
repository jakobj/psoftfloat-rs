use std::ops::Sub;

use crate::SoftFloat16;

impl Sub for SoftFloat16 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + -other
    }
}
