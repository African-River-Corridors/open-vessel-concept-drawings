//! Message Ids beside the engine's English (ADR 0014, spec 0005).
//!
//! The core writes its messages in English, and that English is part of the results (and of their
//! hash). It does no translation and takes no language. Beside each English message it records a
//! [`Msg`]: the Message Id in the Text Library (`locales/en/engine.ftl`) and the typed arguments, so
//! the edge (the app) can say the same thing in another language. The [`text!`](crate::text!) macro
//! writes both from one call, so the English cannot drift from its id.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt;
use core::ops::Deref;

/// One argument of a message.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    /// A measured number: its value, and its text as the English prints it. A translation shows the
    /// same digits with its own decimal sign.
    Num { value: f64, text: String },
    /// A whole count, for plural forms.
    Count(i64),
    /// Text the engine took from its inputs or data (an Option, Cargo or tank name).
    Text(String),
    /// A rule id, clause citation or quoted rule text: never translated.
    Cite(String),
    /// Another message, translated in its turn (a condition or Cargo name).
    Msg(Msg),
    /// The key of an input, shown as its label. The English writes the key in backticks (the
    /// template has them around the placeholder); a key that is not an input (a file key, a
    /// profile key, a command) stays as the key.
    Input(String),
}

impl Arg {
    /// A number with the text the English shows for it.
    pub fn num(value: f64, text: impl Into<String>) -> Self {
        Arg::Num {
            value,
            text: text.into(),
        }
    }

    /// A number as the English prints it to `places` decimals (Python `f"{x:.{places}f}"`).
    pub fn fixed(value: f64, places: usize) -> Self {
        Arg::num(value, crate::pyfmt::fixed(value, places))
    }

    /// A number as the English prints it in full (Python `str(x)`).
    pub fn py(value: f64) -> Self {
        Arg::num(value, crate::pyfmt::py_str(value))
    }

    pub fn cite(text: impl Into<String>) -> Self {
        Arg::Cite(text.into())
    }

    /// The key of an input (see [`Arg::Input`]).
    pub fn input(key: impl Into<String>) -> Self {
        Arg::Input(key.into())
    }

    /// The text this argument puts in the English message.
    pub fn english(&self) -> String {
        match self {
            Arg::Num { text, .. } => text.clone(),
            Arg::Count(n) => n.to_string(),
            Arg::Text(t) | Arg::Cite(t) | Arg::Input(t) => t.clone(),
            Arg::Msg(m) => m.english.clone(),
        }
    }
}

impl From<String> for Arg {
    fn from(t: String) -> Self {
        Arg::Text(t)
    }
}

impl From<&str> for Arg {
    fn from(t: &str) -> Self {
        Arg::Text(t.into())
    }
}

impl From<&String> for Arg {
    fn from(t: &String) -> Self {
        Arg::Text(t.clone())
    }
}

impl From<usize> for Arg {
    fn from(n: usize) -> Self {
        Arg::Count(n as i64)
    }
}

impl From<i64> for Arg {
    fn from(n: i64) -> Self {
        Arg::Count(n)
    }
}

impl From<Msg> for Arg {
    fn from(m: Msg) -> Self {
        Arg::Msg(m)
    }
}

impl From<Text> for Arg {
    fn from(t: Text) -> Self {
        match t.msg {
            Some(m) => Arg::Msg(m),
            None => Arg::Text(t.en),
        }
    }
}

/// A Message Id with its arguments, and the English it gave.
#[derive(Debug, Clone, PartialEq)]
pub struct Msg {
    pub id: &'static str,
    pub args: Vec<(&'static str, Arg)>,
    /// The English the engine wrote for it.
    pub english: String,
}

/// An English message and, where the engine gives one, its Message Id. It reads, compares and
/// serialises as its English, so the results and their hash are what they were.
#[derive(Debug, Clone, Default)]
pub struct Text {

    pub en: String,
    pub msg: Option<Msg>,
}

impl Text {
    /// English with no Message Id (text from data, or not yet in the Text Library).
    pub fn plain(en: impl Into<String>) -> Self {
        Text {
            en: en.into(),
            msg: None,
        }
    }

    /// Fixed English (a constant) with its Message Id and no arguments.
    pub fn fixed(id: &'static str, en: &str) -> Self {
        Text::from_msg(Msg {
            id,
            args: Vec::new(),
            english: en.into(),
        })
    }

    pub fn from_msg(msg: Msg) -> Self {
        Text {
            en: msg.english.clone(),
            msg: Some(msg),
        }
    }
}

impl Deref for Text {
    type Target = str;
    fn deref(&self) -> &str {
        &self.en
    }
}

impl Borrow<str> for Text {
    fn borrow(&self) -> &str {
        &self.en
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.en)
    }
}

impl From<String> for Text {
    fn from(en: String) -> Self {
        Text::plain(en)
    }
}

impl From<&str> for Text {
    fn from(en: &str) -> Self {
        Text::plain(en)
    }
}

impl From<Text> for String {
    fn from(t: Text) -> Self {
        t.en
    }
}

/// Texts are equal when their English is: the Message Id is a reading aid, not part of the result.
impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.en == other.en
    }
}

impl Eq for Text {}

impl PartialEq<str> for Text {
    fn eq(&self, other: &str) -> bool {
        self.en == other
    }
}

impl PartialEq<&str> for Text {
    fn eq(&self, other: &&str) -> bool {
        self.en == *other
    }
}

impl PartialEq<String> for Text {
    fn eq(&self, other: &String) -> bool {
        &self.en == other
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Text {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.en)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __arg_type {
    ($name:ident) => {
        $crate::text::Arg
    };
}

#[doc(hidden)]
pub mod __private {
    pub use alloc::{format, vec};
}

/// An English message and its Message Id, from one call.
///
/// ```
/// use ovcd_core::text::{Arg, Text};
/// let fb = 1.234_f64;
/// let t: Text = ovcd_core::text!("check-example", "freeboard {fb} m", fb = Arg::num(fb, "1.23"));
/// assert_eq!(t, "freeboard 1.23 m");
/// assert_eq!(t.msg.unwrap().id, "check-example");
/// ```
///
/// The format string is the English, with each argument named; each argument is anything that
/// converts into an [`Arg`]. The English puts in each argument's [`Arg::english`] text.
#[macro_export]
macro_rules! text {
    ($id:literal, $fmt:literal $(, $name:ident = $arg:expr)* $(,)?) => {{
        // Every argument is evaluated before any name is bound, so an argument may use a local
        // variable that has the name of another argument.
        #[allow(unused_parens)]
        let ($($name),*): ($($crate::__arg_type!($name)),*) = ($(::core::convert::Into::<$crate::text::Arg>::into($arg)),*);
        let english = $crate::text::__private::format!($fmt $(, $name = $name.english())*);
        $crate::text::Text::from_msg($crate::text::Msg {
            id: $id,
            args: $crate::text::__private::vec![$((::core::stringify!($name), $name)),*],
            english,
        })
    }};
}
