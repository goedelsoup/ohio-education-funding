//! The scholarship annual report's five participation-by-year charts, read off their geometry.
//!
//! The 2025 report draws one chart per programme of participation by fiscal year, from each
//! programme's first year to FY2025, and prints no value on any of them. The text layer carries the
//! axis labels and the year row and nothing else, which is why [`super::jpsn`] left them alone:
//! "recovering them means reading the chart's geometry, which is a different instrument". This is
//! that instrument. The charts are vector drawings, not images, so `pdftocairo -svg` gives every
//! mark's position in page points, and the gridlines it gives in the same frame are the scale.
//!
//! # How a value is read
//!
//! Three things come off one page and each is checked against the others before anything is
//! written:
//!
//! - **The tick values and the year row** from the text layer, starting at the chart's title.
//! - **The gridlines** from the drawing: horizontal stroked segments, grouped by their shared
//!   extent so that two charts on one page do not mix. The group that has exactly as many lines
//!   as there are tick labels, and that spans the series, is the axis.
//! - **The series**: an area's upper edge, a line's vertices, or the tops of a set of bars,
//!   with exactly as many points as the year row has years.
//!
//! The ticks are paired with the gridlines top to bottom and fitted by least squares, and a value
//! is a mark's height on that fit. A label that does not sit on its line fails the read rather
//! than bending the scale.
//!
//! # What the charts measure is not what all of them say
//!
//! Two are titled "participants" and plot the FY2013 historical file's **applications** column for
//! every year that file covers, and one is titled "applications" and plots what its own report
//! calls students. The fixture carries the title's word in `measure` and nothing else, because the
//! extractor can only know what the chart claims. What it actually drew is a finding about the
//! fixture, and `crates/project/tests/` is where it is pinned.
//!
//! # Precision
//!
//! `pupils_per_point` is the fitted scale, so a reader can turn a drawing tolerance in points into
//! one in pupils for that chart. The tolerance itself is not decided here. It is a claim about how
//! closely the department's charting tool places a mark, and the evidence for it is the overlap
//! with figures published elsewhere, which this crate does not read.

/// One row per programme per fiscal year the programme's chart draws.
pub const SCHOLARSHIP_CHART_HEADER: &[&str] = &[
    "program",
    "fiscal_year",
    "measure",
    "value",
    "pupils_per_point",
];

/// How a chart draws its series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// A filled area. The value is the upper edge at each year.
    Area,
    /// A polyline through one vertex per year.
    Line,
    /// One filled rectangle per year. The value is its top.
    Bars,
}

/// One chart: which programme it is, the title it is found by, and how it draws.
#[derive(Debug, Clone, Copy)]
pub struct Chart {
    /// The fixture's slug for the programme, as `scholarship-programs.csv` spells it.
    pub program: &'static str,
    /// A run of the chart's title, as the text layer prints it. Unique within the report.
    pub title: &'static str,
    /// How the series is drawn.
    pub mark: Mark,
}

/// The five charts, in the order the report presents them.
pub const CHARTS: &[Chart] = &[
    Chart {
        program: "traditional-edchoice",
        title: "TOTAL PARTICIPANTS IN TRADITIONAL EDCHOICE",
        mark: Mark::Area,
    },
    Chart {
        program: "edchoice-expansion",
        title: "TOTAL EXPANSION PARTICIPANTS",
        mark: Mark::Line,
    },
    Chart {
        program: "cleveland",
        title: "TOTAL PARTICIPANTS IN CLEVELAND SCHOLARSHIP",
        mark: Mark::Area,
    },
    Chart {
        program: "autism",
        title: "TOTAL PARTICIPANTS IN AUTISM SCHOLARSHIP",
        mark: Mark::Line,
    },
    Chart {
        program: "jon-peterson",
        title: "TOTAL JPSN APPLICATIONS",
        mark: Mark::Bars,
    },
];

