//! What the Fair School Funding Plan pays a district **to do**, at the margin.
//!
//! [`crate::hold_harmless`] inventoried the four devices that hold a district above its computed
//! amount, [`crate::guarantee_origin`] partitioned the districts they reach, and
//! [`crate::decline_adjustment`] priced four responses to one of those partitions. Every one of
//! those measurements is backward-looking: who is held, why, and what retiring a device would
//! cost. None of them asks what the system pays a district to do next year.
//!
//! This is that question, and it is answered by perturbation: the levers already exist, and
//! [`crate::policy::apply`] already takes the pupil count as an argument, so a margin is the
//! difference between two runs rather than a new model.
//!
//! # Four margins, and the formula prices them across seven orders of magnitude
//!
//! | margin | what the board actually decides | the median district's price | population |
//! |---|---|--:|--:|
//! | one resident pupil, in a district `[K]` holds | nothing — residency is not chosen | **$0.00** | 144 |
//! | one resident pupil, in a district `[I]` holds | nothing | **$503.40** | 167 |
//! | one resident pupil, on formula | nothing | **$8,516.08** | 298 |
//! | one dollar of assessed valuation | abatement consent, and little else | **$0.0123** | 268 |
//! | one open-enrolment FTE retained | admission policy, R.C. 3313.98 | **$8,241.61** | 9 |
//! | the pupil that clears `[M]`'s threshold | admission policy | **$516,984** | 1 |
//!
//! Read down the column and the shape is the finding. **The three margins no district chooses are
//! priced at zero, at transportation, and at a per-pupil amount; the two a board votes on are
//! priced at the whole statewide average base cost and at half a million dollars for one child.**
//! It is the exact inverse of what an incentive system would be built to do, and no part of it
//! was designed — each price is a side effect of paying a level rather than an increment.
//!
//! # Whether any of it is an incentive, which is a separate question from its price
//!
//! The brief this module answers anticipated that the honest result might be *nothing*, because
//! nobody chooses their enrolment or their valuation. Half of that holds and half of it does not,
//! and the line runs between the first four rows of the table and the last two.
//!
//! **A price on something nobody chooses is not an incentive, and it is still a fact about the
//! formula.** No board votes on how many children live in its district. What the zero in the first
//! row means is not that Ohio pays a district to shrink — it is that a held district is paid
//! *nothing for the marginal pupil*, so every additional child it teaches is a pure cost against a
//! cheque that does not move. [`crate::enrollment_decline`] found the median member of its cluster
//! drawing $464.06 a pupil for enrolment lost and called that an outcome rather than an incentive.
//! It is: the average is $464.06 and the margin is zero, and the two are consistent because the
//! guarantee pays a level. A district cannot act on an average.
//!
//! **The last two rows are different, and this is where the expectation fails.** Which
//! open-enrolment pupils a district accepts is a policy its board adopts under R.C. 3313.98, in a
//! year, and both of the formula's largest marginal prices are attached to it. That is not a claim
//! that any board has acted on either; it is the claim that these two are the only prices here a
//! board *could* act on, and they are the two largest by three orders of magnitude.
//!
//! # 1. The resident pupil, and the three regimes it falls into
//!
//! One pupil fewer, everything else held, measured on
//! [`crate::policy::Outcome::total_state_support`] — realized aid, transportation and `[K]`:
//!
//! | | districts | ADM | median marginal | median average | ratio |
//! |---|--:|--:|--:|--:|--:|
//! | `[K]` pays the district ([`Response::Insulated`]) | 144 | 23.53% | **$0.00** | $6,599.32 | **0.0000** |
//! | on the guarantee only ([`Response::TransportationOnly`]) | 167 | 31.69% | $503.40 | $3,939.58 | 0.1512 |
//! | on formula ([`Response::Full`]) | 298 | 44.78% | $8,516.08 | $8,516.08 | 1.0000 |
//!
//! **294 districts lose exactly zero foundation aid and 144 lose exactly zero of anything.** The
//! middle row is an identity rather than an approximation: for all 167 the marginal pupil costs
//! the district *precisely* its transportation aid and nothing else, to the cent
//! (`tests/what_the_formula_pays_a_district_to_do.rs`), because the guarantee holds `[H] + [I]`
//! level while `[J]` is outside it and follows the roll.
//!
//! Statewide, [`statewide`] takes 1% off every district's enrolment at once: 14,019.4 pupils, and
//! the state saves **$45,704,433**, which is **$3,260.09 a pupil against an average of $5,756.93
//! — 56.63%**. Two fifths of Ohio's per-pupil funding does not respond to a pupil.
//!
//! ## The formula row is arithmetic, and saying so is the honest part
//!
//! For a district on formula the marginal pupil is its average pupil to within a billionth, and that
//! is a property of the model before it is a fact about Ohio. R.C. 3317.017(B) multiplies a
//! per-pupil residual by current-year enrolled ADM, so base cost aid genuinely is linear in the
//! count; the categoricals are counts of their own that [`crate::policy::apply`] scales by the
//! ADM ratio, which is the documented approximation and is exact at modelled enrolment. **The
//! finding is the zeros, not the ones.** A ratio of 1.0000 carries no information at all.
//!
//! Five districts publish no transportation components, so their transportation does not follow
//! the roll either — Grandview Heights, Lakewood City, Kelleys Island, Ottawa Hills and College
//! Corner. Two of them are the only districts left insulated when `[K]` is repealed, which is a
//! fixture artefact and not a policy result. It is named here so nobody reports it as one.
//!
//! # 2. The valuation dollar, and the levy the formula cannot see
//!
//! **There is no millage term.** The department's FY2027 model carries 165 columns and not one of
//! them is a rate any district votes on: `capacity_rate` is the sliding scale
//! R.C. 3317.017(A)(4) computes from income, and `tax_returns` is a count of filers. Local
//! capacity is `(0.6 V + 0.2 AGI + 0.2 M x R) x rate` and millage appears nowhere in it. Two
//! districts with the same tax base and twenty mills between their rates receive the same state
//! aid to the cent.
//!
//! So the answer to "is a district paid for not passing a levy" is **no, and it cannot be**: the
//! formula charges a district for its tax *base* and never for its tax *rate*. What it does
//! charge for is assessed value, and there the number is real. A dollar of new taxable valuation
//! costs the median exposed district **1.232 cents of state aid a year** once it has worked
//! through the three-year average, across a range of 0.806 to 1.485 cents
//! ([`Wealth::steady`]) — `0.6 x capacity rate`, amplified by the four categoricals that
//! R.C. 3317.022 multiplies by the state share percentage.
//!
//! It bites on **268 districts of 609**. For the other 341 it is exactly zero: 138 sit on the
//! minimum state share, where the floor censors the whole quantity, and the guaranteed and
//! `[K]`-held districts have their loss put straight back by the device above them.
//!
//! ## H.B. 920 decides whether the value that was charged for bought anything
//!
//! [`millage`] is the other half, and it splits the 268 in two:
//!
//! | | districts | charge per $1 of value | local revenue per $1 | recaptured |
//! |---|--:|--:|--:|--:|
//! | at the twenty-mill floor | 87 | $0.0127 | $0.0200 | **63.26%**, up to 74.24% |
//! | above it, reduction factors operating | 181 | $0.0121 | **$0.00** | — |
//!
//! Above the floor R.C. 319.301 reduces the effective rate of existing levies as reappraisal
//! raises value, so the levy yields the dollars it always yielded and the district gains nothing
//! — while R.C. 3317.017 charges it for the higher value anyway. **For 181 districts a
//! reappraisal of existing property is a pure loss of state aid against no local gain at all.**
//! At the floor reduction factors have stopped and the value does yield revenue, of which the
//! state takes back about five eighths.
//!
//! The split is taken at the published effective rate with no tolerance, as
//! [`millage::FloorStatus`] defines it: at or under twenty mills is at the floor. Allowing a
//! hundredth of a mill moves 17 districts across, 87 to 104, because a district can sit a
//! ten-thousandth above the guarantee the floor gives it. Nothing below turns on which is used,
//! and the number is pinned both ways in the tests.
//!
//! The first year is a third of that, and the reason is statute rather than lag: R.C.
//! 3317.017(A)(1)(a) charges the **lesser** of the most recent year and the three-year average,
//! and 602 of 609 districts are on the lagged branch, so a rise enters the charge one third at a
//! time ([`Wealth::first_year`]).
//!
//! ## Where the agency is, and where this stops
//!
//! A board does not choose its valuation, but it is not a spectator either: Ohio gives school
//! boards a statutory role in consenting to certain property tax exemptions, and an exemption is
//! the one instrument that lowers the base this charge is levied on. **Nothing in this corpus
//! models abatement**, no catalog record reaches district-level exempted value, and the question
//! of whether the charge is large enough to move a consent vote is recorded open rather than
//! answered here. What is established is the price: 1.2 cents on the dollar, and 100% of the
//! revenue forgone for the 172 above the floor.
//!
//! # 3. The boundary is a kink, not a cliff — and it is thinly populated in children
//!
//! Realized aid is `max(formula aid, the FY2020 base)`, which is continuous. A district one
//! dollar below its base and one a dollar above receive within two dollars of each other;
//! **there is no step, and no district is better off for being under.** What changes at the
//! boundary is the *slope*, from one to zero, and that is the whole of the effect.
//!
//! #381 found the population near the edge large — 33 districts under 1.05x, 73 between 1.05 and
//! 1.25x — and that is a measurement of districts already held. Measured from the other side, in
//! the unit a district actually loses, it is small: [`headroom`] converts each formula district's
//! distance to its own floor into pupils, and the median is **206.6 pupils, six of 315 are inside
//! ten, and Waynesfield-Goshen Local is 2.3 pupils from the floor**.
//!
//! In years at each district's own FY2024-FY2026 trend the edge is much closer — of the 238
//! formula districts with a falling roll the median reaches its floor in **10.1 years, 17 inside
//! one year and 68 inside two bienniums**. Franklin City is 11.5 pupils and 0.13 years away. That
//! is the same clock [`crate::enrollment_decline`] measured from inside the cluster, where 50 of
//! 89 crossed in a single year, seen from outside it.
//!
//! # 4. `[K]` changes no sign. It sets three margins of four to zero
//!
//! The fourth question in the brief was whether the backstop reverses anything, and it does not:
//! `[K] = max([L1] - ([H] + [I] + [J]), 0)` moves one for one against a total, so a cut it
//! absorbs is absorbed exactly and never overshot. No district in the panel is paid more for
//! having fewer pupils, less valuation, or less of anything else.
//!
//! What it does instead is stronger. It takes the resident-pupil margin from $503.40 to zero for
//! 144 districts, the valuation margin to zero for all of them, and — the case that matters most —
//! it takes the **open-enrolment clawback** to zero for 12 of the 21 guaranteed districts the
//! clawback reaches. And the one margin it cannot touch is the largest one in the formula:
//! `[L]`, `[M]` and `[O]` are the only lines outside `[L1]`'s subtrahend, so `[M]`'s cliff pays
//! into a district's hand however deeply the floors hold it.
//!
//! # The two margins a board can actually vote on
//!
//! **The open-enrolment clawback charges the whole statewide average base cost.** `[I1]` takes
//! [`crate::panel::OPEN_ENROLLMENT_CLAWBACK_PER_FTE`] — $8,241.61, the statewide average base
//! cost per pupil and *not* the district's state share of it — off the guarantee of a district
//! whose open-enrolment enrolment fell beyond its threshold. 43 districts are charged, 21 are on
//! the guarantee, and for the **9 that `[K]` does not backstop** one open-enrolment FTE is worth
//! $8,241.61 against $319.46 to $1,902.03 for one of the district's own children — **between 4.3
//! and 25.8 times as much**. The state pays these districts nothing at the margin for a resident
//! pupil and charges them full freight for someone else's.
//!
//! **`[M]` is a cliff and the pupil at its edge is worth half a million dollars.**
//! [`crate::panel::Supplements`] already records the cliff in dollars — $250 on the whole roll for
//! a three-year rise of 3%, and New Lexington at 2.9502% forgoing $430,476.80. In pupils it is
//! [`Cliff::per_pupil`]: New Lexington needs **0.833 more pupils**, so the marginal child is worth
//! **$516,984**. Two districts are within one pupil of the threshold and fifteen are within ten.
//!
//! Both are admission decisions under R.C. 3313.98, taken by a board, in a year. They are the only
//! two margins here that are.
//!
//! # What this does not measure
//!
//! [`crate::policy::Outcome::total_state_support`] is realized aid, transportation and `[K]` — the
//! three channels the lever model carries. It is narrower than the payment report's `[R]`, which
//! also holds `[L]`, `[M]`, `[O]`, preschool special education and special education
//! transportation, so the resident-pupil margins above understate by whatever of those follows the
//! roll. `[M]` is handled separately here for exactly that reason, and it is the only one of them
//! with a threshold in it.

