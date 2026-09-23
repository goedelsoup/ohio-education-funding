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
//! line, and the keys are unique across both registries and the figure manifest. A plane binds
//! every coordinate of every position, and a curve binds three numbers per line: both ends and
//! the worst departure from its reference. Four shapes, one rule — whatever a reader can quote
//! off the picture is a number the figure manifest has stood behind.

use std::collections::BTreeSet;

use figures::{
    compute_all, compute_all_curves, compute_all_planes, compute_all_scatters, compute_all_series,
    compute_all_spreads, Row, CURVES, FIGURES, PLANES, SCATTERS, SERIES, SERIES_CONTRACT_VERSION,
    SPREADS,
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
        // Distinct within a group rather than within the series, because `Row::group` is a
        // second level of the axis: a panel of seven rules with five wealth fifths each is
        // thirty-five rows and five labels, and the pair is what names one of them. Where a
        // series has no groups this is the old assertion unchanged.
        let mut labels = BTreeSet::new();
        for r in &entry.rows {
            assert!(!r.label.is_empty(), "{}: a row has no label", s.key);
            assert!(
                labels.insert((r.group, r.label)),
                "{}: two rows of {:?} are labelled {:?}",
                s.key,
                r.group.unwrap_or("the series"),
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

/// The document parses as JSON, declares its contract, and carries every member of every array
/// exactly once.
///
/// All five arrays: a cloud, a plane, a curve and a spread are keyed the same way a series is
/// and each draws one line per point for the same reason a series draws one per row, so the
/// counts below are over the five registries together and a drawing silently dropped would fail
/// the first of them.
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
        SERIES.len() + SCATTERS.len() + PLANES.len() + CURVES.len() + SPREADS.len(),
        "every series, cloud, plane, curve and spread is written exactly once"
    );
    let rows: usize = compute_all_series().iter().map(|e| e.rows.len()).sum();
    // Per cloud: every point, every panel, and every axis, which is the shared x and one y per
    // panel. An axis leads with its label like everything else the writer emits.
    let drawn: usize = compute_all_scatters()
        .iter()
        .map(|e| e.cloud.points.len() + 2 * e.cloud.panels.len() + 1)
        .sum();
    // And per plane: every position, and its two axes. No panels, because a plane is one frame.
    let placed: usize = compute_all_planes()
        .iter()
        .map(|e| e.positions.points.len() + 2)
        .sum();
    // Per curve: every line, its two axes, and the reference when it has one. A curve's *points*
    // are bare coordinates and carry no label, which is why they are not counted here — the index
    // position is the label, and repeating it thirteen times per line would be a longer diff
    // saying nothing.
    let traced: usize = compute_all_curves()
        .iter()
        .map(|e| e.traces.lines.len() + 2 + usize::from(e.traces.reference.is_some()))
        .sum();
    // Per spread: every mark, every panel, every boundary, every census row, everyone
    // unreached, and its two axes. A boundary is written as its two endpoints and they carry no
    // labels of their own, for the reason a curve's points do not: the line is the named thing.
    let spread: usize = compute_all_spreads()
        .iter()
        .map(|e| {
            e.regions
                .panels
                .iter()
                .map(|p| p.marks.len())
                .sum::<usize>()
                + e.regions.panels.len()
                + e.regions.boundaries.len()
                + e.regions.census.len()
                + e.regions.unreached.len()
                + 2
        })
        .sum();
    assert_eq!(
        json.matches("{\"label\": ").count(),
        rows + drawn + placed + traced + spread,
        "every row, panel, axis, point, position, line, mark, boundary, region and absence is \
         written exactly once"
    );
    for key in SERIES
        .iter()
        .map(|s| s.key)
        .chain(SCATTERS.iter().map(|s| s.key))
        .chain(PLANES.iter().map(|p| p.key))
        .chain(CURVES.iter().map(|c| c.key))
        .chain(SPREADS.iter().map(|s| s.key))
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

/// The endpoint rule as a plane has it, which is the strictest form: **every** position names a
/// figure for each of its two coordinates.
///
/// A series binds its ends and a cloud binds its fit, because in both cases those are the numbers
/// a reader can actually quote. A plane of seven named policies is different: every point is
/// labelled, every point is quotable by name, and both of its coordinates are readable off the
/// frame. So there is no end to bind and no line to pin — there is only all of them.
#[test]
fn every_position_names_a_figure_for_both_of_its_coordinates() {
    for entry in compute_all_planes() {
        let p = entry.plane;
        assert!(
            !entry.positions.points.is_empty(),
            "{}: a plane with no positions",
            p.key
        );
        for point in &entry.positions.points {
            for (what, key) in [("x", point.x_figure), ("y", point.y_figure)] {
                assert!(
                    !key.is_empty(),
                    "{}: the position {:?} is placed on {what} and names no figure for it. Bind \
                     it as an ordinary figure in FIGURES first; a plane names its points, so \
                     every coordinate on it is a number a reader can quote.",
                    p.key,
                    point.label
                );
            }
        }
    }
}

/// Every figure a position names exists, is measured in its axis' unit, and reproduces the
/// coordinate in magnitude.
///
/// Magnitude for the reason a row and a fit are compared that way: the figure manifest exports a
/// signed quantity unsigned with the direction in its key, because the corpus' numeral reader
/// cannot see a minus. Here that is not a technicality — the sign of each coordinate *is* the
/// finding, so it is drawn rather than written, and the key says which way it went.
#[test]
fn a_position_reproduces_the_figures_it_names() {
    let figures = compute_all();
    for entry in compute_all_planes() {
        let p = entry.plane;
        for point in &entry.positions.points {
            for (what, key, value, unit) in [
                ("x", point.x_figure, point.x, entry.positions.x.unit),
                ("y", point.y_figure, point.y, entry.positions.y.unit),
            ] {
                let figure = figures
                    .iter()
                    .find(|c| c.figure.key == key)
                    .unwrap_or_else(|| {
                        panic!(
                            "{}: the position {:?} names {key} for its {what}, which FIGURES \
                             does not carry",
                            p.key, point.label
                        )
                    });
                assert_eq!(
                    figure.figure.unit, unit,
                    "{}: the {what} axis is measured in {unit:?} and {key} in {:?}",
                    p.key, figure.figure.unit
                );
                let drift = (value.abs() - figure.figure.pinned).abs();
                assert!(
                    drift <= figure.figure.tolerance,
                    "{}: the position {:?} is at {what} = {value} and names {key}, pinned at {} \
                     \u{2014} {drift} apart in magnitude, tolerating {}. The two are meant to be \
                     one computation; if they have diverged, the plane is drawing something the \
                     corpus does not quote.",
                    p.key,
                    point.label,
                    figure.figure.pinned,
                    figure.figure.tolerance
                );
            }
        }
    }
}

/// Both axes of every plane are linear, hold every position, and hold **zero**.
///
/// Zero is not a data value here and it is the most important place on the picture: it is the
/// rule in force, and a plane exists to say which side of it a policy falls on for each of two
/// measures at once. An axis fitted to the points alone would drop it the moment every point
/// landed on one side, and then the drawing would show a spread where the claim is a sign.
#[test]
fn every_axis_of_every_plane_holds_zero_and_every_position() {
    for entry in compute_all_planes() {
        let p = entry.plane;
        let axes = [("x", &entry.positions.x), ("y", &entry.positions.y)];
        for (what, axis) in axes {
            assert!(
                !axis.log,
                "{}: the {what} axis is signed, and a log axis cannot place a negative",
                p.key
            );
            assert!(
                axis.min <= 0.0 && axis.max >= 0.0,
                "{}: the {what} axis runs {} to {} and does not hold zero, which is the rule in \
                 force and the line the whole plane is read against",
                p.key,
                axis.min,
                axis.max
            );
        }
        for point in &entry.positions.points {
            for (what, axis, value) in [
                ("x", &entry.positions.x, point.x),
                ("y", &entry.positions.y, point.y),
            ] {
                assert!(
                    value.is_finite() && value >= axis.min && value <= axis.max,
                    "{}: the position {:?} is at {what} = {value}, outside the drawn {} to {}",
                    p.key,
                    point.label,
                    axis.min,
                    axis.max
                );
            }
        }
    }
}

/// Plane keys are unique, name their owner's directory, and collide with nothing already keyed.
#[test]
fn every_plane_key_is_unique_and_names_its_owner() {
    let taken: BTreeSet<&str> = FIGURES
        .iter()
        .map(|f| f.key)
        .chain(SERIES.iter().map(|s| s.key))
        .chain(SCATTERS.iter().map(|s| s.key))
        .collect();
    let mut seen = BTreeSet::new();
    for p in PLANES {
        assert!(seen.insert(p.key), "{}: two planes share this key", p.key);
        assert!(
            !taken.contains(p.key),
            "{}: is also a figure, series or cloud key, and a corpus node could not say which it \
             bound",
            p.key
        );
        let directory = p
            .owner
            .strip_prefix("crates/")
            .unwrap_or_else(|| panic!("{}: owner {:?} is not under crates/", p.key, p.owner));
        assert!(
            p.key.starts_with(&format!("{directory}/")),
            "{}: owned by {} and so should be keyed `{directory}/…`",
            p.key,
            p.owner
        );
        assert!(
            p.key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '/'),
            "{}: keys are lower-case kebab so a corpus node can hold one without quoting",
            p.key
        );
        assert!(
            p.label.len() > 20,
            "{}: {:?} does not say what the plane is",
            p.key,
            p.label
        );
        assert!(
            !p.subject.is_empty() && p.subject == p.subject.to_lowercase(),
            "{}: the subject is what one point is, singular and lower case",
            p.key
        );
    }
}

/// Every position is labelled, and distinctly, so the two coordinates belong to a named thing.
#[test]
fn every_position_is_named_and_writable() {
    for entry in compute_all_planes() {
        let p = entry.plane;
        let mut seen = BTreeSet::new();
        for point in &entry.positions.points {
            assert!(
                !point.label.is_empty(),
                "{}: a position has no label, and a plane is quoted by name",
                p.key
            );
            assert!(
                seen.insert(point.label.as_str()),
                "{}: two positions are labelled {:?}",
                p.key,
                point.label
            );
        }
        let strings = [p.key, p.owner, p.label, p.subject]
            .into_iter()
            .chain(entry.positions.points.iter().map(|q| q.label.as_str()))
            .chain(entry.positions.points.iter().map(|q| q.x_figure))
            .chain(entry.positions.points.iter().map(|q| q.y_figure))
            .chain([entry.positions.x.label, entry.positions.y.label]);
        for text in strings {
            assert!(
                !text.contains(['"', '\\']) && !text.chars().any(char::is_control),
                "{}: {text:?} carries a character the manifest writer cannot escape",
                p.key
            );
        }
    }
}

/// The endpoint rule as a curve has it: every line names a figure for both ends and for its worst
/// departure from the reference.
///
/// A series binds its two extreme rows, a cloud its fit, a plane every coordinate. A curve is
/// none of those: reordering its index would destroy what it says, so its extremes are not
/// "largest and smallest" but *first and last*, per line, and the claim it is usually drawn for —
/// that one line stays flat against the reference while the other falls away — is a maximum over
/// the whole index and not a value at any point on it. Three figures per line is what it takes to
/// put all of that behind the figure manifest.
#[test]
fn every_line_of_every_curve_names_a_figure_for_both_ends_and_its_worst_departure() {
    for entry in compute_all_curves() {
        let c = entry.curve;
        assert!(
            entry.traces.lines.len() > 1,
            "{}: a curve with one line is a series with a continuous axis; what makes it a curve \
             is the comparison between the lines at the same position",
            c.key
        );
        for line in &entry.traces.lines {
            assert!(
                line.points.len() > 2,
                "{}: the line {:?} has {} point(s), which is a pair of bars rather than a shape",
                c.key,
                line.label,
                line.points.len()
            );
            for (what, key) in [
                ("its first point", line.first_figure),
                ("its last point", line.last_figure),
                ("its worst departure", line.worst.figure),
            ] {
                assert!(
                    !key.is_empty(),
                    "{}: the line {:?} names no figure for {what}. Bind it as an ordinary figure \
                     in FIGURES first; all three are numbers a reader takes off the picture.",
                    c.key,
                    line.label
                );
            }
        }
    }
}

/// Every figure a line names exists, is on the vertical axis' unit, and reproduces the number.
///
/// In magnitude, for the reason a row, a fit and a position are all compared that way. A worst
/// departure is already unsigned — it is a distance from the reference, and a line above and a
/// line below by the same amount have departed equally — so for that one the two comparisons
/// coincide, which is what makes the figure quotable as "within three points" without a sign.
#[test]
fn a_line_reproduces_the_figures_it_names() {
    let figures = compute_all();
    for entry in compute_all_curves() {
        let c = entry.curve;
        for line in &entry.traces.lines {
            let first = line.points.first().expect("a line with points").y;
            let last = line.points.last().expect("a line with points").y;
            for (what, key, value) in [
                ("its first point", line.first_figure, first),
                ("its last point", line.last_figure, last),
                ("its worst departure", line.worst.figure, line.worst.gap),
            ] {
                let figure = figures
                    .iter()
                    .find(|f| f.figure.key == key)
                    .unwrap_or_else(|| {
                        panic!(
                            "{}: the line {:?} names {key} for {what}, which FIGURES does not \
                             carry",
                            c.key, line.label
                        )
                    });
                assert_eq!(
                    figure.figure.unit, entry.traces.y.unit,
                    "{}: the vertical axis is measured in {:?} and {key} in {:?}",
                    c.key, entry.traces.y.unit, figure.figure.unit
                );
                let drift = (value.abs() - figure.figure.pinned).abs();
                assert!(
                    drift <= figure.figure.tolerance,
                    "{}: the line {:?} has {what} at {value} and names {key}, pinned at {} \
                     \u{2014} {drift} apart in magnitude, tolerating {}. The two are meant to be \
                     one computation; if they have diverged, the curve is drawing something the \
                     corpus does not quote.",
                    c.key,
                    line.label,
                    figure.figure.pinned,
                    figure.figure.tolerance
                );
            }
        }
    }
}

/// The worst departure each line names is the worst one it actually has, and it is where it says.
///
/// The figure is a *maximum over the index*, which is the one number on a curve that no
/// coordinate states and so the one a registry entry could get wrong without any point moving.
/// Recomputed here from the drawn points rather than taken on trust from the owning crate.
#[test]
fn the_worst_departure_is_the_worst_one_on_the_drawn_line() {
    for entry in compute_all_curves() {
        let c = entry.curve;
        let Some(reference) = &entry.traces.reference else {
            continue;
        };
        for line in &entry.traces.lines {
            let (at, gap) = line
                .points
                .iter()
                .map(|p| (p.x, (p.y - reference.value).abs()))
                .fold((f64::NAN, f64::MIN), |worst, here| {
                    if here.1 > worst.1 {
                        here
                    } else {
                        worst
                    }
                });
            assert!(
                (line.worst.gap - gap).abs() < 1e-12,
                "{}: the line {:?} claims a worst departure of {} and the drawn points give {gap}",
                c.key,
                line.label,
                line.worst.gap
            );
            assert!(
                (line.worst.at - at).abs() < 1e-12,
                "{}: the line {:?} puts its worst departure at {} and the drawn points put it at \
                 {at}",
                c.key,
                line.label,
                line.worst.at
            );
        }
    }
}

/// Every curve is drawn on a linear index that holds every point, and a vertical axis that holds
/// the **reference** as well as every value.
///
/// The reference inside the frame is the whole of what makes the picture an argument: the lines
/// are not read for their level but for their distance from it, and a reference outside the drawn
/// domain is a comparison the reader has been asked to make and not shown.
#[test]
fn every_curve_frames_its_reference_as_well_as_its_lines() {
    for entry in compute_all_curves() {
        let c = entry.curve;
        for (what, axis) in [("x", &entry.traces.x), ("y", &entry.traces.y)] {
            assert!(
                !axis.log,
                "{}: the {what} axis is read for distances along it, which a log axis distorts",
                c.key
            );
            assert!(
                axis.min < axis.max,
                "{}: the {what} axis runs {} to {}",
                c.key,
                axis.min,
                axis.max
            );
        }
        if let Some(reference) = &entry.traces.reference {
            assert!(
                reference.value >= entry.traces.y.min && reference.value <= entry.traces.y.max,
                "{}: the reference sits at {} and the drawn range is {} to {}",
                c.key,
                reference.value,
                entry.traces.y.min,
                entry.traces.y.max
            );
        }
        for line in &entry.traces.lines {
            let mut previous = f64::MIN;
            for point in &line.points {
                assert!(
                    point.x > previous,
                    "{}: the line {:?} steps from {previous} to {} — a curve's index is ordered, \
                     and a chart that joined its points in any other order would be drawing a \
                     shape the data does not have",
                    c.key,
                    line.label,
                    point.x
                );
                previous = point.x;
                assert!(
                    point.x >= entry.traces.x.min && point.x <= entry.traces.x.max,
                    "{}: the line {:?} has a point at {}, outside the drawn {} to {}",
                    c.key,
                    line.label,
                    point.x,
                    entry.traces.x.min,
                    entry.traces.x.max
                );
                assert!(
                    point.y.is_finite()
                        && point.y >= entry.traces.y.min
                        && point.y <= entry.traces.y.max,
                    "{}: the line {:?} has a value of {}, outside the drawn {} to {}",
                    c.key,
                    line.label,
                    point.y,
                    entry.traces.y.min,
                    entry.traces.y.max
                );
            }
        }
    }
}

/// Every line of a curve runs over the same index, so the comparison between them is a comparison.
#[test]
fn every_line_of_a_curve_is_drawn_over_the_same_index() {
    for entry in compute_all_curves() {
        let c = entry.curve;
        let first = entry.traces.lines.first().expect("a curve with lines");
        let index: Vec<f64> = first.points.iter().map(|p| p.x).collect();
        for line in &entry.traces.lines {
            let here: Vec<f64> = line.points.iter().map(|p| p.x).collect();
            assert_eq!(
                here, index,
                "{}: the line {:?} runs over a different index from {:?}, and a curve exists to \
                 be read across its lines at one position",
                c.key, line.label, first.label
            );
        }
    }
}

/// Curve keys are unique, name their owner's directory, and collide with nothing already keyed.
#[test]
fn every_curve_key_is_unique_and_names_its_owner() {
    let taken: BTreeSet<&str> = FIGURES
        .iter()
        .map(|f| f.key)
        .chain(SERIES.iter().map(|s| s.key))
        .chain(SCATTERS.iter().map(|s| s.key))
        .chain(PLANES.iter().map(|p| p.key))
        .collect();
    let mut seen = BTreeSet::new();
    for c in CURVES {
        assert!(seen.insert(c.key), "{}: two curves share this key", c.key);
        assert!(
            !taken.contains(c.key),
            "{}: is also a figure, series, cloud or plane key, and a corpus node could not say \
             which it bound",
            c.key
        );
        let directory = c
            .owner
            .strip_prefix("crates/")
            .unwrap_or_else(|| panic!("{}: owner {:?} is not under crates/", c.key, c.owner));
        assert!(
            c.key.starts_with(&format!("{directory}/")),
            "{}: owned by {} and so should be keyed `{directory}/…`",
            c.key,
            c.owner
        );
        assert!(
            c.key
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '/'),
            "{}: keys are lower-case kebab so a corpus node can hold one without quoting",
            c.key
        );
        assert!(
            c.label.len() > 20,
            "{}: {:?} does not say what the curve is",
            c.key,
            c.label
        );
        assert!(
            !c.subject.is_empty() && c.subject == c.subject.to_lowercase(),
            "{}: the subject is what one step along the index is, singular and lower case",
            c.key
        );
    }
}

