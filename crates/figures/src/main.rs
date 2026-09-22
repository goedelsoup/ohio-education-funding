//! `edfund-figures` — write the corpus's figure manifest, or its series manifest, to stdout.
//!
//! A thin shell over [`figures`], on the same pattern as `bundle`'s binary: everything worth
//! testing is in the library, and each committed artefact is a redirect.
//!
//! ```text
//! cargo run --quiet -p figures > crates/figures.json
//! cargo run --quiet -p figures series > crates/series.json
//! ```
//!
//! `mise run //:generated` diffs what this prints against both committed copies, so a calculator
//! change that moves a figure or a bar and does not regenerate is a red gate rather than a corpus
//! check comparing against last month.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => print!("{}", figures::manifest()),
        ["series"] => print!("{}", figures::series_manifest()),
        other => {
            eprintln!(
                "edfund-figures: takes no argument (the figure manifest) or `series` (the series \
                 manifest), not {other:?}"
            );
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
