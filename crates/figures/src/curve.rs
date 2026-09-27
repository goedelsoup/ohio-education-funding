//! The fourth chart registry: [`CURVES`], two or more named lines over one ordered index, drawn
//! against a [`Reference`].
//!
//! A curve's index is monotone and the claim is a shape along it, so neither extreme of the
//! picture is the thing a reader quotes: both ends of each line are, and so is the worst
//! departure from the reference. Three figures per line, which is what makes "flat the whole way"
//! a claim the figure manifest has stood behind.

use super::*;

/// One curve the wiki draws: named lines over one ordered index, against a reference.
///
/// The fourth drawing, and a different shape from all three of the others. A [`Series`] is
/// categories, which could be reordered without changing what the chart says; a [`Scatter`] is a
/// population with no order at all; a [`Plane`] is a handful of named alternatives on two axes.
/// A curve's index is **monotone** and the claim lives along it — that the one line falls away
/// and the other does not. Drawing that as thirteen pairs of bars would put the reader in front
/// of the one presentation a shape survives least well.
///
/// What stands in for the endpoint rule is three figures per line rather than two per chart: see
/// [`Line`], and the module docs for why the reference is the one number here that is not pinned.
pub struct Curve {
    /// `<crate-directory>/<what-it-is>`, in the one key namespace the other registries share.
    pub key: &'static str,
    /// The crate that owns the computation, as the corpus cites it.
    pub owner: &'static str,
    /// What the whole drawing is, in words; its title.
    pub label: &'static str,
    /// What one step along the index is, singular and lower case — "horizon". Written into the
    /// caption, and into the tick labels, so a reader is told what the lines run over.
    pub subject: &'static str,
    /// The computation, over the same inputs the figures use.
    pub compute: fn(&Inputs) -> Traces,
}

/// The line a [`Curve`]'s lines are read against, which is not one of them.
pub struct Reference {
    /// What the line means, in words.
    pub label: &'static str,
    /// Where it sits, in the vertical axis' units.
    pub value: f64,
}

/// The lines of a [`Curve`] and the frame they are drawn in.
pub struct Traces {
    /// The shared index. One axis for every line, because the comparison between the lines at the
    /// same position is the whole of what a curve is for.
    pub x: Axis,
    /// The vertical axis, framed to hold every line **and** the reference: a reference drawn
    /// outside the frame is a comparison the reader is asked to make and not shown.
    pub y: Axis,
    /// The line the others are read against, where there is one.
    pub reference: Option<Reference>,
    /// The lines, in the order the owning crate returned them.
    pub lines: Vec<Line>,
}

impl Traces {
    /// Frame a set of lines around their own extremes **and** the reference.
    ///
    /// The reference is inside the domain by construction rather than by luck. A vertical axis
    /// fitted to the values alone would drop it the moment every line sat on one side of it, and
    /// then the chart would be drawing a set of lines where the claim is a set of distances.
    ///
    /// # Panics
    ///
    /// If there are no lines, a line has no points, or a value is not finite.
    #[must_use]
    pub fn framed(
        x: Axis,
        y: &'static str,
        unit: Unit,
        reference: Option<Reference>,
        lines: Vec<Line>,
    ) -> Self {
        assert!(!lines.is_empty(), "a curve with no lines");
        assert!(
            lines.iter().all(|line| !line.points.is_empty()),
            "a curve with an empty line"
        );
        let values = lines
            .iter()
            .flat_map(|line| line.points.iter().map(|p| p.y))
            .chain(reference.iter().map(|r| r.value));
        let (low, high) = values.fold((f64::MAX, f64::MIN), |(lo, hi), v| {
            assert!(v.is_finite(), "{y}: {v} cannot be drawn");
            (lo.min(v), hi.max(v))
        });
        let pad = (high - low) * CLOUD_PAD;
        Self {
            x,
            y: Axis {
                label: y,
                unit,
                min: low - pad,
                max: high + pad,
                log: false,
            },
            reference,
            lines,
        }
    }
}

