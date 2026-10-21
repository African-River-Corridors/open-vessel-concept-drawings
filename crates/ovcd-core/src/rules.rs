//! Rule Sets as plain typed data (spec 0004, ADR 0013): the domain-free model of the `arc-rules`
//! kernel, re-exported so that OVCD code names it as before. The loader in `ovcd-format` reads a
//! Rule Set file; the core parses nothing.

pub use arc_rules::model::*;
