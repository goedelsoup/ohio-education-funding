//! The six components computed *inside* foundation funding.
//!
//! Every one of these is part of `core_foundation_funding`, which is the base the guarantee holds
//! a district at. The five that are not are in [`super::supplements`], and the line between them
//! is the department's own — see [`super::record::DistrictRecord::total_state_support`].
//!
//! What they share is that each is a published amount this corpus reproduces from its inputs, so
//! the weights and rates below are not documentation of the formula: they are the formula, and
//! `crates/project/tests/the_statute_behind_the_weights.rs` asserts the statute agrees with them.

use edfund_core::{Adm, Dollars};

/// Disadvantaged Pupil Impact Aid, for one district.
///
/// # The mechanism is a blend and a squared index
///
/// DPIA is $525m and is neither a weight times a count, like special education, nor an
/// equalisation off wealth, like targeted assistance. It works in three steps:
///
/// 1. **Blend two poverty counts.** 65% of the FY2025 economically disadvantaged ADM plus 35% of
///    the FY2026 directly certified ADM. The two disagree substantially — direct certification is
///    administrative and finds a **median 61%** of what the disadvantaged count does, and fewer
///    than half of them in 146 districts — so the 35% weight pulls the funded count below the
///    disadvantaged count everywhere. The second vintage is a year behind the weights beside it,
///    which is the model rather than a mistake: see [`Dpia::directly_certified_adm`].
/// 2. **Express it as a share** of the district's enrolled ADM.
/// 3. **Index that share against the state's**, 0.5334 — and *square it*.
///
/// The squaring is the whole character of the program and nothing in a DPIA total shows it. Aid
/// scales with the **square** of relative poverty, so a district at twice the state's rate scores
/// four times the index rather than twice. Verified for all 609 districts.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dpia {
    /// `d1a` — FY2025 economically disadvantaged ADM.
    pub economically_disadvantaged_adm: Adm,
    /// `d1b` — FY2026 directly certified ADM. Consistently the smaller of the two.
    ///
    /// # The vintage is a year behind the weights, and the act says it should not be
    ///
    /// H.B. 96 sets this term on "the ADM of students directly certified as economically
    /// disadvantaged **for the fiscal year for which the DPIA payment is calculated**", and the
    /// LSC greenbook writes the FY2027 row of the formula out with `FY 2027 Directly certified
    /// ADM` in it. The FY2027 workbook this panel is read from carries **FY2026** against the
    /// FY2027 65/35 weights: its `DPIA` sheet heads the column `d1b FY26 Directly Certified ADM`
    /// and its `Directions` sheet sources it from the `FY26 Nov #2` collection.
    ///
    /// **Because the workbook is a simulation.** FY2027 direct certification has not been
    /// collected, so the model substitutes the latest year that has — the same `FY26 Nov #2`
    /// collection every other count in it comes from, the current-year enrolled ADM at the end of
    /// [`super::HISTORY_YEARS`] included. The label is the department's own word and is not an
    /// error; what it means is that the DPIA figures here are computed on a count the actual
    /// FY2027 payment will not use.
    ///
    /// Settled at #174 and pinned by
    /// `crates/project/tests/what_the_act_says_and_the_code_does_not.rs`, which shows the same
    /// thing arithmetically: `d1a` sums to within 1.1% of the greenbook's FY2025 economically
    /// disadvantaged estimate, and `d1b` sums 15% below its FY2025 direct-certification figure.
    pub directly_certified_adm: Adm,
    /// `d1` — the 65/35 blend.
    pub weighted_adm: Adm,
    /// `d2` — the blend as a share of enrolled ADM.
    pub percentage: f64,
    /// `d3` — `(d2 / 0.5334)²`.
    pub index: f64,
}

/// The DPIA blend weights, per-pupil amount, and the statewide share `d3` indexes against.
pub const DPIA_BLEND: (f64, f64) = (0.65, 0.35);
/// What each weighted disadvantaged pupil generates.
pub const DPIA_PER_PUPIL: Dollars = 422.0;
/// The statewide economically disadvantaged percentage.
pub const DPIA_STATEWIDE_PERCENTAGE: f64 = 0.533_380_310_606_710_3;