/// How far, in points, a tick label's line may sit from the fitted scale before the read fails.
///
/// The five charts fit to within 0.06 points. A quarter of a point is four times that and still a
/// fifth of the narrowest gap between two gridlines on any of them.
const GRID_RESIDUAL: f64 = 0.25;

/// One `<path>` element, in page coordinates.
#[derive(Debug, Clone, PartialEq)]
struct Path {
    /// The `fill` attribute, `None` where it is `none` or absent.
    fill: Option<String>,
    /// Whether the element has a `stroke`.
    stroked: bool,
    /// Each subpath's vertices. A curve contributes its end point only.
    subpaths: Vec<Vec<(f64, f64)>>,
}

/// The value of `name="..."` inside one element's text.
fn attribute<'a>(element: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!(" {name}=\"");
    let at = element.find(&marker)? + marker.len();
    let rest = &element[at..];
    Some(&rest[..rest.find('"')?])
}

/// The six numbers of a `matrix(a, b, c, d, e, f)` transform, or the identity.
fn matrix(element: &str) -> Result<[f64; 6], String> {
    let Some(transform) = attribute(element, "transform") else {
        return Ok([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    };
    let inner = transform
        .strip_prefix("matrix(")
        .and_then(|rest| rest.strip_suffix(')'))
        .ok_or_else(|| format!("a transform that is not a matrix: {transform}"))?;
    let numbers: Vec<f64> = inner
        .split(',')
        .map(|n| n.trim().parse::<f64>())
        .collect::<Result<_, _>>()
        .map_err(|cause| format!("an unreadable transform {transform}: {cause}"))?;
    numbers
        .try_into()
        .map_err(|_| format!("a matrix without six numbers: {transform}"))
}

/// Every `<path>` element of `svg`, with its vertices carried into page coordinates.
///
/// `pdftocairo` writes only `M`, `L`, `C` and `Z`, in absolute coordinates, so those are all this
/// reads. Anything else is an error rather than a skipped command, because a skipped command would
/// shift every vertex after it.
fn paths(svg: &str) -> Result<Vec<Path>, String> {
    let mut out = Vec::new();
    for piece in svg.split("<path ").skip(1) {
        // Every attribute is found by its leading space, the first one included.
        let element = format!(
            " {}",
            &piece[..piece.find("/>").ok_or("a <path> that does not close")?]
        );
        let element = element.as_str();
        let Some(d) = attribute(element, "d") else {
            continue;
        };
        let [a, b, c, dd, e, f] = matrix(element)?;
        let place = |x: f64, y: f64| (a * x + c * y + e, b * x + dd * y + f);
        let tokens: Vec<&str> = d.split_whitespace().collect();
        let number = |i: usize| -> Result<f64, String> {
            tokens
                .get(i)
                .ok_or_else(|| format!("a path that ends mid-command: {d}"))?
                .parse::<f64>()
                .map_err(|cause| format!("an unreadable coordinate in {d}: {cause}"))
        };
        let mut subpaths: Vec<Vec<(f64, f64)>> = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "M" => {
                    subpaths.push(vec![place(number(i + 1)?, number(i + 2)?)]);
                    i += 3;
                }
                "L" => {
                    let point = place(number(i + 1)?, number(i + 2)?);
                    subpaths
                        .last_mut()
                        .ok_or("a line before any move")?
                        .push(point);
                    i += 3;
                }
                "C" => {
                    let point = place(number(i + 5)?, number(i + 6)?);
                    subpaths
                        .last_mut()
                        .ok_or("a curve before any move")?
                        .push(point);
                    i += 7;
                }
                "Z" => i += 1,
                other => return Err(format!("a path command this reader does not know: {other}")),
            }
        }
        subpaths.retain(|subpath| subpath.len() > 1);
        out.push(Path {
            fill: attribute(element, "fill")
                .filter(|fill| *fill != "none")
                .map(str::to_string),
            stroked: attribute(element, "stroke").is_some_and(|stroke| stroke != "none"),
            subpaths,
        });
    }
    Ok(out)
}

