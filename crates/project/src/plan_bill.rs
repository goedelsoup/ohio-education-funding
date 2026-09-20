//! The bill the Fair School Funding Plan was drafted in, and what its text settles.
//!
//! H.B. 1 of the 134th General Assembly — Callender and Sweeney, February 2021, "Create new
//! school financing system". It passed the House, died in the Senate, and H.B. 110 enacted the
//! formula four months later.
//!
//! # This is not authority for what the law is
//!
//! Where a bill and the act that followed differ, the act governs, and nothing here should ever
//! be read as stating current law — [`crate::statute`] is for that. This repository has published
//! a proposal as an appropriation before: six claims cited LSC's redbook, which analyses a bill as
//! introduced, and [`crate::ledger::budget_analysis::Edition`] exists so a caller has to name
//! which document it is reading. A bill version is the same hazard with a wider mouth, because it
//! looks exactly like statute.
//!
//! So the use this module is admitted for is **structural** — which provisions were written
//! together, citing what — and no figure, rate or threshold is taken from it.
//!
//! # What it settles
//!
//! `formula-component/fsfp-local-capacity-measure` asked three times why R.C. 3317.0217(C)(1)
//! adjusts its denominator for one of the channels R.C. 3317.03(A)(2) lists and not the rest, and
//! got two wrong answers — an artefact of the section predating school choice, then the surviving
//! half of a matched pair with prior law's netting. Both were inferences about drafting history
//! read off the shape of the surviving text, and the surviving text carries no such history.
//!
//! This bill does. It enacts R.C. 3317.0217 and amends R.C. 3317.03 in one document, and the
//! denominator already reads exactly as it does in force today. Same author, same bill, same
//! sitting.
//!
//! **And the list has not moved either.** H.B. 1 enumerates **ten** channels at (A)(2), `(a)`
//! through `(j)`, and current law enumerates the same ten. An earlier reading of this corpus said
//! nine — it had counted a truncated print of the section, and the count reached a test, a
//! decision record and four paragraphs of node prose before the two lists were set side by side.
//! The adjustment takes one of ten.
//!
//! # Reading it
//!
//! The publisher prints line numbers down the right margin, and `pdftotext -layout` leaves them
//! inside the sentences as bare four-digit integers. [`flat`] strips them, because a phrase search
//! over the raw text fails on any sentence long enough to cross a line — which is most of them.

/// The committed extract: the whole bill, as introduced.
pub const FIXTURE: &str = include_str!("../fixtures/hb1-134-as-introduced.txt");

/// The bill with its margin line numbers removed and its line breaks collapsed.
///
/// # Why the number has to come off positionally
///
/// The publisher prints a line number in the right margin, which `pdftotext -layout` leaves as a
/// trailing field: `X the district's enrolled ADM for that fiscal year].    4538`. A sentence
/// spans three lines with two of these inside it, so a phrase search over the raw text finds
/// almost nothing.
///
/// Stripping them **by token shape is wrong and silently corrupts the document.** A first attempt
/// dropped every four-digit token and turned "for fiscal year 2022 and each fiscal year
/// thereafter" into "for fiscal year and each fiscal year thereafter" — in a corpus about school
/// funding, where a stray year changes what a provision says. The number is identified by its
/// position at the end of a line, never by looking like one.
///
/// Page furniture goes the same way: `H. B. No. 1 Page 213 As Introduced` is a header that lands
/// mid-sentence when the lines are joined, and it is matched as a whole phrase.
#[must_use]
pub fn flat() -> String {
    let mut out = String::new();
    for line in FIXTURE.lines() {
        let trimmed = line.trim_end();
        // The margin number is the last whitespace-separated field, and only when the line has
        // something before it — a line that is nothing but digits is a page number, not a margin.
        let body = match trimmed.rsplit_once(char::is_whitespace) {
            Some((head, tail))
                if !head.trim().is_empty() && tail.chars().all(|c| c.is_ascii_digit()) =>
            {
                head
            }
            _ => trimmed,
        };
        out.push_str(body.trim());
        out.push(' ');
    }
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut cleaned = collapsed.as_str().to_string();
    // "H. B. No. 1 Page 213 As Introduced" is a running header, and joining the lines drops it
    // into the middle of whatever sentence spanned the page break.
    while let Some(at) = cleaned.find("H. B. No. 1 Page ") {
        let tail = &cleaned[at..];
        let Some(end) = tail.find("As Introduced") else {
            break;
        };
        let after = at + end + "As Introduced".len();
        cleaned = format!("{} {}", &cleaned[..at], &cleaned[after..]);
    }
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One section of the bill, from its `Sec. <number>.` heading to the next section's.
///
/// Returns `None` where the bill does not carry that section, which for a section the bill's own
/// title says it enacts means the extract and the title have come apart.
#[must_use]
pub fn section(number: &str) -> Option<String> {
    let flat = flat();
    let heading = format!("Sec. {number}. ");
    let start = flat.find(&heading)?;
    let rest = &flat[start + heading.len()..];
    let end = rest.find("Sec. ").unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

/// The consecutive lettered items at the start of `text`, stopping where the run breaks.
///
/// `(a)` through `(j)` for R.C. 3317.03(A)(2). Reads the letters rather than the entries, because
/// what the corpus claims about that list is how many channels it has and which one is cited.
///
/// **The caller must bound the slice.** This walks forward while the letters stay consecutive, and
/// the Revised Code nests lettered lists inside lettered lists — run it over the whole section and
/// it sails out of (A)(2) at `(j)` and keeps counting into the next division's list, returning
/// sixteen. Pass the span between the list's own stem and whatever follows it.
#[must_use]
pub fn letters(text: &str) -> Vec<char> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut expected = 'a';
    for window in chars.windows(3) {
        if window[0] == '(' && window[1] == expected && window[2] == ')' {
            out.push(expected);
            expected = char::from(expected as u8 + 1);
        }
    }
    out
}

/// The bill's own amending title: every section it amends, enacts and repeals.
///
/// The title is the bill's self-description and the thing to check a claim about scope against —
/// a section this module reads should appear in it.
#[must_use]
pub fn amending_title() -> String {
    let flat = flat();
    let start = flat.find("To amend sections").unwrap_or(0);
    let end = flat[start..]
        .find("BE IT ENACTED")
        .map_or(flat.len(), |at| start + at);
    flat[start..end].to_string()
}

/// Every record in the extract, for callers that want the publisher's own line breaks.
#[must_use]
pub fn raw() -> &'static str {
    FIXTURE
}
