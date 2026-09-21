//! What a rolling anchor for the guarantee costs, and why it is a larger instrument than the
//! fixed one rather than a cheaper one.
//!
//! [`crate::enrollment_decline`] found that 50 of 89 districts crossed onto the guarantee between
//! FY2026 and FY2027, and observed that *"under a prior-year anchor most of them would never have
//! crossed at all, because the anchor would have moved with them"*. That is the question this
//! module answers. The anchor does move with them, and the answer is the other way round.
//!
//! # A prior-year anchor is a ratchet
//!
//! `realized = max(formula, anchor)`, so last year's realized aid is at or above last year's
//! formula amount by construction. Set this year's anchor to it and aid can never fall — for
//! anybody, for any reason, in any year. A **fixed** anchor catches only districts that have
//! fallen below a level chosen once in FY2020. A **rolling** one catches every district whose
//! formula amount falls at all.
//!
//! Ohio's formula amount falls for most districts most years: the cost side is frozen at FY2022
//! and FY2024 statewide averages while the capacity side rolls a three-tax-year window forward, so
//! state share fell in 540 of 609 districts in one year ([`crate::prior_model`]). Against the
//! FY2027 model:
//!
//! | anchor | `[I]` statewide | districts held |
//! |---|--:|--:|
//! | the FY2020 funding base, as enacted | $878,954,627 | 294 |
//! | the district's own FY2026 realized aid | **$1,025,415,233** | **455** |
//! | the mean of its FY2025 and FY2026 realized aid | $1,082,158,897 | 446 |
//! | its FY2026 `[Hb]`, the comparable quantity | $540,591,743 | **540** |
//!
//! The last row is the one to read twice. `[Hb]` is the formula's own output before the phase-in,
//! so it is the figure a FY2026-to-FY2027 comparison should use — and against it a rolling anchor
//! is **cheaper in dollars and wider in population than either**: it holds 540 of 609 districts
//! for $540.6m. A ratchet on a falling quantity catches almost everybody for a little, where the
//! fixed anchor catches half as many for twice as much. That is the trade the whole question is
//! about, and it is visible before any projection is run.
//!
//! # And for the 89 it is the same number to the cent
//!
//! Every member of the enrollment cluster is already on the floor in FY2027, so its FY2027
//! realized aid *is* its FY2020 base, and a ratchet started there holds it at exactly the same
//! level. Walked to FY2032 under the shipped projection, the two anchors give the cluster
//! **$144,560,557.55** of guarantee, **89** districts held, and the same total state support to
//! the cent. The crossing finding does not translate, for a reason that is visible in the
//! algebra rather than in the data: a `max` only ever moves an anchor **up**.
//!
//! # The shape that does what the question wanted is a decay
//!
//! What #390 was reaching for is an anchor that follows a district *down*, and no `max` against a
//! past aid level does that. The rule that does is a bound on the fall:
//!
//! ```text
//! realized = max( formula, factor x last year's realized )
//! ```
//!
//! — "no district's foundation aid falls more than `1 − factor` in a year". It is date-free, it
//! reaches a district that crosses in any year, it moves with the district in the direction that
//! matters, and it extinguishes itself for a district whose enrollment settles. Walked to FY2032:
//!
//! | rule | `[I]` statewide | held | `[I]` to the 89 | of the 89 |
//! |---|--:|--:|--:|--:|
//! | fixed FY2020 base | $932,058,576 | 312 | $144,560,558 | 89 |
//! | prior-year ratchet | $989,357,267 | 565 | $144,560,558 | 89 |
//! | fall capped at 1% a year | $784,791,925 | 277 | $85,028,780 | 71 |
//! | fall capped at 2% a year | $665,088,657 | 246 | $45,151,279 | 47 |
//! | fall capped at 3% a year | $572,939,686 | 218 | $27,264,597 | 26 |
//! | fall capped at 5% a year | $423,983,113 | 185 | $12,027,818 | 6 |
//!
//! # It is a different population, not a weaker floor
//!
//! A decay is anchored on **current** aid, so for a district well above its FY2020 base it is more
//! generous than the fixed anchor and not less. At FY2036 on the undamped projection, a 1% cap
//! holds **more** districts than the enacted rule while paying less: **469 against 388**, at
//! $1,074,962,630 against $1,295,115,297. Fewer dollars, spread over more districts, and the
//! districts are not the same ones — statewide total state support is $7.7m **higher** under the
//! 1% cap than under the enacted anchor, which is why [`absorbed`] reads above 1.0 there. Whether
//! a decay is cheaper at all depends entirely on the rate; at 2% it is cheaper on both counts.
//!
//! # `[K]` absorbs it, exactly as it absorbed everything in #400
//!
//! [`crate::decline_adjustment`] found that the guarantee is this cluster's nominal payer and
//! Section 265.225's `[K]` its real one. The same is true of the anchor's shape. Cutting the
//! cluster's guarantee from $144,560,558 to $45,151,279 with a 2% cap takes **$3,949,559** of
//! total state support — **96.0% absorbed** ([`absorbed`]). Statewide the share is 76.3%.
//!
//! # Which leaves the one quantity a shape was supposed to move where it was
//!
//! The do-nothing case in #400 is not a shortfall but a provenance: at FY2036 **26.9%** of what
//! the cluster receives rides on a hold-harmless. Every anchor shape priced here leaves that
//! share where it is, because the money it takes off `[I]` arrives on `[K]`:
//!
//! | the 89 at FY2036, undamped | `[I]` | `[K]` | held share |
//! |---|--:|--:|--:|
//! | fixed FY2020 base | $335.7m | $53.8m | **26.94%** |
//! | fall capped at 1% | $223.1m | $163.7m | **26.81%** |
//! | fall capped at 2% | $127.4m | $259.5m | **26.81%** |
//! | fall capped at 5% | $16.9m | $370.0m | **26.81%** |
//!
//! **Changing the anchor changes which hold-harmless pays, not whether one does** — and it moves
//! the money from the codified device, R.C. 3317.019, to the uncodified one that is renewed a
//! biennium at a time. That is the finding, and it is an argument against re-shaping the anchor
//! on its own rather than for any particular shape.
//!
//! # What the three years of comparable aid were actually needed for
//!
//! #390 recorded the work as data-constrained: testing a rolling anchor "needs several
//! consecutive years of *comparable* aid per district, and the corpus has about three."
//!
//! Both halves are true and they do not compose. A rolling anchor is **recursive** — the anchor
//! in a year is a function of the aid the model computes in the year before it — so pricing one
//! forward needs no historical aid at all, only the FY2027 model and the projection already
//! built. Three years is what a **backtest** needs, and a backtest is not what a price is.
//!
//! And the three years will not support one. [`comparability`] measures them: the FY2020 funding
//! base is identical across the FY2025 payment report, the FY2026 model and the FY2027 panel for
//! **all 609** districts, which is the check that they are the same districts on the same lines.
//! `[Hb]`, the formula's own output, then falls a median **5.2%** and **6.0%** across the two
//! legs, in 549 and 540 of 609 districts, and the two legs correlate at only **0.545** across
//! districts. That is the regime moving rather than the districts, which is what #390 warned it
//! would be and what `a-break-in-a-series-is-a-population-change-first` is about. The instinct
//! was right about the data and wrong about what the data was for.
//!
//! # No lever, again
//!
//! [`Anchor::needs_a_lever`] is false three times, for the reason
//! [`crate::decline_adjustment`] and [`crate::refresh`] give: an anchor rule is a per-district
//! function of a per-district history, and [`crate::policy::Policy`]'s fields are scalars. A walk
//! is a module.
//!
//! # Not priced here
//!
//! **Anything the decay does to districts outside the cluster.** 565 districts are held under a
//! ratchet and 246 under a 2% cap; who they are, and by wealth or poverty, is the question
//! #400 left open about its own shapes and it is still open about these. The statewide sign
//! flipping at a 1% cap — more districts, more total support, less guarantee — is the visible
//! edge of it.
//!
//! **A backtest.** See above: the three observed years cannot support one, and saying so is the
//! result rather than a caveat on it.
//!
//! **Community and STEM schools.** Every figure is over the 609 districts the FY2027 model
//! carries, as in #383 and #400.

