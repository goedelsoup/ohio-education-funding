//! Ohio's school construction program, measured from the side that receives it.
//!
//! # The question, and why the corpus thought it could not be answered
//!
//! `program/classroom-facilities-assistance` records the amount as "Not yet populated", the
//! eligibility rule's share schedule as "not yet established", and gives one reason for both: the
//! program's appropriation "is in the capital appropriations act, which no connector retrieves".
//! That is true about the paying side. It is not true about the receiving side, which this
//! repository has held for sixteen years without reading.
//!
//! The survey's state revenue column is a sum of fourteen items and the panel kept only the sum.
//! One of the fourteen is `C35`, which the survey calls **nonspecified** — "state revenue amounts
//! that pertain to more than one of the above categories, but for which reporting units could not
//! provide distinct amounts by category". In Ohio that is where the construction money is.
//!
//! # The identification, which is empirical and has to be
//!
//! `C35` is a catch-all by definition, so it is not a facilities line and nothing here treats it
//! as one. Four things make it the construction money all the same, and no other item in the
//! survey's state block does any of them:
//!
//! - It correlates **0.47 to 0.72 with capital outlay per pupil in every one of the fifteen years
//!   the panel holds**, while general formula assistance never clears 0.12 and is negative in
//!   ten of them. The two years published by the Bureau rather than by NCES behave like the
//!   thirteen that are not, so this is not an artifact of one publisher's recoding. See
//!   [`against_capital`] and [`formula_against_capital`].
//! - **95.9% of it lands in district-years with real capital spending.** Only $518m of $12.7bn
//!   arrives where a district spent under $200 per pupil on capital.
//! - Within a district, the year its `C35` peaks is a year it was building — for **82%** of the
//!   districts whose peak is worth reading, capital outlay that year beats their own median across
//!   the panel. Holding the district fixed rules out size and wealth. See [`peaks_are_build_years`].
//! - The share of a district's capital outlay it covers **falls monotonically with district
//!   wealth**, from 57.6% for the poorest fifth to 19.0% for the wealthiest, at `r = −0.44`. That
//!   is the program's own rule, and it is the rule the corpus had marked verified against a
//!   description of the statute rather than against a measurement. See [`by_wealth`].
//!
//! # What the series says
//!
//! **$12.7 billion reached Ohio's school districts between FY2009 and FY2024**, against $33.3
//! billion of capital spending — the state paid **38.0%** of what Ohio's districts spent on
//! buildings across sixteen years.
//!
//! **Every one of the 606 districts received some of it**, the smallest fifteen-year total being
//! $730,000. That is worth stating because the program is described everywhere, including in this
//! corpus, as a queue ordered by poverty: it is, and it still reached the whole state. What
//! distinguishes a poor district from a wealthy one here is not whether the state paid but how
//! much of the bill it took — three times as much of it.
//!
//! And the program was already past its peak when the panel opens. Districts received $1.357bn in
//! FY2009 and $493m in FY2013, a fall of nearly two thirds in four years, and the series has never
//! returned to its opening level in nominal dollars. The clearest structural response to *DeRolph*
//! is the one channel of Ohio school funding this corpus had no series for, and it had been
//! shrinking for the whole time the corpus can see it.
//!
//! # Three limits, each of which would produce a wrong reading
//!
//! **The share here is of capital outlay, not of assessed project cost.** The program sets a state
//! share of an approved project's cost; this divides state money by *all* the capital a district
//! spent that year, including locally funded work outside any project. The two are not the same
//! number and the one here is the lower of them. The gradient is the finding; the level is a bound.
//!
//! **Cash flows are not project shares.** A project's state and local money arrive in different
//! years, which is why [`by_wealth`] sums the whole panel per district before dividing rather than
//! averaging annual ratios — doing it the other way flattens the gradient by a third.
//!
//! **`C35` is not only facilities.** The 4% that lands where nothing was built is whatever else
//! Ohio could not split across the survey's categories, and there is no rule here that separates
//! it. A district-year figure from this module is the state's nonspecified revenue, read as
//! construction money because of the four facts above and not because the survey says so.

use std::collections::BTreeMap;

use crate::ohio_panel::{self, PanelRow};

/// The span every figure here is computed over, inclusive — which is the whole panel.
///
/// FY2014 is absent from the archive for every agency, so it is fifteen years across a sixteen-
/// year span. The last two are the Bureau's own file rather than NCES's and carry the same
/// columns; keeping them in is what shows the identification does not depend on one publisher.
pub const IDENTIFIED: [u16; 2] = [2009, 2024];

/// A district-year is a build year when its capital outlay clears this, per pupil.
///
/// Not a statutory threshold — a reading threshold. Ohio's districts spend a few hundred dollars
/// per pupil on capital in an ordinary year and several thousand in a project year, and this sits
/// in the gap.
pub const BUILD_YEAR: f64 = 1_000.0;

