use crate::soft_float16::NAN;
use crate::SoftFloat16;

pub trait MulAdd {
    fn mul_add(v0: Self, v1: Self, v2: Self) -> Self;
}

impl MulAdd for SoftFloat16 {
    fn mul_add(v0: Self, v1: Self, v2: Self) -> Self {
        println!("----------------");

        // TODO handle special numbers
        let (sign, exponent, significand) = mul(v0, v1);

        add(sign, exponent, significand, v2)
    }

}

fn mul(v0: SoftFloat16, v1: SoftFloat16) -> (u16, i16, u32) {
    let (sign0, exponent0, significand0) = (
        SoftFloat16::sign(v0),
        SoftFloat16::exponent(v0),
        SoftFloat16::significand(v0),
    );

    let (sign1, exponent1, significand1) = (
        SoftFloat16::sign(v1),
        SoftFloat16::exponent(v1),
        SoftFloat16::significand(v1),
    );

    let sign = sign0 ^ sign1;

    // handle denormals and implicit bit
    let (exponent0, significand0) = if exponent0 == 0 {
        // TODO introduce function for `normalize`
        // normalize
        let mut exponent = 1_i16;
        let mut significand = significand0;
        assert!(significand != 0);
        while significand & (1 << 10) == 0 {
            significand <<= 1;
            exponent -= 1;
        }
        (exponent, significand)
    } else {
        (exponent0 as i16, significand0 | 0x400)
    };

    let (exponent1, significand1) = if exponent1 == 0 {
        // normalize
        let mut exponent = 1_i16;
        let mut significand = significand1;
        assert!(significand != 0);
        while significand & (1 << 10) == 0 {
            significand <<= 1;
            exponent -= 1;
        }
        (exponent, significand)
    } else {
        (exponent1 as i16, significand1 | 0x400)
    };

    let exponent = exponent0 + exponent1 - 15; // biased exponent of result
    let significand = (significand0 as u32) * (significand1 as u32);
    assert!(significand < (1 << (2 * 11))); // result can not have more than 2 * 11 bits
    assert!((significand & (1 << (1 + 10 + 10))) != 0 | (significand & (1 << (10 + 10)))); // result looks like 1x.x{10}x{10} or 1.x{10}x{10}

    (sign, exponent, significand)
}

fn add(sign0: u16, exponent0: i16, significand0: u32, v1: SoftFloat16) -> SoftFloat16 {
    let (sign1, exponent1, significand1) = (
        SoftFloat16::sign(v1),
        SoftFloat16::exponent(v1),
        SoftFloat16::significand(v1),
    );

    let significand0 = significand0 as u64;

    // handle denormals and implicit bit
    let (exponent1, significand1) = if exponent1 == 0 {
        (1_i16, significand1 as u64)
    } else {
        (exponent1 as i16, (significand1 | 0x400) as u64)
    };

    // align second significand (which looks like x.x{10}) to result of
    // multiplication (which looks like xx.x{10}x{10})
    let significand1 = significand1 << 10;

    // align decimal point of second number
    let shift = exponent1 - exponent0;

    // if shift is too large, not even grs bits are influenced by first summand
    // and we can return early
    if shift >= 11 + 3 + 1 {
        return v1;
    }

    let significand1 = if shift < 0 {
        significand1 >> -shift
    } else {
        significand1 << shift
    };

    let (sign, exponent, significand) = if sign0 == sign1 {
        let (sign, exponent, significand) = (sign0, exponent0, significand0 + significand1);
        // result is at least 1.0{10}0{10}
        assert!(significand >= (1 << (10 + 10)));
        // result is at most 11{10}0011.1{10}1{10}
        assert!(significand < (1 << (1 + 10 + 2 + 2 + 10 + 10)));

        println!("{:036b} +", significand0);
        println!("{:036b} =", significand1);
        println!("{:036b} <- sig", significand);
        println!("{} <- exp", exponent);

        // normalize
        // first we align to look like 1.x{10}x{10}
        let mut sticky = 0;
        let mut exponent = exponent;
        let mut significand = significand;
        while significand >= (1 << (1 + 10 + 10)) {
            sticky |= significand & 1;
            significand >>= 1;
            exponent += 1;
        }
        // second we align to look like 1.x{10}x{3}, keeping grs bits
        let sticky_bits = (1 << 7) - 1;
        let sticky = (significand & sticky_bits != 0) as u64 | sticky;
        let significand = ((significand >> 7) | sticky) as u16;

        println!("{:036b} <- sig (after)", significand);
        println!("{} <- exp (after)", exponent);

        (sign, exponent, significand)
    } else {
        todo!();
    };

    let (exponent, significand) = if exponent <= 0 {
        // must convert to denormal number; make exponent representable by
        // shifting significand
        let shift = 1 - exponent;
        let sticky_bits = (1 << shift) - 1;
        let sticky = (significand & sticky_bits != 0) as u16;
        let significand = (significand >> shift) | sticky;
        (1, significand)
    } else {
        (exponent as u16, significand)
    };

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

    // denormals have exponent 0
    let exponent = if exponent == 1 && significand < 0x400 {
        0
    } else {
        exponent
    };

    // cut off implicit bit and allow into exponent
    SoftFloat16::from_bits(sign << 15 | (exponent << 10 | significand & 0x3FF) + rnd)
}

