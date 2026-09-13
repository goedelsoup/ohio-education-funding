//! What a transportation proration factor is a quotient of.
//!
//! `parameter/appropriation-proration-factor` holds that a factor below one "says the entitlement
//! was computed, the appropriation was compared against it, and the two did not bind this year",
//! and asks which appropriation each of the plan's three factors is set against. LSC answers the
//! shape of the question in one line, written for a transportation proration two decades older
//! than this one:
//!
//! > Adjustment percentage = (Earmarked appropriation)/(Total statewide allocation)
//!
//! So a factor is a quotient, and this module holds both halves of it for the two years the
//! department's calculator can be observed over. The numerator comes from LSC's **enacted**
//! analysis of H.B. 96 — [`budget_analysis`] — and the denominator from the calculators
//! themselves, FY2027 through [`mod@panel`] and FY2026 through [`prior_model`].
//!
//! # The denominator is bigger than the model
//!
//! That is the finding this exists to carry. The FY2027 workbook states a special education
//! transportation factor of `0.91745974`, and the earmark it divides is $194,820,866; the
//! allocation those two numbers imply is **$212.3m**, against the **$199.1m** the workbook's own
//! 611 districts account for. The $13.3m left over is not a rounding: the greenbook says who it
//! belongs to, in the paragraph describing the earmark —
//!
//! > County DD boards and ESCs are funded through a nearly identical special education
//! > transportation formula, except that the state share percentage for these entities is a
//! > uniform amount equal to the minimums for traditional districts.
//!
//! A proration factor is therefore the one number in the calculator that measures a population
//! the calculator does not contain. FY2026 bounds the same quantity from the other side: the
//! factor is 1.0, so the whole statewide allocation fitted inside $176,897,678, and the districts
//! took $165,119,518 of it.
//!
//! # And FY2027 prorates because the floor stepped, not because the line was cut
//!
//! The earmark **rose 10.13%** between the two years. The district allocation rose 20.56%, and
//! [`Movement`] splits that into the three things that moved: the prior year's reported costs
//! (+$26.8m), the state share percentages, which fell and so took $4.2m back off, and the
//! statutory floor under the state share, which H.B. 96 stepped from 45.83% to 50% and which adds
//! **$11.3m** on its own.

use edfund_core::{csv, Dollars};

use crate::ledger::budget_analysis::{
    self, Edition, FOUNDATION_AID_EARMARKS, FOUNDATION_AID_REMAINDER,
    SPECIAL_EDUCATION_TRANSPORTATION,
};
use crate::{panel, prior_model};

/// The committed series of statewide transportation factors, one row per fiscal year.
const FIXTURE: &str = include_str!("../fixtures/transportation-rates.csv");

/// The header this reader was written against.
const EXPECTED_HEADER: &str =
    "fiscal_year,per_rider,per_mile,sped_proration,minimum_state_share,inflation_factor";

/// The years both calculators cover, oldest first.
pub const YEARS: (u16, u16) = (2026, 2027);

/// One fiscal year's statewide transportation factors, as its calculator states them.
///
/// # Why this is in the library
///
/// Three test files read this fixture and two of them had written their own parser for it — the
/// same drift [`budget_analysis`] was lifted out of. A third reader was the point at which to
/// stop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rates {
    /// The year the model computes.
    pub fiscal_year: u16,
    /// `[s1]` per weighted rider.
    pub per_rider: Dollars,
    /// `[s2]` per bus mile, before the 180-day annualisation.
    pub per_mile: Dollars,
    /// The special education transportation proration factor.
    pub sped_proration: f64,
    /// The floor under the state share, which R.C. 3317.0212 steps by a twenty-fourth a year.
    pub minimum_state_share: f64,
    /// A 5% inflation factor that sits beside the rates in both years and is in no section.
    pub inflation_factor: f64,
}

/// Every year's factors, oldest first.
///
/// # Panics
///
/// If the fixture's header is not the one this reader was written against.
#[must_use]
pub fn rates() -> Vec<Rates> {
    csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            Some(Rates {
                fiscal_year: row.str(0).parse().ok()?,
                per_rider: row.num(1)?,
                per_mile: row.num(2)?,
                sped_proration: row.num(3)?,
                minimum_state_share: row.num(4)?,
                inflation_factor: row.num(5)?,
            })
        })
        .collect()
}

