//! Format floats the way Python does. Not a port of a Python file: Python builds message text with
//! `str(x)` and f-string specs, and Rust's formatting differs, so every later module that writes
//! message text goes through this one.
//!
//! What matches Python 3.12:
//! - `py_str`: `str(x)` / `repr(x)`: shortest round-trip digits, always a `.0` or an exponent,
//!   exponent form from 1e16 and below 1e-4, `inf`, `-inf`, `nan`, `-0.0`.
//! - `fixed`: `f"{x:.Nf}"`: exact binary value rounded half to even, `-0.00` keeps its sign.
//! - `general`: `f"{x:g}"`: six significant digits, trailing zeros removed, exponent form
//!   below 1e-4 and from 1e6.
//! - `round_to`: `round(x, n)`: the same exact half-to-even rounding, read back as a float.
//! - `percent`: `f"{x:.N%}"`: `x * 100` as a float, then `fixed`, then `%`.
//! - `comma_fixed`: `f"{x:,.Nf}"`.
//!
//! The tests (`tests/pyfmt.rs`) hold expected strings printed by CPython itself.

use alloc::format;
use alloc::string::{String, ToString};

/// Python `nan`, `inf` or `-inf`, for the three values that never reach the digit code.
fn non_finite(x: f64) -> Option<&'static str> {
    if x.is_nan() {
        Some("nan")
    } else if x.is_infinite() {
        Some(if x == f64::NEG_INFINITY {
            "-inf"
        } else {
            "inf"
        })
    } else {
        None
    }
}

/// Split Rust's `{:e}` output (`-1.2345e-5`) into (negative, digits, decimal exponent).
fn split_exp(text: &str) -> (bool, String, i32) {
    let (negative, rest) = match text.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, text),
    };
    let (mantissa, exp) = rest.split_once('e').unwrap_or((rest, "0"));
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    (negative, digits, exp.parse().unwrap_or(0))
}

/// Python's exponent suffix: sign always shown, at least two digits.
fn exp_suffix(exp: i32) -> String {
    let sign = if exp.is_negative() { '-' } else { '+' };
    format!("e{sign}{:02}", exp.unsigned_abs())
}

/// Shortest digits that read back as `x`, the way Python picks them.
///
/// Rust's `{:e}` is shortest too, but when two shortest candidates both read back as `x` it can
/// pick the one further from the true value (`-109654900867286.13` where Python prints `...12`).
/// Python picks the closest. The closest `n`-digit decimal is exactly the correctly rounded one,
/// so use that whenever it reads back as `x`.
fn shortest_digits(x: f64) -> (bool, String, i32) {
    let shortest = split_exp(&format!("{x:e}"));
    let n = shortest.1.len();
    let closest_text = format!("{x:.prec$e}", prec = n - 1);
    if closest_text.parse::<f64>() == Ok(x) {
        split_exp(&closest_text)
    } else {
        shortest
    }
}

/// `str(x)` for a float.
pub fn py_str(x: f64) -> String {
    if let Some(s) = non_finite(x) {
        return s.to_string();
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0.0" } else { "0.0" }.to_string();
    }
    let (negative, digits, exp) = shortest_digits(x);
    let sign = if negative { "-" } else { "" };
    let n = digits.len() as i32;
    let decpt = exp + 1; // position of the decimal point relative to the digits
    let body = if -4 < decpt && decpt <= 16 {
        if decpt <= 0 {
            format!("0.{}{digits}", "0".repeat((-decpt) as usize))
        } else if decpt >= n {
            format!("{digits}{}.0", "0".repeat((decpt - n) as usize))
        } else {
            let (int, frac) = digits.split_at(decpt as usize);
            format!("{int}.{frac}")
        }
    } else {
        let (first, rest) = digits.split_at(1);
        let mantissa = if rest.is_empty() {
            first.to_string()
        } else {
            format!("{first}.{rest}")
        };
        format!("{mantissa}{}", exp_suffix(exp))
    };
    format!("{sign}{body}")
}

/// `f"{x:.{precision}f}"`.
pub fn fixed(x: f64, precision: usize) -> String {
    match non_finite(x) {
        Some(s) => s.to_string(),
        None => format!("{x:.precision$}"),
    }
}

/// Python `round(x, ndigits)` on a float, for `ndigits >= 0`.
///
/// CPython rounds the exact binary value half to even and reads the digits back, so this is
/// `fixed` followed by a parse. `round(2.25, 1)` is `2.2` and `round(2.675, 2)` is `2.67`.
/// NaN and infinity come back unchanged.
pub fn round_to(x: f64, ndigits: usize) -> f64 {
    if !x.is_finite() {
        return x;
    }
    format!("{x:.ndigits$}")
        .parse()
        .expect("a fixed-point float text parses")
}

/// `f"{x:g}"` (default precision 6).
pub fn general(x: f64) -> String {
    const SIG: i32 = 6;
    if let Some(s) = non_finite(x) {
        return s.to_string();
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0" } else { "0" }.to_string();
    }
    // Round to six significant digits first: the rounded exponent picks the notation.
    let (negative, digits, exp) = split_exp(&format!("{x:.5e}"));
    let sign = if negative { "-" } else { "" };
    if (-4..SIG).contains(&exp) {
        let decimals = (SIG - 1 - exp) as usize;
        let text = format!("{:.decimals$}", x.abs());
        let text = if text.contains('.') {
            text.trim_end_matches('0').trim_end_matches('.')
        } else {
            text.as_str()
        };
        format!("{sign}{text}")
    } else {
        let digits = digits.trim_end_matches('0');
        let (first, rest) = digits.split_at(1.min(digits.len()));
        let mantissa = if rest.is_empty() {
            first.to_string()
        } else {
            format!("{first}.{rest}")
        };
        format!("{sign}{mantissa}{}", exp_suffix(exp))
    }
}

/// `f"{x:.{precision}%}"`.
pub fn percent(x: f64, precision: usize) -> String {
    format!("{}%", fixed(x * 100.0, precision))
}

/// `f"{x:,.{precision}f}"`.
pub fn comma_fixed(x: f64, precision: usize) -> String {
    let text = fixed(x, precision);
    if non_finite(x).is_some() {
        return text;
    }
    let (sign, unsigned) = match text.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", text.as_str()),
    };
    let (int, frac) = match unsigned.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (unsigned, None),
    };
    let mut out = String::from(sign);
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if let Some(f) = frac {
        out.push('.');
        out.push_str(f);
    }
    out
}
