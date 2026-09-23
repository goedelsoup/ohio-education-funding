//! Writing the two manifests.
//!
//! Hand-rolled for the reason the feed's serializer is: the workspace has no external
//! dependencies, deliberately, so that a committed result is reproducible years from now without
//! a dependency resolution succeeding first. Both documents are flatter than the feed — one array
//! of six-field objects, and one document holding two arrays, of short labelled columns and of
//! clouds whose every member is a number or a name — so they do not need [`bundle`'s `Obj`/`Arr`
//! machinery](../../bundle/index.html), and they do not need an escaper either.
//!
//! A cloud is the first thing either manifest writes that is not a `&'static str`: a point's
//! label is a district's name, read from a fixture. So the escaping check is no longer only a
//! guard against a typo in this repository, and `scatters` asserts on every one of them rather
//! than on the registry's own strings alone.
//!
//! It needs the *absence* of an escaper to be checked rather than assumed. Every string here is a
//! `&'static str` written in `lib.rs`, so a quote or a backslash in one is a compile-time-fixable
//! typo and not a runtime input — but "nobody would type that" is how a hand-rolled serializer
//! emits invalid JSON eventually. `every_string_is_safe_to_write_unescaped` in `tests/` is the
//! standing check, and a label that fails it should gain a word rather than an escaper.

use core::fmt::Write;

use crate::{
    compute_all, compute_all_bands, compute_all_curves, compute_all_planes, compute_all_scatters,
    compute_all_series, compute_all_spreads, Axis, Cloud, Fit, Positions, Ranges, Regions, Row,
    Traces, Unit, CONTRACT_VERSION, SERIES_CONTRACT_VERSION,
};

/// The characters that would need escaping, and therefore may not appear in a key or a label.
pub(crate) const UNWRITABLE: &[char] = &['"', '\\'];

/// Whether a string can be written into the manifest without escaping.
///
/// Control characters as well as the two obvious ones: JSON forbids a raw `\n` inside a string,
/// and a label written with a line continuation in the source is the way one would arrive.
#[must_use]
pub(crate) fn writable(text: &str) -> bool {
    !text.contains(UNWRITABLE) && !text.chars().any(char::is_control)
}

/// How a value is written, which is decided by its unit.
///
/// A count is written as an integer because it is one, and a reader diffing the artefact should
/// not have to decide whether `65.0` and `65` are the same figure. Everything else is written at
/// the shortest precision that round-trips, which is what Rust's `{:?}` for `f64` gives.
fn number(unit: Unit, value: f64) -> String {
    assert!(
        value.is_finite(),
        "a figure computed to {value}, which is not a number JSON can carry"
    );
    if unit == Unit::Count {
        assert!(
            value.fract() == 0.0,
            "a count computed to {value}, which is not whole"
        );
        format!("{}", value as i64)
    } else {
        format!("{value:?}")
    }
}

/// The manifest, as it is committed to `crates/figures.json`.
///
/// One member per line at both levels: this is a committed artefact and a figure that moved should
/// be one changed line, the same reason the feed's outer document is laid out that way.
#[must_use]
pub fn manifest() -> String {
    let computed = compute_all();
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"contract\": \"{CONTRACT_VERSION}\",");
    let _ = writeln!(out, "  \"figures\": [");
    for (at, entry) in computed.iter().enumerate() {
        let f = entry.figure;
        assert!(
            writable(f.key) && writable(f.owner) && writable(f.label),
            "{}: a key, owner or label carries a character this writer cannot escape",
            f.key
        );
        let comma = if at + 1 == computed.len() { "" } else { "," };
        let _ = writeln!(
            out,
            "    {{\"key\": \"{}\", \"owner\": \"{}\", \"unit\": \"{}\", \"value\": {}, \
             \"label\": \"{}\"}}{comma}",
            f.key,
            f.owner,
            f.unit.name(),
            number(f.unit, entry.value),
            f.label,
        );
    }
    out.push_str("  ]\n}\n");
    out
}