/// Ohio's six special education categories, for one district.
///
/// # The weights are the policy
///
/// Category 1 is weighted 0.2435 and Category 6 is weighted 3.9554 — a range of sixteen. Each
/// weight multiplies the statewide average base cost per pupil, so a Category 6 pupil is funded at
/// nearly four times what a pupil with no disability generates and a Category 1 pupil at a
/// quarter of it.
///
/// The distribution is not what the weights alone suggest. **Category 6 is 15% of the pupils and
/// 48% of the money**; Category 2 is 65% of the pupils and 34%. Between them those two are 82% of
/// a $722m program, and they are opposite shapes — many pupils at a low weight against few at a
/// very high one. Category 4 is 1,060 pupils statewide and $5.7m.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpecialEducation {
    /// ADM in each category, 1 through 6.
    pub adm: [Adm; 6],
    /// The aid each category produces, after the district's state share percentage.
    pub aid: [Dollars; 6],
}

/// The statutory weight on each category, as the FY2027 workbook states them.
pub const SPECIAL_EDUCATION_WEIGHTS: [f64; 6] = [0.2435, 0.6179, 1.4845, 1.9812, 2.6830, 3.9554];

impl SpecialEducation {
    /// The six aid amounts, summed. Equal to [`Categoricals::special_education`].
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.aid.iter().sum()
    }

    /// Pupils across all six categories.
    #[must_use]
    pub fn total_adm(&self) -> Adm {
        self.adm.iter().sum()
    }
}

/// The six categorical programs, per district, as `Detail_SFPR` publishes them.
///
/// # Why this is six numbers and not one
///
/// [`super::record::DistrictRecord::categorical_funding`] infers the total as core foundation funding less the
/// state share of base cost. That inference is exact — `[A]` plus these six is `[H] Foundation
/// Funding` to the cent — and it produces a quantity no reader can interrogate. The total is
/// **$2.76bn, 43% of formula aid**, against a base cost half this project decomposed into the 22
/// elements of R.C. 3317.011 eight phases ago. For the median district, 42% of its formula aid
/// was a single unexplained number.
///
/// The six do not behave alike, which is why the lump misleads. Targeted assistance is
/// equalisation and falls to zero once a district has enough valuation — Columbus receives none.
/// DPIA is poverty-driven and is Columbus's largest categorical at $40m. A total that adds them
/// describes neither.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Categoricals {
    /// `[B]` — equalisation for low-valuation districts.
    pub targeted_assistance: Dollars,
    /// `[C]` — special education weights.
    pub special_education: Dollars,
    /// `[D]` — Disadvantaged Pupil Impact Aid.
    pub dpia: Dollars,
    /// `[E]` — English learner weights.
    pub english_learners: Dollars,
    /// `[F]` — gifted identification and service.
    pub gifted: Dollars,
    /// `[G]` — career-technical education weights.
    pub career_technical: Dollars,
}

impl Categoricals {
    /// The six, summed. Equal to [`super::record::DistrictRecord::categorical_funding`] by
    /// construction.
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.targeted_assistance
            + self.special_education
            + self.dpia
            + self.english_learners
            + self.gifted
            + self.career_technical
    }
}

/// Ohio's five career-technical categories, for one district.
///
/// # The same shape as special education, against a different base
///
/// Weight times FTE times a base cost times the state share, exactly as
/// [`SpecialEducation`] — but the base cost is career-technical's own,
/// [`CTE_BASE_COST_PER_PUPIL`], which is 20% above the statewide average every other weighted
/// categorical uses. The weight is not the whole of the difference a CTE pupil makes.
///
/// A sixth weight, [`CTE_ASSOCIATED_WEIGHT`], is applied to the sum of all five FTE rather than to
/// any one category, and pays for services rather than instruction.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CareerTechnical {
    /// FTE in each category, 1 through 5.
    pub fte: [Adm; 5],
    /// The aid each category produces, after the district's state share percentage.
    pub aid: [Dollars; 5],
    /// Associated services, weighted against total FTE.
    pub associated_services: Dollars,
}

/// The five career-technical weights and the associated-services weight, as the workbook states.
pub const CTE_WEIGHTS: [f64; 5] = [0.6230, 0.5905, 0.2154, 0.1830, 0.1570];
/// Applied to total CTE FTE rather than to any single category.
pub const CTE_ASSOCIATED_WEIGHT: f64 = 0.0294;
/// The career-technical base cost per pupil the five weights multiply.
pub const CTE_BASE_COST_PER_PUPIL: Dollars = 9855.62;