/// The chart's tick values, top to bottom, and its year row, from the page's text layer.
///
/// Read from the first line holding `title` onwards: every line that is a lone number is a tick,
/// and the first line that is nothing but years ends the axis. Lines that are neither — the
/// title's own wrapped second line — are passed over.
fn axis(text: &str, title: &str) -> Result<(Vec<f64>, Vec<u16>), String> {
    let mut lines = text.lines().skip_while(|line| !line.contains(title));
    if lines.next().is_none() {
        return Err(format!("no chart titled `{title}` on the page"));
    }
    let mut ticks = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit() || c == ',') {
            let tick = trimmed
                .replace(',', "")
                .parse::<f64>()
                .map_err(|cause| format!("an unreadable tick `{trimmed}`: {cause}"))?;
            ticks.push(tick);
            continue;
        }
        let years: Vec<u16> = trimmed
            .split_whitespace()
            .map_while(|token| {
                token
                    .parse::<u16>()
                    .ok()
                    .filter(|y| (1990..2100).contains(y))
            })
            .collect();
        if years.len() > 1 && years.len() == trimmed.split_whitespace().count() {
            if ticks.len() < 2 {
                return Err(format!("`{title}` has {} tick labels", ticks.len()));
            }
            if ticks.windows(2).any(|pair| pair[0] <= pair[1]) {
                return Err(format!("`{title}`'s ticks do not fall: {ticks:?}"));
            }
            return Ok((ticks, years));
        }
    }
    Err(format!("`{title}` has no year row"))
}

/// The page y of each year's mark, left to right, for a series of `count` points.
fn series(paths: &[Path], mark: Mark, count: usize) -> Result<Vec<(f64, f64)>, String> {
    let increasing = |points: &[(f64, f64)]| points.windows(2).all(|pair| pair[0].0 < pair[1].0);
    let mut found: Vec<Vec<(f64, f64)>> = match mark {
        // The upper edge, then two corners on the baseline, then the close.
        Mark::Area => paths
            .iter()
            .filter(|path| path.fill.is_some())
            .flat_map(|path| &path.subpaths)
            .filter(|subpath| subpath.len() == count + 2)
            .map(|subpath| subpath[..count].to_vec())
            .filter(|edge| increasing(edge))
            .collect(),
        Mark::Line => paths
            .iter()
            .filter(|path| path.fill.is_none() && path.stroked)
            .flat_map(|path| &path.subpaths)
            .filter(|subpath| subpath.len() == count && increasing(subpath))
            .cloned()
            .collect(),
        // Rectangles grouped by fill, and the one colour with a bar for every year.
        Mark::Bars => {
            let mut by_fill: Vec<(&str, Vec<(f64, f64)>)> = Vec::new();
            for path in paths {
                let Some(fill) = path.fill.as_deref() else {
                    continue;
                };
                for subpath in &path.subpaths {
                    let xs: Vec<f64> = subpath.iter().map(|p| p.0).collect();
                    let ys: Vec<f64> = subpath.iter().map(|p| p.1).collect();
                    let left = xs.iter().copied().fold(f64::MAX, f64::min);
                    let right = xs.iter().copied().fold(f64::MIN, f64::max);
                    let top = ys.iter().copied().fold(f64::MAX, f64::min);
                    let bottom = ys.iter().copied().fold(f64::MIN, f64::max);
                    let corners = subpath
                        .iter()
                        .all(|&(x, y)| (x == left || x == right) && (y == top || y == bottom));
                    if subpath.len() >= 4 && corners && right > left {
                        let centre = f64::midpoint(left, right);
                        match by_fill.iter_mut().find(|(f, _)| *f == fill) {
                            Some((_, bars)) => bars.push((centre, top)),
                            None => by_fill.push((fill, vec![(centre, top)])),
                        }
                    }
                }
            }
            by_fill
                .into_iter()
                .map(|(_, mut bars)| {
                    bars.sort_by(|a, b| a.0.total_cmp(&b.0));
                    bars
                })
                .filter(|bars| bars.len() == count)
                .collect()
        }
    };
    match found.len() {
        1 => Ok(found.remove(0)),
        n => Err(format!("{n} {mark:?} series with {count} points, not one")),
    }
}

