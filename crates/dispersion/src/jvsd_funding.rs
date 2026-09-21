//! What Ohio pays its forty-nine joint vocational school districts, across the act that changed it.
//!
//! The third population the state funds by formula, after school districts and community schools,
//! and the only one whose funding this corpus can watch across an amendment to its own method.
//! [`super::lea_directory::joint_vocational_districts`] holds the membership — who the
//! forty-nine are, and the two independent routes that had to agree before the corpus believed
//! it. This module is the money: six years of the department's own payment reports, FY2022
//! through FY2027.
//!
//! # Why it is here and not in a calculator crate
//!
//! Same reason as [`super::community_school_funding`]: nothing in them can join it. A JVSD has
//! no IRN in the 609-district panel `project::policy` prices its levers over. It *does* have
//! taxing authority, unlike a community school, and so it does have a valuation and a local
//! share — which makes the shape of this population's formula far closer to a district's, and
//! makes the absence of a join a fact about the panel rather than about the formula.
//!
//! # What the series is for: item 7 of H.B. 96
//!
//! Item 7 rewrote the JVSD state share of base cost into per-pupil form: a half-mill charge
//! against the lesser of the annual and three-year-average valuation, divided by base cost
//! enrolled ADM, taken against per-pupil base cost, floored at a 10% minimum state share.
//! [`Method`] is which law a year is under, and it is read off the department's own schema —
//! the three per-pupil columns do not exist before FY2026.
//!
//! # The rewrite does not change the percentage
//!
//! Base cost per-pupil is aggregate base cost over the same base cost enrolled ADM that divides
//! the charge, so the ADM cancels and item 7's share is prior law's share exactly. That identity
//! is [`District::implied_state_share`], and it reproduces the published percentage on **all 49
//! districts in all six years, on both sides of the amendment**.
//!
//! **What does not cancel is the ADM the percentage multiplies.** Prior law paid *aggregate base
//! cost less the charge-off* — a subtraction, over base cost enrolled ADM, a three-year average.
//! Item 7 pays *per-pupil base cost times enrolled ADM times the share* — a multiplication, over
//! the current year. The amendment moves JVSD base cost aid from a smoothed enrolment count onto
//! a live one, and it moves money only for districts whose enrolment is not flat.
//! [`item_7_incidence`] prices that, and it can be priced honestly in exactly one year: see
//! below.
//!
//! # FY2027 is provisional in a way no earlier year is
//!
//! The FY2027 report is a September payment, the first of an open year. Its enrolled ADM is
//! FY2026's on 43 of the 49 districts, because FY2027 enrolment does not exist in September 2026.
//! Under prior law that would have been harmless — the share multiplied an average already known.
//! Under item 7 the share multiplies the current year, so **the provisional ADM now reaches the
//! money**. [`District::enrolment_is_provisional`] carries it, and [`item_7_incidence`] is
//! offered for FY2026 alone: it is the only year that is both under item 7 and closed.
//!
//! # Nobody is on the floor
//!
//! The 10% minimum state share is maintained by item 7 from prior law, and **no joint vocational
//! district has been on it in any of these six years**. A floor's population and a floor's reach
//! are different questions, and on this population the floor's population is nobody.
//!
//! The same district holds the minimum in all six years — Cuyahoga Valley Career Center — and its
//! share does not fall steadily toward the floor so much as bounce along above it: 0.20730,
//! 0.13695, 0.23689, 0.22018, 0.13739, 0.10923. **Neither the bounce nor the direction is the
//! amendment**, and they are not the same effect. The direction is a squeeze that predates item 7
//! by four years: this district's charged valuation rises 35.7% across the span while its
//! aggregate base cost rises 20.8%, and the share is one less their ratio. The bounce on top is
//! aggregate base cost moving year to year, which on a district this small a single staffing
//! threshold can do. The lesser-of rule explains none of it — the three-year average binds in all
//! six years here, so [`District::property_valuation_used`] never switches.
//!
//! [`lowest_state_share_by_year`] is the series. Reading its last value as the end of a slide
//! into the floor would be reading a small district's base cost build-up as policy.