/// One district's whole run: what the state paid toward capital, and what was spent.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Information Retrieval Number.
    pub irn: String,
    /// State nonspecified revenue summed across the panel, in dollars.
    pub received: f64,
    /// Capital outlay summed across the same years, from every source.
    pub spent: f64,
    /// Fall membership in the last year the district reports.
    pub pupils: f64,
    /// Assessed value per pupil, from the profile report.
    pub valuation_per_pupil: f64,
    /// How many years of the panel both columns are reported for.
    pub years: usize,
}

impl District {
    /// The state's share of this district's capital spending across the panel.
    ///
    /// Capped at one: a district can receive state money in a year it spends nothing, and the
    /// ratio then says more about cash flow than about a share.
    #[must_use]
    pub fn state_share(&self) -> f64 {
        if self.spent <= 0.0 {
            return 0.0;
        }
        (self.received / self.spent).min(1.0)
    }
}

/// Every district the panel and the profile report can both name, over [`IDENTIFIED`].
#[must_use]
pub fn frame() -> Vec<District> {
    let profiles: BTreeMap<String, f64> = crate::profile::districts()
        .into_iter()
        .filter_map(|d| Some((d.irn, d.valuation_per_pupil?)))
        .collect();

    let mut totals: BTreeMap<String, (f64, f64, f64, usize)> = BTreeMap::new();
    for row in ohio_panel::panel().iter().filter(|r| in_window(r)) {
        let (Some(received), Some(spent)) = (row.state_nonspecified, row.capital_outlay) else {
            continue;
        };
        let entry = totals.entry(row.irn.clone()).or_default();
        entry.0 += received;
        entry.1 += spent;
        entry.2 = row.enrollment;
        entry.3 += 1;
    }

    totals
        .into_iter()
        .filter_map(|(irn, (received, spent, pupils, years))| {
            let valuation_per_pupil = *profiles.get(&irn)?;
            (valuation_per_pupil > 0.0).then_some(District {
                irn,
                received,
                spent,
                pupils,
                valuation_per_pupil,
                years,
            })
        })
        .collect()
}

/// Whether a row is a comparable district inside the identified window.
fn in_window(row: &PanelRow) -> bool {
    row.comparable
        && row.enrollment > 0.0
        && (IDENTIFIED[0]..=IDENTIFIED[1]).contains(&row.fiscal_year)
}

/// What the state paid districts toward capital in each year, and what they spent, in dollars.
#[must_use]
pub fn by_year() -> BTreeMap<u16, (f64, f64)> {
    let mut out: BTreeMap<u16, (f64, f64)> = BTreeMap::new();
    for row in ohio_panel::panel().iter().filter(|r| in_window(r)) {
        if let (Some(received), Some(spent)) = (row.state_nonspecified, row.capital_outlay) {
            let entry = out.entry(row.fiscal_year).or_default();
            entry.0 += received;
            entry.1 += spent;
        }
    }
    out
}

/// The correlation between state nonspecified revenue per pupil and capital outlay per pupil.
///
/// One reading per year, in order. The identification rests on this being positive and large in
/// every year rather than on any single one of them.
#[must_use]
pub fn against_capital() -> Vec<(u16, f64)> {
    correlate(|row| row.state_nonspecified)
}

/// The same for general formula assistance, which is the control.
///
/// If `C35` tracked capital because *all* state money does, this would track it too. It does not.
#[must_use]
pub fn formula_against_capital() -> Vec<(u16, f64)> {
    correlate(|row| row.general_formula_assistance)
}

/// One state item against capital outlay, per pupil, year by year.
fn correlate(pick: fn(&PanelRow) -> Option<f64>) -> Vec<(u16, f64)> {
    let mut by_year: BTreeMap<u16, (Vec<f64>, Vec<f64>)> = BTreeMap::new();
    for row in ohio_panel::panel().iter().filter(|r| in_window(r)) {
        if let (Some(item), Some(spent)) = (pick(row), row.capital_outlay) {
            let entry = by_year.entry(row.fiscal_year).or_default();
            entry.0.push(item / row.enrollment);
            entry.1.push(spent / row.enrollment);
        }
    }
    by_year
        .into_iter()
        .filter_map(|(year, (item, capital))| {
            let fit = crate::wealth_neutrality(&item, &capital).ok()?;
            Some((year, fit.correlation))
        })
        .collect()
}

/// Mean state share of capital spending within each fifth of districts by wealth, poorest first.
///
/// Summed across the panel per district and then divided, because a project's state and local
/// money do not arrive in the same year. Each fifth's figure is its districts' total state money
/// over their total capital spending, so a large district counts for what it spent.
///
/// # Panics
///
/// If the frame holds fewer than five districts, which means the profile join broke.
#[must_use]
pub fn by_wealth() -> [f64; 5] {
    let mut districts = frame();
    assert!(districts.len() >= 5, "the frame is too small to quintile");
    districts.sort_by(|a, b| a.valuation_per_pupil.total_cmp(&b.valuation_per_pupil));

    let size = districts.len() / 5;
    let mut out = [0.0; 5];
    for (index, slot) in out.iter_mut().enumerate() {
        let end = if index == 4 {
            districts.len()
        } else {
            (index + 1) * size
        };
        let slice = &districts[index * size..end];
        let received: f64 = slice.iter().map(|d| d.received).sum();
        let spent: f64 = slice.iter().map(|d| d.spent).sum();
        *slot = if spent > 0.0 { received / spent } else { 0.0 };
    }
    out
}