use std::collections::BTreeMap;

use edfund_core::{Adm, Dollars};

use crate::capacity_denominator::{frame, Basis, District};
use crate::panel::{
    DistrictRecord, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL, ENROLLMENT_GROWTH_THRESHOLD,
    OPEN_ENROLLMENT_CLAWBACK_PER_FTE,
};
use crate::policy::{apply, Policy, Statewide};

/// A cent. Every margin here is a difference of two dollar figures, so this is the tolerance at
/// which one of them counts as not having moved.
pub const CENT: Dollars = 0.01;

/// The share of its enrolment [`statewide`] takes off every district at once.
///
/// One per cent, which is about a fifth of what the median declining district has already lost
/// since FY2020 and small enough that no district crosses a floor it was not already near.
pub const SHOCK: f64 = 0.01;

/// The weight R.C. 3317.017(A)(1) puts on assessed valuation in the capacity blend.
///
/// Re-exported from [`local_capacity`] because the marginal charge on a dollar of value is this
/// number times the district's capacity rate, and a reader should not have to leave this module
/// to see which number it is.
pub const VALUATION_WEIGHT: f64 = local_capacity::VALUATION_WEIGHT;

/// Years in the average R.C. 3317.017(A)(1)(a) takes the lesser of.
///
/// A rise in valuation enters the charge one third at a time for as long as the three-year average
/// is the lesser of the two, which for FY2027 is 602 of 609 districts.
pub const AVERAGE_YEARS: f64 = 3.0;

