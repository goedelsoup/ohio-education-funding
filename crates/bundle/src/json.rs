//! A JSON writer that closes what it opens.
//!
//! The serializer this replaces maintained the document's punctuation by hand: 91 sites
//! writing `push_str(&format!(…))`, separators in four incompatible styles, and two places
//! that chopped a closing brace back off with `truncate` so a caller could append one more
//! field. [`super::fields`] carries the scar in its doc comment — the first version left the
//! object open for exactly that reason and three of five callers forgot to close it, nesting
//! `career_technical` inside `special_education`.
//!
//! The fix is structural rather than disciplinary. [`Obj`] and [`Arr`] write their closing
//! delimiter in `Drop`, so an unbalanced document is not something a caller can express; and
//! each tracks whether it has written a member, so a separator is not something a caller can
//! get wrong. Neither is a rule to follow — both are the only thing the type permits.
//!
//! # Format
//!
//! The emitted bytes match what this feed has always emitted, because the committed feed is
//! the regression test: `": "` after every key, `", "` between members, and no whitespace of
//! any other kind. Nothing here pretty-prints — the outer document's line breaks are written
//! by the caller, which is where they were before.

use super::{escape, num, opt, share};
use core::fmt::Write;

/// An open JSON object. Closes itself.
pub(crate) struct Obj<'a> {
    out: &'a mut String,
    first: bool,
}

/// An open JSON array. Closes itself.
pub(crate) struct Arr<'a> {
    out: &'a mut String,
    first: bool,
}

impl<'a> Obj<'a> {
    /// Open an object at the end of `out`.
    pub(crate) fn new(out: &'a mut String) -> Self {
        out.push('{');
        Self { out, first: true }
    }

    /// Write a key, with the separator before it if one is due.
    fn key(&mut self, k: &str) {
        if !self.first {
            self.out.push_str(", ");
        }
        self.first = false;
        let _ = write!(self.out, "\"{k}\": ");
    }

    /// A number, to four places, trailing zeros trimmed.
    pub(crate) fn num(&mut self, k: &str, v: f64) {
        self.key(k);
        self.out.push_str(&num(v));
    }

    /// A number that may be absent, emitted as `null` when it is.
    pub(crate) fn opt(&mut self, k: &str, v: Option<f64>) {
        self.key(k);
        self.out.push_str(&opt(v));
    }

    /// A fraction, to eight places. See [`super::share`] on why not four.
    pub(crate) fn share(&mut self, k: &str, v: f64) {
        self.key(k);
        self.out.push_str(&share(v));
    }

    /// A string, escaped.
    pub(crate) fn text(&mut self, k: &str, v: &str) {
        self.key(k);
        let _ = write!(self.out, "\"{}\"", escape(v));
    }

    /// A count, written as an integer.
    ///
    /// Not routed through [`super::num`]: a count is exact and should not acquire a decimal
    /// representation on the way out, even one that trims back to the same digits.
    pub(crate) fn count(&mut self, k: &str, v: usize) {
        self.key(k);
        let _ = write!(self.out, "{v}");
    }

    /// A boolean.
    pub(crate) fn flag(&mut self, k: &str, v: bool) {
        self.key(k);
        let _ = write!(self.out, "{v}");
    }

    /// A nested object, which closes when the returned value is dropped.
    pub(crate) fn obj(&mut self, k: &str) -> Obj<'_> {
        self.key(k);
        Obj::new(self.out)
    }

    /// A nested array, which closes when the returned value is dropped.
    pub(crate) fn arr(&mut self, k: &str) -> Arr<'_> {
        self.key(k);
        Arr::new(self.out)
    }
}

impl Drop for Obj<'_> {
    fn drop(&mut self) {
        self.out.push('}');
    }
}

impl<'a> Arr<'a> {
    /// Open an array at the end of `out`.
    pub(crate) fn new(out: &'a mut String) -> Self {
        out.push('[');
        Self { out, first: true }
    }

