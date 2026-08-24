//! `edfund-figures` — write the corpus's figure manifest to stdout.
//!
//! A thin shell over [`figures`], on the same pattern as `bundle`'s binary: everything worth
//! testing is in the library, and the committed artefact is a redirect.
//!
//! ```text
//! cargo run --quiet -p figures > crates/figures.json
//! ```
//!
//! `mise run //:generated` diffs what this prints against the committed copy, so a calculator
//! change that moves a figure and does not regenerate is a red gate rather than a corpus check
//! comparing against last month.

fn main() {
    print!("{}", figures::manifest());
}
