//! What entered the general fund that was not the general fund's own revenue.
//!
//! # The question, and the line that answers it
//!
//! `metric/general-fund-cash-balance` records that statewide ending cash rose every year from
//! $8.37bn in FY2020 to $11.20bn in FY2024 and then fell, and asks, open: *"How much of the
//! FY2021-FY2024 build-up was ESSER booked in the general fund, which the forecast lines cannot
//! separate and the federal award data could."*
//!
//! The forecast lines can separate it, and one of them had never been read. [`crate::finances`]
//! carries two revenue totals: `total_revenue`, which is the district's own receipts and
//! deliberately excludes transfers, and `total_revenue_and_sources`, which is every dollar that
//! entered the fund and is the one that closes the cash identity. **Their difference is transfers,
//! advances and note proceeds** — the only route by which money a district received into a special
//! revenue fund can reach its general fund.
//!
//! Nothing in this workspace read the second column. Its extractor writes it and every consumer
//! took the first.
//!
//! # What it shows
//!
//! Statewide the line more than doubles across the relief years and falls back when the grants
//! end: $0.282bn in FY2020, $0.362bn, $0.396bn, $0.533bn, $0.720bn through FY2024, then $0.434bn
//! in FY2025. That is the right shape for relief money moving into the general fund, and
//! [`by_year`] is it.
//!
//! # And what it does not show
//!
//! The shape is not the same thing as the size, and the size is where the node's question lives.
//!
//! **The build-up does not track a district's relief.** Correlation between ESSER surge per pupil
//! — [`crate::esser::District::surge_per_pupil`] — and the FY2021-FY2024 cash build-up per pupil
//! is **0.075** across 606 districts. If the balances were relief, the districts that received
//! more of it would hold more of them. They do not, and that is the whole answer at the level the
//! question was asked.
//!
//! **The attributable share depends on a counterfactual the corpus cannot pin.** How much of the
//! excess transfer is *relief* rather than ordinary growth needs a level the line would have run
//! at without the grants, and the panel offers two candidates, neither clean:
//!
//! | baseline | excess FY2021-FY2024 | share of the build-up | relief-attributable |
//! |---|---:|---:|---:|
//! | FY2020 | $0.860bn | 46.9% | 15.1% - 39.4% |
//! | mean of FY2020 and FY2025 | $0.574bn | 31.3% | 6.4% - 8.8% |
//!
//! FY2020 already carries ESSER I, which arrived in the spring of 2020, so a baseline taken there
//! is contaminated in the direction of understating the excess. FY2025 is not a clean post-period
//! either: ESSER III's obligation deadline fell in September 2024 and liquidation ran into 2025,
//! so subtracting it over-corrects. The truth is between two baselines this corpus cannot choose
//! between, and the range is stated rather than a midpoint pinned.
//!
//! The two slopes inside each range are the same disagreement in miniature: a least-squares fit on
//! 606 districts is pulled by the large ones, and the difference between the outer quartile
//! medians is not. Both are reported by [`attribution`].
//!
//! # What is established
//!
//! Under **every** combination above, relief accounts for **less than two fifths of the build-up
//! and most likely under a tenth**. Read with [`crate::esser::federal_against_the_general_fund`],
//! which shows federal revenue rising 139% across the same years while general-fund revenue fell,
//! the account is consistent: ESSER did not enter the general fund as revenue, a little of it
//! entered as transfers, and the balances that accumulated were mostly something else.

use std::collections::BTreeMap;

use edfund_core::FiscalYear;

/// The fiscal years the build-up is measured across, as the node states it.
pub const BUILD_UP: [u16; 2] = [2021, 2024];

/// The years outside the relief window that the transfers line can be levelled against.
///
/// Neither is clean. FY2020 carries ESSER I; FY2025 carries ESSER III's liquidation.
pub const BASELINE_YEARS: [u16; 2] = [2020, 2025];

/// One fiscal year of the general fund, in the three totals that separate its own money from
/// everything else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Year {
    /// Filing bodies reporting all three lines.
    pub districts: usize,
    /// The district's own receipts, in dollars.
    pub revenue: f64,
    /// Every dollar that entered the fund.
    pub revenue_and_sources: f64,
    /// Ending cash at 30 June.
    pub ending_cash: f64,
}

