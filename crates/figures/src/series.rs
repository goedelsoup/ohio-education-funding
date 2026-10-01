//! The first chart registry: [`SERIES`], one short column of labelled [`Row`]s per entry.
//!
//! What keeps it honest is the endpoint rule, held in `tests/a_series_is_bound_at_its_ends.rs`:
//! the largest and smallest row of every series name a figure in [`FIGURES`] that reproduces
//! them. A column is the one chart form whose categories can be reordered without changing what
//! it says, and its extremes are what a reader quotes.

use super::*;

/// One column the wiki draws: a short list of labelled values on one unit and one axis.
///
/// Where a [`Figure`] carries a pin, a series carries none of its own. Its endpoints name figures
/// (see [`Row::figure`]), and those figures are pinned; the rest of the column is held by
/// `mise run //:generated` diffing the committed document, which is enough for values that no
/// prose quotes.
pub struct Series {
    /// `<crate-directory>/<what-it-is>`, in the same namespace as a figure key and never equal
    /// to one: a corpus node binds series and figures in separate lists.
    pub key: &'static str,
    /// The crate that owns the computation, as the corpus cites it.
    pub owner: &'static str,
    /// What every row's value is measured in.
    pub unit: Unit,
    /// What the rows are indexed by, in words — the categorical axis of the chart.
    pub axis: &'static str,
    /// What the column is, in words; the chart's title.
    pub label: &'static str,
    /// The computation, over the same inputs the figures use.
    pub compute: fn(&Inputs) -> Vec<Row>,
}

/// One row of a [`Series`].
pub struct Row {
    /// The category this row is, as the axis writes it.
    pub label: &'static str,
    /// The value, **signed**. A chart draws direction; the figure manifest states it in a key.
    pub value: f64,
    /// What a reader hovering the mark should be told, when the label and the value are not
    /// enough on their own. Absent, the chart says `label: value`.
    pub hover: Option<&'static str>,
    /// The run of rows this one belongs to, where the axis has a second level — a family, a
    /// class, a year. A chart may band the rows by it; a table under the chart may head them.
    ///
    /// Absent on a column of four bars that are simply four things. Present where the axis is
    /// long enough that a reader needs to be told which part of the subject they are inside of.
    pub group: Option<&'static str>,
    /// The denominator, where the value is a count out of a population that is not the whole.
    ///
    /// A census of bounds is the case this exists for: most of the modelled formula's bounds are
    /// defined for all 609 districts, seven of them for the 604 that run buses, and one for the
    /// 43 that are charged. `212` against `604` and `212` against `609` are different claims,
    /// and a chart that draws the numerator alone cannot tell a reader which it made.
    pub of: Option<f64>,
    /// The authority this row's value rests on, where the row is a claim with a source of its
    /// own rather than a slice of the series' one computation.
    ///
    /// Not the same thing as [`Self::hover`]: a hover is what to say about a mark, and this is
    /// where the row came from. A table drawn from the series prints it as a column.
    pub cites: Option<&'static str>,
    /// Why this row is the one the chart was drawn to point at, in a phrase, when one of them is
    /// the finding.
    ///
    /// A chart with a subject draws it in the contrasting hue with this as a direct label — see
    /// `Bar.current` and `Bar.direct` in `web/src/lib/chart.ts`, which are the same two ideas.
    /// It exists because the finding a series is sometimes drawn for is a row at *zero*, and a
    /// zero-length mark says nothing: the bound that cannot bind has to be marked rather than
    /// dropped, and at most one row a series carries may be.
    pub marked: Option<&'static str>,
    /// The [`Figure`] in [`FIGURES`] whose pin this row reproduces in magnitude, when there is
    /// one. Required on the largest and smallest row of every series — the endpoint rule in the
    /// module docs — and welcome on any other.
    pub figure: Option<&'static str>,
}

/// A series and the rows it came out with on this run.
pub struct ComputedSeries {
    /// The registry entry.
    pub series: &'static Series,
    /// What [`Series::compute`] returned.
    pub rows: Vec<Row>,
}

/// Run every series in [`SERIES`], in registry order.
#[must_use]
pub fn compute_all_series() -> Vec<ComputedSeries> {
    let inputs = Inputs::shared();
    SERIES
        .iter()
        .map(|series| ComputedSeries {
            series,
            rows: (series.compute)(inputs),
        })
        .collect()
}

/// The five wealth fifths of #444's small multiples, least wealthy first.
const WEALTH_FIFTHS: [&str; 5] = ["Least wealthy", "Second", "Third", "Fourth", "Wealthiest"];

/// And the five disadvantage fifths, least poor first.
const POVERTY_FIFTHS: [&str; 5] = ["Least poor", "Second", "Third", "Fourth", "Poorest"];

