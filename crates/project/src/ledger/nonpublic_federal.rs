//! Federal money that reaches chartered nonpublic schools, kept apart from Category 3.
//!
//! [`super::nonpublic_support`] is the state's three Category 3 lines. Two federal channels also
//! reach nonpublic pupils, and neither is part of that category: they are federal grants, with
//! federal formulas, paid as services through public agencies rather than as money to the school.
//! Summing either into Category 3 would describe a state choice that was not made.
//!
//! | Channel | Held as | Years |
//! |---|---|---|
//! | Emergency Assistance to Non-Public Schools | fund `3HQ0`, ALI `200651` in the catalog | FY2022-FY2025 |
//! | IDEA Part B proportionate share | the department's final allocations | FY2021-FY2024 |
//!
//! # One ALI, three programs
//!
//! The catalog reuses an ALI once its program closes, and `200651` has been three programs in
//! three funds: Child Nutrition Services in fund `5B1`, Title IID Technology under the 2009
//! stimulus in fund `3DM0`, and EANS in fund `3HQ0`. [`super::nonpublic_support::claims`] keys on
//! the ALI alone, which is safe for Category 3 and not here: [`eans`] reads through
//! [`super::nonpublic_support::claims_in`], and [`ALI_PROGRAMS`] pins the collision so a fourth
//! use of the number shows up as a failing test, not as a quiet merge.
//!
//! # EANS was never enacted
//!
//! Every edition records EANS's enacted appropriation as **$0**. The money came from the federal
//! grant and the Controlling Board raised the line mid-biennium, so the only appropriation figures
//! are `adjusted` ones, and the catalog carries two of them: FY2023 and FY2025. [`eans_adjusted`]
//! returns those. It does not fill in FY2022 or FY2024, which no held edition states.
//!
//! # The proportionate share is a set-aside, not a transfer
//!
//! 34 CFR 300.133 requires each agency that receives an IDEA Part B subgrant to spend a share of
//! it, proportional to the parentally placed private-school children with disabilities in its
//! territory, on equitable services for those children. The share stays the agency's money and
//! buys services the agency controls. [`idea_part_b`] totals what the department's allocation
//! files say that share was.

use std::collections::BTreeMap;

use edfund_core::{csv, FiscalYear};

use super::nonpublic_support::{self, Claim, Year};

/// The fund EANS was appropriated in: `3HQ0`, the federal stimulus fund GEER also sits in.
pub const EANS_FUND: &str = "3HQ0";

/// The ALI EANS was appropriated in. Not unique to EANS — see [`ALI_PROGRAMS`].
pub const EANS_ALI: &str = "200651";

/// The funds that have used [`EANS_ALI`], with the title each prints, oldest first.
pub const ALI_PROGRAMS: [(&str, &str); 3] = [
    ("5B1", "Child Nutrition Services"),
    ("3DM0", "Title IID Technology"),
    (EANS_FUND, "Emergency Assistance to Non-Public Schools"),
];

/// Every claim the catalog makes about EANS, oldest edition first.
#[must_use]
pub fn eans_claims() -> Vec<Claim> {
    nonpublic_support::claims_in(EANS_FUND, EANS_ALI)
}

/// The years EANS spent money, latest vintage per year, restated into `base` dollars.
///
/// A year is included where its latest edition records an `actual` above zero. Over this fixture
/// that is FY2022 to FY2025, all four answered by the 2025 edition.
#[must_use]
pub fn eans(base: FiscalYear) -> Vec<Year> {
    nonpublic_support::series_in(EANS_FUND, EANS_ALI, base)
        .into_iter()
        .filter(|year| year.kind == "actual" && year.nominal > 0.0)
        .collect()
}

/// Everything EANS spent, in the dollars of each year: the sum of [`eans`]'s nominal amounts.
#[must_use]
pub fn eans_total() -> f64 {
    eans(FiscalYear(2025)).iter().map(|year| year.nominal).sum()
}

/// EANS's `adjusted` appropriations: `(fiscal_year, edition, amount)`, latest edition per year.
///
/// The only appropriation figures the line has, since every enacted figure is $0.
#[must_use]
pub fn eans_adjusted() -> Vec<(u16, u16, f64)> {
    let mut held: BTreeMap<u16, (u16, f64)> = BTreeMap::new();
    for claim in eans_claims() {
        if claim.kind != "adjusted" {
            continue;
        }
        if let Some(amount) = claim.amount {
            held.insert(claim.fiscal_year, (claim.edition, amount));
        }
    }
    held.into_iter()
        .map(|(fiscal_year, (edition, amount))| (fiscal_year, edition, amount))
        .collect()
}

