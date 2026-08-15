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
/// `33.0.0` extends `appropriations` back to **FY1998** and adds a third value to its `source`
/// field, `act`. Breaking on the enum: a consumer switching on `workbook` or `catalog` now meets a
/// third case. The four new years are read from the enrolled acts rather than from any analysis of
/// them, because the greenbook series begins with the 124th General Assembly and the Catalog
/// reaches FY2002. They stop at FY1998 because the legislature's own index of what it holds stops
/// at the 122nd.
///
/// The same revision corrects every enacted total from FY2002 to FY2013. `200906 Tangible Tax
/// Exemption - Education` is a tax reimbursement and was not excluded, so it sat inside the
/// department: $73,500,000 in FY2002, 0.94%, decaying to $10,707,622 by FY2009. Because it phases
/// out it inflated the early years more than the late ones, understating real growth.
///
/// `32.0.0` reworked `meal_program`. It now runs FY1998 through FY2014 rather than FY2001 through
/// FY2011, and `share` became **nullable**. Breaking in the strongest sense available here: a
/// consumer that reads the block the way `29.0.0` published it will find a null where it expects a
/// number, which is the intent. From FY2012 the report is published as three files — Traditional,
/// Provision 2 and Community Eligibility — and only the first still counts applications. Adding
/// them gives a share that falls thirteen points in three years because the poorest sponsors
/// stopped collecting forms, so those Octobers carry `floor` and `ceiling` and no share at all.
/// `streams` says which kind of year a row is.
///
/// The same revision corrects FY2001. The file is the only comma-delimited one in the series, nine
/// of its rows carry a comma inside a school name, and read positionally two of them put a site
/// IRN in Cleveland City's enrollment column — 192,147 pupils against its real 73,562. Every
/// figure `29.0.0` and later published for FY2001 was computed on that: the share was 27.7% and is
/// 29.5%.
///
/// `31.0.0` added `appropriation_lines`: the department's line items with the act that created
/// each, from the Catalog's `originally established by` clause. Additive in shape and breaking in
/// meaning, because it changes what a consumer can say about the budget — not how much it is but
/// how old it is, and roughly half the lines answer "unknown" rather than being filled from an
/// earlier edition that reused their number.
///
/// `30.0.0` added `appropriations`: what the General Assembly set aside for the department, by
/// fiscal year, FY2002 through FY2027, continuous for the first time. Breaking rather than
/// additive because it changes what the feed *is* in the same way `28.0.0` did — every other
/// figure here is an output of the funding system, and this is an input to it. An appropriation is
/// a ceiling and a payment is what was made; differencing the two produces a number that means
/// nothing, and the deflator now reaches FY2002 so the series can be read in constant dollars,
/// which is the only way it can be read at all.
///
/// `29.0.0` added `meal_program`: the free and reduced-price lunch share for FY2001 through
/// FY2011, from the Office for Child Nutrition's MR-81. Additive in shape and breaking in
/// meaning, on the same reasoning as `28.0.0` — this is now the third population and the third
/// enrollment count in one feed, and it is the only one whose denominator changes *inside* its
/// own series, at FY2010. Every row carries [`MealProgramYear::basis`] so a consumer cannot plot
/// the series as continuous without having been told it is not. It carries no dollars and is
/// absent from the deflator by design.
///
/// `28.0.0` added the historical axis: a `history` array carrying the Census Bureau's survey of
/// Ohio school systems for FY2009 through FY2022, and a `deflator` extended to cover it. Breaking
/// rather than additive because it changes what the feed *is*. Every other figure here is the Fair
/// School Funding Plan computing one year for the 609 traditional districts; these are roughly 950
/// agencies a year measured by somebody else on a different enrollment count, and the two do not
/// reconcile. A consumer that rendered them on one axis without saying which was which would be
/// making the mistake the block exists to let it avoid.
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
pub const CONTRACT_VERSION: &str = "34.0.0";

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
    /// The share of taxable value the charge-off reaches, after the reappraisal phase-in.
    ///
    /// One where the district's county has finished phasing in a revaluation, below one where it
    /// has not. This is what makes the counterfactual run on **recognized valuation** rather than
    /// on total taxable value, which is what the corpus wrongly used until it read the mechanism's
    /// actual definition. See `regime_diff::recognized_valuation`.
    pub recognized_share: f64,
    /// The tax year the district's county last reappraised or updated.
    pub reappraisal_year: u16,
    /// How much less the charge-off is on recognized valuation than on total taxable value.
    ///
    /// Zero for a district past its phase-in. The point of publishing it is that its size is
    /// decided by the county's place on the Department of Taxation's calendar and by nothing
    /// about the district itself.
    pub overstated_by: Option<Dollars>,
}

/// Special education's six categories for one district: pupils and the aid they generate.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpecialEducation {
    /// ADM in each category, 1 through 6.
    pub adm: [f64; 6],
    /// The aid each produces, after the district's state share.
    pub aid: [Dollars; 6],
}

/// Disadvantaged Pupil Impact Aid, for one district: a blend of two counts and a squared index.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dpia {
    /// FY2025 economically disadvantaged ADM, weighted 65%.
    pub economically_disadvantaged_adm: f64,
    /// FY2026 directly certified ADM, weighted 35%. Consistently the smaller of the two.
    pub directly_certified_adm: f64,
    /// The blend of the two.
    pub weighted_adm: f64,
    /// That blend as a share of enrolled ADM.
    pub percentage: f64,
    /// The share indexed against the statewide 0.5334, **squared**.
    pub index: f64,
}

/// Targeted assistance, for one district: two tiers that measure different things and add.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TargetedAssistance {
    /// Assessed valuation and federal adjusted gross income, the 60/40 halves of weighted wealth.
    pub property_valuation: Dollars,
    /// Federal adjusted gross income, the 40% half.
    pub federal_gross_income: Dollars,
    /// The blend of the two.
    pub weighted_wealth: Dollars,
    /// The median district's total wealth over this district's.
    pub capacity_index: f64,
    /// 0.8% of the shortfall below that median, phased by district size.
    pub capacity_amount: Dollars,
    /// Weighted wealth per **resident** pupil — enrolled, less open-enrolling in, plus out.
    pub wealth_per_pupil: Dollars,
    /// The median per pupil over this district's, so poorer scores higher.
    pub wealth_index: f64,
    /// A rate against wealth per pupil, paid on **enrolled** pupils. Zero below an index of 0.8.
    pub wealth_amount: Dollars,
    /// The count the wealth tier measures against, which is not the one it pays on.
    pub resident_adm: f64,
    /// Whether the district qualifies for the supplemental tier, which pays nothing.
    pub supplement_eligible: bool,
}

/// Career-technical education's five categories, plus associated services.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CareerTechnical {
    /// FTE in each category, 1 through 5.
    pub fte: [f64; 5],
    /// The aid each produces, after the district's state share.
    pub aid: [Dollars; 5],
    /// A sixth weight against the sum of all five FTE.
    pub associated_services: Dollars,
}

/// English learners' three categories, whose weights descend rather than ascend.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnglishLearners {
    /// ADM in each category, 1 through 3.
    pub adm: [f64; 3],
    /// The aid each produces, after the district's state share.
    pub aid: [Dollars; 3],
}

/// Gifted: two per-pupil amounts and three kinds of unit, with floors and a cap.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Gifted {
    /// $24 per K-6 pupil, after the state share.
    pub identification: Dollars,
    /// $2.50 per enrolled pupil, after the state share.
    pub referral: Dollars,
    /// Identified gifted FTE, which drive the specialist units.
    pub fte_k8: f64,
    /// Identified gifted FTE in grades 9-12.
    pub fte_9_12: f64,
    /// Units then dollars, for each of the three unit kinds.
    pub coordinator_units: f64,
    /// What the coordinator units pay.
    pub coordinator_aid: Dollars,
    /// Intervention specialist units for K-8, floored at 0.3.
    pub specialist_k8_units: f64,
    /// What those units pay.
    pub specialist_k8_aid: Dollars,
    /// Intervention specialist units for 9-12, floored at 0.3 and priced lower.
    pub specialist_9_12_units: f64,
    /// What those units pay.
    pub specialist_9_12_aid: Dollars,
    /// Whether every unit this district draws is a floor rather than an earned entitlement.
    pub entirely_on_the_floor: bool,
}

/// The six categorical programs, per district.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Categoricals {
    /// Equalisation for low-valuation districts. Zero for 135 of 609.
    pub targeted_assistance: Dollars,
    /// Six weighted categories of disability.
    pub special_education: Dollars,
    /// Disadvantaged Pupil Impact Aid, driven by the economically disadvantaged count.
    pub dpia: Dollars,
    /// Three weights by time in the country.
    pub english_learners: Dollars,
    /// Identification and service.
    pub gifted: Dollars,
    /// Career-technical education weights.
    pub career_technical: Dollars,
}

