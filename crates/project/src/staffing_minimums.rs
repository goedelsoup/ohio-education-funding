//! What R.C. 3317.011's staffing floors are worth, and who would be on the guarantee without them.
//!
//! [`foundation::minimums`] names the floors and prices one district's. This joins them to the
//! department's FY2027 panel and to the two things the corpus already knows about the guarantee:
//! [`crate::guarantee_origin`]'s partition of the districts it pays, and
//! [`crate::enrollment_decline`]'s cluster of the ones it pays for having fewer children.
//!
//! Issue #389 put the floors forward as an explanation of a shape the corpus had measured and not
//! explained: the statewide guarantee rate by ADM sextile is **non-monotone**, reaching 16.8% of
//! the smallest sextile and 69.3% of the third. The proposed mechanism was a cliff — "a district
//! below a minimum is funded as though it were larger; a district just above it is not".
//!
//! **Half of that is right and the mechanism is not.**
//!
//! # 1. The floors are worth $187.1m, and they are why the smallest sextile is not guaranteed
//!
//! Seven floors add **$189,474,883** to aggregate base cost and two ceilings take **$2,328,808**
//! back, so the net is **$187,146,075** — 1.59% of Ohio's $11.77bn. Strike them all out and 46
//! more districts fall onto the guarantee, 294 becoming 340, of which **30 are in the smallest
//! ADM sextile and 42 of the 46 are in the smallest two**:
//!
//! | ordered on base cost enrolled ADM | 1 | 2 | 3 | 4 | 5 | 6 |
//! |---|--:|--:|--:|--:|--:|--:|
//! | guaranteed as the section stands | **18** | 39 | 71 | 60 | 49 | 57 |
//! | guaranteed with the floors struck out | **48** | 51 | 74 | 61 | 49 | 57 |
//!
//! So the floors are doing exactly the job the issue credits them with, and they account for
//! **about half** of the non-monotonicity: the gap between the first and third sextile is 53
//! districts with them and 26 without.
//!
//! This file read the residual 26 as "not size", on the reasoning that base cost's only
//! size-dependent terms are the floors and the two banded salaries. The reasoning holds and the
//! conclusion does not: **base cost is not the only place the plan measures size**. The residual
//! is targeted assistance's capacity tier, which compares two whole-district totals and so reads
//! enrolment — see [`crate::size_terms`], where striking both out closes the gap to zero.
//!
//! The sextiles are ordered on base cost enrolled ADM, which is the count every threshold in the
//! section is against. `the_other_half_of_the_guarantee_and_the_two_things_it_pays_for.rs` orders
//! its own sextile row on current-year enrolled ADM and reads `17 / 41 / 70 / 61 / 48 / 57`; the
//! two denominators put a handful of districts in adjacent bands and neither row is wrong.
//!
//! # 2. There is no cliff, because a `max` is continuous
//!
//! Every floor is `max(ADM / ratio, n)`, which meets the bare ratio *exactly* at the threshold —
//! asserted as a property of the function in [`foundation::minimums`], not observed. What a
//! threshold changes is the slope. Below it the per-pupil uplift is a hyperbola falling to zero;
//! at it, zero; above it, zero.
//!
//! Measured on the level, the jump at every threshold is inside the grain of the department's
//! own two-decimal rounding of funded position counts — between the **13th and the 90th
//! percentile** of the jumps that rounding produces at ADM values with no threshold anywhere near
//! them, and the largest jump in a 13,514-step sweep is $6,054.61 at 4,398.76 ADM, which is not a
//! threshold. A district cannot tell a threshold from a rounding boundary by looking at its base
//! cost.
//!
//! # 3. What the floors actually do is open a wedge between the marginal and the average pupil
//!
//! This is the shape the cliff hypothesis was reaching for. [`wedge`] measures it at each
//! district's own enrolment, holding its own grade composition and its own building count, so
//! nothing here is a synthetic district:
//!
//! | ADM sextile | median ADM | marginal pupil ÷ average pupil |
//! |---|--:|--:|
//! | 1 | 533 | **0.642** |
//! | 2 | 871 | 0.747 |
//! | 3 | 1,192 | 0.874 |
//! | 4 | 1,621 | 0.980 |
//! | 5 | 2,482 | 1.009 |
//! | 6 | 5,246 | 0.994 |
//!
//! Monotone to parity and then flat: the wedge closes at about 2,000 ADM, which is where the
//! last floor reaching a material number of districts stops binding, and it does not reverse.
//! **313 of 608 districts have a marginal pupil worth less than 95% of their average pupil**, and
//! the most extreme is worth 0.169 of it.
//!
//! That is a claim about the base cost *function*, and it is not the same claim
//! [`crate::margin`] makes about the marginal pupil's price in a year. R.C. 3317.017(B)
//! multiplies a per-pupil residual by current-year enrolled ADM, so this year's aid is linear in
//! this year's count and `margin`'s $8,516.08 is right. The wedge arrives on the **other** clock,
//! phasing in over three years as the loss works through [`DistrictRecord::base_cost_adm`]'s
//! three-year average and the per-pupil amount itself moves. Two clocks, one pupil.
//!
//! The wedge is not the whole of what that second clock carries. Local capacity per pupil rises
//! on the same denominator, which is [`crate::guarantee_origin`]'s per-pupil term and a separate
//! mechanism working the other way. What is claimed here is the base cost side of it, which is
//! the side the floors are on.
//!
//! # 4. They are already a declining-enrolment adjustment, and a small one
//!
//! Because the floors are a `max` against a falling quotient, a district that shrinks past a
//! threshold *gains* uplift. The direction the issue expected is backwards: shrinking through a
//! threshold is the moment protection switches **on**.
//!
//! Since the guarantee's FY2020 anchor, 457 of 608 districts gained uplift this way, worth
//! $21.1m statewide. For [`crate::enrollment_decline`]'s 89-district cluster — the ones on the
//! guarantee for no reason but having fewer children — it is **$6,421,058, 5.91% of the
//! $108,568,921 the guarantee pays them**, at a median **$30.81** a pupil against the $464.06 a
//! pupil the guarantee pays. [`crate::decline_adjustment`] priced four candidate shapes for a declining-enrolment
//! adjustment without noticing that R.C. 3317.011 already contains one. It is in kind rather
//! than in cash, it is capped by size rather than by rate, and it is worth a seventeenth of the
//! instrument that is currently doing the job.
//!
//! # What this module holds fixed, and which way that cuts
//!
//! [`without_the_floors`] moves base cost aid and nothing else. Three channels are held:
//!
//! - **The base-cost-denominated categoricals.** Special education, English learners and
//!   career-technical are priced on the *statewide average* base cost per pupil
//!   ([`crate::panel::AVERAGE_BASE_COST_PER_PUPIL`]), which a statewide repeal would lower by
//!   **1.59%** — $8,257.55 to $8,126.22. Moving it would lower formula aid further and put *more* districts onto the
//!   guarantee, so **46 is a lower bound on the count** rather than an estimate either side of.
//! - **Local capacity.** R.C. 3317.017 computes it from valuation and income and never from base
//!   cost, so it is right for it not to move. Which is also why the floors are so expensive: for
//!   a district off the minimum state share, base cost aid is `base cost − capacity`, so the
//!   state pays the **whole** of the uplift and not its state share of it. $187.1m of base cost
//!   is $156.4m of aid, 83.6%.
//! - **The guarantee's own floor.** `[H2]` is a FY2020 figure produced under the Bridge formula
//!   and contains no part of R.C. 3317.011, which was enacted for FY2022. The floors are on one
//!   side of the `max` only, and that asymmetry is the whole mechanism of §1 above.
//!
//! # Where the numbers do not support a bunching story
//!
//! The issue asks how many districts sit just above a threshold. Within 5% of one:
//! 19 below and 14 above at special teachers, 23 and 11 at wellness, 23 and 14 at other
//! administrators, 21 and 15 at fiscal support, 5 and 4 at EMIS. Every one leans below — and so
//! does the enrolment distribution. At 148 control points between 400 and 6,000 ADM with no
//! threshold within 10%, the same ratio has a **median of 1.20** and 84 of them lean below too,
//! and each threshold's ratio sits between the 51st and the 80th percentile of that distribution.
//!
//! Which is what a threshold nobody can manipulate their enrolment against should look like, and
//! is worth saying because the counterfactual in §1 would read very differently if districts were
//! choosing which side to sit on.
//!
//! # Whether the floors are in the right places
//!
//! Not answerable here, and answerable for one of them elsewhere. [`crate::administrator_staffing`]
//! sets the (F)(3) floor against the one staff count a committed source carries, the District
//! Profile Report's FTE administrators; the other six floors have no count to be set against.

