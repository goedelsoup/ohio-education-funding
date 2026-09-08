//! The department's own count of every way an Ohio child is schooled outside a traditional
//! district, from the annual `Ohio's Education Landscape` fact sheet.
//!
//! One table, headed **School Options**, printed once a year on a two-page sheet. It is the only
//! place the department publishes a home-education count in machine-reachable form, and it prints
//! it beside the vouchers, the community schools, the private schools and the open-enrolment
//! transfers, so the channels can be read against each other rather than one at a time.
//!
//! # Why this is parsed by column and not by label
//!
//! The sheet is laid out in two columns and `pdftotext -layout` interleaves them, so a line
//! carries the left column's text and the right column's on the same row. **`Community Schools`
//! appears on both sides of the same line with different meanings** — 342 as a count of schools
//! on the left, 116,973 as an enrolment on the right — and a reader that searched by label would
//! take whichever it met first. The `School Options` heading fixes the right column's start, and
//! everything here is read from that offset rightward.
//!
//! Two rows also wrap: `Online Community Schools` puts its label, its value and its
//! `(e-schools)` qualifier on three consecutive lines, with unrelated left-column text between.
//! A label with no number carries forward to the next number in the same column, and a
//! parenthesised fragment attaches to the row above it.

use std::collections::BTreeMap;

/// Columns of the fixture.
pub const LANDSCAPE_HEADER: &[&str] = &["channel", "enrollment", "parent"];

/// The heading that fixes the right-hand column.
const HEADING: &str = "School Options";

/// The statewide public enrolment printed elsewhere on the same sheet, and the denominator every
/// share here is taken against.
///
/// It is emitted as the fixture's last row despite belonging to a different table, because the
/// alternative is a figure the corpus quotes and nothing recomputes — which is the transcription
/// this repository keeps finding wrong. It is **not** a channel and not a parent of any: home
/// education and the chartered private schools are outside this count, and only the community
/// schools are inside it.
const TOTAL_LABEL: &str = "Total Enrollment";

/// The channel every row must be one of, and what it is a part of.
///
/// Pinned rather than discovered. A programme that stops being reported should fail this build
/// rather than quietly leave the fixture a row shorter — the same rule the scholarship report's
/// reader follows, and for the same reason: an absent row and a renamed one look identical
/// downstream.
const CHANNELS: &[(&str, &str)] = &[
    ("Community Schools", ""),
    ("Inter-District Open Enrollment", ""),
    ("Public Vouchers for Private School", ""),
    ("EdChoice Scholarship", "Public Vouchers for Private School"),
    ("EdChoice Expansion", "Public Vouchers for Private School"),
    (
        "Cleveland Scholarship",
        "Public Vouchers for Private School",
    ),
    (
        "Jon Peterson Special Needs",
        "Public Vouchers for Private School",
    ),
    ("Autism Scholarship", "Public Vouchers for Private School"),
    ("Home School", ""),
    ("Chartered Private Schools", ""),
    ("Joint Vocational School Districts", ""),
    ("Brick-and-Mortar Community Schools", "Community Schools"),
    ("Online Community Schools", "Community Schools"),
];

/// A label with its parenthesised qualifier and bullet removed.
///
/// `Chartered Private Schools (711)` carries the number of such schools inside the label, and
/// `Online Community Schools (e-schools)` a gloss. Neither is the enrolment, and a fixture keyed
/// on a label that embeds a count would change key the year the count changed.
fn canonical(label: &str) -> String {
    let without_bullet = label.trim_start_matches(['•', '\u{2022}', ' ']).trim();
    match without_bullet.find('(') {
        Some(at) => without_bullet[..at].trim().to_string(),
        None => without_bullet.to_string(),
    }
}

/// A trailing integer with thousands separators, if the text ends in one.
fn trailing_number(text: &str) -> Option<u64> {
    let tail = text.rsplit(char::is_whitespace).next()?;
    if tail.is_empty() || !tail.chars().all(|c| c.is_ascii_digit() || c == ',') {
        return None;
    }
    tail.replace(',', "").parse().ok()
}

/// The statewide public enrolment the sheet prints as its own total.
///
/// # Errors
///
/// Returns a message if the line is absent or carries no number.
pub fn total_enrollment(text: &str) -> Result<u64, String> {
    text.lines()
        .find(|line| line.trim_start().starts_with(TOTAL_LABEL))
        .and_then(|line| {
            // The row is `Total Enrollment  1,665,521  100%`; the percentage is last.
            line.split_whitespace().filter_map(trailing_number).max()
        })
        .ok_or_else(|| format!("the sheet carries no `{TOTAL_LABEL}` row"))
}

