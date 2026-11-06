//! Excel's read of number text and literals whose `n` (fraction digits less the exponent) lies
//! outside 1..15: the 15 kept digits `m` times a power of ten built from two x87 tables, each
//! product rounded to a 64-bit significand and the result stored to a double
//! (docs/excel-semantics/numbers.md "Literals past n = 15 multiply by a table of 10^±16q", T-4467).
//!
//! With `n = 16q + r` (0 <= r <= 15), the read is `m * 10^-r` (the n <= 15 model of
//! [`crate::decimal_round::x87_text_to_f64`]), then times `10^-16q` as one table entry for
//! q <= 16, or times `10^-16(q-16)` and then `10^-256` past it. A positive exponent reads the same
//! with positive powers. Every entry is 10^k rounded once to 64 bits. This holds at all 200
//! Train, 112 Hold and 170 fresh-sweep cells of probes/t4467-literal-large-exponent and -fresh
//! (excel-graph, 2026-11-06), where the nearest double holds at 90, 62 and 20:
//! `=2.036E-77` is 2.0360000000000002E-77, one ulp above the nearest double.

use crate::x87_register::Register;

const TEN_TO_MINUS_16Q: [(u64, i32); 16] = [
    (0xE69594BEC44DE15B, -117), // 10^-16
    (0xCFB11EAD453994BA, -170), // 10^-32
    (0xBB127C53B17EC159, -223), // 10^-48
    (0xA87FEA27A539E9A5, -276), // 10^-64
    (0x97C560BA6B0919A6, -329), // 10^-80
    (0x88B402F7FD75539B, -382), // 10^-96
    (0xF64335BCF065D37D, -436), // 10^-112
    (0xDDD0467C64BCE4A1, -489), // 10^-128
    (0xC7CABA6E7C5382C9, -542), // 10^-144
    (0xB3F4E093DB73A093, -595), // 10^-160
    (0xA21727DB38CB0030, -648), // 10^-176
    (0x91FF83775423CC06, -701), // 10^-192
    (0x8380DEA93DA4BC60, -754), // 10^-208
    (0xECE53CEC4A314EBE, -808), // 10^-224
    (0xD5605FCDCF32E1D7, -861), // 10^-240
    (0xC0314325637A193A, -914), // 10^-256
];
const TEN_TO_16Q: [(u64, i32); 16] = [
    (0x8E1BC9BF04000000, -10), // 10^16
    (0x9DC5ADA82B70B59E, 43),  // 10^32
    (0xAF298D050E4395D7, 96),  // 10^48
    (0xC2781F49FFCFA6D5, 149), // 10^64
    (0xD7E77A8F87DAF7FC, 202), // 10^80
    (0xEFB3AB16C59B14A3, 255), // 10^96
    (0x850FADC09923329E, 309), // 10^112
    (0x93BA47C980E98CE0, 362), // 10^128
    (0xA402B9C5A8D3A6E7, 415), // 10^144
    (0xB616A12B7FE617AA, 468), // 10^160
    (0xCA28A291859BBF93, 521), // 10^176
    (0xE070F78D3927556B, 574), // 10^192
    (0xF92E0C3537826146, 627), // 10^208
    (0x8A5296FFE33CC930, 681), // 10^224
    (0x9991A6F3D6BF1766, 734), // 10^240
    (0xAA7EEBFB9DF9DE8E, 787), // 10^256
];

/// The table entry 10^(±16q), `q` in 1..=16, as a register.
fn sixteen_power(q: i64, negative: bool) -> Option<Register> {
    let table = if negative {
        &TEN_TO_MINUS_16Q
    } else {
        &TEN_TO_16Q
    };
    let index = usize::try_from(q.checked_sub(1)?).ok()?;
    let (significand, exponent) = *table.get(index)?;
    Some(Register::from_parts(
        false,
        u128::from(significand),
        exponent,
    ))
}

