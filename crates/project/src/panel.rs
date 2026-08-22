//! The district panel: the department's FY2027 model, as records.
//!
//! Every district Ohio funds, with what the department computed for it. The fixture is embedded
//! at compile time, so this is pure and deterministic like the calculators — no filesystem, no
//! clock, no network — and a projection run years from now reads the same numbers.
//!
//! # What the columns mean, and one that was mislabelled
//!
//! `enrolled_adm_fy24/25/26` are the three years R.C. 3317.011 averages to get base cost
//! enrolled ADM for FY2027. The department's `Base_Cost` sheet labels the same three columns
//! FY22/FY23/FY24 and is stale; the earlier version of this fixture inherited that, so every
//! enrollment-trend figure in the corpus was named for the wrong pair of years. See
//! `crates/connect/src/fixtures.rs`.
//!
//! **FY2026 is not fully an actual.** The calculator is published in December 2025, before that
//! fiscal year closes, so the last observation in every district's history is partly a
//! departmental estimate. A projection that starts from it inherits that, and the label
//! "observed" is doing some work it should not have to do alone.

use edfund_core::{Adm, Dollars, FiscalYear};
use foundation::DistrictEnrollment;

use crate::series::Observation;

/// The department's FY2027 funding model, one row per district.
const FIXTURE: &str = include_str!("../../foundation/fixtures/fy27-department-model.csv");

/// The fiscal year the model computes.
pub const MODEL_YEAR: FiscalYear = FiscalYear(2027);

/// The three fiscal years of enrolled ADM the model averages.
pub const HISTORY_YEARS: [FiscalYear; 3] = [FiscalYear(2024), FiscalYear(2025), FiscalYear(2026)];

/// The minimum state share of base cost operative in the FY2027 model.
///
/// The department's `Notes` sheet states `0.1` for FY2026 and FY2027. It is **not** the 5% the
/// Fair School Funding Plan was enacted with; each biennial budget sets it, and it doubled.
/// 138 of 609 districts sit exactly on it.
pub const MINIMUM_STATE_SHARE: f64 = edfund_core::MINIMUM_STATE_SHARE_FY2027;

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
///    disadvantaged count everywhere.
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
/// [`DistrictRecord::categorical_funding`] infers the total as core foundation funding less the
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
    /// The six, summed. Equal to [`DistrictRecord::categorical_funding`] by construction.
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

/// The performance supplement, for one district.
///
/// # The only place Ohio's funding formula pays on measured outcomes
///
/// $55.7m, and structurally unlike everything else here: every other component pays for inputs — a
/// pupil, a category, a tax base — and this pays for a **rating**. It sits outside foundation
/// funding, in `[R] Total State Support`, so the guarantee does not hold a district at it.
///
/// Three routes qualify a district, and only one of them is about being good:
///
/// - an overall rating above 3.5 stars — 288 districts;
/// - a progress component rating of 3 or more — a further 120, whatever their overall rating;
/// - a progress rating higher than the year before — a further 18, whatever the level.
///
/// The amount then scales with the **greater** of the overall and progress ratings, at $13 a pupil
/// a point. So the payment runs from $32.50 to $65.00 per pupil, and a district that qualifies by
/// improving from a low base is paid on that low base.
///
/// # It runs the opposite way to the rest of the formula
///
/// Sorted by economically disadvantaged share, mean supplement per pupil runs **$54.74, $43.24,
/// $36.21, $30.03, $23.31** from the least-poor quintile to the poorest, and the share of
/// districts qualifying runs 91% to 49%. The least-poor districts receive **2.3 times** per pupil
/// what the poorest do.
///
/// That is worth stating precisely rather than as an accusation. Ohio's attainment measures track
/// composition — this corpus has already established that spending per *weighted* pupil against
/// performance is substantially a composition proxy — so a program keyed to them will follow
/// composition whatever its intent. The finding is not that the department is paying wealthy
/// districts on purpose; it is that a $55.7m component of a formula built to equalise is
/// distributed inversely to need, and no published figure says so.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PerformanceSupplement {
    /// `O1` — the overall rating, 0 to 5 in half steps. `None` for the one district rated `N/A`.
    pub stars: Option<f64>,
    /// `O2`/`O3` — the progress component rating, and the year before it.
    pub progress: Option<f64>,
    /// The year before, which the third qualifying route compares against.
    pub progress_prior: Option<f64>,
    /// `Q4` — whether any of the three routes qualified it.
    pub eligible: bool,
    /// `O` — the amount.
    pub amount: Dollars,
}

/// Per pupil, per rating point.
pub const PERFORMANCE_SUPPLEMENT_PER_POINT: Dollars = 13.0;
/// Above this overall rating a district qualifies outright.
pub const PERFORMANCE_STAR_THRESHOLD: f64 = 3.5;
/// At or above this progress rating it qualifies whatever its overall rating.
pub const PERFORMANCE_PROGRESS_THRESHOLD: f64 = 3.0;

impl PerformanceSupplement {
    /// The rating the payment is computed on: the greater of the two, where both exist.
    #[must_use]
    pub fn paid_rating(&self) -> Option<f64> {
        match (self.stars, self.progress) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    /// Which of the three routes qualified the district, for a page that wants to say.
    ///
    /// Ordered as the workbook's nested `IF` evaluates them, so a district meeting two is
    /// attributed to the first — which is what the department's own sheet does.
    #[must_use]
    pub fn route(&self) -> Option<&'static str> {
        if !self.eligible {
            return None;
        }
        if self.stars.is_some_and(|s| s > PERFORMANCE_STAR_THRESHOLD) {
            Some("an overall rating above 3.5 stars")
        } else if self
            .progress
            .is_some_and(|p| p >= PERFORMANCE_PROGRESS_THRESHOLD)
        {
            Some("a progress rating of 3 or more")
        } else {
            Some("a progress rating higher than the year before")
        }
    }
}

/// The two supplements on the `Base_Enrollment Growth` sheet, for one district.
///
/// # One is unconditional and one is a cliff
///
/// The **base funding supplement** is $40 for every pupil in every district, $56.1m statewide, with
/// no test of any kind. It is the simplest line in the whole calculation.
///
/// The **enrollment growth supplement** is $250 a pupil for a district whose enrolment rose at
/// least **3%** over three years — and it pays on *every* pupil, not on the pupils gained. 43
/// districts draw it, $39.4m between them, against a median district that **shrank 4.8%**.
///
/// Paying on the whole roll rather than the increment turns the threshold into a cliff with real
/// money on it. New Lexington grew **2.9502%** and drew nothing; three hundredths of a percentage
/// point further and it would have received **$430,477**. Two other districts sit between 2.7%
/// and 3%.
///
/// # And it points the opposite way to the guarantee
///
/// The guarantee holds a district at a historical amount when its enrolment *falls*; this pays a
/// premium when it *rises*. The same formula cushions movement in both directions, which is
/// defensible per district and means the formula responds to enrolment change less than its
/// per-pupil construction suggests.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Supplements {
    /// `L` — $40 a pupil, unconditional.
    pub base_funding: Dollars,
    /// `M1a`/`M1B` — the two ends of the three-year comparison. FY2023 is a fourth ADM year the
    /// panel does not otherwise hold.
    pub adm_fy23: Adm,
    /// `M1` — the three-year change as a fraction.
    pub enrollment_change: f64,
    /// `M2`/`M` — whether it cleared 3%, and what that paid.
    pub growth_eligible: bool,
    /// What clearing it paid: $250 on every pupil, not on the pupils gained.
    pub growth: Dollars,
}

/// Per pupil, every district, no test.
pub const BASE_FUNDING_SUPPLEMENT_PER_PUPIL: Dollars = 40.0;
/// Per pupil — on the whole roll, not the increment — for a district clearing the threshold.
pub const ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL: Dollars = 250.0;
/// Three per cent over three years, as a cliff.
pub const ENROLLMENT_GROWTH_THRESHOLD: f64 = 0.03;

impl Supplements {
    /// What clearing the threshold would have paid a district that did not.
    ///
    /// `None` where the district did clear it. The point of the figure is the cliff: for a
    /// district just below, this is the cost of three hundredths of a percentage point.
    #[must_use]
    pub fn forgone(&self, enrolled_adm: Adm) -> Option<Dollars> {
        (!self.growth_eligible).then_some(enrolled_adm * ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL)
    }
}

