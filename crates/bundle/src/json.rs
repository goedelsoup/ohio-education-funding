//! A JSON writer that closes what it opens.
//!
//! [`super::Bundle::to_json`] maintained the document's punctuation by hand across 169 sites:
//! 84 `push_str(&format!(…))`, 68 bare `push_str("`, 17 raw pushes of a brace or a comma,
//! separators in four incompatible styles, and two places that wrote a trailing `", "` and then
//! chopped it back off with `truncate`. [`super::fields`] carries the scar in its doc comment —
//! the first version left its object open so a caller could add one more field, three of five
//! callers forgot to close it, and `career_technical` ended up nested inside
//! `special_education`.
//!
//! The fix is structural rather than disciplinary. [`Obj`] and [`Arr`] write their closing
//! delimiter in `Drop`, so an unbalanced document is not something a caller can express; and
//! each tracks whether it has written a member, so a separator is not something a caller can
//! get wrong. Neither is a rule to follow — both are the only thing the type permits. There are
//! now zero hand-typed delimiters in the serializer.
//!
//! # Format
//!
//! The emitted bytes match what this feed has always emitted, because the committed feed is
//! the regression test: `": "` after every key, and `", "` between members of a container laid
//! out on one line.
//!
//! Two layouts, because the feed has two. The outer document puts one member per line so that
//! 6.19 MB of it diffs legibly; a district puts its sixty-odd fields on one line for the same
//! reason, since a district that changed should be one changed line. [`Obj::block`] and
//! [`Arr::block`] write the first, [`Obj::new`] and [`Arr::new`] the second, and the line breaks
//! that used to be typed into a hundred format strings are now a property of the container.

use crate::serialize::{escape, num, opt, share};
use core::fmt::Write;

/// How a container lays its members out.
#[derive(Clone, Copy)]
enum Layout {
    /// All on one line, `", "` between members.
    Inline,
    /// One member per line, indented by `pad`, with the closing delimiter two spaces back.
    ///
    /// Two spaces because that is the feed's indent unit, and the closing delimiter of a block
    /// sits at its *parent's* level — `pad` is the members' level, so the close is `pad` less
    /// one unit. A block opened at the document root has `pad = "  "` and closes at column zero.
    Block { pad: &'static str },
}

impl Layout {
    /// Write whatever comes before the next member: a separator if one is due, then the indent.
    fn open_slot(self, out: &mut String, first: bool) {
        match self {
            Self::Inline => {
                if !first {
                    out.push_str(", ");
                }
            }
            Self::Block { pad } => {
                if !first {
                    out.push(',');
                }
                out.push('\n');
                out.push_str(pad);
            }
        }
    }

    /// Write whatever comes before the closing delimiter.
    ///
    /// Nothing for an inline container. For a block, the line break and the closing indent —
    /// *including when the block is empty*, which is not an aesthetic choice: two tests in
    /// `tests/serialization.rs` assert that an absent history serializes as `"history": [\n  ],`
    /// rather than as a missing key, and `[]` would fail both. An empty container that still
    /// occupies its two lines is how this feed says "asked and empty" rather than "not asked".
    fn close_slot(self, out: &mut String) {
        if let Self::Block { pad } = self {
            out.push('\n');
            out.push_str(pad.get(2..).unwrap_or_default());
        }
    }
}

/// An open JSON object. Closes itself.
pub(crate) struct Obj<'a> {
    out: &'a mut String,
    first: bool,
    layout: Layout,
}

/// An open JSON array. Closes itself.
pub(crate) struct Arr<'a> {
    out: &'a mut String,
    first: bool,
    layout: Layout,
}

impl<'a> Obj<'a> {
    /// Open an object at the end of `out`, all on one line.
    pub(crate) fn new(out: &'a mut String) -> Self {
        Self::with(out, Layout::Inline)
    }

