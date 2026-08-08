//! Export a versioned JSON feed of the corpus's district-level findings.
//!
//! [`web/`](../../../web/) consumes a bundled export rather than reading
//! [`.yidam/corpus/`](../../../.yidam/corpus/) directly. The corpus is markdown and YAML written
//! for traversal by people and agents; the platform needs numbers for 609 districts. This
//! crate is the seam between them.
//!
//! # Contract version
//!
//! The bundle carries [`CONTRACT_VERSION`]. A consumer that does not recognise it should
//! refuse to render rather than guess, because a field silently changing meaning is worse than
//! a page that does not load. Bump it on any change to field names or units.
//!
//! # Checkpoints, and why a duplicated implementation is acceptable here
//!
//! The scenario builder in the web layer re-derives what `project::policy::apply` computes, in
//! TypeScript, so that moving a slider does not require a round trip. Two implementations of
//! the same formula is normally a bad trade: they drift, and the one nobody runs is the one
//! that is wrong.
//!
//! [`Checkpoint`] is the answer. The bundle carries Rust-computed results for a set of named
//! policies, and the page verifies its own arithmetic against them **before** it will render a
//! scenario. If the two disagree the page says so and disables the tab. The duplication is then
//! load-bearing in only one direction: the Rust is authoritative and the TypeScript has to prove
//! it agrees, on every page load, against the real 609-district panel.
//!
//! # Why hand-rolled JSON
//!
//! The workspace has no external dependencies, deliberately — a committed
//! [`scenario`](../../../.yidam/corpus/scenario/) result should be reproducible years from now
//! without a dependency resolution succeeding first. Serializing a fixed, known schema is a
//! few dozen lines, so that constraint costs nothing here.

#![forbid(unsafe_code)]

use edfund_core::Dollars;

/// The bundle schema version. Bump on any change to field names, units, or semantics.
///
/// `9.0.0` wired the [`millage`] crate in. Breaking because [`District::at_millage_floor`]
/// changes answer for 21 districts: it compared the effective Class I rate to a literal `20.0`,
/// so a district charging *less* than twenty mills was reported as being above the floor with
/// reduction factors operative, which is the opposite of its position. The feed also gains
/// [`District::voted_operating_millage`] — a column that was in the profile CSV from the start
/// and never parsed — and the [`MillageAnalysis`] block computed from it.
///
/// `7.0.0` and `8.0.0` added the base cost build-up, Table SD-1 and spending by function.
///
/// `6.0.0` added the price index and the statewide financial aggregates. Breaking because the
/// feed now carries figures a consumer can deflate, and a page that shows the FY2020-FY2025
/// panel in nominal dollars is not merely imprecise — across a span in which CPI-U rose 25.1%,
/// a nominal statement about it can have the wrong sign.
///
/// `5.0.0` added the actuals: a `finances` array per district carrying six closed fiscal years
/// of what it received, raised, spent, and held. Additive in shape but breaking in meaning — it
/// is the first per-district figure in the feed that is a record rather than a model, and a
/// consumer that rendered it beside the FY2027 calculator's output without saying which was
/// which would present a measurement and a projection as the same kind of claim.
///
/// `4.0.0` added the projection axis: `adm_history` on every district, so the page can carry
/// enrollment forward itself, and a `projection` block holding the forecast's method, its prior,
/// and [`ForecastCheckpoint`]s the page must reproduce before it may draw a band. Breaking rather
/// than additive because `adm_history` is required, not nullable — a district without it cannot
/// be projected, and a feed that omitted it would produce a page silently missing half its
/// panel.
///
/// `3.0.0` added the outcome axis: a nullable `outcome` object per district carrying the
/// Performance Index, the Progress effect size, need shares, and spending on both denominators,
/// plus the statewide correlations that say how to read them. Nullable because three districts
/// have no report card — see [`project::crosswalk`].
///
/// `2.0.0` added the scenario inputs and checkpoints, and renamed the enrollment-change years
/// from FY2022-FY2024 to FY2024-FY2026 — the years the department's `ADM Data` sheet declares.
/// The values did not change; what they are called did, which is exactly the kind of silent
/// meaning change the version guard exists for.
pub const CONTRACT_VERSION: &str = "12.0.0";

/// How close to the floor counts as being on it, in mills.
///
/// Half a hundredth of a mill: Table SD-1 publishes effective rates to four decimals, and a
/// floored rate arrives as `20.0000` in 135 districts and within this band in 20 more. The
/// tolerance is a rounding allowance, not a judgement — 54 further districts sit between
/// `20.005` and `20.05`, close enough that the distinction carries no meaning for a reader but
/// far enough that calling them floored would be an invention rather than a rounding.
/// [`Statewide::near_millage_floor`] counts them instead of hiding them.
const FLOOR_TOLERANCE: f64 = 0.005;

/// The width of the band [`Statewide::near_millage_floor`] counts, in mills above the floor.
const NEAR_FLOOR_BAND: f64 = 0.05;

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
    /// Economically disadvantaged share, 2024-25, top-coded.
    pub economically_disadvantaged: Option<f64>,
    /// English learner share, 2024-25.
    pub english_learner: Option<f64>,
    /// Students with disabilities share, 2024-25.
    pub students_with_disabilities: Option<f64>,
}

impl DistrictOutcome {
    /// Federal money as a share of this district's operating spending.
    ///
    /// The share rather than the dollars, wherever one number has to stand for this. Both parts
    /// are published per **need-weighted** pupil, so the dollars carry a denominator that has to
    /// be named every time it appears; the ratio of two figures on the same denominator does not.
    /// It is the one spending statistic on this site that can be set beside any other district's
    /// without asking which pupil count either divides by.
    #[must_use]
    pub fn federal_share(&self) -> Option<f64> {
        let (federal, total) = (
            self.per_equivalent_pupil_federal?,
            self.per_equivalent_pupil?,
        );
        (total > 0.0).then_some(federal / total)
    }
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

/// The base cost build-up for one district, all twenty-two elements of R.C. 3317.011.
///
/// # Why the feed carries the parts and not just the total
///
/// `base_cost_per_pupil` answers "how much"; this answers "why". Base cost is assembled from
/// statutory staffing ratios applied to a district's own enrollment, priced at statewide average
/// salaries — so the number a district argues about is the sum of twenty-two decisions, and the
/// interface showed only the sum.
///
/// # And why it carries the department's figure beside its own
///
/// This is computed by `foundation`, not read. That is a claim, so the published aggregate travels
/// with it and [`BaseCostBuildUp::residual`] is the difference — a dollar or so on figures in the
/// millions, from twenty-two elements each rounded where the department rounds. Publishing the
/// residual is the difference between reproducing a number and asserting that you have.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BaseCostBuildUp {
    /// A1 — classroom teachers, at the statutory ratio for each grade band.
    pub classroom_teachers: Dollars,
    /// A2 — special teachers, at one per 150 pupils.
    pub special_teachers: Dollars,
    /// A3 — substitutes.
    pub substitutes: Dollars,
    /// A4 — professional development.
    pub professional_development: Dollars,
    /// A — teacher base cost, R.C. 3317.011(D).
    pub teachers: Dollars,
    /// B1 — guidance counselors.
    pub counselors: Dollars,
    /// B2 — librarians and media staff.
    pub librarians: Dollars,
    /// B3 — student wellness and success staff.
    pub wellness: Dollars,
    /// B4 — academic co-curricular activities.
    pub academic_cocurricular: Dollars,
    /// B5 — building safety and security.
    pub safety: Dollars,
    /// B6 — supplies and academic content.
    pub supplies: Dollars,
    /// B7 — student technology.
    pub technology: Dollars,
    /// B — student support base cost, R.C. 3317.011(E).
    pub student_support: Dollars,
    /// C1 — superintendent. The one price in the formula that varies with district size.
    pub superintendent: Dollars,
    /// C2 — treasurer.
    pub treasurer: Dollars,
    /// C3 — other district administrators, priced at 82.8% of the superintendent band.
    pub other_administrators: Dollars,
    /// C4 — fiscal support.
    pub fiscal_support: Dollars,
    /// C5 — EMIS support.
    pub emis: Dollars,
    /// C6 — district leadership support.
    pub leadership_support: Dollars,
    /// C7 — information technology centre support.
    pub itc: Dollars,
    /// C — district leadership and accountability, R.C. 3317.011(F).
    pub district_leadership: Dollars,
    /// D1 — building leadership, priced at 79.38% of the superintendent band.
    pub building_leadership_staff: Dollars,
    /// D2 — building leadership support.
    pub building_support: Dollars,
    /// D3 — building operation.
    pub building_operation: Dollars,
    /// D — building leadership and operation, R.C. 3317.011(G).
    pub building_leadership: Dollars,
    /// E — athletic co-curricular activities, R.C. 3317.011(H).
    pub athletic_cocurricular: Dollars,
    /// Funded classroom teaching positions, as the department rounds them.
    pub funded_classroom_teachers: f64,
    /// Funded special teaching positions.
    pub funded_special_teachers: f64,
    /// A + B + C + D + E, as computed here.
    pub computed_aggregate: Dollars,
    /// What the department published for the same district.
    pub published_aggregate: Dollars,
    /// `computed_aggregate - published_aggregate`. Accumulated rounding across the elements.
    pub residual: Dollars,
}