/// Transportation, for one district — the largest thing outside foundation funding.
///
/// # $726m, plus $183m of special education transportation, and none of it in the formula
///
/// Transportation alone is larger than special education — the **second-largest single program in
/// Ohio's school funding**, after targeted assistance — and with special education transportation
/// beside it the pair exceeds DPIA, gifted, career-technical and English learners combined. The
/// corpus carried both inside an unexplained remainder. Nothing about the mechanism resembles the
/// formula.
///
/// **Two competing bases and the district gets the greater.** Per weighted rider at $1,337.175, or
/// per bus mile at $6.867 across a 180-day year. **350 of 611 districts are paid on the mile
/// base**, so the choice flips for more than half the state — a district's transportation aid can
/// be a function of its geography rather than its ridership, and which one is not visible in the
/// amount.
///
/// **Non-public riders count double.** Weighted ridership is `public + 2 x non-public + 1.5 x
/// community or STEM`. A district transporting a private-school child is funded at twice the rate
/// of its own pupil. Non-public riders are 4.5% of riders and 8.5% of weighted ridership.
///
/// **The state minimum share is 50%.** Against the formula's 10%, and **440 of 611 districts sit
/// on it** — 72%, against 23% on the formula's floor. For most of the state, local capacity does
/// not determine transportation aid at all. That is the single largest difference between how
/// Ohio equalises instruction and how it equalises getting to it.
///
/// **Two supplements pulling opposite ways.** The efficiency supplement pays up to 15% more for
/// filling buses — riders per bus over a capacity target, ramping from an index of 1.0 to 1.5. The
/// density supplement pays sparse districts, `(28 - riders per square mile) / 100` times the mile
/// base times 0.55. One rewards concentration and the other compensates for its absence, and 388
/// districts draw the second while 406 draw the first.
///
/// **And its own guarantee.** `[F]` holds 38 districts at their FY2021 transportation funding,
/// $24.8m. This is a **second** transitional guarantee, separate from the one on foundation
/// funding, and the corpus has a node for only one of them.
///
/// # The proration factor is the finding a dollar total cannot carry
///
/// Special education transportation is multiplied by **0.91746**. A proration factor below one
/// means the appropriation did not cover the computed entitlement and every district's amount was
/// scaled down to fit — so the published figure is not what the formula says a district is owed,
/// it is what was available divided among them. Nothing else in this corpus has a parameter of
/// that kind, and it cannot be recovered from the amount.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Transportation {
    /// `[a1]`/`[a2]`/`[a3]` — riders by the kind of school they attend.
    pub public_riders: f64,
    /// Weighted double.
    pub nonpublic_riders: f64,
    /// Weighted one and a half.
    pub community_riders: f64,
    /// `[b]` — the three at 1, 2 and 1.5.
    pub weighted_riders: f64,
    /// `[c]`/`[d]` — paid at 35% and 50% of the rider rate.
    pub mass_transit_riders: f64,
    /// Type 5 and 6 vehicles.
    pub other_riders: f64,
    /// `[e]` through `[h]` — the mile base and the two supplements' inputs.
    pub bus_miles: f64,
    /// Type 1 and 2 buses, which the efficiency index divides riders by.
    pub assigned_buses: f64,
    /// What the efficiency index measures riders per bus against.
    pub rider_capacity_target: f64,
    /// `[D2]`/`[E2]` — the published indices the two supplements are computed from, each rounded
    /// to four places by the sheet before use.
    pub efficiency_index: f64,
    /// Riders per square mile, which the density supplement pays against.
    pub district_density: f64,
    /// The area the density supplement spreads riders over.
    pub square_miles: f64,
    /// `[j]` — reported cost, before proration.
    pub reported_sped_cost: Dollars,
    /// `[A]` through `[E]` — the payments.
    pub school_bus: Dollars,
    /// Mass transit riders at 35% of the rider rate.
    pub mass_transit: Dollars,
    /// Other vehicle types at 50%.
    pub other: Dollars,
    /// Up to 15% more for filling buses.
    pub efficiency: Dollars,
    /// And a payment for not being able to.
    pub density: Dollars,
    /// `[F1]`/`[F]` — the FY2021 base and the guarantee it produces.
    pub fy21_base: Dollars,
    /// A second transitional guarantee, holding 38 districts at their FY2021 amount.
    pub guarantee: Dollars,
    /// `[G]`/`[J]` — the total, and special education transportation beside it.
    pub total: Dollars,
    /// Prorated at 0.91746, because the appropriation did not cover the entitlement.
    pub special_education: Dollars,
}

/// Per weighted rider, and per bus mile across a school year.
pub const TRANSPORT_PER_RIDER: Dollars = 1337.175;
/// The mile base, which more than half the state is actually paid on.
pub const TRANSPORT_PER_MILE: Dollars = 6.867;
/// A school year, as the mile base counts it.
pub const TRANSPORT_SCHOOL_DAYS: f64 = 180.0;
/// Non-public riders count double; community and STEM school riders one and a half times.
pub const TRANSPORT_NONPUBLIC_WEIGHT: f64 = 2.0;
/// Community and STEM school riders.
pub const TRANSPORT_COMMUNITY_WEIGHT: f64 = 1.5;
/// Mass transit and other vehicle types, as fractions of the rider rate.
pub const TRANSPORT_MASS_TRANSIT_RATE: f64 = 0.35;
/// Other vehicle types, type 5 and 6.
pub const TRANSPORT_OTHER_RATE: f64 = 0.50;
/// Fifty per cent, against the formula's ten.
pub const TRANSPORT_MINIMUM_STATE_SHARE: f64 = 0.5;
/// The efficiency supplement's ceiling and the index band it ramps across.
pub const TRANSPORT_EFFICIENCY_CEILING: f64 = 0.15;
/// Below the lower bound nothing; across the band it ramps to the ceiling.
pub const TRANSPORT_EFFICIENCY_BAND: (f64, f64) = (1.0, 1.5);
/// The density supplement's pivot and rate.
pub const TRANSPORT_DENSITY_PIVOT: f64 = 28.0;
/// What fraction of the mile base the density supplement pays.
pub const TRANSPORT_DENSITY_RATE: f64 = 0.55;
/// What the appropriation could actually cover of the special education entitlement.
pub const TRANSPORT_SPED_PRORATION: f64 = 0.917_459_740_976_215;

impl Transportation {
    /// All riders on type 1 and 2 buses, unweighted.
    #[must_use]
    pub fn riders(&self) -> f64 {
        self.public_riders + self.nonpublic_riders + self.community_riders
    }

    /// `[A1]` — what the rider base would pay before the state share.
    #[must_use]
    pub fn per_rider_base(&self) -> Dollars {
        self.weighted_riders * TRANSPORT_PER_RIDER
    }

    /// `[A2]` — what the mile base would pay before the state share.
    #[must_use]
    pub fn per_mile_base(&self) -> Dollars {
        self.bus_miles * TRANSPORT_PER_MILE * TRANSPORT_SCHOOL_DAYS
    }

    /// Whether the mile base is the one this district is actually paid on.
    ///
    /// True for more than half the state, and invisible in the amount. A district paid on miles
    /// gains nothing from carrying more children on the same routes; one paid on riders gains
    /// nothing from covering more ground.
    #[must_use]
    pub fn paid_on_miles(&self) -> bool {
        self.per_mile_base() > self.per_rider_base()
    }

    /// `[E2]` recomputed from the counts, for checking the published figure against its inputs.
    #[must_use]
    pub fn density_from_inputs(&self) -> f64 {
        if self.square_miles > 0.0 {
            self.riders() / self.square_miles
        } else {
            0.0
        }
    }

    /// `[D2]` recomputed the same way, through the sheet's own intermediate rounding.
    #[must_use]
    pub fn efficiency_index_from_inputs(&self) -> f64 {
        if self.assigned_buses <= 0.0 || self.rider_capacity_target <= 0.0 {
            return 0.0;
        }
        let per_bus = (self.riders() / self.assigned_buses * 10_000.0).round() / 10_000.0;
        (per_bus / self.rider_capacity_target * 10_000.0).round() / 10_000.0
    }

    /// The five payments, summed — which is the total before proration.
    #[must_use]
    pub fn before_proration(&self) -> Dollars {
        self.school_bus + self.mass_transit + self.other + self.efficiency + self.density
    }

    /// What special education transportation would have been had the appropriation covered it.
    ///
    /// The published figure is the prorated one. This is the entitlement it was scaled down from,
    /// and the difference is what the line was short.
    #[must_use]
    pub fn special_education_unprorated(&self) -> Dollars {
        self.special_education / TRANSPORT_SPED_PRORATION
    }
}

