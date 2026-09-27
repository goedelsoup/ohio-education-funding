//! The second chart registry: [`SCATTERS`], a whole population on a continuous predictor.
//!
//! A [`Cloud`] carries the [`Panel`]s a reader can switch between and the [`Fit`] the owning
//! crate computed for each. A column has endpoints and a cloud does not, so what stands in for
//! the endpoint rule is the fit: a panel names the figures that pin its slope and its r-squared,
//! and those are the only two numbers printed on it.

use super::*;

/// One cloud the wiki draws: a population, on one shared predictor, under one or more panels.
///
/// The second drawing this manifest carries, and a different shape from [`Series`] rather than a
/// variant of it. A column is four labelled values a reader could have read in a table; a cloud is
/// six hundred districts a table cannot hold, and what it is read for is a *shape*. So its points
/// carry no categorical label, its axes are continuous, and what stands in for the endpoint rule
/// is the **fit**: each panel names the figures that pin its slope and its r-squared, which are
/// the only numbers printed on it and so the only ones a reader can quote off it.
///
/// A fitted line is a claim about a model, which is why `web/src/lib/chart.ts` forbids the web
/// layer from drawing one of its own. This is where such a claim is allowed to be made, because
/// here it has a checkpoint behind it: the line is computed by the crate that owns the
/// computation, through the points that same crate returned, and both its numbers are pinned.
pub struct Scatter {
    /// `<crate-directory>/<what-it-is>`, in the one key namespace [`Series`] and [`Figure`] share.
    pub key: &'static str,
    /// The crate that owns the computation, as the corpus cites it.
    pub owner: &'static str,
    /// What the whole drawing is, in words; its title.
    pub label: &'static str,
    /// What one point is, singular and lower case — "district". Written into the caption, so that
    /// a reader is told what the cloud is six hundred of.
    pub subject: &'static str,
    /// The computation, over the same inputs the figures use.
    pub compute: fn(&Inputs) -> Cloud,
}

/// One panel as its caller states it, before the shared scale decides the frame it is drawn in.
///
/// The frame is deliberately not here: [`Cloud::at_one_scale`] computes every panel's domain from
/// the predictor's, and a caller able to supply one could supply one that redraws its own slope.
pub struct PanelSpec {
    /// What the panel is called, which is a sentence about what this outcome is to the argument.
    pub label: &'static str,
    /// What its vertical axis measures, in words.
    pub axis: &'static str,
    /// What that is measured in.
    pub unit: Unit,
    /// The line through it.
    pub fit: Fit,
}

/// One point of a [`Cloud`] — one subject, once, with a value under every panel.
pub struct Point {
    /// What the tooltip calls it.
    pub label: String,
    /// Its position on the shared predictor.
    pub x: f64,
    /// Its position on each panel's outcome, aligned to [`Cloud::panels`].
    pub ys: Vec<f64>,
}

/// One panel of a [`Cloud`]: a second outcome for the same subjects on the same predictor.
pub struct Panel {
    /// What this panel's outcome is, in words; the panel's own title.
    pub label: &'static str,
    /// The outcome axis, on a domain [`Cloud::at_one_scale`] has matched to its siblings'.
    pub y: Axis,
    /// The line through this panel's points.
    pub fit: Fit,
}

/// A line a crate fitted, and the two numbers a reader takes off it.
pub struct Fit {
    /// The line's low end, at [`Cloud::x`]'s own low end, in the axes' units.
    pub from: (f64, f64),
    /// Its high end.
    pub to: (f64, f64),
    /// The slope, in the space the fit was made in — logs, here.
    pub slope: f64,
    /// The share of the outcome's variance the line accounts for.
    pub r_squared: f64,
    /// The [`Figure`] that pins [`Self::slope`].
    pub slope_figure: &'static str,
    /// The [`Figure`] that pins [`Self::r_squared`].
    pub r_squared_figure: &'static str,
}