use std::collections::BTreeMap;

use edfund_core::{Adm, Dollars};
use foundation::minimums::{self, Ceiling, Minimum};
use foundation::{aggregate_base_cost, DistrictEnrollment, StatewideFactors};

use crate::guarantee_origin;
use crate::panel::{DistrictRecord, MINIMUM_STATE_SHARE};

/// How far a district's ADM may sit from a threshold and still count as beside it.
///
/// Used only by [`beside_a_threshold`], and stated as a constant because the count is meaningless
/// without it.
pub const BESIDE: f64 = 0.05;

/// One floor's reach across the panel.
#[derive(Debug, Clone, PartialEq)]
pub struct Census {
    /// The floor.
    pub minimum: Minimum,
    /// Districts whose funded count it sets.
    pub binds: usize,
    /// What it adds to their aggregate base cost.
    pub direct: Dollars,
    /// What it adds a second time through another division's input.
    ///
    /// Non-zero for [`Minimum::OtherAdministrators`] alone: R.C. 3317.011(F)(6)(a) reads the
    /// floored administrator count, so the administrator floor raises leadership support too.
    /// Attributed here rather than to [`Minimum::LeadershipSupport`], which cannot bind.
    pub induced: Dollars,
}

impl Census {
    /// Everything this floor is worth.
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.direct + self.induced
    }
}