/// The same quintiles computed the other way, as the mean of each district-year's own ratio.
///
/// Kept so the choice [`by_wealth`] makes is checkable rather than asserted. A project's state and
/// local money do not arrive in the same year, so an annual ratio is a cash-flow figure; averaging
/// them flattens the wealth gradient this program is defined by.
///
/// # Panics
///
/// If the frame holds fewer than five districts.
#[must_use]
pub fn by_wealth_annual() -> [f64; 5] {
    let mut wealth: BTreeMap<String, f64> = BTreeMap::new();
    for district in frame() {
        wealth.insert(district.irn.clone(), district.valuation_per_pupil);
    }
    let mut ratios: Vec<(f64, f64)> = Vec::new();
    for row in ohio_panel::panel().iter().filter(|r| in_window(r)) {
        let (Some(received), Some(spent), Some(value)) = (
            row.state_nonspecified,
            row.capital_outlay,
            wealth.get(&row.irn).copied(),
        ) else {
            continue;
        };
        if spent > 0.0 {
            ratios.push((value, (received / spent).min(1.0)));
        }
    }
    assert!(ratios.len() >= 5, "the frame is too small to quintile");
    ratios.sort_by(|a, b| a.0.total_cmp(&b.0));

    let size = ratios.len() / 5;
    let mut out = [0.0; 5];
    for (index, slot) in out.iter_mut().enumerate() {
        let end = if index == 4 {
            ratios.len()
        } else {
            (index + 1) * size
        };
        let slice = &ratios[index * size..end];
        #[allow(clippy::cast_precision_loss)]
        let n = slice.len() as f64;
        *slot = slice.iter().map(|(_, share)| share).sum::<f64>() / n;
    }
    out
}

/// The correlation between log wealth per pupil and a district's fifteen-year state share.
///
/// # Panics
///
/// If the frame is empty or has no variation in wealth, both of which mean the fixtures moved.
#[must_use]
pub fn share_against_wealth() -> f64 {
    let districts = frame();
    let wealth: Vec<f64> = districts
        .iter()
        .map(|d| d.valuation_per_pupil.ln())
        .collect();
    let share: Vec<f64> = districts.iter().map(District::state_share).collect();
    crate::wealth_neutrality(&wealth, &share)
        .expect("the frame pairs")
        .correlation
}

/// How much of the state's nonspecified revenue lands where nothing was built.
///
/// Returned as `(stray, total)` in dollars, where stray is the part in district-years whose
/// capital outlay is under a fifth of [`BUILD_YEAR`] per pupil. It is the bound on reading this
/// column as construction money, and it is 4.5%.
#[must_use]
pub fn outside_build_years() -> (f64, f64) {
    let mut stray = 0.0;
    let mut total = 0.0;
    for row in ohio_panel::panel().iter().filter(|r| in_window(r)) {
        let (Some(received), Some(spent)) = (row.state_nonspecified, row.capital_outlay) else {
            continue;
        };
        total += received;
        if spent / row.enrollment < BUILD_YEAR / 5.0 {
            stray += received;
        }
    }
    (stray, total)
}

/// Districts whose largest year of state capital money is a year they were building.
///
/// Returned as `(matched, examined)` over districts whose peak year clears $500 per pupil — below
/// that the "peak" is noise. The within-district version of [`against_capital`]: it holds the
/// district fixed, so it cannot be a between-district artifact of size or wealth.
#[must_use]
pub fn peaks_are_build_years() -> (usize, usize) {
    let mut by_district: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for row in ohio_panel::panel().iter().filter(|r| in_window(r)) {
        if let (Some(received), Some(spent)) = (row.state_nonspecified, row.capital_outlay) {
            by_district
                .entry(row.irn.clone())
                .or_default()
                .push((received / row.enrollment, spent / row.enrollment));
        }
    }

    let (mut matched, mut examined) = (0, 0);
    for years in by_district.values() {
        if years.len() < 8 {
            continue;
        }
        let peak = years
            .iter()
            .enumerate()
            .max_by(|a, b| a.1 .0.total_cmp(&b.1 .0))
            .expect("a non-empty run");
        if peak.1 .0 < 500.0 {
            continue;
        }
        let mut others: Vec<f64> = years
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != peak.0)
            .map(|(_, year)| year.1)
            .collect();
        others.sort_by(f64::total_cmp);
        let median = others[others.len() / 2];
        examined += 1;
        if peak.1 .1 > median {
            matched += 1;
        }
    }
    (matched, examined)
}