/// One tax year of a district's property tax base and the tax charged on it.
///
/// From Table SD-1, the Department of Taxation's own per-district table — a different department
/// from the one that publishes the funding model, which is what makes it worth carrying: the
/// state's two halves describe the same district and are not obliged to agree. Where they overlap
/// they do, to 0.01 mills across all 606 districts that appear in both.
///
/// Two years are carried rather than one because the mechanism this data exists to show only
/// exists as a change. H.B. 920's reduction factors roll an effective rate back as valuation
/// rises, and cannot roll it below twenty mills — so what a reappraisal does to a district's
/// revenue depends entirely on which side of that floor it sits, and a single year cannot show it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PropertyTaxYear {
    /// Tax year. Offset from the fiscal year, and collected across the following calendar year.
    pub tax_year: u16,
    /// Class I: residential and agricultural, which carry their own reduction factor.
    pub class1_value: Dollars,
    /// Class II: everything else real — commercial, industrial, mineral, railroad.
    pub class2_value: Dollars,
    /// Public utility tangible property, which is neither class and is not reduced.
    pub public_utility_value: Dollars,
    /// Class I + Class II + public utility.
    pub total_value: Dollars,
    /// Agricultural value, inside Class I.
    pub agricultural_value: Dollars,
    /// Residential value, inside Class I. Seven-tenths of the state's base.
    pub residential_value: Dollars,
    /// Commercial value, inside Class II.
    pub commercial_value: Dollars,
    /// Industrial value, inside Class II.
    pub industrial_value: Dollars,
    /// Mineral value, inside Class II.
    pub mineral_value: Dollars,
    /// Railroad value, inside Class II.
    pub railroad_value: Dollars,
    /// Effective Class I operating millage, after reduction factors.
    pub class1_rate: f64,
    /// Effective Class II operating millage.
    pub class2_rate: f64,
    /// Class I tax charged for current expenses.
    pub class1_taxes_charged: Dollars,
    /// Class II tax charged for current expenses.
    pub class2_taxes_charged: Dollars,
    /// Real property tax charged, both classes, excluding joint vocational operating levies.
    pub real_property_taxes_charged: Dollars,
    /// Public utility tax charged.
    pub public_utility_taxes_charged: Dollars,
    /// Total value over [`PropertyTaxYear::adm`].
    pub value_per_pupil: Dollars,
    /// The pupil count Table SD-1 divides by, which is **not** the funding formula's.
    ///
    /// Carried explicitly because the two departments publish the same numerator over different
    /// denominators and the difference is large. Multiply the District Profile Report's
    /// `assessed_valuation_per_pupil` by its enrolled ADM and you recover this table's
    /// `total_value` to 1.000 for all 606 districts carrying both — the taxable valuations are
    /// identical to the dollar. The pupil counts are not: Columbus is 43,019 to the Department of
    /// Education and 71,947 here, Youngstown 4,322 against 9,655.
    ///
    /// Taxation counts children residing in the district; Education's enrolled ADM counts the
    /// ones the district teaches. The gap is charter, voucher and open-enrolment-out students, so
    /// it is widest in exactly the districts where valuation per pupil does the most work in the
    /// aid formula. A page that prints one figure against the other's median is comparing two
    /// quantities that share a name and nothing else.
    pub adm: f64,
}

/// Where a district's operating money went in FY2025, per pupil, by function.
///
/// The report card's spending file, and therefore **not** the audited actuals in
/// [`District::finances`]: a different source, a different basis, and a per-pupil figure rather
/// than a total. The two answer "what did it spend it on" and "what changed hands", and this feed
/// keeps them apart because a reader who added them would be double-counting.
///
/// `classroom_instruction` and `nonclassroom` are the department's own two roll-ups and partition
/// operating spending exactly; the named functions below sit inside one or the other.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpendingByFunction {
    /// Unweighted ADM, FY2025 — the headcount denominator, not the need-weighted one.
    pub adm: f64,
    /// Total operating expenditure per pupil, FY2025.
    pub operating_per_pupil: Dollars,
    /// Classroom instruction, the department's roll-up.
    pub classroom_instruction: Dollars,
    /// Everything else, the department's other roll-up.
    pub nonclassroom: Dollars,
    /// Instruction.
    pub instruction: Dollars,
    /// Pupil support.
    pub pupil_support: Dollars,
    /// Instructional staff support.
    pub instructional_staff_support: Dollars,
    /// General administration.
    pub general_admin: Dollars,
    /// School administration.
    pub school_admin: Dollars,
    /// Operations and maintenance.
    pub operations_maintenance: Dollars,
    /// Pupil transportation.
    pub pupil_transportation: Dollars,
    /// Other support services.
    pub other_support: Dollars,
    /// Food service.
    pub food_service: Dollars,
}

/// H.B. 920 applied to one district, using the [`millage`] crate rather than restating it.
///
/// # Why this is computed and not quoted
///
/// Every other property-tax figure in the feed is a published number copied across. These are
/// the [`millage`] calculator run against two tax years of Table SD-1, which lets the page say
/// three things no published column states: how much of the voted rate the reduction factors
/// have removed, what the factors alone predict for the current year, and how far the observed
/// rate departs from that prediction.
///
/// # The residual is the interesting field
///
/// Reduction factors apply to *existing* levies on *existing* property. New construction and
/// newly voted millage are exempt from them by statute. So the gap between the predicted rate
/// and the observed one is not error — it is precisely the millage the factors do not reach,
/// and its sign says which way. Positive means new levies or new construction outran the
/// reduction; negative means levies expired faster than the factors alone would explain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MillageAnalysis {
    /// The tax year the observed and predicted rates describe.
    pub tax_year: u16,
    /// Effective Class I rate the prior tax year, the base the prediction runs from.
    pub prior_rate: f64,
    /// Effective Class I rate this tax year, as Table SD-1 publishes it.
    pub observed_rate: f64,
    /// What reduction factors alone predict: the prior rate scaled by the change in Class I
    /// value, held at the statutory floor. [`millage::effective_millage`].
    pub predicted_rate: f64,
    /// `observed_rate - predicted_rate`, in mills. What the factors cannot account for.
    pub residual: f64,
    /// Whether the floor is what stopped the reduction, per [`millage::FloorStatus`].
    pub at_floor: bool,
    /// Fraction of the voted rate H.B. 920 has removed, cumulatively, since each levy passed.
    /// `None` where the profile CSV carries no voted millage.
    pub cumulative_reduction: Option<f64>,
    /// What one mill raises per pupil against this district's real property base.
    /// [`millage::yield_of`] at one mill, over ADM. The local half of the formula in one number.
    pub yield_per_mill_per_pupil: Dollars,
}

/// What the mechanism the Fair School Funding Plan replaced would charge this district today.
///
/// [`regime_diff::at_fy2027`], which holds the plan's own computed base cost fixed and swaps only
/// the local share: instead of the local capacity measure, the charge-off's flat statutory
/// millage against assessed valuation. It is a counterfactual at current inputs, **not** a
/// reconstruction of any year the charge-off governed — those need the era's formula amount,
/// cost-of-doing-business factor and DPIA, none of which this corpus holds.
///
/// # Why this belongs beside the property tax
///
/// The charge-off *was* a millage calculation: a rate the legislature set, multiplied by a
/// district's valuation, subtracted from its cost. Its documented failure is that the rate was
/// uniform while H.B. 920 made effective rates anything but, so a district whose own rate had
/// fallen below the charge-off rate was charged for revenue it could not collect. The corpus has
/// asserted that since it was written. With Table SD-1 it is countable, and it is not a fringe
/// case: half the state is below the terminal rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegimeCounterfactual {
    /// The statutory rate the counterfactual runs at — [`regime_diff::TERMINAL_MILLS`].
    pub charge_off_mills: f64,
    /// Deemed local share per pupil under the charge-off: the rate against valuation.
    pub charge_off_local_share: Option<Dollars>,
    /// Local capacity per pupil as the plan measures it, recovered by subtraction.
    ///
    /// `None` where the minimum state share binds and all that is knowable is that capacity
    /// exceeds a threshold. Not zero: a censored quantity is not a small one, and substituting
    /// zero would invert the comparison for the districts where it is most interesting.
    pub local_capacity: Option<Dollars>,
    /// Base cost aid per pupil under the charge-off, floored at zero — it had no minimum share.
    pub aid_charge_off: Option<Dollars>,
    /// Base cost aid per pupil as the plan computes it.
    pub aid_fsfp: Option<Dollars>,
    /// Plan minus charge-off, per pupil. Positive means the district gained by the change.
    pub difference: Option<Dollars>,
    /// What the one aligned component fails to explain. Zero is the expected answer.
    pub residual: Option<Dollars>,
    /// Whether the charge-off would have run past the whole base cost it was subtracted from.
    ///
    /// The charge-off had no minimum state share — that is the plan's invention — so these
    /// districts would receive nothing at all. Ohio's answer was a supplement rather than a
    /// floor, and neither this crate nor `regime-diff` models it.
    pub exceeds_base_cost: bool,
    /// Effective Class I mills short of the charge-off rate, where the district is short.
    ///
    /// The phantom revenue mechanism, per district. `None` where the district's own effective
    /// rate is at or above the rate it would be charged at.
    pub mills_short_of_charge_off: Option<f64>,
}