impl CareerTechnical {
    /// The five aid amounts plus associated services. Equal to [`Categoricals::career_technical`].
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.aid.iter().sum::<Dollars>() + self.associated_services
    }

    /// FTE across all five categories.
    #[must_use]
    pub fn total_fte(&self) -> Adm {
        self.fte.iter().sum()
    }
}

/// Ohio's three English learner categories, for one district.
///
/// # The weights descend
///
/// 0.2104, 0.1577, 0.1053 — Category 1 is funded at twice Category 3. Category 1 is the most
/// recently arrived learner, so the program pays most in a pupil's first year and tapers over the
/// next two. Every other weighted categorical in the plan runs the other way, paying more as need
/// deepens; this one pays more as need is newest.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnglishLearners {
    /// ADM in each category, 1 through 3.
    pub adm: [Adm; 3],
    /// The aid each category produces, after the district's state share percentage.
    pub aid: [Dollars; 3],
}

/// The three English learner weights, in the order the sheet gives them.
pub const ENGLISH_LEARNER_WEIGHTS: [f64; 3] = [0.2104, 0.1577, 0.1053];
/// The statewide average base cost per pupil the EL and special education weights multiply.
pub const AVERAGE_BASE_COST_PER_PUPIL: Dollars = 8241.61;

impl EnglishLearners {
    /// The three aid amounts, summed. Equal to [`Categoricals::english_learners`].
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.aid.iter().sum()
    }

    /// Learners across all three categories.
    #[must_use]
    pub fn total_adm(&self) -> Adm {
        self.adm.iter().sum()
    }
}

/// The gifted program, for one district.
///
/// # The only categorical that is not a weight times a count
///
/// Two per-pupil amounts and three kinds of **unit** — a headcount entitlement priced at a
/// salary-like figure. The units carry floors, and the floors are the policy:
///
/// - a coordinator unit per 3,300 enrolled pupils, floored at **0.5** and capped at **8**;
/// - an intervention specialist unit per 140 identified gifted pupils in K-8, floored at **0.3**;
/// - the same per 140 in grades 9-12, floored at **0.3**, priced $8,404 lower.
///
/// A district that identifies **no gifted pupils at all** still draws 0.5 + 0.3 + 0.3 units, worth
/// $93,993 before the state share. Gifted is the one categorical with a floor rather than a
/// proportion, and the reason is that the units buy staff: half a coordinator is the smallest
/// thing you can hire.
///
/// The cap binds from 26,400 enrolled pupils upward, so the largest districts are held to eight
/// coordinators however many pupils they have.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Gifted {
    /// K-6 enrolment, the base identification is paid against.
    pub adm_k6: Adm,
    /// Identified gifted FTE, by grade band. These drive the specialist units.
    pub fte_k8: Adm,
    /// Identified gifted FTE in grades 9-12.
    pub fte_9_12: Adm,
    /// `[F1]` — $24 per K-6 pupil, after the state share.
    pub identification: Dollars,
    /// `[F2]` — $2.50 per enrolled pupil, after the state share.
    pub referral: Dollars,
    /// `[F3]` — carried by the sheet as a value rather than computed. Zero for every district.
    pub professional_development: Dollars,
    /// Units then dollars, for each of the three unit kinds.
    pub coordinator_units: f64,
    /// Coordinator units, floored and capped.
    pub coordinator_aid: Dollars,
    /// What those units pay.
    pub specialist_k8_units: f64,
    /// Intervention specialist units for K-8, floored at 0.3.
    pub specialist_k8_aid: Dollars,
    /// What those units pay.
    pub specialist_9_12_units: f64,
    /// Intervention specialist units for 9-12, floored at 0.3 and priced lower.
    pub specialist_9_12_aid: Dollars,
}

