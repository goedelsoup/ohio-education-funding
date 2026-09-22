//! The crate half of the series cross-check.
//!
//! `web/tests/unit/corpusSeries.spec.ts` asserts that a corpus node binds a series
//! `crates/series.json` carries, and that the node also binds the figures at the series' ends.
//! These are the assertions that make the second half meaningful: that every series *has* figures
//! at its ends, that they are figures the manifest actually carries, and that they reproduce the
//! rows they stand for. The registry invariants sit here for the reason the figure manifest's do —
//! a defect in `SERIES` reported by the consumer would be reported to whoever is editing the
//! corpus, which is the wrong desk.

use std::collections::BTreeSet;

use figures::{compute_all, compute_all_series, Row, FIGURES, SERIES, SERIES_CONTRACT_VERSION};

/// The rows a reader would quote: the top of the chart and the bottom of it.
fn endpoints(rows: &[Row]) -> [&Row; 2] {
    let largest = rows
        .iter()
        .max_by(|a, b| a.value.total_cmp(&b.value))
        .expect("a series has rows");
    let smallest = rows
        .iter()
        .min_by(|a, b| a.value.total_cmp(&b.value))
        .expect("a series has rows");
    [largest, smallest]
}

/// The endpoint rule: the largest and smallest row of every series name an ordinary figure.
///
/// A chart is read by its extremes, and a reader who quotes the tallest bar has quoted a number.
/// The figure manifest is where numbers the corpus quotes are pinned and bound, so the extremes
/// of every series have to be there — this is what makes a series a *view* of the figures rather
/// than a second, unpinned source of them.
#[test]
fn the_largest_and_smallest_row_of_every_series_name_a_figure() {
    for entry in compute_all_series() {
        let s = entry.series;
        assert!(
            entry.rows.len() >= 2,
            "{}: {} row(s), which is a figure and not a column",
            s.key,
            entry.rows.len()
        );
        for end in endpoints(&entry.rows) {
            assert!(
                end.figure.is_some(),
                "{}: the row {:?} at {} is an endpoint of the chart and names no figure. Bind it \
                 as an ordinary figure in FIGURES first; a chart cannot draw a number the figure \
                 manifest has not stood behind.",
                s.key,
                end.label,
                end.value
            );
        }
    }
}

/// Every figure a row names exists, is on the row's unit, and reproduces the row in magnitude.
///
/// Magnitude and not value, because the figure manifest's convention is that a correlation is
/// exported unsigned with the direction in its key, and a chart draws the direction instead. The
/// comparison is against the figure's *pin* and within its *tolerance*, so a row is held exactly
/// as tightly as the figure it stands on, and `the_manifest_reproduces_its_pins` is what holds
/// the pin to the calculator.
#[test]
fn a_row_that_names_a_figure_reproduces_it() {
    let figures = compute_all();
    for entry in compute_all_series() {
        let s = entry.series;
        for r in &entry.rows {
            let Some(key) = r.figure else { continue };
            let figure = figures
                .iter()
                .find(|c| c.figure.key == key)
                .unwrap_or_else(|| {
                    panic!(
                        "{}: the row {:?} names {key}, which FIGURES does not carry",
                        s.key, r.label
                    )
                });
            assert_eq!(
                figure.figure.unit, s.unit,
                "{}: the row {:?} is measured in {:?} and names a figure measured in {:?}",
                s.key, r.label, s.unit, figure.figure.unit
            );
            let drift = (r.value.abs() - figure.figure.pinned).abs();
            assert!(
                drift <= figure.figure.tolerance,
                "{}: the row {:?} is {} and names {key}, pinned at {} \u{2014} {drift} apart in \
                 magnitude, tolerating {}. The two are meant to be one computation; if they have \
                 diverged, the series is drawing something the corpus does not quote.",
                s.key,
                r.label,
                r.value,
                figure.figure.pinned,
                figure.figure.tolerance
            );
        }
    }
}

