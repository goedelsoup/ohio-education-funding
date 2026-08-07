//! A pull parser for the subset of XML that SpreadsheetML uses.
//!
//! Not a general XML implementation and not trying to be. It handles elements, attributes,
//! text, CDATA, comments, and the five predefined entities plus numeric references — which is
//! everything the department's workbooks contain. It does not validate, resolve external
//! entities, or track namespace bindings.
//!
//! # Namespaces are stripped, not resolved
//!
//! SpreadsheetML puts everything in one default namespace and uses exactly one prefixed
//! attribute, `r:id`. Rather than carry a binding table for that, this parser drops prefixes:
//! `<x:row>` and `<row>` both report as `row`, and `r:id` is looked up as `id`. The cost is
//! that a document mixing two namespaces with colliding local names would be misread. No
//! spreadsheet does, and the alternative is a namespace stack for one attribute.
//!
//! Events borrow from the input, so parsing a 40 MB sheet allocates only where an entity is
//! actually decoded.

use std::borrow::Cow;

/// Something the parser found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event<'a> {
    /// `<name ...>` — a tag with children to come.
    Open(Tag<'a>),
    /// `<name ... />` — a tag with no children.
    Empty(Tag<'a>),
    /// `</name>`.
    Close(&'a str),
    /// Character data between tags, still entity-encoded. Use [`decode_entities`].
    Text(&'a str),
}

/// An element name and its raw attribute text.
///
/// Attributes are scanned on demand rather than collected into a map, because a sheet has
/// millions of cells and almost every one is asked for at most two of its attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag<'a> {
    /// Local name, with any namespace prefix removed.
    pub name: &'a str,
    attributes: &'a str,
}

impl<'a> Tag<'a> {
    /// The value of an attribute by local name, entity-decoded.
    ///
    /// A namespace prefix on the attribute is ignored, so `r:id` answers to `"id"`.
    #[must_use]
    pub fn attr(&self, name: &str) -> Option<Cow<'a, str>> {
        let mut rest = self.attributes;
        while let Some(equals) = rest.find('=') {
            let key = rest[..equals].trim();
            let key = key.rsplit(':').next().unwrap_or(key);
            let after = &rest[equals + 1..];
            let after = after.trim_start();
            let quote = after.as_bytes().first().copied()?;
            if quote != b'"' && quote != b'\'' {
                return None;
            }
            let body = &after[1..];
            let end = body.find(quote as char)?;
            if key == name {
                return Some(decode_entities(&body[..end]));
            }
            rest = &body[end + 1..];
        }
        None
    }
}

/// A cursor over an XML document.
pub struct Reader<'a> {
    source: &'a str,
    position: usize,
}

impl<'a> Reader<'a> {
    /// Start reading a document.
    #[must_use]
    pub const fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    /// The next event, or `None` at end of document.
    ///
    /// Whitespace-only text between tags is skipped; SpreadsheetML never carries meaning in
    /// it, and reporting it would double the event count on a pretty-printed file.
    pub fn next_event(&mut self) -> Option<Event<'a>> {
        loop {
            if self.position >= self.source.len() {
                return None;
            }
            let rest = &self.source[self.position..];
            if !rest.starts_with('<') {
                let end = rest.find('<').unwrap_or(rest.len());
                let text = &rest[..end];
                self.position += end;
                if !text.trim().is_empty() {
                    return Some(Event::Text(text));
                }
                continue;
            }

            if rest.starts_with("<!--") {
                self.position += rest.find("-->").map_or(rest.len(), |at| at + 3);
                continue;
            }
            if let Some(body) = rest.strip_prefix("<![CDATA[") {
                let end = body.find("]]>").unwrap_or(body.len());
                self.position += "<![CDATA[".len() + end + "]]>".len();
                // CDATA is returned verbatim; decode_entities leaves it alone because it
                // contains no entities by definition.
                return Some(Event::Text(&body[..end]));
            }
            if rest.starts_with("<?") || rest.starts_with("<!") {
                self.position += rest.find('>').map_or(rest.len(), |at| at + 1);
                continue;
            }

            let end = rest.find('>')?;
            let inner = &rest[1..end];
            self.position += end + 1;

            if let Some(name) = inner.strip_prefix('/') {
                return Some(Event::Close(local_name(name.trim())));
            }
            let (inner, empty) = match inner.strip_suffix('/') {
                Some(trimmed) => (trimmed, true),
                None => (inner, false),
            };
            let split = inner
                .find(|c: char| c.is_ascii_whitespace())
                .unwrap_or(inner.len());
            let tag = Tag {
                name: local_name(&inner[..split]),
                attributes: &inner[split..],
            };
            return Some(if empty {
                Event::Empty(tag)
            } else {
                Event::Open(tag)
            });
        }
    }

    /// Consume events until the currently open element named `name` closes, returning the
    /// concatenated text of every descendant.
    ///
    /// This is how a shared string or an inline string is read: its text may be split across
    /// several `<t>` runs, and the runs must be joined in document order.
    pub fn text_until_close(&mut self, name: &str) -> String {
        let mut depth = 0usize;
        let mut out = String::new();
        while let Some(event) = self.next_event() {
            match event {
                Event::Open(tag) if tag.name == name => depth += 1,
                Event::Close(closing) if closing == name => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                Event::Text(text) => out.push_str(&decode_entities(text)),
                _ => {}
            }
        }
        out
    }
}

fn local_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