use std::collections::BTreeMap;

const FIXTURE: &str = include_str!("../fixtures/jvsd-foundation-payments.csv");

const EXPECTED_HEADER: &str = "fiscal_year,irn,district,county,enrolled_adm,\
    base_cost_enrolled_adm,enrolment_is_provisional,base_cost_per_pupil,aggregate_base_cost,\
    property_valuation_annual,property_valuation_three_year_average,property_valuation_used,\
    state_share_percentage,state_share_of_base_cost,core_foundation_funding,guarantee,\
    formula_transition_supplement,base_funding_supplement,total_state_support";

/// How many columns a row of the extract has.
const WIDTH: usize = 19;

/// The half-mill a joint vocational district's valuation is charged.
///
/// R.C. 3317.16, and item 7 did not change it: prior law charged it against the district and
/// item 7 charges it per pupil, which is the same percentage either way.
pub const LOCAL_CAPACITY_MILLAGE: f64 = 0.0005;

/// The minimum state share of base cost, which item 7 maintained from prior law.
///
/// Nobody is on it. See the module note.
pub const MINIMUM_STATE_SHARE: f64 = 0.10;

/// How many joint vocational school districts Ohio has.
///
/// Fixed across the span, and agreed by two sources that do not share a route: the department's
/// payment reports, and [`super::lea_directory::joint_vocational_districts`] reading the federal
/// directory.
pub const DISTRICT_COUNT: usize = 49;

/// The first fiscal year the extract holds.
pub const FIRST_YEAR: u16 = 2022;
/// The last, which is the open year — see [`District::enrolment_is_provisional`].
pub const LAST_YEAR: u16 = 2027;

/// The first fiscal year computed under item 7 of H.B. 96.
pub const FIRST_ITEM_7_YEAR: u16 = 2026;

/// Which law a year's state share of base cost was computed under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Method {
    /// Aggregate base cost less a flat half-mill charge-off, floored at 10%.
    ///
    /// A **subtraction**. The percentage the report prints is a quotient derived afterwards, not
    /// an operand — which is why enrolment appears nowhere in it.
    PriorLaw,
    /// Per-pupil base cost less per-pupil local capacity, over per-pupil base cost, applied to
    /// current-year enrolled ADM.
    ///
    /// A **multiplication**, and the ADM it multiplies is the substance of the change.
    Item7,
}

impl Method {
    /// The law in force for a fiscal year.
    #[must_use]
    pub const fn in_force(fiscal_year: u16) -> Self {
        if fiscal_year >= FIRST_ITEM_7_YEAR {
            Self::Item7
        } else {
            Self::PriorLaw
        }
    }

    /// A short name for a table or a legend.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PriorLaw => "Prior law",
            Self::Item7 => "H.B. 96 item 7",
        }
    }
}