/// The School Options table, one row per channel, oldest publication conventions notwithstanding.
///
/// # Errors
///
/// Returns a message if the heading is absent, or if any pinned channel is missing from the
/// table — either is the sheet having been redesigned, which must fail loudly rather than
/// produce a shorter fixture.
pub fn build_landscape_channels(text: &str) -> Result<Vec<Vec<String>>, String> {
    let lines: Vec<&str> = text.lines().collect();
    let heading = lines
        .iter()
        .position(|line| line.contains(HEADING))
        .ok_or_else(|| format!("the sheet carries no `{HEADING}` heading"))?;
    // A **character** offset, not the byte one `str::find` returns. The two agree only while
    // everything left of the heading is ASCII, and this sheet's bullets are not — they sit just
    // right of the boundary today, and a layout that moved one left of it would slice the wrong
    // column and read plausible wrong numbers.
    let column = lines[heading]
        .find(HEADING)
        .map(|byte| lines[heading][..byte].chars().count())
        .ok_or("the heading moved between two reads of the same line")?;

    let mut found: BTreeMap<String, u64> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut pending: Option<String> = None;

    for line in lines.iter().skip(heading + 1) {
        let right = match line.char_indices().nth(column) {
            Some((at, _)) => line[at..].trim_end(),
            None => continue,
        };
        let cell = right.trim();
        if cell.is_empty() {
            continue;
        }
        // A parenthesised fragment on its own is a qualifier for the row above, and carries no
        // number of its own.
        if cell.starts_with('(') {
            continue;
        }
        match trailing_number(cell) {
            // `<label>  <number>` — the ordinary row.
            Some(value) if cell.split_whitespace().count() > 1 => {
                let label = canonical(cell.rsplit_once(char::is_whitespace).map_or("", |x| x.0));
                if !label.is_empty() {
                    order.push(label.clone());
                    found.insert(label, value);
                    pending = None;
                }
            }
            // A bare number belongs to the label that could not fit beside it.
            Some(value) => {
                if let Some(label) = pending.take() {
                    order.push(label.clone());
                    found.insert(label, value);
                }
            }
            // A label with no number yet.
            None => pending = Some(canonical(cell)),
        }
        if order.len() == CHANNELS.len() {
            break;
        }
    }

    let mut rows = Vec::with_capacity(CHANNELS.len() + 1);
    for (channel, parent) in CHANNELS {
        let value = found.get(*channel).ok_or_else(|| {
            format!("the School Options table no longer carries `{channel}`; it read {order:?}")
        })?;
        rows.push(vec![
            (*channel).to_string(),
            value.to_string(),
            (*parent).to_string(),
        ]);
    }
    rows.push(vec![
        TOTAL_LABEL.to_string(),
        total_enrollment(text)?.to_string(),
        String::new(),
    ]);
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two-column interleave, reduced to the shape that broke a label-first reader.
    const SHEET: &str = "\
Types of Schools          Schools   Percent          School Options                     Enrollment
All Public Schools          3,550      100%          Community Schools                     116,973
Traditional Public           3,117     87.8%
Community Schools              342      9.6%         Public Vouchers for Private School     150,914
Vocational Schools              76      2.1%                • EdChoice Scholarship            41,234
                                                            • EdChoice Expansion              88,095
                                                            • Cleveland Scholarship            8,028
                                                            • Jon Peterson Special Needs       8,551
                                                            • Autism Scholarship               5,006
                                                     Inter-District Open Enrollment          79,006
                                                     Home School                             53,051
                                                     Chartered Private Schools (711)        173,156
                                                     Joint Vocational School Districts       49,524
                                                     Brick-and-Mortar Community Schools      83,959
                                                     Online Community Schools
All Educators               22,792      100%                                                 33,014
                                                     (e-schools)
Total Enrollment         1,665,521       100%
";

    #[test]
    fn the_right_column_is_read_and_not_the_left_one_with_the_same_name() {
        let rows = build_landscape_channels(SHEET).expect("the sheet is well formed");
        let by_name: BTreeMap<&str, &str> = rows
            .iter()
            .map(|r| (r[0].as_str(), r[1].as_str()))
            .collect();
        // 342 is the left column's count of community schools on the same line.
        assert_eq!(by_name["Community Schools"], "116973");
        assert_eq!(by_name["Home School"], "53051");
    }

    #[test]
    fn a_row_whose_value_wrapped_onto_the_next_line_is_still_read() {
        let rows = build_landscape_channels(SHEET).expect("the sheet is well formed");
        let online = rows
            .iter()
            .find(|r| r[0] == "Online Community Schools")
            .expect("the wrapped row is present");
        assert_eq!(online[1], "33014");
        assert_eq!(online[2], "Community Schools");
    }

    #[test]
    fn the_label_keeps_no_count_that_could_change_under_it() {
        assert_eq!(
            canonical("Chartered Private Schools (711)"),
            "Chartered Private Schools"
        );
        assert_eq!(canonical("   • EdChoice Expansion"), "EdChoice Expansion");
    }

    #[test]
    fn a_channel_that_stops_being_reported_fails_rather_than_shortening_the_fixture() {
        let without = SHEET.replace("Home School                             53,051", "");
        let error = build_landscape_channels(&without).expect_err("a channel is missing");
        assert!(error.contains("Home School"), "{error}");
    }

    #[test]
    fn the_statewide_total_is_the_enrollment_and_not_the_percentage() {
        assert_eq!(total_enrollment(SHEET), Ok(1_665_521));
    }

    /// The denominator is built rather than quoted, and is marked as belonging to no channel.
    #[test]
    fn the_denominator_is_the_last_row_and_is_not_a_channel() {
        let rows = build_landscape_channels(SHEET).expect("the sheet is well formed");
        assert_eq!(rows.len(), CHANNELS.len() + 1);
        let last = rows.last().expect("the rows are not empty");
        assert_eq!(last[0], "Total Enrollment");
        assert_eq!(last[1], "1665521");
        assert_eq!(last[2], "", "the total is a parent of nothing");
        assert!(
            !CHANNELS.iter().any(|(name, _)| *name == last[0]),
            "the total must not also be pinned as a channel"
        );
    }

    /// A sheet that stops printing its own total fails rather than losing the denominator.
    #[test]
    fn a_sheet_without_a_total_fails_rather_than_dropping_the_denominator() {
        let without = SHEET.replace("Total Enrollment         1,665,521       100%", "");
        let error = build_landscape_channels(&without).expect_err("the total is missing");
        assert!(error.contains("Total Enrollment"), "{error}");
    }
}