/// Replace XML entity references with the characters they stand for.
///
/// Borrows unchanged when there is nothing to replace, which is the common case.
#[must_use]
pub fn decode_entities(text: &str) -> Cow<'_, str> {
    if !text.contains('&') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let Some(semicolon) = tail.find(';') else {
            out.push_str(tail);
            return Cow::Owned(out);
        };
        let entity = &tail[1..semicolon];
        match entity {
            "amp" => out.push('&'),
            "lt" => out.push('<'),
            "gt" => out.push('>'),
            "quot" => out.push('"'),
            "apos" => out.push('\''),
            _ => {
                let decoded = entity
                    .strip_prefix('#')
                    .and_then(|digits| match digits.strip_prefix(['x', 'X']) {
                        Some(hex) => u32::from_str_radix(hex, 16).ok(),
                        None => digits.parse::<u32>().ok(),
                    })
                    .and_then(char::from_u32);
                match decoded {
                    Some(c) => out.push(c),
                    // An entity this parser does not know is left as written rather than
                    // dropped: a district name is better wrong-looking than silently altered.
                    None => out.push_str(&tail[..=semicolon]),
                }
            }
        }
        rest = &tail[semicolon + 1..];
    }
    out.push_str(rest);
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn events(source: &str) -> Vec<Event<'_>> {
        let mut reader = Reader::new(source);
        let mut out = Vec::new();
        while let Some(event) = reader.next_event() {
            out.push(event);
        }
        out
    }

    #[test]
    fn reads_open_text_and_close() {
        let out = events("<a><b>text</b></a>");
        assert_eq!(out.len(), 5);
        assert!(matches!(&out[0], Event::Open(t) if t.name == "a"));
        assert_eq!(out[2], Event::Text("text"));
        assert_eq!(out[3], Event::Close("b"));
    }

    #[test]
    fn an_empty_element_is_distinct_from_an_open_one() {
        let out = events(r#"<row r="2"/>"#);
        assert_eq!(out.len(), 1);
        assert!(matches!(&out[0], Event::Empty(t) if t.name == "row"));
    }

    #[test]
    fn namespace_prefixes_are_dropped_from_names_and_attributes() {
        let out = events(r#"<x:sheet name="Base_Cost" r:id="rId3"/>"#);
        let Event::Empty(tag) = &out[0] else {
            panic!("expected an empty element")
        };
        assert_eq!(tag.name, "sheet");
        assert_eq!(tag.attr("name").as_deref(), Some("Base_Cost"));
        assert_eq!(tag.attr("id").as_deref(), Some("rId3"));
    }

    #[test]
    fn attributes_may_be_single_quoted_and_spaced() {
        let out = events("<c r = 'A1'  t='s' />");
        let Event::Empty(tag) = &out[0] else {
            panic!("expected an empty element")
        };
        assert_eq!(tag.attr("r").as_deref(), Some("A1"));
        assert_eq!(tag.attr("t").as_deref(), Some("s"));
        assert_eq!(tag.attr("missing"), None);
    }

    #[test]
    fn declarations_doctypes_and_comments_are_skipped() {
        let out = events("<?xml version='1.0'?><!-- note --><!DOCTYPE x><a/>");
        assert_eq!(out.len(), 1);
        assert!(matches!(&out[0], Event::Empty(t) if t.name == "a"));
    }

    #[test]
    fn whitespace_between_tags_is_not_an_event() {
        let out = events("<a>\n  <b/>\n</a>");
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn decodes_the_predefined_entities() {
        assert_eq!(
            decode_entities("a &amp; b &lt;c&gt; &quot;d&quot;"),
            "a & b <c> \"d\""
        );
        assert_eq!(decode_entities("it&apos;s"), "it's");
    }

    #[test]
    fn decodes_numeric_and_hex_references() {
        assert_eq!(decode_entities("caf&#233;"), "café");
        assert_eq!(decode_entities("caf&#xE9;"), "café");
    }

    #[test]
    fn text_with_no_entity_is_borrowed_not_copied() {
        assert!(matches!(decode_entities("plain"), Cow::Borrowed(_)));
    }

    #[test]
    fn an_unknown_entity_survives_rather_than_vanishing() {
        assert_eq!(decode_entities("a&nbsp;b"), "a&nbsp;b");
        assert_eq!(decode_entities("bare & ampersand"), "bare & ampersand");
    }

    #[test]
    fn joins_text_runs_across_child_elements() {
        // A shared string split by formatting is stored as several <t> runs and must be
        // rejoined in order.
        let mut reader = Reader::new("<si><r><t>Ada </t></r><r><t>Exempted</t></r></si>rest");
        assert!(matches!(reader.next_event(), Some(Event::Open(t)) if t.name == "si"));
        assert_eq!(reader.text_until_close("si"), "Ada Exempted");
        assert_eq!(reader.next_event(), Some(Event::Text("rest")));
    }

    #[test]
    fn nested_elements_of_the_same_name_do_not_end_the_scan_early() {
        let mut reader = Reader::new("<is><is>inner</is>outer</is>");
        assert!(matches!(reader.next_event(), Some(Event::Open(_))));
        assert_eq!(reader.text_until_close("is"), "innerouter");
    }

    #[test]
    fn cdata_is_returned_verbatim() {
        let out = events("<t><![CDATA[a < b & c]]></t>");
        assert_eq!(out[1], Event::Text("a < b & c"));
    }
}
