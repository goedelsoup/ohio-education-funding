//! The two readers of raw document text that more than one extractor needs.
//!
//! `flatten` used to exist twice under two names — `collapse` in the transfer reader and
//! `flatten` in the scholarship reader — with identical three-line bodies. Splitting the file
//! made that visible; one name is what survived.

/// The characters of a fixed-width row from `start` up to `end`, by position and not by byte.
///
/// The reports are read Latin-1 byte for byte and re-encoded, so a row carrying a high byte is
/// longer in bytes than in the columns the printer laid out. One line of the FY2013 community
/// report is, and a byte-wise slice would take its site IRN from the wrong place.
pub(super) fn fixed(line: &[char], start: usize, end: usize) -> String {
    line.get(start..end.min(line.len()))
        .unwrap_or_default()
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
}

/// Collapse runs of whitespace, so a figure split across a line break still reads as one token.
pub(super) fn flatten(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The number immediately before `marker`, written with thousands separators.
///
/// The reports state a count before the thing counted — "8,186 students received services from"
/// — so the figure is found by locating the phrase and reading backwards from it. Returns `None`
/// when what precedes the marker is not a number, which is how "Ohio's school districts" is told
/// apart from "487 school districts" without a second pattern.
pub(super) fn count_before(text: &str, marker: usize) -> Option<f64> {
    let head = &text[..marker];
    let digits: String = head
        .chars()
        .rev()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .filter(|c| *c != ',')
        .collect();
    if digits.is_empty() {
        return None;
    }
    digits.chars().rev().collect::<String>().parse().ok()
}