/// One axis of #444's small multiples: seven rules, five fifths each, per pupil.
///
/// One grouped series rather than seven, because seven separate documents would each need their
/// own pair of endpoint figures and the thing a reader is being shown is the comparison between
/// them. `figures` is aligned to [`Reach::rules`] and then to `labels`, so a binding sits at the
/// rule and the fifth it belongs to.
fn by_fifths(
    i: &Inputs,
    axis: project::anchor_incidence::Axis,
    labels: [&'static str; 5],
    figures: [[Option<&'static str>; 5]; 7],
) -> Vec<Row> {
    i.reach
        .rules()
        .into_iter()
        .zip(figures)
        .flat_map(move |((group, frame), binds)| {
            project::anchor_incidence::by(frame, axis, 5, &i.reach.cluster)
                .into_iter()
                .map(move |band| Row {
                    label: labels[band.rank - 1],
                    value: band.per_pupil(),
                    hover: None,
                    group: Some(group),
                    of: None,
                    cites: None,
                    marked: None,
                    figure: binds[band.rank - 1],
                })
        })
        .collect()
}

/// The columns the wiki draws. See the module docs for the endpoint rule every entry obeys.
pub static SERIES: &[Series] = &[
    // #483's column. The designated set against the population it is cut from, in each of the
    // three published editions — the numerator grows in both transitions while the denominator
    // shrinks in both, which is a shape no level can carry and the reason the row states its
    // `of`. The ends are 2024-2025 and 2026-2027 and both are figures the node binds.
    Series {
        key: "dispersion/edchoice-designated-by-edition",
        owner: "crates/dispersion",
        unit: Unit::Count,
        axis: "Edition of the designated list, by the school year it governs",
        label: "Buildings designated for traditional EdChoice in each published edition, against \
                every building the edition lists",
        compute: |_| {
            use dispersion::designated_editions::{Edition, EDITIONS};
            EDITIONS
                .into_iter()
                .map(|which| {
                    let (designated, listed) = edchoice_edition_counts(which);
                    Row {
                        label: which.school_year(),
                        value: designated as f64,
                        hover: match which {
                            Edition::Y2425 => Some(
                                "The bottom-fifth window is cut from two rankings here, not three",
                            ),
                            _ => None,
                        },
                        group: None,
                        of: Some(listed as f64),
                        cites: None,
                        marked: None,
                        figure: match which {
                            Edition::Y2425 => Some("dispersion/edchoice-designated-buildings-2425"),
                            Edition::Y2526 => None,
                            Edition::Y2627 => Some("dispersion/edchoice-designated-buildings"),
                        },
                    }
                })
                .collect()
        },
    },
    // #416's finding, as the four bars it was found by: the summed business share is a near-zero
    // because industrial runs one way and mineral and public utility the other. The chart is
    // signed so that the cancellation is visible, and the figures it names are the magnitudes
    // the corpus quotes.
    Series {
        key: "dispersion/fy2016-step-by-business-class",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        axis: "Property class, TY2021 share of assessed value",
        label: "Correlation of the FY2016 move in state revenue per pupil with each business \
                property class",
        compute: |i| {
            vec![
                Row {
                    label: "Industrial",
                    value: fy2016_step_against(i, |d| d.industrial_share),
                    hover: None,
                    group: None,
                    of: None,
                    cites: None,
                    marked: None,
                    figure: Some("dispersion/fy2016-step-against-industrial-share"),
                },
                Row {
                    label: "Mineral",
                    value: fy2016_step_against(i, |d| d.mineral_share),
                    hover: None,
                    group: None,
                    of: None,
                    cites: None,
                    marked: None,
                    figure: Some("dispersion/fy2016-step-against-mineral-share-negative"),
                },
                Row {
                    label: "Public utility",
                    value: fy2016_step_against(i, |d| d.public_utility_share),
                    hover: None,
                    group: None,
                    of: None,
                    cites: None,
                    marked: None,
                    figure: Some("dispersion/fy2016-step-against-public-utility-share-negative"),
                },
                Row {
                    label: "All three summed",
                    value: fy2016_step_against(
                        i,
                        dispersion::fy2016::District::summed_business_share,
                    ),
                    hover: None,
                    group: None,
                    of: None,
                    cites: None,
                    marked: None,
                    figure: Some("dispersion/fy2016-step-against-summed-business-share"),
                },
            ]
        },
    },
    // #410's census, as the chart it was always a table of: every bound the modelled formula
    // states, ranked by how many of the 609 districts it is the operative term for. The long
    // column this manifest was extended for — `group`, `of`, `cites` and `marked` all exist
    // because thirty-eight rows on one count is a table as much as it is a chart, and because
    // the row that matters most is at zero.
    Series {
        key: "project/bounds-census",
        owner: "crates/project",
        unit: Unit::Count,
        axis: "Bound, ranked by the districts it is the operative term for",
        label: "Every bound in the modeled formula, and how many districts each one is the \
                operative term for",
        compute: |i| {
            census_ranked(i)
                .into_iter()
                .map(|row| {
                    let bound = row.bound;
                    Row {
                        label: bound.short(),
                        #[allow(clippy::cast_precision_loss)]
                        value: row.operative as f64,
                        hover: match bound.reach() {
                            // The one row whose count is not a fact about Ohio. R.C.
                            // 3317.017(A)(4)(d)(i) states its ceiling against the fortieth
                            // highest district, so forty are on it whatever the incomes are.
                            project::bounds::Reach::Fixed(_) => {
                                Some("Fixed by construction: the ceiling is stated against a rank")
                            }
                            _ => None,
                        },
                        group: Some(bound.family().label()),
                        #[allow(clippy::cast_precision_loss)]
                        of: Some(row.population as f64),
                        cites: Some(bound.authority()),
                        marked: match bound.reach() {
                            project::bounds::Reach::Unreachable => Some("cannot bind"),
                            _ => None,
                        },
                        figure: None,
                    }
                })
                .enumerate()
                .map(|(at, row)| Row {
                    // The endpoint rule, bound to the ranking rather than to a bound's name: the
                    // top and the bottom of the chart are the two numbers a reader quotes off it,
                    // and `census_extreme` computes those two figures the same positional way.
                    figure: match at {
                        0 => Some("project/bounds-the-most-reached-is-operative-for"),
                        at if at + 1 == project::bounds::Bound::all().len() => {
                            Some("project/bounds-the-least-reached-is-operative-for")
                        }
                        _ => None,
                    },
                    ..row
                })
                .collect()
        },
    },
    // #444's second and third drawings: the same seven rules, cut into fifths on each of the two
    // axes #433 cut and only one of which is usually shown. Two documents rather than one,
    // because the per-pupil figures differ by two orders of magnitude between the axes — $138.58
    // against $4.06 in the ratchet's first fifth — and one scale across both would flatten the
    // second into nothing. Separate series make the separate scale structural rather than a
    // promise in a caption.
    Series {
        key: "project/what-each-anchor-rule-pays-by-wealth",
        owner: "crates/project",
        unit: Unit::Dollars,
        axis: "Fifth of districts by assessed valuation per pupil, outside the cluster",
        label: "What each anchor rule pays per pupil by wealth, on total state support, against \
                the enacted anchor",
        compute: |i| {
            by_fifths(
                i,
                project::anchor_incidence::Axis::Valuation,
                WEALTH_FIFTHS,
                [
                    [
                        Some("project/a-ratchets-gain-per-pupil-in-the-least-wealthy-fifth"),
                        None,
                        None,
                        None,
                        Some("project/a-ratchets-gain-per-pupil-in-the-wealthiest-fifth"),
                    ],
                    [
                        None,
                        None,
                        None,
                        Some("project/a-caps-cut-per-pupil-in-the-fourth-wealth-fifth"),
                        None,
                    ],
                    [
                        Some("project/the-gentle-caps-gain-per-pupil-in-the-least-wealthy-fifth"),
                        None,
                        None,
                        None,
                        Some("project/the-gentle-caps-cut-per-pupil-in-the-wealthiest-fifth"),
                    ],
                    [
                        Some(
                            "project/the-rolling-counts-gain-per-pupil-in-the-least-wealthy-fifth",
                        ),
                        None,
                        None,
                        None,
                        Some("project/the-rolling-counts-gain-per-pupil-in-the-wealthiest-fifth"),
                    ],
                    [
                        None,
                        None,
                        None,
                        None,
                        Some("project/the-mirror-inside-h-per-pupil-in-the-wealthiest-fifth"),
                    ],
                    [
                        None,
                        None,
                        None,
                        None,
                        Some("project/the-mirror-beside-per-pupil-in-the-wealthiest-fifth"),
                    ],
                    [None, None, None, None, None],
                ],
            )
        },
    },
    Series {
        key: "project/what-each-anchor-rule-pays-by-disadvantaged-share",
        owner: "crates/project",
        unit: Unit::Dollars,
        axis: "Fifth of districts by economically disadvantaged share, outside the cluster",
        label: "What each anchor rule pays per pupil by disadvantaged share, on total state \
                support, against the enacted anchor",
        compute: |i| {
            by_fifths(
                i,
                project::anchor_incidence::Axis::Poverty,
                POVERTY_FIFTHS,
                [
                    [
                        None,
                        None,
                        None,
                        None,
                        Some("project/a-ratchets-gain-per-pupil-in-the-poorest-fifth"),
                    ],
                    [
                        None,
                        Some("project/a-caps-cut-per-pupil-in-the-second-poverty-fifth"),
                        None,
                        None,
                        Some("project/a-caps-cut-per-pupil-in-the-poorest-fifth"),
                    ],
                    [None, None, None, None, None],
                    [None, None, None, None, None],
                    [None, None, None, None, None],
                    [
                        None,
                        None,
                        Some("project/the-mirror-beside-gain-per-pupil-in-the-third-poverty-fifth"),
                        None,
                        None,
                    ],
                    [None, None, None, None, None],
                ],
            )
        },
    },
];