use std::collections::BTreeMap;

use edfund_core::{Adm, Dollars, FiscalYear};

use crate::hold_harmless::transition_supplement_under;
use crate::panel::DistrictRecord;
use crate::policy::{apply, GuaranteeRule, Policy, Statewide};
use crate::series::{project, Method, Prior};

/// The first year a walk computes. FY2027 is the modelled year and is where a walk starts from.
pub const FIRST_PROJECTED_YEAR: u16 = 2028;

/// Smallest top-up counted as the guarantee binding: half a cent, as [`crate::refresh::MOVED`].
pub const HELD: Dollars = 0.005;

/// What holds a district up, when the formula computes less than it did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    /// R.C. 3317.019 as enacted: the FY2020 funding base, net of the open-enrolment clawback.
    ///
    /// Fixed in the year it names and receding from every year after it.
    Fixed,
    /// The district's own realized aid in the year before.
    ///
    /// The obvious rolling alternative, and a **ratchet**: realized aid is at or above formula aid
    /// by construction, so an anchor set to it means aid never falls, for anybody, ever.
    PriorYear,
    /// The same, multiplied down — a bound on how far aid may fall in a year.
    ///
    /// `factor` is what survives: 0.98 is "no district's foundation aid falls more than 2% in a
    /// year". This is the only one of the three that follows a district **down**, which is what
    /// [`Self::PriorYear`] was expected to do and does not.
    Decayed {
        /// The share of last year's realized aid the floor stands at. In `(0, 1]`.
        factor: f64,
    },
}