    /// Open an object whose members each get a line, indented by `pad`.
    pub(crate) fn block(out: &'a mut String, pad: &'static str) -> Self {
        Self::with(out, Layout::Block { pad })
    }

    fn with(out: &'a mut String, layout: Layout) -> Self {
        out.push('{');
        Self {
            out,
            first: true,
            layout,
        }
    }

    /// Write a key, with whatever separator and indent are due before it.
    fn key(&mut self, k: &str) {
        self.layout.open_slot(self.out, self.first);
        self.first = false;
        let _ = write!(self.out, "\"{k}\": ");
    }

    /// A number, to four places, trailing zeros trimmed.
    pub(crate) fn num(&mut self, k: &str, v: f64) {
        self.key(k);
        self.out.push_str(&num(v));
    }

    /// A string, escaped.
    pub(crate) fn text(&mut self, k: &str, v: &str) {
        self.key(k);
        let _ = write!(self.out, "\"{}\"", escape(v));
    }

    /// Raw text as a member's value, for the one figure the feed rounds its own way.
    pub(crate) fn raw(&mut self, k: &str, v: &str) {
        self.key(k);
        self.out.push_str(v);
    }

    /// A count or a year, written as an integer.
    ///
    /// Not routed through [`super::num`]: an exact quantity should not acquire a decimal
    /// representation on the way out, even one that trims back to the same digits.
    pub(crate) fn count(&mut self, k: &str, v: impl core::fmt::Display) {
        self.key(k);
        let _ = write!(self.out, "{v}");
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

    /// A count that may be absent, emitted as `null` when it is.
    pub(crate) fn opt_count(&mut self, k: &str, v: Option<impl core::fmt::Display>) {
        self.key(k);
        match v {
            Some(v) => {
                let _ = write!(self.out, "{v}");
            }
            None => self.out.push_str("null"),
        }
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

    /// A nested object laid out one member per line.
    pub(crate) fn block_obj(&mut self, k: &str, pad: &'static str) -> Obj<'_> {
        self.key(k);
        Obj::block(self.out, pad)
    }

    /// A nested array laid out one element per line.
    pub(crate) fn block_arr(&mut self, k: &str, pad: &'static str) -> Arr<'_> {
        self.key(k);
        Arr::block(self.out, pad)
    }
}

impl Drop for Obj<'_> {
    fn drop(&mut self) {
        self.layout.close_slot(self.out);
        self.out.push('}');
    }
}

impl<'a> Arr<'a> {
    /// Open an array at the end of `out`, all on one line.
    pub(crate) fn new(out: &'a mut String) -> Self {
        Self::with(out, Layout::Inline)
    }

    /// Open an array whose elements each get a line, indented by `pad`.
    pub(crate) fn block(out: &'a mut String, pad: &'static str) -> Self {
        Self::with(out, Layout::Block { pad })
    }

    fn with(out: &'a mut String, layout: Layout) -> Self {
        out.push('[');
        Self {
            out,
            first: true,
            layout,
        }
    }

    /// Write whatever separator and indent are due, then hand the buffer to the caller.
    fn slot(&mut self) {
        self.layout.open_slot(self.out, self.first);
        self.first = false;
    }

    /// A number element.
    pub(crate) fn num(&mut self, v: f64) {
        self.slot();
        self.out.push_str(&num(v));
    }

    /// A count or a year element, written as an integer.
    ///
    /// As [`Obj::count`]: an exact quantity should not pick up a decimal representation on the
    /// way out.
    pub(crate) fn count(&mut self, v: impl core::fmt::Display) {
        self.slot();
        let _ = write!(self.out, "{v}");
    }

    /// An object element, which closes when the returned value is dropped.
    pub(crate) fn obj(&mut self) -> Obj<'_> {
        self.slot();
        Obj::new(self.out)
    }
}

impl Drop for Arr<'_> {
    fn drop(&mut self) {
        self.layout.close_slot(self.out);
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
