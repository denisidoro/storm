use anyhow::{Context, Result};
use std::u128;

pub fn gen_password(password: &str, filename: &str) -> Result<String> {
    let base = format!("{}_{}\n", password, filename);
    let hashed_bytes = md5::compute(base).0;
    let decimal = array_u8_to_decimal(hashed_bytes);
    let lowercase = format_radix(decimal, 36)?;
    Ok(lowercase.to_uppercase())
}

fn array_u8_to_decimal(arr: [u8; 16]) -> u128 {
    let mut d = 0;
    let base: u128 = 256;
    for (p, b) in arr.into_iter().rev().enumerate() {
        let s = base.pow(p as u32) * (b as u128);
        d += s;
    }
    d
}

pub fn format_radix(mut x: u128, radix: u128) -> Result<String> {
    let mut result = vec![];

    loop {
        let m = x % radix;
        x /= radix;
        let n = std::char::from_digit(m as u32, radix as u32).context("base must be 2 <= b <= 36")?;
        result.push(n);
        if x == 0 {
            break;
        }
    }

    Ok(result.into_iter().rev().collect())
}