/// A population drawn on one predictor, once, under every panel that reads it.
pub struct Cloud {
    /// The shared predictor. One axis for every panel, because the comparison across panels is
    /// the point and two x axes fitted separately would be two different pictures.
    pub x: Axis,
    /// The subjects, in the order the owning crate returned them.
    pub points: Vec<Point>,
    /// The outcomes, each drawn against [`Self::x`].
    pub panels: Vec<Panel>,
}

impl Cloud {
    /// Build a cloud whose panels are all drawn at **one scale**, and say what that means.
    ///
    /// A slope is an angle only if a unit of the outcome is drawn the same length as a unit of the
    /// predictor. Two panels whose y axes are each fitted to their own data are two different
    /// scales, and then a flat cloud magnified to fill its frame reads as a noisy sloped one —
    /// which is the exact misreading a pair of panels like this exists to refute.
    ///
    /// So every panel's y axis is given **the same span as the x axis**, in logs. A slope of one
    /// is then a diagonal, a slope of zero is level, and the two panels are comparable because
    /// they are the same picture with one quantity swapped. The frame is square, which is what
    /// `scatterSpec`'s identity line already spends a fixed height on and for the same reason:
    /// where the geometry is the claim, a frame that is not square draws a different claim.
    ///
    /// Each panel is centred on the **mid-range of what it has to hold** — its own points and its
    /// own fitted line, together. Not the mean of the logs, which is where the line passes: the
    /// per-pupil panel holds Kelleys Island Local at fifty times the typical district, and
    /// centring on the mean would spend four log units of frame below a cloud with nothing in
    /// them. Not the points alone either, because a line of slope one nearly fills a square frame
    /// and will leave it unless the frame is placed around it.
    ///
    /// # Panics
    ///
    /// If there are no points, no panels, a value a log axis cannot place, or a panel whose
    /// points and line together want more room than the x axis spans. The last is the one worth
    /// stating: it means the relationship is steeper than one, and the author has to choose
    /// between the diagonal reading and the whole cloud rather than silently losing points off
    /// the top of the frame.
    #[must_use]
    pub fn at_one_scale(x: Axis, points: Vec<Point>, panels: Vec<PanelSpec>) -> Self {
        assert!(!points.is_empty(), "a cloud with no points");
        assert!(!panels.is_empty(), "a cloud with no panels");
        assert!(x.log && x.min > 0.0, "the shared predictor is a log axis");
        let span = (x.max / x.min).ln();
        Self {
            x,
            panels: panels
                .into_iter()
                .enumerate()
                .map(
                    |(
                        at,
                        PanelSpec {
                            label,
                            axis,
                            unit,
                            fit,
                        },
                    )| {
                        let held = points
                            .iter()
                            .map(|p| {
                                let value = *p.ys.get(at).expect("a value under every panel");
                                assert!(
                                    value > 0.0 && value.is_finite(),
                                    "{:?} is {value} on panel {at}, which a log axis cannot place",
                                    p.label
                                );
                                value
                            })
                            .chain([fit.from.1, fit.to.1])
                            .map(f64::ln);
                        let (low, high) = held
                            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                                (lo.min(v), hi.max(v))
                            });
                        assert!(
                            high - low <= span,
                            "{label:?}: its points and its fitted line span {} log units and the \
                         shared scale is {span}. One unit of this outcome has to be drawn as long \
                         as one unit of the predictor or the slope is not an angle, so a panel \
                         that wants more room is a panel whose relationship is steeper than one.",
                            high - low
                        );
                        let centre = (low + high) / 2.0;
                        Panel {
                            label,
                            y: Axis {
                                label: axis,
                                unit,
                                min: (centre - span / 2.0).exp(),
                                max: (centre + span / 2.0).exp(),
                                log: true,
                            },
                            fit,
                        }
                    },
                )
                .collect(),
            points,
        }
    }
}

