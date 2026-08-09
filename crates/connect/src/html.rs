//! Enough HTML to read a statute, and no more.
//!
//! `codes.ohio.gov` serves the Revised Code as **server-rendered HTML**. That is the whole reason
//! the `ohio-laws` connector exists: its recorded blocker was "serves HTML with no bulk export",
//! which is a statement about the absence of a convenience and was read for two years as a
//! statement about the absence of the data. The text is there, in the response body, for anyone
//! who fetches it.
//!
//! # Why this is a text extractor and not an HTML parser
//!
//! A parser would build a tree. Nothing here wants a tree — it wants the words, in order, with the
//! markup gone. So this does the four things that separates the two:
//!
//!   1. drops `<script>` and `<style>` **including their contents**, which a naive tag-stripper
//!      leaves behind as a page full of JavaScript;
//!   2. turns **block** tags into line breaks, so structure survives as newlines instead of
//!      running words together — while leaving **inline** tags transparent, because a statutory
//!      cross-reference is a link and breaking on it splits one citation across three lines;
//!   3. unescapes the entities the site actually emits;
//!   4. collapses the whitespace that step two produces.
//!
//! It would be wrong for a document whose meaning depends on nesting. A statute's does not: its
//! structure is in its own numbering — `(A)`, `(1)`, `(a)` — which is text, and which survives.

/// Elements that sit *inside* a sentence rather than delimiting one.
///
/// The distinction earns its keep on this source. A statute cross-reference is marked up as
/// `section <a>3317.011</a> of the Revised Code`, so treating every tag as a break splits one
/// clause across three lines and makes the citation unsearchable. Treating these as transparent
/// keeps the sentence whole; everything else still breaks, so `(A)` and `(B)` stay on their own
/// lines and the statute's numbering survives as structure.
const INLINE: &[&str] = &[
    "a", "span", "em", "strong", "b", "i", "u", "sup", "sub", "small", "abbr", "cite", "code",
];

/// Strip markup from a fragment of HTML, leaving the readable text.
///
/// Block structure becomes newlines, inline markup closes up, and runs of whitespace collapse to
/// one space per line.
#[must_use]
pub fn to_text(source: &str) -> String {
    let without_code = drop_elements(source, &["script", "style", "noscript"]);
    let mut out = String::with_capacity(without_code.len() / 2);
    let mut tag = String::new();
    let mut in_tag = false;
    for ch in without_code.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                if !is_inline(&tag) {
                    out.push('\n');
                }
            }
            _ if in_tag => tag.push(ch),
            _ => out.push(ch),
        }
    }
    let unescaped = unescape(&out);
    unescaped
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether a tag's name — with any leading slash and trailing attributes removed — is inline.
fn is_inline(tag: &str) -> bool {
    let name = tag
        .trim_start_matches('/')
        .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .next()
        .unwrap_or_default();
    INLINE.iter().any(|i| i.eq_ignore_ascii_case(name))
}

/// Remove named elements together with everything between their tags.
///
/// Matches the opening tag by prefix so that `<script async src=…>` is caught as readily as
/// `<script>`. An unclosed element takes the rest of the document with it, which is the safe
/// direction: leaving a page of minified JavaScript in the output is worse than losing a tail.
fn drop_elements(source: &str, names: &[&str]) -> String {
    let mut out = String::with_capacity(source.len());
    let lowered = source.to_ascii_lowercase();
    let mut i = 0;
    'outer: while i < source.len() {
        for name in names {
            let open = format!("<{name}");
            if lowered[i..].starts_with(&open) {
                let close = format!("</{name}>");
                match lowered[i..].find(&close) {
                    Some(end) => i += end + close.len(),
                    None => break 'outer,
                }
                continue 'outer;
            }
        }
        let ch = source[i..].chars().next().expect("in bounds");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Resolve the character entities this source emits, plus numeric ones.
fn unescape(source: &str) -> String {
    if !source.contains('&') {
        return source.to_string();
    }
    let mut out = String::with_capacity(source.len());
    let bytes: Vec<char> = source.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != '&' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        // An entity is short; anything longer is a stray ampersand.
        let end = (i + 1..bytes.len().min(i + 12)).find(|j| bytes[*j] == ';');
        let Some(end) = end else {
            out.push('&');
            i += 1;
            continue;
        };
        let name: String = bytes[i + 1..end].iter().collect();
        let resolved = match name.as_str() {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
            "nbsp" | "#160" => Some(' '),
            "mdash" => Some('—'),
            "ndash" => Some('–'),
            "rsquo" => Some('\''),
            "lsquo" => Some('\''),
            "ldquo" => Some('"'),
            "rdquo" => Some('"'),
            "hellip" => Some('…'),
            "sect" => Some('§'),
            other => other
                .strip_prefix('#')
                .and_then(|digits| digits.parse::<u32>().ok())
                .and_then(char::from_u32),
        };
        match resolved {
            Some(ch) => {
                out.push(ch);
                i = end + 1;
            }
            None => {
                out.push('&');
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_contents_do_not_survive_as_text() {
        // The failure a naive tag-stripper produces: a page of JavaScript reading as prose.
        let html = "<p>before</p><script>var x = 1 < 2;</script><p>after</p>";
        let text = to_text(html);
        assert!(text.contains("before") && text.contains("after"));
        assert!(!text.contains("var x"), "{text}");
    }

    #[test]
    fn tags_become_boundaries_rather_than_disappearing() {
        // `<p>one</p><p>two</p>` must not read as "onetwo".
        let text = to_text("<p>one</p><p>two</p>");
        assert_eq!(text, "one\ntwo");
    }

    #[test]
    fn an_inline_link_does_not_split_the_sentence_around_it() {
        // The case this distinction exists for: a statutory cross-reference is a link, and
        // breaking on it turns one citation into three lines and makes it unsearchable.
        let html = r#"<p>under section <a href="/x">3317.011</a> of the Revised Code.</p>"#;
        assert_eq!(to_text(html), "under section 3317.011 of the Revised Code.");
    }

    #[test]
    fn block_structure_still_breaks_so_the_statutes_numbering_survives() {
        let html = "<p>(A) first</p><div>(B) second</div>";
        assert_eq!(to_text(html), "(A) first\n(B) second");
    }

    #[test]
    fn an_attribute_containing_an_angle_bracket_word_is_still_dropped() {
        let text = to_text(r#"<div class="a"><span data-x="y">kept</span></div>"#);
        assert_eq!(text, "kept");
    }

    #[test]
    fn the_entities_a_statute_page_uses_are_resolved() {
        let text =
            to_text("<p>A&nbsp;&amp;&nbsp;B &sect;&#160;3317.013 &quot;x&quot; &#39;y&#39;</p>");
        assert_eq!(text, "A & B § 3317.013 \"x\" 'y'");
    }

    #[test]
    fn a_bare_ampersand_survives_rather_than_eating_the_line() {
        assert_eq!(
            to_text("<p>Ohio & Erie, 5 &lt; 6</p>"),
            "Ohio & Erie, 5 < 6"
        );
    }

    #[test]
    fn an_unclosed_script_takes_the_tail_rather_than_leaking_code() {
        let text = to_text("<p>kept</p><script>never { closed");
        assert_eq!(text, "kept");
    }

    #[test]
    fn runs_of_whitespace_inside_a_line_collapse() {
        assert_eq!(to_text("<p>a    b\t\tc</p>"), "a b c");
    }
}