/// Every floor's reach, in statutory order.
#[must_use]
pub fn census(panel: &[DistrictRecord]) -> Vec<Census> {
    let factors = StatewideFactors::fy2027();
    Minimum::ALL
        .iter()
        .map(|minimum| Census {
            minimum: *minimum,
            binds: panel
                .iter()
                .filter(|record| minimum.binds(&record.enrollment))
                .count(),
            direct: panel
                .iter()
                .map(|record| minimum.uplift(&record.enrollment, &factors))
                .sum(),
            induced: if *minimum == Minimum::OtherAdministrators {
                panel
                    .iter()
                    .map(|record| minimums::uplift(&record.enrollment, &factors).induced)
                    .sum()
            } else {
                0.0
            },
        })
        .collect()
}

/// What the two ceilings take back, and from how many districts.
#[must_use]
pub fn ceilings(panel: &[DistrictRecord]) -> Vec<(Ceiling, usize, Dollars)> {
    let factors = StatewideFactors::fy2027();
    Ceiling::ALL
        .iter()
        .map(|ceiling| {
            (
                *ceiling,
                panel
                    .iter()
                    .filter(|record| ceiling.binds(&record.enrollment))
                    .count(),
                panel
                    .iter()
                    .map(|record| ceiling.reduction(&record.enrollment, &factors))
                    .sum(),
            )
        })
        .collect()
}

/// Districts within [`BESIDE`] of a threshold, below it and above it.
///
/// The counts the bunching question needs, for the five floors that have an ADM threshold at all.
#[must_use]
pub fn beside_a_threshold(panel: &[DistrictRecord]) -> Vec<(Minimum, usize, usize)> {
    Minimum::ALL
        .iter()
        .filter_map(|minimum| {
            let threshold = minimum.threshold()?;
            let count = |lo: f64, hi: f64| {
                panel
                    .iter()
                    .filter(|record| {
                        let adm = record.base_cost_adm();
                        adm >= lo && adm < hi
                    })
                    .count()
            };
            Some((
                *minimum,
                count(threshold * (1.0 - BESIDE), threshold),
                count(threshold, threshold * (1.0 + BESIDE)),
            ))
        })
        .collect()
}

/// One district under a section with every staffing floor struck out.
#[derive(Debug, Clone, PartialEq)]
pub struct WithoutTheFloors {
    /// The district, keyed the only way district figures may be keyed.
    pub irn: String,
    /// Its name, for a table a reader can check.
    pub name: String,
    /// Base cost enrolled ADM, the count every threshold is against.
    pub adm: Adm,
    /// What the floors add to its aggregate base cost.
    pub uplift: Dollars,
    /// What striking them out takes off its base cost aid — a negative number.
    pub aid_delta: Dollars,
    /// Whether the guarantee pays it as the section stands.
    pub guaranteed: bool,
    /// Whether the guarantee would pay it with the floors struck out.
    pub guaranteed_without: bool,
}

impl WithoutTheFloors {
    /// Whether striking the floors out is what puts this district on the guarantee.
    #[must_use]
    pub fn newly_guaranteed(&self) -> bool {
        !self.guaranteed && self.guaranteed_without
    }
}

