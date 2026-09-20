//! What a linear projection holds per pupil, and what holding it is worth.
//!
//! [`crate::policy::apply`] computes base cost aid at a substituted enrolment by scaling the
//! department's published state share:
//!
//! ```text
//! base cost aid = base cost state share x (projected ADM / modelled ADM)
//! ```
//!
//! which is exactly linear in the pupil count. That is exact where every deterministic result in
//! this crate is computed — at modelled enrolment — and [`crate::report::forecast`] is the one
//! caller for which it is not, because it substitutes an enrolment the formula's other inputs
//! were never computed at. Opened as [#407].
//!
//! # The two terms the linear path freezes, and they point opposite ways
//!
//! Write the state's share of base cost out and there are two per-pupil quantities in it:
//!
//! ```text
//! base cost aid = max(base cost per pupil - capacity per pupil, 10% x base cost per pupil)
//!                 x current-year enrolled ADM
//! ```
//!
//! Scaling the *product* by the pupil count holds both factors at their FY2027 values. Neither is
//! constant in the count, and the two move in opposite directions:
//!
//! | per pupil, district shrinking | why it moves | which way the linear path errs |
//! |---|---|---|
//! | **base cost** rises | R.C. 3317.011's staffing floors are a `max` and do not follow a roll down — [`crate::staffing_minimums`] measures a marginal pupil worth 0.642 of the average one in the smallest sextile | linear **understates** aid |
//! | **local capacity** rises | R.C. 3317.017's wealth blend is a dollar amount over a shrinking denominator — [`crate::capacity_denominator`] proves the numerator is a function of wealth alone | linear **overstates** aid |
//!
//! Aid is the *difference* of the two, so the errors do not cancel, they compete. [`restated`]
//! recomputes each from its own inputs at the projected count — aggregate base cost from
//! [`foundation::aggregate_base_cost`] on the enrolment [`foundation::DistrictEnrollment::scaled_to`] scales,
//! and capacity from the published charge over the projected denominator — and [`Terms`] runs
//! either one alone.
//!
//! # What it is worth: $31.7m, two thirds of the effect being reported, and a 43% wider band
//!
//! At FY2032, current law, the method the feed runs:
//!
//! | | statewide realized aid |
//! |---|--:|
//! | at observed enrolment | $7,281.2m |
//! | projected, linear — what the feed publishes | $7,233.1m |
//! | projected, base cost per pupil recomputed alone | $7,244.4m |
//! | projected, capacity per pupil recomputed alone | $7,192.1m |
//! | **projected, both** | **$7,201.5m** |
//!
//! So the correction is **−$31.7m, 0.44% of projected aid** — and 66% of the −$48.1m enrollment
//! effect the forecast exists to report. The capacity term is **3.6 times** the base cost term
//! and carries the sign; #407 reached for the base cost one, which #389 had just measured, and it
//! is the smaller of the two and points the other way.
//!
//! **The interval moves more than the point does.** Because both terms are convex in the count,
//! the correction is larger at the low end of the enrollment band than at the high end, so the
//! aid band widens rather than shifting: $6,957.9m–$7,581.9m becomes $6,853.4m–$7,739.6m, and the
//! `(+/-4.3%)` this corpus publishes for FY2032 becomes `(+/-6.2%)`. `forecast`'s own note that
//! the enrollment band's low end is the aid band's low end survives — aid still falls with the
//! count, and no end is reordered.
//!
//! # The averaged count is a cushion, and a damped projection spends it
//!
//! R.C. 3317.011 funds base cost on `max(mean of three years, the current year)`, so a district
//! in decline is funded on a count **above** the children it teaches: statewide the ratio of base
//! cost ADM to current-year enrolled ADM is **1.0165** in the FY2027 model. At FY2032 the shipped
//! projection puts it at **1.0000**. The cushion is gone, because the damping flattens the series
//! and a flat series has no three-year average to sit above.
//!
//! So base cost ADM falls **2.86%** over the horizon while enrolled ADM falls **1.26%**, and more
//! than half of the first number is the cushion closing rather than pupils leaving. Carry the base
//! cost count one-for-one with enrolled ADM instead and the whole correction is **−$13.3m** rather
//! than −$31.7m: **58% of it is the cushion**.
//!
//! That is a property of the projection and not of the statute — the corpus already records that
//! at the published horizon the trend is simply gone, statewide ADM moving 12 pupils between
//! FY2032 and FY2036 — and it cuts both ways. A district really would lose the cushion if its
//! decline really did flatten. What cannot be said is that the statute absorbs the error: at this
//! horizon the averaging more than doubles it.
//!
//! # The sign is exactly the difference of the two terms, for every district it can reach
//!
//! For a district on formula and off the minimum state share, the sign of the error is the sign
//! of `Δ base cost per pupil − Δ capacity per pupil` for **252 of 252** districts at FY2032 —
//! not a tendency, an identity, because those two terms are the whole of the mechanism. Which
//! is why the population splits the way it does rather than by size: **233 districts are
//! overstated and 65 understated**, the understated ones being those the wedge opens faster for
//! than the denominator does, plus the growing districts, for which every sign above reverses.
//!
//! # Who it can reach, which is exactly the districts the guarantee does not pay
//!
//! The guarantee is a `max`, so a guaranteed district is inert to all of this. At FY2032 the
//! linear path puts **312 districts** on the guarantee and leaves **297** on formula — and all
//! 297 of them move, against **one** of the 312, the single district the correction lifts off the
//! guarantee. The bound is not approximate: the population is the formula population.
//!
//! The correction then sends **15 districts the other way**, onto the guarantee, so 312 becomes
//! 326 — which is [`crate::staffing_minimums`]'s observation that a falling formula side is
//! absorbed there, arriving from a second direction. **120 districts move by more than 1% of
//! their aid and 10 by more than 5%.**
//!
//! # Why `forecast` is not corrected, which is not the same as the approximation being fine
//!
//! The linear path holds the district's **wealth per pupil** fixed: scaling `base cost per pupil −
//! capacity per pupil` by the count is the arithmetic of a tax base that shrinks exactly as fast
//! as its children leave. The corrected path holds its **wealth** fixed, in nominal dollars, for
//! as long as the horizon runs. Neither is a forecast of an Ohio tax base, and this crate has
//! none to offer — [`crate`]'s own note says valuation cannot be projected from the committed
//! data, and 60% of the capacity blend is valuation.
//!
//! So the two runs are not a wrong answer and a right one. They are the ends of the interval a
//! projected local share actually sits in, and $31.7m is its width. Publishing the width is worth
//! more than picking an end, and picking one would move every band in the feed on the strength of
//! an assumption about property values that nothing here has measured.
//!
//! The base cost half is not in that condition — R.C. 3317.011 says what aggregate base cost is at
//! any enrolment, and [`foundation`] computes it — but it is the smaller term, and correcting it
//! alone would move the published figure in the direction the larger term says is wrong.
//!
//! [#407]: https://github.com/goedelsoup/ohio-education-funding/issues/407