/// What happens to a district's cheque when it loses one pupil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response {
    /// `[K]` holds the district at a **total**, so nothing moves at all. A dollar of foundation
    /// aid or transportation the pupil takes away is a dollar the supplement puts back.
    Insulated,
    /// The guarantee holds `[H] + [I]` level and `[J]` is outside it, so the district loses its
    /// transportation aid for the pupil and nothing else.
    TransportationOnly,
    /// Neither floor binds: the pupil leaves with the state's whole per-pupil entitlement.
    Full,
}

impl Response {
    /// The device that sets this regime, for a table that has to name it.
    #[must_use]
    pub const fn device(self) -> &'static str {
        match self {
            Self::Insulated => "[K] formula transition supplement",
            Self::TransportationOnly => "[I] temporary transitional aid guarantee",
            Self::Full => "none",
        }
    }

    /// Which regime a record is in, read off the devices rather than off the measurement.
    ///
    /// Kept separate from [`Pupil::marginal`] on purpose: the classification comes from the two
    /// published columns and the measurement from two runs of the model, so a test can check that
    /// they agree instead of a single function asserting that they do.
    #[must_use]
    pub fn of(record: &DistrictRecord) -> Self {
        if record.transition.transition_supplement > 0.0 {
            Self::Insulated
        } else if record.on_guarantee() {
            Self::TransportationOnly
        } else {
            Self::Full
        }
    }
}