/// Every line and reference is labelled, distinctly, and writable unescaped.
#[test]
fn every_line_is_named_and_writable() {
    for entry in compute_all_curves() {
        let c = entry.curve;
        let mut seen = BTreeSet::new();
        for line in &entry.traces.lines {
            assert!(
                !line.label.is_empty(),
                "{}: a line has no label, and a curve is read by which line is which",
                c.key
            );
            assert!(
                seen.insert(line.label),
                "{}: two lines are labelled {:?}",
                c.key,
                line.label
            );
        }
        let strings = [c.key, c.owner, c.label, c.subject]
            .into_iter()
            .chain(entry.traces.lines.iter().map(|l| l.label))
            .chain(entry.traces.lines.iter().map(|l| l.first_figure))
            .chain(entry.traces.lines.iter().map(|l| l.last_figure))
            .chain(entry.traces.lines.iter().map(|l| l.worst.figure))
            .chain(entry.traces.reference.iter().map(|r| r.label))
            .chain([entry.traces.x.label, entry.traces.y.label]);
        for text in strings {
            assert!(
                !text.contains(['"', '\\']) && !text.chars().any(char::is_control),
                "{}: {text:?} carries a character the manifest writer cannot escape",
                c.key
            );
        }
    }
}

/// The endpoint rule as a spread has it: **every region of the census names a figure**.
///
/// A series binds its two extreme rows, a cloud its fit, a plane every coordinate, a curve each
/// line's two ends and its worst departure. None of those is available here — there is no
/// extreme worth quoting among six hundred anonymous districts, no fitted line, and a coordinate
/// nobody reads off. What a reader takes off this picture is *how many are in that part of it*,
/// and so that is what is pinned: a count per region, each an ordinary figure.
#[test]
fn every_region_of_every_spread_names_a_figure() {
    for entry in compute_all_spreads() {
        let s = entry.spread;
        assert!(
            !entry.regions.census.is_empty(),
            "{}: a spread with no census is a picture whose regions nothing stands behind",
            s.key
        );
        for tally in &entry.regions.census {
            assert!(
                !tally.figure.is_empty(),
                "{}: the region {:?} holds {} subject(s) and names no figure for it. Bind it as \
                 an ordinary figure in FIGURES first; a count off a spread is the only number a \
                 reader can quote from one.",
                s.key,
                tally.label,
                tally.subjects
            );
        }
    }
}