/// One of the two bias curves: what the two published quantities were off by, over one population.
///
/// Both curves are this function with a different `of`, because the pair of *lines* is the
/// finding — the mean district’s bias and the whole state’s are two numbers rather
/// than two estimates of one — and the pair of *curves* is the population axis #431 put under it.
///
/// Zero is the reference, and like the 68.3% the coverage curve is read against it is a
/// definition rather than a measurement, so it is pinned nowhere: a forecast on the line is right
/// on average and one off it is wrong in a *direction*.
///
/// The worst departure each line names is a magnitude, which is all [`Departure`] holds and all
/// `crates/figures.json` can carry. Every one of these four lines starts below zero, crosses it,
/// and ends further from it than it has been at any horizon before — so the sign is not lost by
/// the line, it is stated by the first point and by the prose that quotes it.
///
/// # Panics
///
/// If no row carries the population `of` selects.
fn bias_traces(
    rows: &[project::backtest::Drift],
    of: fn(&project::backtest::Drift) -> Option<project::backtest::Bias>,
    y: &'static str,
    mean_district: [&'static str; 3],
    total: [&'static str; 3],
) -> Traces {
    let line = |label: &'static str,
                measure: fn(&project::backtest::Bias) -> f64,
                keys: [&'static str; 3]| {
        let value = |d: &project::backtest::Drift| of(d).map(|b| measure(&b));
        let (at, drift) = project::backtest::worst_drift(rows, value);
        Line {
            label,
            points: rows
                .iter()
                .filter_map(|d| {
                    value(d).map(|y| Coordinate {
                        x: f64::from(d.horizon),
                        y,
                    })
                })
                .collect(),
            first_figure: keys[0],
            last_figure: keys[1],
            worst: Departure {
                at: f64::from(at),
                gap: drift.abs(),
                figure: keys[2],
            },
        }
    };
    let deepest = rows
        .iter()
        .filter(|d| of(d).is_some())
        .map(|d| d.horizon)
        .max()
        .expect("a bias profile carrying no row for this population");
    Traces::framed(
        Axis {
            label: "Years ahead",
            unit: Unit::Count,
            min: 1.0,
            max: f64::from(deepest),
            log: false,
        },
        y,
        Unit::Ratio,
        Some(Reference {
            label: "No bias",
            value: 0.0,
        }),
        vec![
            line("The mean district", |b| b.mean_district, mean_district),
            line("The state total", |b| b.total, total),
        ],
    )
}

/// The curves the wiki draws. See [`Curve`] for what stands in for the endpoint rule here.
pub static CURVES: &[Curve] = &[
    Curve {
        key: "project/what-the-band-held-at-every-horizon",
        owner: "crates/project",
        label: "What the ±1σ band actually held, at every horizon the panel reaches, over every \
            origin and district it admits",
        subject: "horizon",
        compute: |i| {
            let line = |label: &'static str,
                        of: fn(&project::backtest::Held) -> f64,
                        first_figure,
                        last_figure,
                        worst_figure| {
                let (at, gap) = project::backtest::worst_gap(&i.coverage, of);
                Line {
                    label,
                    points: i
                        .coverage
                        .iter()
                        .map(|held| Coordinate {
                            x: f64::from(held.horizon),
                            y: of(held),
                        })
                        .collect(),
                    first_figure,
                    last_figure,
                    worst: Departure {
                        at: f64::from(at),
                        gap,
                        figure: worst_figure,
                    },
                }
            };
            Traces::framed(
                Axis {
                    label: "Years ahead",
                    unit: Unit::Count,
                    min: 1.0,
                    max: f64::from(project::backtest::DEEPEST_HORIZON),
                    log: false,
                },
                "Share of forecasts inside the band",
                Unit::Share,
                Some(Reference {
                    label: "What ±1σ claims to hold",
                    value: project::backtest::NOMINAL_ONE_SIGMA_COVERAGE,
                }),
                vec![
                    line(
                        "Every error, origins pooled",
                        |held| held.pooled,
                        "project/pooled-coverage-at-one-year",
                        "project/pooled-coverage-at-thirteen-years",
                        "project/the-pooled-bands-worst-gap-from-nominal",
                    ),
                    line(
                        "The same errors, each origin's mean removed",
                        |held| held.within_origin,
                        "project/cross-district-coverage-at-one-year",
                        "project/cross-district-coverage-at-thirteen-years",
                        "project/the-cross-district-bands-worst-gap-from-nominal",
                    ),
                ],
            )
        },
    },
    Curve {
        key: "project/the-bias-before-the-closure",
        owner: "crates/project",
        label: "How far each of the two published quantities sat from the truth at every horizon, \
            over the forecasts whose target year is FY2020 or earlier \u{2014} the mean \
            district\u{2019}s log error and the whole state\u{2019}s, which are not one number",
        subject: "horizon",
        compute: |i| {
            bias_traces(
                &i.bias,
                |d| d.before_the_closure,
                "Bias, as a log error",
                [
                    "project/the-mean-district-bias-at-one-year-negative",
                    "project/the-mean-district-bias-at-nine-years-before-the-closure",
                    "project/the-worst-mean-district-bias-before-the-closure",
                ],
                [
                    "project/the-total-bias-at-one-year-negative",
                    "project/the-total-bias-at-nine-years-before-the-closure",
                    "project/the-worst-total-bias-before-the-closure",
                ],
            )
        },
    },
    Curve {
        key: "project/the-bias-across-the-closure",
        owner: "crates/project",
        label: "The same two quantities over every scored forecast, the school closures included \
            \u{2014} four horizons deeper than the panel reaches without them, and further from \
            zero at every one of them",
        subject: "horizon",
        compute: |i| {
            bias_traces(
                &i.bias,
                |d| Some(d.across_it),
                "Bias, as a log error",
                [
                    "project/the-mean-district-bias-at-one-year-negative",
                    "project/the-mean-district-bias-at-thirteen-years",
                    "project/the-worst-mean-district-bias-across-the-closure",
                ],
                [
                    "project/the-total-bias-at-one-year-negative",
                    "project/the-total-bias-at-thirteen-years",
                    "project/the-worst-total-bias-across-the-closure",
                ],
            )
        },
    },
];

/// A curve and the lines it came out with on this run.
pub struct ComputedCurve {
    /// The registry entry.
    pub curve: &'static Curve,
    /// What [`Curve::compute`] returned.
    pub traces: Traces,
}

/// Run every curve in [`CURVES`], in registry order.
#[must_use]
pub fn compute_all_curves() -> Vec<ComputedCurve> {
    let inputs = Inputs::shared();
    CURVES
        .iter()
        .map(|curve| ComputedCurve {
            curve,
            traces: (curve.compute)(inputs),
        })
        .collect()
}