/// What one pupil is worth to one district.
#[derive(Debug, Clone, PartialEq)]
pub struct Pupil {
    /// Information Retrieval Number. 609 districts share about 580 names, so this is the key.
    pub irn: String,
    /// District name as the model publishes it.
    pub name: String,
    /// Current-year enrolled ADM, the count the perturbation is taken around.
    pub adm: Adm,
    /// Total state support at modelled enrolment less the same at one pupil fewer.
    pub marginal: Dollars,
    /// The same on [`crate::biennium::Measure::FoundationAid`] alone, which is where the
    /// guarantee reaches and transportation does not.
    pub foundation: Dollars,
    /// Transportation alone, which is the whole of the difference between the two for a district
    /// the guarantee holds.
    pub transportation: Dollars,
    /// Total state support per pupil at modelled enrolment.
    pub average: Dollars,
    /// The regime, from the published devices.
    pub response: Response,
}

impl Pupil {
    /// The marginal pupil as a share of the average one.
    ///
    /// Zero where a floor holds the district, one where the formula pays it. `0.0` for a district
    /// with no ADM or no support, which is a district this measure says nothing about.
    #[must_use]
    pub fn ratio(&self) -> f64 {
        if self.average.abs() < CENT {
            return 0.0;
        }
        self.marginal / self.average
    }
}