use std::borrow::Cow;

use edfund_core::{Adm, Dollars, FiscalYear};
use foundation::{aggregate_base_cost, StatewideFactors};

use crate::panel::{DistrictRecord, MINIMUM_STATE_SHARE};
use crate::policy::{apply, Policy, Statewide};
use crate::report::{forecast_with, projected_series, window_ending_at, EnrollmentEffect};
use crate::series::{Basis, Method, Prior};

/// Base cost enrolled ADM from a three-year window of enrolled ADM, newest last.
///
/// R.C. 3317.011's count, and an identity rather than a model: run against the FY2024, FY2025 and
/// FY2026 columns the panel carries, this reproduces the department's published base cost
/// enrolled ADM for **all 609 districts**, worst residual 0.000067 of a pupil — which is the
/// grain of the two-decimal ADM the workbook rounds to. The `max` binds for the 105 growing
/// districts and the mean binds for the rest.
///
/// It is stated here rather than in [`foundation`] because it is a statement about a *series*,
/// and `foundation` is given one year's enrolment and asked what it costs.
#[must_use]
pub fn base_cost_adm(window: [Adm; 3]) -> Adm {
    ((window[0] + window[1] + window[2]) / 3.0).max(window[2])
}

/// Which of the two per-pupil terms a restatement recomputes.
///
/// Separable because they are separate mechanisms in separate sections, and worth separating
/// because they carry opposite signs: reporting only their sum would hide that the smaller one
/// is the one #407 named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Terms {
    /// Recompute base cost per pupil from R.C. 3317.011 at the projected count.
    pub base_cost: bool,
    /// Recompute local capacity per pupil from the published charge over the projected count.
    pub capacity: bool,
}

