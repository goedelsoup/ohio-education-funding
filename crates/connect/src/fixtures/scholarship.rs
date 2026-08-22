//! The scholarship programmes' annual report, as counts and dollars per programme.
//!
//! The report is prose with figures in it rather than a table, so each programme is found by name
//! and its two numbers taken from the text around it. The programme list is pinned: a programme
//! that stops being reported should fail here rather than silently leave the fixture.

use super::text::flatten;

/// The five programmes, in the order the report presents them.
///
/// Matched on a phrase the report uses in running prose rather than on a heading, because the
/// headings appear twice — once in the table of contents with dot leaders, once in the body — and
/// the table of contents carries no figures. The prose sentence appears only in the body.
const PROGRAMMES: &[(&str, &str)] = &[
    (
        "traditional-edchoice",
        "Traditional EdChoice Scholarship Program",
    ),
    (
        "edchoice-expansion",
        "EdChoice Expansion Scholarship Program",
    ),
    ("cleveland", "Cleveland Scholarship Program"),
    ("autism", "Autism Scholarship Program"),
    (
        "jon-peterson",
        "Jon Peterson Special Needs Scholarship Program",
    ),
];

/// The number immediately before `marker`, written with thousands separators.
fn count_before(text: &str, marker: usize) -> Option<f64> {
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

/// The dollar amount following the first occurrence of `marker` in `text`.
fn dollars_after(text: &str, marker: &str) -> Option<f64> {
    let at = text.find(marker)? + marker.len();
    let rest = &text[at..];
    let sign = rest.find('$')?;
    let after = &rest[sign + 1..];
    let end = after
        .find(|c: char| !c.is_ascii_digit() && c != ',' && c != '.')
        .unwrap_or(after.len());
    // `end` stops at the first character that is neither digit, comma nor point — which for
    // "$283,149,735.14." is the space *after* the sentence stop, so the stop comes along. Parsing
    // "283149735.14." fails, and it fails silently into an empty column rather than loudly.
    after[..end]
        .trim_end_matches('.')
        .chars()
        .filter(|c| *c != ',')
        .collect::<String>()
        .parse()
        .ok()
}

/// One row per scholarship programme: participation, expenditure, and the report's own average.
///
/// # How a programme is located
///
/// By its participation sentence — "N students utilized the `<programme>`", or for Jon Peterson
/// "N students received services from" — and not by its heading. The headings appear twice, once
/// in the table of contents with dot leaders and once in the body, and the table of contents
/// carries no figures. The sentence appears only in the body.
///
/// Each programme's section then runs to the next programme's participation sentence, and the two
/// dollar figures are read inside that window. Anchoring on order alone would work on this
/// edition and break silently on an edition that reordered the programmes.
///
/// # Why the average is carried rather than computed
///
/// Because the report's does not equal expenditure over participation, and the gap is the finding.
/// Autism reconciles to the cent; the other three published averages sit 1.5-3.4% above the
/// implied figure. Recomputing the column would erase that, and declaring the report wrong would
/// claim more than is known — the likeliest reading is that the two divide by different student
/// counts, and the report does not say. Both numbers are committed and the disagreement is pinned
/// in `crates/project/tests/` rather than resolved.
///
/// Jon Peterson publishes no expenditure total and no average: its spending appears only as six
/// disability-category figures in a chart. Those two columns are left empty, which states the
/// absence rather than filling it.
///
/// # Errors
///
/// If fewer than five programmes carry a participation sentence, which means the report was
/// restructured and every figure downstream of it should be treated as unread.
pub fn scholarship_programmes(text: &str) -> Result<Vec<Vec<String>>, String> {
    const MARKERS: [&str; 2] = ["students utilized the ", "students received services from "];
    let flat = flatten(text);

    // Section starts, in document order, with the count that opens each one.
    let mut sections: Vec<(usize, f64)> = Vec::new();
    for marker in MARKERS {
        for (at, _) in flat.match_indices(marker) {
            if let Some(students) = count_before(&flat, at) {
                sections.push((at, students));
            }
        }
    }
    sections.sort_by_key(|(at, _)| *at);

    if sections.len() != PROGRAMMES.len() {
        return Err(format!(
            "the scholarship report names {} programmes, not {}; its structure changed",
            sections.len(),
            PROGRAMMES.len()
        ));
    }

    let mut out = Vec::new();
    for (i, (at, students)) in sections.iter().enumerate() {
        let end = sections.get(i + 1).map_or(flat.len(), |(next, _)| *next);
        let window = &flat[*at..end];
        // Which programme this is, read from the sentence itself rather than assumed from order.
        let (key, name) = PROGRAMMES
            .iter()
            .find(|(_, phrase)| window.contains(phrase))
            .ok_or_else(|| format!("the section at {at} names no known programme"))?;
        out.push(vec![
            (*key).to_string(),
            (*name).to_string(),
            format!("{students:.0}"),
            dollars_after(window, "totaled").map_or(String::new(), |v| format!("{v:.2}")),
            dollars_after(window, "per participant was")
                .map_or(String::new(), |v| format!("{v:.2}")),
        ]);
    }
    Ok(out)
}