/// One joint vocational school district in one fiscal year.
///
/// Every money field is dollars for the year. The two ADMs are the point of the row: they are
/// what item 7 moved the state share between.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// The fiscal year.
    pub fiscal_year: u16,
    /// The district's IRN. The only unique key: two JVSDs share neither, but names collide with
    /// the traditional districts around them.
    pub irn: String,
    /// Its name as the department writes it.
    pub name: String,
    /// The county the department attributes it to. A JVSD usually spans several; this is one.
    pub county: String,
    /// `[a]`: enrolled ADM for the current year. What item 7 made the state share multiply.
    pub enrolled_adm: f64,
    /// `[b]`: base cost enrolled ADM, a three-year average. What the base cost build-up divides
    /// by, and what prior law's state share effectively applied to.
    pub base_cost_enrolled_adm: f64,
    /// Whether [`Self::enrolled_adm`] is a count or last year's number carried forward.
    ///
    /// True for FY2027 and no other year. Under item 7 this reaches the money; under prior law it
    /// would not have.
    pub enrolment_is_provisional: bool,
    /// `[F]`/`[A1]`: aggregate base cost per pupil of base cost enrolled ADM.
    ///
    /// Absent in FY2022, whose base cost sheet does not publish it — prior law never divided by a
    /// pupil. Where it is present from FY2026 it is the department's **rounded** figure, which is
    /// what it pays on.
    pub base_cost_per_pupil: Option<f64>,
    /// `[E]`: aggregate base cost, the five sub-components summed.
    pub aggregate_base_cost: f64,
    /// `[f]`: the most recent annual property valuation.
    pub property_valuation_annual: f64,
    /// `[g]`: the three-year average property valuation.
    pub property_valuation_three_year_average: f64,
    /// `[h]`: the lesser of the two, which is what the half-mill is charged against.
    pub property_valuation_used: f64,
    /// `[A3]`: the state share of base cost, as a fraction.
    pub state_share_percentage: f64,
    /// `[A]`: the state share of base cost, in dollars.
    pub state_share_of_base_cost: f64,
    /// `[F]`: core foundation funding — base cost, special education, DPIA, English learners and
    /// career-technical, after the phase-in.
    pub core_foundation_funding: f64,
    /// `[G]`: the temporary transitional aid guarantee. One district receives it in FY2026.
    pub guarantee: f64,
    /// `[H]`: the formula transition supplement, against an FY2021 base.
    pub formula_transition_supplement: f64,
    /// `[I]`: the base funding supplement, which exists only from FY2026 and reaches all 49.
    ///
    /// The tag is the trap this fixture's extractor is mostly about: `[I]` is *total state
    /// support* in FY2024 and FY2025 and this line in FY2026.
    pub base_funding_supplement: f64,
    /// `[J]`: total state support, as the department totals it.
    pub total_state_support: f64,
}

impl District {
    /// Which law this row's state share was computed under.
    #[must_use]
    pub const fn method(&self) -> Method {
        Method::in_force(self.fiscal_year)
    }

    /// The half-mill charged against this district's valuation, in dollars.
    #[must_use]
    pub fn local_capacity(&self) -> f64 {
        self.property_valuation_used * LOCAL_CAPACITY_MILLAGE
    }

    /// The same charge per pupil, as item 7 expresses it.
    #[must_use]
    pub fn local_capacity_per_pupil(&self) -> f64 {
        self.local_capacity() / self.base_cost_enrolled_adm
    }

    /// The state share percentage this row's own inputs imply.
    ///
    /// One expression for both laws, because the per-pupil rewrite cancels the ADM out of
    /// numerator and denominator alike. It reproduces
    /// [`Self::state_share_percentage`] on every row of the fixture, which is what makes
    /// "item 7 did not change the percentage" a measured statement rather than an algebraic one.
    #[must_use]
    pub fn implied_state_share(&self) -> f64 {
        (1.0 - self.local_capacity() / self.aggregate_base_cost).max(MINIMUM_STATE_SHARE)
    }

    /// What prior law's arithmetic pays this district for base cost.
    ///
    /// Aggregate base cost less the charge-off — a subtraction, with no ADM in it. This
    /// reproduces the published payment on every FY2022-FY2025 row.
    #[must_use]
    pub fn base_cost_aid_under_prior_law(&self) -> f64 {
        (self.aggregate_base_cost - self.local_capacity())
            .max(self.aggregate_base_cost * MINIMUM_STATE_SHARE)
    }

    /// What item 7's arithmetic pays: per-pupil base cost times *current-year* enrolled ADM times
    /// the share.
    ///
    /// `None` for FY2022, whose report does not publish a per-pupil base cost at all. This
    /// reproduces the published payment on every FY2026 and FY2027 row, to the cent.
    #[must_use]
    pub fn base_cost_aid_under_item_7(&self) -> Option<f64> {
        Some(self.base_cost_per_pupil? * self.enrolled_adm * self.state_share_percentage)
    }