/// One school district's presence in one Ohio House district.
#[derive(Debug, Clone, PartialEq)]
pub struct HouseDistrictMember {
    /// The school district, by IRN.
    pub irn: String,
    /// Its name, carried so a page need not join back to the district array.
    pub name: String,
    /// How much of the *school district* lies in this House district.
    pub share: f64,
    /// How much of this *House district's* apportioned pupils this school district provides.
    pub share_of_house_district: f64,
    /// Apportioned pupils and aid for this pair. An estimate; see [`HouseDistrict`].
    pub adm: f64,
    /// Apportioned state aid as the district receives it.
    pub realized_aid: Dollars,
    /// Whether the school district lies entirely inside this House district.
    pub wholly_inside: bool,
}

/// One of Ohio's 99 House districts, with the school funding apportioned to it.
///
/// # These figures are estimates, and nothing in Ohio's system publishes them
///
/// The department computes funding per school district and stops. No House district is a unit of
/// account anywhere in the funding system, and 339 of 609 school districts straddle two or more of
/// them — so a House district total has to be *derived*, by splitting each school district's
/// figures across the House districts it overlaps in proportion to under-18 population from the
/// 2020 census.
///
/// The one guarantee is that the split is exact in aggregate: each school district's shares sum to
/// one, so the 99 House districts sum to the statewide total to the cent. Everything else about a
/// House district figure is an estimate, and any page showing one has to say so.
#[derive(Debug, Clone, PartialEq)]
pub struct HouseDistrict {
    /// `001` through `099`.
    pub number: String,
    /// The school districts in it, largest contributor first.
    pub members: Vec<HouseDistrictMember>,
    /// Apportioned enrolled ADM.
    pub adm: f64,
    /// Apportioned state aid as districts receive it, guarantee included.
    pub realized_aid: Dollars,
    /// Apportioned state share of base cost.
    pub base_cost_state_share: Dollars,
    /// Apportioned categorical funding — the other half of formula aid.
    pub categorical_funding: Dollars,
    /// Apportioned guarantee: what the formula does not justify, in this member's schools.
    pub guarantee: Dollars,
    /// Districts overlapping this House district that are on the guarantee.
    pub districts_on_guarantee: usize,
    /// And those at the minimum state share.
    pub districts_at_minimum_state_share: usize,
    /// Of the districts here, those that lie entirely inside this House district.
    pub districts_wholly_inside: usize,
}

/// Which House districts a school district lies in, and how much of it is in each.
#[derive(Debug, Clone, PartialEq)]
pub struct HouseDistrictShare {
    /// `001` through `099`.
    pub number: String,
    /// How much of the school district lies in that House district.
    pub share: f64,
}

/// The payments outside foundation funding, for one district.
///
/// `[H] Foundation Funding` is base cost plus the six categoricals, and the guarantee holds a
/// district at it. These sit in `[R] Total State Support` instead, so nothing cushions a fall in
/// either: a district that drops a star, or slips below 3% growth, loses the money outright.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Supplements {
    /// The overall star rating and the progress component rating. `None` where unrated.
    pub stars: Option<f64>,
    /// The progress component rating, which the payment uses when it is the higher of the two.
    pub progress: Option<f64>,
    /// Whether any of the three routes qualified the district, and what it was paid.
    pub performance_eligible: bool,
    /// $13 a pupil times the greater of the two ratings.
    pub performance: Dollars,
    /// $40 a pupil, every district, no test.
    pub base_funding: Dollars,
    /// The three-year enrolment change the 3% growth test is applied to.
    pub enrollment_change: f64,
    /// Whether the three-year change cleared 3%.
    pub growth_eligible: bool,
    /// $250 on every pupil, for a district that cleared 3%.
    pub growth: Dollars,
    /// What clearing it would have paid a district that did not. `None` where it did.
    pub growth_forgone: Option<Dollars>,
}

/// Transportation, for one district — the largest thing outside foundation funding.
///
/// $726m, plus $183m of special education transportation. Transportation alone is larger than
/// special education, making it the second-largest single program in Ohio's school funding after
/// targeted assistance, and it shares almost nothing with the formula: two competing rate bases
/// with the district paid the greater, a 50% state minimum share against the formula's 10%, two
/// supplements that reward opposite things, its own guarantee on a FY2021 base, and a proration
/// factor on the special education line meaning the appropriation did not cover the entitlement.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Transportation {
    /// Riders by the kind of school they attend. Non-public count double, community 1.5 times.
    pub public_riders: f64,
    /// Weighted double.
    pub nonpublic_riders: f64,
    /// Weighted one and a half.
    pub community_riders: f64,
    /// The three, weighted.
    pub weighted_riders: f64,
    /// What each of the two competing bases would pay before the state share.
    pub per_rider_base: Dollars,
    /// The other one.
    pub per_mile_base: Dollars,
    /// Whether the mile base is the one this district is actually paid on.
    pub paid_on_miles: bool,
    /// The state share actually applied, after the 50% floor.
    pub effective_state_share: f64,
    /// The five payments.
    pub school_bus: Dollars,
    /// Mass transit riders at 35% of the rider rate.
    pub mass_transit: Dollars,
    /// Other vehicle types at 50%.
    pub other: Dollars,
    /// Up to 15% more for filling buses.
    pub efficiency: Dollars,
    /// And a payment for not being able to.
    pub density: Dollars,
    /// Riders per bus over a capacity target.
    pub efficiency_index: f64,
    /// Riders per square mile.
    pub district_density: f64,
    /// A second transitional guarantee, on a FY2021 base.
    pub fy21_base: Dollars,
    /// What that base holds the district at.
    pub guarantee: Dollars,
    /// The total, and special education transportation beside it.
    pub total: Dollars,
    /// Prorated at 0.91746.
    pub special_education: Dollars,
    /// What it would have been without the proration.
    pub special_education_unprorated: Dollars,
}

/// Preschool special education, for one district.
///
/// A flat $4,000 a pupil whatever the category — 69% of the program, and not reduced by the state
/// share — plus the six school-age weights at half, all prorated. The proration is the point: the
/// sheet carries its appropriation limit beside the factor, and at the stated factor the program
/// runs $908,184 over it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PreschoolSpecialEducation {
    /// ADM in each category, 1 through 6.
    pub adm: [f64; 6],
    /// The aid each produces.
    pub aid: [Dollars; 6],
    /// The six, summed.
    pub total: Dollars,
    /// What the flat $4,000 component alone is worth.
    pub flat_component: Dollars,
    /// What the program would pay without the proration.
    pub unprorated: Dollars,
}

/// The guarantee's machinery, and the second hold-harmless stacked on it.
///
/// The guarantee is not "hold the district at its old amount": it is the FY2021 funding base less
/// an **open-enrolment clawback** less foundation funding. And a *second* hold-harmless sits above
/// it against a larger FY2021 base, one that includes transportation — reaching 17 districts the
/// guarantee does not.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Transition {
    /// The FY2021 amount the guarantee compares foundation funding against.
    pub funding_base: Dollars,
    /// Open enrolment FTE, last year and this.
    pub open_enrollment_prior: f64,
    /// This year.
    pub open_enrollment_current: f64,
    /// How much of a loss is absorbed before the clawback applies, and what it costs beyond it.
    pub open_enrollment_threshold: f64,
    /// What the loss beyond it costs the guarantee.
    pub open_enrollment_adjustment: Dollars,
    /// A FY2021 base that includes transportation, and the supplement it produces.
    pub fy21_funding_base: Dollars,
    /// What that larger base holds the district at.
    pub transition_supplement: Dollars,
}

/// Where a district sits among America's, on federal definitions.
///
/// Ohio describing itself cannot say whether Ohio is unusual, and every other source in this feed
/// is Ohio describing itself. This is the exception: 10,382 comparable school districts in every
/// state, reported on the Census Bureau's own definitions.
///
/// Three caveats travel with it. The year is **FY2022** against the model's FY2027. The
/// denominator is the **federal** fall membership, not Ohio's ADM. And the comparison set excludes
/// charter agencies and non-unified districts, because a community school's finances are not a
/// school district's — leaving them in put Ohio's smallest agencies at an 8% local share.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NationalPosition {
    /// Local revenue as a share of total, and where that sits among all comparable districts.
    pub local_share: f64,
    /// Where that share sits among all comparable districts.
    pub local_share_percentile: f64,
    /// Total revenue per pupil on the federal count, and its percentile.
    pub revenue_per_pupil: f64,
    /// Where that sits.
    pub revenue_per_pupil_percentile: f64,
    /// Current spending per pupil, and its percentile. Zero where unreported.
    pub spending_per_pupil: f64,
    /// And that.
    pub spending_per_pupil_percentile: f64,
}

