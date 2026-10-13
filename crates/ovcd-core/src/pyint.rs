//! Python `round(x)` on a float: an `int` of any size, or the error Python raises.
//!
//! Python ints have no upper bound, so `round(1e300)` is an exact 301-digit integer, and
//! `round(inf)` and `round(nan)` raise. A rounded money figure is therefore a [`PyInt`], kept as
//! its exact decimal text.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// An integer of any size, as exact decimal text (a leading `-` when negative, no leading zeros).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyInt(String);

impl PyInt {
    /// The exact decimal text, as Python's `str(int)` and JSON write it.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PyInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<i64> for PyInt {
    fn from(i: i64) -> Self {
        PyInt(i.to_string())
    }
}

impl PartialEq<i64> for PyInt {
    fn eq(&self, other: &i64) -> bool {
        self.0 == other.to_string()
    }
}

/// The error `round(x)` raises for a float that has no integer value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundError {
    /// `round(nan)`: Python raises `ValueError`.
    Nan,
    /// `round(inf)` or `round(-inf)`: Python raises `OverflowError`.
    Infinity,
}

impl RoundError {
    /// The Python exception class name.
    pub fn type_name(self) -> &'static str {
        match self {
            RoundError::Nan => "ValueError",
            RoundError::Infinity => "OverflowError",
        }
    }

    /// `str(exception)`.
    pub fn message(self) -> &'static str {
        match self {
            RoundError::Nan => "cannot convert float NaN to integer",
            RoundError::Infinity => "cannot convert float infinity to integer",
        }
    }
}

/// Python `round(x)`: half to even, exact at any size.
pub fn round(x: f64) -> Result<PyInt, RoundError> {
    if x.is_nan() {
        return Err(RoundError::Nan);
    }
    if x.is_infinite() {
        return Err(RoundError::Infinity);
    }
    let r = libm::rint(x);
    if libm::fabs(r) < 9.0e18 {
        return Ok(PyInt::from(r as i64));
    }
    // |r| >= 9e18 has no fraction: r = mantissa * 2^shift, with a 53-bit mantissa.
    let bits = r.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = (bits & ((1u64 << 52) - 1)) | (1u64 << 52);
    let shift = exponent - 1075;
    // Base 1e9 limbs, least significant first.
    let mut limbs: Vec<u32> = Vec::new();
    let mut rest = mantissa;
    while rest > 0 {
        limbs.push((rest % 1_000_000_000) as u32);
        rest /= 1_000_000_000;
    }
    for _ in 0..shift {
        let mut carry = 0u32;
        for limb in limbs.iter_mut() {
            let doubled = *limb * 2 + carry;
            carry = doubled / 1_000_000_000;
            *limb = doubled % 1_000_000_000;
        }
        if carry > 0 {
            limbs.push(carry);
        }
    }
    let mut text = String::new();
    if r < 0.0 {
        text.push('-');
    }
    let mut first = true;
    for limb in limbs.iter().rev() {
        if first {
            text.push_str(&limb.to_string());
            first = false;
        } else {
            text.push_str(&alloc::format!("{limb:09}"));
        }
    }
    Ok(PyInt(text))
}

/// With `serde`: a number where it fits in 128 bits, else the exact digits as text.
#[cfg(feature = "serde")]
impl serde::Serialize for PyInt {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Ok(i) = self.0.parse::<i64>() {
            serializer.serialize_i64(i)
        } else if let Ok(i) = self.0.parse::<i128>() {
            serializer.serialize_i128(i)
        } else {
            serializer.serialize_str(&self.0)
        }
    }
}