impl Terms {
    /// Both, which is the corrected run.
    pub const BOTH: Self = Self {
        base_cost: true,
        capacity: true,
    };
    /// The staffing floors alone — the term #407 and #389 name.
    pub const BASE_COST: Self = Self {
        base_cost: true,
        capacity: false,
    };
    /// The wealth denominator alone — the term that carries the sign.
    pub const CAPACITY: Self = Self {
        base_cost: false,
        capacity: true,
    };
    /// Neither, which must reproduce [`crate::report::forecast`] exactly.
    pub const NEITHER: Self = Self {
        base_cost: false,
        capacity: false,
    };
}

/// One district restated at a projected enrolment, for [`crate::report::forecast_with`].
///
/// `window` is the three-year run of projected enrolled ADM ending at the year being forecast;
/// [`base_cost_adm`] takes the averaged count off it and `window[2]` is the current-year one.
///
/// # What is moved, and what deliberately is not
///
/// Two fields: `base_cost_per_pupil`, and `base_cost_state_share` **with the same movement added
/// to `core_foundation_funding` beside it**. The second half is not an implementation detail —
/// [`crate::policy::apply`] derives the categoricals as `core foundation funding − base cost
/// state share`, so moving the share alone moves nothing at all: the two cancel and the outcome
/// is identical to the cent. That cancellation is what makes the published split of one
/// published total inert at current law, and it is the trap this function exists on the other
/// side of.
///
/// Everything else is left exactly as the linear path has it. The categoricals still scale with
/// the count, `[H2]` and `[K]`'s FY2021 base are published figures from years that have already
/// happened, and local capacity's *numerator* is held for the reason the module note gives.
///
/// # Calibration, and the one district that needs it
///
/// The reconstruction is fitted to the published figure at the modelled count and applied as a
/// ratio, so a restatement at the modelled enrolment is the department's own number to the cent
/// and `Terms::NEITHER` is the identity. The ratio is 1.000000 at the median and within 1e-4 for
/// **608 of 609 districts**; the exception is **Akron City at +0.27%**, whose published base cost
/// state share divides out to 18,892.45 pupils — `[a] Enrolled ADM` — rather than the 18,842.45
/// of `[b3]` that R.C. 3317.017(B) names and every other district uses. Akron is also the only
/// district in the model whose two counts differ at all, which is what
/// [`DistrictRecord::categorical_enrolled_adm`] already names it for.
///
/// Returns the record unchanged where there is nothing to restate: no published capacity, no
/// positive enrolment on either denominator, or a reconstruction that is not positive.
#[must_use]
pub fn restated<'a>(
    record: &'a DistrictRecord,
    window: [Adm; 3],
    terms: Terms,
    factors: &StatewideFactors,
) -> Cow<'a, DistrictRecord> {
    let modelled = record.base_cost_adm();
    let Some(capacity_per_pupil) = record.published_capacity_per_pupil else {
        return Cow::Borrowed(record);
    };
    if modelled <= 0.0 || record.current_year_adm <= 0.0 {
        return Cow::Borrowed(record);
    }
    let projected = base_cost_adm(window);
    if projected <= 0.0 {
        return Cow::Borrowed(record);
    }

    // The wealth blend before it was divided by anything, which is what `capacity_denominator`
    // establishes is the quantity R.C. 3317.017 actually charges.
    let charge = capacity_per_pupil * modelled;
    let base_cost = |count: Adm| aggregate_base_cost(&record.enrollment.scaled_to(count), factors);
    let share_per_pupil = |count: Adm| {
        let base = if terms.base_cost {
            base_cost(count).aggregate / count
        } else {
            base_cost(modelled).aggregate / modelled
        };
        let capacity = if terms.capacity {
            charge / count
        } else {
            capacity_per_pupil
        };
        (base - capacity).max(base * MINIMUM_STATE_SHARE)
    };

    let at_modelled = share_per_pupil(modelled);
    if at_modelled <= 0.0 {
        return Cow::Borrowed(record);
    }
    let published_per_pupil = record.base_cost_state_share / record.current_year_adm;
    let share =
        share_per_pupil(projected) * (published_per_pupil / at_modelled) * record.current_year_adm;

    let mut out = record.clone();
    if terms.base_cost {
        out.base_cost_per_pupil = base_cost(projected).aggregate / projected;
    }
    out.core_foundation_funding += share - record.base_cost_state_share;
    out.base_cost_state_share = share;
    Cow::Owned(out)
}