/// One row of a series, as a line of the document.
///
/// The optional members are written only when present, so a row that says nothing beyond its
/// label and value is the short line a reader expects, and the consumer reads their absence as
/// absence rather than as an empty string. `of` is written the way the row's own value is: a
/// denominator of a count is a count.
fn row(unit: Unit, r: &Row) -> String {
    assert!(
        writable(r.label)
            && r.hover.is_none_or(writable)
            && r.group.is_none_or(writable)
            && r.cites.is_none_or(writable)
            && r.marked.is_none_or(writable),
        "{:?}: a row label, hover, group, citation or mark carries a character this writer \
         cannot escape",
        r.label
    );
    let mut line = format!(
        "{{\"label\": \"{}\", \"value\": {}",
        r.label,
        number(unit, r.value)
    );
    if let Some(hover) = r.hover {
        let _ = write!(line, ", \"hover\": \"{hover}\"");
    }
    if let Some(group) = r.group {
        let _ = write!(line, ", \"group\": \"{group}\"");
    }
    if let Some(of) = r.of {
        let _ = write!(line, ", \"of\": {}", number(unit, of));
    }
    if let Some(cites) = r.cites {
        let _ = write!(line, ", \"cites\": \"{cites}\"");
    }
    if let Some(marked) = r.marked {
        let _ = write!(line, ", \"marked\": \"{marked}\"");
    }
    if let Some(figure) = r.figure {
        assert!(
            writable(figure),
            "{:?}: names a figure key this writer cannot escape",
            r.label
        );
        let _ = write!(line, ", \"figure\": \"{figure}\"");
    }
    line.push('}');
    line
}

/// The series manifest, as it is committed to `crates/series.json`.
///
/// One series per object and one row per line inside it, for the same reason [`manifest`] is one
/// figure per line: a bar that moved should be one changed line in the diff the gate prints.
#[must_use]
pub fn series_manifest() -> String {
    let computed = compute_all_series();
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"contract\": \"{SERIES_CONTRACT_VERSION}\",");
    let _ = writeln!(out, "  \"series\": [");
    for (at, entry) in computed.iter().enumerate() {
        let s = entry.series;
        assert!(
            writable(s.key) && writable(s.owner) && writable(s.axis) && writable(s.label),
            "{}: a key, owner, axis or label carries a character this writer cannot escape",
            s.key
        );
        let _ = writeln!(
            out,
            "    {{\"key\": \"{}\", \"owner\": \"{}\", \"unit\": \"{}\", \"axis\": \"{}\", \
             \"label\": \"{}\", \"rows\": [",
            s.key,
            s.owner,
            s.unit.name(),
            s.axis,
            s.label,
        );
        for (row_at, r) in entry.rows.iter().enumerate() {
            let comma = if row_at + 1 == entry.rows.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(out, "      {}{comma}", row(s.unit, r));
        }
        let comma = if at + 1 == computed.len() { "" } else { "," };
        let _ = writeln!(out, "    ]}}{comma}");
    }
    out.push_str("  ],\n");
    scatters(&mut out);
    out.push_str(",\n");
    planes(&mut out);
    out.push_str(",\n");
    curves(&mut out);
    out.push_str(",\n");
    spreads(&mut out);
    out.push_str(",\n");
    bands(&mut out);
    out.push_str("}\n");
    out
}

/// A number on a continuous axis, at the shortest precision that round-trips.
///
/// No [`Unit`] here, and so no integer case: a cloud's axes are dollars and pupils at once and
/// neither is the other's. `{:?}` is what the figure manifest already writes everything that is
/// not a count as.
fn coordinate(value: f64) -> String {
    assert!(
        value.is_finite(),
        "a cloud carries {value}, which is not a number JSON can carry"
    );
    format!("{value:?}")
}

/// One axis, as a member of the object that holds it.
fn axis(name: &str, a: &Axis) -> String {
    assert!(
        writable(a.label),
        "{:?}: an axis label carries a character this writer cannot escape",
        a.label
    );
    format!(
        "\"{name}\": {{\"label\": \"{}\", \"unit\": \"{}\", \"min\": {}, \"max\": {}, \
         \"log\": {}}}",
        a.label,
        a.unit.name(),
        coordinate(a.min),
        coordinate(a.max),
        a.log,
    )
}