/// The clouds the wiki draws. See [`Scatter`] for what stands in for the endpoint rule here.
pub static SCATTERS: &[Scatter] = &[
    // #417's four-number table, as the two pictures it was found by. The left cloud lies on a
    // diagonal and the right one lies flat, which is the whole of the argument that
    // R.C. 3317.0217(B)(4)(b)(i) reads enrolment rather than the wealth per pupil it names. Both
    // panels are the same 609 districts on the same predictor at one scale, so a difference
    // between the two clouds is a difference in the data and not in the frames.
    Scatter {
        key: "project/what-the-capacity-tiers-index-measures",
        owner: "crates/project",
        label: "What the capacity tier's index varies with: the total it reads, and the \
                per-pupil amount it names",
        subject: "district",
        compute: |i| {
            let points = project::size_terms::index_points(&i.panel);
            let fits = project::size_terms::what_the_index_measures(&i.panel);
            let (low, high) = points
                .iter()
                .map(|p| p.adm)
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                    (lo.min(v), hi.max(v))
                });
            // The frame's own ends, off the extreme districts by the margin the panels leave, and
            // the ends the fitted lines are evaluated at: a line is drawn across the frame it is
            // fitted in, not between the two districts that happen to bound it.
            let margin = (high / low).ln() * CLOUD_PAD;
            let (from, to) = ((low.ln() - margin).exp(), (high.ln() + margin).exp());
            // `least_squares` centres its design, so `coefficients[0]` is the mean of the outcome
            // and not the fitted value at zero — the line runs through the centroid, and reading
            // the intercept as an intercept would put it several log units off the cloud.
            let centre = points.iter().map(|p| p.adm.ln()).sum::<f64>() / points.len() as f64;
            let line = |fit: &dispersion::Regression,
                        slope_figure: &'static str,
                        r_squared_figure: &'static str| {
                let at = |adm: f64| {
                    (fit.coefficients[0] + fit.coefficients[1] * (adm.ln() - centre)).exp()
                };
                Fit {
                    from: (from, at(from)),
                    to: (to, at(to)),
                    slope: fit.coefficients[1],
                    r_squared: fit.r_squared,
                    slope_figure,
                    r_squared_figure,
                }
            };
            Cloud::at_one_scale(
                Axis {
                    label: "Base cost enrolled ADM",
                    unit: Unit::Pupils,
                    min: from,
                    max: to,
                    log: true,
                },
                points
                    .iter()
                    .map(|p| Point {
                        label: p.name.clone(),
                        x: p.adm,
                        ys: vec![p.weighted_wealth, p.wealth_per_pupil],
                    })
                    .collect(),
                vec![
                    PanelSpec {
                        label: "Total weighted wealth [A], which the tier compares",
                        axis: "Total weighted wealth",
                        unit: Unit::Dollars,
                        fit: line(
                            &fits.total,
                            "project/capacity-tier-index-on-enrolment",
                            "project/capacity-tier-index-on-enrolment-fit",
                        ),
                    },
                    PanelSpec {
                        label: "Weighted wealth per resident pupil [D], which it names",
                        axis: "Weighted wealth per resident pupil",
                        unit: Unit::Dollars,
                        fit: line(
                            &fits.per_pupil,
                            "project/wealth-per-pupil-index-on-enrolment-negative",
                            "project/wealth-per-pupil-index-on-enrolment-fit",
                        ),
                    },
                ],
            )
        },
    },
];

/// A cloud and the shape it came out with on this run.
pub struct ComputedCloud {
    /// The registry entry.
    pub scatter: &'static Scatter,
    /// What [`Scatter::compute`] returned.
    pub cloud: Cloud,
}

/// Run every cloud in [`SCATTERS`], in registry order.
#[must_use]
pub fn compute_all_scatters() -> Vec<ComputedCloud> {
    let inputs = Inputs::shared();
    SCATTERS
        .iter()
        .map(|scatter| ComputedCloud {
            scatter,
            cloud: (scatter.compute)(inputs),
        })
        .collect()
}
