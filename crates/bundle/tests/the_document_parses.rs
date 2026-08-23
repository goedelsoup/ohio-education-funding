//! The feed the crate actually publishes, parsed rather than pattern-matched.
//!
//! Every other test here serializes a hand-written one- or two-district fixture. None of them
//! parses the result: the closest, `a_district_object_repeats_no_key`, walks braces by hand
//! across the first district only and at depth one only.
//!
//! That is how `special_education` and `categoricals` shipped emitted **twice per district** for
//! several contract versions. The golden-file diff in CI catches a figure that *changes*; it
//! cannot catch one that was wrong in the commit that introduced it, because the committed feed
//! and the fresh build agree — both are wrong in the same way.
//!
//! So this parses `build().to_json()` — the real path, the whole document — with a
//! recursive-descent reader that carries no dependency, and asserts two things the pattern
//! matching could not: that the output is well-formed JSON, and that no object repeats a key at
//! any depth.

use std::collections::BTreeSet;

/// Where a fault is, in dotted path form: `districts[17].base_cost_build_up.teachers`.
type Path = String;

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    /// Every duplicated key found, by path. Collected rather than raised, so one run reports
    /// all of them instead of only the first.
    duplicates: Vec<Path>,
    /// How deep the deepest object or array nests, which the assertions below pin.
    depth: usize,
    max_depth: usize,
}

impl<'a> Reader<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            at: 0,
            duplicates: Vec::new(),
            depth: 0,
            max_depth: 0,
        }
    }

    fn fail(&self, what: &str) -> String {
        let from = self.at.saturating_sub(60);
        let to = (self.at + 60).min(self.bytes.len());
        format!(
            "{what} at byte {}: ...{}...",
            self.at,
            String::from_utf8_lossy(&self.bytes[from..to])
        )
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn eat(&mut self, byte: u8) -> Result<(), String> {
        if self.peek() == Some(byte) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.fail(&format!("expected {:?}", byte as char)))
        }
    }

    fn value(&mut self, path: &str) -> Result<(), String> {
        self.space();
        match self.peek() {
            Some(b'{') => self.object(path),
            Some(b'[') => self.array(path),
            Some(b'"') => self.string().map(|_| ()),
            Some(b't') => self.word("true"),
            Some(b'f') => self.word("false"),
            Some(b'n') => self.word("null"),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.fail("expected a value")),
        }
    }

    fn object(&mut self, path: &str) -> Result<(), String> {
        self.eat(b'{')?;
        self.depth += 1;
        self.max_depth = self.max_depth.max(self.depth);
        let mut seen: BTreeSet<String> = BTreeSet::new();
        self.space();
        if self.peek() == Some(b'}') {
            self.at += 1;
            self.depth -= 1;
            return Ok(());
        }
        loop {
            self.space();
            let key = self.string()?;
            if !seen.insert(key.clone()) {
                self.duplicates.push(format!("{path}.{key}"));
            }
            self.space();
            self.eat(b':')?;
            self.value(&format!("{path}.{key}"))?;
            self.space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b'}') => {
                    self.at += 1;
                    self.depth -= 1;
                    return Ok(());
                }
                _ => return Err(self.fail("expected ',' or '}'")),
            }
        }
    }

    fn array(&mut self, path: &str) -> Result<(), String> {
        self.eat(b'[')?;
        self.depth += 1;
        self.max_depth = self.max_depth.max(self.depth);
        self.space();
        if self.peek() == Some(b']') {
            self.at += 1;
            self.depth -= 1;
            return Ok(());
        }
        let mut index = 0usize;
        loop {
            self.value(&format!("{path}[{index}]"))?;
            index += 1;
            self.space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    self.depth -= 1;
                    return Ok(());
                }
                _ => return Err(self.fail("expected ',' or ']'")),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.eat(b'"')?;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err(self.fail("a string never closes")),
                Some(b'"') => {
                    self.at += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.at += 1;
                    let escape = self
                        .peek()
                        .ok_or_else(|| self.fail("a trailing backslash"))?;
                    self.at += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hex = self
                                .bytes
                                .get(self.at..self.at + 4)
                                .ok_or_else(|| self.fail("a truncated \\u escape"))?;
                            let hex = std::str::from_utf8(hex)
                                .map_err(|_| self.fail("a non-ASCII \\u escape"))?;
                            u32::from_str_radix(hex, 16)
                                .map_err(|_| self.fail("a malformed \\u escape"))?;
                            self.at += 4;
                            out.push('\u{fffd}');
                        }
                        other => {
                            return Err(
                                self.fail(&format!("an undefined escape \\{}", other as char))
                            )
                        }
                    }
                }
                // A raw control character is what an unescaped newline in a district name
                // would look like, and it is not legal JSON.
                Some(c) if c < 0x20 => {
                    return Err(self.fail(&format!("a raw control byte {c:#04x} in a string")))
                }
                Some(_) => {
                    let start = self.at;
                    while let Some(c) = self.peek() {
                        if c == b'"' || c == b'\\' || c < 0x20 {
                            break;
                        }
                        self.at += 1;
                    }
                    out.push_str(&String::from_utf8_lossy(&self.bytes[start..self.at]));
                }
            }
        }
    }

    fn number(&mut self) -> Result<(), String> {
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        while matches!(
            self.peek(),
            Some(b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
        ) {
            self.at += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.at])
            .map_err(|_| self.fail("a number that is not UTF-8"))?;
        // Parsed rather than merely scanned: `1.2.3` and `-` both pass a character filter and
        // are not numbers, and a consumer would fail on them where this would not.
        text.parse::<f64>()
            .map(|_| ())
            .map_err(|_| self.fail(&format!("{text:?} is not a number")))
    }

    fn word(&mut self, word: &str) -> Result<(), String> {
        if self.bytes[self.at..].starts_with(word.as_bytes()) {
            self.at += word.len();
            Ok(())
        } else {
            Err(self.fail(&format!("expected {word}")))
        }
    }
}