/// The y of each gridline in the group that is this chart's axis, top to bottom.
///
/// A gridline is a horizontal stroked segment. Segments are grouped by their extent, because two
/// charts on one page draw two sets at different widths. The axis is the group with one line per
/// tick label whose span holds every point of the series.
fn gridlines(paths: &[Path], ticks: usize, marks: &[(f64, f64)]) -> Result<Vec<f64>, String> {
    let mut groups: Vec<((f64, f64), Vec<f64>)> = Vec::new();
    for subpath in paths
        .iter()
        .filter(|path| path.stroked && path.fill.is_none())
        .flat_map(|path| &path.subpaths)
        .filter(|subpath| subpath.len() == 2)
    {
        let ((x0, y0), (x1, y1)) = (subpath[0], subpath[1]);
        if (y0 - y1).abs() > 0.001 || (x1 - x0).abs() < 1.0 {
            continue;
        }
        let extent = (
            (x0.min(x1) * 10.0).round() / 10.0,
            (x0.max(x1) * 10.0).round() / 10.0,
        );
        let lines = match groups.iter_mut().find(|(e, _)| *e == extent) {
            Some((_, lines)) => lines,
            None => {
                groups.push((extent, Vec::new()));
                &mut groups.last_mut().expect("just pushed").1
            }
        };
        if !lines.iter().any(|y| (y - y0).abs() < 0.01) {
            lines.push(y0);
        }
    }
    let (left, right) = (marks[0].0, marks[marks.len() - 1].0);
    let mut found: Vec<Vec<f64>> = groups
        .into_iter()
        .filter(|((x0, x1), lines)| {
            let (top, bottom) = lines
                .iter()
                .fold((f64::MAX, f64::MIN), |(t, b), y| (t.min(*y), b.max(*y)));
            lines.len() == ticks
                && *x0 <= left + 0.5
                && *x1 >= right - 0.5
                && marks
                    .iter()
                    .all(|(_, y)| *y >= top - 0.5 && *y <= bottom + 0.5)
        })
        .map(|(_, mut lines)| {
            lines.sort_by(f64::total_cmp);
            lines
        })
        .collect();
    match found.len() {
        1 => Ok(found.remove(0)),
        n => Err(format!(
            "{n} gridline groups of {ticks} lines span the series, not one"
        )),
    }
}

/// The least-squares line `y = intercept + slope * value` through the tick and gridline pairs,
/// and the farthest any pair sits from it.
fn fit(pairs: &[(f64, f64)]) -> (f64, f64, f64) {
    let n = pairs.len() as f64;
    let (sv, sy) = pairs
        .iter()
        .fold((0.0, 0.0), |(a, b), (v, y)| (a + v, b + y));
    let (svv, svy) = pairs
        .iter()
        .fold((0.0, 0.0), |(a, b), (v, y)| (a + v * v, b + v * y));
    let slope = (n * svy - sv * sy) / (n * svv - sv * sv);
    let intercept = (sy - slope * sv) / n;
    let residual = pairs
        .iter()
        .map(|(v, y)| (y - (intercept + slope * v)).abs())
        .fold(0.0, f64::max);
    (intercept, slope, residual)
}

