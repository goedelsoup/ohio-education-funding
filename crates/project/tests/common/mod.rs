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
/// rounding alone; the inputs are stored to nine places, which contributes a relative error far
/// below the second term. Anything outside both is a mechanism that has not been understood.
pub fn close(computed: f64, published: f64) -> bool {
    agrees_within(0.01, computed, published)
}

/// [`close`] with the absolute allowance named, for a reconstruction that passes through more of
/// the department's roundings than one.
///
/// Every caller of this owes the reader a measured worst case. A tolerance chosen to make a test
/// pass is a test that has stopped checking anything.
pub fn agrees_within(allowance: f64, computed: f64, published: f64) -> bool {
    (computed - published).abs() <= allowance.max(published.abs() * 1e-9)
}

/// [`close`], for a total reconstructed by adding `parts` separately-rounded amounts.
///
/// Each part is stored to the cent and so carries up to half a cent of rounding, and the published
/// total is rounded once more. Adding six of them can legitimately land a cent away from a figure
/// that is right — which is a statement about the fixture's storage and not about the formula.
pub fn reconciles(computed: f64, published: f64, parts: usize) -> bool {
    (computed - published).abs() <= 0.005 * (parts as f64 + 1.0) + published.abs() * 1e-9
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
