//! How a year is reckoned, and the per-year rows every series is made of.
//!
//! These are the shapes a consumer meets before any district: a year's kind, a point in a
//! series, an outcome, a finance row, a casino distribution, the deflator.

use crate::*;

/// How a year is reckoned, because Ohio reckons three ways and they do not line up.
///
/// A tax year is a calendar year, and the revenue it raises reaches a district in the *following*
/// fiscal year. A school year straddles two calendar years and is published as `2024-25`. A fiscal
/// year runs July to June and is named for the June. Every one of those is "2024" to somebody, and
/// a feed that renders all three as a bare number invites the reader to subtract them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YearKind {
    /// July to June, named for the June. `FY2027`.
    Fiscal,
    /// A calendar year of valuation and levy. `2024 tax year`.
    Tax,
    /// September to June, named for both. `2024-25`.
    School,
}

impl YearKind {
    /// The token the feed writes, and the discriminant a consumer switches on.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fiscal => "fiscal",
            Self::Tax => "tax",
            Self::School => "school",
        }
    }
}

/// What year one series in this feed is measured in, and where that came from.
///
/// # Why this is a block rather than a field on each figure
///
/// Because the year is a property of the *source*, not of the number. The report card publishes
/// one year at a time; the Census survey publishes one year at a time; a district's valuation is
/// one tax year. Hanging a year on each of the two hundred-odd numeric fields would repeat the
/// same string two hundred times and still not say which of them moved together.
///
/// # Why it is in the feed at all
///
/// Because until now it was in doc comments and in hand-typed strings on the web pages —
/// `/// Performance Index, 2024-25` here, a literal `"2024-25"` in an Astro `<meta>` description
/// there. The web layer carried about 190 four-digit year literals, and a literal cannot go stale
/// visibly: regenerating a constant produces no diff, which is the same failure `connect`'s
/// `index` module had when its node count was the literal `58`.
///
/// The consumer looks a series up by [`SeriesYear::series`] and renders [`SeriesYear::label`].
/// Neither the year nor its form is ever composed on the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesYear {
    /// The key a consumer looks up. Finer than the object where an object mixes years — the
    /// report card's attainment and its spending are one extract and two reckonings, so they are
    /// `outcome.performance` and `outcome.spending` rather than one `outcome`.
    pub series: String,
    /// Which of Ohio's three reckonings this is.
    pub kind: YearKind,
    /// Ready to render, in the form its publisher uses. `FY2027`, `2024-25`, `2024`.
    ///
    /// A string and not a number because a school year has no single number, and forcing one
    /// would mean the feed choosing between `2024` and `2025` for a period that is both.
    pub label: String,
    /// What published it, in the words the page can print beside the figure.
    pub source: String,
}

/// How close to the floor counts as being on it, in mills.
///
/// Half a hundredth of a mill: Table SD-1 publishes effective rates to four decimals, and a
/// floored rate arrives as `20.0000` in 135 districts and within this band in 20 more. The
/// tolerance is a rounding allowance, not a judgement — 54 further districts sit between
/// `20.005` and `20.05`, close enough that the distinction carries no meaning for a reader but
/// far enough that calling them floored would be an invention rather than a rounding.
/// [`Statewide::near_millage_floor`] counts them instead of hiding them.
pub(crate) const FLOOR_TOLERANCE: f64 = 0.005;

/// The width of the band [`Statewide::near_millage_floor`] counts, in mills above the floor.
pub(crate) const NEAR_FLOOR_BAND: f64 = 0.05;

/// The outcome side of a district, where the report card covers it.
///
/// # Two spending figures and two poverty figures, both on purpose
///
/// `per_equivalent_pupil` divides by a need-weighted count and is the department's published
/// figure; `per_enrolled_pupil` divides by the headcount. Against a composition-driven outcome
/// the first is substantially a composition proxy, and the corpus's central denominator finding
/// is the gap between them. Shipping only one would make that finding unstateable in the
/// interface that is supposed to explain it.
///
/// `economically_disadvantaged` is the report card's, which is top-coded by community
/// eligibility. The profile report's untop-coded share stays on [`District`] itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistrictOutcome {
    /// Performance Index, 2024-25. Ohio's attainment-level measure.
    pub performance_index: Option<f64>,
    /// Performance Index, 2023-24.
    pub performance_index_prior: Option<f64>,
    /// Performance Index, 2022-23.
    pub performance_index_earliest: Option<f64>,
    /// Value-added effect size — Ohio's growth measure, already a three-year average.
    pub progress_effect_size: Option<f64>,
    /// The same measure over a single year, which the department also publishes.
    ///
    /// Carried so the smoothing is a visible choice rather than an invisible one. This site uses
    /// the three-year figure everywhere; until now it never said a second figure existed.
    ///
    /// The two turn out to agree wherever agreement means anything: of the 534 districts printing
    /// a non-zero value on both, 44 point opposite ways, and **not one of the 44 has both
    /// magnitudes above 0.05**. Every disagreement is a district within 0.04 of zero on both
    /// measures — no measured growth either way, and a sign that is arbitrary. Which is worth
    /// stating precisely, because the naive test is badly misleading: 72 districts print an exact
    /// `0.00` on one measure, and a bare `a > 0.0 != b > 0.0` counts every one of those as a
    /// disagreement and reports 76.
    pub progress_effect_size_one_year: Option<f64>,
    /// Operating expenditure per enrolled pupil, FY2025.
    pub per_enrolled_pupil: Option<Dollars>,
    /// Operating expenditure per need-weighted pupil, FY2025. The published figure.
    pub per_equivalent_pupil: Option<Dollars>,
    /// The federal part of [`DistrictOutcome::per_equivalent_pupil`].
    pub per_equivalent_pupil_federal: Option<Dollars>,
    /// The state and local part. The two add to the whole for every district that has them.
    pub per_equivalent_pupil_state_local: Option<Dollars>,
    /// Economically disadvantaged share, 2024-25, top-coded. A **fraction**, as every share here is.
    ///
    /// The report card publishes this as 0 to 100 and `main.rs` divides on the way in. Distinct
    /// from [`District::economically_disadvantaged`], which is the profile report's untop-coded
    /// share — the two are different variables, and until `35.0.0` they were also different units.
    pub economically_disadvantaged: Option<f64>,
    /// English learner share, 2024-25. A fraction; the report card publishes 0 to 100.
    pub english_learner: Option<f64>,
    /// Students with disabilities share, 2024-25. A fraction; the report card publishes 0 to 100.
    pub students_with_disabilities: Option<f64>,
}