/// `10^±r` for `r` in 1..=15 rounded to 64 bits: 10^r is exact in 64 bits, and 10^-r is its
/// x87 reciprocal, as the n <= 15 read divides (`decimal_round::x87_multiply`).
fn small_power(r: i64, negative: bool) -> Option<Register> {
    let power = Register::from_parts(false, 10_u128.checked_pow(u32::try_from(r).ok()?)?, 0);
    Some(if negative {
        Register::from_parts(false, 1, 0).div(power)
    } else {
        power
    })
}

/// `digits * 10^place` as Excel reads it, for `|place| >= 16` and nonzero `digits` below 2^64;
/// `None` elsewhere (beyond 10^±512, past any number Excel accepts).
pub(crate) fn x87_table_read(digits: u64, place: i64) -> Option<f64> {
    if digits == 0 || place.unsigned_abs() < 16 {
        return None;
    }
    let negative = place < 0;
    let magnitude = i64::try_from(place.unsigned_abs()).ok()?;
    let (q, r) = (magnitude / 16, magnitude % 16);
    let mut value = Register::from_parts(false, u128::from(digits), 0);
    if r != 0 {
        value = value.mul(small_power(r, negative)?);
    }
    if q <= 16 {
        value = value.mul(sixteen_power(q, negative)?);
    } else {
        // probes/t4467-literal-large-exponent Train: the 17 cells at n >= 272 take 10^-16(q-16)
        // before 10^-256; the other order misses 3 of them, one product of the two misses 3.
        value = value
            .mul(sixteen_power(q - 16, negative)?)
            .mul(sixteen_power(16, negative)?);
    }
    Some(value.store())
}

#[cfg(test)]
mod tests {
    use super::{x87_table_read, TEN_TO_16Q, TEN_TO_MINUS_16Q};

    /// Each entry is 10^(±16q) rounded to 64 bits: its significand has the top bit set, and as a
    /// double (one more rounding) it is the nearest double of the power, which Rust's parse gives.
    #[test]
    fn table_entries_are_the_powers() {
        for (index, (&(minus, minus_exp), &(plus, plus_exp))) in
            TEN_TO_MINUS_16Q.iter().zip(TEN_TO_16Q.iter()).enumerate()
        {
            let k = 16 * (index + 1);
            assert!(minus >> 63 == 1 && plus >> 63 == 1);
            let near =
                |s: u64, e: i32| super::Register::from_parts(false, u128::from(s), e).store();
            assert_eq!(
                near(minus, minus_exp),
                format!("1e-{k}").parse::<f64>().unwrap()
            );
            if k <= 304 {
                assert_eq!(
                    near(plus, plus_exp),
                    format!("1e{k}").parse::<f64>().unwrap()
                );
            }
        }
    }

    /// Cells of probes/t4467-literal-large-exponent and -fresh (excel-graph, 2026-11-06).
    #[test]
    fn reads_match_excel() {
        let above = |x: f64| f64::from_bits(x.to_bits() + 1);
        let below = |x: f64| f64::from_bits(x.to_bits() - 1);
        // t4467-bessely-d32-refill E1: one ulp above the nearest double.
        assert_eq!(x87_table_read(2036, -80), Some(above(2.036e-77)));
        assert_eq!(x87_table_read(2035, -80), Some(2.035e-77));
        assert_eq!(x87_table_read(2037, -80), Some(2.037e-77));
        // Train A83 and A76, n >= 272 (10^-16(q-16), then 10^-256).
        assert_eq!(x87_table_read(87, -274), Some(above(8.7e-273)));
        assert_eq!(
            x87_table_read(59_851_504, -299),
            Some(below(5.985_150_4e-292))
        );
        // Train A16 and A67, positive exponents.
        assert_eq!(x87_table_read(4, 126), Some(above(4e126)));
        assert_eq!(x87_table_read(37, 46), Some(below(3.7e47)));
        // Below 16 places this read does not apply.
        assert_eq!(x87_table_read(491, -8), None);
        assert_eq!(x87_table_read(0, -80), None);
    }
}