impl Anchor {
    /// The floor this rule puts under a district, given what it was paid last year.
    #[must_use]
    pub fn of(self, record: &DistrictRecord, prior_realized_aid: Dollars) -> Dollars {
        match self {
            Self::Fixed => record.guarantee_floor(),
            Self::PriorYear => prior_realized_aid,
            Self::Decayed { factor } => factor * prior_realized_aid,
        }
    }

    /// Whether aid can ever fall under this rule.
    ///
    /// The property the whole question turns on. Only [`Self::PriorYear`] is a ratchet; the fixed
    /// anchor is not one because a district above it may fall to it, and a decay is not one
    /// because the floor itself falls.
    #[must_use]
    pub fn is_a_ratchet(self) -> bool {
        matches!(self, Self::PriorYear)
    }

    /// Whether pricing the rule needs a new [`Policy`] field.
    ///
    /// False for all three, as in [`crate::decline_adjustment::Shape::needs_a_lever`]. An anchor
    /// rule is a per-district function of a per-district history and a `Policy` field is a scalar;
    /// a walk is a module.
    #[must_use]
    pub const fn needs_a_lever(self) -> bool {
        false
    }

    /// What the rule pays for one more pupil lost, in words.
    ///
    /// Asked of every shape whether or not its cost is quantified, as
    /// [`crate::decline_adjustment::Shape::rewards_at_the_margin`] is.
    #[must_use]
    pub const fn rewards_at_the_margin(self) -> &'static str {
        match self {
            Self::Fixed => {
                "crossing the floor, which is the behaviour already in force: a district whose \
                 formula amount falls below its FY2020 base is held there and every further pupil \
                 lost is free to it"
            }
            Self::PriorYear => {
                "the same, for every district rather than for the ones below a level chosen in \
                 FY2020 — a district that loses a pupil is held at what it had, so the pupil is \
                 free to it whatever its base"
            }
            Self::Decayed { .. } => {
                "losing pupils faster than the cap. A district shrinking slower than the decay is \
                 paid by the formula and the rule never reaches it; one shrinking faster is held, \
                 and is held at less each year rather than at the same amount forever"
            }
        }
    }
}

/// One district at the end of a walk.
#[derive(Debug, Clone, PartialEq)]
pub struct Standing {
    /// Information Retrieval Number.
    pub irn: String,
    /// Projected enrolled ADM in the final year.
    pub adm: Adm,
    /// What the formula computes at that enrollment, before any floor.
    pub formula_aid: Dollars,
    /// What the district is actually paid, `max(formula, anchor)`.
    pub realized_aid: Dollars,
    /// Transportation, which no anchor touches and which `[K]` counts.
    pub transportation: Dollars,
    /// `[K]`, re-derived against this walk's own output.
    pub transition_supplement: Dollars,
}

impl Standing {
    /// The top-up the anchor pays — `[I]` under whichever rule the walk ran.
    #[must_use]
    pub fn guarantee(&self) -> Dollars {
        (self.realized_aid - self.formula_aid).max(0.0)
    }

    /// Whether the anchor is what determines this district's aid.
    #[must_use]
    pub fn on_the_floor(&self) -> bool {
        self.guarantee() > HELD
    }

    /// Realized aid, transportation and `[K]` together, as `[R] Total State Support` adds them.
    #[must_use]
    pub fn total_state_support(&self) -> Dollars {
        self.realized_aid + self.transportation + self.transition_supplement
    }
}