/// One district, as the web layer needs it.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Information Retrieval Number, the stable statewide identifier.
    pub irn: String,
    /// District name as published.
    pub name: String,
    /// Where this district sits among America's. `None` for the one K-8 district the comparison
    /// set excludes, which is carried without a position rather than given an invented one.
    pub national: Option<NationalPosition>,
    /// The performance supplement and the two enrolment supplements, outside the formula.
    pub supplements: Supplements,
    /// Transportation, the largest thing outside it.
    pub transportation: Transportation,
    /// Preschool special education, the last line in the same gap.
    pub preschool_special_education: PreschoolSpecialEducation,
    /// The guarantee's machinery, and the transition supplement stacked on it.
    pub transition: Transition,
    /// The Ohio House districts this district lies in, largest share first.
    ///
    /// Usually one — 270 of 609 districts sit inside a single House district — and up to eleven.
    /// Derived from census blocks; see [`HouseDistrict`] for what that does and does not support.
    pub house_districts: Vec<HouseDistrictShare>,
    /// The county the department attributes the district to.
    ///
    /// One county per district, which is the department's own simplification: school district
    /// boundaries cross county lines freely and the calculator picks one anyway. Good enough to
    /// group peers by, and not good enough to sum into a figure called the county's.
    pub county: String,
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
    /// The part of `categorical_funding` priced in the statewide average base cost per pupil.
    ///
    /// Special education, English learners and career-technical, which are each
    /// `weight x $8,241.61 x count x state share`. A base cost lever moves these too, so the
    /// scenario needs them separated. Emitted rather than re-derived in the browser so that
    /// *which* programs count is decided once, in `project::panel`, and the two implementations
    /// of `apply` cannot disagree about it.
    pub base_cost_denominated_categoricals: Dollars,
    /// Special education's six weighted categories: ADM then aid, Category 1 through 6.
    ///
    /// The weights span a factor of sixteen and the money runs against them — Category 6 is 15%
    /// of pupils and 48% of the program, Category 2 the reverse.
    pub special_education: SpecialEducation,
    /// The other five, each decomposed to the mechanism that produces it.
    ///
    /// Reading these apart is the point. The six programs answer different questions and move for
    /// opposite reasons — targeted assistance rises as a district gets poorer in *property*, DPIA
    /// as its *pupils* get poorer, gifted barely moves at all because it is mostly a staffing
    /// floor. A page that shows six dollar amounts still cannot say which of those a district's
    /// money is.
    pub dpia: Dpia,
    /// The largest, and the only equalisation among the six.
    pub targeted_assistance: TargetedAssistance,
    /// Five weights against a career-technical base cost, plus associated services.
    pub career_technical: CareerTechnical,
    /// Three weights that descend rather than ascend.
    pub english_learners: EnglishLearners,
    /// Two per-pupil amounts and three kinds of unit, with floors no other categorical has.
    pub gifted: Gifted,
    /// `[a] Enrolled ADM` — the pupil count four of the six categoricals are paid on.
    ///
    /// Not [`District::adm`], which averages three years, and not [`District::current_year_adm`],
    /// which is `[b3] FY26 Enrolled ADM`. It equals the latter in 608 of 609 districts and differs
    /// in Akron by fifty pupils. Carried so a per-pupil figure computed here uses the denominator
    /// the department paid on.
    pub categorical_adm: f64,
    /// The same, as its six parts.
    ///
    /// The total was a residual for eight phases — core foundation funding less the state share of
    /// base cost, which is exact and uninterrogable. It is 43% of formula aid, and the six behave
    /// nothing alike: targeted assistance is equalisation and is zero for 135 districts, DPIA
    /// tracks poverty. A page showing the sum cannot say which a district's money is.
    pub categoricals: Categoricals,
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

    /// Above the floor, but by less than `NEAR_FLOOR_BAND` — where the binary stops meaning
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
    /// Districts receiving nothing from targeted assistance, the largest categorical program.
    ///
    /// It is equalisation: it switches off once a district has enough valuation per pupil. That
    /// the largest single program in the state reaches only four districts in five is invisible
    /// while the six categoricals are reported as one number.
    pub districts_without_targeted_assistance: usize,
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
    /// Districts the guarantee paid under **both** policies.
    ///
    /// Not the same as [`Checkpoint::unmoved`], and the gap between them is informative: a
    /// formula district can be unmoved because the lever pulled does not touch it, while a
    /// guarantee district is unmoved because nothing pulled can touch it until the formula
    /// overtakes its frozen baseline.
    pub held_throughout: usize,
    /// Districts the policy lifted off the guarantee onto the formula.
    pub lifted_off: usize,
    /// Districts the policy pushed from the formula onto the guarantee.
    pub pushed_on: usize,
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
    ///
    /// **The model's year, and not the page's.** A district page shows this beside a 2024 tax
    /// year, a 2024-25 report card, an FY2022 Census survey and a five-year forecast reaching back
    /// to FY2020. It is the year of the formula and of nothing else; see [`Bundle::series_years`]
    /// for what each other block is measured in.
    pub fiscal_year: u16,
    /// The year every other series in this feed is measured in, by series key.
    ///
    /// Ordered by key so a diff of the feed is readable. See [`SeriesYear`].
    pub series_years: Vec<SeriesYear>,
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
    /// The Census survey year by year, oldest first. Empty if the panel is absent.
    ///
    /// The only part of the feed that reaches before FY2020, and the only part measured on
    /// something other than the department's own formula. See [`HistoryYear`].
    pub history: Vec<HistoryYear>,
    /// The appropriation lines themselves, with the act that created each. Empty if absent.
    ///
    /// The current edition only, ordered by line item. See [`AppropriationLine`].
    pub appropriation_lines: Vec<AppropriationLine>,
    /// What the General Assembly appropriated, year by year, oldest first. Empty if absent.
    ///
    /// The only block in this feed that is an input to the funding system rather than an output
    /// of it. See [`AppropriationYear`].
    pub appropriations: Vec<AppropriationYear>,
    /// The meal-program poverty share, October by October, oldest first. Empty if absent.
    ///
    /// Reaches back further than [`Self::history`] — FY2001 against FY2009 — and on a third
    /// measurement again. See [`MealProgramYear`].
    pub meal_program: Vec<MealProgramYear>,
    /// Ohio's 99 House districts, with school funding apportioned across them.
    pub house_districts: Vec<HouseDistrict>,
    /// And its 33 Senate districts, each exactly three House districts.
    ///
    /// A less approximate view than the House one: seats three times larger mean 392 of 609
    /// school districts sit wholly inside a single Senate district, against 270 for the House.
    pub senate_districts: Vec<HouseDistrict>,
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