/// One year's factors.
#[must_use]
pub fn rates_for(fiscal_year: u16) -> Option<Rates> {
    rates().into_iter().find(|r| r.fiscal_year == fiscal_year)
}

/// One district's two inputs to the special education transportation formula, and its payment.
///
/// The formula is `max(state share, floor) x reported cost x proration`, and the reported cost is
/// the **prior** fiscal year's — so the FY2027 column is what districts spent in FY2025.
#[derive(Debug, Clone, PartialEq)]
pub struct SpedCost {
    /// Information Retrieval Number.
    pub irn: String,
    /// `[b4]` the district's own state share percentage, before the floor.
    ///
    /// The `Transportation` sheet prints it again as `[i] State Share Percent` and the two agree
    /// to the last digit on every district in both workbooks, so the panel's published column
    /// stands in for a column the panel does not carry.
    pub state_share: f64,
    /// `[j]` what it reported spending last year.
    pub reported_cost: Dollars,
    /// `[J]` what it is paid, after the year's proration.
    pub paid: Dollars,
}

/// Every district's inputs for one year, from that year's calculator.
///
/// The two Lake Erie island districts are in the department's 611 and not in the 609-district
/// panel; both report zero transportation cost and zero payment in both workbooks, so the two
/// populations give the same statewide totals here. See
/// `the_exclusion_that_was_computed_once` for the other place that difference shows up.
#[must_use]
pub fn sped_costs(fiscal_year: u16) -> Option<Vec<SpedCost>> {
    match fiscal_year {
        2026 => Some(
            prior_model::frame()
                .into_iter()
                .map(|row| SpedCost {
                    irn: row.irn,
                    state_share: row.state_share_percentage,
                    reported_cost: row.reported_sped_transport_cost,
                    paid: row.special_education_transportation,
                })
                .collect(),
        ),
        2027 => Some(
            panel::panel()
                .iter()
                .map(|record| SpedCost {
                    irn: record.irn.clone(),
                    state_share: record.published_state_share.unwrap_or(0.0),
                    reported_cost: record.transportation.reported_sped_cost,
                    paid: record.transportation.special_education,
                })
                .collect(),
        ),
        _ => None,
    }
}

/// What the districts in a year's calculator are allocated, against what the act appropriated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Allocation {
    /// The year the model computes.
    pub fiscal_year: u16,
    /// The prior year's reported cost, statewide.
    pub reported_cost: Dollars,
    /// `sum(max(state share, floor) x reported cost)` — the allocation before proration.
    pub districts: Dollars,
    /// `sum([J])` — what the districts are actually paid.
    pub paid: Dollars,
    /// How many districts the floor actually pays: state share under it, and a cost to reimburse.
    ///
    /// The reported-cost condition is what makes the two years comparable. A district that
    /// reports nothing is under the floor and is paid nothing either way, and there are 52 of
    /// them in FY2026 and 48 in FY2027 — so the plain count of districts under the floor, which
    /// `parameter/transportation-cost-rates` carries as **357 and 440** on the department's 611,
    /// moves partly with the number of districts that filed.
    pub at_the_floor: usize,
    /// The enacted earmark: LSC's numerator.
    pub appropriation: Dollars,
    /// The factor the workbook states.
    pub stated_factor: f64,
}

impl Allocation {
    /// The statewide allocation the stated factor implies — LSC's denominator, solved for.
    #[must_use]
    pub fn implied_statewide(&self) -> Dollars {
        self.appropriation / self.stated_factor
    }

    /// What that denominator holds which the calculator's districts do not.
    ///
    /// County DD boards and educational service centres, per the greenbook. Positive in a year
    /// that prorates; in a year that does not, the factor is 1.0 by construction and this is only
    /// the headroom the appropriation had left, which [`Allocation::headroom`] says better.
    #[must_use]
    pub fn outside_the_model(&self) -> Dollars {
        self.implied_statewide() - self.districts
    }

    /// The appropriation less what the districts alone are allocated.
    ///
    /// Positive means the districts fit inside the line with room for everyone else; negative
    /// means they do not fit on their own.
    #[must_use]
    pub fn headroom(&self) -> Dollars {
        self.appropriation - self.districts
    }

    /// The factor the districts alone would have produced.
    ///
    /// The distance between this and [`Allocation::stated_factor`] is the whole of the case that
    /// the denominator reaches past the model.
    #[must_use]
    pub fn districts_only_factor(&self) -> f64 {
        self.appropriation / self.districts
    }
}

