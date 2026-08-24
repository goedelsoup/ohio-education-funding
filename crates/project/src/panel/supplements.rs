//! The five components paid *outside* foundation funding.
//!
//! Real money, paid to the district, and outside the base the guarantee holds it at: pupil
//! transportation, preschool special education, special education transportation, the
//! performance supplement, and the two flat supplements. The six components inside the base are
//! in [`super::categoricals`].
//!
//! Two of them are prorated to an appropriation rather than computed to a formula, which is the
//! reason this boundary is worth a file: a component whose amount depends on what the General
//! Assembly appropriated cannot be simulated by changing a weight.

use edfund_core::{Adm, Dollars};

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
/// What the appropriation could actually cover of the **special education** transportation
/// entitlement.
///
/// Not [`TRANSPORT_PRORATION`], and the two are the answer to a question that was open long
/// enough to be filed: are these one parameter with a stale copy, or two quantities? Two. The
/// redbook says so on ALI 200502's own purpose page — *"Transportation for special education
/// students who cannot be transported by regular school bus is reimbursed **separately through a
/// formula funded outside state foundation aid**"* — so the regular entitlement and this one are
/// paid by different mechanisms and prorate independently. The data agrees: this factor
/// reproduces `trans_special_education` from `trans_reported_sped_cost` to within half a cent for
/// every district, and the regular total needs no factor at all.
pub const TRANSPORT_SPED_PRORATION: f64 = 0.917_459_740_976_215;

/// What the appropriation covered of the **regular** transportation entitlement: all of it.
///
/// A dial that has been used and is not being used. Payments under the transportation formula are
/// a component of state foundation aid, funded through GRF ALI 200502, and where the
/// appropriation falls short the department scales them; in the FY2027 model it did not have to.
/// The implied factor recovered from the department's own columns —
/// `total / (before_proration() + guarantee)` — is 1.0 to within 2e-7 across all 605 districts
/// that have an entitlement, which is float noise rather than a reduction.
///
/// Kept as a named constant rather than left implicit because 1.0 is a *finding* about a year and
/// not a property of the formula, and because the value that makes it interesting is the one it
/// takes when a biennium underfunds the line.
///
/// This is the last of 57 statutory parameters that lived in `connect`, byte-identical to this
/// module's and linked to it by nothing the compiler could see. The other 56 went in `546d19d`;
/// this one waited on the question its sibling above now answers.
pub const TRANSPORT_PRORATION: f64 = 1.0;

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
/// # The proration factor and the limit beside it were carried over together
///
/// The sheet carries a **limit of $147,500,000** in a cell beside the factor, which makes this the
/// clearest statement in the whole workbook of what a proration is: a budget divided by an
/// entitlement. At the stated factor of 0.96854448 the program totals **$148,408,184** —
/// **$908,184 over that cell**. The factor that would reach it is 0.96261747.
///
/// **But the cell is not the FY2027 appropriation.** $147,500,000 is the FY2025 estimate; the
/// enacted act's FY2026 and FY2027 remainder for this program is $153,976,832, which the program
/// is $5,568,648 *under*. See `tests/the_appropriation_behind_the_proration.rs`. So no proration
/// arises on the year being modelled, and the factor and the limit were carried over from the
/// prior biennium together. A published proration factor is not evidence that a proration applied.
///
/// A third cell on the same sheet states $146,708,228.07, matching neither the column above it nor
/// either limit. As published the figures on this sheet are mutually inconsistent, and that is
/// worth recording rather than smoothing over.
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
/// The limit the sheet prints beside the factor, which the program exceeds by $908,184.
///
/// **This is the FY2025 estimate, not the FY2027 appropriation.** The enacted FY2026 and FY2027
/// figure is $153,976,832 — see [`PREK_SPED_APPROPRIATION_FY27`] — and the program is $5,568,648
/// under it, so no proration arises. Kept because it is what the workbook states and what the
/// factor was set against.
pub const PREK_SPED_APPROPRIATION: Dollars = 147_500_000.0;
/// The enacted FY2026 and FY2027 remainder for preschool special education.
///
/// From the act's LSC analysis rather than the calculator, which never prints it. The program at
/// the stated factor totals $148,408,184 against this, $5,568,648 under.
pub const PREK_SPED_APPROPRIATION_FY27: Dollars = 153_976_832.0;

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
    /// `[H2]` — the **FY2020** amount the guarantee compares foundation funding against.
    ///
    /// It is also the origin the phase-in interpolates from: R.C. 3317.022 pays
    /// `funding base + pct x (computed - funding base)`, so this column sets what a district
    /// receives at a 0% phase-in as well as the floor under it. R.C. 3317.02(N)(1) builds it from
    /// FY2020 funding before the Executive Order 2020-19D reductions.
    ///
    /// Negative for one district: Richmond Heights Local, -$40,179.23, where the FY2020 community
    /// school and scholarship deductions (N)(1)(b) subtracts came to more than the funding did.
    pub funding_base: Dollars,
    /// `[H3]` — the DPIA part of that base, which the phase-in dials separately.
    ///
    /// R.C. 3317.02(N)(2) anchors it to the district's **FY2019** DPIA payment rather than to
    /// FY2020, and (N)(1)(b)(i) subtracts the same figure from the general slice, so the two
    /// partition `[H2]` exactly.
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

#[cfg(test)]
mod tests {
    use crate::panel::panel;

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
