//! The deflator's index points, checked against the Bureau of Labor Statistics.
//!
//! [`deflate`](../../deflate/) marks every point of its CPI-U June series
//! [`Confidence::Verified`](deflator::Confidence::Verified). That is a claim about the outside
//! world, and this is what makes it checkable: the committed extract in
//! `crates/connect/fixtures/cpi-u-june.tsv` is the Bureau's own published rows for
//! `CUUR0000SA0` period `M06`, taken verbatim from the flat file whose digest is pinned in
//! `crates/connect/source-digests.txt`.
//!
//! If a point in `deflate` is ever edited without going back to the source, this fails.

use connect::cpi::{self, ALL_ITEMS_NSA, JUNE};

const EXTRACT: &str = include_str!("../fixtures/cpi-u-june.tsv");

#[test]
fn every_committed_index_point_matches_the_published_series() {
    let observations = cpi::parse_series(EXTRACT, ALL_ITEMS_NSA, JUNE);
    let checks = cpi::check_committed_series(&observations);

    assert_eq!(
        checks.len(),
        27,
        "FY2000 through FY2026, less the years the Bureau has no June for"
    );
    let disagreements: Vec<String> = checks
        .iter()
        .filter(|check| !check.agrees())
        .map(|check| {
            format!(
                "{}: committed {:.3}, published {}",
                check.fiscal_year,
                check.committed,
                check
                    .published
                    .map_or_else(|| "absent".into(), |v| format!("{v:.3}"))
            )
        })
        .collect();
    assert!(
        disagreements.is_empty(),
        "the deflator disagrees with the Bureau at:\n  {}",
        disagreements.join("\n  ")
    );
}

#[test]
fn the_extract_covers_more_than_the_years_the_deflator_uses() {
    // The whole June series is committed, not only FY2000-FY2022. Extending the deflator
    // backwards should not require a re-fetch — and the 1913-forward coverage is what makes a
    // corpus that starts in 1851 able to grow towards its own beginning.
    let observations = cpi::parse_series(EXTRACT, ALL_ITEMS_NSA, JUNE);
    assert!(
        observations.len() > 100,
        "{} observations",
        observations.len()
    );
    assert!(observations.first().is_some_and(|o| o.calendar_year < 1950));
}

#[test]
fn the_extract_holds_exactly_one_series_and_period() {
    // A stray series in the extract would let a wrong number pass the check above.
    for line in EXTRACT.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        assert_eq!(fields[0], ALL_ITEMS_NSA, "{line}");
        assert_eq!(fields[2], JUNE, "{line}");
    }
}

#[test]
fn the_fy2016_correction_is_pinned_to_the_published_value() {
    // The error this connector was built to find. Pinned so that reverting it fails loudly
    // rather than quietly restoring a hand-typed digit.
    let observations = cpi::parse_series(EXTRACT, ALL_ITEMS_NSA, JUNE);
    let published = observations
        .iter()
        .find(|o| o.calendar_year == 2016)
        .expect("June 2016 is in the extract");
    assert!((published.index - 241.018).abs() < 0.000_5, "{published:?}");
}
