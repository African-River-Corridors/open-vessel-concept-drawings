//! Python string behaviours the core must copy exactly: `repr(str)`, `str.isspace`, `strip`,
//! `splitlines`, `float(str)` and `str.isidentifier`. They exist because Python's error and check
//! messages embed `repr` of user text, and because the rates-file reader splits and strips as Python does.
//!
//! Outside ASCII, the answers come from `pystr_tables.rs`, generated from Python 3.12's own Unicode
//! database (`tools/gen_pystr_tables.py`). That is the Python the frozen oracle runs on.

use crate::pystr_tables::{DECIMAL_ZEROS, IDENTIFIER_CONTINUE, IDENTIFIER_START, NOT_PRINTABLE};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// `str.isspace` for one character (Python's whitespace set, not Rust's).
pub fn is_space(c: char) -> bool {
    matches!(c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | '\u{1c}'..='\u{1f}' | ' ' | '\u{85}' | '\u{a0}'
        | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}'
        | '\u{205f}' | '\u{3000}')
}

/// `s.strip()`.
pub fn strip(s: &str) -> &str {
    s.trim_matches(is_space)
}

/// `s.rstrip()`.
pub fn rstrip(s: &str) -> &str {
    s.trim_end_matches(is_space)
}

/// `s.splitlines()`: splits on `\n`, `\r`, `\r\n`, `\v`, `\f`, `\x1c`-`\x1e`, `\x85`, ` `, ` `.
pub fn splitlines(s: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let is_break = matches!(
            c,
            '\n' | '\r' | '\u{b}' | '\u{c}' | '\u{1c}'
                ..='\u{1e}' | '\u{85}' | '\u{2028}' | '\u{2029}'
        );
        if !is_break {
            continue;
        }
        lines.push(&s[start..i]);
        let mut end = i + c.len_utf8();
        if c == '\r' && matches!(chars.peek(), Some((_, '\n'))) {
            chars.next();
            end += 1;
        }
        start = end;
    }
    if start < s.len() {
        lines.push(&s[start..]);
    }
    lines
}

/// Is `c` inside one of the sorted, inclusive `(lo, hi)` ranges?
fn in_ranges(ranges: &[(u32, u32)], c: char) -> bool {
    let cp = c as u32;
    ranges
        .binary_search_by(|&(lo, hi)| {
            if cp < lo {
                core::cmp::Ordering::Greater
            } else if cp > hi {
                core::cmp::Ordering::Less
            } else {
                core::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

fn is_non_printable(c: char) -> bool {
    in_ranges(&NOT_PRINTABLE, c)
}

/// `repr(s)` for a `str`.
pub fn repr_str(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if is_non_printable(c) => {
                let cp = c as u32;
                if cp <= 0xff {
                    out.push_str(&format!("\\x{cp:02x}"));
                } else if cp <= 0xffff {
                    out.push_str(&format!("\\u{cp:04x}"));
                } else {
                    out.push_str(&format!("\\U{cp:08x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `repr(list_of_str)`, such as `['a', 'b']`.
pub fn repr_str_list<S: AsRef<str>>(items: &[S]) -> String {
    let parts: Vec<String> = items.iter().map(|s| repr_str(s.as_ref())).collect();
    format!("[{}]", parts.join(", "))
}

/// A Unicode decimal digit as the ASCII digit of the same value; any other character unchanged.
fn decimal_to_ascii(c: char) -> char {
    let cp = c as u32;
    if c.is_ascii() {
        return c;
    }
    match DECIMAL_ZEROS.binary_search(&cp) {
        Ok(_) => '0',
        Err(0) => c,
        Err(i) => {
            let offset = cp - DECIMAL_ZEROS[i - 1];
            if offset < 10 {
                char::from(b'0' + offset as u8)
            } else {
                c
            }
        }
    }
}

/// `float(s)` for a `str`; `None` where Python raises `ValueError`.
pub fn parse_float(s: &str) -> Option<f64> {
    // Python reads any Unicode decimal digit as its ASCII digit before it parses.
    // Python turns non-ASCII whitespace into a space, then strips only the ASCII C whitespace
    // (` \t\n\r\v\f`). So `\x1c`-`\x1f` are whitespace for `str.strip` but not for `float`.
    let ascii: String = s
        .chars()
        .map(|c| {
            if !c.is_ascii() && is_space(c) {
                ' '
            } else {
                decimal_to_ascii(c)
            }
        })
        .collect();
    let t = ascii.trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{b}' | '\u{c}'));
    if t.is_empty() {
        return None;
    }
    // Underscores are allowed only between digits.
    let chars: Vec<char> = t.chars().collect();
    let mut cleaned = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c == '_' {
            let before = i > 0 && chars[i - 1].is_ascii_digit();
            let after = i + 1 < chars.len() && chars[i + 1].is_ascii_digit();
            if !(before && after) {
                return None;
            }
        } else {
            cleaned.push(c);
        }
    }
    // Rust accepts a few spellings Python rejects: keep to ASCII digits, sign, point, exponent,
    // and the words inf / infinity / nan.
    let lower = cleaned.to_ascii_lowercase();
    let body = lower.trim_start_matches(['+', '-']);
    if lower.len() - body.len() > 1 {
        return None;
    }
    if matches!(body, "inf" | "infinity" | "nan") {
        return lower.parse::<f64>().ok();
    }
    if body.is_empty()
        || !body
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | '+' | '-'))
    {
        return None;
    }
    lower.parse::<f64>().ok()
}

/// `s.isidentifier()`: XID_Start (or `_`) first, then XID_Continue, as Python's database has them.
pub fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if in_ranges(&IDENTIFIER_START, c) => {}
        _ => return false,
    }
    chars.all(|c| in_ranges(&IDENTIFIER_CONTINUE, c))
}