/// One fitted line: its two ends, its two numbers, and the two figures those are pinned by.
fn fit(f: &Fit) -> String {
    assert!(
        writable(f.slope_figure) && writable(f.r_squared_figure),
        "a fit names a figure key this writer cannot escape"
    );
    format!(
        "\"fit\": {{\"from\": [{}, {}], \"to\": [{}, {}], \"slope\": {}, \"rSquared\": {}, \
         \"slopeFigure\": \"{}\", \"rSquaredFigure\": \"{}\"}}",
        coordinate(f.from.0),
        coordinate(f.from.1),
        coordinate(f.to.0),
        coordinate(f.to.1),
        coordinate(f.slope),
        coordinate(f.r_squared),
        f.slope_figure,
        f.r_squared_figure,
    )
}

/// The clouds, as the second array of the series manifest.
///
/// One point per line, like every other row this writer emits: six hundred districts is a long
/// diff, and a district whose wealth moved should be one changed line of it rather than a
/// re-flowed paragraph.
fn scatters(out: &mut String) {
    let computed = compute_all_scatters();
    let _ = writeln!(out, "  \"scatters\": [");
    for (at, entry) in computed.iter().enumerate() {
        let s = entry.scatter;
        assert!(
            writable(s.key) && writable(s.owner) && writable(s.label) && writable(s.subject),
            "{}: a key, owner, label or subject carries a character this writer cannot escape",
            s.key
        );
        let Cloud { x, points, panels } = &entry.cloud;
        let _ = writeln!(
            out,
            "    {{\"key\": \"{}\", \"owner\": \"{}\", \"subject\": \"{}\", \"label\": \"{}\", {},",
            s.key,
            s.owner,
            s.subject,
            s.label,
            axis("x", x),
        );
        let _ = writeln!(out, "     \"panels\": [");
        for (panel_at, panel) in panels.iter().enumerate() {
            assert!(
                writable(panel.label),
                "{}: a panel label carries a character this writer cannot escape",
                s.key
            );
            let comma = if panel_at + 1 == panels.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(
                out,
                "       {{\"label\": \"{}\", {}, {}}}{comma}",
                panel.label,
                axis("y", &panel.y),
                fit(&panel.fit),
            );
        }
        let _ = writeln!(out, "     ],");
        let _ = writeln!(out, "     \"points\": [");
        for (point_at, point) in points.iter().enumerate() {
            assert!(
                writable(&point.label),
                "{}: the point {:?} carries a character this writer cannot escape",
                s.key,
                point.label
            );
            assert_eq!(
                point.ys.len(),
                panels.len(),
                "{}: the point {:?} carries {} value(s) under {} panel(s)",
                s.key,
                point.label,
                point.ys.len(),
                panels.len()
            );
            let ys: Vec<String> = point.ys.iter().map(|y| coordinate(*y)).collect();
            let comma = if point_at + 1 == points.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(
                out,
                "       {{\"label\": \"{}\", \"x\": {}, \"ys\": [{}]}}{comma}",
                point.label,
                coordinate(point.x),
                ys.join(", "),
            );
        }
        let comma = if at + 1 == computed.len() { "" } else { "," };
        let _ = writeln!(out, "     ]}}{comma}");
    }
    out.push_str("  ]");
}

