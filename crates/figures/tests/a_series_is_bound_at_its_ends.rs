//! The crate half of the series cross-check.
//!
//! `web/tests/unit/corpusSeries.spec.ts` asserts that a corpus node binds a series
//! `crates/series.json` carries, and that the node also binds the figures at the series' ends.
//! These are the assertions that make the second half meaningful: that every series *has* figures
//! at its ends, that they are figures the manifest actually carries, and that they reproduce the
//! rows they stand for. The registry invariants sit here for the reason the figure manifest's do —
//! a defect in `SERIES` reported by the consumer would be reported to whoever is editing the
//! corpus, which is the wrong desk.
//!
//! A cloud has no rows, so it has no endpoints; what a reader quotes off a fitted panel is its
//! slope and its r-squared, and those are what the same rule holds. The assertions below are the
//! same three in the same order — every panel names figures, the figures exist and reproduce the
//! line, and the keys are unique across both registries and the figure manifest.

use std::collections::BTreeSet;

use figures::{
    compute_all, compute_all_scatters, compute_all_series, Row, FIGURES, SCATTERS, SERIES,
    SERIES_CONTRACT_VERSION,
};

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

/// The document parses as JSON, declares its contract, and carries every member of both arrays
/// exactly once.
///
/// Both arrays, since the document gained a second: a cloud is keyed the same way a series is and
/// draws one line per point for the same reason a series draws one per row, so the counts below
/// are over the two registries together and a cloud silently dropped would fail the first of them.
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
        SERIES.len() + SCATTERS.len(),
        "every series and every cloud is written exactly once"
    );
    let rows: usize = compute_all_series().iter().map(|e| e.rows.len()).sum();
    // Per cloud: every point, every panel, and every axis, which is the shared x and one y per
    // panel. An axis leads with its label like everything else the writer emits.
    let drawn: usize = compute_all_scatters()
        .iter()
        .map(|e| e.cloud.points.len() + 2 * e.cloud.panels.len() + 1)
        .sum();
    assert_eq!(
        json.matches("{\"label\": ").count(),
        rows + drawn,
        "every row, panel, axis and point is written exactly once"
    );
    for key in SERIES
        .iter()
        .map(|s| s.key)
        .chain(SCATTERS.iter().map(|s| s.key))
    {
        assert!(
            json.contains(&format!("\"key\": \"{key}\"")),
            "{key} is missing from the manifest"
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

/// The endpoint rule as a cloud has it: every panel's slope and r-squared name an ordinary figure.
///
/// A scatter prints exactly two numbers and they are these. A panel whose r-squared no figure pins
/// is a panel that can drift a tenth without the figure gate noticing, and a reader who quotes it
/// has quoted an unpinned number off a picture — which is the thing the whole rule exists to
/// prevent.
#[test]
fn every_panel_of_every_cloud_names_a_figure_for_both_of_its_numbers() {
    for entry in compute_all_scatters() {
        let s = entry.scatter;
        assert!(
            !entry.cloud.panels.is_empty(),
            "{}: a cloud with no panels",
            s.key
        );
        for panel in &entry.cloud.panels {
            for (what, key) in [
                ("slope", panel.fit.slope_figure),
                ("r-squared", panel.fit.r_squared_figure),
            ] {
                assert!(
                    !key.is_empty(),
                    "{}: the panel {:?} prints its {what} and names no figure for it. Bind it as \
                     an ordinary figure in FIGURES first; a chart cannot print a number the \
                     figure manifest has not stood behind.",
                    s.key,
                    panel.label
                );
            }
        }
    }
}

/// Every figure a fit names exists, is a ratio, and reproduces the line in magnitude.
///
/// Magnitude for the reason a row is compared that way: the figure manifest exports a signed
/// quantity unsigned with the direction in its key, because the corpus's numeral reader cannot
/// see a minus. The panel draws the direction instead, which is the whole point of drawing it.
#[test]
fn a_fit_reproduces_the_figures_it_names() {
    let figures = compute_all();
    for entry in compute_all_scatters() {
        let s = entry.scatter;
        for panel in &entry.cloud.panels {
            for (what, key, value) in [
                ("slope", panel.fit.slope_figure, panel.fit.slope),
                ("r-squared", panel.fit.r_squared_figure, panel.fit.r_squared),
            ] {
                let figure = figures
                    .iter()
                    .find(|c| c.figure.key == key)
                    .unwrap_or_else(|| {
                        panic!(
                            "{}: the panel {:?} names {key} for its {what}, which FIGURES does \
                             not carry",
                            s.key, panel.label
                        )
                    });
                assert_eq!(
                    figure.figure.unit,
                    figures::Unit::Ratio,
                    "{}: {key} is a {what} and so is a ratio",
                    s.key
                );
                let drift = (value.abs() - figure.figure.pinned).abs();
                assert!(
                    drift <= figure.figure.tolerance,
                    "{}: the panel {:?} has {what} {value} and names {key}, pinned at {} \u{2014} \
                     {drift} apart in magnitude, tolerating {}. The two are meant to be one \
                     computation; if they have diverged, the chart is printing something the \
                     corpus does not quote.",
                    s.key,
                    panel.label,
                    figure.figure.pinned,
                    figure.figure.tolerance
                );
            }
        }
    }
}

/// Every panel is drawn at the same scale, and that scale is the predictor's own.
///
/// The assertion `Cloud::at_one_scale` makes on the way in, restated on the way out, because it
/// is the whole reason the pair is a picture rather than two pictures: a unit of any outcome is
/// drawn the same length as a unit of the predictor, so a slope is an angle and the panels are
/// comparable to each other. To 12 places, which is exponentiation and a logarithm apart.
#[test]
fn every_panel_spans_exactly_what_the_predictor_spans() {
    for entry in compute_all_scatters() {
        let s = entry.scatter;
        let x = &entry.cloud.x;
        assert!(
            x.log && x.min > 0.0,
            "{}: the predictor is a log axis",
            s.key
        );
        let span = (x.max / x.min).ln();
        for panel in &entry.cloud.panels {
            assert!(
                panel.y.log,
                "{}: {:?} is not a log axis",
                s.key, panel.label
            );
            let theirs = (panel.y.max / panel.y.min).ln();
            assert!(
                (theirs / span - 1.0).abs() < 1e-12,
                "{}: {:?} spans {theirs} log units against the predictor's {span}, so its slope \
                 of {} would be drawn as {}",
                s.key,
                panel.label,
                panel.fit.slope,
                panel.fit.slope * span / theirs
            );
        }
    }
}

/// Every point carries a value under every panel, inside the frame, with a name to be called by.
///
/// The first thing either manifest writes that is not a `&'static str`: a point's label is a
/// district's name out of a fixture. So the escaping check that is a typo guard everywhere else
/// is a check on data here, and it is worth the same assertion the registry's own strings get.
#[test]
fn every_point_is_placeable_and_named() {
    for entry in compute_all_scatters() {
        let s = entry.scatter;
        let panels = entry.cloud.panels.len();
        assert!(
            entry.cloud.points.len() >= 30,
            "{}: {} point(s), which is a table and not a cloud",
            s.key,
            entry.cloud.points.len()
        );
        for point in &entry.cloud.points {
            assert!(!point.label.is_empty(), "{}: a point has no label", s.key);
            assert!(
                !point.label.contains(['"', '\\']) && !point.label.chars().any(char::is_control),
                "{}: {:?} carries a character the manifest writer cannot escape",
                s.key,
                point.label
            );
            assert_eq!(
                point.ys.len(),
                panels,
                "{}: {:?} carries {} value(s) under {panels} panel(s)",
                s.key,
                point.label,
                point.ys.len()
            );
            assert!(
                point.x > 0.0 && point.x.is_finite(),
                "{}: {:?} is at {} on the predictor, which a log axis cannot place",
                s.key,
                point.label,
                point.x
            );
            for (at, panel) in entry.cloud.panels.iter().enumerate() {
                let y = point.ys[at];
                assert!(
                    y >= panel.y.min && y <= panel.y.max,
                    "{}: {:?} is at {y} on {:?}, whose frame is {} to {} \u{2014} the frame is \
                     computed to hold every point, so one outside it is a frame computed from \
                     something other than these points",
                    s.key,
                    point.label,
                    panel.label,
                    panel.y.min,
                    panel.y.max
                );
            }
        }
    }
}

/// A cloud's key is unique across both registries and the figure manifest, and names its owner.
#[test]
fn every_cloud_key_is_unique_and_names_its_owner() {
    let taken: BTreeSet<&str> = FIGURES
        .iter()
        .map(|f| f.key)
        .chain(SERIES.iter().map(|s| s.key))
        .collect();
    let mut seen = BTreeSet::new();
    for s in SCATTERS {
        assert!(seen.insert(s.key), "{}: two clouds share this key", s.key);
        assert!(
            !taken.contains(s.key),
            "{}: is also a figure or series key, and a corpus node could not say which it bound",
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
            s.label.len() > 20,
            "{}: {:?} does not say what the cloud is",
            s.key,
            s.label
        );
        assert!(
            !s.subject.is_empty() && s.subject == s.subject.to_lowercase(),
            "{}: the subject is what one point is, singular and lower case",
            s.key
        );
        for text in [s.key, s.owner, s.label, s.subject] {
            assert!(
                !text.contains(['"', '\\']) && !text.chars().any(char::is_control),
                "{}: {text:?} carries a character the manifest writer cannot escape",
                s.key
            );
        }
    }
}