/// Preschool special education, for one district — the last line outside foundation funding.
///
/// # A flat amount and a half-weight, which nothing else in the formula combines
///
/// $148m. Each category pays `(ADM x $4,000) + (ADM x weight x average base cost x state share x
/// 0.5)`, and the whole is prorated. So a preschool pupil generates a **flat $4,000 whatever their
/// category** — 69% of the program — plus a weighted amount at **half** the school-age rate,
/// against the same six weights.
///
/// **The state share applies only to the weighted half.** The $4,000 is paid in full to every
/// district, which makes this the one component in Ohio's school funding where the wealthiest
/// district and the poorest are funded identically for most of what they receive. Whether that is
/// deliberate levelling or an artefact of bolting a flat grant onto a weighted formula is not
/// established here. `[open]`
///
/// Halving the weights also compresses the program relative to the school-age one: Category 6 is
/// 3.9554 there and effectively 1.9777 here, so the range between the highest and lowest category
/// narrows — and against a $4,000 flat floor it narrows much further. 5,761 Category 6 pupils draw
/// $52.9m and 10,842 Category 2 pupils draw $50.6m, which is far closer to parity than school-age
/// special education's 15%-of-pupils-48%-of-money.
///
/// # The proration no longer fits the appropriation it was set against
///
/// The sheet carries its **appropriation limit of $147,500,000** in a cell beside the factor,
/// which makes this the clearest statement in the whole workbook of what a proration is: a budget
/// divided by an entitlement. At the stated factor of 0.96854448 the program totals
/// **$148,408,184** — **$908,184 over the limit**. The factor that would reach it is 0.96261747.
///
/// A third cell on the same sheet states $146,708,228.07, matching neither the column above it nor
/// the cap. The likeliest reading is that the factor was calibrated against an earlier ADM vintage
/// and the counts were refreshed without recalibrating it. This is a projection published before
/// the fiscal year rather than an actual, so recalibration before payment is expected — but as
/// published the three figures are mutually inconsistent, and that is worth recording rather than
/// smoothing over.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PreschoolSpecialEducation {
    /// ADM in each category, 1 through 6 — the same categories as school age.
    pub adm: [Adm; 6],
    /// The aid each produces: the flat amount plus the half-weight, prorated.
    pub aid: [Dollars; 6],
    /// The six, as the sheet totals them.
    pub total: Dollars,
}

/// Paid per preschool pupil whatever their category, and not reduced by the state share.
pub const PREK_SPED_FLAT_PER_PUPIL: Dollars = 4000.0;
/// The school-age weights apply at half here.
pub const PREK_SPED_WEIGHT_FRACTION: f64 = 0.5;
/// The stated factor, and the appropriation it was calibrated against.
pub const PREK_SPED_PRORATION: f64 = 0.968_544_48;
/// A limit the program now exceeds by $908,184.
pub const PREK_SPED_APPROPRIATION: Dollars = 147_500_000.0;

impl PreschoolSpecialEducation {
    /// Pupils across all six categories.
    #[must_use]
    pub fn total_adm(&self) -> Adm {
        self.adm.iter().sum()
    }

    /// What the flat component alone is worth, after proration. Most of the program.
    #[must_use]
    pub fn flat_component(&self) -> Dollars {
        self.total_adm() * PREK_SPED_FLAT_PER_PUPIL * PREK_SPED_PRORATION
    }

    /// What the program would have paid had the appropriation covered it.
    #[must_use]
    pub fn unprorated(&self) -> Dollars {
        self.total / PREK_SPED_PRORATION
    }
}

/// The guarantee's own machinery, and the second hold-harmless stacked on top of it.
///
/// # The guarantee is not "hold the district at its old amount"
///
/// `[I] Temporary Transitional Aid Guarantee` is `funding base − open enrolment adjustment −
/// foundation funding`, floored at zero — and the middle term is a **clawback** the corpus did not
/// know about.
///
/// A guaranteed district whose open enrolment FTE has fallen by more than `max(10% of last year,
/// 20 FTE)` has its guarantee reduced by the **statewide average base cost per pupil** for every
/// FTE beyond that threshold. **43 districts, $5.1m withheld.** Columbus lost 106.2 FTE against a
/// threshold of 24.3 and had $674,561 cut; Cuyahoga Falls lost 118.1 and had $640,025 cut.
///
/// The rate is the striking part. It is $8,241.61 — the **full** average base cost per pupil, not
/// the district's state share of it. A district at the 10% minimum state share was receiving about
/// $824 of state money for that pupil and loses ten times as much guarantee when the pupil goes.
/// Whether that is intended as a strong incentive or is an artefact of using a convenient
/// statewide figure is not established. `[open]`
///
/// # And there is a third FY2021 anchor
///
/// `[K] Formula Transition Supplement` is a **second** hold-harmless on top of the first, against a
/// larger base: `max(FY21 funding base − (foundation funding + guarantee + supplemental targeted
/// assistance + transportation), 0)`. $63.6m to 144 districts — **17 of which are not on the
/// guarantee at all**, so this reaches districts the guarantee does not.
///
/// With transportation's own guarantee that makes three mechanisms anchored to FY2021, on three
/// different bases, each holding a different set of districts. The corpus has one node for them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Transition {
    /// `[H2]` — the FY2021 amount the guarantee compares foundation funding against.
    pub funding_base: Dollars,
    /// `[H3]` — the DPIA part of that base, which the phase-in dials separately.
    pub funding_base_econ_dis: Dollars,
    /// `[h1]`/`[h2]` — open enrolment FTE, last year and this.
    pub open_enrollment_prior: f64,
    /// This year.
    pub open_enrollment_current: f64,
    /// `[I2]` — how much of a loss the district may absorb before the clawback applies.
    pub open_enrollment_threshold: f64,
    /// `[I1]` — what the loss beyond that threshold costs its guarantee.
    pub open_enrollment_adjustment: Dollars,
    /// `[L1]` — a FY2021 base that includes transportation, unlike `[H2]`.
    pub fy21_funding_base: Dollars,
    /// `[K]` — the second hold-harmless, against that larger base.
    pub transition_supplement: Dollars,
}

/// Charged per open enrolment FTE lost beyond the threshold: the full statewide average base cost
/// per pupil, not the district's state share of it.
pub const OPEN_ENROLLMENT_CLAWBACK_PER_FTE: Dollars = 8241.61;
/// The threshold is the greater of a tenth of last year's FTE and this floor.
pub const OPEN_ENROLLMENT_THRESHOLD_FRACTION: f64 = 0.1;
/// The floor on it, so a small district is not clawed back over a handful of FTE.
pub const OPEN_ENROLLMENT_THRESHOLD_FLOOR: f64 = 20.0;

impl Transition {
    /// Open enrolment FTE lost since last year. Negative where the district gained.
    #[must_use]
    pub fn open_enrollment_lost(&self) -> f64 {
        self.open_enrollment_prior - self.open_enrollment_current
    }

    /// The loss beyond what the threshold absorbs, which is what the clawback is charged on.
    #[must_use]
    pub fn clawed_back_fte(&self) -> f64 {
        (self.open_enrollment_lost() - self.open_enrollment_threshold).max(0.0)
    }

    /// What the guarantee would have been without the clawback.
    ///
    /// `None` where no clawback applied. The difference is the point: a district's guarantee is
    /// reported as one number and is two, and only one of them is about its funding.
    #[must_use]
    pub fn guarantee_before_clawback(&self, guarantee: Dollars) -> Option<Dollars> {
        (self.open_enrollment_adjustment > 0.0 && guarantee > 0.0)
            .then_some(guarantee + self.open_enrollment_adjustment)
    }
}

