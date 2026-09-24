//! A JSON reader, because one publisher here answers in JSON and nothing else did.
//!
//! Every other source this crate retrieves is a spreadsheet, a PDF or a delimited file. The
//! report card's data API is the first that answers in JSON, and this workspace carries no
//! crates.io dependencies — so the choice was a parser or a set of regular expressions over a
//! 50 KB document whose shape is nested. `spreadsheet` already hand-writes DEFLATE, OLE2 and
//! XML on the same reasoning; JSON is the smallest of the four by a wide margin.
//!
//! # What it is not
//!
//! Not a serialiser, and not a general document model. There is no `to_string`, no numeric tower
//! and no borrowing: [`Value`] owns its strings, numbers are `f64`, and the accessors return
//! `Option` rather than a `Result` describing the path that failed. A caller wanting a better
//! error than "absent" should say what was absent itself, which is what
//! [`super::fixtures::ctpd`] does.
//!
//! # What it is strict about
//!
//! Trailing commas, unquoted keys, single quotes, `NaN`, and leading `+` are all refused — the
//! extensions a forgiving parser accepts are the ones that make a corrupted document look like a
//! valid one. Duplicate keys keep the **last** occurrence, matching every mainstream
//! implementation. Unpaired surrogates are refused rather than replaced, because a replacement
//! character in a district name would be a silent corruption of a join key.
//!
//! Depth is capped at [`MAX_DEPTH`]. A JSON parser that recurses on input is a stack overflow
//! waiting for a hostile or broken document, and an overflow is not a catchable error in Rust.

use std::collections::BTreeMap;

/// How deeply values may nest before the parser refuses the document.
///
/// The report card's deepest response nests six levels. This is two orders of magnitude above
/// that, and it exists to make a runaway document an error rather than an abort.
pub const MAX_DEPTH: usize = 256;

/// A parsed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// Any number. JSON has one numeric type and so does this.
    Number(f64),
    /// A string, with escapes already resolved.
    Text(String),
    /// An array.
    Array(Vec<Value>),
    /// An object. Ordered by key rather than by appearance, which this crate never depends on.
    Object(BTreeMap<String, Value>),
}

impl Value {
    /// The value at `key`, if this is an object that has one.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(map) => map.get(key),
            _ => None,
        }
    }

    /// This value as a string slice, if it is one.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }

    /// This value as a number, if it is one.
    #[must_use]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// This value's elements, if it is an array.
    #[must_use]
    pub fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    /// The string at `key`, trimmed, if there is one and it is not empty.
    ///
    /// The trimming is deliberate: this publisher ships names with trailing spaces — the CTPD
    /// index carries `Ashtabula County Technical and Career Center  CTPD` with two — and an
    /// untrimmed name is a join key that fails for a reason nobody can see.
    #[must_use]
    pub fn text(&self, key: &str) -> Option<&str> {
        let raw = self.get(key)?.as_str()?.trim();
        (!raw.is_empty()).then_some(raw)
    }

    /// The number at `key`, if there is one.
    #[must_use]
    pub fn number(&self, key: &str) -> Option<f64> {
        self.get(key)?.as_f64()
    }
}

/// Parse a complete JSON document.
///
/// # Errors
///
/// Returns a message naming the byte offset and what was expected there. Trailing content after
/// the top-level value is an error, not a stopping point: a truncated download that happens to
/// end mid-array would otherwise parse as a shorter document.
pub fn parse(source: &str) -> Result<Value, String> {
    let bytes = source.as_bytes();
    let mut at = 0usize;
    skip_space(bytes, &mut at);
    let value = parse_value(bytes, &mut at, 0)?;
    skip_space(bytes, &mut at);
    if at != bytes.len() {
        return Err(format!("trailing content at byte {at}"));
    }
    Ok(value)
}