/// What one pupil fewer does to every district, under a stated policy.
///
/// The perturbation holds everything but current-year enrolled ADM: the FY2020 funding base, the
/// FY2021 base `[K]` measures against, base cost enrolled ADM and every published count stay
/// where they are. That is the first-year margin and not the eventual one — base cost ADM is a
/// three-year figure, so a departure takes three years to reach the count base cost is computed
/// on. It is the right horizon for an incentive, which is a question about next year.
#[must_use]
pub fn pupil(panel: &[DistrictRecord], policy: &Policy) -> Vec<Pupil> {
    let statewide = Statewide::under(panel, policy);
    panel
        .iter()
        .filter(|record| record.current_year_adm > 1.0)
        .map(|record| {
            let at = apply(record, policy, &statewide, record.current_year_adm);
            let less = apply(record, policy, &statewide, record.current_year_adm - 1.0);
            Pupil {
                irn: record.irn.clone(),
                name: record.name.clone(),
                adm: record.current_year_adm,
                marginal: at.total_state_support() - less.total_state_support(),
                foundation: at.realized_aid - less.realized_aid,
                transportation: at.transportation - less.transportation,
                average: at.total_state_support() / record.current_year_adm,
                response: Response::of(record),
            }
        })
        .collect()
}

/// What the state saves when every district loses the same share of its roll.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aggregate {
    /// Pupils removed statewide.
    pub pupils: Adm,
    /// Total state support given up.
    pub saved: Dollars,
    /// Total state support per pupil before the shock.
    pub average: Dollars,
}

impl Aggregate {
    /// What one of those pupils was worth.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.pupils <= 0.0 {
            return 0.0;
        }
        self.saved / self.pupils
    }

    /// The marginal pupil as a share of the average one, statewide.
    ///
    /// The one number that answers "how much of Ohio's per-pupil funding is actually per pupil".
    #[must_use]
    pub fn share(&self) -> f64 {
        if self.average.abs() < CENT {
            return 0.0;
        }
        self.per_pupil() / self.average
    }
}