/// The third array: every plane, its two axes, and its positions with both figure keys.
///
/// Every position writes both of its figure keys, without the `Option` the row writer carries:
/// a plane's rule is that all of them are bound, so a missing one is a bug rather than a shape
/// the document is allowed to take.
fn planes(out: &mut String) {
    let computed = compute_all_planes();
    let _ = writeln!(out, "  \"planes\": [");
    for (at, entry) in computed.iter().enumerate() {
        let p = entry.plane;
        assert!(
            writable(p.key) && writable(p.owner) && writable(p.label) && writable(p.subject),
            "{}: a key, owner, label or subject carries a character this writer cannot escape",
            p.key
        );
        let Positions { x, y, points } = &entry.positions;
        let _ = writeln!(
            out,
            "    {{\"key\": \"{}\", \"owner\": \"{}\", \"subject\": \"{}\", \"label\": \"{}\", \
             {}, {},",
            p.key,
            p.owner,
            p.subject,
            p.label,
            axis("x", x),
            axis("y", y),
        );
        let _ = writeln!(out, "     \"points\": [");
        for (point_at, point) in points.iter().enumerate() {
            assert!(
                writable(&point.label) && writable(point.x_figure) && writable(point.y_figure),
                "{}: the position {:?} carries a character this writer cannot escape",
                p.key,
                point.label
            );
            let comma = if point_at + 1 == points.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(
                out,
                "       {{\"label\": \"{}\", \"x\": {}, \"y\": {}, \"xFigure\": \"{}\", \
                 \"yFigure\": \"{}\"}}{comma}",
                point.label,
                coordinate(point.x),
                coordinate(point.y),
                point.x_figure,
                point.y_figure,
            );
        }
        let comma = if at + 1 == computed.len() { "" } else { "," };
        let _ = writeln!(out, "     ]}}{comma}");
    }
    out.push_str("  ]");
}

/// The fourth array: every curve, its frame, its reference, and its lines with their three keys.
///
/// A line writes all three of its figure keys unconditionally, on the same rule the plane writer
/// states: a curve's binding rule is that both ends and the worst departure are pinned, so a
/// missing one is a bug rather than a shape the document may take. The **reference** carries no
/// key, and that absence is the one thing here a reader has to be told rather than infer — see
/// the `CURVES` docs for why a definition is not a figure.
fn curves(out: &mut String) {
    let computed = compute_all_curves();
    let _ = writeln!(out, "  \"curves\": [");
    for (at, entry) in computed.iter().enumerate() {
        let c = entry.curve;
        assert!(
            writable(c.key) && writable(c.owner) && writable(c.label) && writable(c.subject),
            "{}: a key, owner, label or subject carries a character this writer cannot escape",
            c.key
        );
        let Traces {
            x,
            y,
            reference,
            lines,
        } = &entry.traces;
        let _ = writeln!(
            out,
            "    {{\"key\": \"{}\", \"owner\": \"{}\", \"subject\": \"{}\", \"label\": \"{}\", \
             {}, {},",
            c.key,
            c.owner,
            c.subject,
            c.label,
            axis("x", x),
            axis("y", y),
        );
        if let Some(reference) = reference {
            assert!(
                writable(reference.label),
                "{}: a reference label carries a character this writer cannot escape",
                c.key
            );
            let _ = writeln!(
                out,
                "     \"reference\": {{\"label\": \"{}\", \"value\": {}}},",
                reference.label,
                coordinate(reference.value),
            );
        }
        let _ = writeln!(out, "     \"lines\": [");
        for (line_at, line) in lines.iter().enumerate() {
            assert!(
                writable(line.label)
                    && writable(line.first_figure)
                    && writable(line.last_figure)
                    && writable(line.worst.figure),
                "{}: the line {:?} carries a character this writer cannot escape",
                c.key,
                line.label
            );
            let _ = writeln!(
                out,
                "       {{\"label\": \"{}\", \"firstFigure\": \"{}\", \"lastFigure\": \"{}\",",
                line.label, line.first_figure, line.last_figure,
            );
            let _ = writeln!(
                out,
                "        \"worst\": {{\"at\": {}, \"gap\": {}, \"figure\": \"{}\"}},",
                coordinate(line.worst.at),
                coordinate(line.worst.gap),
                line.worst.figure,
            );
            let _ = writeln!(out, "        \"points\": [");
            for (point_at, point) in line.points.iter().enumerate() {
                let comma = if point_at + 1 == line.points.len() {
                    ""
                } else {
                    ","
                };
                let _ = writeln!(
                    out,
                    "          {{\"x\": {}, \"y\": {}}}{comma}",
                    coordinate(point.x),
                    coordinate(point.y),
                );
            }
            let comma = if line_at + 1 == lines.len() { "" } else { "," };
            let _ = writeln!(out, "        ]}}{comma}");
        }
        let comma = if at + 1 == computed.len() { "" } else { "," };
        let _ = writeln!(out, "     ]}}{comma}");
    }
    out.push_str("  ]");
}

