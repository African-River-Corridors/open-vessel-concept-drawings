//! Seam: `ovcd_core::pystr`. Expected values are what CPython 3 prints for the same input.

use ovcd_core::pystr::*;

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
