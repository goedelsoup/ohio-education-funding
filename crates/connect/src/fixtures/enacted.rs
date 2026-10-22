//! The school funding chapter of an enacted budget act, cut out of the whole act.
//!
//! An act runs to thousands of pages and the corpus needs one department's appropriations out of
//! it. The heading is matched rather than a page range, because the page range moves every
//! biennium and the heading does not.

/// The heading the school funding portion of an LSC final analysis begins at.
///
/// The department's own name, in the document's heading case. "Primary and secondary education"
/// was the first guess and appears nowhere — the rebuild reported the fixture skipped and said so,
/// which is the behaviour the skip exists for.
const FUNDING_HEADING: &str = "DEPARTMENT OF EDUCATION AND WORKFORCE";

/// Reduce a final analysis to the part that funds schools.
///
/// # Why an extract rather than the whole document
///
/// The H.B. 96 final analysis is 26,000 lines and legislates on everything from Medicaid to
/// liquor permits. Committing it whole would put a megabyte of unrelated law in the repository
/// and make the diff on the next budget unreadable. The school funding portion is what any node
/// here cites.
///
/// Returns `None` if the heading is absent, which means the document is not the document this was
/// written against — better than committing a slice of the wrong thing.
#[must_use]
pub fn extract_school_funding(text: &str) -> Option<String> {
    // The heading appears in the contents list first and as a body heading after; take the last
    // occurrence before the education content, which is the body one.
    let start = text.match_indices(FUNDING_HEADING).map(|(i, _)| i).nth(1)?;
    let rest = &text[start..];
    // Runs to the next top-level department heading.
    let end = ["DEPARTMENT OF HIGHER EDUCATION", "DEPARTMENT OF MEDICAID"]
        .iter()
        .filter_map(|m| rest.find(m))
        .filter(|i| *i > 10_000)
        .min()
        .unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

/// The heading the final analysis's school district levy limits begin at.
const LEVY_HEADING: &str = "Levy restrictions due to excess carry-over";

/// The heading they run to: the overview of a provision the Governor vetoed and the General
/// Assembly let stand vetoed, which is where the analysis turns from what the act does to what
/// it would have done.
const LEVY_END: &str = "20-mill floor: overview";

/// Reduce a final analysis to its school district levy limits.
///
/// The property tax provisions the corpus cites are in the analysis's tax chapter, not the
/// department's, so [`extract_school_funding`] does not reach them: the carry-over limit on new
/// current expense levies, the end of new emergency and substitute levies, the end of renewing
/// and increasing, the end of replacement levies, the ballot's "market value", and the vetoed
/// change to the twenty-mill floor. Each of the first five says the General Assembly overrode the
/// veto, and the passage says when they apply. Those two facts are what an amendment's effective
/// date on `codes.ohio.gov` cannot say.
///
/// Returns `None` if either heading is absent after the contents list.
#[must_use]
pub fn extract_levy_limits(text: &str) -> Option<String> {
    // The heading is in the contents list first, with its dot leaders, and in the body second.
    let start = text.match_indices(LEVY_HEADING).map(|(i, _)| i).nth(1)?;
    let rest = &text[start..];
    let end = rest.find(LEVY_END)?;
    Some(rest[..end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_levy_limits_are_the_body_passage_and_stop_before_the_vetoed_overview() {
        let text = "Contents\n  Levy restrictions due to excess carry-over ..... 483\n  \
                    20-mill floor: overview ..... 485\nOther law\n  \
                    Levy restrictions due to excess carry-over (VETO OVERRIDDEN)\n  \
                    Replacement property tax levies (VETO OVERRIDDEN)\n  \
                    20-mill floor limits (VETOED)\n      20-mill floor: overview\n  Detail";
        let slice = extract_levy_limits(text).expect("both headings present");
        assert!(slice.starts_with("Levy restrictions due to excess carry-over (VETO OVERRIDDEN)"));
        assert!(slice.contains("Replacement property tax levies"));
        assert!(slice.ends_with("20-mill floor limits (VETOED)"));
        assert!(!slice.contains("Detail"));
    }

    #[test]
    fn a_contents_list_alone_is_not_the_passage() {
        let text = "Levy restrictions due to excess carry-over ..... 483\n20-mill floor: overview";
        assert_eq!(extract_levy_limits(text), None);
    }
}