/// One district as the department modelled it.
#[derive(Debug, Clone, PartialEq)]
pub struct DistrictRecord {
    /// Information Retrieval Number, the stable statewide identifier.
    pub irn: String,
    /// District name as published.
    pub name: String,
    /// Grade-band ADM and building count, ready for [`foundation`].
    pub enrollment: DistrictEnrollment,
    /// Funded classroom teachers, as the department computed them.
    pub funded_classroom_teachers: f64,
    /// Funded special teachers, as the department computed them.
    pub funded_special_teachers: f64,
    /// Aggregate base cost, all five sub-components.
    pub aggregate_base_cost: Dollars,
    /// Aggregate base cost divided by base cost enrolled ADM.
    pub base_cost_per_pupil: Dollars,
    /// The state's share of base cost alone, before every categorical.
    pub base_cost_state_share: Dollars,
    /// Base cost share plus targeted assistance, special education, DPIA, English learner,
    /// gifted, and career-technical. This is formula aid.
    pub core_foundation_funding: Dollars,
    /// `[R] Total State Support` — every state payment the report carries.
    ///
    /// Wider than [`DistrictRecord::realized_aid`], which is core foundation funding plus the
    /// guarantee. The difference is transportation, preschool special education, special
    /// education transportation, and the performance supplement: real money, paid to the
    /// district, and **outside the base the guarantee holds it at**.
    ///
    /// The distinction is not pedantic. Comparing `realized_aid` against a figure that behaves
    /// like this one — a district's booked state receipts, for instance — compares a narrow
    /// construction against a broad one and produces a shortfall that is definitional rather
    /// than real. This corpus did exactly that for two phases.
    pub total_state_support: Dollars,
    /// `U - Total Transfers` — educational service center charges plus other adjustments.
    ///
    /// **Not the voucher channel.** Under the Fair School Funding Plan community and STEM
    /// students are funded directly rather than deducted from the resident district, and no line
    /// in this report carries a scholarship deduction. Cleveland Municipal's transfers are
    /// -$3.8M against $322.6M of total state support; a deduction channel would be two orders
    /// larger. Carried so that claim is checkable rather than asserted — see
    /// [`crate::finances`] and the `deduction` skill for what is still missing.
    pub total_transfers: Dollars,
    /// `S - Educational Service Center` alone — negative for 606 of 609 districts and never
    /// positive, which is what a charge looks like rather than a channel.
    pub service_center_charge: Dollars,
    /// `T - Other Adjustments` alone — the report's only unlabelled line, and therefore the whole
    /// of the space in which a voucher or community-school deduction could still be sitting.
    ///
    /// It cannot be sitting there, and this field is what settles it. The negative half totals
    /// $95.6m across 577 districts — 1.12% of total state support — against Ohio scholarship
    /// spending on the order of a billion a year. A deduction channel would have to be ten times
    /// the size of the entire residual it would have to hide in.
    ///
    /// The question was open since genesis and was previously answered by direction: transfers
    /// run both ways, so the *line* is not a deduction. That left "a deduction could be inside
    /// its negative half" unresolved, because `total_transfers` mixes the service-centre charge
    /// with the residual. Splitting them measures the residual on its own.
    pub other_adjustments: Dollars,
    /// `V - Net State Funding` — total state support after transfers.
    pub net_state_funding: Dollars,
    /// Local capacity per pupil **as the department publishes it**, from `Detail_SFPR`.
    ///
    /// [`DistrictRecord::implied_local_capacity_per_pupil`] recovers this by subtracting state
    /// aid from base cost, which works for 470 districts and is impossible for the rest: where
    /// the minimum state share binds, the subtraction gives a number smaller than the truth and
    /// the corpus returns `None` rather than a plausible wrong figure.
    ///
    /// It does not have to be recovered. The department computes it and publishes it on a sheet
    /// this repository did not read for eleven phases, for every district including the ones the
    /// subtraction censors.
    pub published_capacity_per_pupil: Option<Dollars>,
    /// `[b4] State Share Percentage`, likewise published rather than inferred.
    pub published_state_share: Option<f64>,
    /// Assessed valuation for the three tax years the capacity measure blends, newest first.
    pub valuation_three_year: [Dollars; 3],
    /// Federal adjusted gross income for its three tax years, newest first.
    pub agi_three_year: [Dollars; 3],
    /// Tax returns filed in the district, the count the median-income term multiplies.
    pub tax_returns: Option<f64>,
    /// `[I5] TY23 Federal Median Income with ADJ Factor` — the blend's third term.
    ///
    /// **Federal**, and adjusted. An earlier pass here inferred the term from the profile report's
    /// Ohio median income, because the `Local_Capacity` sheet that states all of this had not been
    /// opened. That fits approximately — well enough to look right — and is the wrong measure:
    /// Columbus's federal median is $46,395 against an Ohio median of $31,555.
    pub median_income: Option<Dollars>,
    /// `[I7] TY23 Statewide Federal Median Income` — the denominator of the income ratio.
    ///
    /// $54,546.64, published on the sheet rather than derivable. The median of district medians
    /// is $41,502 and is a different quantity.
    pub statewide_median_income: Option<Dollars>,
    /// `[C5]` — the income ratio of the 40th highest district, which tops out the sliding scale.
    ///
    /// Published rather than reconstructed. Reconstructing it from the panel gave 1.4151 against
    /// a published 1.46504, and it is a discretionary number rather than a derived one.
    pub benchmark_ratio: Option<f64>,
    /// `[C6] Local Capacity Percentage` — the rate, as the department computes it.
    pub published_capacity_rate: Option<f64>,
    /// The six categorical programs, each read rather than inferred as a lump.
    pub categoricals: Categoricals,
    /// The second-largest categorical, decomposed into the six weighted categories.
    pub special_education: SpecialEducation,
    /// The third-largest, and the one whose mechanism is neither a weight nor an equalisation.
    pub dpia: Dpia,
    /// The largest, and the only equalisation among the six.
    pub targeted_assistance: TargetedAssistance,
    /// Five weights against a career-technical base cost, plus associated services.
    pub career_technical: CareerTechnical,
    /// Three weights that descend rather than ascend.
    pub english_learners: EnglishLearners,
    /// Two per-pupil amounts and three kinds of unit, with floors no other categorical has.
    pub gifted: Gifted,
    /// `[a] Enrolled ADM` — a seventh pupil count, and the one four of the six categoricals are
    /// paid on.
    ///
    /// Not [`DistrictRecord::current_year_adm`], which is `[b3] FY26 Enrolled ADM`, and not
    /// [`DistrictRecord::base_cost_adm`], which averages three years. Akron's are 18,892.45 and
    /// 18,842.45 — fifty pupils apart on the same sheet, four columns from each other.
    ///
    /// Targeted assistance, gifted, career-technical and English learners are all computed against
    /// this one. Reproducing any of them from `current_year_adm` gets an answer that is close, and
    /// wrong, which is the failure mode that is hardest to notice.
    pub categorical_enrolled_adm: Adm,
    /// The performance supplement, and the ratings that gate it.
    pub performance: PerformanceSupplement,
    /// The base funding supplement and the enrollment growth supplement.
    pub supplements: Supplements,
    /// Transportation, and special education transportation beside it.
    pub transportation: Transportation,
    /// Preschool special education: a flat $4,000 a pupil plus the weights at half.
    pub preschool_special_education: PreschoolSpecialEducation,
    /// The guarantee's machinery, and the transition supplement stacked on it.
    pub transition: Transition,
    /// The county the department attributes the district to, from `Base_Cost`.
    ///
    /// One county per district, which is a **simplification the department makes and this corpus
    /// inherits**. School district boundaries follow historical township and municipal lines and
    /// cross county lines freely; the calculator picks one anyway, presumably the seat of the
    /// district's administrative office. So a county grouping built on this is the department's
    /// own attribution rather than a geographic fact, and a district's pupils and valuation are
    /// not all inside the county named here.
    ///
    /// That is acceptable for comparison — the question "how does my district compare with its
    /// neighbours" wants a peer group, not a polygon — and it is not acceptable for anything that
    /// sums to a county total and calls it the county's. The site does the former.
    pub county: String,
    /// Temporary transitional aid guarantee.
    pub guarantee: Dollars,
    /// Enrolled ADM in each of [`HISTORY_YEARS`].
    pub adm_history: [Adm; 3],
    /// Current-year enrolled ADM — the last of [`HISTORY_YEARS`].
    ///
    /// Distinct from base cost ADM and used as a distinct denominator: R.C. 3317.017 multiplies
    /// the state share *per pupil* by current-year enrolled ADM, while base cost is computed on
    /// the three-year figure. Dividing the state share by the wrong one turns a hard floor at
    /// exactly 10% into a smear from 9.2% upward, which is how this was found.
    pub current_year_adm: Adm,
    /// Assessed valuation per pupil, FY2023. The only observation there is.
    pub valuation_per_pupil: Option<Dollars>,
}

impl DistrictRecord {
    /// Base cost enrolled ADM — the three-year average the formula funds on.
    #[must_use]
    pub fn base_cost_adm(&self) -> Adm {
        self.enrollment.base_cost_enrolled_adm
    }

    /// Whether the district is funded by the guarantee rather than the formula.
    #[must_use]
    pub fn on_guarantee(&self) -> bool {
        self.guarantee > 0.0
    }

    /// State aid as the district actually receives it.
    #[must_use]
    pub fn realized_aid(&self) -> Dollars {
        self.core_foundation_funding + self.guarantee
    }

    /// The level the guarantee holds this district at, recoverable only when it is on it.
    ///
    /// For a district on the guarantee this is its FY2020 receipt — a Bridge formula year Ohio
    /// froze rather than computed. For a district on formula the baseline is unobservable from
    /// here, because the guarantee is the only thing that reveals it.
    #[must_use]
    pub fn guarantee_baseline(&self) -> Option<Dollars> {
        self.on_guarantee().then(|| self.realized_aid())
    }

    /// Everything in formula aid that is not base cost: the categoricals.
    ///
    /// Held apart because they respond to different levers. Raising base cost moves the first
    /// and not the second, and the FY2022 phase-in applied 16.67% to base cost and **0%** to
    /// Disadvantaged Pupil Impact Aid — so a single "phase-in percentage" is already known to
    /// be the wrong shape for this question.
    #[must_use]
    pub fn categorical_funding(&self) -> Dollars {
        (self.core_foundation_funding - self.base_cost_state_share).max(0.0)
    }