/// Every figure a census row names exists, is a count, and reproduces the region exactly.
///
/// Exactly, and not in magnitude the way a row, a fit, a position and a line are all compared: a
/// count is unsigned already and it is an integer, so there is no rounding to tolerate and no
/// direction to have dropped. A tolerance here would only be a place for a district to hide.
#[test]
fn a_region_reproduces_the_figure_it_names() {
    let figures = compute_all();
    for entry in compute_all_spreads() {
        let s = entry.spread;
        for tally in &entry.regions.census {
            let figure = figures
                .iter()
                .find(|f| f.figure.key == tally.figure)
                .unwrap_or_else(|| {
                    panic!(
                        "{}: the region {:?} names {}, which FIGURES does not carry",
                        s.key, tally.label, tally.figure
                    )
                });
            assert_eq!(
                figure.figure.unit,
                figures::Unit::Count,
                "{}: {} counts districts in {:?} and so is a count",
                s.key,
                tally.figure,
                tally.label
            );
            #[allow(clippy::cast_precision_loss)]
            let subjects = tally.subjects as f64;
            assert!(
                (subjects - figure.figure.pinned).abs() < 0.5,
                "{}: the region {:?} holds {} and names {}, pinned at {} \u{2014} the two are \
                 meant to be one computation, and a spread whose regions have drifted from the \
                 figures is a picture the corpus does not quote.",
                s.key,
                tally.label,
                tally.subjects,
                tally.figure,
                figure.figure.pinned
            );
        }
    }
}

