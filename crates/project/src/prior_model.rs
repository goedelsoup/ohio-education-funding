//! The department's FY2026 model beside its FY2027 one, which is the only interval the Fair
//! School Funding Plan can be observed over.
//!
//! # The two clocks
//!
//! The plan sets a district's state aid as the difference between what its education costs and
//! what it can raise locally, and it reads those two things off **different calendars**.
//!
//! The cost side is frozen. H.B. 96 "retains the use of FY 2022 data for the various statewide
//! average salary and compensation amounts" and maintains the FY2024 statewide averages through
//! FY2027 — see [`crate::base_cost`]. Across these two years the median district's base cost per
//! pupil moves **+0.05%**.
//!
//! The capacity side is not frozen. `[V2]`, `[V3]` and `[V4]` are the three most recent tax
//! years' total valuation and the window rolls forward every year: FY2026's `Local_Capacity`
//! sheet heads them TY2024, TY2023 and TY2022 and FY2027's heads them TY2025, TY2024 and TY2023.
//! Across the same two years the median district's per-pupil local capacity moves **+9.34%**.
//!
//! The valuation itself is not compared here. The panel's `valuation_per_pupil` is the District
//! Profile Report's FY2023 figure and the calculator's `[C1]` is the three-year lesser-of rule
//! over a different denominator; pairing them reads +2.39% and means nothing. [`Quantity`] holds
//! only quantities both models publish under the same definition, which is the point of it.
//!
//! State share is `(base cost - capacity) / base cost`, so a frozen numerator against a rolling
//! subtrahend can only fall. It falls in **540 of 609 districts**, by a median **4.11 percentage
//! points**, and the state's share of the aggregate base cost goes **35.35% to 31.52%** in one
//! year. Nobody voted on that. See [`state_share_fall`] and [`holding_capacity_still`].
//!
//! # What this closes
//!
//! Two nodes recorded a question as unanswerable "because the corpus holds one year of the
//! calculator":
//!
//! - `formula-component/fsfp-local-capacity-measure` — "how far the value has moved between
//!   biennia is still unmeasured here … but it is now a question about a distribution rather than
//!   about a decision". The decision is stable and the distribution is not: the benchmark ratio
//!   `[C5]` moves 1.46670942 to 1.46503636, **-0.11%**, while what it multiplies moves +9.34%.
//! - `formula-component/fsfp-disadvantaged-pupil-impact-aid` — "How far it has moved between
//!   biennia is still unrecorded here". The statewide economically disadvantaged percentage moves
//!   **0.565990 to 0.533380**, and the blend weights moved with it: **75/25 to 65/35**.
//!
//! The cache held both workbooks the whole time. What was missing was the extraction, and the
//! catalog entry that recorded the decision not to take it argued from the wrong sheet — see
//! [`connect::fixtures::fy26`](../../../connect/src/fixtures/fy26.rs).

use std::collections::BTreeMap;

use edfund_core::{Adm, Dollars};

use crate::panel::{self, DistrictRecord, MINIMUM_STATE_SHARE};

/// The committed extract.
const FIXTURE: &str = include_str!("../fixtures/fy26-department-model.csv");

/// The header this reader was written against.
const EXPECTED_HEADER: &str = "irn,district,enrolled_adm,base_cost_enrolled_adm,\
                               aggregate_base_cost,base_cost_per_pupil,capacity_per_pupil,\
                               state_share_percentage,valuation_per_pupil,\
                               local_capacity_percentage,benchmark_ratio,\
                               dpia_econ_disadvantaged_adm,dpia_directly_certified_adm,\
                               dpia_weighted_adm,dpia_percentage,dpia_aid";

/// The disadvantaged pupil blend weights the FY2026 model uses.
///
/// `(economically disadvantaged, directly certified)`, against
/// [`panel::categoricals::DPIA_BLEND`]'s `(0.65, 0.35)` for FY2027. The identity
/// `d1 = 0.75 * d1a + 0.25 * d1b` holds to the last published digit on all 610 rows of the
/// FY2026 sheet, the statewide total included.
pub const DPIA_BLEND: (f64, f64) = (0.75, 0.25);

/// The statewide economically disadvantaged percentage the FY2026 model's `d3` indexes against.
pub const DPIA_STATEWIDE_PERCENTAGE: f64 = 0.565_990_245;

/// The ratio `[C5]` of the fortieth-highest district, which caps the local capacity percentage.
pub const BENCHMARK_RATIO: f64 = 1.466_709_42;

/// The statewide scalars each year's calculator states once and multiplies everywhere.
const SCALARS: &str = include_str!("../fixtures/calculator-parameters.csv");