/// A set of districts at the end of a walk.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Held {
    /// The year walked to.
    pub fiscal_year: FiscalYear,
    /// How many districts are summarized.
    pub districts: usize,
    /// Their projected enrolled ADM.
    pub adm: Adm,
    /// Formula aid at that enrollment.
    pub formula_aid: Dollars,
    /// The anchor's top-up, `[I]`.
    pub guarantee: Dollars,
    /// `[K]`.
    pub transition_supplement: Dollars,
    /// All three channels together.
    pub total_state_support: Dollars,
    /// How many of them the anchor pays.
    pub on_the_floor: usize,
}

impl Held {
    /// Total state support per projected pupil.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.total_state_support / self.adm
    }

    /// The share of what these districts receive that arrives through a hold-harmless.
    ///
    /// The quantity [`crate::decline_adjustment::Trajectory::held_share`] names, and the one every
    /// anchor shape priced here leaves where it found it.
    #[must_use]
    pub fn held_share(&self) -> f64 {
        if self.total_state_support <= 0.0 {
            return 0.0;
        }
        (self.guarantee + self.transition_supplement) / self.total_state_support
    }
}

/// Summarize a walk's standings.
#[must_use]
pub fn summarize<'a>(
    fiscal_year: FiscalYear,
    standings: impl Iterator<Item = &'a Standing>,
) -> Held {
    let mut out = Held {
        fiscal_year,
        districts: 0,
        adm: 0.0,
        formula_aid: 0.0,
        guarantee: 0.0,
        transition_supplement: 0.0,
        total_state_support: 0.0,
        on_the_floor: 0,
    };
    for standing in standings {
        out.districts += 1;
        out.adm += standing.adm;
        out.formula_aid += standing.formula_aid;
        out.guarantee += standing.guarantee();
        out.transition_supplement += standing.transition_supplement;
        out.total_state_support += standing.total_state_support();
        if standing.on_the_floor() {
            out.on_the_floor += 1;
        }
    }
    out
}

/// Walk every district from FY2027 to `through`, one year at a time, under `anchor`.
///
/// # Why this is a walk and [`crate::decline_adjustment::nothing`] is a call
///
/// A fixed anchor is a constant, so the year a district is asked about is the only year that has
/// to be computed. A rolling one is a **recursion**: the floor in a year is a function of the aid
/// the model paid in the year before it, so every intervening year has to be computed whether or
/// not anybody reads it. That difference is also the answer to the data question #390 recorded —
/// the recursion runs forward out of the FY2027 model and needs no history at all.
///
/// The guarantee is switched off in the policy and re-applied here. [`crate::policy::apply`]
/// computes `max(formula, guarantee_floor())` against a constant it reads off the record, and the
/// point of this module is to put a different number there; running it at
/// [`GuaranteeRule::Removed`] returns the formula amount untouched, which is what an anchor is
/// then taken against. `[K]` is recomputed from the result exactly as `apply` recomputes it,
/// because it is a function of the model's own output.
///
/// `method` is carried per district through [`DistrictRecord::projection_method`], as
/// [`crate::decline_adjustment::nothing`] carries it.
#[must_use]
pub fn walk(
    panel: &[DistrictRecord],
    through: FiscalYear,
    method: Method,
    anchor: Anchor,
) -> BTreeMap<String, Standing> {
    let law = Policy::current_law();
    let statewide = Statewide::under(panel, &law);
    let without_the_floor = Policy {
        guarantee: GuaranteeRule::Removed,
        ..law
    };

    // The walk starts from what the FY2027 model actually paid, which is the last realized figure
    // that is observed rather than computed.
    let mut realized: BTreeMap<String, Dollars> = panel
        .iter()
        .map(|record| (record.irn.clone(), record.realized_aid()))
        .collect();
    let mut standings: BTreeMap<String, Standing> = BTreeMap::new();

    for year in FIRST_PROJECTED_YEAR..=through.0 {
        let mut next = BTreeMap::new();
        standings = BTreeMap::new();
        for record in panel {
            let Some(adm) = project(
                &record.adm_observations(),
                FiscalYear(year),
                record.projection_method(method),
                Prior::none(),
            )
            .into_iter()
            .find(|projection| projection.fiscal_year == FiscalYear(year))
            .map(|projection| projection.point) else {
                continue;
            };
            let bare = apply(record, &without_the_floor, &statewide, adm);
            let floor = anchor.of(record, realized[&record.irn]);
            let paid = bare.formula_aid.max(floor);
            next.insert(record.irn.clone(), paid);
            standings.insert(
                record.irn.clone(),
                Standing {
                    irn: record.irn.clone(),
                    adm,
                    formula_aid: bare.formula_aid,
                    realized_aid: paid,
                    transportation: bare.transportation,
                    transition_supplement: transition_supplement_under(
                        record,
                        paid,
                        bare.transportation,
                    ),
                },
            );
        }
        realized = next;
    }
    standings
}