    /// Item 7's arithmetic over prior law's ADM: the counterfactual the amendment is priced
    /// against.
    ///
    /// The same expression as [`Self::base_cost_aid_under_item_7`] with the smoothed three-year
    /// count in place of the current one, because **swapping that ADM is the entire amendment**.
    /// Pricing item 7 as [`Self::base_cost_aid_under_item_7`] less
    /// [`Self::base_cost_aid_under_prior_law`] would be defensible and is noisier: those two are
    /// rounded differently — one off a per-pupil figure the department rounds to the cent, the
    /// other off a full-precision aggregate — so their difference carries a residue of up to
    /// several dollars on a district that the amendment does not move at all. Holding the share
    /// and the per-pupil cost fixed and varying only the ADM isolates the mechanism exactly.
    ///
    /// The two agree to that rounding, which
    /// `the_two_statements_of_prior_law_agree_to_their_rounding` checks.
    #[must_use]
    pub fn base_cost_aid_on_smoothed_enrolment(&self) -> Option<f64> {
        Some(self.base_cost_per_pupil? * self.base_cost_enrolled_adm * self.state_share_percentage)
    }

    /// How much the amendment moves for this district.
    ///
    /// The share applied to the difference between the two ADMs, which is what item 7 changed and
    /// all it changed. Positive where current enrolment runs above the three-year average,
    /// negative below, and **exactly zero** where they are equal. `None` for FY2022.
    #[must_use]
    pub fn item_7_delta(&self) -> Option<f64> {
        Some(
            self.base_cost_per_pupil?
                * self.state_share_percentage
                * (self.enrolled_adm - self.base_cost_enrolled_adm),
        )
    }

    /// Current enrolment over the smoothed count the base cost build-up uses.
    ///
    /// The single ratio the amendment turns on: above one and a district gains, below one it
    /// loses, and at one item 7 is a restatement that moves nothing.
    #[must_use]
    pub fn enrolment_ratio(&self) -> f64 {
        self.enrolled_adm / self.base_cost_enrolled_adm
    }
}

/// Every district-year in the extract.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or a row is not the full
/// width — either of which means the extract is not the file this reader expects.
#[must_use]
pub fn districts() -> Vec<District> {
    let mut lines = FIXTURE.lines();
    let header = lines.next().unwrap_or_default().trim();
    assert_eq!(
        header, EXPECTED_HEADER,
        "the joint vocational district funding fixture's columns have moved"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let cells: Vec<&str> = line.split(',').collect();
            assert_eq!(
                cells.len(),
                WIDTH,
                "the JVSD funding fixture is uniform-width; this row is not: {line}"
            );
            let number = |index: usize| cells[index].trim().parse::<f64>().unwrap_or(0.0);
            let optional = |index: usize| cells[index].trim().parse::<f64>().ok();
            District {
                fiscal_year: cells[0].trim().parse().unwrap_or_else(|_| {
                    panic!("the JVSD funding fixture has a row with no fiscal year: {line}")
                }),
                irn: cells[1].trim().to_string(),
                name: cells[2].trim().to_string(),
                county: cells[3].trim().to_string(),
                enrolled_adm: number(4),
                base_cost_enrolled_adm: number(5),
                enrolment_is_provisional: cells[6].trim() == "yes",
                base_cost_per_pupil: optional(7),
                aggregate_base_cost: number(8),
                property_valuation_annual: number(9),
                property_valuation_three_year_average: number(10),
                property_valuation_used: number(11),
                state_share_percentage: number(12),
                state_share_of_base_cost: number(13),
                core_foundation_funding: number(14),
                guarantee: number(15),
                formula_transition_supplement: number(16),
                base_funding_supplement: number(17),
                total_state_support: number(18),
            }
        })
        .collect()
}

/// One fiscal year's forty-nine districts.
#[must_use]
pub fn year(fiscal_year: u16) -> Vec<District> {
    districts()
        .into_iter()
        .filter(|district| district.fiscal_year == fiscal_year)
        .collect()
}

/// One fiscal year's districts, by IRN.
#[must_use]
pub fn year_by_irn(fiscal_year: u16) -> BTreeMap<String, District> {
    year(fiscal_year)
        .into_iter()
        .map(|district| (district.irn.clone(), district))
        .collect()
}

/// Total state support to the population, by fiscal year.
#[must_use]
pub fn total_state_support_by_year() -> BTreeMap<u16, f64> {
    let mut out: BTreeMap<u16, f64> = BTreeMap::new();
    for district in districts() {
        *out.entry(district.fiscal_year).or_default() += district.total_state_support;
    }
    out
}