impl MulAdd for f32 {
    fn mul_add(_v0: Self, _v1: Self, _v2: Self) -> Self {
        unimplemented!();
    }
}

impl MulAdd for i32 {
    fn mul_add(_v0: Self, _v1: Self, _v2: Self) -> Self {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_add() {
        for (v0, v1, v2) in [
            (0x3e00, 0x3e00, 0x3e00),
            (0x3e00, 0x3e00, 0x3a00),
            (0x3e00, 0x3e00, 0x4200),
            (0x1e00, 0x1e00, 0x3200),
            (0x1e00, 0x1e00, 0x0200),
            (0x1a00, 0x1a00, 0x0100),
            (0x1, 0x1, 0x7bff),
            (0x5900, 0x5900, 0x1),
            // 8444 BB7E B430 B430
            // (0x8444, 0xBB7E, 0xB430),
        ] {
            let x0 = SoftFloat16::from_bits(v0);
            let x1 = SoftFloat16::from_bits(v1);
            let x2 = SoftFloat16::from_bits(v2);
            let y = SoftFloat16::mul_add(x0, x1, x2);
            let y_f = SoftFloat16::from(f32::from(x0).mul_add(f32::from(x1), f32::from(x2)));
            println!("\n{:016b} {} <- y", SoftFloat16::to_bits(y), y);
            println!("{:016b} {} <- y_f", SoftFloat16::to_bits(y_f), y_f);
            assert_eq!(SoftFloat16::to_bits(y), SoftFloat16::to_bits(y_f));
        }
    }

    #[test]
    #[ignore]
    fn test_all_mul_add() {
        for i in 0..u16::MAX {
            for j in 0..u16::MAX {
                for k in 0..u16::MAX {
                    let x0 = SoftFloat16::from_bits(i);
                    let x1 = SoftFloat16::from_bits(j);
                    let x2 = SoftFloat16::from_bits(k);
                    let y = SoftFloat16::mul_add(x0, x1, x2);
                    let y_f = SoftFloat16::from(f32::from(x0).mul_add(f32::from(x1), f32::from(x2)));
                    if y == NAN || y_f == NAN {
                        assert!(y == NAN, "{:?}", (i, j, k));
                        assert!(y_f == NAN, "{:?}", (i, j, k));
                    } else {
                        assert_eq!(
                            SoftFloat16::to_bits(y),
                            SoftFloat16::to_bits(y_f),
                            "{:?}",
                            (i, j, k)
                        );
                    }
                }
            }
        }
    }
}