/// The enacted FY2026 and FY2027 appropriation for preschool special education.
///
/// The remainder of GRF ALI 200540 after its five other earmarks, identical in both years and
/// established from LSC's enacted analysis by
/// `parameter/appropriation-proration-factor`. Repeated here as a constant because what it is
/// wanted for is checking the department's proration factor against it, and a check whose
/// expected value lives in prose is not a check.
pub const PRESCHOOL_APPROPRIATION: Dollars = 153_976_832.0;

/// Every statewide scalar both calculators state, by fiscal year and then by column name.
///
/// A map rather than a struct of thirty-odd fields because what a `parameter` node wants is one
/// row of it across every year — [`scalar`] — and because a column added to the fixture should
/// reach a caller without a type changing shape.
///
/// # Panics
///
/// If the fixture's first line is not a header, or a row's width differs from it.
#[must_use]
pub fn scalars() -> BTreeMap<u16, BTreeMap<String, f64>> {
    let mut lines = SCALARS.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split(',').collect();
    let mut out: BTreeMap<u16, BTreeMap<String, f64>> = BTreeMap::new();
    for line in lines {
        let cells: Vec<&str> = line.split(',').collect();
        assert_eq!(
            cells.len(),
            header.len(),
            "calculator-parameters.csv row is {} wide against a {}-wide header",
            cells.len(),
            header.len()
        );
        let Ok(fiscal_year) = cells[0].parse::<u16>() else {
            continue;
        };
        let row = out.entry(fiscal_year).or_default();
        for (name, cell) in header.iter().zip(&cells).skip(1) {
            if let Ok(value) = cell.trim().parse::<f64>() {
                row.insert((*name).to_string(), value);
            }
        }
    }
    out
}

/// One scalar across every year the calculators cover, oldest first.
///
/// The series a `parameter` node's `series:` field should be read off. `name` is a column of
/// `crates/project/fixtures/calculator-parameters.csv`.
#[must_use]
pub fn scalar(name: &str) -> BTreeMap<u16, f64> {
    scalars()
        .into_iter()
        .filter_map(|(fiscal_year, row)| Some((fiscal_year, *row.get(name)?)))
        .collect()
}

/// The scalars that differ between the two years, and the ones that hold, by name.
///
/// Returned as `(moved, held)`. The second list is the longer one and is the finding: every
/// weight and every base cost in the plan is the same number in both years.
#[must_use]
pub fn scalars_that_moved() -> (Vec<String>, Vec<String>) {
    let by_year = scalars();
    let mut moved = Vec::new();
    let mut held = Vec::new();
    let Some((first, rest)) = by_year.values().next().zip(by_year.values().last()) else {
        return (moved, held);
    };
    for (name, value) in first {
        match rest.get(name) {
            Some(later) if (later - value).abs() < 1e-9 => held.push(name.clone()),
            Some(_) => moved.push(name.clone()),
            None => {}
        }
    }
    (moved, held)
}

/// Whether a year's preschool proration factor is that year's appropriation over its demand.
///
/// Returned as `(the factor the workbook states, the factor its own year's arithmetic gives)`.
/// The second is [`PRESCHOOL_APPROPRIATION`] divided by the demand the workbook implies — its
/// published statewide total divided by the factor it applied.
///
/// **FY2026 closes and FY2027 does not.** In FY2026 the stated factor and the computed one agree
/// to eight decimal places, which settles what
/// `parameter/appropriation-proration-factor` records as open: the factor is *measured* — the
/// line divided by the demand — and nobody decides it. In FY2027 the program's demand is 0.5%
/// **below** its appropriation, so its own arithmetic gives no proration at all, and the
/// workbook applies one anyway.
#[must_use]
pub fn preschool_proration(fiscal_year: u16) -> Option<(f64, f64)> {
    let row = scalars().remove(&fiscal_year)?;
    let stated = *row.get("preschool_proration_factor")?;
    let total = *row.get("preschool_total_funds")?;
    if stated <= 0.0 || total <= 0.0 {
        return None;
    }
    Some((stated, PRESCHOOL_APPROPRIATION / (total / stated)))
}

/// What the preschool program is paid against what it is appropriated, for one year.
///
/// Returned as `(paid, appropriated)`. Negative headroom is a shortfall the proration exists to
/// prevent; positive headroom means no proration should arise.
#[must_use]
pub fn preschool_headroom(fiscal_year: u16) -> Option<(Dollars, Dollars)> {
    let row = scalars().remove(&fiscal_year)?;
    Some((*row.get("preschool_total_funds")?, PRESCHOOL_APPROPRIATION))
}