/// One chart's rows, from its page's text layer and drawing.
///
/// # Errors
///
/// If the title, ticks, year row, series or axis cannot each be found exactly once, or if a tick
/// label sits more than a quarter of a point of drawing from the scale the others fit.
pub fn chart_rows(chart: &Chart, text: &str, svg: &str) -> Result<Vec<Vec<String>>, String> {
    let (ticks, years) = axis(text, chart.title)?;
    let paths = paths(svg)?;
    let marks =
        series(&paths, chart.mark, years.len()).map_err(|e| format!("{}: {e}", chart.title))?;
    let lines =
        gridlines(&paths, ticks.len(), &marks).map_err(|e| format!("{}: {e}", chart.title))?;
    // Ticks are read top to bottom and gridlines sorted top to bottom, so they pair in order.
    let pairs: Vec<(f64, f64)> = ticks.iter().copied().zip(lines).collect();
    let (intercept, slope, residual) = fit(&pairs);
    if residual > GRID_RESIDUAL {
        return Err(format!(
            "{}: a tick label sits {residual:.3} points off the fitted scale",
            chart.title
        ));
    }
    let measure = if chart.title.contains("APPLICATIONS") {
        "applications"
    } else {
        "participants"
    };
    Ok(years
        .iter()
        .zip(&marks)
        .map(|(year, (_, y))| {
            vec![
                chart.program.to_string(),
                year.to_string(),
                measure.to_string(),
                format!("{:.1}", (y - intercept) / slope),
                format!("{:.3}", 1.0 / slope.abs()),
            ]
        })
        .collect())
}