/// What share of a change in the guarantee never reaches total state support.
///
/// `1.0` is fully absorbed — every dollar taken off `[I]` comes back on `[K]`, which is the
/// finding [`crate::decline_adjustment`] reached about a dated phase-down and this module reaches
/// about the anchor's shape. `NaN` where the two walks hold the same guarantee and there is no
/// change to attribute.
#[must_use]
pub fn absorbed(as_enacted: &Held, alternative: &Held) -> f64 {
    let moved = as_enacted.guarantee - alternative.guarantee;
    if moved.abs() <= HELD {
        return f64::NAN;
    }
    1.0 - (as_enacted.total_state_support - alternative.total_state_support) / moved
}

/// An anchor taken from a year the department has published rather than from one this crate
/// projects.
///
/// The one-year counterfactual the observed record can actually support. FY2027 is recomputed with
/// its floor replaced, which is the crossing #390 asks about measured on the year it happened in
/// rather than on a projection of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Observed {
    /// The FY2020 funding base — current law, and the control.
    Fy2020Base,
    /// The district's FY2026 realized aid, from [`crate::prior_model`].
    PriorYearRealized,
    /// The mean of its FY2025 and FY2026 realized aid.
    ///
    /// FY2025 is the final payment report ([`crate::baseline`]) rather than a calculator, so this
    /// is two published years and one modelled one.
    TwoYearMeanRealized,
    /// Its FY2026 `[Hb]` — the formula's own output, before the phase-in and before the guarantee.
    ///
    /// The comparable quantity rather than the paid one: FY2026 runs at an 83.33% phase-in and
    /// FY2027 at 100%, so realized aid carries an interpolation that `[Hb]` does not.
    PriorYearComputed,
}

impl Observed {
    /// A short label for a table.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fy2020Base => "the FY2020 funding base",
            Self::PriorYearRealized => "FY2026 realized aid",
            Self::TwoYearMeanRealized => "the FY2025-FY2026 mean",
            Self::PriorYearComputed => "FY2026 [Hb]",
        }
    }
}

/// One district's published aid in the two years before the modelled one.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Published {
    fy2025_realized: Dollars,
    fy2026_realized: Dollars,
    fy2025_computed: Dollars,
    fy2026_computed: Dollars,
    fy2025_base: Dollars,
    fy2026_base: Dollars,
}

/// The FY2025 payment report and the FY2026 model, joined on IRN.
fn published() -> BTreeMap<String, Published> {
    let mut out: BTreeMap<String, Published> = BTreeMap::new();
    let fy2025: BTreeMap<String, (Dollars, Dollars, Dollars)> = crate::baseline::frame()
        .into_iter()
        .map(|row| {
            (
                row.irn.clone(),
                (
                    row.foundation + row.guarantee,
                    row.foundation_calculated,
                    row.funding_base,
                ),
            )
        })
        .collect();
    for row in crate::prior_model::frame() {
        let Some((realized, computed, base)) = fy2025.get(&row.irn) else {
            continue;
        };
        out.insert(
            row.irn.clone(),
            Published {
                fy2025_realized: *realized,
                fy2026_realized: row.foundation_funding + row.guarantee,
                fy2025_computed: *computed,
                fy2026_computed: row.foundation_calculated,
                fy2025_base: *base,
                fy2026_base: row.funding_base,
            },
        );
    }
    out
}