/// Parse `text`, returning every duplicated key path and how deep the document nests.
fn parse(text: &str) -> Result<(Vec<Path>, usize), String> {
    let mut reader = Reader::new(text);
    reader.value("")?;
    reader.space();
    if reader.at != reader.bytes.len() {
        return Err(reader.fail("trailing bytes after the document"));
    }
    Ok((reader.duplicates, reader.max_depth))
}

#[test]
fn the_published_feed_is_well_formed_json() {
    let feed = bundle::build::build().to_json();
    let (_, depth) = parse(&feed).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        depth >= 4,
        "the feed nests districts inside blocks inside the root; a depth of {depth} means the \
         parser stopped early rather than that the document got flatter"
    );
}

#[test]
fn no_object_in_the_published_feed_repeats_a_key() {
    let feed = bundle::build::build().to_json();
    let (duplicates, _) = parse(&feed).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        duplicates.is_empty(),
        "{} object(s) repeat a key. A repeated key is not a parse error — every consumer takes \
         the last and drops the first silently, which is how `special_education` and \
         `categoricals` shipped twice per district for several contract versions:\n  {}",
        duplicates.len(),
        duplicates
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// The two blocks that had no execution coverage at all before this file existed.
///
/// `Bundle::national` is 58 lines emitting nine scalars and a nested `states` array;
/// `Bundle::deflator` is 19 more. Every serialization test builds a hand-written fixture with
/// both set to `None`, so neither emitter had ever run under a test — the golden-file diff was
/// the only thing standing between them and a first-commit bug.
///
/// Parsing the real document runs them. This asserts they are *there*, so that coverage cannot
/// be lost by quietly making either one `None`: the parser would still pass, having simply been
/// given less to read.
#[test]
fn the_blocks_that_only_the_real_build_populates_are_in_it() {
    let built = bundle::build::build();
    let national = built.national.as_ref().expect("the national comparison");
    assert!(
        !national.states.is_empty(),
        "the states array is what makes the national block more than nine scalars"
    );
    let deflator = built.deflator.as_ref().expect("the deflator");
    assert!(!deflator.points.is_empty(), "a deflator with no points");

    // The years the feed carries and the index cannot reach, which used to leave through a
    // `filter_map` and be visible to nothing. CPI-U June runs FY2000-FY2026 and the
    // appropriations series runs FY1998-FY2027, so it is the two ends of that series.
    assert_eq!(
        deflator.uncovered,
        vec![1998, 1999, 2027],
        "the deflator's gap against the feed changed"
    );
    assert!(
        deflator
            .uncovered
            .iter()
            .all(|year| !deflator.points.iter().any(|(covered, _)| covered == year)),
        "a year is named as uncovered and carries a point"
    );

    // And that both survive into the document, rather than being built and dropped.
    let feed = built.to_json();
    assert!(parse(&feed).is_ok());
    assert!(
        feed.contains("\"deflator\": {"),
        "the deflator block is not emitted"
    );
    assert!(
        feed.contains("\"uncovered\": [1998, 1999, 2027]"),
        "the deflator's uncovered years are not emitted"
    );
}

#[test]
fn the_parser_refuses_the_documents_it_is_meant_to_refuse() {
    // Asserted because a validator that accepts everything passes every test it is given, and
    // the two above would then be worth nothing.
    for (bad, why) in [
        ("{\"a\": 1,}", "a trailing comma"),
        ("{\"a\" 1}", "a missing colon"),
        ("{\"a\": }", "a missing value"),
        ("[1, 2", "an unclosed array"),
        ("{\"a\": 1} junk", "trailing bytes"),
        ("{\"a\": 1.2.3}", "a number that is not one"),
        ("{\"a\": -}", "a bare minus"),
        ("{\"a\": nul}", "a truncated literal"),
        ("{\"a\": \"x", "an unterminated string"),
        ("{\"a\": \"\\q\"}", "an undefined escape"),
    ] {
        assert!(parse(bad).is_err(), "accepted {why}: {bad}");
    }

    // And that it accepts the shapes the feed actually contains.
    for good in [
        "{}",
        "[]",
        "{\"a\": [], \"b\": {}}",
        "{\"a\": -3.25, \"b\": 1e-9, \"c\": null, \"d\": true}",
        "{\"a\": \"quote \\\" backslash \\\\ newline \\n tab \\t unicode \\u00e9\"}",
    ] {
        assert!(parse(good).is_ok(), "refused {good}");
    }
}

#[test]
fn a_repeated_key_is_found_wherever_it_is_nested() {
    // The bug this exists for was one level down, inside a district — not at the root.
    let (duplicates, _) = parse("{\"districts\": [{\"a\": 1, \"b\": {\"c\": 1, \"c\": 2}}]}")
        .expect("a repeated key is legal JSON, which is the whole problem");
    assert_eq!(duplicates, vec![".districts[0].b.c".to_string()]);
}
