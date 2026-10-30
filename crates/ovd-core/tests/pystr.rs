//! Seam: `ovd_core::pystr`. Expected values are what CPython 3 prints for the same input.

use ovd_core::pystr::*;

#[test]
fn repr_picks_quotes_as_python_does() {
    assert_eq!(repr_str("it's"), "\"it's\"");
    assert_eq!(repr_str("a\"b'c"), "'a\"b\\'c'");
    assert_eq!(repr_str("tab\there\u{1}"), "'tab\\there\\x01'");
    assert_eq!(repr_str("x\\y"), "'x\\\\y'");
}

#[test]
fn repr_of_a_list_of_text() {
    assert_eq!(repr_str_list(&["a", "b'"]), "['a', \"b'\"]");
}

#[test]
fn splitlines_matches_python() {
    assert_eq!(splitlines("a\r\nb\rc\u{b}d\n"), vec!["a", "b", "c", "d"]);
    assert_eq!(splitlines(""), Vec::<&str>::new());
    assert_eq!(splitlines("a\n\nb"), vec!["a", "", "b"]);
}

#[test]
fn float_parsing_matches_python() {
    assert_eq!(parse_float("1_000"), Some(1000.0));
    assert_eq!(parse_float("1__0"), None);
    assert_eq!(parse_float("_1"), None);
    assert_eq!(parse_float("1e3"), Some(1000.0));
    assert_eq!(parse_float(" 2.5 "), Some(2.5));
    assert_eq!(parse_float("--1"), None);
    assert_eq!(parse_float("+inf"), Some(f64::INFINITY));
    assert_eq!(parse_float("Infinity"), Some(f64::INFINITY));
    assert!(parse_float("nan").is_some_and(f64::is_nan));
    assert_eq!(parse_float("1."), Some(1.0));
    assert_eq!(parse_float(".5"), Some(0.5));
    assert_eq!(parse_float("e5"), None);
    assert_eq!(parse_float("0x10"), None);
    assert_eq!(parse_float(""), None);
}

#[test]
fn python_whitespace_is_not_rust_whitespace() {
    // `\x1f` is whitespace to `str.isspace` but not to Rust's `char::is_whitespace`.
    assert_eq!(strip("\u{1f} a \u{1f}"), "a");
    assert!(is_identifier("rate_1"));
    assert!(!is_identifier("1rate"));
    assert!(!is_identifier("a-b"));
    assert!(!is_identifier(""));
}

#[test]
fn repr_escapes_what_python_calls_non_printable() {
    assert_eq!(repr_str("\u{a0}"), "'\\xa0'");
    assert_eq!(repr_str("\u{7f}"), "'\\x7f'");
    assert_eq!(repr_str("\u{85}"), "'\\x85'");
    assert_eq!(repr_str("\u{1f}"), "'\\x1f'");
    assert_eq!(repr_str("a\u{ad}b"), "'a\\xadb'");
    assert_eq!(repr_str("\u{200b}"), "'\\u200b'");
    assert_eq!(repr_str("\u{2028}"), "'\\u2028'");
}

#[test]
fn repr_keeps_printable_non_ascii() {
    assert_eq!(repr_str("\u{100}x"), "'\u{100}x'");
    assert_eq!(repr_str("\u{e9}"), "'\u{e9}'");
    assert_eq!(repr_str("\u{1f600}"), "'\u{1f600}'");
}

#[test]
fn more_float_spellings_match_python() {
    assert_eq!(parse_float("+-1"), None);
    assert_eq!(parse_float("-+1"), None);
    assert_eq!(parse_float("+"), None);
    assert_eq!(parse_float("-"), None);
    assert_eq!(parse_float("1e+5"), Some(100000.0));
    assert_eq!(parse_float("1e-5"), Some(1e-5));
    assert_eq!(parse_float("1a"), None);
    assert_eq!(parse_float("e"), None);
    assert_eq!(parse_float("+1"), Some(1.0));
    assert_eq!(parse_float("-.5"), Some(-0.5));
    assert_eq!(parse_float("1e"), None);
    assert_eq!(parse_float("inf1"), None);
    assert_eq!(parse_float("-inf"), Some(f64::NEG_INFINITY));
    assert!(parse_float("+nan").is_some_and(f64::is_nan));
    assert_eq!(parse_float("1.5.2"), None);
    assert_eq!(parse_float("+ 1"), None);
}

#[test]
fn identifiers_follow_pythons_unicode_tables() {
    assert!(is_identifier("caf\u{e9}"));
    assert!(is_identifier("a\u{301}")); // a + combining acute: XID_Continue
    assert!(is_identifier("_x1"));
    assert!(is_identifier("\u{6c34}\u{6e29}")); // CJK letters
    assert!(!is_identifier("a\u{b2}")); // superscript two is No, not XID_Continue
    assert!(!is_identifier("\u{301}a")); // a combining mark cannot start one
    assert!(!is_identifier("1a"));
    assert!(!is_identifier(""));
    assert!(!is_identifier("a b"));
}

#[test]
fn float_reads_unicode_decimal_digits() {
    assert_eq!(parse_float("\u{661}\u{662}"), Some(12.0)); // Arabic-Indic
    assert_eq!(parse_float("\u{967}.\u{6f5}"), Some(1.5)); // Devanagari, extended Arabic-Indic
    assert_eq!(parse_float("\u{ff11}e\u{ff12}"), Some(100.0)); // fullwidth
    assert_eq!(parse_float("\u{b2}"), None); // superscript is not a decimal digit
    assert_eq!(parse_float("1\u{a0}2"), None);
    assert_eq!(parse_float("\u{a0}12\u{3000}"), Some(12.0)); // surrounding space is stripped
}

#[test]
fn repr_escapes_what_python_calls_not_printable() {
    assert_eq!(repr_str("caf\u{e9}"), "'caf\u{e9}'");
    assert_eq!(repr_str("a\u{2003}b"), "'a\\u2003b'");
    assert_eq!(repr_str("\u{378}"), "'\\u0378'"); // unassigned
    assert_eq!(repr_str("\u{e000}"), "'\\ue000'"); // private use
    assert_eq!(repr_str("\u{1f600}"), "'\u{1f600}'");
    assert_eq!(repr_str("\u{e0001}"), "'\\U000e0001'"); // a tag character
}

#[test]
fn float_strips_only_ascii_c_whitespace_and_reads_other_spaces_as_blanks() {
    assert_eq!(parse_float("\u{1f}1"), None);
    assert_eq!(parse_float("1\u{1c}"), None);
    assert_eq!(parse_float(" \t\n\r\u{b}\u{c}1\u{b}"), Some(1.0));
    assert_eq!(parse_float("\u{a0}1\u{85}\u{2003}"), Some(1.0));
}