/// FY2027 recomputed with its floor taken from an observed year instead of from FY2020.
///
/// No projection is involved: this is the modelled year at its modelled enrollment, with one
/// number replaced. A district the FY2025 report or the FY2026 model does not carry is skipped,
/// and [`Held::districts`] says how many were summarized.
#[must_use]
pub fn at_the_modelled_year(panel: &[DistrictRecord], observed: Observed) -> Held {
    let law = Policy::current_law();
    let statewide = Statewide::under(panel, &law);
    let without_the_floor = Policy {
        guarantee: GuaranteeRule::Removed,
        ..law
    };
    let prior = published();
    let standings: Vec<Standing> = panel
        .iter()
        .filter_map(|record| {
            let published = prior.get(&record.irn)?;
            let floor = match observed {
                Observed::Fy2020Base => record.guarantee_floor(),
                Observed::PriorYearRealized => published.fy2026_realized,
                Observed::TwoYearMeanRealized => {
                    (published.fy2025_realized + published.fy2026_realized) / 2.0
                }
                Observed::PriorYearComputed => published.fy2026_computed,
            };
            let bare = apply(
                record,
                &without_the_floor,
                &statewide,
                record.current_year_adm,
            );
            let paid = bare.formula_aid.max(floor);
            Some(Standing {
                irn: record.irn.clone(),
                adm: record.current_year_adm,
                formula_aid: bare.formula_aid,
                realized_aid: paid,
                transportation: bare.transportation,
                transition_supplement: transition_supplement_under(
                    record,
                    paid,
                    bare.transportation,
                ),
            })
        })
        .collect();
    summarize(FiscalYear(2027), standings.iter())
}

/// Whether the three published years can carry a district series, measured rather than assumed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Comparability {
    /// Districts the FY2025 report, the FY2026 model and the FY2027 panel all carry.
    pub districts: usize,
    /// Of those, how many state the same FY2020 funding base in all three.
    pub funding_base_identical: usize,
    /// Upper-middle change in `[Hb]` from FY2025 to FY2026.
    pub computed_median_first_leg: f64,
    /// And from FY2026 to FY2027.
    pub computed_median_second_leg: f64,
    /// How many districts' `[Hb]` falls across the first leg.
    pub computed_falls_first_leg: usize,
    /// And across the second.
    pub computed_falls_second_leg: usize,
    /// How far the two legs agree across districts.
    ///
    /// A district trend would repeat; a regime change would not. This is the number that decides
    /// whether the three years are a series or three regimes.
    pub leg_correlation: f64,
}

impl Comparability {
    /// Whether the three years can support a backtest of a rolling anchor.
    ///
    /// They cannot, and this is the sentence rather than a threshold: a series whose two legs
    /// agree at 0.545 across districts is measuring the regime, not the district. The bar is
    /// stated at 0.8 because that is where a two-leg agreement stops being dominated by whatever
    /// moved for everybody at once, and the measured value is nowhere near it.
    #[must_use]
    pub fn supports_a_backtest(&self) -> bool {
        self.funding_base_identical == self.districts && self.leg_correlation >= 0.8
    }
}

/// Measure the three published years against each other.
#[must_use]
pub fn comparability(panel: &[DistrictRecord]) -> Comparability {
    let prior = published();
    let (mut first, mut second) = (Vec::new(), Vec::new());
    let mut out = Comparability {
        districts: 0,
        funding_base_identical: 0,
        computed_median_first_leg: 0.0,
        computed_median_second_leg: 0.0,
        computed_falls_first_leg: 0,
        computed_falls_second_leg: 0,
        leg_correlation: 0.0,
    };
    for record in panel {
        let Some(published) = prior.get(&record.irn) else {
            continue;
        };
        out.districts += 1;
        if (published.fy2025_base - published.fy2026_base).abs() < 0.01
            && (published.fy2026_base - record.transition.funding_base).abs() < 0.01
        {
            out.funding_base_identical += 1;
        }
        if published.fy2025_computed > 0.0
            && published.fy2026_computed > 0.0
            && record.core_foundation_funding > 0.0
        {
            first.push(published.fy2026_computed / published.fy2025_computed - 1.0);
            second.push(record.core_foundation_funding / published.fy2026_computed - 1.0);
        }
    }
    out.computed_falls_first_leg = first.iter().filter(|x| **x < 0.0).count();
    out.computed_falls_second_leg = second.iter().filter(|x| **x < 0.0).count();
    out.leg_correlation = correlation(&first, &second);
    out.computed_median_first_leg = upper_middle(first);
    out.computed_median_second_leg = upper_middle(second);
    out
}

/// Pearson correlation of two equal-length series.
fn correlation(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let cov: f64 = xs.iter().zip(ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let sx: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum::<f64>().sqrt();
    let sy: f64 = ys.iter().map(|y| (y - my).powi(2)).sum::<f64>().sqrt();
    if sx <= 0.0 || sy <= 0.0 {
        return 0.0;
    }
    cov / (sx * sy)
}

/// The crates' median: sort, then take the upper middle. Never the mean of two.
fn upper_middle(mut values: Vec<f64>) -> f64 {
    assert!(!values.is_empty(), "no values to take a median of");
    values.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a measured rate"));
    values[values.len() / 2]
}