/// A fraction, to eight places.
///
/// [`num`] rounds to four, which is right for dollars and wrong for a share of one: a district
/// contributing 0.00060997 of a House district would be stored as 0.0006, and the shares of a
/// district split many ways would no longer sum to one in the feed even though they do in the
/// arithmetic that produced them. Eight places keeps the published figures self-consistent, so a
/// consumer adding them up gets the same answer this repository does.
fn share(v: f64) -> String {
    if v.is_finite() {
        format!("{v:.8}")
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

/// `"<name>": {"<first>": [...], "<second>": [...], …}` — two parallel arrays and any scalars.
///
/// Closed here rather than by the caller. The first version left it open so that callers with a
/// trailing field could add one before the brace, and three of the five call sites then forgot to
/// close it — which nested `career_technical` inside `special_education` and made `dpia` a child
/// of `english_learners`. An object that is only valid if the caller remembers something is a
/// worse interface than one that takes the something.
fn array_pair(
    name: &str,
    first: &str,
    a: &[f64],
    second: &str,
    b: &[f64],
    extra: &[(&str, f64)],
) -> String {
    let list = |xs: &[f64]| xs.iter().map(|x| num(*x)).collect::<Vec<_>>().join(", ");
    let mut body = format!("\"{first}\": [{}], \"{second}\": [{}]", list(a), list(b));
    for (key, value) in extra {
        body.push_str(&format!(", \"{key}\": {}", num(*value)));
    }
    format!("\"{name}\": {{{body}}}")
}

/// `"<name>": {"k": v, …}` — an object of numeric fields and any boolean flags. Also closed.
fn fields(name: &str, entries: &[(&str, f64)], flags: &[(&str, bool)]) -> String {
    let mut body = entries
        .iter()
        .map(|(key, value)| format!("\"{key}\": {}", num(*value)))
        .collect::<Vec<_>>()
        .join(", ");
    for (key, value) in flags {
        body.push_str(&format!(", \"{key}\": {value}"));
    }
    format!("\"{name}\": {{{body}}}")
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

        // Sorted on the way out rather than trusted to arrive sorted, so the committed feed is
        // byte-identical whatever order the caller assembled these in. Every fixture in this
        // repository has to rebuild identically from a clean checkout.
        let mut years: Vec<&SeriesYear> = self.series_years.iter().collect();
        years.sort_by(|a, b| a.series.cmp(&b.series));
        s.push_str("  \"series_years\": [\n");
        for (i, y) in years.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"series\": \"{}\", \"kind\": \"{}\", \"label\": \"{}\", \"source\": \"{}\"}}",
                escape(&y.series),
                y.kind.as_str(),
                escape(&y.label),
                escape(&y.source)
            ));
            if i + 1 < years.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

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
            (
                "districts_without_targeted_assistance",
                w.districts_without_targeted_assistance,
            ),
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
                 \"gainers\": {}, \"losers\": {}, \"unmoved\": {}, \"on_guarantee\": {}, \
                 \"held_throughout\": {}, \"lifted_off\": {}, \"pushed_on\": {}}}",
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
                c.on_guarantee,
                c.held_throughout,
                c.lifted_off,
                c.pushed_on
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

        // One line per year, oldest first, because the thing a reader is looking for here is the
        // shape across years rather than any single one.
        s.push_str("  \"history\": [\n");
        for (i, h) in self.history.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"fiscal_year\": {}, \"districts\": {}",
                h.fiscal_year, h.districts
            ));
            for (key, value) in [
                ("local_share", h.local_share),
                ("state_share", h.state_share),
                ("federal_share", h.federal_share),
                ("poorest_local_per_pupil", h.poorest_local_per_pupil),
                ("richest_local_per_pupil", h.richest_local_per_pupil),
                ("gap_per_pupil", h.gap_per_pupil),
                ("state_closes_per_pupil", h.state_closes_per_pupil),
                ("federal_closes_per_pupil", h.federal_closes_per_pupil),
            ] {
                s.push_str(&format!(", \"{key}\": {}", num(value)));
            }
            s.push('}');
            if i + 1 < self.history.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        // `basis` is written on every row rather than only where it changes, because a consumer
        // reading one row must be able to tell what its share divides by without scanning for the
        // last row that said so. `streams` is written on every row for the stronger version of the
        // same reason: from FY2012 `share` is null, and a consumer meeting that has to be able to
        // tell a year with no single answer from a year whose figure went missing.
        s.push_str("  \"meal_program\": [\n");
        for (i, m) in self.meal_program.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"fiscal_year\": {}, \"sponsors\": {}, \"enrollment\": {}, \
                 \"approved\": {}, \"identified\": {}, \"share\": {}, \"floor\": {}, \
                 \"ceiling\": {}, \"without_applications\": {}, \"streams\": {}, \
                 \"basis\": \"{}\"}}",
                m.fiscal_year,
                m.sponsors,
                num(m.enrollment),
                num(m.approved),
                num(m.identified),
                m.share.map_or_else(|| "null".to_string(), num),
                num(m.floor),
                num(m.ceiling),
                num(m.without_applications),
                m.streams,
                escape(&m.basis)
            ));
            if i + 1 < self.meal_program.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        // `source` on every row rather than only where it changes, for the same reason
        // `meal_program` writes its basis on every row: a consumer reading one row must be able to
        // tell which document it came from without scanning back for the last row that said.
        s.push_str("  \"appropriations\": [\n");
        for (i, a) in self.appropriations.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"fiscal_year\": {}, \"enacted\": {}, \"foundation_funding\": {}, \
                 \"items\": {}, \"source\": \"{}\"}}",
                a.fiscal_year,
                num(a.enacted),
                num(a.foundation_funding),
                a.items,
                escape(&a.source)
            ));
            if i + 1 < self.appropriations.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        // `established_by` is written even when empty, and `general_assembly` as `null` rather
        // than omitted, so a consumer can tell "this line names no founding act" from "this feed
        // predates the field".
        s.push_str("  \"appropriation_lines\": [\n");
        for (i, l) in self.appropriation_lines.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"fund\": \"{}\", \"ali\": \"{}\", \"name\": \"{}\", \
                 \"established_by\": \"{}\", \"general_assembly\": {}, \"convened\": {}, \
                 \"discontinued\": {}}}",
                escape(&l.fund),
                escape(&l.ali),
                escape(&l.name),
                escape(&l.established_by),
                l.general_assembly
                    .map_or("null".to_string(), |v| v.to_string()),
                l.convened.map_or("null".to_string(), |v| v.to_string()),
                l.discontinued
            ));
            if i + 1 < self.appropriation_lines.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        // The two chambers. Written before the district array because a reader scanning the
        // feed meets the derived aggregate before the exact per-district figures it came from, and
        // the block's own `basis` field is where the estimate is labelled as one.
        for (key, seats) in [
            ("house_districts", &self.house_districts),
            ("senate_districts", &self.senate_districts),
        ] {
            s.push_str(&format!("  \"{key}\": [\n"));
            for (i, h) in seats.iter().enumerate() {
                s.push_str(&format!("    {{\"number\": \"{}\", ", escape(&h.number)));
                for (key, value) in [
                    ("adm", h.adm),
                    ("realized_aid", h.realized_aid),
                    ("base_cost_state_share", h.base_cost_state_share),
                    ("categorical_funding", h.categorical_funding),
                    ("guarantee", h.guarantee),
                ] {
                    s.push_str(&format!("\"{key}\": {}, ", num(value)));
                }
                s.push_str(&format!(
                    "\"districts_on_guarantee\": {}, \"districts_at_minimum_state_share\": {}, \
                 \"districts_wholly_inside\": {}, \"members\": [",
                    h.districts_on_guarantee,
                    h.districts_at_minimum_state_share,
                    h.districts_wholly_inside
                ));
                for (k, m) in h.members.iter().enumerate() {
                    if k > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&format!(
                        "{{\"irn\": \"{}\", \"name\": \"{}\", \"share\": {}, \
                     \"share_of_house_district\": {}, \"adm\": {}, \"realized_aid\": {}, \
                     \"wholly_inside\": {}}}",
                        escape(&m.irn),
                        escape(&m.name),
                        share(m.share),
                        share(m.share_of_house_district),
                        num(m.adm),
                        num(m.realized_aid),
                        m.wholly_inside
                    ));
                }
                s.push_str("]}");
                if i + 1 < seats.len() {
                    s.push(',');
                }
                s.push('\n');
            }
            s.push_str("  ],\n");
        }

        s.push_str("  \"districts\": [\n");

        for (i, d) in self.districts.iter().enumerate() {
            s.push_str("    {");
            s.push_str(&format!("\"irn\": \"{}\", ", escape(&d.irn)));
            s.push_str(&format!("\"name\": \"{}\", ", escape(&d.name)));
            s.push_str(&format!("\"county\": \"{}\", ", escape(&d.county)));
            s.push_str(&fields(
                "supplements",
                &[
                    ("performance", d.supplements.performance),
                    ("base_funding", d.supplements.base_funding),
                    ("growth", d.supplements.growth),
                    ("enrollment_change", d.supplements.enrollment_change),
                ],
                &[
                    ("performance_eligible", d.supplements.performance_eligible),
                    ("growth_eligible", d.supplements.growth_eligible),
                ],
            ));
            s.truncate(s.len() - 1);
            s.push_str(&format!(
                ", \"stars\": {}, \"progress\": {}, \"growth_forgone\": {}}}, ",
                opt(d.supplements.stars),
                opt(d.supplements.progress),
                opt(d.supplements.growth_forgone)
            ));
            match &d.national {
                None => s.push_str("\"national\": null, "),
                Some(n) => {
                    s.push_str(&fields(
                        "national",
                        &[
                            ("local_share", n.local_share),
                            ("local_share_percentile", n.local_share_percentile),
                            ("revenue_per_pupil", n.revenue_per_pupil),
                            (
                                "revenue_per_pupil_percentile",
                                n.revenue_per_pupil_percentile,
                            ),
                            ("spending_per_pupil", n.spending_per_pupil),
                            (
                                "spending_per_pupil_percentile",
                                n.spending_per_pupil_percentile,
                            ),
                        ],
                        &[],
                    ));
                    s.push_str(", ");
                }
            }
            s.push_str(&fields(
                "transition",
                &[
                    ("funding_base", d.transition.funding_base),
                    ("open_enrollment_prior", d.transition.open_enrollment_prior),
                    (
                        "open_enrollment_current",
                        d.transition.open_enrollment_current,
                    ),
                    (
                        "open_enrollment_threshold",
                        d.transition.open_enrollment_threshold,
                    ),
                    (
                        "open_enrollment_adjustment",
                        d.transition.open_enrollment_adjustment,
                    ),
                    ("fy21_funding_base", d.transition.fy21_funding_base),
                    ("transition_supplement", d.transition.transition_supplement),
                ],
                &[],
            ));
            s.push_str(", ");
            s.push_str(&array_pair(
                "preschool_special_education",
                "adm",
                &d.preschool_special_education.adm,
                "aid",
                &d.preschool_special_education.aid,
                &[
                    ("total", d.preschool_special_education.total),
                    (
                        "flat_component",
                        d.preschool_special_education.flat_component,
                    ),
                    ("unprorated", d.preschool_special_education.unprorated),
                ],
            ));
            s.push_str(", ");
            s.push_str(&fields(
                "transportation",
                &[
                    ("public_riders", d.transportation.public_riders),
                    ("nonpublic_riders", d.transportation.nonpublic_riders),
                    ("community_riders", d.transportation.community_riders),
                    ("weighted_riders", d.transportation.weighted_riders),
                    ("per_rider_base", d.transportation.per_rider_base),
                    ("per_mile_base", d.transportation.per_mile_base),
                    (
                        "effective_state_share",
                        d.transportation.effective_state_share,
                    ),
                    ("school_bus", d.transportation.school_bus),
                    ("mass_transit", d.transportation.mass_transit),
                    ("other", d.transportation.other),
                    ("efficiency", d.transportation.efficiency),
                    ("density", d.transportation.density),
                    ("efficiency_index", d.transportation.efficiency_index),
                    ("district_density", d.transportation.district_density),
                    ("fy21_base", d.transportation.fy21_base),
                    ("guarantee", d.transportation.guarantee),
                    ("total", d.transportation.total),
                    ("special_education", d.transportation.special_education),
                    (
                        "special_education_unprorated",
                        d.transportation.special_education_unprorated,
                    ),
                ],
                &[("paid_on_miles", d.transportation.paid_on_miles)],
            ));
            s.push_str(", ");
            s.push_str("\"house_districts\": [");
            for (i, h) in d.house_districts.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(&format!(
                    "{{\"number\": \"{}\", \"share\": {}}}",
                    escape(&h.number),
                    share(h.share)
                ));
            }
            s.push_str("], ");
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
                "\"categorical_funding\": {}, \"base_cost_denominated_categoricals\": {}, ",
                num(d.categorical_funding),
                num(d.base_cost_denominated_categoricals)
            ));
            // Special education, then the other five, then the six totals. Emitted once — this
            // block and the categoricals beside it were pasted twice, so every district in the
            // shipped feed carried both keys twice. JSON takes the last, so nothing was wrong on
            // the page and about 120KB of the payload was a copy of itself.
            s.push_str(&array_pair(
                "special_education",
                "adm",
                &d.special_education.adm,
                "aid",
                &d.special_education.aid,
                &[],
            ));
            s.push_str(", ");
            s.push_str(&array_pair(
                "career_technical",
                "fte",
                &d.career_technical.fte,
                "aid",
                &d.career_technical.aid,
                &[(
                    "associated_services",
                    d.career_technical.associated_services,
                )],
            ));
            s.push_str(", ");
            s.push_str(&array_pair(
                "english_learners",
                "adm",
                &d.english_learners.adm,
                "aid",
                &d.english_learners.aid,
                &[],
            ));
            s.push_str(", ");
            s.push_str(&fields(
                "dpia",
                &[
                    (
                        "economically_disadvantaged_adm",
                        d.dpia.economically_disadvantaged_adm,
                    ),
                    ("directly_certified_adm", d.dpia.directly_certified_adm),
                    ("weighted_adm", d.dpia.weighted_adm),
                    ("percentage", d.dpia.percentage),
                    ("index", d.dpia.index),
                ],
                &[],
            ));
            s.push_str(", ");
            s.push_str(&fields(
                "targeted_assistance",
                &[
                    (
                        "property_valuation",
                        d.targeted_assistance.property_valuation,
                    ),
                    (
                        "federal_gross_income",
                        d.targeted_assistance.federal_gross_income,
                    ),
                    ("weighted_wealth", d.targeted_assistance.weighted_wealth),
                    ("capacity_index", d.targeted_assistance.capacity_index),
                    ("capacity_amount", d.targeted_assistance.capacity_amount),
                    ("wealth_per_pupil", d.targeted_assistance.wealth_per_pupil),
                    ("wealth_index", d.targeted_assistance.wealth_index),
                    ("wealth_amount", d.targeted_assistance.wealth_amount),
                    ("resident_adm", d.targeted_assistance.resident_adm),
                ],
                &[(
                    "supplement_eligible",
                    d.targeted_assistance.supplement_eligible,
                )],
            ));
            s.push_str(", ");
            s.push_str(&fields(
                "gifted",
                &[
                    ("identification", d.gifted.identification),
                    ("referral", d.gifted.referral),
                    ("fte_k8", d.gifted.fte_k8),
                    ("fte_9_12", d.gifted.fte_9_12),
                    ("coordinator_units", d.gifted.coordinator_units),
                    ("coordinator_aid", d.gifted.coordinator_aid),
                    ("specialist_k8_units", d.gifted.specialist_k8_units),
                    ("specialist_k8_aid", d.gifted.specialist_k8_aid),
                    ("specialist_9_12_units", d.gifted.specialist_9_12_units),
                    ("specialist_9_12_aid", d.gifted.specialist_9_12_aid),
                ],
                &[("entirely_on_the_floor", d.gifted.entirely_on_the_floor)],
            ));
            s.push_str(&format!(
                ", \"categorical_adm\": {}, ",
                num(d.categorical_adm)
            ));
            s.push_str(&fields(
                "categoricals",
                &[
                    ("targeted_assistance", d.categoricals.targeted_assistance),
                    ("special_education", d.categoricals.special_education),
                    ("dpia", d.categoricals.dpia),
                    ("english_learners", d.categoricals.english_learners),
                    ("gifted", d.categoricals.gifted),
                    ("career_technical", d.categoricals.career_technical),
                ],
                &[],
            ));
            s.push_str(", ");
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
                    s.push_str(&format!(
                        "\"recognized_share\": {}, \"reappraisal_year\": {}, ",
                        share(r.recognized_share),
                        r.reappraisal_year
                    ));
                    for (key, value) in [
                        ("charge_off_local_share", r.charge_off_local_share),
                        ("local_capacity", r.local_capacity),
                        ("aid_charge_off", r.aid_charge_off),
                        ("aid_fsfp", r.aid_fsfp),
                        ("difference", r.difference),
                        ("residual", r.residual),
                        ("mills_short_of_charge_off", r.mills_short_of_charge_off),
                        ("overstated_by", r.overstated_by),
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
            county: "Perry".into(),
            national: Some(NationalPosition {
                local_share: 0.4123,
                local_share_percentile: 0.6104,
                revenue_per_pupil: 16_402.0,
                revenue_per_pupil_percentile: 0.5512,
                spending_per_pupil: 15_118.0,
                spending_per_pupil_percentile: 0.5308,
            }),
            transition: Transition {
                funding_base: 12_400_000.0,
                open_enrollment_prior: 214.5,
                open_enrollment_current: 96.4,
                open_enrollment_threshold: 21.45,
                open_enrollment_adjustment: 800_412.0,
                fy21_funding_base: 13_100_000.0,
                transition_supplement: 41_900.0,
            },
            preschool_special_education: PreschoolSpecialEducation {
                adm: [6.4, 15.2, 1.0, 0.0, 0.0, 1.0],
                aid: [31_800.0, 78_100.0, 5_900.0, 0.0, 0.0, 8_400.0],
                total: 124_200.0,
                flat_component: 90_900.0,
                unprorated: 128_236.0,
            },
            transportation: Transportation {
                public_riders: 812.0,
                nonpublic_riders: 41.0,
                community_riders: 18.0,
                weighted_riders: 921.0,
                per_rider_base: 1_231_538.0,
                per_mile_base: 1_402_119.0,
                paid_on_miles: true,
                effective_state_share: 0.5,
                school_bus: 701_059.5,
                mass_transit: 0.0,
                other: 4_812.0,
                efficiency: 62_190.0,
                density: 91_411.0,
                efficiency_index: 1.2044,
                district_density: 11.7,
                fy21_base: 812_004.0,
                guarantee: 0.0,
                total: 859_472.5,
                special_education: 118_204.0,
                special_education_unprorated: 128_838.0,
            },
            supplements: Supplements {
                stars: Some(4.0),
                progress: Some(3.0),
                performance_eligible: true,
                performance: 70_236.0,
                base_funding: 84_312.0,
                enrollment_change: -0.0412,
                growth_eligible: false,
                growth: 0.0,
                growth_forgone: Some(527_000.0),
            },
            // Two House districts, unevenly split, so the serializer's array separator is
            // exercised and a district that straddles a boundary is the case under test.
            house_districts: vec![
                HouseDistrictShare {
                    number: "094".into(),
                    share: 0.7312,
                },
                HouseDistrictShare {
                    number: "072".into(),
                    share: 0.2688,
                },
            ],
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
            // Special education, English learners and career-technical of the above — the part a
            // base cost lever moves along with base cost.
            base_cost_denominated_categoricals: 2_370_119.0,
            special_education: SpecialEducation {
                adm: [10.9, 105.2, 6.0, 1.0, 10.8, 7.1],
                aid: [21_000.0, 320_000.0, 44_000.0, 9_800.0, 143_000.0, 138_000.0],
            },
            // Each of the five decompositions, with distinguishable values in every slot: the
            // serializer writes arrays and nested objects, and a fixture of zeroes would let a
            // transposed pair through.
            dpia: Dpia {
                economically_disadvantaged_adm: 1_050.25,
                directly_certified_adm: 640.5,
                weighted_adm: 906.84,
                percentage: 0.4302,
                index: 0.6504,
            },
            targeted_assistance: TargetedAssistance {
                property_valuation: 210_000_000.0,
                federal_gross_income: 190_000_000.0,
                weighted_wealth: 202_000_000.0,
                capacity_index: 1.9413,
                capacity_amount: 1_520_000.0,
                wealth_per_pupil: 95_800.25,
                wealth_index: 2.8884,
                wealth_amount: 1_580_000.0,
                resident_adm: 2_108.5,
                supplement_eligible: true,
            },
            career_technical: CareerTechnical {
                fte: [40.5, 22.25, 8.0, 4.5, 1.25],
                aid: [180_000.0, 78_000.0, 10_000.0, 4_800.0, 1_100.0],
                associated_services: 26_100.0,
            },
            english_learners: EnglishLearners {
                adm: [6.5, 3.25, 1.5],
                aid: [7_800.0, 2_900.0, 1_300.0],
            },
            gifted: Gifted {
                identification: 24_500.0,
                referral: 4_200.0,
                fte_k8: 61.0,
                fte_9_12: 28.5,
                coordinator_units: 0.6387,
                coordinator_aid: 41_100.0,
                specialist_k8_units: 0.4357,
                specialist_k8_aid: 29_200.0,
                specialist_9_12_units: 0.3,
                specialist_9_12_aid: 18_200.0,
                entirely_on_the_floor: false,
            },
            categorical_adm: 2_107.80,
            categoricals: Categoricals {
                targeted_assistance: 3_100_000.0,
                special_education: 2_100_000.0,
                dpia: 2_300_000.0,
                english_learners: 12_000.0,
                gifted: 226_562.0,
                career_technical: 300_000.0,
            },
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
                // Perry County reappraised in TY2023, so a third of that revaluation is still
                // deferred and the charge-off reaches 92.0% of the district's taxable value —
                // $517 per pupil it is therefore not asked for. The real figures for the real
                // district, so this fixture cannot drift into describing a place that does not
                // exist.
                recognized_share: 0.91965761,
                reappraisal_year: 2023,
                overstated_by: Some(517.374),
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
            districts_without_targeted_assistance: 135,
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
            // Two entries and two reckonings, so the emitter's sort and its `kind` discriminant
            // are both exercised by the fixture every other test in this module builds on.
            series_years: vec![
                SeriesYear {
                    series: "millage".into(),
                    kind: YearKind::Tax,
                    label: "2024".into(),
                    source: "Table SD-1".into(),
                },
                SeriesYear {
                    series: "formula".into(),
                    kind: YearKind::Fiscal,
                    label: "FY2027".into(),
                    source: "DEW FY27 calculator".into(),
                },
            ],
            senate_districts: vec![HouseDistrict {
                number: "031".into(),
                adm: 4_812.3,
                realized_aid: 30_795_000.0,
                base_cost_state_share: 18_300_000.0,
                categorical_funding: 12_495_000.0,
                guarantee: 0.0,
                districts_on_guarantee: 1,
                districts_at_minimum_state_share: 2,
                districts_wholly_inside: 1,
                members: vec![HouseDistrictMember {
                    irn: "049056".into(),
                    name: "Northern Local".into(),
                    share: 1.0,
                    share_of_house_district: 1.0,
                    adm: 4_812.3,
                    realized_aid: 30_795_000.0,
                    wholly_inside: true,
                }],
            }],
            house_districts: vec![HouseDistrict {
                number: "094".into(),
                adm: 1_604.1,
                realized_aid: 10_265_000.0,
                base_cost_state_share: 6_100_000.0,
                categorical_funding: 4_165_000.0,
                guarantee: 0.0,
                districts_on_guarantee: 0,
                districts_at_minimum_state_share: 1,
                districts_wholly_inside: 0,
                members: vec![HouseDistrictMember {
                    irn: "049056".into(),
                    name: "Northern Local".into(),
                    share: 0.7312,
                    share_of_house_district: 1.0,
                    adm: 1_604.1,
                    realized_aid: 10_265_000.0,
                    wholly_inside: false,
                }],
            }],
            contract_version: CONTRACT_VERSION.into(),
            provenance: "test".into(),
            fiscal_year: 2027,
            statewide: zero_statewide(),
            checkpoints,
            projection: None,
            deflator: None,
            national: None,
            history: Vec::new(),
            // Two years across the source change, so anything serializing this fixture carries
            // both labels rather than one.
            // One dated line and one undated, so a serializer that omitted `null` rather than
            // writing it would fail here.
            appropriation_lines: vec![
                AppropriationLine {
                    fund: "GRF".into(),
                    ali: "200502".into(),
                    name: "Pupil Transportation".into(),
                    established_by: "H.B. 191 of the 112th G.A.".into(),
                    general_assembly: Some(112),
                    convened: Some(1977),
                    discontinued: false,
                },
                AppropriationLine {
                    fund: "GRF".into(),
                    ali: "200321".into(),
                    name: "Operating Expenses".into(),
                    established_by: String::new(),
                    general_assembly: None,
                    convened: None,
                    discontinued: false,
                },
            ],
            appropriations: vec![
                AppropriationYear {
                    fiscal_year: 2013,
                    enacted: 9_322_046_458.0,
                    foundation_funding: 6_349_290_686.0,
                    items: 115,
                    source: "catalog".into(),
                },
                AppropriationYear {
                    fiscal_year: 2014,
                    enacted: 9_871_965_322.0,
                    foundation_funding: 6_547_098_389.0,
                    items: 109,
                    source: "workbook".into(),
                },
            ],
            // Three rows spanning both breaks: the basis change, so anything serializing this
            // fixture has to carry both names rather than one, and the split into three files,
            // so it has to carry a row whose share is absent rather than zero.
            meal_program: vec![
                MealProgramYear {
                    fiscal_year: 2009,
                    sponsors: 812,
                    enrollment: 1_000_000.0,
                    approved: 412_000.0,
                    identified: 0.0,
                    share: Some(0.412),
                    floor: 0.412,
                    ceiling: 0.412,
                    without_applications: 0.0,
                    streams: 1,
                    basis: "adm".into(),
                },
                MealProgramYear {
                    fiscal_year: 2010,
                    sponsors: 844,
                    enrollment: 1_000_000.0,
                    approved: 437_000.0,
                    identified: 0.0,
                    share: Some(0.437),
                    floor: 0.437,
                    ceiling: 0.437,
                    without_applications: 0.0,
                    streams: 1,
                    basis: "ce".into(),
                },
                MealProgramYear {
                    fiscal_year: 2014,
                    sponsors: 901,
                    enrollment: 1_000_000.0,
                    approved: 333_000.0,
                    identified: 105_000.0,
                    share: None,
                    floor: 0.438,
                    ceiling: 0.484,
                    without_applications: 0.166,
                    streams: 3,
                    basis: "ce".into(),
                },
            ],
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
            held_throughout: 294,
            lifted_off: 0,
            pushed_on: 0,
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

    /// No key appears twice inside one district object.
    ///
    /// `special_education` and `categoricals` were each emitted twice per district for several
    /// contract versions. Nothing was visibly wrong — JSON takes the last occurrence, and both
    /// copies were identical — so about 120KB of every 3.4MB feed was a copy of itself and no test
    /// noticed. Duplicate keys are also the shape a genuine bug takes when two branches both write
    /// a field and only one is right.
    #[test]
    fn a_district_object_repeats_no_key() {
        let json = bundle(vec![sample()], vec![checkpoint()]).to_json();
        let start = json.find("\"districts\": [").expect("a districts array");
        let district = &json[start..];

        // Scan the first district object only, tracking brace depth so nested objects do not end
        // it early. Keys at depth one are the district's own.
        let mut depth = 0_i32;
        let mut seen: Vec<&str> = Vec::new();
        let bytes = district.as_bytes();
        let mut i = district.find('{').expect("an object");
        while i < bytes.len() {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                b'"' if depth == 1 => {
                    let rest = &district[i + 1..];
                    if let Some(end) = rest.find('"') {
                        let key = &rest[..end];
                        // A key is followed by a colon; a string *value* is not.
                        if district[i + 1 + end + 1..].starts_with(':') {
                            assert!(
                                !seen.contains(&key),
                                "the district object emits \"{key}\" twice"
                            );
                            seen.push(key);
                        }
                        i += end + 1;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        assert!(
            seen.len() > 20,
            "expected to have walked a full district, saw {} keys: {seen:?}",
            seen.len()
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
    fn every_meal_program_row_names_the_count_it_divides_by() {
        // The denominator changes inside this series, at FY2010. A row that does not carry its
        // own basis is a row a consumer will plot against the one before it, and the whole reason
        // the block is safe to publish is that it refuses to let that happen silently.
        let json = bundle(vec![], vec![]).to_json();
        // The counts are asserted with the share so the row stays self-checking: 412000/1000000
        // is 0.412, and a serializer that dropped a field or transposed two would fail here
        // rather than ship a share nothing can verify.
        assert!(
            json.contains(
                "\"enrollment\": 1000000, \"approved\": 412000, \"identified\": 0, \
                 \"share\": 0.412, \"floor\": 0.412, \"ceiling\": 0.412, \
                 \"without_applications\": 0, \"streams\": 1, \"basis\": \"adm\""
            ),
            "{json}"
        );
        assert!(
            json.contains(
                "\"enrollment\": 1000000, \"approved\": 437000, \"identified\": 0, \
                 \"share\": 0.437, \"floor\": 0.437, \"ceiling\": 0.437, \
                 \"without_applications\": 0, \"streams\": 1, \"basis\": \"ce\""
            ),
            "{json}"
        );
        // And the split year writes a null rather than a number, beside a band that is not
        // degenerate. A serializer that wrote `0` here would publish a poverty rate of nothing.
        assert!(
            json.contains(
                "\"approved\": 333000, \"identified\": 105000, \"share\": null, \
                 \"floor\": 0.438, \"ceiling\": 0.484, \"without_applications\": 0.166, \
                 \"streams\": 3, \"basis\": \"ce\""
            ),
            "{json}"
        );
    }

    #[test]
    fn the_meal_program_block_carries_no_dollars() {
        // A share is dimensionless and needs no deflator. If a dollar field ever lands here it
        // will need one, and the deflator does not reach FY2001 — so the failure would be a
        // nominal figure silently presented across a span in which prices rose by half.
        let json = bundle(vec![], vec![]).to_json();
        let block = json
            .split("\"meal_program\": [")
            .nth(1)
            .and_then(|rest| rest.split("],").next())
            .unwrap_or_default();
        for money in ["_per_pupil", "dollars", "amount", "total"] {
            assert!(
                !block.contains(money),
                "meal_program grew a `{money}` field; give it a denominator in \
                 web/src/lib/denominators.ts and a deflator that reaches FY2001, or drop it"
            );
        }
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

    #[test]
    fn the_year_index_is_emitted_sorted_whatever_order_it_was_assembled_in() {
        // Every fixture in this repository has to rebuild byte-identically from a clean checkout,
        // and the caller assembles these in the order the blocks happen to be built. The fixture
        // holds `millage` before `formula` for exactly this reason.
        let json = bundle(vec![sample()], vec![]).to_json();
        let formula = json
            .find("\"series\": \"formula\"")
            .expect("formula is in the index");
        let millage = json
            .find("\"series\": \"millage\"")
            .expect("millage is in the index");
        assert!(formula < millage, "the index is written in key order");
    }

    #[test]
    fn a_year_carries_the_reckoning_it_is_on_and_not_only_its_digits() {
        /*
         * The whole point of the block. A tax year is a calendar year whose revenue reaches the
         * district in the *following* fiscal year, so `2024` on a millage figure and `FY2024` on a
         * spending figure are eleven months apart. A consumer that gets only the digits cannot
         * tell them apart and will happily subtract them.
         */
        let json = bundle(vec![sample()], vec![]).to_json();
        assert!(json.contains("{\"series\": \"millage\", \"kind\": \"tax\", \"label\": \"2024\""));
        assert!(
            json.contains("{\"series\": \"formula\", \"kind\": \"fiscal\", \"label\": \"FY2027\"")
        );
    }

    #[test]
    fn an_absent_history_is_an_empty_array_rather_than_a_missing_key() {
        // A consumer that reads `history` and finds nothing there should get a series of length
        // zero and render the rest of the page, not a `undefined.map` two components later.
        assert!(bundle(vec![sample()], vec![])
            .to_json()
            .contains("\"history\": [\n  ],"));
    }

    #[test]
    fn every_history_year_carries_both_halves_of_what_the_page_draws() {
        let mut feed = bundle(vec![sample()], vec![]);
        feed.history = vec![HistoryYear {
            fiscal_year: 2009,
            districts: 612,
            local_share: 0.4801,
            state_share: 0.4302,
            federal_share: 0.0897,
            poorest_local_per_pupil: 4_012.0,
            richest_local_per_pupil: 9_988.0,
            gap_per_pupil: 5_976.0,
            state_closes_per_pupil: 2_760.0,
            federal_closes_per_pupil: 570.0,
        }];
        let json = feed.to_json();
        // The mix and the equalization measure are two findings joined on the year, and a page
        // drawing one without the other would report where the money came from while saying
        // nothing about whom it reached.
        for field in [
            "\"fiscal_year\": 2009",
            "\"districts\": 612",
            "\"local_share\": 0.4801",
            "\"gap_per_pupil\": 5976",
            "\"state_closes_per_pupil\": 2760",
            "\"federal_closes_per_pupil\": 570",
        ] {
            assert!(json.contains(field), "{field} missing: {json}");
        }
    }

    #[test]
    fn the_residual_is_what_no_level_of_government_closes() {
        // The number the series exists to show. State aid's *share* of the gap holds steady
        // across the panel while the gap grows, so the part nobody closes grows with it — and a
        // page that showed only the percentage would report that as stability.
        let year = HistoryYear {
            gap_per_pupil: 5_976.0,
            state_closes_per_pupil: 2_760.0,
            federal_closes_per_pupil: 570.0,
            ..HistoryYear::default()
        };
        assert!((year.residual_per_pupil() - 2_646.0).abs() < 1e-9);
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

/// One year of the Census survey, as the historical view needs it.
///
/// # Why this grain and not the formula's
///
/// Everything else in this feed is the Fair School Funding Plan computing FY2027 for 609
/// traditional districts. This is the only block that reaches back, and it reaches back on a
/// different measurement entirely: the Census Bureau's survey of what school systems actually
/// took in, which covers roughly 950 Ohio agencies a year including community schools and
/// educational service centers, on the Bureau's own enrollment count rather than ADM.
///
/// The two do not reconcile and are not meant to. A funding formula figure and a revenue survey
/// figure for the same district in the same year routinely disagree, which is exactly why the
/// [catalog](../../.yidam/catalog/census-f33-school-system-finances.md) exists.
///
/// # Comparable only
///
/// Every share and every quartile here is computed over the agencies the survey marks comparable
/// — roughly two-thirds of the rows — because that is the population the corpus's existing
/// single-year figures were computed over. A series whose first point is not comparable to the
/// number already on the page is worse than no series.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HistoryYear {
    /// The survey year, as a fiscal year.
    pub fiscal_year: u16,
    /// Comparable agencies the year is computed over.
    pub districts: usize,
    /// Local revenue as a share of total.
    pub local_share: f64,
    /// State revenue as a share of total.
    pub state_share: f64,
    /// Federal revenue as a share of total.
    pub federal_share: f64,
    /// Mean local revenue per pupil in the poorest quartile of districts.
    pub poorest_local_per_pupil: f64,
    /// And in the richest.
    pub richest_local_per_pupil: f64,
    /// The gap between them, which is what the other levels are measured against.
    ///
    /// Named for its denominator, as every per-pupil field in the feed is. `gap` alone would
    /// have escaped the web layer's denominator guard, which reads field names — and this whole
    /// block divides by a pupil count that is not the one any other figure on the site uses.
    pub gap_per_pupil: f64,
    /// Dollars per pupil of that gap closed by state aid.
    pub state_closes_per_pupil: f64,
    /// And by federal aid.
    pub federal_closes_per_pupil: f64,
}

impl HistoryYear {
    /// What neither level closes — the part a district actually experiences.
    #[must_use]
    pub fn residual_per_pupil(&self) -> f64 {
        self.gap_per_pupil - self.state_closes_per_pupil - self.federal_closes_per_pupil
    }
}

/// One fiscal year of what the General Assembly appropriated to the department.
///
/// # An appropriation is not a payment
///
/// This is what was set aside, not what a district received. An appropriation is a ceiling, and
/// the formula's own proration factor exists because at least one line has been a residual
/// claimant. A difference between this and the payment reports is not an error in either, and the
/// two must never be differenced to produce a third figure.
///
/// # Why the source is carried
///
/// Two publications answer for this series and they are not interchangeable, even though they
/// agree. The workbooks and greenbooks cover every year but four; the Catalog of Budget Line Items
/// covers **FY2006-07 and FY2012-13**, the two bienniums the workbook route cannot reach — one
/// because the 126th's greenbook has no line-item table at all, the other because LSC serves that
/// biennium's two workbook variants as the same file.
///
/// Across the 1,712 claims where both speak, the two extractions do not differ by a cent. Carrying
/// [`Self::source`] is therefore not a hedge about accuracy; it is so a reader can see that four
/// years of this series rest on a different document from the rest, and check them separately if
/// the difference ever starts to matter.
///
/// # No dollars per pupil here, deliberately
///
/// A statewide appropriation divided by a pupil count would be a per-pupil figure on a denominator
/// no other figure in this feed uses, sitting one division away from the formula's own per-pupil
/// numbers. The block carries totals and nothing else.
#[derive(Debug, Clone, PartialEq)]
pub struct AppropriationYear {
    /// The fiscal year.
    pub fiscal_year: u16,
    /// Everything the department was appropriated that year, in that year's dollars.
    ///
    /// Excludes the property tax reimbursement lines, which are numbered as the department's and
    /// are not its budget — `200903` alone is $1.3 billion a year.
    pub enacted: f64,
    /// The two foundation funding lines together: GRF `200550` and Lottery `200612`.
    ///
    /// The formula's own appropriation, as against everything else the department is given.
    pub foundation_funding: f64,
    /// How many line items the total is over.
    pub items: usize,
    /// Which publication answers for this year: `workbook` or `catalog`.
    pub source: String,
}

/// One appropriation line the department is funded through, and the act that created it.
///
/// # What this is for
///
/// [`AppropriationYear`] says how much the department was given. This says what the giving is made
/// of. Together they carry a fact neither carries alone: the department's budget is accreted
/// rather than designed — the lines in force were created by acts spanning half a century, and
/// the oldest still-live one predates every funding regime this corpus documents.
///
/// # Half of them say nothing about their origin
///
/// [`Self::general_assembly`] is `None` for roughly half the lines, because the Catalog's legal
/// basis cites only their current authority. Those are carried as unknown rather than filled from
/// an earlier edition with the same number: a line item number is reused — `200604` names three
/// different programmes across three funds in this series — so inheriting an origin down a number
/// attributes one programme's founding act to another's.
///
/// # `discontinued` is a label, not a finding
///
/// The publisher's own mark, and it does not distinguish abolition from consolidation: a line
/// folded into another is discontinued too. Whether the department's disappearing lines were
/// abolished or folded is an open question in `state-foundation-aid` that this does not settle.
#[derive(Debug, Clone, PartialEq)]
pub struct AppropriationLine {
    /// The fund it is paid from.
    pub fund: String,
    /// The six-digit line item number.
    pub ali: String,
    /// Its name in the current edition.
    pub name: String,
    /// The act that established it, as the Catalog writes it. Empty when it names none.
    pub established_by: String,
    /// That act's General Assembly, and the year it convened. `None` when no act is named.
    pub general_assembly: Option<u16>,
    /// The year that General Assembly convened. `None` alongside `general_assembly`.
    pub convened: Option<u16>,
    /// Whether the Catalog marks the line discontinued.
    pub discontinued: bool,
}

/// One October of the free and reduced-price lunch report, as a share.
///
/// # What this measures, and what it does not
///
/// Not enrollment and not poverty. MR-81 is the Office for Child Nutrition's meal-program report:
/// a count of *applications approved* for free or reduced-price lunch, over the denominator a
/// lunch claim is filed against. It is here because R.C. 3317.03(B)(21) hands the definition of
/// "economically disadvantaged" to the department, free-lunch eligibility has been the
/// department's operative test, and this is the longest run of that test available — seventeen
/// years where the rest of the feed has six.
///
/// [Disadvantaged pupil impact aid](../../.yidam/corpus/formula-component/fsfp-disadvantaged-pupil-impact-aid.yml)
/// is paid on that count, so this is the closest thing the feed carries to a history of what the
/// formula's poverty weight is computed on.
///
/// # Why there are no dollars here
///
/// Deliberately. A share is dimensionless, so this block needs no deflator and cannot be shown in
/// real terms — which is the point: the underlying counts are on a denominator no other figure in
/// this feed uses, and a dollar figure computed on it would be one division away from being
/// compared to a formula-side number. See [`Self::basis`] for the second reason.
///
/// # Sponsors are not districts
///
/// [`Self::sponsors`] counts *public sponsors*, which includes county boards of developmental
/// disabilities and community schools alongside traditional districts. The count rising across
/// the window is mostly community schools opening, not districts appearing, and it is carried
/// here so that a reader watching the share move can see the population move underneath it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MealProgramYear {
    /// The October counted, as a fiscal year.
    pub fiscal_year: u16,
    /// Public sponsors the year is computed over, after excluding published corruption.
    ///
    /// The FY2005 file gives one elementary school an enrollment of 342,332. It is excluded by
    /// name upstream rather than repaired, so this count is one lower that year than the file
    /// implies.
    pub sponsors: usize,
    /// The denominator in force that year, summed over those sponsors.
    ///
    /// Carried rather than implied. Two reasons, and the second is the load-bearing one: a share
    /// on its own cannot be checked, and the web layer's denominator guard walks *field names* —
    /// so a block whose only fields are `share` and `sponsors` is invisible to it. `enrollment`
    /// is a name that guard recognises, which is what forces this block to declare what it
    /// divides by. See `web/src/lib/denominators.ts`.
    pub enrollment: f64,
    /// Free and reduced-price applications approved, summed over those sponsors.
    ///
    /// From FY2012 this is short by every child in a community-eligibility school, because those
    /// sponsors collect no applications. See [`Self::streams`].
    pub approved: f64,
    /// Directly certified children in community-eligibility schools. Zero before FY2012.
    ///
    /// Not an approval and not comparable to one. Direct certification reaches families already on
    /// SNAP, TANF, foster care or a homeless roll; an application reaches anyone under the income
    /// line who files. The programme's own reckoning of the gap is the 1.6 multiplier behind
    /// [`Self::ceiling`].
    pub identified: f64,
    /// [`Self::approved`] over [`Self::enrollment`], which is the figure worth reading — while the
    /// report is one file.
    ///
    /// `None` from FY2012, and that is the finding rather than a gap. Three publications counting
    /// three different things do not add up to a share, so those years carry
    /// [`Self::floor`] and [`Self::ceiling`] instead and nothing writes a number between them.
    pub share: Option<f64>,
    /// The lowest share the source supports: approvals plus directly certified children.
    ///
    /// Equal to [`Self::share`] while the report is one file.
    pub floor: f64,
    /// The highest: what every sponsor may claim for, which under community eligibility is the
    /// directly certified count times 1.6, capped at enrollment school by school.
    ///
    /// Equal to [`Self::share`] while the report is one file.
    pub ceiling: f64,
    /// The share of the October's enrollment under sponsors that collect no applications.
    ///
    /// Zero through FY2011 and a sixth by FY2014. This is the size of the hole in
    /// [`Self::approved`], and it grows because community eligibility is open to schools whose
    /// poverty is already high — the population leaving the applications-based count is not a
    /// random sample of it.
    pub without_applications: f64,
    /// How many files the October was published as: one through FY2011, three from FY2012.
    ///
    /// The field a consumer has to read before drawing a line. From FY2012 the report splits into
    /// Traditional, Provision 2 and Community Eligibility, and only the first still counts
    /// applications — so a series that joins across this reads the poorest sponsors leaving the
    /// form as poverty falling.
    pub streams: usize,
    /// Which denominator that is: `adm` through FY2009, `ce` from FY2010.
    ///
    /// The definition changes mid-series. `CECount` is "the highest daily number of students with
    /// access to the program", which is neither ADM nor the count that preceded it, so the share
    /// is not continuous across FY2009/FY2010 and nothing here splices it. A consumer that plots
    /// this as one line without breaking it at the basis change is making the error this field
    /// exists to prevent.
    pub basis: String,
}