    /// The part of [`Self::categorical_funding`] priced in the statewide average base cost.
    ///
    /// Special education, English learners and career-technical are each
    /// `weight x $8,241.61 x count x state share`, where the multiplicand is
    /// [`AVERAGE_BASE_COST_PER_PUPIL`] — one number the department computes over every district.
    /// **So a change in base cost per pupil moves these mechanically**, and a scenario that scales
    /// base cost and leaves them alone is not describing anything Ohio would do.
    ///
    /// # What is deliberately not in here
    ///
    /// **DPIA and targeted assistance**, which are the two largest categoricals at $1.89bn
    /// together. They are *index*-driven: a district's poverty share over the state's, its wealth
    /// over a median. Under a change that moves every district, numerator and denominator move
    /// together and the effect largely cancels. That is why the exposure is a quarter of base cost
    /// rather than three quarters, and it is a real property of the mechanisms rather than a
    /// simplification.
    ///
    /// **Gifted**, whose $54.4m is mostly unit funding at stated salary prices — $85,776 for a
    /// coordinator, $89,378 and $80,974 for the two specialist bands. A cost-input refresh that
    /// raised teacher salaries would raise these too, but by their own amount and not by this
    /// factor: nothing in the calculator links them. Scaling them here would be an assumption
    /// wearing the same clothes as an identity.
    ///
    /// **Preschool special education's weighted half**, $45.7m, which *is* denominated the same
    /// way — but sits outside `[H] Foundation Funding` and so outside
    /// [`Self::categorical_funding`] entirely. The scenario has never modelled it. It is the
    /// difference between the $858m the corpus recorded as exposed and the $812m reachable here.
    #[must_use]
    pub fn base_cost_denominated_categoricals(&self) -> Dollars {
        self.special_education.total()
            + self.english_learners.total()
            + self.career_technical.total()
    }

    /// The state's share of base cost, as a fraction of base cost per pupil.
    ///
    /// Per pupil on both sides, with the department's denominators: the share amount is on
    /// current-year enrolled ADM, base cost per pupil is on the three-year figure.
    #[must_use]
    pub fn state_share_fraction(&self) -> f64 {
        if self.base_cost_per_pupil <= 0.0 || self.current_year_adm <= 0.0 {
            return 0.0;
        }
        (self.base_cost_state_share / self.current_year_adm) / self.base_cost_per_pupil
    }

    /// Whether the minimum state share is what sets this district's base cost aid.
    #[must_use]
    pub fn at_minimum_state_share(&self) -> bool {
        (self.state_share_fraction() - MINIMUM_STATE_SHARE).abs() < 0.0005
    }

    /// Local capacity per pupil, implied by what the state pays.
    ///
    /// The formula is `state share per pupil = base cost per pupil − local capacity per pupil`,
    /// so local capacity is recoverable by subtraction — **except** where the floor binds,
    /// which is precisely where local capacity is highest and the subtraction would give a
    /// number smaller than the truth. `None` there rather than a plausible wrong figure.
    #[must_use]
    pub fn implied_local_capacity_per_pupil(&self) -> Option<Dollars> {
        (!self.at_minimum_state_share() && self.current_year_adm > 0.0)
            .then(|| self.base_cost_per_pupil - self.base_cost_state_share / self.current_year_adm)
    }

    /// Enrolled ADM as a series, for [`crate::series::project`].
    #[must_use]
    pub fn adm_observations(&self) -> Vec<Observation> {
        HISTORY_YEARS
            .iter()
            .zip(self.adm_history)
            .map(|(fiscal_year, value)| Observation {
                fiscal_year: *fiscal_year,
                value,
            })
            .collect()
    }
}

/// Column positions in the fixture. One place to edit when the header changes.
mod column {
    pub const IRN: usize = 0;
    pub const NAME: usize = 1;
    pub const BASE_COST_ADM: usize = 2;
    pub const BUILDINGS: usize = 3;
    pub const KINDERGARTEN: usize = 4;
    pub const GRADES_1_3: usize = 5;
    pub const GRADES_4_8: usize = 6;
    pub const GRADES_9_12: usize = 7;
    pub const CAREER_TECHNICAL: usize = 8;
    pub const GRADES_9_12_TOTAL: usize = 9;
    pub const FUNDED_CLASSROOM: usize = 10;
    pub const FUNDED_SPECIAL: usize = 11;
    pub const AGGREGATE_BASE_COST: usize = 13;
    pub const BASE_COST_PER_PUPIL: usize = 14;
    pub const GUARANTEE: usize = 15;
    pub const ADM_FY24: usize = 16;
    pub const ADM_FY25: usize = 17;
    pub const ADM_FY26: usize = 18;
    pub const VALUATION_PER_PUPIL: usize = 19;
    pub const CORE_FOUNDATION: usize = 20;
    pub const BASE_COST_STATE_SHARE: usize = 21;
    pub const TOTAL_STATE_SUPPORT: usize = 22;
    pub const TOTAL_TRANSFERS: usize = 23;
    /// `S - Educational Service Center`, split out of the transfer total.
    pub const SERVICE_CENTER: usize = 24;
    /// `T - Other Adjustments`, the residual and the only unlabelled line in the report.
    pub const OTHER_ADJUSTMENTS: usize = 25;
    pub const NET_STATE_FUNDING: usize = 26;
    /// `[b1] Per Pupil Capacity Amount` from `Detail_SFPR` — the department's own local capacity.
    pub const CAPACITY_PER_PUPIL: usize = 27;
    /// `[b4] State Share Percentage`.
    pub const STATE_SHARE_PERCENTAGE: usize = 28;
    pub const VALUATION_TY25: usize = 29;
    pub const VALUATION_TY24: usize = 30;
    pub const VALUATION_TY23: usize = 31;
    pub const AGI_TY24: usize = 32;
    pub const AGI_TY23: usize = 33;
    pub const AGI_TY22: usize = 34;
    pub const TAX_RETURNS: usize = 35;
    pub const FEDERAL_MEDIAN_INCOME: usize = 36;
    pub const STATEWIDE_MEDIAN_INCOME: usize = 37;
    pub const BENCHMARK_RATIO: usize = 38;
    pub const PUBLISHED_CAPACITY_RATE: usize = 39;
    pub const TARGETED_ASSISTANCE: usize = 40;
    pub const SPECIAL_EDUCATION: usize = 41;
    pub const DPIA: usize = 42;
    pub const ENGLISH_LEARNERS: usize = 43;
    pub const GIFTED: usize = 44;
    pub const CATEGORICAL_CTE: usize = 45;
    /// Six special education ADM counts, then the six aid amounts they produce.
    pub const SPED_FIRST_ADM: usize = 46;
    pub const SPED_FIRST_AID: usize = 52;
    pub const DPIA_ECON_DISADVANTAGED_ADM: usize = 58;
    pub const DPIA_DIRECTLY_CERTIFIED_ADM: usize = 59;
    pub const DPIA_WEIGHTED_ADM: usize = 60;
    pub const DPIA_PERCENTAGE: usize = 61;
    pub const DPIA_INDEX: usize = 62;
    /// Five career-technical FTE counts, then the five aid amounts, then associated services.
    pub const CTE_FIRST_FTE: usize = 63;
    pub const CTE_FIRST_AID: usize = 68;
    pub const CTE_ASSOCIATED_SERVICES: usize = 73;
    /// Three English learner ADM counts, then the three aid amounts.
    pub const EL_FIRST_ADM: usize = 74;
    pub const EL_FIRST_AID: usize = 77;
    pub const GIFTED_ADM_K6: usize = 80;
    pub const GIFTED_FTE_K8: usize = 81;
    pub const GIFTED_FTE_9_12: usize = 82;
    pub const GIFTED_IDENTIFICATION: usize = 83;
    pub const GIFTED_REFERRAL: usize = 84;
    pub const GIFTED_PROFESSIONAL_DEVELOPMENT: usize = 85;
    pub const GIFTED_COORDINATOR_UNITS: usize = 86;
    pub const GIFTED_COORDINATOR_AID: usize = 87;
    pub const GIFTED_SPECIALIST_K8_UNITS: usize = 88;
    pub const GIFTED_SPECIALIST_K8_AID: usize = 89;
    pub const GIFTED_SPECIALIST_9_12_UNITS: usize = 90;
    pub const GIFTED_SPECIALIST_9_12_AID: usize = 91;
    pub const TA_OPEN_ENROLLMENT_IN: usize = 92;
    pub const TA_OPEN_ENROLLMENT_OUT: usize = 93;
    pub const TA_FY19_WEALTH_INDEX: usize = 94;
    pub const TA_FY19_ENROLLED_ADM: usize = 95;
    pub const TA_FY19_TOTAL_ADM: usize = 96;
    pub const TA_PROPERTY_VALUATION: usize = 97;
    pub const TA_FEDERAL_GROSS_INCOME: usize = 98;
    pub const TA_WEIGHTED_WEALTH: usize = 99;
    pub const TA_CAPACITY_INDEX: usize = 100;
    pub const TA_CAPACITY_AMOUNT: usize = 101;
    pub const TA_WEALTH_PER_PUPIL: usize = 102;
    pub const TA_WEALTH_INDEX: usize = 103;
    pub const TA_WEALTH_AMOUNT: usize = 104;
    pub const TA_SUPPLEMENTAL: usize = 105;
    pub const TA_SUPPLEMENT_ELIGIBLE: usize = 106;
    /// `[a] Enrolled ADM` — the count the four categorical sheets are paid on.
    pub const CATEGORICAL_ENROLLED_ADM: usize = 107;
    pub const COUNTY: usize = 108;
    pub const PERFORMANCE_STARS: usize = 109;
    pub const PERFORMANCE_PROGRESS: usize = 110;
    pub const PERFORMANCE_PROGRESS_PRIOR: usize = 111;
    pub const PERFORMANCE_ELIGIBLE: usize = 112;
    pub const PERFORMANCE_SUPPLEMENT: usize = 113;
    pub const BASE_FUNDING_SUPPLEMENT: usize = 114;
    pub const ADM_FY23: usize = 115;
    pub const ENROLLMENT_CHANGE_THREE_YEAR: usize = 116;
    pub const GROWTH_SUPPLEMENT_ELIGIBLE: usize = 117;
    pub const ENROLLMENT_GROWTH_SUPPLEMENT: usize = 118;
    /// Transportation: eleven inputs then nine payments, in sheet order.
    pub const TRANS_FIRST_INPUT: usize = 119;
    pub const TRANS_FIRST_PAYMENT: usize = 132;
    /// The guarantee's machinery, emitted before the preschool block.
    pub const FUNDING_BASE: usize = 141;
    pub const FUNDING_BASE_ECON_DIS: usize = 142;
    pub const OPEN_ENROLLMENT_PRIOR: usize = 143;
    pub const OPEN_ENROLLMENT_CURRENT: usize = 144;
    pub const OPEN_ENROLLMENT_THRESHOLD: usize = 145;
    pub const OPEN_ENROLLMENT_ADJUSTMENT: usize = 146;
    pub const FY21_FUNDING_BASE: usize = 147;
    pub const FORMULA_TRANSITION_SUPPLEMENT: usize = 148;
    /// Preschool special education: six counts, six amounts, and their total.
    pub const PREK_FIRST_ADM: usize = 149;
    pub const PREK_FIRST_AID: usize = 155;
    pub const PREK_TOTAL: usize = 161;
}

