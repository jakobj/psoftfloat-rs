use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct SoftFloat16(u16);

pub const POS_INFINITY: SoftFloat16 = SoftFloat16(0x7c00);
pub const NEG_INFINITY: SoftFloat16 = SoftFloat16(0xfc00);
pub const NAN: SoftFloat16 = SoftFloat16(0x7e00);
pub const POS_ZERO: SoftFloat16 = SoftFloat16(0x0);
pub const NEG_ZERO: SoftFloat16 = SoftFloat16(0x8000);

impl SoftFloat16 {
    pub fn clz(v: u16) -> u16 {
        if v == 0 {
            16
        } else {
            // branchless binary search
            // compare https://stackoverflow.com/a/10866821
            let mut n = 0;
            let mut v = v;
            let mut s;
            s = ((v & 0xFF00 == 0) as u16) << 3;
            n += s;
            v <<= s;
            s = ((v & 0xF000 == 0) as u16) << 2;
            n += s;
            v <<= s;
            s = ((v & 0xC000 == 0) as u16) << 1;
            n += s;
            v <<= s;
            s = (v & 0x8000 == 0) as u16;
            n += s;
            n
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_clz() {
        for i in 0..u16::MAX {
            let x = SoftFloat16::from_bits(i);
            assert_eq!(SoftFloat16::clz(x.0), x.0.leading_zeros() as u16);
        }
    }
}