/// One year's special education transportation allocation, both halves of its factor.
///
/// # Panics
///
/// If the year's earmark row is not in the enacted analysis, by way of
/// [`budget_analysis::row`].
#[must_use]
pub fn special_education(fiscal_year: u16) -> Option<Allocation> {
    let rates = rates_for(fiscal_year)?;
    let costs = sped_costs(fiscal_year)?;
    let earmark = budget_analysis::pupil_transportation_earmarks(
        Edition::Enacted,
        SPECIAL_EDUCATION_TRANSPORTATION,
    );
    Some(Allocation {
        fiscal_year,
        reported_cost: costs.iter().map(|c| c.reported_cost).sum(),
        districts: allocation_at(&costs, rates.minimum_state_share),
        paid: costs.iter().map(|c| c.paid).sum(),
        at_the_floor: costs
            .iter()
            .filter(|c| c.state_share < rates.minimum_state_share && c.reported_cost > 0.0)
            .count(),
        appropriation: enacted_year(fiscal_year, earmark)?,
        stated_factor: rates.sped_proration,
    })
}

/// What a rise in reported cost does to the special education proration factor.
///
/// # Why this is parameterised by the cost rise and not by a fuel price
///
/// The obvious question is what a diesel shock does to this factor, and it cannot be asked that
/// way. Neither rate in R.C. 3317.0212 is a price: both are **trimmed means of what districts
/// reported spending in the prior year**, so a fuel shock reaches the payment only through
/// districts' own books and only a year later. The size of that pass-through is not identified by
/// anything this repository can reach —
/// `dispersion::tests::the_fuel_response_the_panel_cannot_identify` fits eleven specifications and
/// every one of them spans zero, and the blocker is not sample size but the absence of a mile
/// column in the F-33 in any year.
///
/// So the coefficient is left where it is. `rise` is the fraction by which **reported cost**
/// increases, whatever moved it, and the arithmetic below is exact given that. A reader who has a
/// fuel-share estimate can compose the two; this module declines to supply one.
///
/// # The two bounds, and why there are two
///
/// The denominator LSC prorates against is larger than the calculator's districts: solving
/// [`Allocation::implied_statewide`] leaves **$13.3m in FY2027** that the districts do not
/// account for, which the greenbook attributes to county DD boards and educational service
/// centres. Whether *their* reported costs rise with the districts' is not established here, so
/// both ends are computed:
///
/// - [`Self::factor_districts_only`] holds that remainder fixed, which is the shallower proration.
/// - [`Self::factor_uniform`] moves the whole denominator, which is the deeper one.
///
/// They are close — the remainder is about 6% of the denominator — and the gap is the honest width
/// of the answer rather than a choice to be made.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Proration {
    /// The year whose allocation is being re-prorated.
    pub fiscal_year: u16,
    /// The fractional rise in reported cost. `0.05` is five per cent.
    pub rise: f64,
    /// The factor if only the calculator's districts report more.
    pub factor_districts_only: f64,
    /// The factor if everything in LSC's denominator rises together.
    pub factor_uniform: f64,
    /// What the districts are allocated after the rise, before proration.
    pub allocation: Dollars,
    /// What they are paid at [`Self::factor_districts_only`].
    pub paid: Dollars,
    /// The allocation they earn and do not receive, at the same factor.
    pub withheld: Dollars,
}

/// The proration factor at a given rise in reported cost, against a fixed appropriation.
///
/// At `rise` of zero this reproduces the year's published factor and its published payment, which
/// is what makes the rest of the curve worth reading.
///
/// # In a year that does not prorate, the result is a lower bound
///
/// [`Allocation::outside_the_model`] solves the remainder out of a factor below 1.0. Where the
/// factor *is* 1.0 — FY2026 — nothing is solved and the method returns the headroom instead, which
/// is an **upper** bound on the remainder. A larger remainder means a larger denominator and so a
/// smaller factor, so the curve this returns for such a year is the deepest proration consistent
/// with what is known, and the true one is at or above it. FY2027 identifies the remainder and
/// needs no such reading.
///
/// # The finding this exists to state
///
/// The plan's self-correction for a cost shock is capped by a number the plan does not contain.
/// Divisions (C) and (D) of R.C. 3317.0212 earn a district more when its reported costs rise, and
/// the earmark inside GRF ALI 200502 decides how much of that it actually receives — so a rate
/// rise that is fully earned can still arrive scaled down, and the scaling appears nowhere in the
/// published per-pupil amount.
#[must_use]
pub fn proration_under(fiscal_year: u16, rise: f64) -> Option<Proration> {
    let allocation = special_education(fiscal_year)?;
    let outside = allocation.outside_the_model();
    let risen = allocation.districts * (1.0 + rise);
    let factor_districts_only = (allocation.appropriation / (risen + outside)).min(1.0);
    let factor_uniform =
        (allocation.appropriation / (allocation.implied_statewide() * (1.0 + rise))).min(1.0);
    Some(Proration {
        fiscal_year,
        rise,
        factor_districts_only,
        factor_uniform,
        allocation: risen,
        paid: risen * factor_districts_only,
        withheld: risen * (1.0 - factor_districts_only),
    })
}