/// One district, as the web layer needs it.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Information Retrieval Number, the stable statewide identifier.
    pub irn: String,
    /// District name as published.
    pub name: String,
    /// Base cost enrolled ADM — the greater of the three-year average and the current year.
    pub adm: f64,
    /// Current-year enrolled ADM, FY2026. The denominator the state share is paid on.
    pub current_year_adm: f64,
    /// District base cost per pupil, FY2027.
    pub base_cost_per_pupil: Dollars,
    /// Aggregate base cost, all five sub-components.
    pub aggregate_base_cost: Dollars,
    /// How that aggregate is assembled, recomputed here rather than quoted.
    pub base_cost_build_up: BaseCostBuildUp,
    /// Property tax base and charge, TY2023 and TY2024. Empty where the district is absent.
    pub property_tax: Vec<PropertyTaxYear>,
    /// Operating spending by function, FY2025. `None` for the two districts without a row.
    pub spending_by_function: Option<SpendingByFunction>,
    /// The state's share of base cost alone, before every categorical.
    pub base_cost_state_share: Dollars,
    /// Targeted assistance, special education, DPIA, English learner, gifted, career-technical.
    pub categorical_funding: Dollars,
    /// State aid per pupil as the formula computes it, before the guarantee.
    pub formula_aid_per_pupil: Dollars,
    /// State aid per pupil as the district receives it.
    pub realized_aid_per_pupil: Dollars,
    /// Temporary transitional aid guarantee, total dollars.
    pub guarantee: Dollars,
    /// Whether the minimum state share is what sets this district's base cost aid.
    pub at_minimum_state_share: bool,
    /// Assessed valuation per pupil, FY2023.
    pub valuation_per_pupil: Option<Dollars>,
    /// Effective Class 1 operating millage, TY2023.
    pub effective_class1_millage: Option<f64>,
    /// Voted current operating millage, TY2023 — the gross rate before reduction factors.
    ///
    /// The rate the district's voters actually approved, which is not the rate anyone pays. It
    /// sat in column 6 of the profile CSV from the first import and was never parsed, which is
    /// why the site could describe H.B. 920 but never say how much of it a district had lost.
    pub voted_operating_millage: Option<f64>,
    /// H.B. 920 run against this district, rather than described. `None` without two tax years.
    pub millage: Option<MillageAnalysis>,
    /// What the mechanism the plan replaced would charge this district. `None` without valuation.
    pub regime: Option<RegimeCounterfactual>,
    /// Total operating expenditure per pupil, FY2024.
    pub operating_expenditure_per_pupil: Option<Dollars>,
    /// Share of students economically disadvantaged, FY2024, as a fraction.
    pub economically_disadvantaged: Option<f64>,
    /// Enrollment change FY2024 to FY2026, as a fraction. FY2026 is partly departmental
    /// estimate, since the calculator is published before that year closes.
    pub enrollment_change: Option<f64>,
    /// Enrolled ADM for FY2024, FY2025, FY2026 — the three years the department's `ADM Data`
    /// sheet carries.
    ///
    /// Shipped as the series rather than only as [`District::enrollment_change`] because the
    /// page projects from it. Three points is not enough to estimate this district's own
    /// variability, which is exactly why the interval comes from the cross-sectional spread
    /// instead; see [`Projection::sigma`].
    pub adm_history: [f64; 3],
    /// Achievement, growth, and need. `None` for the three districts with no report card.
    pub outcome: Option<DistrictOutcome>,
    /// Six closed fiscal years of actuals, oldest first. Empty where no filing was found.
    pub finances: Vec<FinanceYear>,
}

impl District {
    /// Whether the district is funded by the guarantee rather than the formula.
    #[must_use]
    pub fn on_guarantee(&self) -> bool {
        self.guarantee > 0.0
    }

    /// Whether reduction factors have stopped operating on this district, so that valuation
    /// growth reaches its revenue.
    ///
    /// # Why this is `<=` and not `== 20.0`
    ///
    /// This compared the effective rate to a literal `20.0` within half a hundredth of a mill,
    /// which got 21 districts backwards. Six of them — Vinton County at 18.70 mills, Chesapeake
    /// Union and Highland at 19.00, Oak Hill Union and Scioto Valley at 19.60, Northwest at
    /// 19.71 — never voted twenty mills of current operating levy, so there is no reduction for
    /// the factors to make and their voted and effective rates are identical to four decimals.
    /// The other fifteen were reduced to just under twenty. All of them were reported as being
    /// *above* the floor with reduction factors operative, which is the reverse of their
    /// position. [`millage::FloorStatus`] answers it correctly: at or below the floor, the
    /// factors have stopped.
    ///
    /// The floor read from [`millage::floor_for`] rather than written here, so that the two
    /// statutory values — twenty mills, and two for a joint vocational district — stay in the
    /// crate that cites the statute. This feed carries traditional districts only.
    #[must_use]
    pub fn at_millage_floor(&self) -> bool {
        let floor = millage::floor_for(edfund_core::AgencyType::City).unwrap_or(20.0);
        self.millage
            .map(|m| m.observed_rate)
            .or(self.effective_class1_millage)
            .is_some_and(|m| m <= floor + FLOOR_TOLERANCE)
    }

    /// Above the floor, but by less than [`NEAR_FLOOR_BAND`] — where the binary stops meaning
    /// anything.
    ///
    /// The site calls floor status the highest-leverage single fact about a district's local
    /// revenue, and for most districts it is. For these it is a coin toss decided in the fourth
    /// decimal place, and 75 districts crossed `20.0000` in one direction or the other between
    /// TY2023 and TY2024. Counting them is the honest alternative to widening the tolerance
    /// until they fall on the side that looks tidier.
    #[must_use]
    pub fn near_millage_floor(&self) -> bool {
        let floor = millage::floor_for(edfund_core::AgencyType::City).unwrap_or(20.0);
        !self.at_millage_floor()
            && self
                .millage
                .map(|m| m.observed_rate)
                .or(self.effective_class1_millage)
                .is_some_and(|m| m <= floor + NEAR_FLOOR_BAND)
    }

    /// The FY2020 baseline the guarantee holds this district at, recoverable only when it is
    /// on the guarantee.
    #[must_use]
    pub fn implied_fy2020_baseline_per_pupil(&self) -> Option<Dollars> {
        self.on_guarantee().then_some(self.realized_aid_per_pupil)
    }
}

/// Statewide context, so a consumer can position any district without recomputing.
#[derive(Debug, Clone, PartialEq)]
pub struct Statewide {
    /// Number of districts in the bundle.
    pub districts: usize,
    /// Districts funded by the guarantee.
    pub on_guarantee: usize,
    /// Districts at the 20-mill floor.
    pub at_millage_floor: usize,
    /// Districts above the floor by less than a twentieth of a mill; see
    /// [`District::near_millage_floor`].
    pub near_millage_floor: usize,
    /// Median voted current operating millage — the rate voters approved.
    pub median_voted_millage: f64,
    /// Median effective Class I rate — the rate anyone pays. The gap is H.B. 920.
    pub median_effective_millage: f64,
    /// Median share of its voted rate a district has lost to reduction factors.
    ///
    /// Not `1 - median_effective / median_voted`. That is the ratio of medians, which is a
    /// different district's arithmetic in the numerator and the denominator and answers no
    /// question anyone asked. This is the median of the per-district ratio.
    pub median_millage_reduction: f64,
    /// What one mill raises per pupil, statewide median.
    ///
    /// The local half of the formula reduced to one number. A mill is the same rate everywhere
    /// and raises hundreds of times as much in one district as in another, which is why
    /// comparing two districts' millage without it compares effort to capacity.
    pub median_yield_per_mill: Dollars,
    /// The lowest yield per mill per pupil in the state.
    pub min_yield_per_mill: Dollars,
    /// The highest.
    pub max_yield_per_mill: Dollars,
    /// Median taxable value per pupil **on Table SD-1's denominator**.
    ///
    /// Separate from [`Statewide::median_valuation_per_pupil`], which is on the District Profile
    /// Report's enrolled ADM. The two numerators are identical to the dollar and the two pupil
    /// counts are not, so a district's SD-1 figure has to be positioned against this median and
    /// not against the other one. See [`PropertyTaxYear::adm`].
    pub median_sd1_value_per_pupil: Dollars,
    /// Districts whose effective Class I rate is below the charge-off rate they would be
    /// charged at — the phantom revenue the mechanism was replaced for producing.
    pub below_charge_off_rate: usize,
    /// Districts the charge-off would leave with no base cost aid at all, having no minimum
    /// state share to stop at.
    pub charge_off_exceeds_base_cost: usize,
    /// Median change in base cost aid per pupil from the charge-off to the plan.
    pub median_regime_difference: Dollars,
    /// Districts whose base cost aid is set by the minimum state share.
    pub at_minimum_state_share: usize,
    /// Median assessed valuation per pupil.
    pub median_valuation_per_pupil: Dollars,
    /// Median operating expenditure per pupil.
    pub median_operating_expenditure_per_pupil: Dollars,
    /// Correlation between valuation per pupil and formula aid per pupil.
    pub wealth_neutrality_formula: f64,
    /// Correlation between valuation per pupil and realized aid per pupil.
    pub wealth_neutrality_realized: f64,
    /// Total guarantee dollars.
    pub guarantee_total: Dollars,
    /// Total realized state aid.
    pub realized_aid_total: Dollars,
    /// The minimum state share this model operates under.
    pub minimum_state_share: f64,
    /// How the funding side relates to the outcome side. `None` if no district joined.
    pub outcomes: Option<OutcomeStatewide>,
    /// Closed fiscal years of actuals, summed over the districts in this feed.
    ///
    /// Summed in Rust rather than left to the page so that the two cannot disagree about which
    /// districts are in the total. The panel behind it covers 660 reporting bodies including
    /// joint vocational districts; this is the 609 traditional districts the feed carries, which
    /// is the population every other figure on the page is over.
    pub finances: Vec<FinanceYear>,
}

/// A policy, in the shape the web layer sends it back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolicyShape {
    /// `as-enacted`, `removed`, `rebase`, or `phase-out`.
    pub guarantee: &'static str,
    /// The factor or remaining share, where the rule takes one.
    pub guarantee_argument: f64,
    /// Multiplier on aggregate base cost.
    pub base_cost_scale: f64,
    /// Minimum state share of base cost.
    pub minimum_state_share: f64,
    /// Appropriated fraction of base cost aid.
    pub phase_in_base_cost: f64,
    /// Appropriated fraction of categorical aid.
    pub phase_in_categorical: f64,
}