/// Take `fraction` off every district's enrolment at once and price it.
///
/// Statewide rather than per district because the floors are what the figure is about: summing
/// 609 per-pupil amounts would answer a different question, in which every district is on formula.
#[must_use]
pub fn statewide(panel: &[DistrictRecord], policy: &Policy, fraction: f64) -> Aggregate {
    let resolved = Statewide::under(panel, policy);
    let mut before = 0.0;
    let mut after = 0.0;
    let mut adm = 0.0;
    for record in panel {
        before += apply(record, policy, &resolved, record.current_year_adm).total_state_support();
        after += apply(
            record,
            policy,
            &resolved,
            record.current_year_adm * (1.0 - fraction),
        )
        .total_state_support();
        adm += record.current_year_adm;
    }
    Aggregate {
        pupils: adm * fraction,
        saved: before - after,
        average: if adm > 0.0 { before / adm } else { 0.0 },
    }
}

/// How far a formula district is from the floor that would stop it losing anything more.
#[derive(Debug, Clone, PartialEq)]
pub struct Headroom {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name.
    pub name: String,
    /// Formula aid over the FY2020 base, in dollars.
    pub gap: Dollars,
    /// The same, in pupils at this district's own marginal rate.
    pub pupils: Adm,
    /// Its annualised enrolled-ADM rate over FY2024-FY2026, negative for a falling roll.
    pub rate: f64,
    /// Years to the floor at that rate. `None` for a district that is not falling, which the
    /// floor never reaches.
    pub years: Option<f64>,
}

/// Every formula district's distance to its own guarantee floor, in dollars, pupils and years.
///
/// The rate is the same two-leg annualisation [`crate::report::enrollment_growth_prior`] uses —
/// `(last / first)^0.5 - 1` over the three-year ADM history — so the years here and the crossing
/// counts in [`crate::enrollment_decline`] are on one clock.
#[must_use]
pub fn headroom(panel: &[DistrictRecord], policy: &Policy) -> Vec<Headroom> {
    let statewide = Statewide::under(panel, policy);
    panel
        .iter()
        .filter_map(|record| {
            let at = apply(record, policy, &statewide, record.current_year_adm);
            if at.on_guarantee || record.current_year_adm <= 1.0 {
                return None;
            }
            let less = apply(record, policy, &statewide, record.current_year_adm - 1.0);
            let per_pupil = at.formula_aid - less.formula_aid;
            if per_pupil <= 0.0 {
                return None;
            }
            let gap = at.formula_aid - record.guarantee_floor();
            let pupils = gap / per_pupil;
            let [first, _, last] = record.adm_history;
            let rate = if first > 0.0 && last > 0.0 {
                (last / first).sqrt() - 1.0
            } else {
                0.0
            };
            let falling = rate * record.current_year_adm;
            Some(Headroom {
                irn: record.irn.clone(),
                name: record.name.clone(),
                gap,
                pupils,
                rate,
                years: (falling < 0.0).then(|| pupils / -falling),
            })
        })
        .collect()
}

/// A district short of `[M]`'s threshold, and what clearing it would pay.
#[derive(Debug, Clone, PartialEq)]
pub struct Cliff {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name.
    pub name: String,
    /// `M1` — the district's published three-year enrolment change.
    pub change: f64,
    /// Pupils it is short of [`ENROLLMENT_GROWTH_THRESHOLD`], on the supplement's own base.
    pub short: Adm,
    /// What clearing the threshold would pay: $250 on the whole roll, not on the pupils gained.
    pub forgone: Dollars,
}

impl Cliff {
    /// What the pupil that clears the threshold is worth.
    ///
    /// The whole supplement, divided by the fraction of a pupil still needed. This is the figure
    /// the cliff is made of: `[M]` pays on the roll and tests on the increment, so the last pupil
    /// before the test passes carries every dollar the other ones do not.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.short <= 0.0 {
            return 0.0;
        }
        self.forgone / self.short
    }
}