fn skip_space(b: &[u8], at: &mut usize) {
    while *at < b.len() && matches!(b[*at], b' ' | b'\t' | b'\n' | b'\r') {
        *at += 1;
    }
}

fn parse_value(b: &[u8], at: &mut usize, depth: usize) -> Result<Value, String> {
    if depth > MAX_DEPTH {
        return Err(format!("nesting deeper than {MAX_DEPTH} at byte {at}"));
    }
    match b.get(*at) {
        None => Err("document ended where a value was expected".to_string()),
        Some(b'{') => parse_object(b, at, depth),
        Some(b'[') => parse_array(b, at, depth),
        Some(b'"') => Ok(Value::Text(parse_string(b, at)?)),
        Some(b't') => literal(b, at, b"true", Value::Bool(true)),
        Some(b'f') => literal(b, at, b"false", Value::Bool(false)),
        Some(b'n') => literal(b, at, b"null", Value::Null),
        Some(_) => parse_number(b, at),
    }
}

fn literal(b: &[u8], at: &mut usize, want: &[u8], value: Value) -> Result<Value, String> {
    if b[*at..].starts_with(want) {
        *at += want.len();
        Ok(value)
    } else {
        Err(format!(
            "expected {} at byte {at}",
            String::from_utf8_lossy(want)
        ))
    }
}

fn parse_object(b: &[u8], at: &mut usize, depth: usize) -> Result<Value, String> {
    *at += 1; // the '{'
    let mut map = BTreeMap::new();
    skip_space(b, at);
    if b.get(*at) == Some(&b'}') {
        *at += 1;
        return Ok(Value::Object(map));
    }
    loop {
        skip_space(b, at);
        if b.get(*at) != Some(&b'"') {
            return Err(format!("expected a quoted key at byte {at}"));
        }
        let key = parse_string(b, at)?;
        skip_space(b, at);
        if b.get(*at) != Some(&b':') {
            return Err(format!("expected ':' after a key at byte {at}"));
        }
        *at += 1;
        skip_space(b, at);
        // Last occurrence wins, as every mainstream implementation does.
        map.insert(key, parse_value(b, at, depth + 1)?);
        skip_space(b, at);
        match b.get(*at) {
            Some(b',') => *at += 1,
            Some(b'}') => {
                *at += 1;
                return Ok(Value::Object(map));
            }
            _ => return Err(format!("expected ',' or '}}' at byte {at}")),
        }
    }
}

fn parse_array(b: &[u8], at: &mut usize, depth: usize) -> Result<Value, String> {
    *at += 1; // the '['
    let mut items = Vec::new();
    skip_space(b, at);
    if b.get(*at) == Some(&b']') {
        *at += 1;
        return Ok(Value::Array(items));
    }
    loop {
        skip_space(b, at);
        items.push(parse_value(b, at, depth + 1)?);
        skip_space(b, at);
        match b.get(*at) {
            Some(b',') => *at += 1,
            Some(b']') => {
                *at += 1;
                return Ok(Value::Array(items));
            }
            _ => return Err(format!("expected ',' or ']' at byte {at}")),
        }
    }
}