/// The rise in reported cost that takes the factor down to `target`, on the shallower bound.
///
/// Inverts [`proration_under`]. `None` where the target is not reachable — above the year's
/// current factor, or outside `(0, 1]`.
#[must_use]
pub fn rise_reaching(fiscal_year: u16, target: f64) -> Option<f64> {
    if target <= 0.0 || target > 1.0 {
        return None;
    }
    let allocation = special_education(fiscal_year)?;
    let outside = allocation.outside_the_model();
    let rise = (allocation.appropriation / target - outside) / allocation.districts - 1.0;
    (rise >= 0.0).then_some(rise)
}

/// The allocation a set of districts produces under a stated floor.
///
/// Separated from [`special_education`] because the counterfactual in [`Movement`] is the same
/// arithmetic with one input held at the other year's value.
#[must_use]
pub fn allocation_at(costs: &[SpedCost], floor: f64) -> Dollars {
    costs
        .iter()
        .map(|c| c.state_share.max(floor) * c.reported_cost)
        .sum()
}

/// Why the district allocation moved between the two years, in the three things that moved.
///
/// Each step holds everything to its left at FY2027 and everything to its right at FY2026, so the
/// four levels are a path from one year's allocation to the other's and the three deltas sum to
/// the total movement exactly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Movement {
    /// FY2026's allocation: its costs, its state shares, its floor.
    pub from: Dollars,
    /// FY2027's reported costs, at FY2026's state shares and floor.
    pub after_cost: Dollars,
    /// And FY2027's state shares, still at FY2026's floor.
    pub after_state_share: Dollars,
    /// And FY2027's floor, which is the year's actual allocation.
    pub to: Dollars,
}

impl Movement {
    /// What the prior year's reported costs added.
    #[must_use]
    pub fn cost(&self) -> Dollars {
        self.after_cost - self.from
    }

    /// What the state share percentages took back off, which is negative: they fell.
    #[must_use]
    pub fn state_share(&self) -> Dollars {
        self.after_state_share - self.after_cost
    }

    /// What the floor's step from 45.83% to 50% added.
    #[must_use]
    pub fn floor(&self) -> Dollars {
        self.to - self.after_state_share
    }
}

/// The decomposition, over the one interval the calculators can be compared across.
///
/// Paired on IRN, so the two island districts — in the department's 611 and not in the panel —
/// are excluded from every level rather than from some of them.
#[must_use]
pub fn movement() -> Option<Movement> {
    let (early, late) = (rates_for(YEARS.0)?, rates_for(YEARS.1)?);
    let (before, after) = (sped_costs(YEARS.0)?, sped_costs(YEARS.1)?);
    let paired = |costs: &[SpedCost], other: &[SpedCost], floor: f64| -> Dollars {
        costs
            .iter()
            .filter(|c| other.iter().any(|o| o.irn == c.irn))
            .map(|c| c.state_share.max(floor) * c.reported_cost)
            .sum()
    };
    let shares_of = |costs: &[SpedCost], from: &[SpedCost], floor: f64| -> Dollars {
        costs
            .iter()
            .filter_map(|c| {
                let share = from.iter().find(|o| o.irn == c.irn)?.state_share;
                Some(share.max(floor) * c.reported_cost)
            })
            .sum()
    };
    Some(Movement {
        from: paired(&before, &after, early.minimum_state_share),
        after_cost: shares_of(&after, &before, early.minimum_state_share),
        after_state_share: paired(&after, &before, early.minimum_state_share),
        to: paired(&after, &before, late.minimum_state_share),
    })
}

