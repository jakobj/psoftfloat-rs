use std::{
    env,
    fmt::Debug,
    io::{self, BufRead},
    ops::{Add, Div, Mul, Sub},
};

use soft_float::RoundTiesEven;
use soft_float::SoftFloat16;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    assert!(args.len() == 2);
    let mut split = args[1].split('_');
    let type_in = split.next().unwrap();
    let op = split.next().unwrap();
    let type_out = split.next().unwrap_or(type_in);

    match (type_in, type_out) {
        ("softfloat16", "f32") => testfloat::<SoftFloat16, f32>(op),
        ("f32", "softfloat16") => testfloat::<f32, SoftFloat16>(op),
        ("softfloat16", "softfloat16") => testfloat::<SoftFloat16, SoftFloat16>(op),
        _ => todo!(),
    };
}

trait ConvertHexStr {
    fn hex_str_to_float(s: &str) -> Self;
    fn float_to_hex_str(v: Self) -> String;
}

impl ConvertHexStr for f32 {
    fn hex_str_to_float(s: &str) -> Self {
        f32::from_bits(u32::from_str_radix(s, 16).expect("should be hex representation of u32"))
    }

    fn float_to_hex_str(v: Self) -> String {
        format!("{:08x}", v.to_bits()).to_uppercase()
    }
}

impl ConvertHexStr for f64 {
    fn hex_str_to_float(s: &str) -> Self {
        f64::from_bits(u64::from_str_radix(s, 16).expect("should be hex representation of u64"))
    }

    fn float_to_hex_str(v: Self) -> String {
        format!("{:016x}", v.to_bits()).to_uppercase()
    }
}

impl ConvertHexStr for SoftFloat16 {
    fn hex_str_to_float(s: &str) -> Self {
        Self::from_bits(u16::from_str_radix(s, 16).expect("should be hex representation of u16"))
    }

    fn float_to_hex_str(v: Self) -> String {
        format!("{:04x}", SoftFloat16::to_bits(v)).to_uppercase()
    }
}

fn testfloat<
    T_IN: ConvertHexStr
        + Add<Output = T_IN>
        + Sub<Output = T_IN>
        + Mul<Output = T_IN>
        + Div<Output = T_IN>
        + Debug
        + Copy
        + RoundTiesEven,
    T_OUT: ConvertHexStr + From<T_IN> + Debug + Copy,
>(
    op: &str,
) {
    for line in io::stdin().lock().lines() {
        // parse line
        let line = line.expect("should be able to read line from stdin");
        let words = line.split_whitespace().collect::<Vec<&str>>();

        // compute using internal arithmetic
        // NOTE this code does not check exception flags!!!
        let s = match op {
            "add" => {
                let (value0, value1) = (
                    T_IN::hex_str_to_float(words[0]),
                    T_IN::hex_str_to_float(words[1]),
                );
                let result = T_IN::add(value0, value1);
                format!(
                    "{} {} {} {}",
                    words[0],
                    words[1],
                    T_OUT::float_to_hex_str(T_OUT::from(result)),
                    words[3]
                )
            }
            "sub" => {
                let (value0, value1) = (
                    T_IN::hex_str_to_float(words[0]),
                    T_IN::hex_str_to_float(words[1]),
                );
                let result = T_IN::sub(value0, value1);
                format!(
                    "{} {} {} {}",
                    words[0],
                    words[1],
                    T_OUT::float_to_hex_str(T_OUT::from(result)),
                    words[3]
                )
            }
            "mul" => {
                let (value0, value1) = (
                    T_IN::hex_str_to_float(words[0]),
                    T_IN::hex_str_to_float(words[1]),
                );
                let result = T_IN::mul(value0, value1);
                format!(
                    "{} {} {} {}",
                    words[0],
                    words[1],
                    T_OUT::float_to_hex_str(T_OUT::from(result)),
                    words[3]
                )
            }
            "div" => {
                let (value0, value1) = (
                    T_IN::hex_str_to_float(words[0]),
                    T_IN::hex_str_to_float(words[1]),
                );
                let result = T_IN::div(value0, value1);
                format!(
                    "{} {} {} {}",
                    words[0],
                    words[1],
                    T_OUT::float_to_hex_str(T_OUT::from(result)),
                    words[3]
                )
            }
            // // sqrt
            // // rem
            // // eq, le, lt
            "round" => {
                let value = T_IN::hex_str_to_float(words[0]);
                let result = T_IN::round_ties_even(value);
                format!(
                    "{} {} {}",
                    words[0],
                    T_OUT::float_to_hex_str(T_OUT::from(result)),
                    words[2]
                )
            }
            "to" => {
                let result = T_IN::hex_str_to_float(words[0]);
                format!(
                    "{} {} {}",
                    words[0],
                    T_OUT::float_to_hex_str(T_OUT::from(result)),
                    words[2]
                )
            }
            _ => todo!(),
        };

        println!("{}", s);
    }
}