/// One district, as the FY2026 model computes it.
#[derive(Debug, Clone, PartialEq)]
pub struct Prior {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's name in the FY2026 model.
    pub name: String,
    /// `[a]` enrolled ADM.
    pub enrolled_adm: Adm,
    /// `[b]` base cost enrolled ADM.
    pub base_cost_enrolled_adm: Adm,
    /// `[F]` aggregate base cost.
    pub aggregate_base_cost: Dollars,
    /// `[b2]` district base cost per pupil.
    pub base_cost_per_pupil: Dollars,
    /// `[b1]` per-pupil local capacity amount.
    pub capacity_per_pupil: Dollars,
    /// `[b4]` state share percentage.
    pub state_share_percentage: f64,
    /// `[C1]` assessed valuation per pupil.
    pub valuation_per_pupil: Dollars,
    /// `[C6]` local capacity percentage.
    pub local_capacity_percentage: f64,
    /// `[C5]` the fortieth-highest district's income ratio, identical on every row.
    pub benchmark_ratio: f64,
    /// `d1a` economically disadvantaged ADM.
    pub dpia_econ_disadvantaged_adm: Adm,
    /// `d1b` directly certified ADM.
    pub dpia_directly_certified_adm: Adm,
    /// `d1` the blend.
    pub dpia_weighted_adm: Adm,
    /// `d2` the blend as a share of enrolled ADM.
    pub dpia_percentage: f64,
    /// The district's disadvantaged pupil impact aid.
    pub dpia_aid: Dollars,
}

/// The FY2026 model, in IRN order.
///
/// # Panics
///
/// If the fixture's header is not the one this reader was written against, or a row's width
/// differs from the header's — both by way of [`edfund_core::csv::rows`].
#[must_use]
pub fn frame() -> Vec<Prior> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .map(|row| Prior {
            irn: row.str(0).to_string(),
            name: row.str(1).to_string(),
            enrolled_adm: row.num(2).unwrap_or(0.0),
            base_cost_enrolled_adm: row.num(3).unwrap_or(0.0),
            aggregate_base_cost: row.num(4).unwrap_or(0.0),
            base_cost_per_pupil: row.num(5).unwrap_or(0.0),
            capacity_per_pupil: row.num(6).unwrap_or(0.0),
            state_share_percentage: row.num(7).unwrap_or(0.0),
            valuation_per_pupil: row.num(8).unwrap_or(0.0),
            local_capacity_percentage: row.num(9).unwrap_or(0.0),
            benchmark_ratio: row.num(10).unwrap_or(0.0),
            dpia_econ_disadvantaged_adm: row.num(11).unwrap_or(0.0),
            dpia_directly_certified_adm: row.num(12).unwrap_or(0.0),
            dpia_weighted_adm: row.num(13).unwrap_or(0.0),
            dpia_percentage: row.num(14).unwrap_or(0.0),
            dpia_aid: row.num(15).unwrap_or(0.0),
        })
        .collect()
}

/// The FY2026 model keyed by IRN.
#[must_use]
pub fn by_irn() -> BTreeMap<String, Prior> {
    frame()
        .into_iter()
        .map(|row| (row.irn.clone(), row))
        .collect()
}

/// A quantity both models publish, so it can be read across the two years.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantity {
    /// `[b1]` per-pupil local capacity.
    CapacityPerPupil,
    /// `[b2]` district base cost per pupil.
    BaseCostPerPupil,
    /// `[b4]` state share percentage.
    StateSharePercentage,
    /// `[C6]` local capacity percentage.
    LocalCapacityPercentage,
    /// `d1a` economically disadvantaged ADM.
    EconomicallyDisadvantagedAdm,
    /// `d1b` directly certified ADM.
    DirectlyCertifiedAdm,
    /// `d1` the blended count.
    WeightedDisadvantagedAdm,
    /// The district's disadvantaged pupil impact aid.
    DpiaAid,
}

impl Quantity {
    /// The FY2026 reading.
    fn prior(self, row: &Prior) -> Option<f64> {
        Some(match self {
            Self::CapacityPerPupil => row.capacity_per_pupil,
            Self::BaseCostPerPupil => row.base_cost_per_pupil,
            Self::StateSharePercentage => row.state_share_percentage,
            Self::LocalCapacityPercentage => row.local_capacity_percentage,
            Self::EconomicallyDisadvantagedAdm => row.dpia_econ_disadvantaged_adm,
            Self::DirectlyCertifiedAdm => row.dpia_directly_certified_adm,
            Self::WeightedDisadvantagedAdm => row.dpia_weighted_adm,
            Self::DpiaAid => row.dpia_aid,
        })
    }