fn parse_string(b: &[u8], at: &mut usize) -> Result<String, String> {
    *at += 1; // the opening quote
    let mut out = String::new();
    loop {
        match b.get(*at) {
            None => return Err("document ended inside a string".to_string()),
            Some(b'"') => {
                *at += 1;
                return Ok(out);
            }
            Some(b'\\') => {
                *at += 1;
                let escape = *b.get(*at).ok_or("document ended inside an escape")?;
                *at += 1;
                match escape {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{8}'),
                    b'f' => out.push('\u{c}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => out.push(parse_escape(b, at)?),
                    other => {
                        return Err(format!(
                            "unknown escape \\{} at byte {at}",
                            char::from(other)
                        ))
                    }
                }
            }
            Some(&byte) if byte < 0x20 => {
                return Err(format!(
                    "raw control byte {byte:#04x} inside a string at {at}"
                ))
            }
            Some(_) => {
                // Copy one whole UTF-8 sequence. The input is `&str`, so it is valid by
                // construction and the only question is where this character ends.
                let start = *at;
                *at += 1;
                while *at < b.len() && (b[*at] & 0b1100_0000) == 0b1000_0000 {
                    *at += 1;
                }
                out.push_str(
                    core::str::from_utf8(&b[start..*at])
                        .map_err(|_| "invalid UTF-8".to_string())?,
                );
            }
        }
    }
}

/// Read a `\uXXXX` escape, pairing surrogates.
///
/// An unpaired surrogate is refused rather than replaced. This crate's JSON carries district and
/// school names that become join keys, and a `U+FFFD` in one of those is a lookup that fails
/// with no visible cause.
fn parse_escape(b: &[u8], at: &mut usize) -> Result<char, String> {
    let first = hex4(b, at)?;
    if !(0xD800..0xDC00).contains(&first) {
        return char::from_u32(first).ok_or_else(|| format!("\\u{first:04X} is not a character"));
    }
    if b.get(*at) != Some(&b'\\') || b.get(*at + 1) != Some(&b'u') {
        return Err(format!(
            "\\u{first:04X} is a lone leading surrogate at {at}"
        ));
    }
    *at += 2;
    let second = hex4(b, at)?;
    if !(0xDC00..0xE000).contains(&second) {
        return Err(format!(
            "\\u{second:04X} is not a trailing surrogate at {at}"
        ));
    }
    let combined = 0x1_0000 + ((first - 0xD800) << 10) + (second - 0xDC00);
    char::from_u32(combined).ok_or_else(|| format!("\\u{combined:X} is not a character"))
}

fn hex4(b: &[u8], at: &mut usize) -> Result<u32, String> {
    let slice = b
        .get(*at..*at + 4)
        .ok_or_else(|| format!("a \\u escape ran off the end at byte {at}"))?;
    let text = core::str::from_utf8(slice).map_err(|_| "invalid UTF-8 in an escape".to_string())?;
    let value =
        u32::from_str_radix(text, 16).map_err(|_| format!("`{text}` is not four hex digits"))?;
    *at += 4;
    Ok(value)
}

fn parse_number(b: &[u8], at: &mut usize) -> Result<Value, String> {
    let start = *at;
    if b.get(*at) == Some(&b'-') {
        *at += 1;
    }
    let digits_from = *at;
    while matches!(b.get(*at), Some(d) if d.is_ascii_digit()) {
        *at += 1;
    }
    if *at == digits_from {
        return Err(format!("expected a number at byte {start}"));
    }
    // A leading zero may not be followed by another digit: `01` is not JSON.
    if b[digits_from] == b'0' && *at - digits_from > 1 {
        return Err(format!("number with a leading zero at byte {start}"));
    }
    if b.get(*at) == Some(&b'.') {
        *at += 1;
        let from = *at;
        while matches!(b.get(*at), Some(d) if d.is_ascii_digit()) {
            *at += 1;
        }
        if *at == from {
            return Err(format!("expected digits after '.' at byte {at}"));
        }
    }
    if matches!(b.get(*at), Some(b'e' | b'E')) {
        *at += 1;
        if matches!(b.get(*at), Some(b'+' | b'-')) {
            *at += 1;
        }
        let from = *at;
        while matches!(b.get(*at), Some(d) if d.is_ascii_digit()) {
            *at += 1;
        }
        if *at == from {
            return Err(format!("expected digits in an exponent at byte {at}"));
        }
    }
    let text = core::str::from_utf8(&b[start..*at]).map_err(|_| "invalid UTF-8".to_string())?;
    text.parse::<f64>()
        .map(Value::Number)
        .map_err(|_| format!("`{text}` is not a number"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_shape_this_crate_actually_receives() {
        let doc = r#"[{"irn":"200097","detail":{"enrollmentDist":[
            {"distName":"Kent City","enrlCtpd":687},
            {"distName":"Schnee Learning Center","enrlCtpd":0}]}}]"#;
        let v = parse(doc).expect("parses");
        let first = &v.as_array().expect("an array")[0];
        assert_eq!(first.text("irn"), Some("200097"));
        let roster = first
            .get("detail")
            .and_then(|d| d.get("enrollmentDist"))
            .and_then(Value::as_array)
            .expect("a roster");
        assert_eq!(roster.len(), 2);
        assert_eq!(roster[0].text("distName"), Some("Kent City"));
        assert_eq!(roster[1].number("enrlCtpd"), Some(0.0));
    }

    /// The publisher ships names with trailing spaces; an untrimmed key fails invisibly.
    #[test]
    fn text_trims_and_treats_blank_as_absent() {
        let v = parse(r#"{"a":"  Ashtabula  ","b":"   ","c":null}"#).expect("parses");
        assert_eq!(v.text("a"), Some("Ashtabula"));
        assert_eq!(v.text("b"), None);
        assert_eq!(v.text("c"), None);
        assert_eq!(v.text("missing"), None);
    }

    /// Truncation must not look like a shorter document.
    #[test]
    fn a_truncated_document_is_an_error_rather_than_a_prefix() {
        assert!(parse(r#"[{"a":1},{"b":"#).is_err());
        assert!(parse(r#"{"a":1"#).is_err());
        assert!(parse(r#""unterminated"#).is_err());
    }

    /// The forgiving extensions are what make a corrupt document look valid.
    #[test]
    fn the_usual_extensions_are_refused() {
        for bad in [
            "{\"a\":1,}",
            "[1,2,]",
            "{a:1}",
            "{'a':1}",
            "[01]",
            "[+1]",
            "[NaN]",
            "[1.]",
            "[1e]",
        ] {
            assert!(parse(bad).is_err(), "{bad} should not parse");
        }
    }

    /// Two values after one another is not one document.
    #[test]
    fn trailing_content_is_refused() {
        assert!(parse("{} {}").is_err());
        assert!(parse("1 2").is_err());
        assert_eq!(parse("  1  "), Ok(Value::Number(1.0)));
    }

    #[test]
    fn escapes_including_surrogate_pairs() {
        assert_eq!(
            parse(r#""aB\n\"\\é""#),
            Ok(Value::Text("aB\n\"\\é".to_string()))
        );
        assert_eq!(parse(r#""😀""#), Ok(Value::Text("😀".to_string())));
    }

    /// A replacement character in a name is a join key that fails with no visible cause.
    #[test]
    fn a_lone_surrogate_is_refused_rather_than_replaced() {
        assert!(parse(r#""\ud83d""#).is_err());
        assert!(parse(r#""\ude00""#).is_err());
        assert!(parse(r#""\ud83dx""#).is_err());
    }

    /// Recursion on input needs a bound, because an overflow cannot be caught.
    #[test]
    fn nesting_is_bounded() {
        let deep = "[".repeat(MAX_DEPTH + 2) + &"]".repeat(MAX_DEPTH + 2);
        assert!(parse(&deep).is_err_and(|why| why.contains("nesting")));
        let fine = "[".repeat(8) + &"]".repeat(8);
        assert!(parse(&fine).is_ok());
    }

    #[test]
    fn duplicate_keys_keep_the_last() {
        assert_eq!(parse(r#"{"a":1,"a":2}"#).unwrap().number("a"), Some(2.0));
    }

    #[test]
    fn empty_containers_and_control_bytes() {
        assert_eq!(parse("{}"), Ok(Value::Object(BTreeMap::new())));
        assert_eq!(parse("[]"), Ok(Value::Array(vec![])));
        assert!(parse("\"a\nb\"").is_err(), "a raw newline is not legal");
    }
}