/// Keys are unique, name their owner's directory, and share no key with a figure.
///
/// A corpus node binds series and figures in two lists, so a key that appeared in both manifests
/// would be two different things a reader could not tell apart by name.
#[test]
fn every_key_is_unique_and_names_its_owner() {
    let figure_keys: BTreeSet<&str> = FIGURES.iter().map(|f| f.key).collect();
    let mut seen = BTreeSet::new();
    for s in SERIES {
        assert!(seen.insert(s.key), "{}: two series share this key", s.key);
        assert!(
            !figure_keys.contains(s.key),
            "{}: is also a figure key, and a corpus node could not say which it bound",
            s.key
        );

        let directory = s
            .owner
            .strip_prefix("crates/")
            .unwrap_or_else(|| panic!("{}: owner {:?} is not under crates/", s.key, s.owner));
        assert!(
            s.key.starts_with(&format!("{directory}/")),
            "{}: owned by {} and so should be keyed `{directory}/…`",
            s.key,
            s.owner
        );
        assert!(
            s.key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '/'),
            "{}: keys are lower-case kebab so a corpus node can hold one without quoting",
            s.key
        );
        assert!(
            FIGURES.iter().any(|f| f.owner == s.owner),
            "{}: owned by {}, which owns no figure \u{2014} so its endpoints cannot be bound",
            s.key,
            s.owner
        );
    }
}

/// Row labels are distinct within a series, and every string is a sentence, writable unescaped.
#[test]
fn every_string_is_safe_to_write_unescaped() {
    for entry in compute_all_series() {
        let s = entry.series;
        assert!(
            s.label.len() > 20,
            "{}: {:?} does not say what the series is",
            s.key,
            s.label
        );
        assert!(!s.axis.is_empty(), "{}: the axis is unnamed", s.key);
        let mut labels = BTreeSet::new();
        for r in &entry.rows {
            assert!(!r.label.is_empty(), "{}: a row has no label", s.key);
            assert!(
                labels.insert(r.label),
                "{}: two rows are labelled {:?}",
                s.key,
                r.label
            );
            assert!(
                r.value.is_finite(),
                "{}: the row {:?} is {}, which JSON cannot carry",
                s.key,
                r.label,
                r.value
            );
        }
        let strings = [s.key, s.owner, s.axis, s.label]
            .into_iter()
            .chain(entry.rows.iter().map(|r| r.label))
            .chain(entry.rows.iter().filter_map(|r| r.hover))
            .chain(entry.rows.iter().filter_map(|r| r.figure));
        for text in strings {
            assert!(
                !text.contains(['"', '\\']) && !text.chars().any(char::is_control),
                "{}: {text:?} carries a character the manifest writer cannot escape",
                s.key
            );
        }
    }
}

/// The same tree produces the same bytes, twice in one process. See the figure manifest's
/// `the_manifest_is_stable` for the hasher that made this worth asserting.
#[test]
fn the_manifest_is_stable() {
    assert_eq!(
        figures::series_manifest(),
        figures::series_manifest(),
        "two runs in one process disagree, so the committed artefact cannot be regenerated"
    );
}

/// The document parses as JSON, declares its own contract, and carries every series and row once.
#[test]
fn the_document_is_readable_and_complete() {
    let json = figures::series_manifest();
    assert!(json.starts_with("{\n"), "the document is an object");
    assert!(
        json.ends_with("}\n"),
        "the document is closed and newline-terminated"
    );
    assert!(
        json.contains(&format!("\"contract\": \"{SERIES_CONTRACT_VERSION}\"")),
        "the document states its contract version"
    );
    assert_eq!(
        json.matches("{\"key\": ").count(),
        SERIES.len(),
        "every series is written exactly once"
    );
    let rows: usize = compute_all_series().iter().map(|e| e.rows.len()).sum();
    assert_eq!(
        json.matches("{\"label\": ").count(),
        rows,
        "every row is written exactly once"
    );
    for s in SERIES {
        assert!(
            json.contains(&format!("\"key\": \"{}\"", s.key)),
            "{} is missing from the manifest",
            s.key
        );
    }
    assert_eq!(
        json.matches('{').count(),
        json.matches('}').count(),
        "the writer closed every object it opened"
    );
    assert_eq!(
        json.matches('[').count(),
        json.matches(']').count(),
        "the writer closed every array it opened"
    );
}