    /// The FY2027 reading.
    fn current(self, record: &DistrictRecord) -> Option<f64> {
        match self {
            Self::CapacityPerPupil => record.published_capacity_per_pupil,
            Self::BaseCostPerPupil => Some(record.base_cost_per_pupil),
            Self::StateSharePercentage => record.published_state_share,
            Self::LocalCapacityPercentage => record.published_capacity_rate,
            Self::EconomicallyDisadvantagedAdm => Some(record.dpia.economically_disadvantaged_adm),
            Self::DirectlyCertifiedAdm => Some(record.dpia.directly_certified_adm),
            Self::WeightedDisadvantagedAdm => Some(record.dpia.weighted_adm),
            Self::DpiaAid => Some(record.categoricals.dpia),
        }
    }
}

/// One district's reading of one quantity in both years.
#[derive(Debug, Clone, PartialEq)]
pub struct Pair {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's name.
    pub name: String,
    /// What the FY2026 model publishes.
    pub fy2026: f64,
    /// What the FY2027 model publishes.
    pub fy2027: f64,
}

impl Pair {
    /// The change in the quantity's own units.
    #[must_use]
    pub fn change(&self) -> f64 {
        self.fy2027 - self.fy2026
    }

    /// The change as a fraction of the FY2026 reading, where that is positive.
    #[must_use]
    pub fn relative(&self) -> Option<f64> {
        (self.fy2026 > 0.0).then(|| self.change() / self.fy2026)
    }

    /// Whether the two readings agree to the last published digit.
    #[must_use]
    pub fn unchanged(&self) -> bool {
        (self.change()).abs() < 1e-9
    }
}

/// Every district both models carry a reading of, in IRN order.
#[must_use]
pub fn pairs(quantity: Quantity) -> Vec<Pair> {
    let current = panel::panel();
    let by_irn: BTreeMap<&str, &DistrictRecord> = current
        .iter()
        .map(|record| (record.irn.as_str(), record))
        .collect();
    frame()
        .into_iter()
        .filter_map(|row| {
            let record = by_irn.get(row.irn.as_str())?;
            Some(Pair {
                fy2026: quantity.prior(&row)?,
                fy2027: quantity.current(record)?,
                irn: row.irn,
                name: row.name,
            })
        })
        .collect()
}

/// The median of a sample, R type 7 at the halfway point on an even count's lower neighbour.
///
/// Local rather than borrowed from `dispersion` so that this module reports the same statistic
/// whichever way a caller reaches it, and so that a reading here can be reproduced by hand.
fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values.get(values.len() / 2).copied().unwrap_or(0.0)
}

/// The median change in a quantity, as `(its own units, a fraction)`.
#[must_use]
pub fn median_change(quantity: Quantity) -> (f64, f64) {
    let pairs = pairs(quantity);
    (
        median(pairs.iter().map(Pair::change).collect()),
        median(pairs.iter().filter_map(Pair::relative).collect()),
    )
}

/// How many districts a quantity fell in, and how many it is read for.
#[must_use]
pub fn fell(quantity: Quantity) -> (usize, usize) {
    let pairs = pairs(quantity);
    (
        pairs.iter().filter(|pair| pair.change() < 0.0).count(),
        pairs.len(),
    )
}

/// The fall in the state share percentage, as `(median percentage points, districts, of how many)`.
///
/// The headline of the module: it is the arithmetic consequence of freezing one side of a
/// subtraction and not the other.
#[must_use]
pub fn state_share_fall() -> (f64, usize, usize) {
    let (level, _) = median_change(Quantity::StateSharePercentage);
    let (fell, of) = fell(Quantity::StateSharePercentage);
    (-level, fell, of)
}

/// The median district's published state share of its base cost, `(FY2026, FY2027)`.
///
/// Over the 609 districts both models compute one for, so it pairs with
/// [`holding_capacity_still`] rather than with either workbook's own row count.
#[must_use]
pub fn median_state_share() -> (f64, f64) {
    let pairs = pairs(Quantity::StateSharePercentage);
    (
        median(pairs.iter().map(|pair| pair.fy2026).collect()),
        median(pairs.iter().map(|pair| pair.fy2027).collect()),
    )
}