/// A Rust-computed result the web layer must reproduce before it is allowed to compute more.
///
/// See the crate note. This is what makes a second implementation of the formula acceptable.
#[derive(Debug, Clone, PartialEq)]
pub struct Checkpoint {
    /// Human label, shown if the check fails.
    pub label: String,
    /// The policy that produced it. Without this a consumer could verify a number while
    /// computing a different scenario from the one the number belongs to.
    pub policy: PolicyShape,
    /// Change in total state aid against current law.
    pub cost: Dollars,
    /// Total realized aid under the policy.
    pub realized_aid: Dollars,
    /// Districts whose aid rises.
    pub gainers: usize,
    /// Districts whose aid falls.
    pub losers: usize,
    /// Districts the policy does not reach.
    pub unmoved: usize,
    /// Districts on the guarantee under the policy.
    pub on_guarantee: usize,
}

/// A Rust-computed *forecast* the web layer must reproduce before it may draw a band.
///
/// The same discipline as [`Checkpoint`], applied to the harder half. Reproducing a simulation
/// checks one function; reproducing a forecast checks the projection, the prior, the compounding
/// of the interval with the horizon, and the decision to re-run the whole formula at each end of
/// the enrollment band rather than scale the central answer — which matters because the
/// guarantee is a `max` and the aid curve has a kink no scaling reproduces.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCheckpoint {
    /// Human label, shown if the check fails.
    pub label: String,
    /// The policy held fixed across the horizon.
    pub policy: PolicyShape,
    /// The fiscal year projected to.
    pub fiscal_year: u16,
    /// Total realized aid at the central enrollment estimate.
    pub realized_aid: Dollars,
    /// Total realized aid at the low end of the enrollment band.
    pub low: Dollars,
    /// Total realized aid at the high end.
    pub high: Dollars,
    /// Projected total ADM.
    pub adm: f64,
    /// Districts on the guarantee at projected enrollment.
    pub on_guarantee: usize,
}

/// How this feed's forecasts were made, and what their interval rests on.
///
/// The page carries its own copy of the projection so a slider does not need a round trip, as it
/// does for the formula. This block is what makes that acceptable: the method and its parameters
/// so the page runs the same one, and [`Projection::checkpoints`] so it has to prove it did.
#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    /// The last observed fiscal year. Everything past it is forecast.
    pub base_year: u16,
    /// The furthest year the checkpoints reach, and the furthest the page should offer.
    pub horizon: u16,
    /// `damped`, `cagr`, `linear`, or `flat`.
    pub method: String,
    /// Per-year decay applied to the fitted growth rate. 1.0 is undamped.
    pub damping: f64,
    /// Standard deviation of annual enrolled-ADM growth **across districts**.
    ///
    /// Not this district's variability — three observations cannot give that. It is how much
    /// districts differ from one another, used as a floor on the uncertainty.
    pub sigma: f64,
    /// Standard deviations spanned on each side of the point.
    pub z: f64,
    /// What produced [`Projection::sigma`]. Printed wherever the band is.
    pub prior_source: String,
    /// Forecasts the consumer must reproduce.
    pub checkpoints: Vec<ForecastCheckpoint>,
}