/// Every panel's own population is a census row, and so is the whole picture's.
///
/// The failure this exists to stop is the cheap one: a panel drawing a hundred and two marks
/// under a caption saying a hundred and three. A panel is a sub-population and its size is the
/// first number a reader takes off it, so it has to be one of the counts the figure manifest
/// stands behind rather than whatever the loop happened to produce. The same for the total,
/// which is the denominator every other region is read against.
#[test]
fn every_panel_of_every_spread_and_the_whole_of_it_is_a_counted_region() {
    for entry in compute_all_spreads() {
        let s = entry.spread;
        let counted: BTreeSet<usize> = entry.regions.census.iter().map(|t| t.subjects).collect();
        for panel in &entry.regions.panels {
            assert!(
                counted.contains(&panel.marks.len()),
                "{}: the panel {:?} draws {} mark(s) and no region of the census counts that \
                 many, so its caption is a number nothing pins",
                s.key,
                panel.label,
                panel.marks.len()
            );
        }
        let drawn: usize = entry.regions.panels.iter().map(|p| p.marks.len()).sum();
        assert!(
            counted.contains(&drawn),
            "{}: {drawn} subject(s) are drawn and no region counts the whole of them \u{2014} \
             the total is the denominator every other region is read against",
            s.key
        );
    }
}