/// Every district that missed `[M]`, with the distance to it in pupils.
///
/// The shortfall is `base x (threshold - change)` from the supplement's **own** two columns rather
/// than from current-year enrolled ADM. `M1` is computed by the department from two columns that
/// both say FY2023 and reconstructing it against the funded count disagrees by up to 0.26
/// percentage points — see `tests/the_two_columns_that_both_say_fy2023.rs`. A cliff measured in
/// fractions of a pupil cannot absorb that.
#[must_use]
pub fn growth_cliff(panel: &[DistrictRecord]) -> Vec<Cliff> {
    panel
        .iter()
        .filter_map(|record| {
            let supplements = &record.supplements;
            if supplements.growth_eligible || supplements.adm_fy23 <= 0.0 {
                return None;
            }
            let short = supplements.adm_fy23
                * (ENROLLMENT_GROWTH_THRESHOLD - supplements.enrollment_change);
            (short > 0.0).then(|| Cliff {
                irn: record.irn.clone(),
                name: record.name.clone(),
                change: supplements.enrollment_change,
                short,
                forgone: record.categorical_enrolled_adm * ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL,
            })
        })
        .collect()
}

/// What one open-enrolment FTE is worth to a district the clawback reaches.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenEnrolment {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name.
    pub name: String,
    /// Whether the guarantee is what pays this district, which is the only place `[I1]` lands.
    pub on_guarantee: bool,
    /// Total state support given up for one more open-enrolment FTE lost.
    pub marginal: Dollars,
    /// The same for one of the district's own children, for the comparison.
    pub resident: Dollars,
}

impl OpenEnrolment {
    /// Another district's child over this district's own, at the margin.
    ///
    /// `None` where the resident margin is zero, which is a ratio nobody should print.
    #[must_use]
    pub fn over_resident(&self) -> Option<f64> {
        (self.resident.abs() >= CENT).then(|| self.marginal / self.resident)
    }
}

/// The clawback margin, for every district `[I1]` is charged against.
///
/// Perturbed by adding one FTE's charge to the published `[I1]` column rather than by
/// recomputing the count: the published adjustment is
/// `clawed_back_fte x OPEN_ENROLLMENT_CLAWBACK_PER_FTE` to within half a cent for all 43 charged
/// districts, so the column is the rate times the count and adding one rate adds one FTE.
///
/// Only districts already past their threshold are here. Below it the threshold absorbs the FTE
/// and the margin is zero, which is a different statement and not this one.
#[must_use]
pub fn open_enrolment(panel: &[DistrictRecord], policy: &Policy) -> Vec<OpenEnrolment> {
    let statewide = Statewide::under(panel, policy);
    panel
        .iter()
        .filter(|record| record.transition.clawed_back_fte() > 0.0 && record.current_year_adm > 1.0)
        .map(|record| {
            let mut charged = record.clone();
            charged.transition.open_enrollment_adjustment += OPEN_ENROLLMENT_CLAWBACK_PER_FTE;
            let at = apply(record, policy, &statewide, record.current_year_adm);
            let with_fte = apply(&charged, policy, &statewide, record.current_year_adm);
            let fewer = apply(record, policy, &statewide, record.current_year_adm - 1.0);
            OpenEnrolment {
                irn: record.irn.clone(),
                name: record.name.clone(),
                on_guarantee: at.on_guarantee,
                marginal: at.total_state_support() - with_fte.total_state_support(),
                resident: at.total_state_support() - fewer.total_state_support(),
            }
        })
        .collect()
}

/// Which side of the twenty-mill floor a district is on, and therefore whether the value it is
/// charged for yields it anything.
///
/// Read off the Department of Taxation's published **effective Class I rate** for
/// [`crate::capacity_denominator::RESIDENT_TAX_YEAR`], which is the rate after reduction factors.
/// Class I is residential and agricultural property, where reappraisal growth mostly lands; the
/// Class II rate is a separate number and a separate question.
#[must_use]
pub fn floor_status(class_one_rate: f64) -> millage::FloorStatus {
    if class_one_rate <= millage::SCHOOL_DISTRICT_FLOOR {
        millage::FloorStatus::AtFloor
    } else {
        millage::FloorStatus::AboveFloor
    }
}