/// Statewide relationships between the funding side and the outcome side.
///
/// Every one is a correlation over the joined panel and none identifies an effect. They are in
/// the feed rather than left to the page to compute, because the page would then have to choose
/// which poverty measure to control for, and that choice moves the answer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutcomeStatewide {
    /// Districts with both a funding record and a report card.
    pub districts: usize,
    /// Poverty against the Performance Index. The dominant relationship in the data.
    pub poverty_vs_performance: f64,
    /// Guarantee status against the Performance Index, raw.
    pub guarantee_vs_performance: f64,
    /// The same, holding poverty constant.
    pub guarantee_vs_performance_controlled: f64,
    /// Spending per enrolled pupil against growth, holding poverty constant.
    pub spending_vs_growth_controlled: f64,
    /// Spending per *weighted* pupil against the Performance Index, raw — the published
    /// near-zero figure whose denominator the corpus disputes.
    pub weighted_spending_vs_performance: f64,
    /// Spending per *enrolled* pupil against the Performance Index, raw.
    pub enrolled_spending_vs_performance: f64,
    /// Median Performance Index among districts on the guarantee.
    pub median_performance_on_guarantee: f64,
    /// Median Performance Index among districts on the formula.
    pub median_performance_on_formula: f64,
    /// Median federal share of operating spending.
    pub median_federal_share: f64,
    /// The highest federal share in the state, and whose it is.
    pub max_federal_share: f64,
    /// Districts where more than a tenth of operating spending is federal.
    pub federal_share_above_tenth: usize,
    /// Federal share against the Performance Index, holding poverty constant.
    ///
    /// Federal education money is allocated substantially by poverty, so the raw association is
    /// mostly a poverty association read backwards. The controlled figure is the one that says
    /// anything, and it is reported beside the raw one rather than instead of it.
    pub federal_share_vs_performance: f64,
    /// The same, raw.
    pub federal_share_vs_performance_raw: f64,
    /// Districts whose two growth measures print non-zero values pointing opposite ways.
    ///
    /// Counted only over districts where both measures are determinate. The department publishes
    /// value-added to two decimals, so a printed `0.00` covers anything in (-0.005, 0.005) and
    /// has no sign to disagree about; 72 districts are in that position and are excluded rather
    /// than silently counted as negative.
    pub growth_measures_disagree: usize,
    /// Districts where both measures print a non-zero value — the denominator for the above.
    pub growth_measures_determinate: usize,
    /// Districts where the two disagree *and* both magnitudes exceed 0.05. It is zero.
    ///
    /// The figure that makes the disagreement readable. Every case is a district sitting on zero,
    /// so the smoothing choice never reverses a district with real measured movement.
    pub growth_measures_disagree_materially: usize,
    /// Correlation between the one-year and three-year growth measures.
    pub growth_measure_agreement: f64,
}

/// One closed fiscal year of a district's general fund. Every figure is an audited actual.
///
/// From the district's own five-year forecast filing, not from the funding calculator. The two
/// are differently constructed and the feed never presents one as a check on the other.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FinanceYear {
    /// Fiscal year, ending 30 June.
    pub fiscal_year: u16,
    /// Unrestricted grants-in-aid: state foundation money as the district books it.
    pub state_aid: Dollars,
    /// Property tax plus income tax — the local levy yield actually collected.
    pub local_tax: Dollars,
    /// Total general fund revenue.
    pub total_revenue: Dollars,
    /// Total expenditures and other financing uses.
    pub total_expenditure: Dollars,
    /// Cash balance at 30 June. What the district holds.
    pub ending_cash: Dollars,
}

/// One fiscal year of the gross casino revenue county student fund.
///
/// Two payments, in January and August, summed. The year is the year of the **payment**, which is
/// how the money lands in a district's books; the halves are not carried separately here because
/// nothing on a district page reads a half-year, and the one thing that turns on the distinction —
/// the closure landing in FY2021 rather than FY2020 — is a property of the whole series and is
/// stated where the series is shown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CasinoYear {
    /// State fiscal year of the two payments.
    pub fiscal_year: u16,
    /// Dollars distributed.
    pub amount: Dollars,
}

/// A price index, so a consumer can restate any year of the panel in any other year's dollars.
///
/// Carried rather than left to the page because the choice of index is a claim. CPI-U is a
/// general consumer index and school costs are majority compensation, for which the Employment
/// Cost Index would be better and has shorter coverage — so the label travels with the numbers
/// and any figure derived from them must name it.
#[derive(Debug, Clone, PartialEq)]
pub struct Deflator {
    /// What the index is. Must be shown wherever a real-dollar figure is.
    pub label: String,
    /// One observation per covered fiscal year, oldest first.
    pub points: Vec<(u16, f64)>,
}
