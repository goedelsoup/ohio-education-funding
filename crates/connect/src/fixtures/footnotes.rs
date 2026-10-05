//! One item out of an issue of LSC's *Budget Footnotes*, cut out of the whole issue.
//!
//! An issue is a month of GRF revenue and spending against estimate, followed by a dozen short
//! "issue updates" on whatever the Legislative Budget Office thought worth a page. The corpus cites
//! one of those updates, and committing the other thirty pages would put Medicaid caseloads and
//! homelessness grants in a fixture that a scholarship test reads.

/// The heading the January 2026 issue's traditional EdChoice item begins at.
///
/// Matched on its first line only: `pdftotext -layout` breaks the heading across two lines, and the
/// second, "EdChoice Increases for the 2026-2027 School Year", is the part LSC would reword for
/// another year.
const EDCHOICE_HEADING: &str = "Number of School Buildings Designated for Traditional";

/// The heading of the item that follows it, where the slice ends.
const NEXT_HEADING: &str = "Department of Development Awards";

/// Reduce the January 2026 issue to its traditional EdChoice item.
///
/// The item is about the FY2027 designated-building list, and its last paragraph is the one the
/// corpus wants: LSC's in-year count of traditional EdChoice students for FY2026 and its estimate
/// of their cost. The page footer and footnote that fall inside the item are kept, since cutting
/// them would mean the fixture is no longer the publisher's text.
///
/// Returns `None` if either heading is absent, or if they occur out of order — the document is
/// then not the one this was written against, and committing a slice of the wrong thing is worse
/// than committing nothing.
#[must_use]
pub fn extract_edchoice_update(text: &str) -> Option<String> {
    // The heading appears in the issue's contents list first, as "Traditional EdChoice Scholarship
    // Program (p. 17)", which does not match; the body heading is the only full match.
    let start = text.find(EDCHOICE_HEADING)?;
    let rest = &text[start..];
    let end = rest.find(NEXT_HEADING)?;
    Some(rest[..end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slice_runs_from_the_heading_to_the_next_item() {
        let issue = "Contents\nTraditional EdChoice Scholarship Program (p. 17)\n\
                     Number of School Buildings Designated for Traditional\n\
                     EdChoice Increases for the 2026-2027 School Year\n\
                     approximately 42,000 students\n\n\
                     Department of Development Awards $18.7 Million for\nHomelessness";
        let slice = extract_edchoice_update(issue).expect("both headings present");
        assert!(slice.starts_with(EDCHOICE_HEADING));
        assert!(slice.ends_with("approximately 42,000 students"));
        assert!(!slice.contains("Contents"));
        assert!(!slice.contains("Homelessness"));
    }

    #[test]
    fn a_missing_heading_is_no_slice_rather_than_the_wrong_one() {
        assert_eq!(
            extract_edchoice_update("Department of Development Awards"),
            None
        );
        assert_eq!(
            extract_edchoice_update("Number of School Buildings Designated for Traditional"),
            None
        );
    }

    #[test]
    fn a_next_heading_before_the_item_does_not_end_it() {
        // The next item's heading is searched for only after the item begins, so one appearing
        // earlier in the issue (in the contents list, say) cannot produce an empty or inverted
        // slice.
        let issue = "Department of Development Awards (p. 18)\n\
                     Number of School Buildings Designated for Traditional\nbody\n\
                     Department of Development Awards $18.7 Million";
        let slice = extract_edchoice_update(issue).expect("both headings present in order");
        assert_eq!(
            slice,
            "Number of School Buildings Designated for Traditional\nbody"
        );
    }
}