/// What the general transportation formula is paid from, and how close it came to binding.
///
/// Returned as `(the districts' transportation funding, the unearmarked remainder of ALI 200502)`.
/// The general proration factor is 1.0 in both years, and 1.0 is a measurement: it says this
/// subtraction came out positive. It does not say by how much, which is the part worth having —
/// **0.80% of the line in FY2026 and 4.82% in FY2027**.
///
/// The remainder is the document's own row rather than this crate's subtraction, which matters
/// because the earmark table and the fund/ALI split are two different tables and only the second
/// is exhaustive.
#[must_use]
pub fn general_headroom(fiscal_year: u16) -> Option<(Dollars, Dollars)> {
    let paid: Dollars = match fiscal_year {
        2026 => prior_model::frame()
            .iter()
            .map(|row| row.transportation_total)
            .sum(),
        2027 => panel::panel()
            .iter()
            .map(|record| record.transportation.total)
            .sum(),
        _ => return None,
    };
    let remainder =
        budget_analysis::pupil_transportation(Edition::Enacted, FOUNDATION_AID_REMAINDER);
    Some((paid, enacted_year(fiscal_year, remainder)?))
}

/// The whole of ALI 200502's earmarks, which is what the remainder is the remainder of.
#[must_use]
pub fn earmarked(fiscal_year: u16) -> Option<Dollars> {
    enacted_year(
        fiscal_year,
        budget_analysis::pupil_transportation(Edition::Enacted, FOUNDATION_AID_EARMARKS),
    )
}

/// The column of a three-column table that belongs to one fiscal year of this biennium.
fn enacted_year(fiscal_year: u16, row: budget_analysis::Row) -> Option<Dollars> {
    match fiscal_year {
        2026 => Some(row.first),
        2027 => Some(row.second),
        _ => None,
    }
}

/// What the transportation floor is worth, and to whom.
///
/// # The floor, which is not the formula's floor
///
/// R.C. 3317.0212(E)(1)(c) pays a district's calculated transportation amount at the **greater of
/// its own state share percentage and a fixed minimum**. H.B. 96 raised that minimum 41.67% →
/// 45.83% → 50% across FY2025-FY2027, and [`Rates::minimum_state_share`] carries it. It is a
/// different floor from the formula's 10% minimum state share, five times the size, and it binds
/// on far more of the state: 438 of the 609 districts this panel costs sit on it, against 23% on
/// the formula's.
///
/// # Why the gross has to be recovered rather than read
///
/// The department's model publishes transportation **net** — each of the five payment components
/// already has the applied share inside it, and `[G] Total` is their sum plus the `[F]` guarantee,
/// exactly, on every district. So a different floor cannot be priced by scaling the total: the
/// share has to come back out first.
///
/// It comes out cleanly because the applied share is known. Dividing a component by
/// `max(state_share, floor_in_force)` recovers the calculated amount, and the recovery is checked
/// rather than assumed — the recovered `[A] School Bus` reproduces `max(weighted riders × the
/// rider rate, bus miles × the mile rate × 180)` on **601 of 601** districts, and the statewide
/// total at the floor in force reproduces the published $726.1M. See
/// `the_floor_that_pays_the_wealthy_districts`.
///
/// # What it is worth, and to whom
///
/// **$289.6M**, which is 40% of Ohio's transportation aid — the floor pays more than the
/// district's own share does for two fifths of the programme. And it runs **up** the wealth
/// distribution, monotonically: $3.38 per pupil in the poorest fifth of districts by assessed
/// valuation against **$398.01 in the richest**, a hundred and eighteen times. Every district in
/// the top two quintiles is on it; thirteen of 120 in the bottom are.
///
/// That is the opposite shape from the rest of the formula, and it is the fact a proposal to move
/// this floor needs stated beside it: raising it is a payment to the districts whose local capacity
/// is highest, and lowering it falls on the same districts.
pub mod floor {
    use edfund_core::Dollars;

    use crate::panel::{self, DistrictRecord};