/// $24 against K-6 enrolment.
pub const GIFTED_IDENTIFICATION_PER_PUPIL: Dollars = 24.0;
/// $2.50 against all enrolment — a different denominator one column away.
pub const GIFTED_REFERRAL_PER_PUPIL: Dollars = 2.50;
/// One coordinator unit per this many enrolled pupils.
pub const GIFTED_COORDINATOR_DIVISOR: f64 = 3300.0;
/// The floor and cap on coordinator units.
pub const GIFTED_COORDINATOR_UNIT_BOUNDS: (f64, f64) = (0.5, 8.0);
/// What a coordinator unit is worth before the state share.
pub const GIFTED_COORDINATOR_UNIT_PRICE: Dollars = 85_776.0;
/// One intervention specialist unit per this many identified gifted pupils, in each band.
pub const GIFTED_SPECIALIST_DIVISOR: f64 = 140.0;
/// The floor on specialist units, in each band.
pub const GIFTED_SPECIALIST_UNIT_FLOOR: f64 = 0.3;
/// The two bands are priced differently, K-8 above 9-12.
pub const GIFTED_SPECIALIST_UNIT_PRICES: (Dollars, Dollars) = (89_378.0, 80_974.0);

impl Gifted {
    /// The two per-pupil amounts plus professional development plus the three unit amounts.
    /// Equal to [`Categoricals::gifted`].
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.identification + self.referral + self.professional_development + self.unit_funding()
    }

    /// `[F] Unit Funding` — the three unit amounts, summed.
    #[must_use]
    pub fn unit_funding(&self) -> Dollars {
        self.coordinator_aid + self.specialist_k8_aid + self.specialist_9_12_aid
    }

    /// Identified gifted pupils across both bands.
    #[must_use]
    pub fn total_fte(&self) -> Adm {
        self.fte_k8 + self.fte_9_12
    }

    /// Whether every unit the district draws is a floor rather than an earned entitlement.
    ///
    /// True where the district's own counts would buy less than the minimum, which is the case
    /// this program exists to cover and which no dollar total distinguishes.
    #[must_use]
    pub fn entirely_on_the_floor(&self) -> bool {
        self.adm_k6 > 0.0
            && self.fte_k8 / GIFTED_SPECIALIST_DIVISOR <= GIFTED_SPECIALIST_UNIT_FLOOR
            && self.fte_9_12 / GIFTED_SPECIALIST_DIVISOR <= GIFTED_SPECIALIST_UNIT_FLOOR
    }
}

/// Targeted assistance, for one district — the largest categorical and the only equalisation.
///
/// # Two tiers that measure different things
///
/// `[G]` is `[C] + [F]`, and the two addends do not answer the same question:
///
/// - **`[C]` the capacity amount** is 0.8% of however far the district's *total* weighted wealth
///   falls below the statewide median district's. It is about the size of the tax base, full stop,
///   so a small district with a small base scores well on it regardless of how few pupils that base
///   serves. The department bounds that with a **size cliff** rather than a per-pupil measure: a
///   district under 200 ADM receives none of this tier, one from 200 to 400 receives 5% of it, and
///   the fraction ramps linearly to 100% only between 400 and 600 pupils.
/// - **`[F]` the wealth amount** is a rate against weighted wealth *per resident pupil*, and cuts
///   off where a district's wealth per pupil reaches 1.25 times the state median.
///
/// # Weighted wealth is 60% property and 40% income
///
/// `[A]` blends assessed valuation with federal adjusted gross income at 60/40 — the same two
/// quantities [`crate::panel::DistrictRecord::published_capacity_rate`] blends for local capacity,
/// at different weights and for a different purpose.
///
/// # Two pupil counts, one line apart
///
/// `[D]` divides weighted wealth by **resident** ADM — enrolled, less those open-enrolling in,
/// plus those open-enrolling out. `[F]` then multiplies the resulting rate by **enrolled** ADM. A
/// district with heavy inbound open enrolment therefore looks wealthier per pupil than it is and
/// is paid on a larger count than it was measured on. Both are correct as written; neither is the
/// same number.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TargetedAssistance {
    /// `[a1]`/`[a2]` — open enrolment in and out, which turn enrolled ADM into resident ADM.
    pub open_enrollment_in: Adm,
    /// `[a2]` — pupils open-enrolling out, added back to reach resident ADM.
    pub open_enrollment_out: Adm,
    /// `[b]`/`[c]`/`[d]` — the FY2019 figures the supplemental tier's eligibility test uses.
    pub fy19_wealth_index: f64,
    /// `[c]` FY2019 enrolled ADM.
    pub fy19_enrolled_adm: Adm,
    /// `[d]` FY2019 total ADM; the eligibility test compares the two.
    pub fy19_total_adm: Adm,
    /// `[A1]`/`[A2]` — the two halves of weighted wealth.
    pub property_valuation: Dollars,
    /// `[A2]` — federal adjusted gross income, the 40% half.
    pub federal_gross_income: Dollars,
    /// `[A]` — blended 60/40.
    pub weighted_wealth: Dollars,
    /// `[B]`/`[C]` — the capacity tier.
    pub capacity_index: f64,
    /// `[C]` — the capacity amount itself.
    pub capacity_amount: Dollars,
    /// `[D]`/`[E]`/`[F]` — the wealth tier.
    pub wealth_per_pupil: Dollars,
    /// `[E]` — the median per pupil over the district's own, so poorer scores higher.
    pub wealth_index: f64,
    /// `[F]` — the wealth amount itself.
    pub wealth_amount: Dollars,
    /// `[I]` — the supplemental tier, and `[H]` the flag that gates it.
    pub supplemental: Dollars,
    /// `[H]` — the flag, which is `N0` rather than `NO` in the department's own sheet.
    pub supplement_eligible: bool,
}