/// The header this loader expects, so a fixture reshaped without updating [`column`] fails
/// loudly rather than reading the wrong column.
///
/// Shared with [`crate::crosswalk`], which reads the same fixture for its IRN column. One
/// constant per fixture rather than one per reader: two copies could drift apart, and the
/// stale one would keep asserting a header the file no longer has.
pub(crate) const EXPECTED_HEADER: &str = "irn,district,base_cost_enrolled_adm,school_buildings,\
adm_kindergarten,adm_grades_1_3,adm_grades_4_8_non_cte,adm_grades_9_12_non_cte,adm_cte,\
adm_grades_9_12_total,funded_classroom_teachers,funded_special_teachers,teacher_base_cost,\
aggregate_base_cost,base_cost_per_pupil,temp_transitional_aid_guarantee,enrolled_adm_fy24,\
enrolled_adm_fy25,enrolled_adm_fy26,assessed_valuation_per_pupil_fy23,core_foundation_funding,\
base_cost_state_share,total_state_support,total_transfers,service_center_charge,other_adjustments,net_state_funding,capacity_per_pupil,state_share_percentage,valuation_ty25,valuation_ty24,valuation_ty23,agi_ty24,agi_ty23,agi_ty22,tax_returns,federal_median_income,statewide_median_income,benchmark_ratio,capacity_rate,targeted_assistance,special_education,dpia,english_learners,gifted,career_technical,sped_adm_cat1,sped_adm_cat2,sped_adm_cat3,sped_adm_cat4,sped_adm_cat5,sped_adm_cat6,sped_aid_cat1,sped_aid_cat2,sped_aid_cat3,sped_aid_cat4,sped_aid_cat5,sped_aid_cat6,dpia_econ_disadvantaged_adm,dpia_directly_certified_adm,dpia_weighted_adm,dpia_percentage,dpia_index,\
cte_fte_cat1,cte_fte_cat2,cte_fte_cat3,cte_fte_cat4,cte_fte_cat5,\
cte_aid_cat1,cte_aid_cat2,cte_aid_cat3,cte_aid_cat4,cte_aid_cat5,cte_associated_services,\
el_adm_cat1,el_adm_cat2,el_adm_cat3,el_aid_cat1,el_aid_cat2,el_aid_cat3,\
gifted_adm_k6,gifted_fte_k8,gifted_fte_9_12,gifted_identification,gifted_referral,\
gifted_professional_development,gifted_coordinator_units,gifted_coordinator_aid,\
gifted_specialist_k8_units,gifted_specialist_k8_aid,gifted_specialist_9_12_units,\
gifted_specialist_9_12_aid,ta_open_enrollment_in,ta_open_enrollment_out,ta_fy19_wealth_index,\
ta_fy19_enrolled_adm,ta_fy19_total_adm,ta_property_valuation,ta_federal_gross_income,\
ta_weighted_wealth,ta_capacity_index,ta_capacity_amount,ta_wealth_per_pupil,ta_wealth_index,\
ta_wealth_amount,ta_supplemental,ta_supplement_eligible,categorical_enrolled_adm,county,\
performance_stars,performance_progress,performance_progress_prior,performance_eligible,\
performance_supplement,base_funding_supplement,enrolled_adm_fy23,enrollment_change_three_year,\
growth_supplement_eligible,enrollment_growth_supplement,\
trans_public_riders,trans_nonpublic_riders,trans_community_riders,trans_weighted_riders,\
trans_mass_transit_riders,trans_other_riders,trans_bus_miles,trans_assigned_buses,\
trans_rider_capacity_target,trans_efficiency_index,trans_district_density,trans_square_miles,trans_reported_sped_cost,trans_school_bus,\
trans_mass_transit,trans_other,trans_efficiency,trans_density,trans_fy21_base,trans_guarantee,\
trans_total,trans_special_education,\
funding_base,funding_base_econ_dis,open_enrollment_fte_prior,open_enrollment_fte_current,\
open_enrollment_threshold,open_enrollment_adjustment,fy21_funding_base,\
formula_transition_supplement,\
prek_sped_adm_cat1,prek_sped_adm_cat2,prek_sped_adm_cat3,prek_sped_adm_cat4,prek_sped_adm_cat5,\
prek_sped_adm_cat6,prek_sped_aid_cat1,prek_sped_aid_cat2,prek_sped_aid_cat3,prek_sped_aid_cat4,\
prek_sped_aid_cat5,prek_sped_aid_cat6,prek_sped_total";

