//! An enrolled act, read as the legislature prints it.
//!
//! # Why a reader for one document
//!
//! `legislation/hb-583-2022` publishes three counts drawn from this fixture — how many sections of
//! R.C. 3317 the act reprints, how many uncodified sections of H.B. 110 it reopens, and how many
//! of its own sections cite the one section it did **not** reopen. All three were computed inside
//! `tests/the_act_that_corrected_the_plan.rs`, which is a file nothing outside the test harness can
//! call, so the corpus could cite the test and could not be checked against it. That is #226, and
//! it is the shape #157 removed for eleven fixtures and #158 for five computations.
//!
//! The functions here are general over an act rather than specialised to this one, because the
//! distinction they draw — a section a bill *reprints* against a section it merely *cites* — is how
//! every Ohio amending act works and not a property of H.B. 583.
//!
//! # A heading is an amendment; a citation is not
//!
//! Ohio amends by reprinting. A section the act changes appears whole, under a `Sec. 3317.011.`
//! heading, and SECTION 2 repeals its prior form. A number that appears only inside somebody
//! else's text is a cross-reference, and the section it points at is untouched. The whole of
//! `hb-583-2022`'s central finding is that R.C. 3317.022 — the section that assembles foundation
//! funding — is in the second class and not the first.
//!
//! # What [`sections_citing`] cannot tell you, and why it does not have to here
//!
//! An enrolled act prints an amendment as strike-and-insert, and the strike is typographic: it
//! does not survive text extraction. So `of section 3314.08 3317.022 of the Revised Code` is one
//! reference with two readings, and this reader sees both numbers alike.
//!
//! Measured on H.B. 583 the ambiguity does not reach the answer. Four of the twenty-three printed
//! occurrences sit in such a pair — two where `3317.022` is the struck half and two where it is
//! the inserted one — and both of the sections holding them, `3317.0215` and `3317.25`, also carry
//! a plain reference. **The set of citing sections is the same ten on every reading.** A different
//! act might not be so lucky, which is why this is stated rather than assumed, and why the count
//! this exports is of *sections* and not of occurrences.

/// Sub. H.B. 583 of the 134th General Assembly, as enrolled — `ohio-session-laws`, code `08_EN`.
pub const HB583: &str = include_str!("../fixtures/hb583-corrections.txt");

/// Every section the act reprints as a heading, in document order, without repeats.
///
/// The section a heading names is one the act **amends**. Filter by prefix to ask about a chapter:
/// `headings(HB583).iter().filter(|h| h.starts_with("3317."))` is the thirteen the corpus states.
#[must_use]
pub fn headings(act: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    for (at, _) in act.match_indices("Sec.") {
        let Some(number) = number_after(act, at + "Sec.".len()) else {
            continue;
        };
        if !out.contains(&number) {
            out.push(number);
        }
    }
    out
}

/// Whether the act reprints a section — that is, amends it rather than pointing at it.
#[must_use]
pub fn reprints(act: &str, number: &str) -> bool {
    headings(act).contains(&number)
}

/// The act's own sections whose text cites a Revised Code section, in document order.
///
/// A citation before the first heading belongs to no section and is not counted; in an enrolled
/// act that region is the amending title, which lists what is being amended rather than saying
/// anything about it.
///
/// See the module docs for what a flattened strike-and-insert does to this, and for the
/// measurement showing it does not move the answer on [`HB583`].
#[must_use]
pub fn sections_citing<'a>(act: &'a str, number: &str) -> Vec<&'a str> {
    let mut heads: Vec<(usize, &str)> = Vec::new();
    for (at, _) in act.match_indices("Sec.") {
        if let Some(found) = number_after(act, at + "Sec.".len()) {
            heads.push((at, found));
        }
    }

    let mut out: Vec<&str> = Vec::new();
    for (at, _) in act.match_indices(number) {
        // `3317.022` must not be the front of `3317.0221`, and must not be the tail of a longer
        // run of digits — either would be a different section wearing this one's prefix.
        let before = act[..at].chars().next_back();
        let after = act[at + number.len()..].chars().next();
        if before.is_some_and(|c| c.is_ascii_digit() || c == '.')
            || after.is_some_and(|c| c.is_ascii_digit() || c == '.')
        {
            continue;
        }
        let Some(&(_, owner)) = heads.iter().rfind(|(start, _)| *start < at) else {
            continue;
        };
        if !out.contains(&owner) {
            out.push(owner);
        }
    }
    out
}

/// The section number printed after a `Sec.`, if one is.
fn number_after(act: &str, from: usize) -> Option<&str> {
    let rest = act.get(from..)?;
    let start = rest.len() - rest.trim_start_matches([' ', '\t', '\n', '\r']).len();
    let body = &rest[start..];
    let end = body
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(body.len());
    // A heading is `Sec. 3317.011.` — the run ends on the terminating period, which is not part of
    // the number. Without that period this is a sentence that happens to start with a numeral.
    let run = &body[..end];
    let number = run.strip_suffix('.')?;
    (!number.is_empty() && number.contains(char::is_numeric)).then_some(number)
}

/// The act's own text, with runs of whitespace collapsed.
///
/// `pdftotext` wraps at the printed column and an act's sentences cross the wrap, so a phrase
/// searched for in the raw fixture is a phrase that is sometimes there and sometimes not.
#[must_use]
pub fn flat(act: &str) -> String {
    act.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_heading_is_a_number_followed_by_its_own_period() {
        assert_eq!(number_after("Sec. 3317.011. (A)", 4), Some("3317.011"));
        assert_eq!(number_after("Sec.  265.150.", 4), Some("265.150"));
        // Running text, not a heading: no terminating period on the number.
        assert_eq!(number_after("Sec. 3317 of", 4), None);
    }

    #[test]
    fn a_citation_is_not_a_longer_section_wearing_its_prefix() {
        let act =
            "Sec. 100.1. under section 3317.022 of the Revised Code and section 3317.0221 too.";
        assert_eq!(sections_citing(act, "3317.022"), vec!["100.1"]);
    }

    #[test]
    fn a_citation_before_the_first_heading_belongs_to_no_section() {
        assert!(
            sections_citing("To amend section 3317.022 of the Revised Code.", "3317.022")
                .is_empty()
        );
    }
}