/// One year of EANS beside the same year of Category 3, both nominal.
#[derive(Debug, Clone, PartialEq)]
pub struct AgainstCategoryThree {
    /// The fiscal year.
    pub fiscal_year: u16,
    /// What EANS spent.
    pub eans: f64,
    /// What [`super::nonpublic_support::category_three`] records for the year.
    pub category_three: f64,
}

impl AgainstCategoryThree {
    /// EANS as a fraction of Category 3 in the same year.
    #[must_use]
    pub fn ratio(&self) -> f64 {
        self.eans / self.category_three
    }
}

/// [`eans`] beside Category 3, year by year. A comparison of size, never a sum.
#[must_use]
pub fn against_category_three() -> Vec<AgainstCategoryThree> {
    let base = FiscalYear(2025);
    let state: BTreeMap<u16, f64> = nonpublic_support::category_three(base)
        .into_iter()
        .map(|year| (year.fiscal_year, year.nominal))
        .collect();
    eans(base)
        .into_iter()
        .filter_map(|year| {
            Some(AgainstCategoryThree {
                fiscal_year: year.fiscal_year,
                eans: year.nominal,
                category_three: *state.get(&year.fiscal_year)?,
            })
        })
        .collect()
}

/// The committed allocation panel: one row per agency per fiscal year.
const IDEA: &str = include_str!("../../fixtures/idea-part-b-allocations.csv");

/// The header [`idea_part_b`] indexes against.
const IDEA_HEADER: &str = "fiscal_year,irn,public_swd,nonpublic_swd,allocation,proportionate_share";

/// The columns of [`IDEA_HEADER`], named where they are read.
mod idea_column {
    pub const FISCAL_YEAR: usize = 0;
    pub const NONPUBLIC_SWD: usize = 3;
    pub const ALLOCATION: usize = 4;
    pub const PROPORTIONATE_SHARE: usize = 5;
}

/// One year of IDEA Part B, statewide, as the department's final allocations add up.
#[derive(Debug, Clone, PartialEq)]
pub struct IdeaYear {
    /// The fiscal year of the grant.
    pub fiscal_year: u16,
    /// How many agencies the edition lists, paid or not.
    pub agencies: usize,
    /// The school-age allocation, summed.
    pub allocation: f64,
    /// The proportionate share, summed. A blank share contributes nothing.
    pub proportionate_share: f64,
    /// How many agencies owe a share above zero.
    pub sharing_agencies: usize,
    /// Nonpublic students with disabilities, summed over the cells printed as a number.
    pub nonpublic_swd: u32,
    /// How many nonpublic count cells are suppressed (`<10` or `-`), so [`Self::nonpublic_swd`]
    /// is a floor whenever this is above zero.
    pub nonpublic_suppressed: usize,
    /// How many agencies print no share at all — blank, not zero.
    pub blank_shares: usize,
}

impl IdeaYear {
    /// The proportionate share as a fraction of the allocation.
    #[must_use]
    pub fn share(&self) -> f64 {
        self.proportionate_share / self.allocation
    }
}

/// Each fiscal year of the allocation panel, oldest first.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against.
#[must_use]
pub fn idea_part_b() -> Vec<IdeaYear> {
    let mut years: BTreeMap<u16, IdeaYear> = BTreeMap::new();
    for row in csv::rows(IDEA, IDEA_HEADER) {
        let fiscal_year: u16 = row
            .str(idea_column::FISCAL_YEAR)
            .parse()
            .expect("a fiscal year");
        let year = years.entry(fiscal_year).or_insert(IdeaYear {
            fiscal_year,
            agencies: 0,
            allocation: 0.0,
            proportionate_share: 0.0,
            sharing_agencies: 0,
            nonpublic_swd: 0,
            nonpublic_suppressed: 0,
            blank_shares: 0,
        });
        year.agencies += 1;
        year.allocation += row.required(idea_column::ALLOCATION);
        match row.num(idea_column::PROPORTIONATE_SHARE) {
            Some(share) => {
                year.proportionate_share += share;
                if share > 0.0 {
                    year.sharing_agencies += 1;
                }
            }
            None => year.blank_shares += 1,
        }
        match row.str(idea_column::NONPUBLIC_SWD) {
            "" => {}
            "<10" | "-" => year.nonpublic_suppressed += 1,
            count => year.nonpublic_swd += count.parse::<u32>().expect("a count"),
        }
    }
    years.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_eans_reader_sees_only_fund_3hq0() {
        assert!(eans_claims().iter().all(|claim| claim.fund == EANS_FUND));
        assert!(!eans_claims().is_empty());
    }

    #[test]
    fn a_blank_idea_share_is_not_counted_as_a_sharing_agency() {
        let fy2021 = idea_part_b()
            .into_iter()
            .find(|year| year.fiscal_year == 2021)
            .expect("FY2021");
        assert_eq!(fy2021.blank_shares, 1);
    }
}