/// Every spread names the subjects it could not place, and draws none of them.
///
/// The second half of the binding rule, and the half about the denominator. A count is a count of
/// a population, and a population is what silently changes when a fixture is extended — so a
/// subject the computation could not reach is carried to the picture and named on it. The
/// crossing check is on the names as drawn, and it caught a real one: district names are not
/// unique, and the Buckeye Local the chained index misses is one of three. A note saying a name
/// is absent, under a cloud in which a reader can find that name, is worse than no note — so an
/// absence is named by something the marks do not answer to.
#[test]
fn every_spread_names_the_subjects_it_could_not_place() {
    for entry in compute_all_spreads() {
        let s = entry.spread;
        assert!(
            !entry.regions.unreached.is_empty(),
            "{}: nothing is unreached. If the identity now places its whole population, say so \
             by leaving this test failing until someone has checked it: an empty list is also \
             what a dropped `filter` looks like.",
            s.key
        );
        let drawn: BTreeSet<&str> = entry
            .regions
            .panels
            .iter()
            .flat_map(|p| p.marks.iter())
            .map(|m| m.label.as_str())
            .collect();
        for missing in &entry.regions.unreached {
            assert!(
                !missing.label.is_empty(),
                "{}: a subject is unreached and unnamed, which is the same as being dropped",
                s.key
            );
            assert!(
                !missing.why.is_empty(),
                "{}: {:?} is unreached for no stated reason, and the note on the picture is \
                 what makes the absence something a reader can act on",
                s.key,
                missing.label
            );
            assert!(
                !drawn.contains(missing.label.as_str()),
                "{}: {:?} is both drawn and unreached. Either the two lists were assembled \
                 from different populations, or \u{2014} district names not being unique \
                 \u{2014} the absence is named by something a reader can find in the cloud \
                 instead.",
                s.key,
                missing.label
            );
        }
    }
}