impl Year {
    /// Transfers, advances and note proceeds — what entered that was not the fund's own revenue.
    #[must_use]
    pub fn transfers(&self) -> f64 {
        self.revenue_and_sources - self.revenue
    }
}

/// The three totals for every year the filings cover.
#[must_use]
pub fn by_year() -> BTreeMap<u16, Year> {
    let mut out: BTreeMap<u16, Year> = BTreeMap::new();
    for district in crate::finances::finances() {
        for record in &district.years {
            let (Some(revenue), Some(sources)) =
                (record.total_revenue, record.total_revenue_and_sources)
            else {
                continue;
            };
            let year = out.entry(record.fiscal_year.0).or_insert(Year {
                districts: 0,
                revenue: 0.0,
                revenue_and_sources: 0.0,
                ending_cash: 0.0,
            });
            year.districts += 1;
            year.revenue += revenue;
            year.revenue_and_sources += sources;
            year.ending_cash += record.ending_cash.unwrap_or(0.0);
        }
    }
    out
}

/// One district's relief, what it moved into its general fund, and what its balance did.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Ohio's identifier.
    pub irn: String,
    /// Federal revenue per pupil at the relief peak over the pre-pandemic year.
    pub relief_per_pupil: f64,
    /// Transfers into the general fund across [`BUILD_UP`] above four times the earlier baseline
    /// year's, per pupil.
    pub excess_transfers_per_pupil: f64,
    /// The same against the mean of both [`BASELINE_YEARS`].
    pub excess_against_both_per_pupil: f64,
    /// Change in ending cash across [`BUILD_UP`], per pupil.
    pub build_up_per_pupil: f64,
}

/// Every district the filings, the survey panel and the relief measure all reach.
///
/// Sorted by relief per pupil, so a quartile cut below is a cut on exposure.
#[must_use]
pub fn districts() -> Vec<District> {
    let relief: BTreeMap<String, f64> = crate::esser::districts()
        .into_iter()
        .map(|d| (d.irn, d.surge_per_pupil))
        .collect();

    let mut pupils: BTreeMap<String, f64> = BTreeMap::new();
    for row in dispersion::ohio_panel::panel() {
        if row.fiscal_year == 2022 && row.enrollment > 0.0 {
            pupils.insert(row.irn.clone(), row.enrollment);
        }
    }

    let mut out = Vec::new();
    for district in crate::finances::finances() {
        let (Some(relief_per_pupil), Some(enrolment)) =
            (relief.get(&district.irn), pupils.get(&district.irn))
        else {
            continue;
        };
        let transfers = |fiscal_year: u16| -> Option<f64> {
            let year = district.year(FiscalYear(fiscal_year))?;
            Some(year.total_revenue_and_sources? - year.total_revenue?)
        };
        let cash = |fiscal_year: u16| -> Option<f64> {
            district.year(FiscalYear(fiscal_year))?.ending_cash
        };
        let (Some(before), Some(after), Some(opening), Some(closing)) = (
            transfers(BASELINE_YEARS[0]),
            transfers(BASELINE_YEARS[1]),
            cash(BUILD_UP[0]),
            cash(BUILD_UP[1]),
        ) else {
            continue;
        };
        let Some(window) = (BUILD_UP[0]..=BUILD_UP[1])
            .map(transfers)
            .sum::<Option<f64>>()
        else {
            continue;
        };
        let years = f64::from(BUILD_UP[1] - BUILD_UP[0] + 1);

        out.push(District {
            irn: district.irn.clone(),
            relief_per_pupil: *relief_per_pupil,
            excess_transfers_per_pupil: (window - years * before) / enrolment,
            excess_against_both_per_pupil: (window - years * (before + after) / 2.0) / enrolment,
            build_up_per_pupil: (closing - opening) / enrolment,
        });
    }
    out.sort_by(|a, b| a.relief_per_pupil.total_cmp(&b.relief_per_pupil));
    out
}