/// Run the whole panel with the floors struck out of R.C. 3317.011.
///
/// # The arithmetic, which is shorter than it looks
///
/// R.C. 3317.017(B) pays `max(base cost per pupil − local capacity per pupil, minimum share ×
/// base cost per pupil) × current-year enrolled ADM`. Lowering base cost per pupil by the
/// floors' worth therefore takes the **whole** uplift off a district paid on the residual, and a
/// tenth of it off a district paid on the floor. Nothing else in the formula reads base cost per
/// pupil except the categoricals, which are held — see the module note.
///
/// Districts with no positive enrolment on either denominator are skipped: there is no per-pupil
/// quantity to move.
#[must_use]
pub fn without_the_floors(panel: &[DistrictRecord]) -> Vec<WithoutTheFloors> {
    let factors = StatewideFactors::fy2027();
    panel
        .iter()
        .filter_map(|record| {
            let base_cost_adm = record.base_cost_adm();
            if base_cost_adm <= 0.0 || record.current_year_adm <= 0.0 {
                return None;
            }
            let uplift = minimums::uplift(&record.enrollment, &factors).total();
            let per_pupil = record.base_cost_per_pupil;
            let without_per_pupil = per_pupil - uplift / base_cost_adm;
            let aid_per_pupil = record.base_cost_state_share / record.current_year_adm;

            let without_aid_per_pupil = if record.at_minimum_state_share() {
                // Already on the floor, and lowering base cost cannot lift it off: capacity is
                // unchanged and the residual it lost to was already the smaller number.
                without_per_pupil * MINIMUM_STATE_SHARE
            } else {
                let capacity = per_pupil - aid_per_pupil;
                (without_per_pupil - capacity).max(without_per_pupil * MINIMUM_STATE_SHARE)
            };

            let aid_delta = (without_aid_per_pupil - aid_per_pupil) * record.current_year_adm;
            Some(WithoutTheFloors {
                irn: record.irn.clone(),
                name: record.name.clone(),
                adm: base_cost_adm,
                uplift,
                aid_delta,
                guaranteed: record.on_guarantee(),
                guaranteed_without: record.core_foundation_funding + aid_delta
                    < record.guarantee_floor() - crate::hold_harmless::CENT,
            })
        })
        .collect()
}

/// The average pupil's base cost against the marginal pupil's, at one enrolment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wedge {
    /// The enrolment the pair is measured at.
    pub adm: Adm,
    /// Aggregate base cost divided by that enrolment.
    pub average: Dollars,
    /// What one more pupil adds to aggregate base cost.
    pub marginal: Dollars,
}

impl Wedge {
    /// The marginal pupil as a fraction of the average one. 1.0 is a formula with no floors left.
    #[must_use]
    pub fn ratio(&self) -> f64 {
        if self.average <= 0.0 {
            return 0.0;
        }
        self.marginal / self.average
    }
}

/// How far apart the marginal and average pupil are, for one district's composition at a stated
/// enrolment.
///
/// # Why the difference is taken over a window and not over one pupil
///
/// The department rounds every funded position count to two decimals, which puts steps of up to
/// about $1,000 into the aggregate at arbitrary enrolments. Over a single pupil those steps are
/// larger than the quantity being measured. [`HALF_WINDOW`] pupils either side averages them out
/// without reaching across more than one threshold.
///
/// # Panics
///
/// If `adm` is not positive, or is smaller than the window — there is no composition to hold at
/// zero pupils, and no district below it to difference against.
#[must_use]
pub fn wedge(enrollment: &DistrictEnrollment, adm: Adm, factors: &StatewideFactors) -> Wedge {
    assert!(
        adm > HALF_WINDOW,
        "no district to difference below {adm} ADM"
    );
    let at = |count: Adm| aggregate_base_cost(&enrollment.scaled_to(count), factors).aggregate;
    Wedge {
        adm,
        average: at(adm) / adm,
        marginal: (at(adm + HALF_WINDOW) - at(adm - HALF_WINDOW)) / (2.0 * HALF_WINDOW),
    }
}

/// Pupils either side of the point [`wedge`] differences over.
pub const HALF_WINDOW: f64 = 25.0;

/// What each district's floors are worth now that was not worth anything at the guarantee's
/// anchor year.
///
/// Positive where a district has shrunk past a threshold since FY2020 and the floor has switched
/// on behind it. Keyed by IRN, and absent for a district
/// [`guarantee_origin::enrollment_index`] cannot reach — one, Buckeye Local.
///
/// The FY2020 enrolment is the current departmental count divided by that index, with grade
/// composition and the building count held: a district that has lost a fifth of its pupils since
/// FY2020 did not close a fifth of a building, and holding the count is what keeps the building
/// support floor visible.
#[must_use]
pub fn acquired_since_anchor(panel: &[DistrictRecord]) -> BTreeMap<String, Dollars> {
    let factors = StatewideFactors::fy2027();
    let index = guarantee_origin::enrollment_index(panel);
    panel
        .iter()
        .filter_map(|record| {
            let ratio = index.get(&record.irn)?;
            let [_, _, latest] = record.adm_history;
            let anchor = latest / ratio;
            if anchor <= 0.0 || record.base_cost_adm() <= 0.0 {
                return None;
            }
            let now = minimums::uplift(&record.enrollment, &factors).total();
            let then = minimums::uplift(&record.enrollment.scaled_to(anchor), &factors).total();
            Some((record.irn.clone(), now - then))
        })
        .collect()
}