/// The one-based page of the report each chart is on, found by its title in the whole text layer.
///
/// `pdftotext` ends every page with a form feed, so a title's page is one more than the number of
/// form feeds before it.
///
/// # Errors
///
/// If a title is not in the report, or is in it more than once.
pub fn chart_pages(text: &str) -> Result<Vec<(Chart, usize)>, String> {
    CHARTS
        .iter()
        .map(|chart| {
            let pages: Vec<usize> = text
                .split('\u{c}')
                .enumerate()
                .filter(|(_, page)| page.contains(chart.title))
                .map(|(index, _)| index + 1)
                .collect();
            match pages.as_slice() {
                [page] => Ok((*chart, *page)),
                _ => Err(format!(
                    "`{}` is on {} pages, not one",
                    chart.title,
                    pages.len()
                )),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A page with an area chart of three years over two ticks: 0 at y=200, 100 at y=100.
    ///
    /// The gridlines are drawn under a flipping transform, as `pdftocairo` writes them; the area is
    /// in page coordinates. A second, wider set of gridlines belongs to another chart.
    const AREA_SVG: &str = r#"<svg>
<path fill="none" stroke="rgb(85%, 85%, 85%)" d="M 0 0 L 300 0 M 0 100 L 300 100 " transform="matrix(1, 0, 0, -1, 50, 200)"/>
<path fill="none" stroke="rgb(85%, 85%, 85%)" d="M 0 0 L 400 0 M 0 100 L 400 100 " transform="matrix(1, 0, 0, -1, 10, 500)"/>
<path fill-rule="nonzero" fill="rgb(41%, 76%, 77%)" d="M 60 150 L 150 125 L 340 100 L 340 200 L 60 200 Z M 60 150 "/>
</svg>"#;

    const AREA_TEXT: &str = "\
        TOTAL WIDGETS BY FISCAL
             YEAR
 100

   0
      2023 2024 2025
";

    fn widgets(mark: Mark) -> Chart {
        Chart {
            program: "widgets",
            title: "TOTAL WIDGETS",
            mark,
        }
    }

    #[test]
    fn an_area_chart_reads_its_upper_edge_on_the_gridline_scale() {
        let rows = chart_rows(&widgets(Mark::Area), AREA_TEXT, AREA_SVG).unwrap();
        let values: Vec<&str> = rows.iter().map(|row| row[3].as_str()).collect();
        assert_eq!(values, ["50.0", "75.0", "100.0"]);
        assert_eq!(rows[0][1], "2023");
        assert_eq!(rows[0][2], "participants");
        assert_eq!(rows[0][4], "1.000");
    }

    #[test]
    fn a_line_chart_reads_its_vertices_through_the_transform() {
        let svg = r#"<svg>
<path fill="none" stroke="rgb(85%, 85%, 85%)" d="M 0 0 L 300 0 M 0 100 L 300 100 " transform="matrix(1, 0, 0, -1, 50, 200)"/>
<path fill="none" stroke-width="2.25" stroke="rgb(5%, 25%, 46%)" d="M 0 0 L 100 20 L 200 80 " transform="matrix(1, 0, 0, -1, 70, 190)"/>
</svg>"#;
        let rows = chart_rows(&widgets(Mark::Line), AREA_TEXT, svg).unwrap();
        let values: Vec<&str> = rows.iter().map(|row| row[3].as_str()).collect();
        assert_eq!(values, ["10.0", "30.0", "90.0"]);
    }

    #[test]
    fn a_bar_chart_takes_the_colour_with_a_bar_for_every_year() {
        // Three blue bars are the series; two red ones belong to another chart.
        let svg = r#"<svg>
<path fill="none" stroke="rgb(85%, 85%, 85%)" d="M 0 0 L 300 0 M 0 100 L 300 100 " transform="matrix(1, 0, 0, -1, 50, 200)"/>
<path fill="rgb(5%, 25%, 46%)" d="M 290 120 L 300 120 L 300 200 L 290 200 Z M 290 120 "/>
<path fill="rgb(5%, 25%, 46%)" d="M 60 180 L 70 180 L 70 200 L 60 200 Z M 60 180 "/>
<path fill="rgb(5%, 25%, 46%)" d="M 170 140 L 180 140 L 180 200 L 170 200 Z M 170 140 "/>
<path fill="rgb(76%, 15%, 22%)" d="M 60 150 L 70 150 L 70 200 L 60 200 Z M 60 150 "/>
<path fill="rgb(76%, 15%, 22%)" d="M 90 150 L 95 150 L 95 200 L 90 200 Z M 90 150 "/>
</svg>"#;
        let text = AREA_TEXT.replace("WIDGETS", "WIDGET APPLICATIONS");
        let chart = Chart {
            title: "TOTAL WIDGET APPLICATIONS",
            ..widgets(Mark::Bars)
        };
        let rows = chart_rows(&chart, &text, svg).unwrap();
        let values: Vec<&str> = rows.iter().map(|row| row[3].as_str()).collect();
        assert_eq!(values, ["20.0", "60.0", "80.0"]);
        assert_eq!(rows[0][2], "applications");
    }

    #[test]
    fn a_tick_label_off_its_line_fails_the_read() {
        // Three ticks on three lines, the middle line a point and a half low of where 50 belongs.
        let svg = r#"<svg>
<path fill="none" stroke="rgb(85%, 85%, 85%)" d="M 50 100 L 350 100 M 50 151.5 L 350 151.5 M 50 200 L 350 200 "/>
<path fill="rgb(41%, 76%, 77%)" d="M 60 150 L 150 125 L 340 100 L 340 200 L 60 200 Z M 60 150 "/>
</svg>"#;
        let text = AREA_TEXT.replace(" 100\n", " 100\n 50\n");
        let error = chart_rows(&widgets(Mark::Area), &text, svg).unwrap_err();
        assert!(error.contains("off the fitted scale"), "{error}");
    }

    #[test]
    fn a_year_row_that_disagrees_with_the_drawing_fails_the_read() {
        let text = AREA_TEXT.replace("2023 2024 2025", "2022 2023 2024 2025");
        let error = chart_rows(&widgets(Mark::Area), &text, AREA_SVG).unwrap_err();
        assert!(error.contains("0 Area series with 4 points"), "{error}");
    }

    #[test]
    fn a_chart_is_found_on_the_page_its_title_is_on_and_only_one() {
        let mut text: String = CHARTS
            .iter()
            .map(|chart| format!("{}\u{c}", chart.title))
            .collect();
        let pages: Vec<usize> = chart_pages(&text)
            .unwrap()
            .iter()
            .map(|(_, page)| *page)
            .collect();
        assert_eq!(pages, [1, 2, 3, 4, 5]);
        text.push_str("TOTAL JPSN APPLICATIONS\u{c}");
        let error = chart_pages(&text).unwrap_err();
        assert!(
            error.contains("JPSN APPLICATIONS` is on 2 pages"),
            "{error}"
        );
    }
}