/// The median FY2027 state share as published, and as it would be on FY2026 capacity.
///
/// Returned as `(published, with the capacity side held still)`. The gap between them is the
/// part of the fall the rolling valuation window accounts for, and it is nearly all of it: the
/// published FY2027 median is **0.3600** and holding capacity at its FY2026 value leaves **0.4123**.
///
/// The floor is applied to the counterfactual as the model applies it, so a district the
/// minimum state share already binds for does not move.
#[must_use]
pub fn holding_capacity_still() -> (f64, f64) {
    let prior = by_irn();
    let mut published = Vec::new();
    let mut counterfactual = Vec::new();
    for record in panel::panel() {
        let (Some(share), Some(before)) = (
            record.published_state_share,
            prior.get(&record.irn).map(|row| row.capacity_per_pupil),
        ) else {
            continue;
        };
        if record.base_cost_per_pupil <= 0.0 {
            continue;
        }
        published.push(share);
        counterfactual.push(
            ((record.base_cost_per_pupil - before) / record.base_cost_per_pupil)
                .max(MINIMUM_STATE_SHARE),
        );
    }
    (median(published), median(counterfactual))
}

/// The state's share of the aggregate base cost in each year, `(FY2026, FY2027)`.
///
/// Weighted by each district's own base cost rather than a mean of shares, which is what the
/// question "how much of the cost of Ohio's schools does the state pay" asks.
#[must_use]
pub fn aggregate_state_share() -> (f64, f64) {
    let prior_total: Dollars = frame()
        .iter()
        .map(|row| row.aggregate_base_cost)
        .sum::<Dollars>();
    let prior_state: Dollars = frame()
        .iter()
        .map(|row| row.aggregate_base_cost * row.state_share_percentage)
        .sum();
    let current = panel::panel();
    let current_total: Dollars = current.iter().map(|r| r.aggregate_base_cost).sum();
    let current_state: Dollars = current
        .iter()
        .map(|r| r.aggregate_base_cost * r.published_state_share.unwrap_or(0.0))
        .sum();
    (prior_state / prior_total, current_state / current_total)
}

/// Districts the minimum state share binds for in each year, `(FY2026, FY2027)`.
///
/// On the department's own published `[b4]`, within the tolerance
/// [`DistrictRecord::at_minimum_state_share`] uses, so the FY2027 figure is the 138 the corpus
/// already carries.
#[must_use]
pub fn at_the_floor() -> (usize, usize) {
    let at = |share: f64| (share - MINIMUM_STATE_SHARE).abs() < 0.000_5;
    let pairs = pairs(Quantity::StateSharePercentage);
    (
        pairs.iter().filter(|pair| at(pair.fy2026)).count(),
        pairs.iter().filter(|pair| at(pair.fy2027)).count(),
    )
}

/// Districts the minimum state share did not bind for in FY2026 and does in FY2027.
#[must_use]
pub fn fell_onto_the_floor() -> Vec<String> {
    let at = |share: f64| (share - MINIMUM_STATE_SHARE).abs() < 0.000_5;
    pairs(Quantity::StateSharePercentage)
        .into_iter()
        .filter(|pair| !at(pair.fy2026) && at(pair.fy2027))
        .map(|pair| pair.name)
        .collect()
}

/// Statewide disadvantaged pupil impact aid in each year, `(FY2026, FY2027)`.
#[must_use]
pub fn dpia_total() -> (Dollars, Dollars) {
    (
        frame().iter().map(|row| row.dpia_aid).sum(),
        panel::panel().iter().map(|r| r.categoricals.dpia).sum(),
    )
}

/// The statewide economically disadvantaged percentage `d3` indexes against, `(FY2026, FY2027)`.
#[must_use]
pub fn dpia_statewide_percentage() -> (f64, f64) {
    (
        DPIA_STATEWIDE_PERCENTAGE,
        panel::categoricals::DPIA_STATEWIDE_PERCENTAGE,
    )
}

/// The blend weights each year puts on the two poverty counts, `(FY2026, FY2027)`.
#[must_use]
pub fn dpia_blend() -> ((f64, f64), (f64, f64)) {
    (DPIA_BLEND, panel::categoricals::DPIA_BLEND)
}

/// The ratio of the fortieth-highest district, which caps the local capacity percentage.
///
/// `(FY2026, FY2027)`. It is the one decision in the local capacity build-up that is a choice
/// about a district rather than a reading of one, and across the interval it barely moves — which
/// is what makes the 9.34% move in what it multiplies a fact about Ohio rather than about the
/// rule.
#[must_use]
pub fn benchmark_ratio() -> (f64, f64) {
    let current = panel::panel()
        .iter()
        .find_map(|record| record.benchmark_ratio)
        .unwrap_or(0.0);
    (BENCHMARK_RATIO, current)
}