/// What a dollar of assessed valuation costs one district in state aid.
#[derive(Debug, Clone, PartialEq)]
pub struct Wealth {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name.
    pub name: String,
    /// `[C6]`, the sliding capacity rate R.C. 3317.017(A)(4) computes from income.
    pub rate: f64,
    /// State aid lost per dollar of local charge — a shade under one, plus what the four
    /// state-share-denominated categoricals add and minus what the two pupil counts differ by.
    pub slope: f64,
    /// State aid lost per dollar of taxable value, once the rise has worked through the
    /// three-year average.
    pub steady: Dollars,
    /// The same in the year the value first appears, which for a district on the lagged branch of
    /// R.C. 3317.017(A)(1)(a) is a third of [`Self::steady`].
    pub first_year: Dollars,
    /// The district's effective Class I rate in mills, after reduction factors.
    pub mills: f64,
    /// Which side of the twenty-mill floor that puts it on.
    pub status: millage::FloorStatus,
}

impl Wealth {
    /// Local revenue per dollar of new Class I value, at this district's own effective rate.
    ///
    /// `None` above the floor, where R.C. 319.301 reduces the rate as the value rises and existing
    /// property yields no new dollars at all. That is not zero dressed as an absence: the district
    /// is charged for value that produced nothing, and a ratio with zero underneath it would be
    /// the wrong shape for that fact.
    #[must_use]
    pub fn local_yield(&self) -> Option<Dollars> {
        self.status
            .valuation_growth_reaches_revenue()
            .then_some(self.mills / 1_000.0)
    }

    /// The share of the local revenue a dollar of new value raises that the state takes back.
    ///
    /// `None` above the floor, for the reason [`Self::local_yield`] gives.
    #[must_use]
    pub fn recapture(&self) -> Option<f64> {
        self.local_yield()
            .filter(|y| *y > 0.0)
            .map(|y| self.steady / y)
    }
}

/// The valuation charge, for the districts a floor does not already zero it for.
///
/// Excluded, and each for its own reason: a district on the guarantee or drawing `[K]` has the
/// loss put back by the device above it, and a district on the minimum state share has the
/// quantity censored — R.C. 3317.017(B)(1) pays the floor regardless of how far capacity exceeds
/// it, so more value changes nothing and less value would have to close a gap this model cannot
/// see the size of.
///
/// The slope is taken by perturbing the local charge by a dollar and re-running
/// [`District::aid_on`], which carries the categorical amplification R.C. 3317.022(A)(3), (5) and
/// (6) write into the state share percentage.
#[must_use]
pub fn wealth(panel: &[DistrictRecord]) -> Vec<Wealth> {
    let charges: BTreeMap<String, District> = frame()
        .into_iter()
        .map(|district| (district.irn.clone(), district))
        .collect();
    let rates: BTreeMap<String, f64> = dispersion::sd1::rows()
        .into_iter()
        .filter(|row| row.tax_year == crate::capacity_denominator::RESIDENT_TAX_YEAR)
        .filter_map(|row| row.class1_rate.map(|rate| (row.irn, rate)))
        .collect();

    panel
        .iter()
        .filter(|record| {
            !record.on_guarantee()
                && record.transition.transition_supplement <= 0.0
                && !record.at_minimum_state_share()
        })
        .filter_map(|record| {
            let district = charges.get(&record.irn)?;
            let rate = record.published_capacity_rate?;
            let mills = rates.get(&record.irn).copied()?;

            let mut bumped = district.clone();
            bumped.local_charge += 1.0;
            let slope =
                district.aid_on(Basis::BaseCostEnrolled) - bumped.aid_on(Basis::BaseCostEnrolled);
            let steady = VALUATION_WEIGHT * rate * slope;

            Some(Wealth {
                irn: record.irn.clone(),
                name: record.name.clone(),
                rate,
                slope,
                steady,
                first_year: steady / AVERAGE_YEARS,
                mills,
                status: floor_status(mills),
            })
        })
        .collect()
}
