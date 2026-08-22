//! How this repository writes a number down.
//!
//! One rule, shared by the two places that emit a figure a human or a parser will read back:
//! the CSV fixtures `connect` builds, and the JSON feed `bundle` serializes. It was written
//! twice, and the second copy is what made the rule worth naming rather than restating.

/// A number at `places` decimals, with trailing zeros trimmed.
///
/// # A trailing zero is not a trailing zero
///
/// The trim only applies past a decimal point, and the guard is the whole point of the
/// function. Trimming the string form of an integer turns 10 into 1 and 30 into 3 — which is
/// what an earlier version of this did to the school building count, silently, for every
/// district that happened to have a multiple of ten. Callers passing `places` of four or more
/// never reach that branch, and one caller passes zero.
///
/// # Why trim at all
///
/// A fixture and a feed are both read in diffs. `20` and `20.0000` are the same number and only
/// one of them is legible, and a figure that gains or loses trailing zeros between two runs would
/// show up as a change when nothing changed.
#[must_use]
pub fn trimmed(value: f64, places: usize) -> String {
    let rendered = format!("{value:.places$}");
    if rendered.contains('.') {
        // Trim in place. Chaining `trim_end_matches` into `to_string` allocates a second time
        // for every number, and the feed alone carries roughly 73,000 of them.
        let keep = rendered.trim_end_matches('0').trim_end_matches('.').len();
        let mut out = rendered;
        out.truncate(keep);
        out
    } else {
        rendered
    }
}

/// A string with everything JSON forbids inside one escaped.
///
/// The five named escapes plus `\u00XX` for every other control character. Shared because it was
/// not: `bundle` had this, and `project`'s draft output hand-rolled a shorter version covering
/// four of the seven cases — which was itself an improvement on emitting the string raw, and is
/// the kind of improvement that ends in three subtly different escapers.
#[must_use]
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The guard that gives the module its docstring.
    #[test]
    fn an_integer_keeps_its_trailing_zeros() {
        assert_eq!(trimmed(10.0, 0), "10");
        assert_eq!(trimmed(30.0, 0), "30");
        assert_eq!(trimmed(100.0, 0), "100");
    }

    #[test]
    fn a_fraction_loses_its_trailing_zeros_and_its_point() {
        assert_eq!(trimmed(847.823, 4), "847.823");
        assert_eq!(trimmed(20.0, 4), "20");
        assert_eq!(trimmed(0.5334, 8), "0.5334");
    }

    /// Rounds rather than truncates, which is what makes two runs of the same figure agree.
    #[test]
    fn the_last_place_is_rounded() {
        assert_eq!(trimmed(1.000_05, 4), "1.0001");
        assert_eq!(trimmed(1.000_04, 4), "1");
    }

    #[test]
    fn every_character_json_forbids_is_escaped() {
        assert_eq!(
            escape("Big \"Walnut\" \\ Local\nDistrict\t2\r\u{1}"),
            "Big \\\"Walnut\\\" \\\\ Local\\nDistrict\\t2\\r\\u0001"
        );
    }
}