/// Every district in the department's FY2027 model.
///
/// # Panics
///
/// If the embedded fixture's header is not the one this loader was written against. That is a
/// build-time mistake, not a runtime condition, and reading shifted columns silently would put
/// wrong numbers into a scenario.
#[must_use]
pub fn panel() -> Vec<DistrictRecord> {
    let mut lines = FIXTURE.lines();
    let header = lines.next().unwrap_or_default().trim();
    assert_eq!(
        header, EXPECTED_HEADER,
        "the FY2027 fixture header changed; update project::panel::column before reading it"
    );

    lines
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(',').map(str::trim).collect();
            let number = |index: usize| {
                fields
                    .get(index)
                    .and_then(|value| value.parse::<f64>().ok())
            };
            let required = |index: usize| number(index).unwrap_or(0.0);

            let base_cost_adm = number(column::BASE_COST_ADM)?;
            if base_cost_adm <= 0.0 {
                return None;
            }
            Some(DistrictRecord {
                irn: fields.get(column::IRN)?.to_string(),
                name: fields.get(column::NAME)?.to_string(),
                enrollment: DistrictEnrollment {
                    kindergarten: required(column::KINDERGARTEN),
                    grades_1_3: required(column::GRADES_1_3),
                    grades_4_8: required(column::GRADES_4_8),
                    grades_9_12: required(column::GRADES_9_12),
                    career_technical: required(column::CAREER_TECHNICAL),
                    grades_9_12_total: required(column::GRADES_9_12_TOTAL),
                    base_cost_enrolled_adm: base_cost_adm,
                    open_buildings: required(column::BUILDINGS),
                    athletics_eligible: true,
                },
                funded_classroom_teachers: required(column::FUNDED_CLASSROOM),
                funded_special_teachers: required(column::FUNDED_SPECIAL),
                aggregate_base_cost: required(column::AGGREGATE_BASE_COST),
                base_cost_per_pupil: required(column::BASE_COST_PER_PUPIL),
                base_cost_state_share: required(column::BASE_COST_STATE_SHARE),
                core_foundation_funding: required(column::CORE_FOUNDATION),
                total_state_support: required(column::TOTAL_STATE_SUPPORT),
                total_transfers: required(column::TOTAL_TRANSFERS),
                service_center_charge: required(column::SERVICE_CENTER),
                other_adjustments: required(column::OTHER_ADJUSTMENTS),
                published_capacity_per_pupil: number(column::CAPACITY_PER_PUPIL),
                published_state_share: number(column::STATE_SHARE_PERCENTAGE),
                valuation_three_year: [
                    required(column::VALUATION_TY25),
                    required(column::VALUATION_TY24),
                    required(column::VALUATION_TY23),
                ],
                agi_three_year: [
                    required(column::AGI_TY24),
                    required(column::AGI_TY23),
                    required(column::AGI_TY22),
                ],
                tax_returns: number(column::TAX_RETURNS),
                median_income: number(column::FEDERAL_MEDIAN_INCOME),
                statewide_median_income: number(column::STATEWIDE_MEDIAN_INCOME),
                benchmark_ratio: number(column::BENCHMARK_RATIO),
                published_capacity_rate: number(column::PUBLISHED_CAPACITY_RATE),
                categoricals: Categoricals {
                    targeted_assistance: required(column::TARGETED_ASSISTANCE),
                    special_education: required(column::SPECIAL_EDUCATION),
                    dpia: required(column::DPIA),
                    english_learners: required(column::ENGLISH_LEARNERS),
                    gifted: required(column::GIFTED),
                    career_technical: required(column::CATEGORICAL_CTE),
                },
                dpia: Dpia {
                    economically_disadvantaged_adm: required(column::DPIA_ECON_DISADVANTAGED_ADM),
                    directly_certified_adm: required(column::DPIA_DIRECTLY_CERTIFIED_ADM),
                    weighted_adm: required(column::DPIA_WEIGHTED_ADM),
                    percentage: required(column::DPIA_PERCENTAGE),
                    index: required(column::DPIA_INDEX),
                },
                special_education: SpecialEducation {
                    adm: std::array::from_fn(|k| required(column::SPED_FIRST_ADM + k)),
                    aid: std::array::from_fn(|k| required(column::SPED_FIRST_AID + k)),
                },
                targeted_assistance: TargetedAssistance {
                    open_enrollment_in: required(column::TA_OPEN_ENROLLMENT_IN),
                    open_enrollment_out: required(column::TA_OPEN_ENROLLMENT_OUT),
                    fy19_wealth_index: required(column::TA_FY19_WEALTH_INDEX),
                    fy19_enrolled_adm: required(column::TA_FY19_ENROLLED_ADM),
                    fy19_total_adm: required(column::TA_FY19_TOTAL_ADM),
                    property_valuation: required(column::TA_PROPERTY_VALUATION),
                    federal_gross_income: required(column::TA_FEDERAL_GROSS_INCOME),
                    weighted_wealth: required(column::TA_WEIGHTED_WEALTH),
                    capacity_index: required(column::TA_CAPACITY_INDEX),
                    capacity_amount: required(column::TA_CAPACITY_AMOUNT),
                    wealth_per_pupil: required(column::TA_WEALTH_PER_PUPIL),
                    wealth_index: required(column::TA_WEALTH_INDEX),
                    wealth_amount: required(column::TA_WEALTH_AMOUNT),
                    supplemental: required(column::TA_SUPPLEMENTAL),
                    supplement_eligible: required(column::TA_SUPPLEMENT_ELIGIBLE) > 0.5,
                },
                career_technical: CareerTechnical {
                    fte: std::array::from_fn(|k| required(column::CTE_FIRST_FTE + k)),
                    aid: std::array::from_fn(|k| required(column::CTE_FIRST_AID + k)),
                    associated_services: required(column::CTE_ASSOCIATED_SERVICES),
                },
                english_learners: EnglishLearners {
                    adm: std::array::from_fn(|k| required(column::EL_FIRST_ADM + k)),
                    aid: std::array::from_fn(|k| required(column::EL_FIRST_AID + k)),
                },
                gifted: Gifted {
                    adm_k6: required(column::GIFTED_ADM_K6),
                    fte_k8: required(column::GIFTED_FTE_K8),
                    fte_9_12: required(column::GIFTED_FTE_9_12),
                    identification: required(column::GIFTED_IDENTIFICATION),
                    referral: required(column::GIFTED_REFERRAL),
                    professional_development: required(column::GIFTED_PROFESSIONAL_DEVELOPMENT),
                    coordinator_units: required(column::GIFTED_COORDINATOR_UNITS),
                    coordinator_aid: required(column::GIFTED_COORDINATOR_AID),
                    specialist_k8_units: required(column::GIFTED_SPECIALIST_K8_UNITS),
                    specialist_k8_aid: required(column::GIFTED_SPECIALIST_K8_AID),
                    specialist_9_12_units: required(column::GIFTED_SPECIALIST_9_12_UNITS),
                    specialist_9_12_aid: required(column::GIFTED_SPECIALIST_9_12_AID),
                },
                categorical_enrolled_adm: required(column::CATEGORICAL_ENROLLED_ADM),
                county: fields.get(column::COUNTY).unwrap_or(&"").to_string(),
                performance: PerformanceSupplement {
                    stars: number(column::PERFORMANCE_STARS),
                    progress: number(column::PERFORMANCE_PROGRESS),
                    progress_prior: number(column::PERFORMANCE_PROGRESS_PRIOR),
                    eligible: required(column::PERFORMANCE_ELIGIBLE) > 0.5,
                    amount: required(column::PERFORMANCE_SUPPLEMENT),
                },
                transportation: {
                    let input = |k: usize| required(column::TRANS_FIRST_INPUT + k);
                    let pay = |k: usize| required(column::TRANS_FIRST_PAYMENT + k);
                    Transportation {
                        public_riders: input(0),
                        nonpublic_riders: input(1),
                        community_riders: input(2),
                        weighted_riders: input(3),
                        mass_transit_riders: input(4),
                        other_riders: input(5),
                        bus_miles: input(6),
                        assigned_buses: input(7),
                        rider_capacity_target: input(8),
                        efficiency_index: input(9),
                        district_density: input(10),
                        square_miles: input(11),
                        reported_sped_cost: input(12),
                        school_bus: pay(0),
                        mass_transit: pay(1),
                        other: pay(2),
                        efficiency: pay(3),
                        density: pay(4),
                        fy21_base: pay(5),
                        guarantee: pay(6),
                        total: pay(7),
                        special_education: pay(8),
                    }
                },
                transition: Transition {
                    funding_base: required(column::FUNDING_BASE),
                    funding_base_econ_dis: required(column::FUNDING_BASE_ECON_DIS),
                    open_enrollment_prior: required(column::OPEN_ENROLLMENT_PRIOR),
                    open_enrollment_current: required(column::OPEN_ENROLLMENT_CURRENT),
                    open_enrollment_threshold: required(column::OPEN_ENROLLMENT_THRESHOLD),
                    open_enrollment_adjustment: required(column::OPEN_ENROLLMENT_ADJUSTMENT),
                    fy21_funding_base: required(column::FY21_FUNDING_BASE),
                    transition_supplement: required(column::FORMULA_TRANSITION_SUPPLEMENT),
                },
                preschool_special_education: PreschoolSpecialEducation {
                    adm: std::array::from_fn(|k| required(column::PREK_FIRST_ADM + k)),
                    aid: std::array::from_fn(|k| required(column::PREK_FIRST_AID + k)),
                    total: required(column::PREK_TOTAL),
                },
                supplements: Supplements {
                    base_funding: required(column::BASE_FUNDING_SUPPLEMENT),
                    adm_fy23: required(column::ADM_FY23),
                    enrollment_change: required(column::ENROLLMENT_CHANGE_THREE_YEAR),
                    growth_eligible: required(column::GROWTH_SUPPLEMENT_ELIGIBLE) > 0.5,
                    growth: required(column::ENROLLMENT_GROWTH_SUPPLEMENT),
                },
                net_state_funding: required(column::NET_STATE_FUNDING),
                guarantee: required(column::GUARANTEE),
                adm_history: [
                    required(column::ADM_FY24),
                    required(column::ADM_FY25),
                    required(column::ADM_FY26),
                ],
                current_year_adm: required(column::ADM_FY26),
                valuation_per_pupil: number(column::VALUATION_PER_PUPIL),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every block of the fixture lands in the columns the header names.
    ///
    /// The header assertion in [`panel`] catches a *renamed* column and the row-length check in
    /// `connect` catches a *missing* one. Neither catches the two disagreeing about **order**,
    /// which is what happened when the transition block was emitted before the preschool block and
    /// listed after it: every value in both blocks was in the wrong field, the row length was
    /// right, the header was right, and the fixture parsed.
    ///
    /// A magnitude check per block is the cheap guard. Each of these is a statewide total that
    /// could not plausibly be produced by a neighbouring column, so a shifted block fails here
    /// rather than in whichever test happens to touch it first.
    #[test]
    fn each_block_of_the_fixture_lands_in_its_own_columns() {
        let panel = panel();
        let total = |pick: fn(&DistrictRecord) -> f64| panel.iter().map(pick).sum::<f64>();

        for (label, got, low, high) in [
            (
                "base cost state share",
                total(|r| r.base_cost_state_share),
                3.0e9,
                4.5e9,
            ),
            (
                "targeted assistance",
                total(|r| r.categoricals.targeted_assistance),
                1.2e9,
                1.5e9,
            ),
            (
                "special education",
                total(|r| r.special_education.total()),
                0.6e9,
                0.8e9,
            ),
            ("DPIA", total(|r| r.categoricals.dpia), 0.4e9, 0.6e9),
            ("gifted", total(|r| r.gifted.total()), 40e6, 70e6),
            (
                "career-technical",
                total(|r| r.career_technical.total()),
                40e6,
                70e6,
            ),
            (
                "English learners",
                total(|r| r.english_learners.total()),
                25e6,
                50e6,
            ),
            (
                "transportation",
                total(|r| r.transportation.total),
                0.6e9,
                0.8e9,
            ),
            (
                "special education transport",
                total(|r| r.transportation.special_education),
                150e6,
                220e6,
            ),
            (
                "preschool special education",
                total(|r| r.preschool_special_education.total),
                130e6,
                170e6,
            ),
            (
                "performance supplement",
                total(|r| r.performance.amount),
                45e6,
                70e6,
            ),
            (
                "base supplement",
                total(|r| r.supplements.base_funding),
                45e6,
                70e6,
            ),
            (
                "growth supplement",
                total(|r| r.supplements.growth),
                25e6,
                55e6,
            ),
            (
                "transition supplement",
                total(|r| r.transition.transition_supplement),
                50e6,
                80e6,
            ),
            ("guarantee", total(|r| r.guarantee), 0.7e9, 1.1e9),
        ] {
            assert!(
                got > low && got < high,
                "{label} totals {got:.0}, outside the band this block should produce — most \
                 likely the fixture's columns and its header disagree about order"
            );
        }
    }

    #[test]
    fn loads_every_district_in_the_model() {
        assert_eq!(panel().len(), 609);
    }

    #[test]
    fn base_cost_adm_is_the_greater_of_the_three_year_average_and_the_current_year() {
        // Not the plain average. For 105 of 609 districts the published figure is the current
        // year instead, and in every one of those the current year is the larger.
        //
        // The asymmetry is the mechanism: a growing district is funded on this year's students
        // immediately, and a shrinking one keeps two years of students it no longer has. Both
        // directions are cushioned in the same direction, toward the district.
        let panel = panel();
        let mut on_current_year = 0;
        for record in &panel {
            let average = record.adm_history.iter().sum::<f64>() / 3.0;
            let expected = average.max(record.current_year_adm);
            assert!(
                (expected - record.base_cost_adm()).abs() < 0.01,
                "{}: max(avg {average:.4}, FY26 {:.4}) vs published {:.4}",
                record.name,
                record.current_year_adm,
                record.base_cost_adm()
            );
            if record.current_year_adm > average + 0.01 {
                on_current_year += 1;
            }
        }
        assert_eq!(on_current_year, 105);
    }

    #[test]
    fn formula_aid_splits_into_base_cost_and_categoricals() {
        for record in panel() {
            let recombined = record.base_cost_state_share + record.categorical_funding();
            assert!(
                (recombined - record.core_foundation_funding).abs() < 0.01,
                "{}: {recombined} vs {}",
                record.name,
                record.core_foundation_funding
            );
        }
    }

    #[test]
    fn the_minimum_state_share_binds_for_the_wealthiest_districts() {
        let panel = panel();
        let at_floor: Vec<&DistrictRecord> = panel
            .iter()
            .filter(|r| r.at_minimum_state_share())
            .collect();
        assert_eq!(
            at_floor.len(),
            138,
            "the FY2027 model puts 138 districts on the minimum state share"
        );
        // And nothing below it: a floor, not a tendency.
        let lowest = panel
            .iter()
            .map(DistrictRecord::state_share_fraction)
            .filter(|f| *f > 0.0)
            .fold(f64::MAX, f64::min);
        assert!(
            lowest >= MINIMUM_STATE_SHARE - 0.0005,
            "a district below the floor at {lowest}"
        );
        // Wealth is why they are there: every one should be above the statewide median.
        let mut valuations: Vec<f64> = panel.iter().filter_map(|r| r.valuation_per_pupil).collect();
        valuations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = valuations[valuations.len() / 2];
        let above = at_floor
            .iter()
            .filter(|r| r.valuation_per_pupil.is_some_and(|v| v > median))
            .count();
        assert!(
            above * 4 >= at_floor.len() * 3,
            "only {above} of {} floor districts are above median wealth",
            at_floor.len()
        );
    }

    #[test]
    fn local_capacity_is_not_recovered_where_the_floor_binds() {
        // Subtraction gives a number there, and the number is wrong: the floor has already
        // truncated what it is derived from.
        for record in panel() {
            if record.at_minimum_state_share() {
                assert_eq!(
                    record.implied_local_capacity_per_pupil(),
                    None,
                    "{}",
                    record.name
                );
            }
        }
    }

    #[test]
    fn implied_local_capacity_is_positive_and_below_base_cost_where_it_is_recoverable() {
        for record in panel() {
            let Some(capacity) = record.implied_local_capacity_per_pupil() else {
                continue;
            };
            assert!(
                capacity > -1.0 && capacity < record.base_cost_per_pupil,
                "{}: {capacity}",
                record.name
            );
        }
    }

    #[test]
    fn a_guarantee_baseline_exists_only_for_a_guaranteed_district() {
        let panel = panel();
        for record in &panel {
            assert_eq!(record.guarantee_baseline().is_some(), record.on_guarantee());
        }
        assert!(panel.iter().filter(|r| r.on_guarantee()).count() > 250);
    }

    #[test]
    fn the_adm_history_reads_as_a_dated_series() {
        let record = &panel()[0];
        let observations = record.adm_observations();
        assert_eq!(observations.len(), 3);
        assert_eq!(observations[0].fiscal_year, FiscalYear(2024));
        assert_eq!(observations[2].fiscal_year, FiscalYear(2026));
    }

    /// The guarantee a clawback reduced is two numbers, and this returns the other one.
    ///
    /// Kept and tested rather than deleted: nothing called it, but its doc makes a claim about
    /// how a district's guarantee is reported, and an untested public function whose doc makes a
    /// claim is a claim nobody checks.
    #[test]
    fn a_clawed_back_guarantee_reports_the_amount_before_the_charge() {
        let districts = panel();
        let charged: Vec<_> = districts
            .iter()
            .filter(|d| d.transition.open_enrollment_adjustment > 0.0)
            .collect();
        assert!(
            !charged.is_empty(),
            "no district in the FY2027 panel carries an open-enrolment clawback, so this test \
             would pass vacuously"
        );

        for d in &charged {
            let after = 1_000_000.0;
            let before = d
                .transition
                .guarantee_before_clawback(after)
                .expect("a district with an adjustment has a before-clawback figure");
            assert!(
                before > after,
                "{}: the guarantee before the clawback must exceed the one after it",
                d.name
            );
            assert!(
                (before - after - d.transition.open_enrollment_adjustment).abs() < 1e-9,
                "{}: the difference between the two figures is the adjustment itself",
                d.name
            );
        }

        // The other half of the claim: where nothing was charged there is no second number,
        // and `None` says so rather than repeating the first.
        let untouched = districts
            .iter()
            .find(|d| d.transition.open_enrollment_adjustment == 0.0)
            .expect("some district was not clawed back");
        assert_eq!(
            untouched.transition.guarantee_before_clawback(1_000_000.0),
            None
        );
    }
}