/// Every mark is named, classed, and inside the frame the whole spread shares.
///
/// The frame is computed across every panel together, which is the only thing that makes "the
/// cloud climbs across the boundary from one panel to the next" a statement about the data
/// rather than about five separately-scaled pictures. A mark outside it is a frame computed from
/// something other than these marks.
#[test]
fn every_mark_of_every_spread_is_named_classed_and_framed() {
    for entry in compute_all_spreads() {
        let s = entry.spread;
        let r = &entry.regions;
        assert!(!r.classes.is_empty(), "{}: a spread with no classes", s.key);
        assert!(
            r.classes.len() <= 3,
            "{}: {} classes, and the palette carries two hues and a neutral",
            s.key,
            r.classes.len()
        );
        assert!(!r.panels.is_empty(), "{}: a spread with no panels", s.key);
        for (what, axis) in [("x", &r.x), ("y", &r.y)] {
            assert!(
                !axis.log,
                "{}: the {what} axis is a signed logarithm already, which a log scale cannot \
                 place",
                s.key
            );
            assert!(
                axis.min < axis.max,
                "{}: the {what} axis runs {} to {}",
                s.key,
                axis.min,
                axis.max
            );
        }
        for panel in &r.panels {
            assert!(
                !panel.marks.is_empty(),
                "{}: the panel {:?} draws nothing",
                s.key,
                panel.label
            );
            for mark in &panel.marks {
                assert!(
                    !mark.label.is_empty(),
                    "{}: a mark in {:?} has no label, and a tooltip is the only way a reader \
                     asks which district that is",
                    s.key,
                    panel.label
                );
                assert!(
                    mark.class < r.classes.len(),
                    "{}: {:?} is class {} of {} \u{2014} it would arrive in the manifest, find \
                     no colour waiting for it, and be drawn as one of the others",
                    s.key,
                    mark.label,
                    mark.class,
                    r.classes.len()
                );
                for (what, axis, value) in [("x", &r.x, mark.x), ("y", &r.y, mark.y)] {
                    assert!(
                        value.is_finite() && value >= axis.min && value <= axis.max,
                        "{}: {:?} is at {what} = {value}, outside the drawn {} to {}",
                        s.key,
                        mark.label,
                        axis.min,
                        axis.max
                    );
                }
            }
        }
    }
}