/// What item 7 moved, in one fiscal year.
///
/// The counterfactual is the same year's own data under the other law, so nothing here is a
/// projection: every term is published, and the only thing being varied is which arithmetic the
/// statute prescribes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Item7Incidence {
    /// The fiscal year priced.
    pub fiscal_year: u16,
    /// Base cost aid to the population as item 7 computes it.
    pub under_item_7: f64,
    /// Base cost aid to the same districts, same year, over the smoothed enrolment prior law
    /// applied the share to. See [`District::base_cost_aid_on_smoothed_enrolment`].
    pub under_prior_law: f64,
    /// How many districts gain, which is how many enrol above their three-year average.
    pub better_off: usize,
    /// How many lose.
    pub worse_off: usize,
}

impl Item7Incidence {
    /// The amendment's cost to the state, in dollars.
    #[must_use]
    pub fn delta(&self) -> f64 {
        self.under_item_7 - self.under_prior_law
    }

    /// The same as a share of what prior law would have paid.
    #[must_use]
    pub fn delta_share(&self) -> f64 {
        self.delta() / self.under_prior_law
    }
}

/// Price item 7 against prior law, in the one year where that is honest.
///
/// **FY2026 only, and that is a statement about the sources rather than a limitation of the
/// arithmetic.** The amendment applies to FY2026 and FY2027. FY2027's report is a September
/// payment whose enrolled ADM is FY2026's on 43 of 49 districts, and enrolled ADM is precisely
/// the term item 7 introduced — so an FY2027 incidence would mostly be measuring a placeholder.
/// Returns `None` for every other year.
///
/// The earlier years are not offered either, for the opposite reason: they are under prior law,
/// so "what item 7 would have paid" there needs a per-pupil base cost their reports do not
/// publish, and FY2022's does not publish at all.
#[must_use]
pub fn item_7_incidence(fiscal_year: u16) -> Option<Item7Incidence> {
    if fiscal_year != FIRST_ITEM_7_YEAR {
        return None;
    }
    let districts = year(fiscal_year);
    if districts.len() != DISTRICT_COUNT {
        return None;
    }
    let mut incidence = Item7Incidence {
        fiscal_year,
        under_item_7: 0.0,
        under_prior_law: 0.0,
        better_off: 0,
        worse_off: 0,
    };
    for district in &districts {
        let item_7 = district.base_cost_aid_under_item_7()?;
        let prior = district.base_cost_aid_on_smoothed_enrolment()?;
        incidence.under_item_7 += item_7;
        incidence.under_prior_law += prior;
        if item_7 > prior {
            incidence.better_off += 1;
        } else {
            incidence.worse_off += 1;
        }
    }
    Some(incidence)
}