/// [`crate::report::forecast`] with both per-pupil terms recomputed at the projected count.
///
/// `Terms::NEITHER` reproduces it exactly; the other three are the rows of the module's table.
#[must_use]
pub fn forecast(
    panel: &[DistrictRecord],
    policy: &Policy,
    through: FiscalYear,
    method: Method,
    prior: Prior,
    terms: Terms,
) -> EnrollmentEffect {
    let factors = StatewideFactors::fy2027();
    forecast_with(
        panel,
        policy,
        through,
        method,
        prior,
        move |record, window| restated(record, window, terms, &factors),
    )
}

/// What the restatement moves for one district, and the two terms that moved it.
#[derive(Debug, Clone, PartialEq)]
pub struct Movement {
    /// The join key, because 609 districts share about 580 names.
    pub irn: String,
    /// District name as published.
    pub name: String,
    /// Base cost enrolled ADM as modelled, which is what the wedge is open or shut at.
    pub adm: Adm,
    /// Base cost enrolled ADM at the projected year.
    pub projected_adm: Adm,
    /// Realized aid on the linear path.
    pub linear: Dollars,
    /// Realized aid with both terms recomputed.
    pub restated: Dollars,
    /// Whether the linear path puts this district on the guarantee, where it is inert.
    pub on_guarantee: bool,
    /// And whether the restated one does, which is how the correction is absorbed.
    pub on_guarantee_restated: bool,
    /// What base cost per pupil does between the two counts.
    pub base_cost_move: Dollars,
    /// What local capacity per pupil does between them.
    pub capacity_move: Dollars,
}

impl Movement {
    /// The correction, as a share of what the linear path pays.
    #[must_use]
    pub fn relative(&self) -> f64 {
        if self.linear == 0.0 {
            return 0.0;
        }
        (self.restated - self.linear) / self.linear
    }

    /// Whether the two terms predict the direction this district actually moved.
    ///
    /// Exact for every district the two terms can reach — see the module note. A district on the
    /// guarantee at either end, or one whose share is set by the minimum state share, is not one
    /// of them: for the first the `max` decides and for the second only base cost is in the
    /// answer at all.
    #[must_use]
    pub fn predicted(&self) -> bool {
        self.base_cost_move - self.capacity_move > 0.0
    }
}

/// Every district's movement at one horizon, under current law.
#[must_use]
pub fn movements(
    panel: &[DistrictRecord],
    policy: &Policy,
    through: FiscalYear,
    method: Method,
    prior: Prior,
) -> Vec<Movement> {
    let factors = StatewideFactors::fy2027();
    let statewide = Statewide::under(panel, policy);
    panel
        .iter()
        .filter_map(|record| {
            // The same two calls `forecast_with` makes, so a row cannot drift from the total it
            // is meant to explain. A district the series cannot carry forward is absent rather
            // than recorded as unmoved: it has no projected enrolment to be restated at.
            let series = projected_series(record, through, method, prior);
            series
                .iter()
                .any(|p| p.fiscal_year == through && p.basis == Basis::Projected)
                .then(|| {
                    let window = window_ending_at(&series, through, |p| p.point);
                    let restatement = restated(record, window, Terms::BOTH, &factors);
                    let linear = apply(record, policy, &statewide, window[2]);
                    let moved = apply(&restatement, policy, &statewide, window[2]);
                    let modelled = record.base_cost_adm();
                    let projected = base_cost_adm(window);
                    let capacity = record.published_capacity_per_pupil.unwrap_or(0.0);
                    let per_pupil = |count: Adm| {
                        aggregate_base_cost(&record.enrollment.scaled_to(count), &factors).aggregate
                            / count
                    };
                    let measurable = modelled > 0.0 && projected > 0.0;
                    Movement {
                        irn: record.irn.clone(),
                        name: record.name.clone(),
                        adm: modelled,
                        projected_adm: projected,
                        linear: linear.realized_aid,
                        restated: moved.realized_aid,
                        on_guarantee: linear.on_guarantee,
                        on_guarantee_restated: moved.on_guarantee,
                        base_cost_move: if measurable {
                            per_pupil(projected) - per_pupil(modelled)
                        } else {
                            0.0
                        },
                        capacity_move: if measurable {
                            capacity * modelled / projected - capacity
                        } else {
                            0.0
                        },
                    }
                })
        })
        .collect()
}