    /// One district's transportation aid, split so another floor can be priced.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Paid {
        /// The district.
        pub irn: String,
        /// `[b4]`, the district's own state share percentage.
        pub state_share: f64,
        /// The calculated amount before any share is applied.
        pub gross: Dollars,
        /// `[F]`, the transportation guarantee, which no share touches.
        pub guarantee: Dollars,
        /// Assessed valuation per pupil, for the incidence question.
        pub valuation_per_pupil: Option<Dollars>,
        /// Enrolled ADM, for a per-pupil reading.
        pub enrolled_adm: f64,
    }

    impl Paid {
        /// The share actually applied at a given floor.
        #[must_use]
        pub fn applied(&self, floor: f64) -> f64 {
            self.state_share.max(floor)
        }

        /// What the district is paid at a given floor.
        #[must_use]
        pub fn under(&self, floor: f64) -> Dollars {
            self.gross * self.applied(floor) + self.guarantee
        }

        /// Whether the floor binds — the district's own share is below it.
        #[must_use]
        pub fn on_floor(&self, floor: f64) -> bool {
            self.state_share < floor
        }

        /// What the floor is worth to this district: the payment above its own share.
        #[must_use]
        pub fn floor_value(&self, floor: f64) -> Dollars {
            self.gross * (floor - self.state_share).max(0.0)
        }
    }

    /// Every district's transportation aid, decomposed against the floor in force that year.
    ///
    /// Districts reporting no transportation payment or no state share are left out rather than
    /// carried at zero, because a zero here is an absence of the programme and not a district paid
    /// nothing by it.
    ///
    /// `None` where the year has no published rates.
    #[must_use]
    pub fn paid(fiscal_year: u16) -> Option<Vec<Paid>> {
        let in_force = super::rates_for(fiscal_year)?.minimum_state_share;
        let recovered = |record: &DistrictRecord| {
            let share = record.published_state_share?;
            let components = record.transportation.components();
            (share > 0.0 && components > 0.0).then(|| Paid {
                irn: record.irn.clone(),
                state_share: share,
                gross: components / share.max(in_force),
                guarantee: record.transportation.guarantee,
                valuation_per_pupil: record.valuation_per_pupil,
                enrolled_adm: record.current_year_adm,
            })
        };
        Some(panel::panel().iter().filter_map(recovered).collect())
    }

    /// Statewide transportation aid at a given floor.
    #[must_use]
    pub fn statewide_under(fiscal_year: u16, floor: f64) -> Option<Dollars> {
        Some(paid(fiscal_year)?.iter().map(|d| d.under(floor)).sum())
    }

    /// What the floor in force is worth statewide, against paying every district its own share.
    #[must_use]
    pub fn value_of_the_floor(fiscal_year: u16) -> Option<Dollars> {
        let in_force = super::rates_for(fiscal_year)?.minimum_state_share;
        Some(statewide_under(fiscal_year, in_force)? - statewide_under(fiscal_year, 0.0)?)
    }

    /// One band of the wealth distribution, and what the floor pays into it.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Band {
        /// `1` is the least wealthy fifth by assessed valuation per pupil.
        pub quintile: usize,
        /// Districts in it.
        pub districts: usize,
        /// Those the floor binds on.
        pub on_floor: usize,
        /// What the floor pays them.
        pub value: Dollars,
        /// Their enrolled ADM between them.
        pub enrolled_adm: f64,
    }

    impl Band {
        /// The floor's value per pupil in this band.
        #[must_use]
        pub fn per_pupil(&self) -> Option<Dollars> {
            (self.enrolled_adm > 0.0).then(|| self.value / self.enrolled_adm)
        }
    }

    /// The floor's incidence across valuation-per-pupil quintiles, least wealthy first.
    ///
    /// Districts publishing no valuation are dropped, because the axis is the valuation.
    ///
    /// `None` where the year has no published rates.
    #[must_use]
    pub fn incidence(fiscal_year: u16) -> Option<Vec<Band>> {
        let floor = super::rates_for(fiscal_year)?.minimum_state_share;
        let mut districts: Vec<Paid> = paid(fiscal_year)?
            .into_iter()
            .filter(|d| d.valuation_per_pupil.is_some())
            .collect();
        districts.sort_by(|a, b| {
            a.valuation_per_pupil
                .unwrap_or_default()
                .total_cmp(&b.valuation_per_pupil.unwrap_or_default())
        });

        let width = districts.len() / 5;
        Some(
            (0..5)
                .map(|index| {
                    let band = if index == 4 {
                        &districts[index * width..]
                    } else {
                        &districts[index * width..(index + 1) * width]
                    };
                    Band {
                        quintile: index + 1,
                        districts: band.len(),
                        on_floor: band.iter().filter(|d| d.on_floor(floor)).count(),
                        value: band.iter().map(|d| d.floor_value(floor)).sum(),
                        enrolled_adm: band.iter().map(|d| d.enrolled_adm).sum(),
                    }
                })
                .collect(),
        )
    }
}