/// The fifth array: every spread, its frame, its classes, its boundaries, its panels, and the
/// census that stands in for the endpoint rule.
///
/// A boundary is written as the **segment it makes across this frame** rather than as the slope
/// and intercept it was declared by, because the consumer draws a line and evaluating the same
/// arithmetic twice in two languages is how the two come to disagree about where the line goes.
///
/// Every [`Tally`](super::Tally) writes its figure key unconditionally, on the rule the plane and
/// the curve writers already state: the census *is* this shape's binding rule, so an unpinned
/// region is a bug rather than a shape the document may take. `unreached` carries no key and
/// cannot — it is a list of subjects the computation could not place, and there is no number in
/// it to pin.
fn spreads(out: &mut String) {
    let computed = compute_all_spreads();
    let _ = writeln!(out, "  \"spreads\": [");
    for (at, entry) in computed.iter().enumerate() {
        let s = entry.spread;
        assert!(
            writable(s.key) && writable(s.owner) && writable(s.label) && writable(s.subject),
            "{}: a key, owner, label or subject carries a character this writer cannot escape",
            s.key
        );
        let Regions {
            x,
            y,
            classes,
            boundaries,
            panels,
            census,
            unreached,
        } = &entry.regions;
        let _ = writeln!(
            out,
            "    {{\"key\": \"{}\", \"owner\": \"{}\", \"subject\": \"{}\", \"label\": \"{}\", \
             {}, {},",
            s.key,
            s.owner,
            s.subject,
            s.label,
            axis("x", x),
            axis("y", y),
        );
        assert!(
            classes.iter().all(|c| writable(c)),
            "{}: a class name carries a character this writer cannot escape",
            s.key
        );
        let named: Vec<String> = classes.iter().map(|c| format!("\"{c}\"")).collect();
        let _ = writeln!(out, "     \"classes\": [{}],", named.join(", "));
        let _ = writeln!(out, "     \"boundaries\": [");
        for (line_at, line) in boundaries.iter().enumerate() {
            assert!(
                writable(line.label),
                "{}: a boundary label carries a character this writer cannot escape",
                s.key
            );
            let (from, to) = line.across(x);
            let comma = if line_at + 1 == boundaries.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(
                out,
                "       {{\"label\": \"{}\", \"from\": [{}, {}], \"to\": [{}, {}]}}{comma}",
                line.label,
                coordinate(from.0),
                coordinate(from.1),
                coordinate(to.0),
                coordinate(to.1),
            );
        }
        let _ = writeln!(out, "     ],");
        let _ = writeln!(out, "     \"census\": [");
        for (row_at, tally) in census.iter().enumerate() {
            assert!(
                writable(&tally.label) && writable(tally.figure),
                "{}: the census row {:?} carries a character this writer cannot escape",
                s.key,
                tally.label
            );
            let comma = if row_at + 1 == census.len() { "" } else { "," };
            let _ = writeln!(
                out,
                "       {{\"label\": \"{}\", \"subjects\": {}, \"figure\": \"{}\"}}{comma}",
                tally.label, tally.subjects, tally.figure,
            );
        }
        let _ = writeln!(out, "     ],");
        let _ = writeln!(out, "     \"unreached\": [");
        for (row_at, missing) in unreached.iter().enumerate() {
            assert!(
                writable(&missing.label) && writable(&missing.why),
                "{}: the unreached subject {:?} carries a character this writer cannot escape",
                s.key,
                missing.label
            );
            let comma = if row_at + 1 == unreached.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(
                out,
                "       {{\"label\": \"{}\", \"why\": \"{}\"}}{comma}",
                missing.label, missing.why,
            );
        }
        let _ = writeln!(out, "     ],");
        let _ = writeln!(out, "     \"panels\": [");
        for (panel_at, panel) in panels.iter().enumerate() {
            assert!(
                writable(&panel.label),
                "{}: a panel label carries a character this writer cannot escape",
                s.key
            );
            let _ = writeln!(out, "       {{\"label\": \"{}\", \"marks\": [", panel.label);
            for (mark_at, mark) in panel.marks.iter().enumerate() {
                assert!(
                    writable(&mark.label),
                    "{}: the mark {:?} carries a character this writer cannot escape",
                    s.key,
                    mark.label
                );
                let comma = if mark_at + 1 == panel.marks.len() {
                    ""
                } else {
                    ","
                };
                let _ = writeln!(
                    out,
                    "         {{\"label\": \"{}\", \"x\": {}, \"y\": {}, \"class\": {}}}{comma}",
                    mark.label,
                    coordinate(mark.x),
                    coordinate(mark.y),
                    mark.class,
                );
            }
            let comma = if panel_at + 1 == panels.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(out, "       ]}}{comma}");
        }
        let comma = if at + 1 == computed.len() { "" } else { "," };
        let _ = writeln!(out, "     ]}}{comma}");
    }
    out.push_str("  ]");
}