/// The statewide median district's weighted wealth, and the median per pupil.
///
/// The sheet publishes both again **excluding North Bass and Middle Bass** — $362,326,436.80 and
/// $253,337.38 — and references neither. The exclusion is displayed and unused.
pub const TA_MEDIAN_WEIGHTED_WEALTH: Dollars = 392_151_306.632_980_76;
/// The median per resident pupil, which `[E]` indexes against.
pub const TA_MEDIAN_WEALTH_PER_PUPIL: Dollars = 276_708.97;
/// Property valuation against federal adjusted gross income.
pub const TA_WEALTH_BLEND: (f64, f64) = (0.6, 0.4);
/// The share of the shortfall below the median district's total wealth the capacity tier pays.
pub const TA_CAPACITY_RATE: f64 = 0.008;
/// The capacity tier's size cliff: nothing below 200 ADM, 5% to 400, ramping to full at 600.
pub const TA_CAPACITY_MINIMUM_ADM: Adm = 200.0;
/// Below this enrolled ADM the capacity tier pays nothing at all.
pub const TA_CAPACITY_RAMP_START: Adm = 400.0;
/// From here to `TA_CAPACITY_FULL_AT` the share ramps linearly from 5% to 100%.
pub const TA_CAPACITY_FULL_AT: Adm = 600.0;
/// At and above this ADM the capacity tier is paid in full.
pub const TA_CAPACITY_SMALL_SHARE: f64 = 0.05;
/// The wealth tier's coefficients. The second is exactly 0.8 times the first, which is why the
/// tier cuts off at a wealth index of 0.8 — that is where the bracket reaches zero, not a
/// separately chosen threshold.
pub const TA_WEALTH_RATE: f64 = 0.014;
/// The wealth tier's rate against the statewide median per pupil.
pub const TA_WEALTH_OFFSET_RATE: f64 = 0.0112;
/// Its rate against the district's own, exactly 0.8 times the first.
pub const TA_WEALTH_INDEX_FLOOR: f64 = 0.8;

impl TargetedAssistance {
    /// The two tiers, summed. Equal to [`Categoricals::targeted_assistance`].
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.capacity_amount + self.wealth_amount
    }

    /// Enrolled ADM less those open-enrolling in, plus those open-enrolling out.
    ///
    /// The count `[D]` is measured against, and not the count `[F]` is paid on.
    #[must_use]
    pub fn resident_adm(&self, enrolled_adm: Adm) -> Adm {
        enrolled_adm - self.open_enrollment_in + self.open_enrollment_out
    }

    /// The fraction of the capacity tier a district of this size receives.
    ///
    /// A step at 200, a shelf to 400, a ramp to 600, then full. Below the step the tier pays
    /// nothing however far below the median the district's wealth falls.
    #[must_use]
    pub fn capacity_size_share(enrolled_adm: Adm) -> f64 {
        if enrolled_adm > TA_CAPACITY_FULL_AT {
            1.0
        } else if enrolled_adm > TA_CAPACITY_RAMP_START {
            0.95 * (enrolled_adm - TA_CAPACITY_RAMP_START)
                / (TA_CAPACITY_FULL_AT - TA_CAPACITY_RAMP_START)
                + TA_CAPACITY_SMALL_SHARE
        } else if enrolled_adm >= TA_CAPACITY_MINIMUM_ADM {
            TA_CAPACITY_SMALL_SHARE
        } else {
            0.0
        }
    }
}
