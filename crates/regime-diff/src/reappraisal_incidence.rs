//! Who a reappraisal reaches, and why it is not the districts the equity node named.
//!
//! # The question
//!
//! `doctrine/equity` recorded a break in Ohio's equalization series at FY2023 and left its cause
//! open between two candidates: *"Ohio's triennial property reappraisals landed across FY2023 and
//! would raise local revenue in the richest quartile mechanically — widening the gap with nothing
//! about state aid having changed — and the Fair School Funding Plan's own phase-in moves state
//! aid over the same years. Separating the two needs the valuation series beside this one, and two
//! observations cannot do it."*
//!
//! Three things were wrong with that, and the third is the one that mattered.
//!
//! # There are not two observations, there are four, and they are staggered
//!
//! Table SD-1 carries assessed valuation for TY2021 through TY2024, and Ohio's 88 counties do not
//! reappraise together. [`crate::recognized_valuation::CYCLE`] holds the calendar, and every
//! county has exactly one event in TY2022-TY2024 — so a district's own county gives it one event
//! year and two quiet ones, and the quiet years measure what it would have done anyway. That is
//! not a difference of two observations; it is a natural experiment with three arms.
//!
//! [`districts`] assembles it and the arms separate cleanly. Median class-I valuation growth in a
//! cohort's own event year is **+21.6%, +35.7% and +32.0%** for the TY2022, TY2023 and TY2024
//! cohorts; in their quiet years it runs **+0.65% to +0.91%**. That is a ratio of twenty-four to
//! fifty-five, which is why the identification does not need a threshold argued for.
//!
//! # A reappraisal does not reach revenue, except where it does
//!
//! The premise underneath "would raise local revenue mechanically" is the part that fails. H.B.
//! 920's reduction factors exist to stop exactly that: R.C. 319.301 recomputes a percentage every
//! year that holds the yield of existing levies flat as values rise. A reappraisal moves a district's
//! revenue only where the factors have stopped operating — at the twenty-mill floor, which is
//! [`millage::FloorStatus::valuation_growth_reaches_revenue`].
//!
//! **And floor districts are poor.** [`by_wealth`] puts **46.1% of the poorest quartile** at the
//! floor against **12.5% of the richest**, falling monotonically across the four. So where a
//! reappraisal reaches revenue at all, it reaches the bottom of the distribution hardest, and the
//! mechanism the node proposed for widening the gap is one that narrows it. The same table has
//! the poorest quartile's local revenue per pupil growing 22.5% across the window against the
//! richest quartile's 11.3%.
//!
//! The floor is read off Table SD-1's class-I effective rate, at or below twenty mills — the
//! convention `bundle` already uses, under which a district that never voted twenty mills is at
//! the floor rather than above it. SD-1 puts 192 of these 608 districts there in TY2023; the
//! District Profile Report's own rate puts 170 of 606, which is the margin between two published
//! series and not a disagreement about the statute.
//!
//! [`gap`] is that stated as a number: replacing each district's event-year growth with its own
//! quiet-year rate moves the FY2023 gap by **+$26** and the FY2024 gap by **-$212**. Reappraisal
//! is not why the gap moved, and in the second year it was holding the gap down.
//!
//! # And the break was not real
//!
//! The third thing, and the reason this module answers a question that has stopped being asked in
//! the form it was posed: there was no FY2023 break. It was one five-pupil island district
//! entering the survey's comparable population — see
//! [`dispersion::ohio_panel::MIN_ENROLMENT`]. With it excluded the state closes 44.9% and 43.9%
//! of the gap in the two years, inside the band the corpus already recorded.
//!
//! What survives is this module's own finding, which does not depend on the artefact: the
//! incidence of a reappraisal in Ohio runs down the wealth distribution and not up it.
//!
//! # What this cannot see
//!
//! Taxes charged for a tax year are collected across two fiscal years, so an event lands partly in
//! FY(T+1) and partly in FY(T+2). The counterfactual charges the whole of it to the first, which
//! overstates the single-year effect it is trying to remove — and it is *still* the wrong sign for
//! the node's hypothesis, which is why the direction is reported and the magnitude is not pinned.
//!
//! The panel ends at FY2024. The TY2024 cohort's revaluation is visible in SD-1 — it is the
//! +32.0% above — but the charges it produces are collected from FY2025, so those 181 districts
//! contribute two quiet revenue years and nothing else. They are the control arm rather than a
//! gap in the data.

