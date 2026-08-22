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