/// Every boundary is a line the reader can actually see, drawn across the frame it belongs to.
///
/// A boundary is where a classification stops being defined, so a boundary that misses the frame
/// is a definition the picture has not shown — the reader is left to take the empty side of it
/// on trust, which is the one thing a spread exists not to ask of them. Checked on the segment
/// the manifest writes rather than on the slope, because that segment is what the web draws.
#[test]
fn every_boundary_crosses_the_frame_it_is_drawn_on() {
    for entry in compute_all_spreads() {
        let s = entry.spread;
        let r = &entry.regions;
        let mut seen = BTreeSet::new();
        for line in &r.boundaries {
            assert!(
                !line.label.is_empty(),
                "{}: a boundary with no label is a rule through a cloud, which reads as a claim",
                s.key
            );
            assert!(
                seen.insert(line.label),
                "{}: two boundaries are labelled {:?}",
                s.key,
                line.label
            );
            assert!(
                line.slope.is_finite() && line.intercept.is_finite(),
                "{}: the boundary {:?} is y = {}x + {}, which cannot be placed",
                s.key,
                line.label,
                line.slope,
                line.intercept
            );
            let ((_, low), (_, high)) = line.across(&r.x);
            let (low, high) = (low.min(high), low.max(high));
            assert!(
                high >= r.y.min && low <= r.y.max,
                "{}: the boundary {:?} runs from {low} to {high} across a frame drawn {} to {}, \
                 so none of it is on the picture",
                s.key,
                line.label,
                r.y.min,
                r.y.max
            );
        }
    }
}