/// The exported feed.
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    /// Schema version; see [`CONTRACT_VERSION`].
    pub contract_version: String,
    /// What the figures describe and where they came from.
    pub provenance: String,
    /// The fiscal year the model computes.
    pub fiscal_year: u16,
    /// Statewide aggregates.
    pub statewide: Statewide,
    /// Reference results the consumer must reproduce.
    pub checkpoints: Vec<Checkpoint>,
    /// How to project, and the forecasts that check the projection. `None` disables the band.
    pub projection: Option<Projection>,
    /// The price index. `None` means the feed can only be shown in nominal dollars.
    pub deflator: Option<Deflator>,
    /// Where Ohio sits among the states. `None` if the Census fixture is absent.
    pub national: Option<National>,
    /// Per-district records.
    pub districts: Vec<District>,
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn num(v: f64) -> String {
    if v.is_finite() {
        format!("{v:.4}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        "null".into()
    }
}

/// One `FinanceYear` as a JSON object. Shared so the per-district and statewide arrays cannot
/// drift into different field names.
fn finance_year(y: &FinanceYear) -> String {
    format!(
        "{{\"fiscal_year\": {}, \"state_aid\": {}, \"local_tax\": {}, \
         \"total_revenue\": {}, \"total_expenditure\": {}, \"ending_cash\": {}}}",
        y.fiscal_year,
        num(y.state_aid),
        num(y.local_tax),
        num(y.total_revenue),
        num(y.total_expenditure),
        num(y.ending_cash)
    )
}

fn opt(v: Option<f64>) -> String {
    v.map_or_else(|| "null".into(), num)
}

impl Bundle {
    /// Serialize to JSON.
    ///
    /// Deterministic: the same bundle always produces byte-identical output, so a committed
    /// feed diffs cleanly and a regenerated one shows only real changes.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut s = String::with_capacity(self.districts.len() * 320 + 4096);
        s.push_str("{\n");
        s.push_str(&format!(
            "  \"contract_version\": \"{}\",\n",
            escape(&self.contract_version)
        ));
        s.push_str(&format!(
            "  \"provenance\": \"{}\",\n",
            escape(&self.provenance)
        ));
        s.push_str(&format!("  \"fiscal_year\": {},\n", self.fiscal_year));

        let w = &self.statewide;
        s.push_str("  \"statewide\": {\n");
        s.push_str(&format!("    \"districts\": {},\n", w.districts));
        s.push_str(&format!("    \"on_guarantee\": {},\n", w.on_guarantee));
        s.push_str(&format!(
            "    \"at_millage_floor\": {},\n",
            w.at_millage_floor
        ));
        s.push_str(&format!(
            "    \"near_millage_floor\": {},\n",
            w.near_millage_floor
        ));
        s.push_str(&format!(
            "    \"median_voted_millage\": {},\n",
            num(w.median_voted_millage)
        ));
        s.push_str(&format!(
            "    \"median_effective_millage\": {},\n",
            num(w.median_effective_millage)
        ));
        s.push_str(&format!(
            "    \"median_millage_reduction\": {},\n",
            num(w.median_millage_reduction)
        ));
        for (key, value) in [
            ("median_yield_per_mill", w.median_yield_per_mill),
            ("min_yield_per_mill", w.min_yield_per_mill),
            ("max_yield_per_mill", w.max_yield_per_mill),
            ("median_sd1_value_per_pupil", w.median_sd1_value_per_pupil),
            ("median_regime_difference", w.median_regime_difference),
        ] {
            s.push_str(&format!("    \"{key}\": {},\n", num(value)));
        }
        for (key, value) in [
            ("below_charge_off_rate", w.below_charge_off_rate),
            (
                "charge_off_exceeds_base_cost",
                w.charge_off_exceeds_base_cost,
            ),
        ] {
            s.push_str(&format!("    \"{key}\": {value},\n"));
        }
        s.push_str(&format!(
            "    \"at_minimum_state_share\": {},\n",
            w.at_minimum_state_share
        ));
        s.push_str(&format!(
            "    \"median_valuation_per_pupil\": {},\n",
            num(w.median_valuation_per_pupil)
        ));
        s.push_str(&format!(
            "    \"median_operating_expenditure_per_pupil\": {},\n",
            num(w.median_operating_expenditure_per_pupil)
        ));
        s.push_str(&format!(
            "    \"wealth_neutrality_formula\": {},\n",
            num(w.wealth_neutrality_formula)
        ));
        s.push_str(&format!(
            "    \"wealth_neutrality_realized\": {},\n",
            num(w.wealth_neutrality_realized)
        ));
        s.push_str(&format!(
            "    \"guarantee_total\": {},\n",
            num(w.guarantee_total)
        ));
        s.push_str(&format!(
            "    \"realized_aid_total\": {},\n",
            num(w.realized_aid_total)
        ));
        s.push_str(&format!(
            "    \"minimum_state_share\": {},\n",
            num(w.minimum_state_share)
        ));
        s.push_str("    \"finances\": [");
        s.push_str(
            &w.finances
                .iter()
                .map(finance_year)
                .collect::<Vec<_>>()
                .join(", "),
        );
        s.push_str("],\n");
        match &w.outcomes {
            None => s.push_str("    \"outcomes\": null\n"),
            Some(o) => s.push_str(&format!(
                "    \"outcomes\": {{\"districts\": {}, \"poverty_vs_performance\": {}, \
                 \"guarantee_vs_performance\": {}, \
                 \"guarantee_vs_performance_controlled\": {}, \
                 \"spending_vs_growth_controlled\": {}, \
                 \"weighted_spending_vs_performance\": {}, \
                 \"enrolled_spending_vs_performance\": {}, \
                 \"median_performance_on_guarantee\": {}, \
                 \"median_performance_on_formula\": {}, \
                 \"median_federal_share\": {}, \"max_federal_share\": {}, \
                 \"federal_share_above_tenth\": {}, \
                 \"federal_share_vs_performance\": {}, \
                 \"federal_share_vs_performance_raw\": {}, \
                 \"growth_measures_disagree\": {}, \
                 \"growth_measures_determinate\": {}, \
                 \"growth_measures_disagree_materially\": {}, \
                 \"growth_measure_agreement\": {}}}\n",
                o.districts,
                num(o.poverty_vs_performance),
                num(o.guarantee_vs_performance),
                num(o.guarantee_vs_performance_controlled),
                num(o.spending_vs_growth_controlled),
                num(o.weighted_spending_vs_performance),
                num(o.enrolled_spending_vs_performance),
                num(o.median_performance_on_guarantee),
                num(o.median_performance_on_formula),
                num(o.median_federal_share),
                num(o.max_federal_share),
                o.federal_share_above_tenth,
                num(o.federal_share_vs_performance),
                num(o.federal_share_vs_performance_raw),
                o.growth_measures_disagree,
                o.growth_measures_determinate,
                o.growth_measures_disagree_materially,
                num(o.growth_measure_agreement),
            )),
        }
        s.push_str("  },\n");

        s.push_str("  \"checkpoints\": [\n");
        for (i, c) in self.checkpoints.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"label\": \"{}\", \"policy\": {{\"guarantee\": \"{}\", \
                 \"guarantee_argument\": {}, \"base_cost_scale\": {}, \
                 \"minimum_state_share\": {}, \"phase_in_base_cost\": {}, \
                 \"phase_in_categorical\": {}}}, \"cost\": {}, \"realized_aid\": {}, \
                 \"gainers\": {}, \"losers\": {}, \"unmoved\": {}, \"on_guarantee\": {}}}",
                escape(&c.label),
                escape(c.policy.guarantee),
                num(c.policy.guarantee_argument),
                num(c.policy.base_cost_scale),
                num(c.policy.minimum_state_share),
                num(c.policy.phase_in_base_cost),
                num(c.policy.phase_in_categorical),
                num(c.cost),
                num(c.realized_aid),
                c.gainers,
                c.losers,
                c.unmoved,
                c.on_guarantee
            ));
            if i + 1 < self.checkpoints.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        match &self.projection {
            None => s.push_str("  \"projection\": null,\n"),
            Some(p) => {
                s.push_str("  \"projection\": {\n");
                s.push_str(&format!("    \"base_year\": {},\n", p.base_year));
                s.push_str(&format!("    \"horizon\": {},\n", p.horizon));
                s.push_str(&format!("    \"method\": \"{}\",\n", escape(&p.method)));
                s.push_str(&format!("    \"damping\": {},\n", num(p.damping)));
                // Six places, not the four `num` gives: sigma is a growth rate around 0.02, and
                // rounding it to 0.0234 would move a ten-year band by enough to fail its own
                // checkpoint.
                s.push_str(&format!("    \"sigma\": {:.6},\n", p.sigma));
                s.push_str(&format!("    \"z\": {},\n", num(p.z)));
                s.push_str(&format!(
                    "    \"prior_source\": \"{}\",\n",
                    escape(&p.prior_source)
                ));
                s.push_str("    \"checkpoints\": [\n");
                for (i, c) in p.checkpoints.iter().enumerate() {
                    s.push_str(&format!(
                        "      {{\"label\": \"{}\", \"policy\": {{\"guarantee\": \"{}\", \
                         \"guarantee_argument\": {}, \"base_cost_scale\": {}, \
                         \"minimum_state_share\": {}, \"phase_in_base_cost\": {}, \
                         \"phase_in_categorical\": {}}}, \"fiscal_year\": {}, \
                         \"realized_aid\": {}, \"low\": {}, \"high\": {}, \"adm\": {}, \
                         \"on_guarantee\": {}}}",
                        escape(&c.label),
                        escape(c.policy.guarantee),
                        num(c.policy.guarantee_argument),
                        num(c.policy.base_cost_scale),
                        num(c.policy.minimum_state_share),
                        num(c.policy.phase_in_base_cost),
                        num(c.policy.phase_in_categorical),
                        c.fiscal_year,
                        num(c.realized_aid),
                        num(c.low),
                        num(c.high),
                        num(c.adm),
                        c.on_guarantee
                    ));
                    if i + 1 < p.checkpoints.len() {
                        s.push(',');
                    }
                    s.push('\n');
                }
                s.push_str("    ]\n  },\n");
            }
        }

        match &self.deflator {
            None => s.push_str("  \"deflator\": null,\n"),
            Some(deflator) => s.push_str(&format!(
                "  \"deflator\": {{\"label\": \"{}\", \"points\": [{}]}},\n",
                escape(&deflator.label),
                deflator
                    .points
                    .iter()
                    .map(|(year, index)| format!(
                        "{{\"fiscal_year\": {year}, \"index\": {}}}",
                        num(*index)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }

        match &self.national {
            None => s.push_str("  \"national\": null,\n"),
            Some(n) => {
                s.push_str(&format!(
                    "  \"national\": {{\"fiscal_year\": {}, \"ohio_local_rank\": {}, \
                     \"ohio_state_rank\": {}, \"ohio_spending_rank\": {}, \
                     \"ohio_property_tax_rank\": {}, \"independent_states\": {}, \
                     \"national_local_share\": {}, \"national_state_share\": {}, \
                     \"national_spending_per_pupil\": {}, \"states\": [",
                    n.fiscal_year,
                    n.ohio_local_rank,
                    n.ohio_state_rank,
                    n.ohio_spending_rank,
                    n.ohio_property_tax_rank,
                    n.independent_states,
                    num(n.national_local_share),
                    num(n.national_state_share),
                    num(n.national_spending_per_pupil),
                ));
                s.push_str(
                    &n.states
                        .iter()
                        .map(|state| {
                            let mut row = format!(
                                "{{\"fips\": \"{}\", \"name\": \"{}\", \"systems\": {}",
                                escape(&state.fips),
                                escape(&state.name),
                                state.systems
                            );
                            for (key, value) in [
                                ("enrollment", state.enrollment),
                                ("total_revenue", state.total_revenue),
                                ("federal_revenue", state.federal_revenue),
                                ("state_revenue", state.state_revenue),
                                ("local_revenue", state.local_revenue),
                                ("property_tax_revenue", state.property_tax_revenue),
                                ("parent_government_revenue", state.parent_government_revenue),
                                ("current_spending", state.current_spending),
                            ] {
                                row.push_str(&format!(", \"{key}\": {}", num(value)));
                            }
                            row.push('}');
                            row
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                s.push_str("]},\n");
            }
        }

        s.push_str("  \"districts\": [\n");

        for (i, d) in self.districts.iter().enumerate() {
            s.push_str("    {");
            s.push_str(&format!("\"irn\": \"{}\", ", escape(&d.irn)));
            s.push_str(&format!("\"name\": \"{}\", ", escape(&d.name)));
            s.push_str(&format!("\"adm\": {}, ", num(d.adm)));
            s.push_str(&format!(
                "\"current_year_adm\": {}, ",
                num(d.current_year_adm)
            ));
            s.push_str(&format!(
                "\"base_cost_per_pupil\": {}, ",
                num(d.base_cost_per_pupil)
            ));
            s.push_str(&format!(
                "\"aggregate_base_cost\": {}, ",
                num(d.aggregate_base_cost)
            ));
            {
                // Two tax years of the Department of Taxation's own table, written as an array so
                // the change between them — which is the whole reason two are carried — is a
                // thing the page can iterate rather than two parallel field sets.
                s.push_str("\"property_tax\": [");
                for (j, y) in d.property_tax.iter().enumerate() {
                    if j > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&format!("{{\"tax_year\": {}, ", y.tax_year));
                    for (name, value) in [
                        ("class1_value", y.class1_value),
                        ("class2_value", y.class2_value),
                        ("public_utility_value", y.public_utility_value),
                        ("total_value", y.total_value),
                        ("agricultural_value", y.agricultural_value),
                        ("residential_value", y.residential_value),
                        ("commercial_value", y.commercial_value),
                        ("industrial_value", y.industrial_value),
                        ("mineral_value", y.mineral_value),
                        ("railroad_value", y.railroad_value),
                        ("class1_rate", y.class1_rate),
                        ("class2_rate", y.class2_rate),
                        ("class1_taxes_charged", y.class1_taxes_charged),
                        ("class2_taxes_charged", y.class2_taxes_charged),
                        ("real_property_taxes_charged", y.real_property_taxes_charged),
                        (
                            "public_utility_taxes_charged",
                            y.public_utility_taxes_charged,
                        ),
                        ("value_per_pupil", y.value_per_pupil),
                        ("adm", y.adm),
                    ] {
                        s.push_str(&format!("\"{name}\": {}, ", num(value)));
                    }
                    s.truncate(s.trim_end_matches(' ').trim_end_matches(',').len());
                    s.push('}');
                }
                s.push_str("], ");
            }
            match &d.spending_by_function {
                None => s.push_str("\"spending_by_function\": null, "),
                Some(f) => {
                    s.push_str("\"spending_by_function\": {");
                    for (name, value) in [
                        ("adm", f.adm),
                        ("operating_per_pupil", f.operating_per_pupil),
                        ("classroom_instruction", f.classroom_instruction),
                        ("nonclassroom", f.nonclassroom),
                        ("instruction", f.instruction),
                        ("pupil_support", f.pupil_support),
                        ("instructional_staff_support", f.instructional_staff_support),
                        ("general_admin", f.general_admin),
                        ("school_admin", f.school_admin),
                        ("operations_maintenance", f.operations_maintenance),
                        ("pupil_transportation", f.pupil_transportation),
                        ("other_support", f.other_support),
                        ("food_service", f.food_service),
                    ] {
                        s.push_str(&format!("\"{name}\": {}, ", num(value)));
                    }
                    s.truncate(s.trim_end_matches(' ').trim_end_matches(',').len());
                    s.push_str("}, ");
                }
            }
            {
                // Twenty-two elements plus the two funded-position counts and the reconciliation
                // against the department's own figure. Written longhand for the same reason the
                // rest of this serializer is: no serde, so no derive.
                let b = &d.base_cost_build_up;
                s.push_str("\"base_cost_build_up\": {");
                for (name, value) in [
                    ("classroom_teachers", b.classroom_teachers),
                    ("special_teachers", b.special_teachers),
                    ("substitutes", b.substitutes),
                    ("professional_development", b.professional_development),
                    ("teachers", b.teachers),
                    ("counselors", b.counselors),
                    ("librarians", b.librarians),
                    ("wellness", b.wellness),
                    ("academic_cocurricular", b.academic_cocurricular),
                    ("safety", b.safety),
                    ("supplies", b.supplies),
                    ("technology", b.technology),
                    ("student_support", b.student_support),
                    ("superintendent", b.superintendent),
                    ("treasurer", b.treasurer),
                    ("other_administrators", b.other_administrators),
                    ("fiscal_support", b.fiscal_support),
                    ("emis", b.emis),
                    ("leadership_support", b.leadership_support),
                    ("itc", b.itc),
                    ("district_leadership", b.district_leadership),
                    ("building_leadership_staff", b.building_leadership_staff),
                    ("building_support", b.building_support),
                    ("building_operation", b.building_operation),
                    ("building_leadership", b.building_leadership),
                    ("athletic_cocurricular", b.athletic_cocurricular),
                    ("computed_aggregate", b.computed_aggregate),
                    ("published_aggregate", b.published_aggregate),
                    ("residual", b.residual),
                ] {
                    s.push_str(&format!("\"{name}\": {}, ", num(value)));
                }
                s.push_str(&format!(
                    "\"funded_classroom_teachers\": {}, \"funded_special_teachers\": {}}}, ",
                    num(b.funded_classroom_teachers),
                    num(b.funded_special_teachers)
                ));
            }
            s.push_str(&format!(
                "\"base_cost_state_share\": {}, ",
                num(d.base_cost_state_share)
            ));
            s.push_str(&format!(
                "\"categorical_funding\": {}, ",
                num(d.categorical_funding)
            ));
            s.push_str(&format!(
                "\"formula_aid_per_pupil\": {}, ",
                num(d.formula_aid_per_pupil)
            ));
            s.push_str(&format!(
                "\"realized_aid_per_pupil\": {}, ",
                num(d.realized_aid_per_pupil)
            ));
            s.push_str(&format!("\"guarantee\": {}, ", num(d.guarantee)));
            s.push_str(&format!("\"on_guarantee\": {}, ", d.on_guarantee()));
            s.push_str(&format!("\"at_millage_floor\": {}, ", d.at_millage_floor()));
            s.push_str(&format!(
                "\"near_millage_floor\": {}, ",
                d.near_millage_floor()
            ));
            s.push_str(&format!(
                "\"at_minimum_state_share\": {}, ",
                d.at_minimum_state_share
            ));
            s.push_str(&format!(
                "\"valuation_per_pupil\": {}, ",
                opt(d.valuation_per_pupil)
            ));
            s.push_str(&format!(
                "\"effective_class1_millage\": {}, ",
                opt(d.effective_class1_millage)
            ));
            s.push_str(&format!(
                "\"voted_operating_millage\": {}, ",
                opt(d.voted_operating_millage)
            ));
            s.push_str("\"millage\": ");
            match &d.millage {
                None => s.push_str("null, "),
                Some(m) => {
                    s.push('{');
                    s.push_str(&format!("\"tax_year\": {}, ", m.tax_year));
                    for (key, value) in [
                        ("prior_rate", m.prior_rate),
                        ("observed_rate", m.observed_rate),
                        ("predicted_rate", m.predicted_rate),
                        ("residual", m.residual),
                        ("yield_per_mill_per_pupil", m.yield_per_mill_per_pupil),
                    ] {
                        s.push_str(&format!("\"{key}\": {}, ", num(value)));
                    }
                    s.push_str(&format!("\"at_floor\": {}, ", m.at_floor));
                    s.push_str(&format!(
                        "\"cumulative_reduction\": {}",
                        opt(m.cumulative_reduction)
                    ));
                    s.push_str("}, ");
                }
            }
            s.push_str("\"regime\": ");
            match &d.regime {
                None => s.push_str("null, "),
                Some(r) => {
                    s.push('{');
                    s.push_str(&format!(
                        "\"charge_off_mills\": {}, ",
                        num(r.charge_off_mills)
                    ));
                    for (key, value) in [
                        ("charge_off_local_share", r.charge_off_local_share),
                        ("local_capacity", r.local_capacity),
                        ("aid_charge_off", r.aid_charge_off),
                        ("aid_fsfp", r.aid_fsfp),
                        ("difference", r.difference),
                        ("residual", r.residual),
                        ("mills_short_of_charge_off", r.mills_short_of_charge_off),
                    ] {
                        s.push_str(&format!("\"{key}\": {}, ", opt(value)));
                    }
                    s.push_str(&format!("\"exceeds_base_cost\": {}", r.exceeds_base_cost));
                    s.push_str("}, ");
                }
            }
            s.push_str(&format!(
                "\"operating_expenditure_per_pupil\": {}, ",
                opt(d.operating_expenditure_per_pupil)
            ));
            s.push_str(&format!(
                "\"economically_disadvantaged\": {}, ",
                opt(d.economically_disadvantaged)
            ));
            s.push_str(&format!(
                "\"enrollment_change\": {}, ",
                opt(d.enrollment_change)
            ));
            s.push_str(&format!(
                "\"adm_history\": [{}, {}, {}], ",
                num(d.adm_history[0]),
                num(d.adm_history[1]),
                num(d.adm_history[2])
            ));
            match &d.outcome {
                None => s.push_str("\"outcome\": null"),
                Some(o) => s.push_str(&format!(
                    "\"outcome\": {{\"performance_index\": {}, \
                     \"performance_index_prior\": {}, \
                     \"performance_index_earliest\": {}, \
                     \"progress_effect_size\": {}, \
                     \"progress_effect_size_one_year\": {}, \"per_enrolled_pupil\": {}, \
                     \"per_equivalent_pupil\": {}, \
                     \"per_equivalent_pupil_federal\": {}, \
                     \"per_equivalent_pupil_state_local\": {}, \
                     \"economically_disadvantaged\": {}, \
                     \"english_learner\": {}, \"students_with_disabilities\": {}}}",
                    opt(o.performance_index),
                    opt(o.performance_index_prior),
                    opt(o.performance_index_earliest),
                    opt(o.progress_effect_size),
                    opt(o.progress_effect_size_one_year),
                    opt(o.per_enrolled_pupil),
                    opt(o.per_equivalent_pupil),
                    opt(o.per_equivalent_pupil_federal),
                    opt(o.per_equivalent_pupil_state_local),
                    opt(o.economically_disadvantaged),
                    opt(o.english_learner),
                    opt(o.students_with_disabilities),
                )),
            }
            s.push_str(", \"finances\": [");
            s.push_str(
                &d.finances
                    .iter()
                    .map(finance_year)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            s.push(']');
            s.push('}');
            if i + 1 < self.districts.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ]\n}\n");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> District {
        District {
            irn: "049056".into(),
            name: "Northern Local".into(),
            adm: 2_193.81,
            current_year_adm: 2_107.80,
            base_cost_per_pupil: 8_100.0,
            aggregate_base_cost: 17_769_861.0,
            // The serializer writes every element; the values do not matter to what this asserts,
            // which is that the shape reaches the JSON.
            base_cost_build_up: BaseCostBuildUp {
                published_aggregate: 17_769_861.0,
                computed_aggregate: 17_769_860.5,
                residual: -0.5,
                ..BaseCostBuildUp::default()
            },
            // Two tax years, because the serializer writes an array and a single-element one
            // would not exercise the separator between them.
            property_tax: vec![
                PropertyTaxYear {
                    tax_year: 2023,
                    class1_rate: 20.0,
                    ..PropertyTaxYear::default()
                },
                PropertyTaxYear {
                    tax_year: 2024,
                    class1_rate: 20.0154,
                    ..PropertyTaxYear::default()
                },
            ],
            spending_by_function: Some(SpendingByFunction {
                operating_per_pupil: 14_027.17,
                ..SpendingByFunction::default()
            }),
            base_cost_state_share: 6_000_000.0,
            categorical_funding: 8_038_562.0,
            formula_aid_per_pupil: 6_400.0,
            realized_aid_per_pupil: 6_400.0,
            guarantee: 0.0,
            at_minimum_state_share: false,
            valuation_per_pupil: Some(279_983.24),
            effective_class1_millage: Some(20.0),
            voted_operating_millage: Some(34.9),
            // Northern Local is one of the 75 districts that crossed 20.0000 between the two tax
            // years, which makes it the right fixture: it is at the floor on the profile's TY2023
            // figure and a hundredth of a mill above it on SD-1's TY2024 one.
            millage: Some(MillageAnalysis {
                tax_year: 2024,
                prior_rate: 20.0,
                observed_rate: 20.0154,
                predicted_rate: 20.0,
                residual: 0.0154,
                at_floor: true,
                cumulative_reduction: Some(0.4269),
                yield_per_mill_per_pupil: 227.35,
            }),
            // 23 mills against $279,983 of valuation is $6,440 — more than half of what the
            // charge-off would have deemed Northern Local able to raise toward its own cost.
            regime: Some(RegimeCounterfactual {
                charge_off_mills: 23.0,
                charge_off_local_share: Some(6_439.61),
                local_capacity: Some(5_263.44),
                aid_charge_off: Some(1_660.39),
                aid_fsfp: Some(2_836.56),
                difference: Some(1_176.17),
                residual: Some(0.0),
                exceeds_base_cost: false,
                mills_short_of_charge_off: Some(2.9846),
            }),
            operating_expenditure_per_pupil: Some(11_986.62),
            economically_disadvantaged: Some(0.3881),
            enrollment_change: Some(-0.03),
            adm_history: [2_173.0, 2_140.0, 2_107.8],
            finances: vec![FinanceYear {
                fiscal_year: 2025,
                state_aid: 10_252_524.0,
                local_tax: 6_000_000.0,
                total_revenue: 21_000_000.0,
                total_expenditure: 22_000_000.0,
                ending_cash: 7_500_000.0,
            }],
            outcome: Some(DistrictOutcome {
                performance_index: Some(89.9),
                performance_index_prior: Some(89.1),
                performance_index_earliest: Some(88.4),
                progress_effect_size: Some(0.0),
                per_enrolled_pupil: Some(14_512.0),
                progress_effect_size_one_year: Some(0.31),
                per_equivalent_pupil: Some(11_986.62),
                // 4.2% federal, the statewide median, and the two parts add to the whole.
                per_equivalent_pupil_federal: Some(503.44),
                per_equivalent_pupil_state_local: Some(11_483.18),
                economically_disadvantaged: Some(38.8),
                english_learner: Some(0.4),
                students_with_disabilities: Some(15.2),
            }),
        }
    }

    fn zero_statewide() -> Statewide {
        Statewide {
            districts: 1,
            on_guarantee: 0,
            at_millage_floor: 1,
            near_millage_floor: 0,
            median_voted_millage: 0.0,
            median_effective_millage: 0.0,
            median_millage_reduction: 0.0,
            median_yield_per_mill: 0.0,
            min_yield_per_mill: 0.0,
            max_yield_per_mill: 0.0,
            median_sd1_value_per_pupil: 0.0,
            below_charge_off_rate: 0,
            charge_off_exceeds_base_cost: 0,
            median_regime_difference: 0.0,
            at_minimum_state_share: 0,
            median_valuation_per_pupil: 0.0,
            median_operating_expenditure_per_pupil: 0.0,
            wealth_neutrality_formula: 0.0,
            wealth_neutrality_realized: 0.0,
            guarantee_total: 0.0,
            realized_aid_total: 0.0,
            minimum_state_share: 0.1,
            finances: vec![FinanceYear {
                fiscal_year: 2025,
                state_aid: 7_890_000_000.0,
                local_tax: 11_000_000_000.0,
                total_revenue: 25_090_000_000.0,
                total_expenditure: 27_600_000_000.0,
                ending_cash: 9_140_000_000.0,
            }],
            outcomes: Some(OutcomeStatewide {
                districts: 606,
                poverty_vs_performance: -0.846,
                guarantee_vs_performance: 0.187,
                guarantee_vs_performance_controlled: 0.035,
                spending_vs_growth_controlled: 0.146,
                weighted_spending_vs_performance: -0.015,
                enrolled_spending_vs_performance: -0.337,
                median_performance_on_guarantee: 89.9,
                median_performance_on_formula: 85.6,
                median_federal_share: 0.042,
                max_federal_share: 0.29,
                federal_share_above_tenth: 47,
                federal_share_vs_performance: -0.11,
                federal_share_vs_performance_raw: -0.58,
                growth_measures_disagree: 44,
                growth_measures_determinate: 534,
                growth_measures_disagree_materially: 0,
                growth_measure_agreement: 0.904,
            }),
        }
    }

    fn bundle(districts: Vec<District>, checkpoints: Vec<Checkpoint>) -> Bundle {
        Bundle {
            contract_version: CONTRACT_VERSION.into(),
            provenance: "test".into(),
            fiscal_year: 2027,
            statewide: zero_statewide(),
            checkpoints,
            projection: None,
            deflator: None,
            national: None,
            districts,
        }
    }

    fn projection() -> Projection {
        Projection {
            base_year: 2026,
            horizon: 2036,
            method: "damped".into(),
            damping: 0.85,
            sigma: 0.023_456_7,
            z: 1.0,
            prior_source: "cross-sectional spread of district annual enrolled-ADM growth".into(),
            checkpoints: vec![ForecastCheckpoint {
                label: "current law, FY2032".into(),
                policy: checkpoint().policy,
                fiscal_year: 2032,
                realized_aid: 7_100_000_000.0,
                low: 6_860_000_000.0,
                high: 7_350_000_000.0,
                adm: 1_500_000.0,
                on_guarantee: 320,
            }],
        }
    }

    fn checkpoint() -> Checkpoint {
        Checkpoint {
            label: "guarantee removed".into(),
            policy: PolicyShape {
                guarantee: "removed",
                guarantee_argument: 0.0,
                base_cost_scale: 1.0,
                minimum_state_share: 0.1,
                phase_in_base_cost: 1.0,
                phase_in_categorical: 1.0,
            },
            cost: -879_000_000.0,
            realized_aid: 6_402_000_000.0,
            gainers: 0,
            losers: 294,
            unmoved: 315,
            on_guarantee: 0,
        }
    }

    #[test]
    fn a_district_with_no_guarantee_is_on_formula() {
        assert!(!sample().on_guarantee());
    }

    /// A district with no SD-1 block falls back to the profile's effective rate.
    fn without_sd1(effective: Option<f64>) -> District {
        District {
            millage: None,
            property_tax: Vec::new(),
            effective_class1_millage: effective,
            ..sample()
        }
    }

    #[test]
    fn exactly_twenty_mills_counts_as_the_floor() {
        assert!(without_sd1(Some(20.0)).at_millage_floor());
        assert!(!without_sd1(Some(37.09)).at_millage_floor());
        assert!(!without_sd1(None).at_millage_floor());
    }

    /// The bug this contract version exists for. Six districts never voted twenty mills of
    /// current operating levy, so reduction factors have nothing to reduce; comparing their rate
    /// to a literal `20.0` for equality reported them as being above the floor with the factors
    /// operative, which is the reverse of their position.
    #[test]
    fn a_rate_below_twenty_mills_is_at_the_floor_not_above_it() {
        // Vinton County Local: 18.70 voted, 18.70 effective, reduction factor zero.
        let vinton = without_sd1(Some(18.7));
        assert!(
            vinton.at_millage_floor(),
            "a district charging 18.70 mills cannot be above a twenty-mill floor"
        );
        assert!(
            !vinton.near_millage_floor(),
            "it is at the floor, not near it"
        );
    }

    /// The floor is the crate's, not a number written here — so a change to the statute is a
    /// change in one place.
    #[test]
    fn the_floor_comes_from_the_millage_crate() {
        let floor = millage::floor_for(edfund_core::AgencyType::City).expect("a school district");
        assert!(without_sd1(Some(floor)).at_millage_floor());
        assert!(!without_sd1(Some(floor + 1.0)).at_millage_floor());
        assert_eq!(
            millage::floor_for(edfund_core::AgencyType::JointVocational),
            Some(2.0),
            "the JVSD floor differs, which is why this is not a literal"
        );
    }

    /// Where the binary stops carrying information. The fixture is Northern Local, which sits at
    /// the floor on the profile's TY2023 rate and 0.0154 mills above it on SD-1's TY2024 one.
    #[test]
    fn a_hundredth_of_a_mill_above_the_floor_is_counted_as_near_it() {
        let northern = sample();
        assert!(!northern.at_millage_floor());
        assert!(northern.near_millage_floor());

        let clearly_above = District {
            millage: Some(MillageAnalysis {
                observed_rate: 24.71,
                ..northern.millage.expect("the fixture has one")
            }),
            ..sample()
        };
        assert!(!clearly_above.at_millage_floor());
        assert!(!clearly_above.near_millage_floor());
    }

    /// SD-1 is the later observation and two departments disagree about 75 districts, so the
    /// classification has to say which one it is using.
    #[test]
    fn sd1_outranks_the_profile_where_both_have_a_rate() {
        let conflicting = District {
            effective_class1_millage: Some(20.0),
            millage: Some(MillageAnalysis {
                observed_rate: 25.31,
                ..sample().millage.expect("the fixture has one")
            }),
            ..sample()
        };
        assert!(
            !conflicting.at_millage_floor(),
            "the profile says floor and SD-1 says 25.31 mills; SD-1 is the later observation"
        );
    }

    #[test]
    fn the_fy2020_baseline_is_only_recoverable_on_the_guarantee() {
        assert_eq!(sample().implied_fy2020_baseline_per_pupil(), None);
        let guaranteed = District {
            guarantee: 1_000_000.0,
            realized_aid_per_pupil: 7_100.0,
            ..sample()
        };
        assert_eq!(
            guaranteed.implied_fy2020_baseline_per_pupil(),
            Some(7_100.0)
        );
    }

    #[test]
    fn json_escapes_quotes_and_backslashes_in_district_names() {
        let odd = District {
            name: r#"St. "Mary" \ Local"#.into(),
            ..sample()
        };
        assert!(bundle(vec![odd], vec![])
            .to_json()
            .contains(r#"St. \"Mary\" \\ Local"#));
    }

    #[test]
    fn missing_values_serialize_as_null_not_zero() {
        let sparse = District {
            valuation_per_pupil: None,
            effective_class1_millage: None,
            operating_expenditure_per_pupil: None,
            economically_disadvantaged: None,
            enrollment_change: None,
            ..sample()
        };
        let json = bundle(vec![sparse], vec![]).to_json();
        assert!(json.contains("\"valuation_per_pupil\": null"));
        assert!(
            !json.contains("\"valuation_per_pupil\": 0"),
            "a missing value must not be indistinguishable from zero"
        );
    }

    #[test]
    fn serialization_is_deterministic() {
        let b = bundle(vec![sample(), sample()], vec![checkpoint()]);
        assert_eq!(b.to_json(), b.to_json());
    }

    #[test]
    fn the_property_tax_years_survive_serialization_in_order() {
        // The page reads the pair as a change, so a reversed or collapsed array would invert every
        // direction it reports rather than failing visibly.
        let json = bundle(vec![sample()], vec![]).to_json();
        let start = json
            .find("\"property_tax\": [")
            .expect("the array is written");
        // Bounded by the array's own close rather than a byte count: one year's block runs to
        // several hundred characters, and a short slice would only ever find the first.
        let end = start + json[start..].find(']').expect("the array closes");
        let block = &json[start..end];
        let first = block.find("\"tax_year\": 2023").expect("the earlier year");
        let second = block.find("\"tax_year\": 2024").expect("the later year");
        assert!(first < second, "tax years are not oldest first: {block}");
    }

    #[test]
    fn a_district_without_a_spending_row_serializes_as_null() {
        // Two of the 609 have none. Writing zeros would be a claim about their spending rather
        // than about the file, and the page needs to be able to tell the difference.
        let mut district = sample();
        district.spending_by_function = None;
        let json = bundle(vec![district], vec![]).to_json();
        assert!(json.contains("\"spending_by_function\": null"), "{json}");
    }

    #[test]
    fn the_bundle_declares_its_contract_version() {
        // Against the constant rather than a literal. A hard-coded version here means a bump has
        // to be made in two places, and the one that gets forgotten is the test — which then fails
        // for the right reason at the wrong moment, long after the change that caused it.
        assert!(bundle(vec![], vec![])
            .to_json()
            .contains(&format!("\"contract_version\": \"{CONTRACT_VERSION}\"")));
    }

    #[test]
    fn a_feed_without_a_projection_says_null_rather_than_omitting_the_key() {
        // A consumer must be able to tell "this feed cannot be projected" from "this feed is
        // from a build that predates projection". The first disables a band; the second is a
        // contract mismatch and should have been caught by the version guard.
        assert!(bundle(vec![], vec![])
            .to_json()
            .contains("\"projection\": null"));
    }

    #[test]
    fn the_projection_block_carries_its_method_and_the_prior_the_band_rests_on() {
        let b = Bundle {
            projection: Some(projection()),
            ..bundle(vec![sample()], vec![checkpoint()])
        };
        let json = b.to_json();
        assert!(json.contains("\"method\": \"damped\""));
        assert!(json.contains("\"damping\": 0.85"));
        assert!(json.contains("\"base_year\": 2026"));
        assert!(json.contains("cross-sectional spread"));
    }

    #[test]
    fn sigma_keeps_six_places_because_four_would_move_a_ten_year_band() {
        // `num` rounds to four, which turns 0.0234567 into 0.0235 — a 0.2% shift in the half
        // width at a ten-year horizon, which is enough to fail the checkpoint it exists to pass.
        let json = Bundle {
            projection: Some(projection()),
            ..bundle(vec![], vec![])
        }
        .to_json();
        assert!(json.contains("\"sigma\": 0.023457"), "{json}");
    }

    #[test]
    fn a_forecast_checkpoint_carries_both_ends_of_its_band() {
        // A point with no interval is the thing this whole axis exists to not ship.
        let json = Bundle {
            projection: Some(projection()),
            ..bundle(vec![], vec![])
        }
        .to_json();
        assert!(json.contains("\"realized_aid\": 7100000000"));
        assert!(json.contains("\"low\": 6860000000"));
        assert!(json.contains("\"high\": 7350000000"));
        assert!(json.contains("\"fiscal_year\": 2032"));
    }

    #[test]
    fn every_district_carries_the_three_years_the_projection_is_fitted_from() {
        // Not nullable: a district without a history cannot be projected, and a page that
        // silently dropped it would report a statewide total over a subset of the panel.
        let json = bundle(vec![sample()], vec![]).to_json();
        assert!(
            json.contains("\"adm_history\": [2173, 2140, 2107.8]"),
            "{json}"
        );
    }

    #[test]
    fn a_district_without_a_report_card_serializes_a_null_outcome() {
        // Three districts have none. `null` rather than an object of nulls, so a consumer can
        // tell "no report card" from "a report card with nothing in it".
        let none = District {
            outcome: None,
            ..sample()
        };
        let json = bundle(vec![none], vec![]).to_json();
        assert!(json.contains("\"outcome\": null"));
        assert!(!json.contains("\"performance_index\""));
    }

    #[test]
    fn the_outcome_block_carries_both_spending_denominators() {
        // The corpus's central denominator finding is the gap between them. Shipping one would
        // make it unstateable in the interface meant to explain it.
        let json = bundle(vec![sample()], vec![]).to_json();
        assert!(json.contains("\"per_enrolled_pupil\": 14512"));
        assert!(json.contains("\"per_equivalent_pupil\": 11986.62"));
    }

    #[test]
    fn the_statewide_outcomes_carry_the_raw_and_the_controlled_figure() {
        // A page showing +0.187 without +0.035 beside it would be stating the confound as a
        // finding, which is the specific thing this axis was built to prevent.
        let json = bundle(vec![], vec![]).to_json();
        assert!(json.contains("\"guarantee_vs_performance\": 0.187"));
        assert!(json.contains("\"guarantee_vs_performance_controlled\": 0.035"));
    }

    #[test]
    fn checkpoints_carry_the_policy_that_produced_them() {
        let json = bundle(vec![], vec![checkpoint()]).to_json();
        assert!(json.contains("\"guarantee\": \"removed\""));
        assert!(json.contains("\"cost\": -879000000"));
        assert!(json.contains("\"unmoved\": 315"));
    }

    #[test]
    fn an_empty_checkpoint_list_still_produces_valid_json() {
        assert!(bundle(vec![sample()], vec![])
            .to_json()
            .contains("\"checkpoints\": [\n  ],"));
    }

    #[test]
    fn the_scenario_inputs_are_present_for_every_district() {
        // The web layer cannot re-derive a policy without these four.
        let json = bundle(vec![sample()], vec![]).to_json();
        for field in [
            "aggregate_base_cost",
            "base_cost_state_share",
            "categorical_funding",
            "current_year_adm",
        ] {
            assert!(json.contains(field), "{field} missing from the feed");
        }
    }
}

/// One state's school finance, from the Census Bureau's Annual Survey of School System Finances.
///
/// A third source, and a federal one. Everything else in this feed comes from Ohio describing
/// itself; the corpus has been able to say what Ohio does and never whether it is unusual.
#[derive(Debug, Clone, PartialEq)]
pub struct StateFinance {
    /// Two-digit FIPS.
    pub fips: String,
    /// State name, or the District of Columbia.
    pub name: String,
    /// School systems with enrolment.
    pub systems: usize,
    /// Fall enrolment, a headcount.
    pub enrollment: f64,
    /// Total revenue, in thousands of dollars as the survey reports it.
    pub total_revenue: Dollars,
    /// Federal revenue, thousands.
    pub federal_revenue: Dollars,
    /// State revenue, thousands.
    pub state_revenue: Dollars,
    /// Local revenue, thousands. Includes parent-government appropriations.
    pub local_revenue: Dollars,
    /// Local revenue from the district's own property tax, thousands. Zero where districts are
    /// dependent; see [`StateFinance::fiscally_independent`].
    pub property_tax_revenue: Dollars,
    /// Appropriations from a parent city or county, thousands.
    pub parent_government_revenue: Dollars,
    /// Current spending, thousands.
    pub current_spending: Dollars,
}

impl StateFinance {
    /// Whether this state's school districts levy their own tax rather than being funded by a
    /// parent government.
    ///
    /// The distinction that makes a property tax comparison possible or impossible. Twelve states
    /// fund schools mostly through a city or county appropriation, so the survey attributes the
    /// tax to the parent and reports the district's own property tax as zero. Massachusetts and
    /// Virginia raise as much from property tax as anywhere and score nothing.
    #[must_use]
    pub fn fiscally_independent(&self) -> bool {
        self.parent_government_revenue < self.local_revenue * 0.10
    }

    /// Local revenue as a share of total. Comparable across both district structures.
    #[must_use]
    pub fn local_share(&self) -> f64 {
        if self.total_revenue > 0.0 {
            self.local_revenue / self.total_revenue
        } else {
            0.0
        }
    }

    /// State revenue as a share of total.
    #[must_use]
    pub fn state_share(&self) -> f64 {
        if self.total_revenue > 0.0 {
            self.state_revenue / self.total_revenue
        } else {
            0.0
        }
    }

    /// Current spending per pupil, in dollars. The survey reports thousands.
    #[must_use]
    pub fn spending_per_pupil(&self) -> f64 {
        if self.enrollment > 0.0 {
            self.current_spending * 1_000.0 / self.enrollment
        } else {
            0.0
        }
    }
}

/// Where Ohio sits among the states, and the figures that put it there.
///
/// # What this settles that nothing else in the corpus could
///
/// The *DeRolph* holding was that Ohio relied too heavily on local property tax. Every figure the
/// corpus has held until now describes Ohio alone, so the claim could be restated and never
/// tested — "too heavily" needs a comparison, and there was nothing to compare against.
///
/// There is now. Ohio raises **51.8% of school revenue locally against a national 43.4%, seventh
/// highest of fifty-one**, and takes **34.4% from the state against a national 43.4%, forty-fifth
/// of fifty-one**. It spends about the national average per pupil and is exactly average on
/// federal money. The distinctive thing about Ohio is not how much its schools cost but who pays.
///
/// # The year, and why it flatters nothing
///
/// FY2022 is the peak of federal pandemic relief, so the federal share is inflated and the local
/// and state shares are correspondingly deflated. That runs against the finding rather than for
/// it: in an ordinary year Ohio's local share would be higher, not lower.
#[derive(Debug, Clone, PartialEq)]
pub struct National {
    /// The survey year, as a fiscal year.
    pub fiscal_year: u16,
    /// Every state and the District of Columbia, alphabetically.
    pub states: Vec<StateFinance>,
    /// Ohio's rank on local share, 1 being the highest, out of all 51.
    pub ohio_local_rank: usize,
    /// Ohio's rank on state share, 1 being the highest.
    pub ohio_state_rank: usize,
    /// Ohio's rank on current spending per pupil.
    pub ohio_spending_rank: usize,
    /// Ohio's rank on property tax share, among fiscally independent states only.
    pub ohio_property_tax_rank: usize,
    /// How many states that comparison is over.
    pub independent_states: usize,
    /// National local share of school revenue.
    pub national_local_share: f64,
    /// National state share.
    pub national_state_share: f64,
    /// National current spending per pupil.
    pub national_spending_per_pupil: f64,
}