/// The band charts, which is the second array to write an `unreached` block and the first whose
/// every label is computed rather than written in `lib.rs`.
///
/// A span's label and hover are `format!`ed out of the fixtures, so the writability check is on
/// the same footing `scatters` put it on: it is guarding a real input, not a typo. The escape
/// hatch if one ever fails is the same one — the composing code gains a word.
fn bands(out: &mut String) {
    let computed = compute_all_bands();
    let _ = writeln!(out, "  \"bands\": [");
    for (at, entry) in computed.iter().enumerate() {
        let b = entry.band;
        assert!(
            writable(b.key)
                && writable(b.owner)
                && writable(b.label)
                && writable(b.subject)
                && b.ends.iter().all(|end| writable(end)),
            "{}: a key, owner, label, subject or end name carries a character this writer cannot \
             escape",
            b.key
        );
        let Ranges {
            measure,
            spans,
            markers,
            whole,
            unreached,
        } = &entry.ranges;
        let _ = writeln!(
            out,
            "    {{\"key\": \"{}\", \"owner\": \"{}\", \"subject\": \"{}\", \"label\": \"{}\", \
             \"ends\": [\"{}\", \"{}\"], {},",
            b.key,
            b.owner,
            b.subject,
            b.label,
            b.ends[0],
            b.ends[1],
            axis("measure", measure),
        );
        let _ = writeln!(out, "     \"spans\": [");
        for (span_at, span) in spans.iter().enumerate() {
            assert!(
                writable(&span.label) && writable(&span.hover),
                "{}: the span {:?} carries a character this writer cannot escape",
                b.key,
                span.label
            );
            let comma = if span_at + 1 == spans.len() { "" } else { "," };
            let _ = write!(
                out,
                "       {{\"label\": \"{}\", \"low\": {}, \"high\": {}, \"hover\": \"{}\"",
                span.label,
                number(measure.unit, span.low),
                number(measure.unit, span.high),
                span.hover,
            );
            for (name, key) in [
                ("lowFigure", span.low_figure),
                ("highFigure", span.high_figure),
            ] {
                if let Some(key) = key {
                    assert!(
                        writable(key),
                        "{}: the span {:?} names a figure key this writer cannot escape",
                        b.key,
                        span.label
                    );
                    let _ = write!(out, ", \"{name}\": \"{key}\"");
                }
            }
            let _ = writeln!(out, "}}{comma}");
        }
        let _ = writeln!(out, "     ],");
        let _ = writeln!(out, "     \"markers\": [");
        for (marker_at, marker) in markers.iter().enumerate() {
            assert!(
                writable(&marker.label) && writable(marker.figure),
                "{}: the marker {:?} carries a character this writer cannot escape",
                b.key,
                marker.label
            );
            let comma = if marker_at + 1 == markers.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(
                out,
                "       {{\"label\": \"{}\", \"value\": {}, \"at\": {}, \"figure\": \
                 \"{}\"}}{comma}",
                marker.label,
                coordinate(marker.value),
                coordinate(marker.at),
                marker.figure,
            );
        }
        let _ = writeln!(out, "     ],");
        assert!(
            writable(&whole.label) && writable(whole.figure),
            "{}: the aggregate carries a character this writer cannot escape",
            b.key
        );
        let _ = writeln!(
            out,
            "     \"whole\": {{\"label\": \"{}\", \"value\": {}, \"figure\": \"{}\"}},",
            whole.label,
            coordinate(whole.value),
            whole.figure,
        );
        let _ = writeln!(out, "     \"unreached\": [");
        for (row_at, missing) in unreached.iter().enumerate() {
            assert!(
                writable(&missing.label) && writable(&missing.why),
                "{}: the unreached floor {:?} carries a character this writer cannot escape",
                b.key,
                missing.label
            );
            let comma = if row_at + 1 == unreached.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(
                out,
                "       {{\"label\": \"{}\", \"why\": \"{}\"}}{comma}",
                missing.label, missing.why,
            );
        }
        let comma = if at + 1 == computed.len() { "" } else { "," };
        let _ = writeln!(out, "     ]}}{comma}");
    }
    out.push_str("  ]\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_writes_its_optional_members_only_when_it_has_them() {
        let bare = Row {
            label: "Industrial",
            value: -0.0321,
            hover: None,
            group: None,
            of: None,
            cites: None,
            marked: None,
            figure: None,
        };
        assert_eq!(
            row(Unit::Ratio, &bare),
            "{\"label\": \"Industrial\", \"value\": -0.0321}"
        );
        let full = Row {
            label: "Mineral",
            value: 12.0,
            hover: Some("Mineral, which is mostly gas wells"),
            group: None,
            of: None,
            cites: None,
            marked: None,
            figure: Some("dispersion/a-key"),
        };
        assert_eq!(
            row(Unit::Count, &full),
            "{\"label\": \"Mineral\", \"value\": 12, \"hover\": \"Mineral, which is mostly gas \
             wells\", \"figure\": \"dispersion/a-key\"}"
        );
        // The members a census row carries, in the order the writer emits them.
        let census = Row {
            label: "Mile base over rider base",
            value: 350.0,
            hover: None,
            group: Some("Transportation"),
            of: Some(604.0),
            cites: Some("R.C. 3317.0212(C)"),
            marked: None,
            figure: None,
        };
        assert_eq!(
            row(Unit::Count, &census),
            "{\"label\": \"Mile base over rider base\", \"value\": 350, \"group\": \
             \"Transportation\", \"of\": 604, \"cites\": \"R.C. 3317.0212(C)\"}"
        );
        let nothing = Row {
            label: "One leadership support staff",
            value: 0.0,
            hover: None,
            group: Some("Base cost"),
            of: Some(609.0),
            cites: Some("R.C. 3317.011(F)(6)(c)"),
            marked: Some("cannot bind"),
            figure: Some("project/a-key"),
        };
        assert!(row(Unit::Count, &nothing).contains("\"marked\": \"cannot bind\""));
    }

    #[test]
    fn a_count_is_written_as_an_integer_and_everything_else_round_trips() {
        assert_eq!(number(Unit::Count, 65.0), "65");
        assert_eq!(number(Unit::Dollars, -44.62), "-44.62");
        assert_eq!(number(Unit::Share, 0.1387), "0.1387");
        // The one that matters: a value the manifest must not round, because the corpus quotes it
        // to the cent and the check compares against what is written here.
        assert_eq!(number(Unit::Dollars, 392_151_306.63), "392151306.63");
    }

    #[test]
    fn a_string_needing_an_escape_is_not_writable() {
        assert!(writable("districts the plan pays more"));
        assert!(!writable("a label with a \" in it"));
        assert!(!writable("a label with a \\ in it"));
        assert!(!writable("a label with a\nnewline"));
    }
}