/// Spread keys are unique, name their owner's directory, and collide with nothing already keyed.
#[test]
fn every_spread_key_is_unique_and_names_its_owner() {
    let taken: BTreeSet<&str> = FIGURES
        .iter()
        .map(|f| f.key)
        .chain(SERIES.iter().map(|s| s.key))
        .chain(SCATTERS.iter().map(|s| s.key))
        .chain(PLANES.iter().map(|p| p.key))
        .chain(CURVES.iter().map(|c| c.key))
        .collect();
    let mut seen = BTreeSet::new();
    for s in SPREADS {
        assert!(seen.insert(s.key), "{}: two spreads share this key", s.key);
        assert!(
            !taken.contains(s.key),
            "{}: is also a figure, series, cloud, plane or curve key, and a corpus node could \
             not say which it bound",
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
            "{}: {:?} does not say what the spread is",
            s.key,
            s.label
        );
        assert!(
            !s.subject.is_empty() && s.subject == s.subject.to_lowercase(),
            "{}: the subject is what one mark is, singular and lower case",
            s.key
        );
    }
}

/// Every panel, class and region of a spread is named, distinctly, and writable unescaped.
///
/// A panel label and a census label are the two captions a spread is read by, and the marks'
/// labels are district names out of a fixture — data rather than registry text, which is the
/// case the escaping check was written for.
#[test]
fn every_region_and_panel_of_a_spread_is_named_and_writable() {
    for entry in compute_all_spreads() {
        let s = entry.spread;
        let r = &entry.regions;
        let mut panels = BTreeSet::new();
        for panel in &r.panels {
            assert!(
                !panel.label.is_empty(),
                "{}: a panel has no caption, and a small multiple is read by which one it is",
                s.key
            );
            assert!(
                panels.insert(panel.label.as_str()),
                "{}: two panels are captioned {:?}",
                s.key,
                panel.label
            );
        }
        let mut regions = BTreeSet::new();
        for tally in &r.census {
            assert!(
                !tally.label.is_empty(),
                "{}: a region has no caption, and a count with no sentence attached is a number \
                 a reader cannot check",
                s.key
            );
            assert!(
                regions.insert(tally.label.as_str()),
                "{}: two regions are captioned {:?}",
                s.key,
                tally.label
            );
        }
        let mut classes = BTreeSet::new();
        for class in &r.classes {
            assert!(
                classes.insert(*class),
                "{}: two classes are named {class:?}, and the legend is what makes a hue mean \
                 anything",
                s.key
            );
        }
        let strings = [s.key, s.owner, s.label, s.subject]
            .into_iter()
            .chain(r.classes.iter().copied())
            .chain(r.boundaries.iter().map(|b| b.label))
            .chain(r.census.iter().map(|t| t.label.as_str()))
            .chain(r.census.iter().map(|t| t.figure))
            .chain(r.unreached.iter().map(|u| u.label.as_str()))
            .chain(r.unreached.iter().map(|u| u.why.as_str()))
            .chain(r.panels.iter().map(|p| p.label.as_str()))
            .chain(
                r.panels
                    .iter()
                    .flat_map(|p| p.marks.iter())
                    .map(|m| m.label.as_str()),
            )
            .chain([r.x.label, r.y.label]);
        for text in strings {
            assert!(
                !text.contains(['"', '\\']) && !text.chars().any(char::is_control),
                "{}: {text:?} carries a character the manifest writer cannot escape",
                s.key
            );
        }
    }
}