use std::collections::BTreeMap;

use edfund_core::AgencyType;

/// The fiscal years the comparison runs between: the last year before the recorded break, and the
/// last year the panel holds.
pub const WINDOW: [u16; 2] = [2022, 2024];

/// The tax year floor status is read at — the one whose charges land in the last year of
/// [`WINDOW`].
pub const FLOOR_TAX_YEAR: u16 = 2023;

/// How far above the floor still counts as at it.
///
/// SD-1 publishes effective rates to four decimals in three of its four tax years and to two in
/// TY2023, which is the year this reads. Half of the last published unit is five thousandths of a
/// mill, and a district at the floor is at it by statute rather than by rounding — R.C.
/// 319.301(E)(2) is a bound, not a target.
pub const PUBLISHED_HALF_UNIT: f64 = 5e-3;

/// One district, its county's place in the reappraisal calendar, and what its local revenue did.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Ohio's identifier.
    pub irn: String,
    /// The county whose calendar it is on.
    pub county: String,
    /// The tax year its county revalued in. Its charges land in the following fiscal year.
    pub event_tax_year: u16,
    /// Whether reduction factors have stopped operating, so valuation growth reaches revenue.
    pub at_the_floor: bool,
    /// Local revenue per pupil, by fiscal year, over the years [`WINDOW`] spans.
    pub local_per_pupil: BTreeMap<u16, f64>,
    /// Class-I assessed valuation by tax year, TY2021 through TY2023.
    pub class1_value: BTreeMap<u16, f64>,
}

impl District {
    /// The fiscal year this district's reappraisal is charged in.
    #[must_use]
    pub const fn event_fiscal_year(&self) -> u16 {
        self.event_tax_year + 1
    }

    /// What its local revenue per pupil grew by in a year its county did nothing.
    ///
    /// The TY2024 cohort has no event inside the window at all, so both of its years are quiet and
    /// this is their geometric mean. For the other two cohorts it is the single quiet year.
    #[must_use]
    pub fn quiet_growth(&self) -> f64 {
        let (Some(first), Some(middle), Some(last)) = (
            self.local_per_pupil.get(&WINDOW[0]),
            self.local_per_pupil.get(&(WINDOW[0] + 1)),
            self.local_per_pupil.get(&WINDOW[1]),
        ) else {
            return f64::NAN;
        };
        match self.event_fiscal_year() {
            y if y == WINDOW[0] + 1 => last / middle - 1.0,
            y if y == WINDOW[1] => middle / first - 1.0,
            _ => (last / first).sqrt() - 1.0,
        }
    }

    /// Local revenue per pupil as it would have run had the county not revalued.
    #[must_use]
    pub fn counterfactual(&self, fiscal_year: u16) -> f64 {
        let base = self
            .local_per_pupil
            .get(&WINDOW[0])
            .copied()
            .unwrap_or(f64::NAN);
        let steps = i32::from(fiscal_year) - i32::from(WINDOW[0]);
        base * (1.0 + self.quiet_growth()).powi(steps)
    }
}

/// Every district Table SD-1 and the survey panel both reach across the whole window.
///
/// Sorted by local revenue per pupil in the first year of [`WINDOW`], so the quartile cuts below
/// are cuts on wealth as the equalization measure defines it.
#[must_use]
pub fn districts() -> Vec<District> {
    let floor = millage::floor_for(AgencyType::City).unwrap_or(20.0);

    let mut taxes: BTreeMap<String, Vec<dispersion::sd1::TaxRow>> = BTreeMap::new();
    for row in dispersion::sd1::rows() {
        taxes.entry(row.irn.clone()).or_default().push(row);
    }

    let mut revenue: BTreeMap<String, BTreeMap<u16, f64>> = BTreeMap::new();
    for row in dispersion::ohio_panel::panel() {
        if row.comparable
            && row.enrollment >= dispersion::ohio_panel::MIN_ENROLMENT
            && (WINDOW[0]..=WINDOW[1]).contains(&row.fiscal_year)
        {
            revenue
                .entry(row.irn.clone())
                .or_default()
                .insert(row.fiscal_year, row.local_revenue / row.enrollment);
        }
    }

    let years: Vec<u16> = (WINDOW[0]..=WINDOW[1]).collect();
    let mut out = Vec::new();
    for (irn, local_per_pupil) in revenue {
        if years.iter().any(|y| !local_per_pupil.contains_key(y)) {
            continue;
        }
        let Some(rows) = taxes.get(&irn) else {
            continue;
        };
        let Some(county) = rows.first().map(|r| r.county.clone()) else {
            continue;
        };
        let Some(cycle) = crate::recognized_valuation::cycle_for(&county) else {
            continue;
        };

        let mut class1_value = BTreeMap::new();
        for row in rows {
            if let Some(value) = row.class1_value {
                class1_value.insert(row.tax_year, value);
            }
        }
        let Some(rate) = rows
            .iter()
            .find(|r| r.tax_year == FLOOR_TAX_YEAR)
            .and_then(|r| r.class1_rate)
        else {
            continue;
        };

        out.push(District {
            irn,
            county,
            event_tax_year: cycle.tax_year,
            at_the_floor: rate <= floor + PUBLISHED_HALF_UNIT,
            local_per_pupil,
            class1_value,
        });
    }
    out.sort_by(|a, b| a.local_per_pupil[&WINDOW[0]].total_cmp(&b.local_per_pupil[&WINDOW[0]]));
    out
}