/// How closely the cash build-up tracks a district's relief.
///
/// **This is the answer at the level the node asked.** It is 0.075: nothing.
#[must_use]
pub fn build_up_against_relief() -> f64 {
    let all = districts();
    correlation(
        &all.iter().map(|d| d.relief_per_pupil).collect::<Vec<_>>(),
        &all.iter().map(|d| d.build_up_per_pupil).collect::<Vec<_>>(),
    )
}

/// How much of the build-up the relief can be held responsible for, under one baseline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attribution {
    /// Excess transfers over the baseline, summed across the districts, in dollars.
    pub excess: f64,
    /// The build-up over the same districts, in dollars.
    pub build_up: f64,
    /// The relief those districts received at the peak, in dollars.
    pub relief: f64,
    /// Extra transfer per dollar of relief per pupil, by least squares over every district.
    pub slope: f64,
    /// The same, read off the difference between the outer quartile medians — which a handful of
    /// large districts cannot pull.
    pub robust_slope: f64,
}

impl Attribution {
    /// Excess transfers as a share of the build-up. The ceiling, since not all of it is relief.
    #[must_use]
    pub fn excess_share(&self) -> f64 {
        self.excess / self.build_up
    }

    /// The share of the build-up attributable to relief, under each slope: robust first.
    #[must_use]
    pub fn attributable(&self) -> (f64, f64) {
        (
            self.robust_slope * self.relief / self.build_up,
            self.slope * self.relief / self.build_up,
        )
    }
}

/// The attribution under a chosen baseline.
///
/// `against_both` takes the mean of [`BASELINE_YEARS`] rather than the earlier one alone. Both are
/// reported because the corpus cannot choose between them; see the module documentation.
#[must_use]
pub fn attribution(against_both: bool) -> Attribution {
    let all = districts();
    let excess_of = |d: &District| {
        if against_both {
            d.excess_against_both_per_pupil
        } else {
            d.excess_transfers_per_pupil
        }
    };

    let mut pupils: BTreeMap<String, f64> = BTreeMap::new();
    for row in dispersion::ohio_panel::panel() {
        if row.fiscal_year == 2022 && row.enrollment > 0.0 {
            pupils.insert(row.irn.clone(), row.enrollment);
        }
    }
    let of = |d: &District| pupils.get(&d.irn).copied().unwrap_or(0.0);

    let excess: f64 = all.iter().map(|d| excess_of(d) * of(d)).sum();
    let build_up: f64 = all.iter().map(|d| d.build_up_per_pupil * of(d)).sum();
    let relief: f64 = all.iter().map(|d| d.relief_per_pupil * of(d)).sum();

    let xs: Vec<f64> = all.iter().map(|d| d.relief_per_pupil).collect();
    let ys: Vec<f64> = all.iter().map(excess_of).collect();
    let n = xs.len() as f64;
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let slope = xs
        .iter()
        .zip(&ys)
        .map(|(x, y)| (x - mx) * (y - my))
        .sum::<f64>()
        / xs.iter().map(|x| (x - mx).powi(2)).sum::<f64>();

    // The outer quartiles only — this slope is the line between their medians, and the two middle
    // ones are not on it.
    let size = all.len() / 4;
    let (poorest, richest) = (&all[..size], &all[3 * size..]);
    let robust_slope = (median(richest.iter().map(excess_of))
        - median(poorest.iter().map(excess_of)))
        / (median(richest.iter().map(|d| d.relief_per_pupil))
            - median(poorest.iter().map(|d| d.relief_per_pupil)));

    Attribution {
        excess,
        build_up,
        relief,
        slope,
        robust_slope,
    }
}

fn median(values: impl Iterator<Item = f64>) -> f64 {
    let mut sample: Vec<f64> = values.collect();
    sample.sort_by(f64::total_cmp);
    sample.get(sample.len() / 2).copied().unwrap_or(f64::NAN)
}

fn correlation(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n < 2.0 {
        return f64::NAN;
    }
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let cov: f64 = xs.iter().zip(ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let sx: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum::<f64>().sqrt();
    let sy: f64 = ys.iter().map(|y| (y - my).powi(2)).sum::<f64>().sqrt();
    cov / (sx * sy)
}
