//! The comparisons more than one integration test in this crate has to make, defined once.
//!
//! Three of these existed in three copies with three bodies. That is worse than it sounds for a
//! corpus whose whole claim is reproduction: `close` disagreed about how much rounding is
//! admissible, so the same reconstruction could be exact in one file and outside tolerance in
//! another, and nothing in either file said which was right. A tolerance is a claim about the
//! fixture's storage, and a claim belongs in one place.
//!
//! `median` had a sharper history — the workspace carried two definitions and published figures
//! from both. There is one now, in [`dispersion`], and the wrapper here exists only to sort first
//! and to name the non-empty precondition.
//!
//! Cargo compiles this module into every test binary that declares it, so a helper any one binary
//! does not call is dead code there. The allow is that and nothing else.
#![allow(dead_code)]

/// A tolerance that admits the fixture's own rounding and nothing else.
///
/// Dollar amounts are stored to the cent, so a reproduction can differ by half a cent from
/// rounding alone. Anything outside that is a mechanism that has not been understood.
pub fn close(computed: f64, published: f64) -> bool {
    agrees_within(0.01, computed, published)
}

/// [`close`] with the absolute allowance named, for a reconstruction that passes through more of
/// the department's roundings than one.
///
/// Every caller of this owes the reader a measured worst case. A tolerance chosen to make a test
/// pass is a test that has stopped checking anything.
///
/// # The allowance is absolute, and used to not be
///
/// This read `allowance.max(published.abs() * 1e-9)`, which is a second tolerance, in a different
/// unit, that wins whenever the figure exceeds ten million dollars. The comment above it named a
/// third — inputs "stored to nine places", conflating nine decimal places of *input* with 1e-9
/// *relative on the output*, which are not the same claim and do not imply one another.
///
/// On the statewide totals this crate reconciles, the relative term reached **$17.57** against a
/// stated allowance of one cent (#125). Injecting a dollar error 90% of the way to that band left
/// 204 of 205 tests green: at that width the assertion was not checking arithmetic, it was
/// checking that the figure was roughly the right size.
///
/// The term bought nothing. Measured across every comparison this module makes on a figure above
/// ten million dollars, the largest real disagreement is **$0.0049** — under the half cent the
/// storage alone explains — and on the largest figure compared, $17,571,781,352.21, it is
/// **$0.000038**. Float noise sits four orders of magnitude below the cent, so the cent covers it
/// without help. Removing the term entirely leaves the crate at 278 passing, 0 failing.
pub fn agrees_within(allowance: f64, computed: f64, published: f64) -> bool {
    (computed - published).abs() <= allowance
}

/// [`close`], for a total reconstructed by adding `parts` separately-rounded amounts.
///
/// Each part is stored to the cent and so carries up to half a cent of rounding, and the published
/// total is rounded once more. Adding six of them can legitimately land a cent away from a figure
/// that is right — which is a statement about the fixture's storage and not about the formula.
///
/// Absolute, for the reason given on [`agrees_within`]: the relative term it used to carry was a
/// second tolerance in a different unit, and the measurements say the storage term alone covers
/// every real disagreement.
pub fn reconciles(computed: f64, published: f64, parts: usize) -> bool {
    (computed - published).abs() <= 0.005 * (parts as f64 + 1.0)
}

/// The median, on the one definition this workspace has.
///
/// Was a local upper-of-two in two of these files, which disagrees with [`dispersion`] on every
/// even-length series.
///
/// # Panics
///
/// On an empty series. Every caller here takes the median of a district panel, so an empty one is
/// a broken fixture rather than a case to handle.
pub fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    dispersion::median(&values).expect("a median is taken of a non-empty series here")
}

/// Collapse a document's page furniture so a quoted phrase matches across a line break.
///
/// The text fixtures are printed legislative documents, and a phrase this corpus quotes is as
/// likely to straddle a line, a page number, or a running header as not.
pub fn flat(document: &str) -> String {
    document.split_whitespace().collect::<Vec<&str>>().join(" ")
}