/// The lowest state share of base cost in each fiscal year, and whose it is.
///
/// The series the 10% floor is approached along: 0.22018 in FY2025 under prior law, 0.10923 in
/// FY2027 under item 7.
#[must_use]
pub fn lowest_state_share_by_year() -> BTreeMap<u16, (String, f64)> {
    let mut out: BTreeMap<u16, (String, f64)> = BTreeMap::new();
    for district in districts() {
        let entry = out
            .entry(district.fiscal_year)
            .or_insert_with(|| (district.name.clone(), district.state_share_percentage));
        if district.state_share_percentage < entry.1 {
            *entry = (district.name.clone(), district.state_share_percentage);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture is six complete years of the same forty-nine districts.
    #[test]
    fn the_extract_is_six_years_of_forty_nine_districts() {
        let mut by_year: BTreeMap<u16, usize> = BTreeMap::new();
        for district in districts() {
            *by_year.entry(district.fiscal_year).or_default() += 1;
        }
        let years: Vec<u16> = by_year.keys().copied().collect();
        assert_eq!(years, (FIRST_YEAR..=LAST_YEAR).collect::<Vec<_>>());
        for (year, count) in by_year {
            assert_eq!(
                count, DISTRICT_COUNT,
                "FY{year} does not carry 49 districts"
            );
        }
    }

    /// The department's forty-nine are the forty-nine this repository found in the federal
    /// directory, which it found by two routes that never touch this file.
    #[test]
    fn the_department_agrees_with_the_directory_about_who_they_are() {
        let ours: std::collections::BTreeSet<String> =
            crate::lea_directory::joint_vocational_districts()
                .into_keys()
                .collect();
        let theirs: std::collections::BTreeSet<String> =
            year(LAST_YEAR).into_iter().map(|d| d.irn).collect();

        assert_eq!(
            ours, theirs,
            "the department's payment report and the CCD directory disagree about which agencies \
             are joint vocational districts"
        );
    }

    /// The charged valuation is the lesser of the two published, every row.
    #[test]
    fn the_plan_charges_the_lesser_valuation() {
        for district in districts() {
            let lesser = district
                .property_valuation_annual
                .min(district.property_valuation_three_year_average);
            assert!(
                (district.property_valuation_used - lesser).abs() < 0.01,
                "FY{} {}: charged on {} rather than the lesser of {} and {}",
                district.fiscal_year,
                district.name,
                district.property_valuation_used,
                district.property_valuation_annual,
                district.property_valuation_three_year_average,
            );
        }
    }

    /// The identity the module is about: one expression reproduces the published percentage on
    /// both sides of the amendment.
    #[test]
    fn the_per_pupil_rewrite_computes_the_same_percentage() {
        for district in districts() {
            assert!(
                (district.implied_state_share() - district.state_share_percentage).abs() < 1e-6,
                "FY{} {} ({:?}): publishes {} and its valuation over its aggregate base cost \
                 gives {}",
                district.fiscal_year,
                district.name,
                district.method(),
                district.state_share_percentage,
                district.implied_state_share(),
            );
        }
    }

    /// And the per-pupil form of the same charge gives the same number, term by term.
    #[test]
    fn the_per_pupil_terms_are_the_whole_district_terms_divided_through() {
        for district in districts() {
            let Some(per_pupil_base_cost) = district.base_cost_per_pupil else {
                continue;
            };
            let restated =
                (per_pupil_base_cost - district.local_capacity_per_pupil()) / per_pupil_base_cost;
            assert!(
                (restated.max(MINIMUM_STATE_SHARE) - district.state_share_percentage).abs() < 1e-5,
                "FY{} {}: the per-pupil restatement gives {restated} against a published {}",
                district.fiscal_year,
                district.name,
                district.state_share_percentage,
            );
        }
    }

    /// Prior law's dollars are a subtraction, and they reproduce as one.
    #[test]
    fn prior_law_pays_aggregate_base_cost_less_the_charge_off() {
        for district in districts()
            .iter()
            .filter(|d| d.method() == Method::PriorLaw)
        {
            assert!(
                (district.base_cost_aid_under_prior_law() - district.state_share_of_base_cost)
                    .abs()
                    < 0.01,
                "FY{} {}: paid {} and aggregate base cost less the charge-off is {}",
                district.fiscal_year,
                district.name,
                district.state_share_of_base_cost,
                district.base_cost_aid_under_prior_law(),
            );
        }
    }

    /// Item 7's dollars are a multiplication over current-year ADM, and they reproduce as one.
    #[test]
    fn item_7_pays_per_pupil_base_cost_times_current_enrolment() {
        for district in districts().iter().filter(|d| d.method() == Method::Item7) {
            let paid = district
                .base_cost_aid_under_item_7()
                .expect("an item 7 year publishes a per-pupil base cost");
            assert!(
                (paid - district.state_share_of_base_cost).abs() < 0.01,
                "FY{} {}: paid {} and per-pupil base cost times enrolled ADM times the share is \
                 {paid}",
                district.fiscal_year,
                district.name,
                district.state_share_of_base_cost,
            );
        }
    }

    /// Enrolment is a fresh count in every closed year and last year's in FY2027.
    #[test]
    fn only_the_open_year_carries_provisional_enrolment() {
        for district in districts() {
            assert_eq!(
                district.enrolment_is_provisional,
                district.fiscal_year == LAST_YEAR,
                "FY{} {} is marked {} for provisional enrolment",
                district.fiscal_year,
                district.name,
                district.enrolment_is_provisional,
            );
        }
    }

    /// Total state support is its four published parts, every row.
    ///
    /// Cheap and worth having because it spans the tag collision the extractor is mostly about:
    /// `[I]` is *total state support* in FY2024-FY2025 and the *base funding supplement* in
    /// FY2026-FY2027. A reader that took the letter instead of the label would put a $559m column
    /// where a $1.4m one belongs, and this sum is what would not balance.
    #[test]
    fn total_state_support_is_the_sum_of_its_parts() {
        for district in districts() {
            let parts = district.core_foundation_funding
                + district.guarantee
                + district.formula_transition_supplement
                + district.base_funding_supplement;
            assert!(
                (district.total_state_support - parts).abs() < 0.011,
                "FY{} {}: totals {} against parts summing to {parts}",
                district.fiscal_year,
                district.name,
                district.total_state_support,
            );
        }
    }

    /// No district has been on the minimum state share in any of the six years.
    #[test]
    fn nobody_sits_on_the_minimum_state_share() {
        for district in districts() {
            assert!(
                district.state_share_percentage > MINIMUM_STATE_SHARE,
                "FY{} {} is on the {MINIMUM_STATE_SHARE} floor at {}",
                district.fiscal_year,
                district.name,
                district.state_share_percentage,
            );
        }
    }

    /// The amendment is priced in FY2026 and offered in no other year.
    #[test]
    fn item_7_is_priced_only_where_the_enrolment_is_real() {
        assert!(item_7_incidence(FIRST_ITEM_7_YEAR).is_some());
        for year in [2022, 2023, 2024, 2025, 2027] {
            assert!(
                item_7_incidence(year).is_none(),
                "FY{year} is not a year item 7 can be priced in"
            );
        }
    }

    /// A district enrolling exactly its three-year average is unmoved by the amendment.
    ///
    /// This is "the rewrite is a restatement except for the ADM" at the level of one district,
    /// and it is exact rather than approximate because [`District::item_7_delta`] varies the ADM
    /// alone.
    #[test]
    fn a_flat_district_is_unmoved_by_the_rewrite() {
        let mut flat = 0;
        for district in districts().iter().filter(|d| d.method() == Method::Item7) {
            let Some(delta) = district.item_7_delta() else {
                continue;
            };
            if (district.enrolment_ratio() - 1.0).abs() > 1e-9 {
                continue;
            }
            flat += 1;
            assert!(
                delta.abs() < 0.01,
                "FY{} {} enrols its average exactly and moves {delta}",
                district.fiscal_year,
                district.name,
            );
        }
        assert!(
            flat > 0,
            "no district enrols exactly its three-year average, so this proves nothing; FY2027 \
             carries 39 of them because its enrolment has not been counted"
        );
    }

    /// The two ways of saying what prior law pays agree to the rounding that separates them, and
    /// not to the cent.
    ///
    /// Prior law's own arithmetic is a subtraction off a full-precision aggregate; the
    /// counterfactual is a multiplication by a per-pupil figure the department has rounded to the
    /// cent and by a share it publishes to seven places. On a large district those two roundings
    /// are worth a few dollars, which is why [`item_7_incidence`] is built on the second and not
    /// on the difference of the two. The bound here is the grain itself — half a cent per pupil,
    /// plus a part in a million of the aggregate — so a real disagreement would still fail.
    #[test]
    fn the_two_statements_of_prior_law_agree_to_their_rounding() {
        for district in districts().iter().filter(|d| d.method() == Method::Item7) {
            let Some(smoothed) = district.base_cost_aid_on_smoothed_enrolment() else {
                continue;
            };
            let grain = 0.005 * district.base_cost_enrolled_adm
                + 1e-6 * district.aggregate_base_cost
                + 0.01;
            let gap = (smoothed - district.base_cost_aid_under_prior_law()).abs();
            assert!(
                gap <= grain,
                "FY{} {}: the subtraction and the per-pupil product differ by {gap}, past the \
                 {grain} their rounding can account for",
                district.fiscal_year,
                district.name,
            );
        }
    }
}