/// One quartile of the wealth distribution, and how much of a reappraisal reaches it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quartile {
    /// Districts in it.
    pub districts: usize,
    /// The share sitting at the twenty-mill floor, where valuation growth reaches revenue.
    pub at_the_floor: f64,
    /// Median local revenue per pupil in the first year of [`WINDOW`].
    pub local_per_pupil: f64,
    /// Median growth in local revenue per pupil across [`WINDOW`], as a share.
    pub local_growth: f64,
}

/// The four wealth quartiles, poorest first.
///
/// The finding is the first column read against the last: the poorest quartile is the one at the
/// floor and the one whose local revenue grew, and the richest is neither.
#[must_use]
pub fn by_wealth() -> [Quartile; 4] {
    let all = districts();
    let size = all.len() / 4;
    std::array::from_fn(|i| {
        let slice = if i < 3 {
            &all[i * size..(i + 1) * size]
        } else {
            &all[3 * size..]
        };
        Quartile {
            districts: slice.len(),
            at_the_floor: slice.iter().filter(|d| d.at_the_floor).count() as f64
                / slice.len() as f64,
            local_per_pupil: median(slice.iter().map(|d| d.local_per_pupil[&WINDOW[0]])),
            local_growth: median(
                slice
                    .iter()
                    .map(|d| d.local_per_pupil[&WINDOW[1]] / d.local_per_pupil[&WINDOW[0]] - 1.0),
            ),
        }
    })
}

/// The quartile gap in one fiscal year, as observed and as it would have been with no reappraisal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Counterfactual {
    /// Richest quartile's mean local revenue per pupil less the poorest quartile's.
    pub observed: f64,
    /// The same, with every district on its own quiet-year growth rate throughout.
    pub without_the_event: f64,
}

impl Counterfactual {
    /// What the reappraisals added to the gap. Negative means they held it down.
    #[must_use]
    pub fn attributable(&self) -> f64 {
        self.observed - self.without_the_event
    }
}

/// The gap with and without the reappraisals, for one fiscal year inside [`WINDOW`].
///
/// Quartiles are cut on the year being measured, as
/// [`dispersion::ohio_panel::equalization_by_year`] cuts them, so the two are comparable.
#[must_use]
pub fn gap(fiscal_year: u16) -> Counterfactual {
    let all = districts();
    let observed = quartile_gap(&all, |d| d.local_per_pupil[&fiscal_year]);
    let without_the_event = quartile_gap(&all, |d| d.counterfactual(fiscal_year));
    Counterfactual {
        observed,
        without_the_event,
    }
}

fn quartile_gap(all: &[District], of: impl Fn(&District) -> f64) -> f64 {
    let mut values: Vec<f64> = all.iter().map(of).collect();
    values.sort_by(f64::total_cmp);
    let size = values.len() / 4;
    let mean = |slice: &[f64]| slice.iter().sum::<f64>() / slice.len() as f64;
    mean(&values[3 * size..]) - mean(&values[..size])
}

fn median(values: impl Iterator<Item = f64>) -> f64 {
    let mut sample: Vec<f64> = values.collect();
    sample.sort_by(f64::total_cmp);
    sample.get(sample.len() / 2).copied().unwrap_or(f64::NAN)
}