    /// Write the separator if one is due, then hand the buffer to the caller.
    fn slot(&mut self) {
        if !self.first {
            self.out.push_str(", ");
        }
        self.first = false;
    }

    /// A number element.
    pub(crate) fn num(&mut self, v: f64) {
        self.slot();
        self.out.push_str(&num(v));
    }

    /// An object element, which closes when the returned value is dropped.
    pub(crate) fn obj(&mut self) -> Obj<'_> {
        self.slot();
        Obj::new(self.out)
    }
}

impl Drop for Arr<'_> {
    fn drop(&mut self) {
        self.out.push(']');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The emitted bytes are the ones this feed has always emitted.
    ///
    /// `": "` after every key and `", "` between members. These are not cosmetic: the
    /// committed feed is 6.19 MB of them, and a writer that emitted `":"` would rewrite every
    /// byte of it while changing nothing a consumer could see.
    #[test]
    fn the_spacing_is_the_spacing_the_feed_already_uses() {
        let mut s = String::new();
        {
            let mut o = Obj::new(&mut s);
            o.text("irn", "000442");
            o.text("name", "Manchester Local");
            o.num("performance", 29370.23);
        }
        assert_eq!(
            s,
            r#"{"irn": "000442", "name": "Manchester Local", "performance": 29370.23}"#
        );
    }

    /// An object closes when it goes out of scope, whether the caller remembers or not.
    ///
    /// This is the property the whole module exists for. The serializer it replaces had a
    /// call site that chopped a closing brace back off so it could append one more field —
    /// the hazard `super::fields` documents having already caused once, where
    /// `career_technical` ended up nested inside `special_education`.
    #[test]
    fn a_nested_object_closes_without_the_caller_closing_it() {
        let mut s = String::new();
        {
            let mut o = Obj::new(&mut s);
            o.num("before", 1.0);
            {
                let mut inner = o.obj("supplements");
                inner.num("performance", 2.0);
                inner.flag("eligible", true);
                inner.opt("stars", None);
            }
            o.num("after", 3.0);
        }
        assert_eq!(
            s,
            r#"{"before": 1, "supplements": {"performance": 2, "eligible": true, "stars": null}, "after": 3}"#
        );
    }

    /// An empty object and an empty array are still closed, and carry no stray separator.
    #[test]
    fn empty_containers_emit_nothing_between_their_delimiters() {
        let mut s = String::new();
        {
            let mut o = Obj::new(&mut s);
            {
                let _ = o.obj("nothing");
            }
            {
                let _ = o.arr("none");
            }
        }
        assert_eq!(s, r#"{"nothing": {}, "none": []}"#);
    }

    /// An array separates its elements and closes itself, including arrays of objects.
    #[test]
    fn an_array_separates_and_closes() {
        let mut s = String::new();
        {
            let mut o = Obj::new(&mut s);
            {
                let mut a = o.arr("years");
                a.num(2024.0);
                a.num(2025.0);
            }
            {
                let mut a = o.arr("rows");
                {
                    let mut e = a.obj();
                    e.num("v", 1.0);
                }
                {
                    let mut e = a.obj();
                    e.num("v", 2.0);
                }
            }
        }
        assert_eq!(
            s,
            r#"{"years": [2024, 2025], "rows": [{"v": 1}, {"v": 2}]}"#
        );
    }

    /// Text is escaped on the way out, so a district name cannot break the document.
    ///
    /// The feed's names come from CSV fixtures, and `escape`'s control-character arm was
    /// uncovered until now.
    #[test]
    fn a_name_with_a_quote_or_a_newline_does_not_break_the_document() {
        let mut s = String::new();
        {
            let mut o = Obj::new(&mut s);
            o.text("name", "Big \"Walnut\" \\ Local\nDistrict\t2");
        }
        assert_eq!(s, r#"{"name": "Big \"Walnut\" \\ Local\nDistrict\t2"}"#);
    }
}
