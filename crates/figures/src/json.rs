//! Writing the manifest.
//!
//! Hand-rolled for the reason the feed's serializer is: the workspace has no external
//! dependencies, deliberately, so that a committed result is reproducible years from now without
//! a dependency resolution succeeding first. This document is flatter than the feed — one array
//! of six-field objects — so it does not need [`bundle`'s `Obj`/`Arr`
//! machinery](../../bundle/index.html), and it does not need an escaper either.
//!
//! It needs the *absence* of an escaper to be checked rather than assumed. Every string here is a
//! `&'static str` written in `lib.rs`, so a quote or a backslash in one is a compile-time-fixable
//! typo and not a runtime input — but "nobody would type that" is how a hand-rolled serializer
//! emits invalid JSON eventually. `every_string_is_safe_to_write_unescaped` in `tests/` is the
//! standing check, and a label that fails it should gain a word rather than an escaper.

use core::fmt::Write;

use crate::{compute_all, Unit, CONTRACT_VERSION};

/// The characters that would need escaping, and therefore may not appear in a key or a label.
pub(crate) const UNWRITABLE: &[char] = &['"', '\\'];

/// Whether a string can be written into the manifest without escaping.
///
/// Control characters as well as the two obvious ones: JSON forbids a raw `\n` inside a string,
/// and a label written with a line continuation in the source is the way one would arrive.
#[must_use]
pub(crate) fn writable(text: &str) -> bool {
    !text.contains(UNWRITABLE) && !text.chars().any(char::is_control)
}

/// How a value is written, which is decided by its unit.
///
/// A count is written as an integer because it is one, and a reader diffing the artefact should
/// not have to decide whether `65.0` and `65` are the same figure. Everything else is written at
/// the shortest precision that round-trips, which is what Rust's `{:?}` for `f64` gives.
fn number(unit: Unit, value: f64) -> String {
    assert!(
        value.is_finite(),
        "a figure computed to {value}, which is not a number JSON can carry"
    );
    if unit == Unit::Count {
        assert!(
            value.fract() == 0.0,
            "a count computed to {value}, which is not whole"
        );
        format!("{}", value as i64)
    } else {
        format!("{value:?}")
    }
}

/// The manifest, as it is committed to `crates/figures.json`.
///
/// One member per line at both levels: this is a committed artefact and a figure that moved should
/// be one changed line, the same reason the feed's outer document is laid out that way.
#[must_use]
pub fn manifest() -> String {
    let computed = compute_all();
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"contract\": \"{CONTRACT_VERSION}\",");
    let _ = writeln!(out, "  \"figures\": [");
    for (at, entry) in computed.iter().enumerate() {
        let f = entry.figure;
        assert!(
            writable(f.key) && writable(f.owner) && writable(f.label),
            "{}: a key, owner or label carries a character this writer cannot escape",
            f.key
        );
        let comma = if at + 1 == computed.len() { "" } else { "," };
        let _ = writeln!(
            out,
            "    {{\"key\": \"{}\", \"owner\": \"{}\", \"unit\": \"{}\", \"value\": {}, \
             \"label\": \"{}\"}}{comma}",
            f.key,
            f.owner,
            f.unit.name(),
            number(f.unit, entry.value),
            f.label,
        );
    }
    out.push_str("  ]\n}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_is_written_as_an_integer_and_everything_else_round_trips() {
        assert_eq!(number(Unit::Count, 65.0), "65");
        assert_eq!(number(Unit::Dollars, -44.62), "-44.62");
        assert_eq!(number(Unit::Share, 0.1387), "0.1387");
        // The one that matters: a value the manifest must not round, because the corpus quotes it
        // to the cent and the check compares against what is written here.
        assert_eq!(number(Unit::Dollars, 392_151_306.63), "392151306.63");
    }

    #[test]
    fn a_string_needing_an_escape_is_not_writable() {
        assert!(writable("districts the plan pays more"));
        assert!(!writable("a label with a \" in it"));
        assert!(!writable("a label with a \\ in it"));
        assert!(!writable("a label with a\nnewline"));
    }
}
