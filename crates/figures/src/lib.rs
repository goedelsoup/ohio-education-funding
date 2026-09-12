//! The figures the corpus quotes, computed from the crates that own them.
//!
//! # What this is for
//!
//! `[verified — crates/regime-diff]` is hand-typed text. Nothing in the repository relates it to
//! `crates/regime-diff`, so the blast radius of a correction is set by whichever files the author
//! happened to open. That is not hypothetical: the `recognized-valuation` correction reached
//! three of its six carriers, and two nodes went on publishing a **reversed sign** on the headline
//! distributional claim, under `[verified]`, live, while every crate test stayed green. See #120
//! and #131.
//!
//! This crate is the other end of that citation. Each entry in [`FIGURES`] is a number the corpus
//! quotes in prose, computed here from the same public API the crate's own tests call, and emitted
//! as [`manifest`] — a committed JSON artefact. `web/tests/unit/corpusFigures.spec.ts` reads it and
//! fails when a node's bound figure no longer agrees with the crate it cites. A corrected
//! calculator therefore reddens every carrier at once, rather than the ones somebody remembered.
//!
//! # Why each figure carries a pin as well as a computation
//!
//! [`Figure::pinned`] duplicates what [`Figure::compute`] returns, and the duplication is the
//! point. Without it a calculator change would regenerate the manifest silently, and the first
//! anyone heard of it would be a corpus check going red in a different language with no statement
//! of what moved. With it, `the_manifest_reproduces_its_pins` fails in the crate that changed,
//! naming the figure and both values, and the corpus edit is a consequence of a decision somebody
//! made rather than the first notice of it.
//!
//! It is the same shape the pins in `crates/regime-diff/tests/` and `crates/project/tests/`
//! already have — `assert_eq!(zeroed, 65)` — hoisted to somewhere the corpus can reach.
//!
//! # What is not here
//!
//! Four kinds of claim, and three of them are permanent. A **rank** yields no numeral in either
//! form the corpus writes it — `seventh highest of fifty-one` has no digits and the `th` of
//! `25th of 51` defeats the token boundary. An **identifier** carries digits inside a
//! `[verified — crates/…]` tag and is not a quantity: an IRN, a bill number, an ALI code, a
//! SHA-256 digest. And **`revisions:`** is unbindable by design, because the corpus is never
//! rewritten to have always been right.
//!
//! The fourth kind is a **count spelled as a word**, and it is the one worth fixing rather than
//! recording — `Twenty districts report an effective Class 1 rate below 20 mills` bound
//! *successfully* to the `20` that meant mills, which is a false pass rather than a miss. Where the
//! corpus states a computed count it now states it in digits.
//!
//! What is *not* missing any more is the crate-side constraint this section used to describe. A
//! figure earns a place here by being computable through a crate's **public** API, and most of the
//! corpus's crate-attributed numerals used to be computed inside a test file on a fixture parser
//! that test declared privately. #157 moved eleven of those into the libraries that own them.
//!
//! `crates/figures.json` is a floor that only rises: `web/tests/unit/corpusFigures.spec.ts` pins
//! the bound count at its value, so the next figure exported is a figure that cannot quietly go
//! unbound — and `uncited-figure` makes an export no node binds a failure, so the manifest cannot
//! grow ahead of the corpus either.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

use edfund_core::FiscalYear;
use project::budget_analysis::{
    self, Edition, ALI_200540_TOTAL, LOTTERY_LINE, PRESCHOOL_REMAINDER, TOTAL_FOUNDATION_AID,
};
use project::panel::{panel, DistrictRecord};
use regime_diff::recognized_valuation::{self, Recognition};
use regime_diff::{panel_at_fy2027, ChargeOffBase, RegimeDiff, TERMINAL_MILLS};

mod json;

pub use json::manifest;

/// The damping `DEFAULT_DAMPING` replaced, restated rather than remembered.
///
/// Every "moving the damping from 0.85 to 0.30 moves" figure below is a difference against a run
/// at this value. Not imported from anywhere, because there is nowhere to import it from: it is a
/// number the repository no longer uses, kept because the corpus quotes differences against it.
const THE_DAMPING_BEFORE_IT_WAS_FITTED: f64 = 0.85;

/// The manifest schema version. Bump on any change to the fields an entry carries.
///
/// Read by the consumer before it reads anything else, on the same rule
/// [`bundle::CONTRACT_VERSION`](../../bundle/index.html) states: a check that does not recognise
/// the document should refuse to run rather than compare fields it may be misreading. A gate that
/// silently passes because it could not find what it was looking for is the failure mode #125
/// catalogued sixteen times.
pub const CONTRACT_VERSION: &str = "1.1.0";

/// What a figure is measured in, which decides how prose is allowed to write it.
///
/// The consumer needs this to read a corpus phrase back into a number. `28.1%` is `0.281` as a
/// [`Share`](Unit::Share) and `28.1` as a [`Ratio`](Unit::Ratio); `$5.1 million` is `5_100_000` as
/// [`Dollars`](Unit::Dollars) and nothing at all as a [`Count`](Unit::Count). Getting that wrong
/// by two orders of magnitude while both values parse is exactly what bundle contract `35.0.0`
/// was bumped for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// A whole number of things. Integral, and compared exactly.
    Count,
    /// Dollars, unscaled — `5_100_000`, never `5.1`.
    Dollars,
    /// A fraction of one. `0.281`, which prose writes `28.1%`.
    Share,
    /// A dimensionless number that is not a fraction of one — a correlation, a multiple.
    Ratio,
    /// A number of pupils, which Ohio measures rather than counts.
    ///
    /// Not a [`Count`](Unit::Count), and the difference is the point. Enrolled ADM is an
    /// *average* daily membership: a district of 327 children has a fractional ADM, and a
    /// projection of one is fractional twice over. `Count` asserts a whole number of things and
    /// compares exactly, which a measured quantity cannot satisfy and should not pretend to.
    /// Prose writes this bare, like a count, and it is compared to the precision the prose
    /// writes it to, like dollars.
    Pupils,
}

impl Unit {
    /// The word the manifest writes.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Dollars => "dollars",
            Self::Share => "share",
            Self::Ratio => "ratio",
            Self::Pupils => "pupils",
        }
    }
}

/// One figure the corpus quotes.
pub struct Figure {
    /// `<crate-directory>/<what-it-is>`. Stable: a corpus node names it, so renaming one is a
    /// corpus edit and the check will say so.
    pub key: &'static str,
    /// The crate that owns the computation, as the corpus cites it — `crates/regime-diff`.
    pub owner: &'static str,
    /// What the number is measured in.
    pub unit: Unit,
    /// What it is, in words, for somebody reading the manifest rather than this file.
    pub label: &'static str,
    /// What this crate means the figure to be. See the module docs for why this is not redundant.
    pub pinned: f64,
    /// How far [`Self::compute`] may sit from [`Self::pinned`]. Zero for a count.
    pub tolerance: f64,
    /// The computation, over inputs built once for the whole manifest.
    pub compute: fn(&Inputs) -> f64,
}

/// One frozen parameter from `project::indexation`, by the name the table gives it.
fn frozen(name: &str) -> &'static project::indexation::Parameter {
    project::indexation::PARAMETERS
        .iter()
        .find(|parameter| parameter.name == name)
        .expect("the figure names a parameter in the table")
}

/// The mean per-pupil shortfall in one quintile of districts ordered by poverty, 0 lowest.
fn quintile_shortfall(panel: &[DistrictRecord], quintile: usize) -> f64 {
    let rows = project::indexation::incidence(panel);
    let mut paired: Vec<(f64, f64)> = panel
        .iter()
        .zip(&rows)
        .map(|(record, row)| (record.dpia.percentage, row.per_pupil))
        .collect();
    paired.sort_by(|a, b| a.0.total_cmp(&b.0));
    let n = paired.len();
    let slice = &paired[quintile * n / 5..(quintile + 1) * n / 5];
    slice.iter().map(|pair| pair.1).sum::<f64>() / slice.len() as f64
}

/// The shared inputs. Built once, because the FY2027 panel and the county abstract are read from
/// fixtures and every regime-diff figure below wants the same two runs over them.
pub struct Inputs {
    /// The 609 districts of the FY2027 department model.
    pub panel: Vec<DistrictRecord>,
    /// The charge-off counterfactual against recognized valuation at TY2024 — the base the
    /// mechanism actually used, and the one the corpus was wrong about for fourteen phases.
    pub at_recognized: Vec<RegimeDiff>,
    /// The same counterfactual against total taxable value, which is what the corpus assumed.
    /// Kept because two of the figures below are the size of that error.
    pub at_total_taxable: Vec<RegimeDiff>,
    /// Ohio's comparable districts quartiled by local revenue per pupil, FY2022, with what each
    /// level of government pays into each quartile.
    pub quartiles: [dispersion::national_peers::WealthQuartile; 4],
    /// The Census Bureau's state-level survey, FY2022 — the only source here that can say whether
    /// Ohio is unusual, and the one the corpus quotes for every national comparison.
    ///
    /// Not [`dispersion::national_peers`], which is the *district* panel and gives Ohio a 51.7%
    /// local share against this table's 51.8%. Two nodes cited the first for four figures only the
    /// second computes; see #157.
    pub states: Vec<dispersion::census_states::StateFinance>,
    /// The FY2024 District Profile Report — 606 traditional districts, and the cross-section most
    /// of the corpus's district-level findings are computed over.
    pub profile: Vec<dispersion::profile::ProfileDistrict>,
    /// Table SD-1, every district and every tax year the abstract carries.
    pub sd1: Vec<dispersion::sd1::TaxRow>,
    /// The general fund's transfers line by year, and the two attributions of the FY2021-FY2024
    /// cash build-up to the federal relief — one levelled on FY2020, one on both baseline years.
    pub transfers: BTreeMap<u16, project::transfers::Year>,
    /// Levelled on FY2020 alone.
    pub attribution_before: project::transfers::Attribution,
    /// Levelled on the mean of FY2020 and FY2025.
    pub attribution_both: project::transfers::Attribution,
    /// The quartile equalization measure for every year of the survey panel.
    ///
    /// Read for the band's two ends and for FY2024. Held here rather than recomputed per figure
    /// because each call walks fifteen years of the panel.
    pub equalization: BTreeMap<u16, dispersion::ohio_panel::Equalization>,
    /// Ohio's districts quartiled on local revenue per pupil in FY2022, with the share of each
    /// quartile at the twenty-mill floor and what its local revenue did through FY2024.
    pub incidence: [regime_diff::reappraisal_incidence::Quartile; 4],
    /// The FY2024 quartile gap as observed and with every district on its own quiet-year rate —
    /// the reappraisals removed and the trend kept.
    pub reappraisal: regime_diff::reappraisal_incidence::Counterfactual,
    /// The 2024-25 report card, joined to the profile report by IRN — 606 districts on both, the
    /// report card rating 607 and the profile report covering 606.
    pub outcomes: Vec<(
        dispersion::report_card::ReportCard,
        Option<dispersion::profile::ProfileDistrict>,
    )>,
    /// The funding model joined to the report card — 606 districts on all three panels, which is
    /// the population every guarantee-against-achievement figure is computed over.
    pub joined: Vec<project::outcomes::Joined>,
    /// Year-over-year movements in the foundation aid appropriation, in constant FY2025 dollars.
    ///
    /// The series `casino-tax-distribution` rests its central null result on: an earmark's arrival
    /// is only readable off an appropriation total if it moves the total by more than the total
    /// ordinarily moves.
    pub movements: Vec<(u16, f64)>,
    /// The casino channel actually distributed to districts, by state fiscal year — the other side
    /// of that comparison, and the one that was assumed rather than measured until `tax-casino`
    /// was wired.
    pub casino: BTreeMap<u16, f64>,
    /// Every school district's presence in every House seat, which is where the corpus's claims
    /// about how badly the two geographies nest are computed.
    pub house: Vec<project::legislative_district::Overlap>,
    /// The base cost reference-year refresh, priced against the FY2027 panel.
    pub refresh: project::drafts::Priced,
    /// The same refresh run together with a half-retired guarantee — the draft whose whole point
    /// is that the two provisions do not add.
    pub fund_the_plan: project::drafts::Priced,
    /// Total taxable value as Table SD-1 publishes it, summed over the districts the county
    /// abstract can recognize.
    pub actual_total: f64,
    /// The same sum with each reappraisal's inflationary increase phased in rather than counted
    /// whole. The difference between the two is what the charge-off does not reach.
    pub recognized_total: f64,
    /// The FY2010-FY2013 real contraction, over the districts the F-33 panel carries in both.
    pub trough: dispersion::ohio_panel::trough::Contraction,
    /// What tracks the depth of that fall, and what does not — the corpus's seven-row null.
    pub predictors: dispersion::ohio_panel::trough::Predictors,
    /// The worst tenth of it, which is real and is not what moved the state.
    pub decile: dispersion::ohio_panel::trough::DeepestDecile,
    /// What refreshing the classroom teacher salary input to FY2024 does, across the state.
    pub refresh_incidence: foundation::Refresh,
    /// The same increase run as a uniform base cost scale, so the guarantee can be applied to
    /// it. A DIFFERENT run from `refresh_incidence` — that one perturbs one salary per district
    /// and this one scales the department's published aggregate — which the corpus states.
    pub refresh_reach: scenario_delta::ScenarioDelta,
    /// The White Paper's sensitivity table on both denominators.
    pub sensitivity: dispersion::report_card::SensitivityTable,
    /// The report card's statewide outcome block — the federal-share distribution and the two
    /// value-added agreement statistics, over the 606 districts joined on all three panels.
    pub outcome_block: bundle::OutcomeStatewide,
    /// Toledo City, FY2025: the expenditure-function row beside the report card row.
    pub toledo: CrossSection,
    /// Perrysburg Exempted Village, the other half of the pair.
    pub perrysburg: CrossSection,
    /// The median district's operating spending per headcount pupil, over the 607 the department
    /// rates. The line `perrysburg-exempted-village` places itself below.
    pub median_operating_per_pupil: f64,
    /// FY2025 spending by function joined to the report card, and the models that take the
    /// corpus's published spending coefficient apart.
    pub composition: Composition,
    /// The corpus's worked pair, over the fifteen years its two nodes said they could not see.
    pub pair: Pair,
    /// ECOT's eight years in the survey, and Cleveland's fifteen — the other two agencies whose
    /// nodes recorded a series as unpopulated that the panel already held.
    pub closed: Closed,
    /// What moved in the panel's state column at FY2016, and what it tracks.
    pub fy2016: Fy2016,
    /// The break the same column has at that year, which belongs to the survey and not to Ohio.
    pub basis: SurveyBasis,
    /// The school construction program, from the side of it the survey counts.
    pub facilities: Facilities,
    /// Every state's Education Finance Incentive Grant equity factor, widest spread first.
    ///
    /// Computed once because each call walks all 10,739 rows of the national district panel, and
    /// six figures here read it.
    pub equity: Vec<dispersion::equity_factor::StateEquity>,
    /// Ohio's factor with and without the statutory poverty adjustment — the pair that brackets
    /// which band Ohio is in, rather than estimating it.
    pub equity_bracket: (f64, f64),
    /// The four projected runs `metric/enrolled-adm` quotes.
    ///
    /// The first projection figures in this manifest, and they are here because of how the ones
    /// they replace went wrong. `the-shrunk-rate` changed the projection method, every figure in
    /// that node's damping paragraph moved, and nothing said so — the numbers were quoted from a
    /// test that pins the *previous* method on purpose, as the evidence `the-fitted-damping` was
    /// decided on. Binding them is what makes the next change to a projection constant redden the
    /// corpus rather than silently restate it.
    pub forecasts: Forecasts,
}

/// The FY2016 move, measured against the tax base and the formula that arrived that year.
pub struct Fy2016 {
    /// Every district the step and Table SD-1 can both be computed for.
    pub districts: Vec<dispersion::fy2016::District>,
    /// Mean step by fifth of total assessed value, smallest first.
    pub by_valuation: [f64; 5],
    /// The step on log total valuation, wealth per pupil and disadvantage together.
    pub model: dispersion::Regression,
}

/// Ohio's school construction program as the survey's receiving side records it.
///
/// Held as a block because every figure walks the whole panel and joins the profile report, and
/// because the identification only means anything as a set. See [`dispersion::facilities`].
pub struct Facilities {
    /// One row per district: state capital money, capital spending, and wealth.
    pub districts: Vec<dispersion::facilities::District>,
    /// State capital money and capital spending by year, in dollars.
    pub by_year: BTreeMap<u16, (f64, f64)>,
    /// State share of capital spending by fifth of district wealth, poorest first.
    pub by_wealth: [f64; 5],
    /// The same averaged over district-years instead, which is the measure not used.
    pub by_wealth_annual: [f64; 5],
    /// Nonspecified state revenue against capital outlay, per year.
    pub against_capital: Vec<(u16, f64)>,
    /// And general formula assistance against it, which is the control.
    pub formula_against_capital: Vec<(u16, f64)>,
    /// Districts whose peak year of state capital money is a build year, over those examined.
    pub peaks: (usize, usize),
}

/// Local and state revenue shares by fiscal year, over comparable districts.
pub type ShareSeries = BTreeMap<u16, (f64, f64)>;

/// The FY2016 break in the panel's state column, and the correction of it.
///
/// Held as a block because every figure in it walks the fifteen-year panel, and because a figure
/// quoting the published measure beside the corrected one has to be sure both came from the same
/// walk. See [`dispersion::survey_basis`].
pub struct SurveyBasis {
    /// Each consecutive pair of years, and how much of the move the deduct predicts.
    pub transitions: Vec<dispersion::survey_basis::Transition>,
    /// Districts falling more than a fifth and the pupils they hold, as published.
    pub published_fallers: (usize, f64),
    /// The same on one basis.
    pub corrected_fallers: (usize, f64),
    /// The FY2016 step on one basis, smallest first.
    pub corrected_step: Vec<(String, f64, f64)>,
    /// The districts' fall across FY2016 and the deduct that stopped being counted, in dollars.
    pub fall_and_deduct: (f64, f64),
    /// Local and state shares by year, as published and on one basis.
    pub shares: (ShareSeries, ShareSeries),
    /// The three exemplars' state revenue per pupil restated on one basis, FY2010 to FY2024 —
    /// Toledo, Perrysburg and Cleveland, keyed by IRN. Each walks the whole panel once.
    pub restated: BTreeMap<&'static str, dispersion::exemplars::Change>,
}

/// ECOT and Cleveland, on the series their nodes recorded as unpopulated.
pub struct Closed {
    /// Every year the panel holds ECOT, oldest first.
    pub ecot: Vec<dispersion::exemplars::Year>,
    /// State revenue and enrolment summed over the comparable districts, by year — the
    /// denominator a community school's draw is read against.
    pub districts: BTreeMap<u16, (f64, f64)>,
    /// Cleveland's state revenue per pupil across the panel's span.
    pub cleveland_state: dispersion::exemplars::Change,
    /// And its enrolment.
    pub cleveland_enrolment: dispersion::exemplars::Change,
}

/// Toledo and Perrysburg on the four measures their nodes recorded as not held.
///
/// The histories are held rather than recomputed because each walks the whole fifteen-year panel,
/// and the step measure walks it seven times.
pub struct Pair {
    /// Toledo's wealth, millage, floor status and spending composition.
    pub toledo: dispersion::exemplars::Standing,
    /// Perrysburg's.
    pub perrysburg: dispersion::exemplars::Standing,
    /// Toledo's state revenue per pupil across the panel's span, nominal and real.
    pub toledo_state: dispersion::exemplars::Change,
    /// And its local revenue.
    pub toledo_local: dispersion::exemplars::Change,
    /// Perrysburg's state revenue.
    pub perrysburg_state: dispersion::exemplars::Change,
    /// And its local revenue, the one that goes the other way.
    pub perrysburg_local: dispersion::exemplars::Change,
    /// Every comparable district's FY2016 step, smallest first.
    pub step: Vec<(String, f64, f64)>,
}

/// The four models that answer `metric/progress-value-added`'s largest recorded omission.
///
/// Held together because they share one join — the report card, the function file and the profile
/// report, over the 606 districts on all three — and because a figure quoting one of them beside
/// another has to be sure they were fitted on the same rows.
pub struct Composition {
    /// The frame itself, for the descriptive figures.
    pub districts: Vec<dispersion::composition::District>,
    /// The department's classroom / non-classroom split against growth.
    pub split: dispersion::Regression,
    /// Total spending and the classroom share together, against growth.
    pub on_growth: dispersion::Regression,
    /// The same pair against attainment level, where the two signs differ.
    pub on_level: dispersion::Regression,
    /// All nine published functions at once, against growth.
    pub functions: dispersion::Regression,
}

/// The projected runs the manifest quotes, computed once because each walks all 609 districts.
pub struct Forecasts {
    /// FY2032 at the shipped method — the nearer of the two horizons this repository publishes.
    pub fy2032: project::report::EnrollmentEffect,
    /// FY2036 at the shipped method.
    pub fy2036: project::report::EnrollmentEffect,
    /// FY2036 at the 0.85 damping that shipped before `the-fitted-damping`, which is what the
    /// corpus's "moving the damping from 0.85 to 0.30 moves" figures are differences against.
    pub fy2036_at_the_old_convention: project::report::EnrollmentEffect,
    /// FY2036 undamped, which the backtest finds very nearly unbiased and which the corpus
    /// quotes as the distance the damping holds the projection above.
    pub fy2036_undamped: project::report::EnrollmentEffect,
    /// FY2032 with the guarantee removed — the other half of `scenario/guarantee-phase-out`'s
    /// central comparison, and the run its "nearly doubles the state's exposure" rests on.
    pub fy2032_guarantee_removed: project::report::EnrollmentEffect,
}

impl Inputs {
    /// Read the fixtures and run both counterfactuals.
    #[must_use]
    pub fn build() -> Self {
        let panel = panel();
        // Built before the struct literal because the outcome block is computed FROM it, and the
        // literal below moves it in.
        let joined = project::outcomes::joined();
        let outcome_block =
            bundle::build::outcome_statewide(&joined).expect("the joined panel is non-empty");
        // Cloned rather than borrowed: `panel` moves into the struct literal below, and the two
        // draft runs need it before that happens.
        let panel_for_drafts = panel.clone();
        let panel_for_reach = panel.clone();
        let panel_for_forecasts = panel.clone();
        let recognized: HashMap<String, Recognition> = recognized_valuation::from_abstract(2024);
        let at_recognized = panel_at_fy2027(
            &panel,
            TERMINAL_MILLS,
            ChargeOffBase::Recognized(&recognized),
        );
        let at_total_taxable = panel_at_fy2027(&panel, TERMINAL_MILLS, ChargeOffBase::TotalTaxable);
        // Summed in IRN order, not in whatever order the map hands them over.
        //
        // `HashMap` seeds a fresh hasher per instance, so its iteration order differs between two
        // maps in one process — and floating-point addition is not associative. Summing over
        // `values()` put the last three digits of three figures on a coin flip, which made
        // `crates/figures.json` a generated artefact that could not be regenerated: `mise run
        // //:generated` was red on a clean tree, the same shape as #124. `the_manifest_is_stable`
        // is what keeps it fixed.
        let mut ordered: Vec<(&String, &Recognition)> = recognized.iter().collect();
        ordered.sort_by(|a, b| a.0.cmp(b.0));
        let actual_total = ordered.iter().map(|(_, r)| r.actual).sum();
        let recognized_total = ordered.iter().map(|(_, r)| r.recognized).sum();
        Self {
            panel,
            at_recognized,
            at_total_taxable,
            quartiles: dispersion::national_peers::ohio_by_local_wealth(),
            states: dispersion::census_states::states(),
            profile: dispersion::profile::districts(),
            sd1: dispersion::sd1::rows(),
            joined,
            outcomes: {
                let by_irn: std::collections::BTreeMap<
                    String,
                    dispersion::profile::ProfileDistrict,
                > = dispersion::profile::districts()
                    .into_iter()
                    .map(|d| (d.irn.clone(), d))
                    .collect();
                dispersion::report_card::report_cards()
                    .into_iter()
                    .map(|card| {
                        let profile = by_irn.get(&card.irn).cloned();
                        (card, profile)
                    })
                    .collect()
            },
            movements: project::appropriations::foundation_movements(BASE),
            casino: dispersion::casino::by_fiscal_year(),
            house: project::legislative_district::overlaps(
                project::legislative_district::Chamber::House,
            ),
            refresh: priced("hb-96-with-refreshed-inputs", &panel_for_drafts),
            fund_the_plan: priced("fund-the-plan-and-retire-the-guarantee", &panel_for_drafts),
            actual_total,
            recognized_total,
            // Computed once here rather than per figure: each of the three walks the whole
            // ten-year panel and deflates it, and fourteen figures below read them.
            transfers: project::transfers::by_year(),
            attribution_before: project::transfers::attribution(false),
            attribution_both: project::transfers::attribution(true),
            equalization: dispersion::ohio_panel::equalization_by_year(),
            incidence: regime_diff::reappraisal_incidence::by_wealth(),
            reappraisal: regime_diff::reappraisal_incidence::gap(2024),
            trough: dispersion::ohio_panel::trough::contraction(),
            predictors: dispersion::ohio_panel::trough::predictors(),
            decile: dispersion::ohio_panel::trough::deepest_decile(),
            sensitivity: dispersion::report_card::sensitivity(),
            refresh_incidence: foundation::refresh(),
            refresh_reach: {
                // The department's own FY2027 computed increase, applied as a uniform scale.
                const COMPUTED_INCREASE: f64 = 465.0e6;
                let aggregate: f64 = panel_for_reach.iter().map(|r| r.aggregate_base_cost).sum();
                scenario_delta::ScenarioDelta::between(
                    &panel_for_reach,
                    &project::policy::Policy::current_law(),
                    &project::policy::Policy {
                        base_cost_scale: 1.0 + COMPUTED_INCREASE / aggregate,
                        ..project::policy::Policy::current_law()
                    },
                )
            },
            outcome_block,
            toledo: CrossSection::of(TOLEDO),
            perrysburg: CrossSection::of(PERRYSBURG),
            median_operating_per_pupil: {
                let mut column: Vec<f64> = dispersion::functions::districts()
                    .iter()
                    .filter_map(|d| d.operating)
                    .collect();
                column.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a published dollar"));
                dispersion::median(&column).expect("the function file is not empty")
            },
            fy2016: Fy2016 {
                districts: dispersion::fy2016::frame(),
                by_valuation: dispersion::fy2016::quintiles_by(|d| d.total_valuation),
                model: dispersion::fy2016::model(),
            },
            facilities: {
                use dispersion::facilities as f;
                Facilities {
                    districts: f::frame(),
                    by_year: f::by_year(),
                    by_wealth: f::by_wealth(),
                    by_wealth_annual: f::by_wealth_annual(),
                    against_capital: f::against_capital(),
                    formula_against_capital: f::formula_against_capital(),
                    peaks: f::peaks_are_build_years(),
                }
            },
            basis: {
                use dispersion::survey_basis::{self as sb, Basis};
                SurveyBasis {
                    transitions: sb::transitions(),
                    published_fallers: sb::fallers(Basis::AsPublished, 0.20),
                    corrected_fallers: sb::fallers(Basis::Gross, 0.20),
                    corrected_step: sb::step(Basis::Gross),
                    fall_and_deduct: dispersion::fy2016::fall_against_deduct(),
                    shares: (sb::shares(Basis::AsPublished), sb::shares(Basis::Net)),
                    restated: ["044909", "045583", "043786"]
                        .into_iter()
                        .map(|irn| {
                            let change =
                                dispersion::exemplars::state_change(irn, 2010, 2024, Basis::Net)
                                    .expect("the panel holds all three in both years");
                            (irn, change)
                        })
                        .collect(),
                }
            },
            closed: {
                use dispersion::exemplars::{self, CLEVELAND, ECOT};
                Closed {
                    ecot: exemplars::history(ECOT),
                    districts: exemplars::district_state_revenue(),
                    cleveland_state: exemplars::change(CLEVELAND, |y| y.state),
                    cleveland_enrolment: exemplars::change(CLEVELAND, |y| y.enrolment),
                }
            },
            pair: {
                use dispersion::exemplars::{self, PERRYSBURG, TOLEDO};
                Pair {
                    toledo: exemplars::standing(TOLEDO).expect("Toledo joins all three panels"),
                    perrysburg: exemplars::standing(PERRYSBURG)
                        .expect("Perrysburg joins all three panels"),
                    toledo_state: exemplars::change(TOLEDO, |y| y.state),
                    toledo_local: exemplars::change(TOLEDO, |y| y.local),
                    perrysburg_state: exemplars::change(PERRYSBURG, |y| y.state),
                    perrysburg_local: exemplars::change(PERRYSBURG, |y| y.local),
                    step: exemplars::fy2016_step(),
                }
            },
            composition: {
                let districts = dispersion::composition::frame();
                Composition {
                    split: dispersion::composition::split_on_growth(&districts),
                    on_growth: dispersion::composition::composition_on(&districts, |d| d.growth),
                    on_level: dispersion::composition::composition_on(&districts, |d| d.level),
                    functions: dispersion::composition::every_function_on_growth(&districts),
                    districts,
                }
            },
            equity: dispersion::equity_factor::by_state(),
            equity_bracket: dispersion::equity_factor::ohio_bracket(),
            forecasts: {
                // `panel_for_forecasts` because `panel` has moved into the literal above.
                let prior = project::report::enrollment_growth_prior(
                    &panel_for_forecasts,
                    project::series::ONE_SIGMA,
                );
                // `toward` is a placeholder: `DistrictRecord::projection_method` replaces it with
                // the district's own long-run rate, which is the whole point of `Shrunk`.
                let shrunk = |damping: f64| project::series::Method::Shrunk {
                    rate: 0.0,
                    damping,
                    weight: project::series::DEFAULT_SHRINK_WEIGHT,
                    toward: 0.0,
                };
                let run = |year: u16, damping: f64| {
                    project::report::forecast(
                        &panel_for_forecasts,
                        &project::policy::Policy::current_law(),
                        FiscalYear(year),
                        shrunk(damping),
                        prior,
                    )
                };
                Forecasts {
                    fy2032: run(2032, project::series::DEFAULT_DAMPING),
                    fy2036: run(2036, project::series::DEFAULT_DAMPING),
                    fy2036_at_the_old_convention: run(2036, THE_DAMPING_BEFORE_IT_WAS_FITTED),
                    fy2036_undamped: run(2036, 1.0),
                    fy2032_guarantee_removed: project::report::forecast(
                        &panel_for_forecasts,
                        &project::policy::Policy {
                            guarantee: project::policy::GuaranteeRule::Removed,
                            ..project::policy::Policy::current_law()
                        },
                        FiscalYear(2032),
                        shrunk(project::series::DEFAULT_DAMPING),
                        prior,
                    ),
                }
            },
        }
    }
}

/// One district's resident pupils per pupil taught, on `dispersion::valuation::TAX_YEAR`.
///
/// # Panics
///
/// If the district does not join both tables, which means one of the fixtures moved.
fn named_ratio(irn: &str) -> f64 {
    dispersion::valuation::district(irn)
        .unwrap_or_else(|| panic!("{irn} joins both tables"))
        .denominator_ratio()
}

/// How many line items one Catalog edition authorises under R.C. 3317.
///
/// # Panics
///
/// If the edition is not in the basis fixture, which means the extract lost a year.
/// Nominal and real growth in the statewide base cost per pupil between two fiscal years.
///
/// Both ends have to carry an amount; every year a figure here names does.
/// One statewide calculator scalar in one fiscal year.
///
/// # Panics
///
/// If the scalar fixture holds no such column for that year, which means the fixture and this
/// manifest have come apart.
fn scalar_at(name: &str, fiscal_year: u16) -> f64 {
    *project::prior_model::scalar(name)
        .get(&fiscal_year)
        .unwrap_or_else(|| panic!("no {name} for FY{fiscal_year}"))
}

fn base_cost_growth(from: u16, to: u16) -> (f64, f64) {
    project::base_cost::growth(from, to).expect("both years carry an amount")
}

/// The current freeze, as `(the anchor amount restated in FY2026 dollars, what is paid, the loss)`.
///
/// The last of [`project::base_cost::freezes`], which is FY2024's and is still running.
fn base_cost_freeze() -> (f64, f64, f64) {
    let runs = project::base_cost::freezes();
    let current = runs.last().expect("the series holds three freezes");
    project::base_cost::erosion(current).expect("the anchor year deflates")
}

fn chapter_lines(edition: u16) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    {
        project::ledger::line_origins::lines_under("3317")
            .get(&edition)
            .unwrap_or_else(|| panic!("the {edition} edition"))
            .len() as f64
    }
}

/// One district's FY2016 step on [`dispersion::fy2016::BASIS`].
///
/// # Panics
///
/// If the district is not in the step, which means the panel lost a year for it.
fn step_of(inputs: &Inputs, irn: &str) -> f64 {
    inputs
        .basis
        .corrected_step
        .iter()
        .find(|(key, _, _)| key == irn)
        .unwrap_or_else(|| panic!("{irn} is not in the corrected step"))
        .1
}

/// Toledo City, in both FY2025 fixtures.
const TOLEDO: &str = "044909";
/// Perrysburg Exempted Village, in both.
const PERRYSBURG: &str = "045583";

/// One district's FY2025 cross-section: its expenditure-function row and its report card row.
///
/// Held as a pair because the corpus quotes them in one breath — *21,160 pupils, $440.2 million of
/// operating expenditure, $20,805 per headcount pupil, a Performance Index of 60.4* crosses the two
/// files twice in a sentence — and every figure the pair of nodes publishes is one file or the
/// other, keyed on the same IRN.
pub struct CrossSection {
    /// Operating spending by function, per headcount pupil, FY2025.
    pub functions: dispersion::functions::Functions,
    /// The 2024-25 report card row for the same district.
    pub card: dispersion::report_card::ReportCard,
}

impl CrossSection {
    /// Both fixtures' rows for one district.
    ///
    /// # Panics
    ///
    /// If either committed fixture does not carry the IRN.
    #[must_use]
    pub fn of(irn: &str) -> Self {
        Self {
            functions: dispersion::functions::district(irn)
                .unwrap_or_else(|| panic!("{irn} is in the expenditure-function file")),
            card: dispersion::report_card::district(irn)
                .unwrap_or_else(|| panic!("{irn} is in the 2024-25 report card")),
        }
    }

    /// Operating spending per headcount pupil, as the department's function file publishes it.
    ///
    /// Not the report card's total divided by its ADM. The two agree to the cent here, and the
    /// published figure is the one the corpus quotes.
    ///
    /// # Panics
    ///
    /// If the district reports no operating total. Both of these do.
    #[must_use]
    pub fn operating(&self) -> f64 {
        self.functions
            .operating
            .expect("the department publishes an operating total for both districts")
    }

    /// One function as a share of that total.
    ///
    /// # Panics
    ///
    /// If the district reports no figure for the function. Both report every one below.
    #[must_use]
    pub fn share(&self, pick: fn(&dispersion::functions::Functions) -> Option<f64>) -> f64 {
        self.functions
            .share(pick(&self.functions))
            .expect("the department publishes this function for both districts")
    }

    /// One report-card field.
    ///
    /// # Panics
    ///
    /// If the district reports no value for it. Both are fully reported.
    #[must_use]
    pub fn card(&self, pick: fn(&dispersion::report_card::ReportCard) -> Option<f64>) -> f64 {
        pick(&self.card).expect("the report card carries this field for both districts")
    }
}

/// H.B. 643 of the 136th, the corpus's one pending bill.
///
/// Read per figure rather than held on [`Inputs`]: it is one row of a committed TSV, against the
/// FY2027 panel and two draft runs that are not.
///
/// # Panics
///
/// If the draft register does not carry the slug.
fn hb643() -> project::drafts::Draft {
    project::drafts::draft("hb-643-136-introduced").expect("the draft register carries this bill")
}

/// How many of an act's reprinted sections are in one chapter.
fn reprinted_in(act: &str, chapter: &str) -> f64 {
    project::act::headings(act)
        .iter()
        .filter(|heading| heading.starts_with(chapter))
        .count() as f64
}

/// The base year every real appropriation figure here is stated in.
///
/// FY2025 because that is the year the corpus's noise-floor paragraph is written in, and a base
/// year is part of a constant-dollar figure rather than a detail of how it was produced.
const BASE: FiscalYear = FiscalYear(2025);

/// Price one draft against the panel.
///
/// # Panics
///
/// If the slug names no draft, which would mean `project/fixtures/draft-provisions.tsv` has been
/// edited without the figures that quote its runs being moved with it.
fn priced(slug: &str, panel: &[DistrictRecord]) -> project::drafts::Priced {
    let draft = project::drafts::draft(slug)
        .unwrap_or_else(|| panic!("no draft {slug:?} in the provisions fixture"));
    project::drafts::price(&draft, panel)
}

/// A figure and what it came out at on this run.
pub struct Computed {
    /// The registry entry.
    pub figure: &'static Figure,
    /// What [`Figure::compute`] returned.
    pub value: f64,
}

/// Run every figure in [`FIGURES`], in registry order.
#[must_use]
pub fn compute_all() -> Vec<Computed> {
    let inputs = Inputs::build();
    FIGURES
        .iter()
        .map(|figure| Computed {
            figure,
            value: (figure.compute)(&inputs),
        })
        .collect()
}

/// One row of the enacted earmark table for ALI 200540.
fn enacted_200540(label: &str) -> f64 {
    budget_analysis::special_education_enhancements(Edition::Enacted, label).second
}

/// The preschool special education program at the department's own stated proration factor.
fn prek_sped_program(i: &Inputs) -> f64 {
    i.panel
        .iter()
        .map(|record| record.preschool_special_education.total)
        .sum()
}

/// A movement in the foundation aid appropriation for one fiscal year, as a magnitude.
///
/// The sign is carried by the figure's key for the reason [`outcome_correlation`] states: prose
/// writes `+$183 million` and `−$101 million`, and the consumer's numeral reader does not read a
/// sign.
///
/// # Panics
///
/// If the year is not in the series, which would mean the appropriation fixtures no longer reach
/// the years the casino channel came online — the two years the whole null result is about.
fn movement(i: &Inputs, fiscal_year: u16) -> f64 {
    i.movements
        .iter()
        .find(|(year, _)| *year == fiscal_year)
        .map(|(_, moved)| moved.abs())
        .unwrap_or_else(|| panic!("FY{fiscal_year} is not in the appropriation series"))
}

/// The movement magnitudes, sorted, which every summary of the noise floor is taken over.
fn movement_magnitudes(i: &Inputs) -> Vec<f64> {
    let mut magnitudes: Vec<f64> = i.movements.iter().map(|(_, moved)| moved.abs()).collect();
    magnitudes.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    magnitudes
}

/// How many House seats each school district has population in.
fn house_reach(i: &Inputs) -> BTreeMap<&str, usize> {
    let mut reach: BTreeMap<&str, usize> = BTreeMap::new();
    for overlap in &i.house {
        *reach.entry(overlap.irn.as_str()).or_default() += 1;
    }
    reach
}

/// How many districts a draft lifts off the guarantee — the count that changes which mechanism
/// pays them rather than how much.
fn lifted_off_the_guarantee(priced: &project::drafts::Priced) -> f64 {
    let effect = priced.effect();
    (effect.baseline.on_guarantee - effect.policy.on_guarantee) as f64
}

/// A correlation over the report card joined to the profile report, as a magnitude.
///
/// The sign is deliberately dropped and carried by the figure's key instead. Prose writes these
/// `−0.846` and `+0.375`, and the consumer's numeral reader does not read a sign — so a figure
/// pinned negative could not be bound to the sentence that states it without the check ignoring
/// direction, which is the one thing #120 was about.
/// The achievement denominator for a year: the mean of the highest two per cent of district
/// scores, with the two per cent taken as a ceiling.
///
/// R.C. 3302.03(D)(1)(c) defines it and does not say how to round "two per cent"; the department
/// publishes the 2024-25 value and only the ceiling reproduces it. See
/// `crates/dispersion/tests/the_weights_behind_the_index.rs`, which is where that is established
/// rather than assumed. This exists because the department publishes the maximum for the current
/// year alone, so any earlier year's has to be recomputed the same way.
/// The TY1981 joint vocational rates implied by the districts sitting just under the floor.
///
/// R.C. 319.301(E)(1) makes the omitted term the excess of a joint vocational district's TY1981
/// current-expense taxes over two-tenths of one per cent of value. Adding a district's shortfall
/// below twenty mills back onto that two-mill base gives the rate the term would have to have
/// been. Returns the smallest and largest across the fourteen.
fn implied_1981_rates(i: &Inputs) -> (f64, f64) {
    let mut implied: Vec<f64> = i
        .profile
        .iter()
        .filter(|d| d.below_twenty_mill_floor() && !d.never_voted_twenty_mills())
        .filter_map(|d| Some(2.0 + (20.0 - d.effective_class1_millage?)))
        .collect();
    implied.sort_by(f64::total_cmp);
    (
        implied.first().copied().unwrap_or(f64::NAN),
        implied.last().copied().unwrap_or(f64::NAN),
    )
}

/// Districts below the three-star release threshold: how many, how many are paid, and how much.
///
/// R.C. 3302.10(N)(1) needs three stars to *begin* a transition out of an academic distress
/// commission, and the performance supplement's progress routes need nothing of the overall
/// rating at all. The gap between the two is what this counts.
fn below_the_release_threshold(i: &Inputs) -> (usize, usize, f64) {
    let below: Vec<_> = i
        .panel
        .iter()
        .filter(|r| r.performance.stars.is_some_and(|s| s < 3.0))
        .collect();
    let paid: Vec<_> = below.iter().filter(|r| r.performance.eligible).collect();
    let total = paid.iter().map(|r| r.performance.amount).sum();
    (below.len(), paid.len(), total)
}

fn achievement_denominator(mut scores: Vec<f64>) -> f64 {
    scores.sort_by(|a, b| b.total_cmp(a));
    let top = (scores.len() * 2).div_ceil(100);
    scores[..top].iter().sum::<f64>() / top as f64
}

/// Every federal identification joined to the building the report card holds for it.
///
/// The join is total — all 408 identifications name a building on the 2024-25 card — which is
/// what lets every count below be a count over one population. Asserted rather than assumed in
/// `crates/dispersion/tests/the_list_the_report_card_does_not_explain.rs`.
fn identified_buildings() -> Vec<(
    dispersion::identified::Identified,
    dispersion::building::Building,
)> {
    let by_irn: BTreeMap<String, dispersion::building::Building> =
        dispersion::building::buildings()
            .into_iter()
            .map(|b| (b.irn.clone(), b))
            .collect();
    dispersion::identified::identifications()
        .into_iter()
        .filter_map(|i| by_irn.get(&i.building_irn).cloned().map(|b| (i, b)))
        .collect()
}

/// The buildings on the Comprehensive Support list, with the identification that put them there.
fn comprehensive_support() -> Vec<(
    dispersion::identified::Identified,
    dispersion::building::Building,
)> {
    identified_buildings()
        .into_iter()
        .filter(|(i, _)| i.status == "csi")
        .collect()
}

/// The Comprehensive Support schools identified more than three years ago.
///
/// The trigger `intervention/more-rigorous-interventions` states — CSI, and no exit within three
/// years — read off the one column that carries it. The 2025 cohorts are excluded whichever route
/// they took, including the 42 whose ATSI clock started in 2022: the plan's three years run from
/// the CSI identification, and theirs is 2025.
fn past_the_three_year_exit() -> Vec<(
    dispersion::identified::Identified,
    dispersion::building::Building,
)> {
    comprehensive_support()
        .into_iter()
        .filter(|(i, _)| matches!(i.year_identified.as_str(), "2018" | "2022"))
        .collect()
}

/// The enacted and actual foundation funding appropriation for one fiscal year.
///
/// Line item 200550, which is the line the FY2020 spending controls landed on and the line the
/// guarantee's statutory base is defined against.
///
/// # Panics
///
/// If the series does not carry both an enacted and an actual figure for that year.
fn foundation_in(fiscal_year: u16) -> (f64, f64) {
    let mut enacted = None;
    let mut actual = None;
    for line in project::ledger::appropriations::lines() {
        if line.line_item != "200550" || line.fiscal_year != fiscal_year {
            continue;
        }
        match line.kind.as_str() {
            "enacted" => enacted = Some(line.amount),
            "actual" => actual = Some(line.amount),
            _ => {}
        }
    }
    match (enacted, actual) {
        (Some(enacted), Some(actual)) => (enacted, actual),
        _ => panic!("FY{fiscal_year} does not carry both an enacted and an actual 200550"),
    }
}

/// The first number LSC writes after `marker` in one of its budget analyses.
///
/// Strips a leading `$` and any thousands separators, and stops at a trailing `%`, `,` or `.`
/// that is not a decimal point. Parsed rather than pinned for the reason [`mills_before`] is: a
/// figure the corpus can only get from prose should fail when the prose moves, not agree with a
/// sentence nobody can find.
///
/// # Panics
///
/// If `marker` is absent, or nothing parseable follows it.
fn amount_after(passage: &str, marker: &str) -> f64 {
    let at = passage
        .find(marker)
        .unwrap_or_else(|| panic!("the analysis no longer says {marker:?}"));
    let rest = &passage[at + marker.len()..];
    let token: String = rest
        .trim_start()
        .trim_start_matches('$')
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .filter(|c| *c != ',')
        .collect();
    token
        .trim_end_matches('.')
        .parse()
        .unwrap_or_else(|_| panic!("what follows {marker:?} is not a number: {token:?}"))
}

/// The millage LSC states immediately before `tail` in one of its budget analyses.
///
/// These figures are read out of committed prose rather than computed over a table, which is the
/// only form the charge-off's last two eras exist in: Ohio Laws' version archive for
/// R.C. 3317.022 begins after the mechanism was retired. Parsing rather than pinning a literal is
/// what makes the figure a check — if LSC's sentence changes shape under a re-extract, this fails
/// instead of quietly agreeing with a number nobody can find any more.
///
/// # Panics
///
/// If `tail` is absent, or the token before the nearest preceding `mill` is not a number. Both
/// mean the extract and this reader have come apart.
fn mills_before(passage: &str, tail: &str) -> f64 {
    let at = passage
        .find(tail)
        .unwrap_or_else(|| panic!("the analysis no longer says {tail:?}"));
    let (head, _) = passage[..at]
        .rsplit_once(" mill")
        .unwrap_or_else(|| panic!("no millage is stated before {tail:?}"));
    head.split_whitespace()
        .next_back()
        .and_then(|token| token.parse().ok())
        .unwrap_or_else(|| panic!("what precedes 'mill' before {tail:?} is not a number"))
}

/// LSC's account of the local share the Evidence-Based Model set, as one paragraph.
fn evidence_based_local_share() -> String {
    project::greenbook::greenbook("hb1").around("Under prior law, school districts contributed", 9)
}

/// LSC's account of what the Bridge formula's state share index implies, as one paragraph.
fn bridge_implied_charge_off() -> String {
    project::greenbook::greenbook("hb59").around("Prior to FY 2010, the school funding formula", 11)
}

/// LSC's account of the targeted assistance payment the Fair School Funding Plan enacted.
///
/// The whole component, both tiers, from the sentence that says what it replaced. Flattened
/// rather than line-anchored because the millage sentences run three of the publisher's lines
/// each with a page footer inside one of them.
fn plan_targeted_assistance() -> String {
    project::greenbook::greenbook("hb110").flat()
}

/// LSC's account of the targeted assistance the plan replaced — Am. Sub. H.B. 59 of the 130th.
fn first_targeted_assistance() -> String {
    project::greenbook::greenbook("hb59").flat()
}

/// LSC's account of capacity aid, the component the plan's capacity tier descends from.
fn capacity_aid(bill: &str) -> String {
    project::greenbook::greenbook(bill).flat()
}

/// The first `n` dollar amounts LSC prints after `marker`, in document order.
///
/// LSC publishes a parameter schedule as a table, and a table survives PDF extraction as a run
/// of `$`-prefixed tokens in reading order with the row labels between them. Reading the run is
/// what makes a figure below a check on the committed extract rather than a literal retyped from
/// it — if the table loses a column under a re-extract, the arithmetic stops agreeing instead of
/// quietly agreeing with a number nobody can find.
///
/// # Panics
///
/// If `marker` is absent, or fewer than `n` amounts follow it. Both mean the extract and this
/// reader have come apart.
fn dollars_after(passage: &str, marker: &str, n: usize) -> Vec<f64> {
    let at = passage
        .find(marker)
        .unwrap_or_else(|| panic!("the analysis no longer says {marker:?}"));
    let out: Vec<f64> = passage[at + marker.len()..]
        .split_whitespace()
        .filter_map(|token| token.strip_prefix('$'))
        .filter_map(|token| token.replace(',', "").parse().ok())
        .take(n)
        .collect();
    assert_eq!(out.len(), n, "only {} amounts follow {marker:?}", out.len());
    out
}

/// A three-column LSC schedule's last column — the third of every group of three.
fn last_column(amounts: &[f64]) -> Vec<f64> {
    amounts.iter().skip(2).step_by(3).copied().collect()
}

/// The six special education weights of the Evidence-Based Model, off H.B. 1's own table.
///
/// The table prints prior law beside current law, so each row carries two numbers and the second
/// is the one the 128th enacted. The Fair School Funding Plan's six are these six times a single
/// constant, which is the figure this feeds.
fn evidence_based_weights() -> [f64; 6] {
    let lsc = project::greenbook::greenbook("hb1").flat();
    let rows = [
        "One \u{2013} Speech only",
        "Two \u{2013} Specific learning disabled, developmentally disabled, other health \u{2013} minor",
        "Three \u{2013} Hearing impaired, severe behavior disabled",
        "Four \u{2013} Vision impaired, other health \u{2013} major",
        "Five \u{2013} Orthopedically disabled, multi-disabled",
        "Six \u{2013} Autism, traumatic brain injury, both visually and hearing impaired",
    ];
    let mut out = [0.0; 6];
    for (slot, row) in out.iter_mut().zip(rows) {
        let at = lsc
            .find(row)
            .unwrap_or_else(|| panic!("H.B. 1's weight table no longer has {row:?}"));
        let pair: Vec<f64> = lsc[at + row.len()..]
            .split_whitespace()
            .filter_map(|token| token.parse().ok())
            .take(2)
            .collect();
        assert_eq!(pair.len(), 2, "{row:?} no longer carries two weights");
        *slot = pair[1];
    }
    out
}

/// The six special education per-pupil amounts of FY2017, the last year they moved.
fn special_education_amounts_fy2017() -> Vec<f64> {
    let lsc = project::greenbook::greenbook("hb64").flat();
    last_column(&dollars_after(
        &lsc,
        "Special Education Per-Pupil Amounts Category FY 2015 FY 2016 FY 2017",
        18,
    ))
}

/// The five career-technical amounts and associated services in FY2017, likewise.
fn career_technical_amounts_fy2017() -> Vec<f64> {
    let lsc = project::greenbook::greenbook("hb64").flat();
    last_column(&dollars_after(
        &lsc,
        "Career-Technical Education and Associated Services Per-Pupil Amounts Category FY 2015 \
         FY 2016 FY 2017",
        18,
    ))
}

/// The single divisor a schedule of dollar amounts implies against a schedule of weights.
///
/// A schedule converted by dividing every amount by one number leaves the six agreeing to the
/// rounding of a four-decimal weight. This returns their mean; the spread is what
/// `crates/project/tests/the_weights_that_were_dollar_amounts.rs` asserts on.
fn implied_divisor(dollars: &[f64], weights: &[f64]) -> f64 {
    assert_eq!(dollars.len(), weights.len());
    dollars.iter().zip(weights).map(|(d, w)| d / w).sum::<f64>() / dollars.len() as f64
}

/// The statewide average base cost per pupil in the plan's first year, as LSC states it.
fn base_cost_fy2022() -> f64 {
    amount_after(
        &project::greenbook::greenbook("hb110").flat(),
        "is used in the calculation of a number of other formula components, is estimated to be ",
    )
}

/// One appropriation line's enacted amount in one fiscal year.
///
/// # Panics
///
/// If the committed series holds no enacted row for that line and year.
fn enacted_line(line_item: &str, fiscal_year: u16) -> f64 {
    project::ledger::appropriations::lines()
        .into_iter()
        .find(|line| {
            line.line_item == line_item && line.fiscal_year == fiscal_year && line.kind == "enacted"
        })
        .unwrap_or_else(|| panic!("no enacted {line_item} for FY{fiscal_year}"))
        .amount
}

/// The FY2006 base funding supplement table, summed from its four itemised rows.
///
/// Summed rather than read off the table's own total, so that the figure is a check on the four
/// components and not a restatement of a number printed beside them.
fn base_funding_supplements_fy2006() -> f64 {
    let lsc = project::greenbook::greenbook("hb66").flat();
    let rows = [
        "Academic Intervention Services",
        "Professional Development",
        "Data-Based Decision Making",
        "Professional Development \u{2013} Data-Based Decision Making",
    ];
    let at = lsc
        .find("Table 4: Base Funding Supplements Per Pupil, FY 2006 and FY 2007")
        .expect("H.B. 66 no longer carries the base funding supplement table");
    let table = &lsc[at..];
    rows.iter()
        .map(|row| {
            let start = table
                .find(row)
                .unwrap_or_else(|| panic!("the table no longer has {row:?}"));
            dollars_after(&table[start..], row, 1)[0]
        })
        .sum()
}

/// Every building IRN on any of the three federal lists.
fn federally_listed() -> BTreeSet<String> {
    dispersion::identified::identifications()
        .into_iter()
        .map(|i| i.building_irn)
        .collect()
}

/// The 2024-25 Performance Index of a rated building.
///
/// # Panics
///
/// Only when called on a building [`dispersion::building::by_index`] did not drop, which is what
/// every caller here does.
fn building_index(b: &dispersion::building::Building) -> f64 {
    b.performance_index.expect("by_index drops the unrated")
}

/// The median of a sample. Sorted here rather than by the caller, because every call site below
/// wants the same thing and one of them getting the sort wrong would move a figure quietly.
fn median(mut sample: Vec<f64>) -> f64 {
    sample.sort_by(f64::total_cmp);
    sample[sample.len() / 2]
}

/// The most common achievement star rating, and how many districts are rated at all.
fn modal_achievement_rating() -> (usize, usize) {
    let mut counts = [0usize; 6];
    let mut rated = 0;
    for card in dispersion::report_card::report_cards() {
        if let Some(stars) = card.achievement_stars {
            counts[stars as usize] += 1;
            rated += 1;
        }
    }
    (counts.into_iter().max().unwrap_or(0), rated)
}

fn outcome_correlation(
    i: &Inputs,
    x: fn(
        &dispersion::report_card::ReportCard,
        Option<&dispersion::profile::ProfileDistrict>,
    ) -> Option<f64>,
    y: fn(&dispersion::report_card::ReportCard) -> Option<f64>,
) -> f64 {
    let (mut xs, mut ys) = (Vec::new(), Vec::new());
    for (card, profile) in &i.outcomes {
        if let (Some(a), Some(b)) = (x(card, profile.as_ref()), y(card)) {
            xs.push(a);
            ys.push(b);
        }
    }
    dispersion::wealth_neutrality(&xs, &ys)
        .expect("a paired series")
        .correlation
        .abs()
}

/// The dispersion of operating expenditure per pupil across the profile report.
///
/// Over the 605 districts that report one — the panel is 606 and one district publishes no
/// operating expenditure at all, which is a fact about coverage rather than about spending.
/// One jurisdiction's row of the equity-factor table.
///
/// # Panics
///
/// If the national panel does not carry `state`.
fn equity_of<'a>(i: &'a Inputs, state: &str) -> &'a dispersion::equity_factor::StateEquity {
    i.equity
        .iter()
        .find(|s| s.state == state)
        .expect("the national panel carries this jurisdiction")
}

fn operating_dispersion(i: &Inputs) -> dispersion::Dispersion {
    let column = dispersion::profile::column(&i.profile, |d| d.operating_expenditure_per_pupil);
    dispersion::Dispersion::of(&column).expect("districts report operating expenditure")
}

/// The median of a profile-report column, over the districts reporting it.
fn profile_median(
    i: &Inputs,
    pick: fn(&dispersion::profile::ProfileDistrict) -> Option<f64>,
) -> f64 {
    let column = dispersion::profile::column(&i.profile, pick);
    dispersion::Dispersion::of(&column)
        .expect("the profile report is not empty")
        .median
}

/// Every district's millage reduction as a fraction of what its voters approved, ascending.
///
/// The per-district ratio, and the median of *that* — not one minus the ratio of the two medians,
/// which is a different and meaningless quantity.
fn millage_reductions(i: &Inputs) -> Vec<f64> {
    let mut out: Vec<f64> = i
        .profile
        .iter()
        .filter_map(|d| Some(1.0 - d.effective_class1_millage? / d.current_operating_millage?))
        .collect();
    out.sort_by(f64::total_cmp);
    out
}

/// Districts at exactly twenty mills on a chosen rate, in one tax year of Table SD-1.
fn sd1_at_twenty(
    i: &Inputs,
    tax_year: u16,
    pick: fn(&dispersion::sd1::TaxRow) -> Option<f64>,
) -> usize {
    i.sd1
        .iter()
        .filter(|row| row.tax_year == tax_year)
        .filter(|row| pick(row).is_some_and(|rate| (rate - 20.0).abs() < 0.005))
        .count()
}

/// The Class I rate implied by the charges, in mills.
///
/// TY2023's workbook publishes the rate to two decimals and the shortfalls below are read in the
/// fifth, so this recomputes rather than reading the column.
fn sd1_class1_rate(row: &dispersion::sd1::TaxRow) -> Option<f64> {
    let value = row.class1_value?;
    let taxes = row.class1_taxes_charged?;
    (value > 0.0).then(|| taxes / value * 1000.0)
}

/// How many districts carry a joint vocational operating levy in one tax year, and how many
/// carry none.
///
/// The difference between Table SD-1's two taxes-charged columns is the joint vocational levy on
/// a district's own parcels, so it is also the membership list the abstract never states.
fn joint_vocational_membership(i: &Inputs, tax_year: u16) -> (usize, usize) {
    let rows: Vec<&dispersion::sd1::TaxRow> = i
        .sd1
        .iter()
        .filter(|row| row.tax_year == tax_year)
        .collect();
    let members = rows
        .iter()
        .filter(|row| dispersion::sd1::joint_vocational_tax(row).is_some_and(|tax| tax > 0.5))
        .count();
    (members, rows.len() - members)
}

/// The two class rates one county's districts fit, taken as one joint vocational district's.
fn county_joint_vocational_fit(
    i: &Inputs,
    county: &str,
    tax_year: u16,
) -> Option<dispersion::sd1::JointVocationalFit> {
    let group: Vec<&dispersion::sd1::TaxRow> = i
        .sd1
        .iter()
        .filter(|row| row.tax_year == tax_year && row.county.eq_ignore_ascii_case(county))
        .collect();
    dispersion::sd1::joint_vocational_fit(&group)
}

/// How far one district sits below the twenty-mill floor, in mills.
fn floor_shortfall(i: &Inputs, irn: &str, tax_year: u16) -> f64 {
    i.sd1
        .iter()
        .find(|row| row.irn == irn && row.tax_year == tax_year)
        .and_then(sd1_class1_rate)
        .map_or(f64::NAN, |rate| 20.0 - rate)
}

/// Districts below the floor on the computed rate in every tax year the abstract carries.
///
/// The persistence is the point: R.C. 319.301(D)(1) recomputes the reduction percentage annually,
/// so a rounding residual would not reproduce.
fn below_the_floor_in_every_tax_year(i: &Inputs) -> usize {
    let years = dispersion::sd1::tax_years();
    let mut by_district: BTreeMap<&str, usize> = BTreeMap::new();
    for row in &i.sd1 {
        if sd1_class1_rate(row).is_some_and(|rate| rate < 20.0 - 5e-5) {
            *by_district.entry(row.irn.as_str()).or_default() += 1;
        }
    }
    by_district
        .values()
        .filter(|seen| **seen == years.len())
        .count()
}

/// One year of the relief control series.
fn relief_aggregate(fiscal_year: u16, pick: fn(&project::esser::YearOfBoth) -> Option<f64>) -> f64 {
    project::esser::federal_against_the_general_fund()
        .get(&fiscal_year)
        .and_then(pick)
        .unwrap_or(f64::NAN)
}

/// Eastland-Fairfield's row in the F-33 Ohio panel for one fiscal year.
fn eastland_fairfield(fiscal_year: u16) -> Option<dispersion::ohio_panel::PanelRow> {
    dispersion::ohio_panel::panel()
        .into_iter()
        .find(|row| row.irn == "051003" && row.fiscal_year == fiscal_year)
}

/// One field of its last closed year in the Auditor's district finances.
fn eastland_fairfield_finances(
    pick: fn(&project::finances::YearRecord) -> Option<edfund_core::Dollars>,
) -> f64 {
    let panel = project::finances::finances();
    project::finances::for_district(&panel, "051003")
        .and_then(|d| d.years.last())
        .and_then(pick)
        .unwrap_or(f64::NAN)
}

/// Rows in the F-33 Ohio panel belonging to a joint vocational district.
fn joint_vocational_panel_rows() -> usize {
    let joint: BTreeSet<String> = dispersion::lea_directory::joint_vocational_districts()
        .into_keys()
        .collect();
    dispersion::ohio_panel::panel()
        .into_iter()
        .filter(|row| joint.contains(&row.irn))
        .count()
}

/// The median comparable district's current spending per pupil in one fiscal year.
fn comparable_median_spending_per_pupil(fiscal_year: u16) -> f64 {
    let mut per_pupil: Vec<f64> = dispersion::ohio_panel::panel()
        .into_iter()
        .filter(|row| row.comparable && row.fiscal_year == fiscal_year && row.enrollment > 0.0)
        .filter_map(|row| Some(row.current_spending? / row.enrollment))
        .collect();
    per_pupil.sort_by(f64::total_cmp);
    per_pupil
        .get(per_pupil.len() / 2)
        .copied()
        .unwrap_or(f64::NAN)
}

/// Guarantee status, Performance Index and poverty over the districts carrying all three.
fn guarantee_against_achievement(i: &Inputs) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let (mut guarantee, mut index, mut poverty) = (Vec::new(), Vec::new(), Vec::new());
    for record in &i.joined {
        if let (Some(pi), Some(ed)) = (
            record.outcome.performance_index,
            record.economically_disadvantaged,
        ) {
            guarantee.push(if record.on_guarantee() { 1.0 } else { 0.0 });
            index.push(pi);
            poverty.push(ed);
        }
    }
    (guarantee, index, poverty)
}

/// The median Performance Index of the districts on, or off, the guarantee.
fn median_index(i: &Inputs, on_guarantee: bool) -> f64 {
    let mut values: Vec<f64> = i
        .joined
        .iter()
        .filter(|r| r.on_guarantee() == on_guarantee)
        .filter_map(|r| r.outcome.performance_index)
        .collect();
    values.sort_by(f64::total_cmp);
    dispersion::median(&values).expect("districts on both sides of the guarantee")
}

/// Districts big enough for an identification *rate* to mean anything.
///
/// The same hundred-pupil floor `project`'s own test applies: below it, one identified pupil
/// Districts the guarantee actually pays, which is the population three nodes state a count for.
fn on_the_guarantee(i: &Inputs) -> Vec<&DistrictRecord> {
    i.panel.iter().filter(|r| r.on_guarantee()).collect()
}

/// Statewide special education ADM and aid, by category. Index 0 is Category 1.
fn special_education_totals(i: &Inputs) -> ([f64; 6], [f64; 6]) {
    let mut adm = [0.0; 6];
    let mut aid = [0.0; 6];
    for record in &i.panel {
        for k in 0..6 {
            adm[k] += record.special_education.adm[k];
            aid[k] += record.special_education.aid[k];
        }
    }
    (adm, aid)
}

/// One category's share of the programme, by pupils and by money.
fn special_education_shares(i: &Inputs, category: usize) -> (f64, f64) {
    let (adm, aid) = special_education_totals(i);
    let k = category - 1;
    (
        adm[k] / adm.iter().sum::<f64>(),
        aid[k] / aid.iter().sum::<f64>(),
    )
}

/// moves a district's share by a percentage point and the rate stops describing practice.
fn sizeable(i: &Inputs) -> Vec<&DistrictRecord> {
    i.panel
        .iter()
        .filter(|r| r.current_year_adm >= 100.0)
        .collect()
}

/// The share of a district's roll it identifies as gifted.
fn identified_share(r: &DistrictRecord) -> f64 {
    (r.gifted.fte_k8 + r.gifted.fte_9_12) / r.current_year_adm
}

/// The performance supplement per pupil by economically-disadvantaged quintile, least-poor first,
/// with the share of each band that qualifies.
///
/// Quintiled on `dpia.percentage` and paid per `categorical_enrolled_adm`, which is the pairing
/// the supplement is actually computed over.
fn performance_quintiles(i: &Inputs) -> [(f64, f64); 5] {
    let mut sample: Vec<(f64, f64, bool)> = i
        .panel
        .iter()
        .filter_map(|r| {
            let adm = r.categorical_enrolled_adm;
            (adm > 0.0 && r.dpia.percentage > 0.0).then(|| {
                (
                    r.dpia.percentage,
                    r.performance.amount / adm,
                    r.performance.eligible,
                )
            })
        })
        .collect();
    sample.sort_by(|a, b| a.0.total_cmp(&b.0));
    let size = sample.len() / 5;
    let mut out = [(0.0, 0.0); 5];
    for (k, slot) in out.iter_mut().enumerate() {
        let band = if k == 4 {
            &sample[4 * size..]
        } else {
            &sample[k * size..(k + 1) * size]
        };
        let per_pupil = band.iter().map(|b| b.1).sum::<f64>() / band.len() as f64;
        let qualifying = band.iter().filter(|b| b.2).count() as f64 / band.len() as f64;
        *slot = (per_pupil, qualifying);
    }
    out
}

/// Districts that missed the three-per-cent enrolment-growth cliff by less than three tenths of a
/// point, with what clearing it would have paid each.
fn just_below_the_growth_cliff(i: &Inputs) -> Vec<(&str, f64, f64)> {
    let mut out: Vec<(&str, f64, f64)> = i
        .panel
        .iter()
        .filter(|r| !r.supplements.growth_eligible && r.supplements.enrollment_change > 0.027)
        .map(|r| {
            (
                r.name.as_str(),
                r.supplements.enrollment_change,
                r.current_year_adm
                    * project::panel::supplements::ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL,
            )
        })
        .collect();
    out.sort_by(|a, b| b.1.total_cmp(&a.1));
    out
}

/// Ohio's row of the Census state table.
fn ohio(i: &Inputs) -> &dispersion::census_states::StateFinance {
    i.states
        .iter()
        .find(|s| s.name == "Ohio")
        .expect("the state table holds Ohio")
}

/// A national figure per pupil, in dollars, from the survey's thousands.
fn national_per_pupil(
    i: &Inputs,
    pick: fn(&dispersion::census_states::StateFinance) -> f64,
) -> f64 {
    i.states.iter().map(pick).sum::<f64>() * 1_000.0
        / i.states.iter().map(|s| s.enrollment).sum::<f64>()
}

/// A national share of total revenue: the aggregate, not the mean of fifty-one state shares.
fn national_share(i: &Inputs, pick: fn(&dispersion::census_states::StateFinance) -> f64) -> f64 {
    let total: f64 = i.states.iter().map(|s| s.total_revenue).sum();
    i.states.iter().map(pick).sum::<f64>() / total
}

/// Districts whose regime difference has a total, i.e. where both regimes can be valued.
fn with_a_total(diffs: &[RegimeDiff]) -> usize {
    diffs
        .iter()
        .filter(|d| d.total_difference().is_some())
        .count()
}

/// Districts the charge-off would have left with no base cost aid at all.
fn zeroed(diffs: &[RegimeDiff]) -> usize {
    diffs
        .iter()
        .filter(|d| d.predecessor_total == Some(0.0))
        .count()
}

/// The median total difference per pupil, over the districts that have one.
fn median_difference(diffs: &[RegimeDiff]) -> f64 {
    let mut totals: Vec<f64> = diffs
        .iter()
        .filter_map(RegimeDiff::total_difference)
        .collect();
    totals.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a regime difference"));
    totals[totals.len() / 2]
}

/// Every district carrying an open-enrolment clawback, as `(count, total, largest, second)`.
fn clawback(panel: &[DistrictRecord]) -> (usize, f64, f64, f64) {
    let mut amounts: Vec<f64> = panel
        .iter()
        .map(|r| r.transition.open_enrollment_adjustment)
        .filter(|a| *a > 0.0)
        .collect();
    amounts.sort_by(|a, b| b.partial_cmp(a).expect("no NaN in a clawback"));
    let total = amounts.iter().sum();
    let at = |i: usize| amounts.get(i).copied().unwrap_or(0.0);
    (amounts.len(), total, at(0), at(1))
}

/// Every figure this repository exports for the corpus to be checked against.
///
/// Ordered by owning crate, then by what the figure is about. The order is the manifest's order,
/// so keep it stable: reordering rewrites a committed artefact for no reason.
pub static FIGURES: &[Figure] = &[
    // ---- crates/bundle -----------------------------------------------------------------------
    //
    // The report card's statewide outcome block, which two `metric/` nodes quote between them:
    // the federal-share distribution on `per-pupil-operating-expenditure`, and the two value-added
    // agreement statistics on `progress-value-added`.
    //
    // Reached through `build::outcome_statewide` rather than `build()`, which would rebuild every
    // panel in the bundle to read one struct.
    Figure {
        key: "bundle/median-federal-share-of-operating-spending",
        owner: "crates/bundle",
        unit: Unit::Share,
        label: "The median district's federal share of operating spending, 2024-25",
        pinned: 0.042_038_401_6,
        tolerance: 0.000_01,
        compute: |i| i.outcome_block.median_federal_share,
    },
    Figure {
        key: "bundle/max-federal-share-of-operating-spending",
        owner: "crates/bundle",
        unit: Unit::Share,
        label: "The highest federal share in the state \u{2014} the most exposed district in \
                Ohio to a decision Ohio does not make",
        pinned: 0.290_419_691_9,
        tolerance: 0.000_01,
        compute: |i| i.outcome_block.max_federal_share,
    },
    Figure {
        key: "bundle/districts-over-a-tenth-federal",
        owner: "crates/bundle",
        unit: Unit::Count,
        label: "Districts where more than a tenth of operating spending is federal",
        pinned: 49.0,
        tolerance: 0.0,
        compute: |i| i.outcome_block.federal_share_above_tenth as f64,
    },
    // Both halves of the correlation, because the raw one must never be quoted alone: federal
    // money is allocated substantially by poverty, so raw is very largely the poverty
    // relationship read backwards. A magnitude each, since prose writes the minus sign and the
    // numeral reader cannot see it.
    Figure {
        key: "bundle/federal-share-against-performance-raw",
        owner: "crates/bundle",
        unit: Unit::Ratio,
        label: "Federal share against the Performance Index, raw",
        pinned: 0.546_227_415_1,
        tolerance: 0.000_01,
        compute: |i| i.outcome_block.federal_share_vs_performance_raw.abs(),
    },
    Figure {
        key: "bundle/federal-share-against-performance-controlled",
        owner: "crates/bundle",
        unit: Unit::Ratio,
        label: "The same, holding economic disadvantage constant \u{2014} the figure that says \
                anything, and a fifth the size of the one that does not",
        pinned: 0.114_562_368_9,
        tolerance: 0.000_01,
        compute: |i| i.outcome_block.federal_share_vs_performance.abs(),
    },
    // The two growth measures, for `progress-value-added`.
    Figure {
        key: "bundle/growth-measures-determinate",
        owner: "crates/bundle",
        unit: Unit::Count,
        label: "Districts printing a non-zero value on both the one-year and three-year growth \
                measures \u{2014} the denominator the disagreement is counted over",
        pinned: 534.0,
        tolerance: 0.0,
        compute: |i| i.outcome_block.growth_measures_determinate as f64,
    },
    Figure {
        key: "bundle/growth-measures-disagree",
        owner: "crates/bundle",
        unit: Unit::Count,
        label: "Of those, how many point opposite ways",
        pinned: 44.0,
        tolerance: 0.0,
        compute: |i| i.outcome_block.growth_measures_disagree as f64,
    },
    Figure {
        key: "bundle/growth-measures-disagree-materially",
        owner: "crates/bundle",
        unit: Unit::Count,
        label: "How many disagree with both magnitudes past 0.05 \u{2014} it is zero, which is \
                what makes the disagreement readable as noise around a district sitting on zero",
        pinned: 0.0,
        tolerance: 0.0,
        compute: |i| i.outcome_block.growth_measures_disagree_materially as f64,
    },
    Figure {
        key: "bundle/growth-measure-agreement",
        owner: "crates/bundle",
        unit: Unit::Ratio,
        label: "The correlation between the one-year and three-year growth measures across the \
                districts determinate on both",
        pinned: 0.903_134_341_1,
        tolerance: 0.000_01,
        compute: |i| i.outcome_block.growth_measure_agreement,
    },
    // ---- crates/deflator ---------------------------------------------------------------------
    //
    // The one figure family here that is a time series rather than a distribution over districts.
    // `metric/per-pupil-operating-expenditure` states the whole restated series and then five
    // sentences about its shape; the sentences are what is exported, because the series itself is
    // in the node as a table and a reader acts on the claims made about it.
    //
    // Every one of these is a `[verified]` claim that stood on nothing a corpus check could reach.
    Figure {
        key: "deflator/real-operating-expenditure-growth",
        owner: "crates/deflator",
        unit: Unit::Share,
        label: "Real growth in Ohio operating expenditure per pupil, FY2000 to FY2022, \
                including federal relief",
        pinned: 0.261_147_73,
        tolerance: 0.000_01,
        compute: |_| deflator::ohio_epp::real_growth_from_fy2000(),
    },
    Figure {
        key: "deflator/real-operating-expenditure-growth-excluding-relief",
        owner: "crates/deflator",
        unit: Unit::Share,
        label: "The same growth on the FY2022 figure that excludes federal COVID relief funds",
        pinned: 0.193_536_25,
        tolerance: 0.000_01,
        compute: |_| deflator::ohio_epp::real_growth_from_fy2000_ex_relief(),
    },
    Figure {
        key: "deflator/nominal-operating-expenditure-growth",
        owner: "crates/deflator",
        unit: Unit::Share,
        label: "The same span undeflated \u{2014} the figure the same record supports and which \
                reads four times larger",
        pinned: 1.167_586_69,
        tolerance: 0.000_01,
        compute: |_| {
            let series = deflator::ohio_epp::nominal_series();
            series[series.len() - 1].dollars / series[0].dollars - 1.0
        },
    },
    Figure {
        key: "deflator/real-operating-expenditure-fy2000",
        owner: "crates/deflator",
        unit: Unit::Dollars,
        label: "Ohio operating expenditure per pupil, FY2000, in constant FY2022 dollars",
        pinned: 12_142.907_28,
        tolerance: 0.01,
        compute: |_| deflator::ohio_epp::real_at(edfund_core::FiscalYear(2000)),
    },
    Figure {
        key: "deflator/real-operating-expenditure-peak",
        owner: "crates/deflator",
        unit: Unit::Dollars,
        label: "The real peak of the series \u{2014} FY2020, two years before the nominal peak",
        pinned: 15_746.733_67,
        tolerance: 0.01,
        compute: |_| deflator::ohio_epp::real_at(edfund_core::FiscalYear(2020)),
    },
    // A magnitude with the direction in the key, per the convention the second tranche set: a
    // signed pin cannot be matched, because the numeral reader sees `7%` and never the minus.
    Figure {
        key: "deflator/real-operating-expenditure-trough-decline",
        owner: "crates/deflator",
        unit: Unit::Share,
        label: "How far the real series fell from FY2010 to FY2014 \u{2014} the decline the \
                nominal series conceals entirely",
        pinned: 0.069_168_25,
        tolerance: 0.000_01,
        compute: |_| {
            1.0 - deflator::ohio_epp::real_at(edfund_core::FiscalYear(2014))
                / deflator::ohio_epp::real_at(edfund_core::FiscalYear(2010))
        },
    },
    Figure {
        key: "deflator/operating-expenditure-fy2022-excluding-relief",
        owner: "crates/deflator",
        unit: Unit::Dollars,
        label: "FY2022 operating expenditure per pupil excluding federal COVID relief funds",
        pinned: 14_493.0,
        tolerance: 0.0,
        compute: |_| deflator::ohio_epp::FY2022_EX_RELIEF,
    },
    Figure {
        key: "deflator/relief-funds-per-pupil-fy2022",
        owner: "crates/deflator",
        unit: Unit::Dollars,
        label: "The gap between the two FY2022 figures \u{2014} federal relief, per pupil",
        pinned: 821.0,
        tolerance: 0.0,
        compute: |_| {
            let series = deflator::ohio_epp::nominal_series();
            series[series.len() - 1].dollars - deflator::ohio_epp::FY2022_EX_RELIEF
        },
    },
    Figure {
        key: "deflator/cpi-growth-fy2000-fy2022",
        owner: "crates/deflator",
        unit: Unit::Share,
        label: "CPI-U growth over the same span, which is what separates the two readings",
        pinned: 0.718_741_08,
        tolerance: 0.000_01,
        compute: |_| {
            deflator::CpiSeries::cpi_u_june()
                .index_growth(edfund_core::FiscalYear(2000), edfund_core::FiscalYear(2022))
                .expect("both endpoints are in the CPI series")
        },
    },
    // ---- crates/dispersion -------------------------------------------------------------------
    //
    // `ohio_by_local_wealth` quartiles the comparable Ohio districts of the Census F-33 panel by
    // local revenue per pupil and reads the other two levels of government against it. The equity
    // node prints the whole 4x3 table and then three sentences about it; the sentences are what
    // is exported here, because a reader acts on those and the table is the working.
    Figure {
        key: "dispersion/comparable-ohio-districts-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Ohio districts the Census F-33 comparability filter admits, FY2022",
        pinned: 611.0,
        tolerance: 0.0,
        compute: |i| i.quartiles.iter().map(|q| q.districts).sum::<usize>() as f64,
    },
    Figure {
        key: "dispersion/poorest-quartile-local-revenue-per-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Local revenue per pupil in the poorest quartile of Ohio districts, FY2022",
        pinned: 4_342.0,
        tolerance: 1.0,
        compute: |i| i.quartiles[0].local_per_pupil,
    },
    Figure {
        key: "dispersion/richest-quartile-local-revenue-per-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Local revenue per pupil in the richest quartile of Ohio districts, FY2022",
        pinned: 13_932.0,
        tolerance: 1.0,
        compute: |i| i.quartiles[3].local_per_pupil,
    },
    Figure {
        key: "project/bienniums-coupling-no-dollar-to-a-rating",
        owner: "crates/project",
        unit: Unit::Count,
        label: "LSC greenbooks from FY2002 to FY2024 carrying no rating-driven formula payment \
                at all, of twelve",
        pinned: 8.0,
        tolerance: 0.0,
        compute: |_| project::rating_payments::quiet().len() as f64,
    },
    Figure {
        key: "project/rating-driven-formula-couplings",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Points at which an Ohio accountability rating has determined a formula payment, \
                FY2002 to FY2027",
        pinned: 5.0,
        tolerance: 0.0,
        compute: |_| project::rating_payments::formula_couplings().len() as f64,
    },
    Figure {
        key: "project/closing-the-achievement-gap-rate",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The share of the formula amount Closing the Achievement Gap paid per pupil in \
                FY2008, in percentage points — Ohio's first rating-driven formula payment",
        pinned: 0.15,
        tolerance: 0.0001,
        compute: |_| {
            // Tied to the greenbook rather than restated: LSC writes the rate twice, as a
            // multiplier and as a percentage, and this is the second.
            project::rating_payments::stated("hb119", "on a per student basis, 0.15% of the", 0.15)
        },
    },
    Figure {
        key: "project/closing-the-achievement-gap-districts",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts qualifying for it — those with both an academic distress index and a \
                poverty index of 1.0 or above",
        pinned: 31.0,
        tolerance: 0.0,
        compute: |_| {
            project::rating_payments::stated(
                "hb119",
                "Thirty-one districts have both academic distress indices and poverty indices greater than or equal to one",
                31.0,
            )
        },
    },
    Figure {
        key: "project/general-fund-transfers-fy2020",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Transfers, advances and note proceeds into Ohio districts' general funds, FY2020 \
                — the year before the relief window",
        pinned: 281_900_000.0,
        tolerance: 500_000.0,
        compute: |i| i.transfers[&2020].transfers(),
    },
    Figure {
        key: "project/general-fund-transfers-fy2024",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The same line at its peak, FY2024",
        pinned: 720_500_000.0,
        tolerance: 500_000.0,
        compute: |i| i.transfers[&2024].transfers(),
    },
    Figure {
        key: "project/general-fund-transfers-fy2025",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And in FY2025, the first year after the grants ended",
        pinned: 433_500_000.0,
        tolerance: 500_000.0,
        compute: |i| i.transfers[&2025].transfers(),
    },
    Figure {
        key: "project/cash-build-up-against-relief",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "How closely a district's FY2021-FY2024 cash build-up tracks the federal relief it \
                received — the correlation, across 606 districts",
        pinned: 0.0751,
        tolerance: 0.0005,
        compute: |_| project::transfers::build_up_against_relief(),
    },
    Figure {
        key: "project/excess-transfers-share-of-the-build-up",
        owner: "crates/project",
        unit: Unit::Share,
        label: "Transfers into the general fund across FY2021-FY2024 above both baseline years' \
                level, as a share of the cash build-up — the ceiling on what relief can explain",
        pinned: 0.3131,
        tolerance: 0.0005,
        compute: |i| i.attribution_both.excess_share(),
    },
    Figure {
        key: "project/relief-attributable-share-of-the-build-up",
        owner: "crates/project",
        unit: Unit::Share,
        label: "And the share the relief itself explains on that baseline, by the slope a few \
                large districts cannot pull",
        pinned: 0.0638,
        tolerance: 0.0005,
        compute: |i| i.attribution_both.attributable().0,
    },
    Figure {
        key: "project/relief-attributable-share-at-its-widest",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The largest share any baseline and slope the corpus can defend attributes to the \
                relief — the FY2020 baseline read by least squares",
        pinned: 0.3938,
        tolerance: 0.0005,
        compute: |i| i.attribution_before.attributable().1,
    },
    Figure {
        key: "dispersion/island-local-revenue-per-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Kelleys Island Local SD's local revenue per pupil, FY2023 — four times the next \
                district in Ohio, and the whole of the break the corpus recorded at FY2023",
        pinned: 214_400.0,
        tolerance: 50.0,
        compute: |_| {
            dispersion::ohio_panel::panel()
                .iter()
                .find(|r| r.irn == "046797" && r.fiscal_year == 2023)
                .map_or(f64::NAN, |r| r.local_revenue / r.enrollment)
        },
    },
    Figure {
        key: "dispersion/equalization-band-narrowest",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The least of the local gap the state closes in any year from FY2012 to FY2024",
        pinned: 0.4002,
        tolerance: 0.0005,
        compute: |i| band(i).0,
    },
    Figure {
        key: "dispersion/equalization-band-widest",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "And the most, in the same thirteen years — the band the series does not leave",
        pinned: 0.4884,
        tolerance: 0.0005,
        compute: |i| band(i).1,
    },
    Figure {
        key: "dispersion/state-share-of-the-local-gap-fy2024",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The share of the local gap state aid closes in FY2024, the last year the survey \
                reaches",
        pinned: 0.4390,
        tolerance: 0.0005,
        compute: |i| i.equalization[&2024].state_share(),
    },
    Figure {
        key: "regime-diff/poorest-quartile-at-the-millage-floor",
        owner: "crates/regime-diff",
        unit: Unit::Share,
        label: "The poorest quartile of Ohio districts sitting at the twenty-mill floor, TY2023, \
                where a reappraisal reaches revenue",
        // Not 0.4605: written as 46.1% that sits exactly on the corpus's rounding tolerance and
        // the phrase check fails on the last bit of the float.
        pinned: 0.46053,
        tolerance: 0.0005,
        compute: |i| i.incidence[0].at_the_floor,
    },
    Figure {
        key: "regime-diff/richest-quartile-at-the-millage-floor",
        owner: "crates/regime-diff",
        unit: Unit::Share,
        label: "And the richest quartile, which is where the corpus placed the reappraisal effect",
        pinned: 0.1250,
        tolerance: 0.0005,
        compute: |i| i.incidence[3].at_the_floor,
    },
    Figure {
        key: "regime-diff/poorest-quartile-local-revenue-growth",
        owner: "crates/regime-diff",
        unit: Unit::Share,
        label: "Median growth in local revenue per pupil, poorest quartile, FY2022 to FY2024",
        pinned: 0.2251,
        tolerance: 0.0005,
        compute: |i| i.incidence[0].local_growth,
    },
    Figure {
        key: "regime-diff/richest-quartile-local-revenue-growth",
        owner: "crates/regime-diff",
        unit: Unit::Share,
        label: "The same for the richest quartile, at half the rate",
        pinned: 0.1129,
        tolerance: 0.0005,
        compute: |i| i.incidence[3].local_growth,
    },
    Figure {
        key: "regime-diff/reappraisal-held-the-gap-down-by",
        owner: "crates/regime-diff",
        unit: Unit::Dollars,
        label: "How much wider the FY2024 quartile gap would be with the reappraisals removed and \
                every district left on its own quiet-year rate",
        pinned: 212.0,
        tolerance: 5.0,
        compute: |i| -i.reappraisal.attributable(),
    },
    Figure {
        key: "dispersion/local-revenue-gap-per-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The local revenue gap the two higher levels are read against — richest quartile \
                less poorest, per pupil, FY2022",
        pinned: 9_590.0,
        tolerance: 1.0,
        compute: |i| i.quartiles[3].local_per_pupil - i.quartiles[0].local_per_pupil,
    },
    Figure {
        key: "dispersion/state-closes-of-the-local-gap",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "How much of the local gap state equalization closes, per pupil, FY2022",
        pinned: 4_448.0,
        tolerance: 1.0,
        compute: |i| i.quartiles[0].state_per_pupil - i.quartiles[3].state_per_pupil,
    },
    Figure {
        key: "dispersion/federal-closes-of-the-local-gap",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "How much of the local gap the federal channel closes, per pupil, FY2022",
        pinned: 913.0,
        tolerance: 1.0,
        compute: |i| i.quartiles[0].federal_per_pupil - i.quartiles[3].federal_per_pupil,
    },
    Figure {
        key: "dispersion/state-share-of-the-local-gap",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "State equalization as a share of the local gap, FY2022",
        pinned: 0.46,
        tolerance: 0.005,
        compute: |i| {
            let q = &i.quartiles;
            (q[0].state_per_pupil - q[3].state_per_pupil)
                / (q[3].local_per_pupil - q[0].local_per_pupil)
        },
    },
    Figure {
        key: "dispersion/federal-share-of-the-local-gap",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The federal channel as a share of the local gap, FY2022",
        pinned: 0.095,
        tolerance: 0.0005,
        compute: |i| {
            let q = &i.quartiles;
            (q[0].federal_per_pupil - q[3].federal_per_pupil)
                / (q[3].local_per_pupil - q[0].local_per_pupil)
        },
    },
    Figure {
        key: "dispersion/local-gap-left-open",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The part of the local gap neither higher level closes, per pupil, FY2022",
        pinned: 4_229.0,
        tolerance: 1.0,
        compute: |i| {
            let q = &i.quartiles;
            (q[3].local_per_pupil - q[0].local_per_pupil)
                - (q[0].state_per_pupil - q[3].state_per_pupil)
                - (q[0].federal_per_pupil - q[3].federal_per_pupil)
        },
    },
    // The White Paper's own sensitivity table, both denominators. Magnitudes with the direction
    // in the key, per convention two — and the direction is the whole finding here: the published
    // measure CHANGES SIGN across the paper's three scenarios while the headcount one does not.
    Figure {
        key: "dispersion/published-estimate-negative-all-districts",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Performance Index against spending per weighted pupil, all rated districts \
                \u{2014} negative, and too small to have a direction",
        pinned: 0.015_450_670_9,
        tolerance: 0.000_01,
        compute: |i| i.sensitivity.published.all_districts.abs(),
    },
    Figure {
        key: "dispersion/published-estimate-negative-excluding-top-spender",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The same less the single highest per-pupil spender \u{2014} still negative, and \
                a quarter the size",
        pinned: 0.004_216_665_8,
        tolerance: 0.000_01,
        compute: |i| i.sensitivity.published.excluding_top_spender.abs(),
    },
    Figure {
        key: "dispersion/published-estimate-positive-excluding-small-districts",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The same less districts under 582 pupils \u{2014} and now POSITIVE. The paper \
                calls this range robustness; a measure that changes sign has none to report",
        pinned: 0.035_822_857_6,
        tolerance: 0.000_01,
        compute: |i| i.sensitivity.published.excluding_small_districts.abs(),
    },
    Figure {
        key: "dispersion/headcount-estimate-negative-all-districts",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The same three scenarios on a headcount divisor: all rated districts",
        pinned: 0.336_524_675_8,
        tolerance: 0.000_01,
        compute: |i| i.sensitivity.headcount.all_districts.abs(),
    },
    Figure {
        key: "dispersion/headcount-estimate-negative-excluding-top-spender",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The same on a headcount divisor, less the top spender \u{2014} the scenario \
                that moves the published measure most and this one least",
        pinned: 0.354_553_510_9,
        tolerance: 0.000_01,
        compute: |i| i.sensitivity.headcount.excluding_top_spender.abs(),
    },
    Figure {
        key: "dispersion/headcount-estimate-negative-excluding-small-districts",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Less the small districts \u{2014} the three move by 0.018 in total and never \
                leave the same conclusion, which is what stability looks like",
        pinned: 0.337_137_690_2,
        tolerance: 0.000_01,
        compute: |i| i.sensitivity.headcount.excluding_small_districts.abs(),
    },
    // The FY2010-FY2013 contraction, from `ohio_panel::trough`. Every one of these was published
    // under `[verified]` against a test that asserted a band around it — `close(share, 0.807,
    // 0.02)` — so the corpus and the crate could disagree indefinitely and both stay green. Four
    // of the nine did.
    Figure {
        key: "dispersion/trough-panel-districts",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts the F-33 panel carries in BOTH FY2010 and FY2013 — the population a \
                change can be computed over, and smaller than either year alone",
        pinned: 610.0,
        tolerance: 0.0,
        compute: |i| i.trough.districts as f64,
    },
    Figure {
        key: "dispersion/trough-districts-falling",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "How many of them spent less per pupil in real terms in FY2013 than in FY2010",
        pinned: 490.0,
        tolerance: 0.0,
        compute: |i| i.trough.falling as f64,
    },
    Figure {
        key: "dispersion/trough-share-falling",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The same, as a share — the contraction was broad rather than a few districts cut \
                hard enough to move an average",
        pinned: 0.803_278_69,
        tolerance: 0.000_01,
        compute: |i| i.trough.falling_share,
    },
    Figure {
        key: "dispersion/trough-median-decline",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The median district's real decline over the window",
        pinned: 0.055_679_23,
        tolerance: 0.000_01,
        compute: |i| i.trough.median_decline,
    },
    Figure {
        key: "dispersion/trough-first-quartile-decline",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The first quartile's real decline",
        pinned: 0.090_901_61,
        tolerance: 0.000_01,
        compute: |i| i.trough.first_quartile_decline,
    },
    Figure {
        key: "dispersion/trough-weighted-decline",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Enrollment-weighted, which is the figure comparable to the Auditor's statewide \
                series — a different survey over a different entity set, same shape",
        pinned: 0.067_952_36,
        tolerance: 0.000_01,
        compute: |i| i.trough.weighted_decline,
    },
    // The negative result, as five correlations. Magnitudes, because prose writes `+0.029` and
    // `-0.186` and the numeral reader sees neither sign.
    Figure {
        key: "dispersion/trough-against-state-share",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "FY2010 state revenue share against the depth of the fall — the hypothesis this \
                refutes, and the one that should have held",
        pinned: 0.020_299_20,
        tolerance: 0.000_01,
        compute: |i| i.predictors.state_share.abs(),
    },
    Figure {
        key: "dispersion/trough-against-federal-share",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "FY2010 federal revenue share against the depth of the fall",
        pinned: 0.026_258_69,
        tolerance: 0.000_01,
        compute: |i| i.predictors.federal_share.abs(),
    },
    Figure {
        key: "dispersion/trough-against-local-share",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "FY2010 local revenue share against the depth of the fall",
        pinned: 0.024_392_39,
        tolerance: 0.000_01,
        compute: |i| i.predictors.local_share.abs(),
    },
    Figure {
        key: "dispersion/trough-against-log-enrolment",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Log enrollment against the depth of the fall — the strongest of the five, and \
                still far smaller than the spread it is trying to explain",
        pinned: 0.196_980_44,
        tolerance: 0.000_01,
        compute: |i| i.predictors.log_enrollment.abs(),
    },
    Figure {
        key: "dispersion/trough-against-prior-spending",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "FY2010 real spending per pupil against the depth of the fall, which is partly \
                mechanical: a district starting higher has more room to fall",
        pinned: 0.167_893_44,
        tolerance: 0.000_01,
        compute: |i| i.predictors.prior_spending.abs(),
    },
    Figure {
        key: "dispersion/trough-deepest-decile-districts",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts in the worst tenth of the contraction",
        pinned: 61.0,
        tolerance: 0.0,
        compute: |i| i.decile.districts as f64,
    },
    Figure {
        key: "dispersion/trough-deepest-decile-decline",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Their mean real decline",
        pinned: 0.175_011_05,
        tolerance: 0.000_01,
        compute: |i| i.decile.mean_decline,
    },
    Figure {
        key: "dispersion/trough-deepest-decile-pupil-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The share of the panel's pupils they hold — which is why the deepest decile is a \
                tail rather than the story",
        pinned: 0.119_861_53,
        tolerance: 0.000_01,
        compute: |i| i.decile.pupil_share,
    },
    // ---- crates/foundation -------------------------------------------------------------------
    //
    // The input-year refresh, from `foundation::refresh`. `scenario/fsfp-input-year-refresh` is
    // the one scenario whose whole subject is a measurement rather than a policy dial, so its
    // incidence figures are the scenario.
    Figure {
        key: "foundation/refresh-panel-districts",
        owner: "crates/foundation",
        unit: Unit::Count,
        label: "Districts in the grade-band panel the refresh is measured over",
        pinned: 604.0,
        tolerance: 0.0,
        compute: |i| i.refresh_incidence.districts as f64,
    },
    Figure {
        key: "foundation/refresh-statewide-base-cost-increase",
        owner: "crates/foundation",
        unit: Unit::Dollars,
        label: "The statewide base cost increase from refreshing the classroom teacher salary \
                input to FY2024 — computed cost, before any state-share or guarantee reduction",
        pinned: 466_233_428.14,
        tolerance: 1.0,
        compute: |i| i.refresh_incidence.statewide_delta,
    },
    Figure {
        key: "foundation/refresh-average-per-pupil",
        owner: "crates/foundation",
        unit: Unit::Dollars,
        label: "The ADM-weighted average increase per pupil",
        pinned: 323.970_247_833,
        tolerance: 0.01,
        compute: |i| i.refresh_incidence.average_per_pupil,
    },
    Figure {
        key: "foundation/refresh-least-affected-per-pupil",
        owner: "crates/foundation",
        unit: Unit::Dollars,
        label: "The least-affected district's increase per pupil",
        pinned: 318.258_703_931,
        tolerance: 0.01,
        compute: |i| i.refresh_incidence.min_per_pupil,
    },
    Figure {
        key: "foundation/refresh-most-affected-per-pupil",
        owner: "crates/foundation",
        unit: Unit::Dollars,
        label: "The most-affected district's, which is about 1.4 times the least — with no \
                policy choice anywhere in the difference",
        pinned: 442.771_078_440,
        tolerance: 0.01,
        compute: |i| i.refresh_incidence.max_per_pupil,
    },
    Figure {
        key: "foundation/refresh-districts-at-the-special-teacher-minimum",
        owner: "crates/foundation",
        unit: Unit::Count,
        label: "Districts small enough that the six-teacher special minimum binds — the single \
                cause of the spread above",
        pinned: 155.0,
        tolerance: 0.0,
        compute: |i| i.refresh_incidence.special_minimum_binds as f64,
    },
    Figure {
        key: "foundation/refresh-per-pupil-where-the-minimum-binds",
        owner: "crates/foundation",
        unit: Unit::Dollars,
        label: "Their mean increase per pupil",
        pinned: 346.279_581_056,
        tolerance: 0.01,
        compute: |i| i.refresh_incidence.bound_mean_per_pupil,
    },
    Figure {
        key: "foundation/refresh-per-pupil-where-it-does-not",
        owner: "crates/foundation",
        unit: Unit::Dollars,
        label: "Everyone else's mean increase per pupil, which is the comparison that makes the \
                staffing minimum the cause rather than a correlate",
        pinned: 322.872_511_005,
        tolerance: 0.01,
        compute: |i| i.refresh_incidence.free_mean_per_pupil,
    },
    Figure {
        key: "foundation/fy2024-classroom-teacher-salary",
        owner: "crates/foundation",
        unit: Unit::Dollars,
        label: "The FY2024 statewide average classroom teacher salary — the year a refresh reads, \
                and the number the whole perturbation runs to",
        pinned: 73_777.08,
        tolerance: 0.0,
        compute: |_| foundation::FY2024_TEACHER_SALARY,
    },
    Figure {
        key: "foundation/fy2022-classroom-teacher-salary",
        owner: "crates/foundation",
        unit: Unit::Dollars,
        label: "The FY2022 reference salary it runs from, as the department's FY2027 calculator \
                carries it",
        pinned: 68_022.22,
        tolerance: 0.0,
        compute: |_| foundation::StatewideFactors::fy2027().teacher_salary,
    },
    // ---- crates/scenario-delta ---------------------------------------------------------------
    //
    // Who a base cost increase does NOT reach, which is the half of the answer the scenario
    // exists to state. The corpus published a decomposition that summed correctly and disagreed
    // with the crate in both terms: 242 unmoved and 52 lifted off against 253 and 41.
    Figure {
        key: "scenario-delta/refresh-districts-unmoved",
        owner: "crates/scenario-delta",
        unit: Unit::Count,
        label: "Districts a base cost increase of the department's own size does not move at \
                all, because the guarantee pays them under both policies",
        pinned: 253.0,
        tolerance: 0.0,
        compute: |i| i.refresh_reach.total().reach.unmoved as f64,
    },
    Figure {
        key: "scenario-delta/refresh-districts-lifted-off",
        owner: "crates/scenario-delta",
        unit: Unit::Count,
        label: "Guaranteed districts an increase this size lifts off the guarantee and onto the \
                formula — the mechanism by which a large enough increase shrinks the guarantee",
        pinned: 41.0,
        tolerance: 0.0,
        compute: |i| i.refresh_reach.total().reach.lifted_off as f64,
    },
    Figure {
        key: "scenario-delta/refresh-districts-gaining",
        owner: "crates/scenario-delta",
        unit: Unit::Count,
        label: "Districts whose aid actually rises",
        pinned: 356.0,
        tolerance: 0.0,
        compute: |i| i.refresh_reach.total().reach.gainers as f64,
    },
    Figure {
        key: "scenario-delta/refresh-adm-share-unmoved",
        owner: "crates/scenario-delta",
        unit: Unit::Share,
        label: "The share of Ohio's modelled enrollment held by the districts an increase does \
                not reach — the figure that says whether the unmoved count is a large fact",
        pinned: 0.430_474_04,
        tolerance: 0.000_01,
        compute: |i| {
            let unmoved = i.refresh_reach.aggregate(|d| !d.moved());
            unmoved.adm / i.refresh_reach.total().adm
        },
    },
    // ---- crates/project ----------------------------------------------------------------------
    // The Census F-33 state table. `litigation/derolph-i-1997` rests its central comparative
    // claim on these, and `revenue-stream/esser` and `title-i` on the federal three — which is
    // why they were the two nodes that had cited the wrong F-33 panel for them.
    //
    // Ohio's *rank* is not here, and the omission is the mechanism working rather than a gap.
    // The corpus writes ranks as ordinals — `seventh highest of fifty-one`, `25th of 51` — and
    // `numerals()` reads neither: a spelled-out ordinal has no digits, and the `th` of `25th`
    // defeats the token boundary that stops `65 more` reading as sixty-five million. A rank is
    // therefore a claim this gate cannot hold, and `.yidam/corpus/README.md` says so rather than
    // this file exporting a figure nothing can bind.
    Figure {
        key: "dispersion/ohio-local-share-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Ohio's local share of school revenue, FY2022, on the Census Bureau's definitions",
        pinned: 0.5177,
        tolerance: 0.0001,
        compute: |i| ohio(i).local_share(),
    },
    Figure {
        key: "dispersion/national-local-share-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The national local share of school revenue, FY2022",
        pinned: 0.4335,
        tolerance: 0.0001,
        compute: |i| national_share(i, |s| s.local_revenue),
    },
    Figure {
        key: "dispersion/ohio-state-share-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Ohio's state share of school revenue, FY2022",
        pinned: 0.3436,
        tolerance: 0.0001,
        compute: |i| ohio(i).state_share(),
    },
    Figure {
        key: "dispersion/national-state-share-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The national state share of school revenue, FY2022",
        pinned: 0.4341,
        tolerance: 0.0001,
        compute: |i| national_share(i, |s| s.state_revenue),
    },
    Figure {
        key: "dispersion/ohio-spending-per-pupil-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Ohio's current spending per pupil, FY2022, on the Bureau's own enrolment count",
        pinned: 14_923.0,
        tolerance: 1.0,
        compute: |i| ohio(i).spending_per_pupil(),
    },
    Figure {
        key: "dispersion/national-spending-per-pupil-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "National current spending per pupil, FY2022",
        pinned: 15_801.0,
        tolerance: 1.0,
        compute: |i| national_per_pupil(i, |s| s.current_spending),
    },
    Figure {
        key: "dispersion/ohio-federal-share-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Ohio's federal share of school revenue, FY2022 — the peak of pandemic relief",
        pinned: 0.1387,
        tolerance: 0.0001,
        compute: |i| ohio(i).federal_share(),
    },
    Figure {
        key: "dispersion/national-federal-share-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The national federal share of school revenue, FY2022",
        pinned: 0.1324,
        tolerance: 0.0001,
        compute: |i| national_share(i, |s| s.federal_revenue),
    },
    Figure {
        key: "dispersion/ohio-federal-revenue-per-pupil-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Ohio's federal revenue per pupil, all federal sources, FY2022",
        pinned: 2_466.0,
        tolerance: 1.0,
        compute: |i| {
            let oh = ohio(i);
            oh.federal_revenue * 1_000.0 / oh.enrollment
        },
    },
    Figure {
        key: "dispersion/national-federal-revenue-per-pupil-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "National federal revenue per pupil, all federal sources, FY2022",
        pinned: 2_493.0,
        tolerance: 1.0,
        compute: |i| national_per_pupil(i, |s| s.federal_revenue),
    },
    // `doctrine/equity`, which is the node the whole crate operationalizes. Every one of these
    // was computed inside `tests/cupp_fy24.rs` on a parser that test declared privately until
    // #157; they are the clearest case of the constraint #158 was filed about.
    //
    // The two correlations are exported as **magnitudes**, with the direction in the key. Prose
    // writes them `−0.549` and `+0.630`, and `numerals()` does not read a sign — so a figure
    // pinned at `-0.549` could not be bound to the sentence that states it without the check
    // ignoring the one thing #120 was about.
    Figure {
        key: "dispersion/operating-expenditure-median",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Median operating expenditure per pupil, FY2024",
        pinned: 15_646.0,
        tolerance: 1.0,
        compute: |i| operating_dispersion(i).median,
    },
    Figure {
        key: "dispersion/operating-expenditure-coefficient-of-variation",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Coefficient of variation of operating expenditure per pupil, FY2024",
        pinned: 0.2016,
        tolerance: 0.0005,
        compute: |i| operating_dispersion(i).coefficient_of_variation,
    },
    // What moved at FY2016. Every figure here is on `dispersion::fy2016::BASIS` — gross of the
    // community-school deduct in both eras — because the published column is not one quantity
    // across that year. The correlations are exported as magnitudes with the direction in the
    // key, per this manifest's convention.
    Figure {
        key: "dispersion/fy2016-step-against-total-valuation-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The FY2016 move in state revenue per pupil against log total assessed value \
                \u{2014} negative, and the strongest correlate of it",
        pinned: 0.2691,
        tolerance: 0.0005,
        compute: |i| {
            let steps: Vec<f64> = i.fy2016.districts.iter().map(|d| d.step).collect();
            let value: Vec<f64> = i
                .fy2016
                .districts
                .iter()
                .map(|d| d.total_valuation.ln())
                .collect();
            dispersion::wealth_neutrality(&value, &steps)
                .expect("paired")
                .correlation
                .abs()
        },
    },
    Figure {
        key: "dispersion/fy2016-step-against-industrial-share",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "And against industrial property share \u{2014} positive, which is the wrong sign \
                for a tangible personal property reading",
        pinned: 0.1175,
        tolerance: 0.0005,
        compute: |i| {
            let steps: Vec<f64> = i.fy2016.districts.iter().map(|d| d.step).collect();
            let industrial: Vec<f64> = i
                .fy2016
                .districts
                .iter()
                .map(|d| d.industrial_share)
                .collect();
            dispersion::wealth_neutrality(&industrial, &steps)
                .expect("paired")
                .correlation
        },
    },
    Figure {
        key: "dispersion/fy2016-step-lowest-valuation-quintile",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Mean FY2016 move for the fifth of districts with the smallest total assessed value",
        pinned: 0.2679,
        tolerance: 0.0005,
        compute: |i| i.fy2016.by_valuation[0],
    },
    Figure {
        key: "dispersion/fy2016-step-highest-valuation-quintile",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "And for the fifth with the largest",
        pinned: 0.1060,
        tolerance: 0.0005,
        compute: |i| i.fy2016.by_valuation[4],
    },
    Figure {
        key: "dispersion/fy2016-step-total-valuation-coefficient-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Log total assessed value against the FY2016 move, standardised, holding wealth \
                per pupil and disadvantage \u{2014} negative",
        pinned: 0.1994,
        tolerance: 0.0005,
        compute: |i| i.fy2016.model.standardized[0].abs(),
    },
    Figure {
        key: "dispersion/fy2016-step-wealth-per-pupil-coefficient-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Wealth per pupil in the same model \u{2014} negative, and two thirds the size",
        pinned: 0.1204,
        tolerance: 0.0005,
        compute: |i| i.fy2016.model.standardized[1].abs(),
    },
    // Whether the performance supplement moves a district's position, which the node that
    // measured its gradient recorded as open.
    Figure {
        key: "project/performance-supplement-districts-moved",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts whose place in the order of state support per pupil changes when the \
                performance supplement is removed, of 609",
        pinned: 370.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::supplement_reach::rank_moves().0 as f64,
    },
    Figure {
        key: "project/performance-supplement-mean-rank-move",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "And how far the average district moves, in places",
        pinned: 1.0673,
        tolerance: 0.0005,
        compute: |_| project::supplement_reach::rank_moves().1,
    },
    Figure {
        key: "project/performance-supplement-quintile-crossings",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts the supplement carries across a quintile boundary of state aid per pupil",
        pinned: 4.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::supplement_reach::quintile_crossings().len() as f64,
    },
    Figure {
        key: "project/state-aid-dispersion-with-supplement",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "Coefficient of variation of state support per pupil across Ohio's districts",
        pinned: 0.5242,
        tolerance: 0.0005,
        compute: |_| project::supplement_reach::dispersion().0,
    },
    Figure {
        key: "project/state-aid-dispersion-without-supplement",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The same with the performance supplement removed \u{2014} wider, so the \
                supplement narrows Ohio's equalisation rather than widening it",
        pinned: 0.5281,
        tolerance: 0.0005,
        compute: |_| project::supplement_reach::dispersion().1,
    },
    Figure {
        key: "project/performance-supplement-share-of-least-poor-aid",
        owner: "crates/project",
        // Under this manifest's share floor, like its pair below: the prose bound to it carries
        // the bare numeral and no per-cent sign.
        unit: Unit::Ratio,
        label: "The supplement as a share of the state aid the least-poor quartile of districts \
                already receives",
        pinned: 0.011408,
        tolerance: 0.0005,
        compute: |_| project::supplement_reach::relative_to_aid()[0],
    },
    Figure {
        key: "project/performance-supplement-share-of-poorest-aid",
        owner: "crates/project",
        // Under this manifest's share floor, so the prose bound to it carries no per-cent sign.
        unit: Unit::Ratio,
        label: "And of the aid the poorest quartile receives \u{2014} a fifth as much, against a \
                per-pupil gradient of 2.56",
        pinned: 0.002173,
        tolerance: 0.0005,
        compute: |_| project::supplement_reach::relative_to_aid()[3],
    },
    Figure {
        key: "project/guaranteed-districts-paid-a-supplement",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts on the guarantee that are paid a performance supplement on top of it, \
                of 294 \u{2014} the supplement is line [O] and the guarantee is computed on [H]",
        pinned: 217.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::supplement_reach::paid_on_the_guarantee().1 as f64,
    },
    // The Catalog of Budget Line Items' other clause: which Revised Code chapter authorises a line
    // this biennium, which changes where the establishing act does not.
    Figure {
        key: "project/evidence-based-model-line-items",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Line items of the state budget authorised under R.C. 3306, the Evidence-Based \
                Model's chapter, across eighteen editions of the Catalog",
        pinned: 1.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| {
            let under = project::ledger::line_origins::lines_under("3306");
            let mut alis: Vec<String> = under.into_values().flatten().collect();
            alis.sort();
            alis.dedup();
            alis.len() as f64
        },
    },
    Figure {
        key: "project/evidence-based-model-editions",
        owner: "crates/project",
        unit: Unit::Count,
        label: "And the editions it appears in, which are FY2009, FY2010 and FY2011",
        pinned: 3.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::ledger::line_origins::lines_under("3306").len() as f64,
    },
    Figure {
        key: "project/chapter-3317-line-items-before-the-model",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Line items authorised under R.C. 3317 in the 2006 edition, before the model",
        pinned: 10.0,
        tolerance: 0.0,
        compute: |_| chapter_lines(2006),
    },
    Figure {
        key: "project/chapter-3317-line-items-under-the-model",
        owner: "crates/project",
        unit: Unit::Count,
        label: "In the 2009 edition, its first, which is the same count as each of its three",
        pinned: 6.0,
        tolerance: 0.0,
        compute: |_| chapter_lines(2009),
    },
    Figure {
        key: "project/chapter-3317-line-items-now",
        owner: "crates/project",
        unit: Unit::Count,
        label: "And in the 2025 edition, under the Fair School Funding Plan",
        pinned: 13.0,
        tolerance: 0.0,
        compute: |_| chapter_lines(2025),
    },
    Figure {
        key: "project/scholarship-line-establishing-span",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Years between the oldest and newest establishing acts of the five line items that \
                pay Ohio's foundation aid and its scholarships",
        pinned: 42.0,
        tolerance: 0.0,
        compute: |_| {
            let convened = project::ledger::line_origins::convened;
            f64::from(convened(133)) - f64::from(convened(112))
        },
    },
    Figure {
        key: "project/deduction-per-pupil-fall-through-the-gap",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "How far the per-pupil amount a district is charged for a pupil it does not teach \
                fell across the four years Ohio had no statewide base cost per pupil",
        pinned: 79.0,
        tolerance: 0.01,
        compute: |_| {
            let first = 5_732.0;
            let last = project::base_cost::DEDUCTION
                .last()
                .and_then(|row| row.stated.dollars)
                .expect("the deduction series ends with an amount");
            first - last
        },
    },
    Figure {
        key: "project/deduction-per-pupil-real-fall-through-the-gap-negative",
        owner: "crates/project",
        unit: Unit::Share,
        label: "And the same fall in constant dollars, across four years in which the formula had \
                no price for a pupil and prices rose anyway",
        pinned: 0.089_008_069_370_486_44,
        tolerance: 0.000_001,
        compute: |_| -project::base_cost::deduction_real_change(),
    },
    // The statewide scalars both calculators state once. Several `parameter` nodes said no
    // prior-year value was held; two years of the calculator were in the cache.
    Figure {
        key: "project/calculator-scalars-that-held",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Statewide scalars in the funding calculator that are the same number in FY2026 \
                and FY2027 \u{2014} every weight and every base cost among them",
        pinned: 25.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::prior_model::scalars_that_moved().1.len() as f64,
    },
    Figure {
        key: "project/calculator-scalars-that-moved",
        owner: "crates/project",
        unit: Unit::Count,
        label: "And the ones that differ, every one of them downstream of a count or a valuation",
        pinned: 10.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::prior_model::scalars_that_moved().0.len() as f64,
    },
    Figure {
        key: "project/base-funding-supplement-per-pupil-fy2026",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The base funding supplement's per-pupil rate in the FY2026 model, against $40 in \
                the FY2027 one",
        pinned: 27.0,
        tolerance: 0.01,
        compute: |_| scalar_at("base_funding_supplement_per_pupil", 2026),
    },
    Figure {
        key: "project/enrolment-growth-supplement-per-pupil-fy2026",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And the enrolment growth supplement's, against $250 a year later",
        pinned: 225.0,
        tolerance: 0.01,
        compute: |_| scalar_at("enrolment_growth_supplement_per_pupil", 2026),
    },
    Figure {
        key: "project/performance-supplement-per-pupil",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The performance supplement's per-pupil rate, which is the same in both years the \
                calculator can be read for",
        pinned: 13.0,
        tolerance: 0.01,
        compute: |_| scalar_at("performance_supplement_per_pupil", 2027),
    },
    Figure {
        key: "project/preschool-proration-factor-fy2026",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The preschool special education proration factor in the FY2026 model, which is \
                that year's appropriation over that year's demand",
        pinned: 0.968_538_11,
        tolerance: 0.000_000_1,
        compute: |_| {
            project::prior_model::preschool_proration(2026)
                .expect("FY2026 is in the scalar fixture")
                .0
        },
    },
    Figure {
        key: "project/preschool-proration-implied-fy2027",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "What the same arithmetic gives for FY2027, where the program's demand is below \
                its appropriation and no proration arises",
        pinned: 1.004_886_704_582_321_3,
        tolerance: 0.000_001,
        compute: |_| {
            project::prior_model::preschool_proration(2027)
                .expect("FY2027 is in the scalar fixture")
                .1
        },
    },
    Figure {
        key: "project/preschool-headroom-fy2027",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "How far preschool special education falls short of its FY2027 appropriation \
                after the factor the calculator applies",
        pinned: 5_568_648.27,
        tolerance: 0.01,
        compute: |_| {
            let (paid, appropriated) =
                project::prior_model::preschool_headroom(2027).expect("FY2027 is in the fixture");
            appropriated - paid
        },
    },
    // The two clocks the Fair School Funding Plan runs on, measured over the one interval the
    // department has published two models for.
    Figure {
        key: "project/capacity-per-pupil-rise-fy2026-to-fy2027",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the median district's per-pupil local capacity gains between the department's \
                FY2026 model and its FY2027 one",
        pinned: 472.440_7,
        tolerance: 0.01,
        compute: |_| project::prior_model::median_change(project::prior_model::Quantity::CapacityPerPupil).0,
    },
    Figure {
        key: "project/base-cost-per-pupil-rise-fy2026-to-fy2027",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And what its base cost per pupil gains over the same interval, with the cost \
                inputs held at FY2022 by act",
        pinned: 4.11,
        tolerance: 0.01,
        compute: |_| project::prior_model::median_change(project::prior_model::Quantity::BaseCostPerPupil).0,
    },
    Figure {
        key: "project/state-share-fall-fy2026-to-fy2027-negative",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The fall in the median district's state share of its base cost across that one \
                year, in percentage points",
        // Percentage points rather than a share, because that is the quantity: the difference of
        // two shares is not one. Written bare in prose for the same reason — a `%` beside it
        // would read as the relative fall, which is 8.57% and a different number.
        pinned: 4.112_500_000_000_002,
        tolerance: 0.000_1,
        compute: |_| project::prior_model::state_share_fall().0 * 100.0,
    },
    Figure {
        key: "project/districts-whose-state-share-fell",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts whose published state share of the base cost is lower in the FY2027 \
                model than in the FY2026 one, of 609",
        pinned: 540.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::prior_model::state_share_fall().1 as f64,
    },
    Figure {
        key: "project/median-state-share-fy2026",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The median district's published state share of its base cost in the FY2026 model",
        pinned: 0.414_003_3,
        tolerance: 0.000_001,
        compute: |_| project::prior_model::median_state_share().0,
    },
    Figure {
        key: "project/median-state-share-fy2027",
        owner: "crates/project",
        unit: Unit::Share,
        label: "And in the FY2027 model, one year later",
        pinned: 0.360_033_2,
        tolerance: 0.000_001,
        compute: |_| project::prior_model::median_state_share().1,
    },
    Figure {
        key: "project/median-state-share-holding-capacity-still",
        owner: "crates/project",
        unit: Unit::Share,
        label: "What the FY2027 median would be if local capacity had stayed at its FY2026 value \
                and only the base cost had moved",
        pinned: 0.412_330_774_574_220_24,
        tolerance: 0.000_001,
        compute: |_| project::prior_model::holding_capacity_still().1,
    },
    Figure {
        key: "project/aggregate-state-share-fy2026",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The state's share of the aggregate cost of Ohio's schools in the FY2026 model",
        pinned: 0.353_501_488_959_158_7,
        tolerance: 0.000_001,
        compute: |_| project::prior_model::aggregate_state_share().0,
    },
    Figure {
        key: "project/aggregate-state-share-fy2027",
        owner: "crates/project",
        unit: Unit::Share,
        label: "And in the FY2027 model, one year later",
        pinned: 0.315_236_926_209_692_4,
        tolerance: 0.000_001,
        compute: |_| project::prior_model::aggregate_state_share().1,
    },
    Figure {
        key: "project/districts-at-the-minimum-state-share-fy2026",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts the 10% minimum state share bound for in the FY2026 model, against 138 \
                a year later",
        pinned: 105.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::prior_model::at_the_floor().0 as f64,
    },
    Figure {
        key: "project/districts-that-fell-onto-the-minimum-state-share",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts the minimum state share did not bind for in FY2026 and does in FY2027",
        pinned: 33.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::prior_model::fell_onto_the_floor().len() as f64,
    },
    Figure {
        key: "project/dpia-statewide-percentage-fy2026",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The statewide economically disadvantaged percentage the FY2026 model indexes each \
                district against",
        pinned: 0.565_990_245,
        tolerance: 0.000_001,
        compute: |_| project::prior_model::dpia_statewide_percentage().0,
    },
    Figure {
        key: "project/dpia-fall-fy2026-to-fy2027-negative",
        owner: "crates/project",
        unit: Unit::Share,
        label: "How far statewide disadvantaged pupil impact aid falls between the two models",
        pinned: 0.075_007_075_000_666_72,
        tolerance: 0.000_001,
        compute: |_| {
            let (before, after) = project::prior_model::dpia_total();
            1.0 - after / before
        },
    },
    Figure {
        key: "project/directly-certified-adm-fall-fy2026-to-fy2027-negative",
        owner: "crates/project",
        unit: Unit::Share,
        label: "How far the median district's directly certified count falls between the two \
                models, under the same label in both",
        pinned: 0.167_988_742_540_178_7,
        tolerance: 0.000_001,
        compute: |_| {
            -project::prior_model::median_change(
                project::prior_model::Quantity::DirectlyCertifiedAdm,
            )
            .1
        },
    },
    // The counts the plan multiplies, both published models. Four nodes read a vintage off a
    // column header -- `Category 1 Career Tech FTE-FY21` -- and the header is four years stale.
    // The scale targeted assistance equalises to, and the second pair of medians beside it.
    Figure {
        key: "project/median-weighted-wealth-rise",
        owner: "crates/project",
        unit: Unit::Share,
        label: "How far statewide median weighted wealth moves between the two published models \
                \u{2014} the quantity targeted assistance equalises to",
        pinned: 0.082_314_914_960_079_92,
        tolerance: 0.000_001,
        compute: |_| {
            scalar_at("median_weighted_wealth", 2027) / scalar_at("median_weighted_wealth", 2026)
                - 1.0
        },
    },
    Figure {
        key: "project/median-weighted-wealth-per-pupil-rise",
        owner: "crates/project",
        unit: Unit::Share,
        label: "And the per-pupil median, which the capacity and wealth tiers are both charged \
                against",
        pinned: 0.092_254_802_666_704_72,
        tolerance: 0.000_001,
        compute: |_| {
            scalar_at("median_weighted_wealth_per_pupil", 2027)
                / scalar_at("median_weighted_wealth_per_pupil", 2026)
                - 1.0
        },
    },
    // Four years of the department's enrolled ADM, and what the fourth one buys.
    Figure {
        key: "project/adm-growth-persistence",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "How much of a district's enrolment growth rate carries into the next year, \
                FY2025 to FY2026 \u{2014} the quantity the projection's damping parameterises",
        pinned: 0.341_205,
        tolerance: 0.000_01,
        compute: |_| project::counts::adm_persistence()[1].2,
    },
    Figure {
        key: "project/adm-growth-persistence-earlier-window",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The same correlation one year earlier, on the consistent series",
        pinned: 0.327_930,
        tolerance: 0.000_01,
        compute: |_| project::counts::adm_persistence()[0].2,
    },
    Figure {
        key: "project/adm-fy2023-columns-that-disagree",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts where the growth supplement's FY2023 enrolled ADM differs from the base \
                cost window's \u{2014} two columns of one workbook under one name",
        pinned: 608.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |i| {
            let base_cost = project::counts::adm_history();
            let base_cost = base_cost.get(&2023).cloned().unwrap_or_default();
            i.panel
                .iter()
                .filter(|record| {
                    base_cost.get(&record.irn).is_some_and(|ours| {
                        (ours - record.supplements.adm_fy23).abs() >= 1e-9
                    })
                })
                .count() as f64
        },
    },
    Figure {
        key: "project/adm-fy2025-restated-districts",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts whose FY2025 enrolled ADM the department restated between its two \
                published models \u{2014} of 611, and by at most a third of a per cent",
        pinned: 140.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| {
            let restated = project::counts::adm_restatement(2025).expect("both models publish it");
            (restated.paired - restated.identical) as f64
        },
    },
    Figure {
        key: "project/english-learner-count-districts-changed",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts whose English learner count differs between the two published models, \
                under the column header that calls the count FY2021",
        pinned: 512.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| {
            let (before, after) = (
                project::counts::year(2026),
                project::counts::year(2027),
            );
            before
                .iter()
                .filter(|(irn, earlier)| {
                    after
                        .get(*irn)
                        .is_some_and(|later| later.english_learner != earlier.english_learner)
                })
                .count() as f64
        },
    },
    Figure {
        key: "project/english-learner-category-one-fall-negative",
        owner: "crates/project",
        unit: Unit::Share,
        label: "How far the most recently arrived English learners fall between the two models \
                \u{2014} a cohort ageing through the taper, not a file nobody refreshed",
        pinned: 0.219_738_035_205_238_6,
        tolerance: 0.000_001,
        compute: |_| -project::counts::stability(project::counts::Series::EnglishLearner(1)).change(),
    },
    Figure {
        key: "project/english-learner-composition-losers",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts whose English learner headcount held or grew between the two models \
                and whose weighted count fell anyway \u{2014} the taper, not the population",
        pinned: 46.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::counts::composition_losers().0 as f64,
    },
    Figure {
        key: "project/career-technical-count-districts-unchanged",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts carrying an identical career-technical FTE across the two models \
                \u{2014} stable, which is not the same as held",
        pinned: 559.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| {
            let (before, after) = (
                project::counts::year(2026),
                project::counts::year(2027),
            );
            before
                .iter()
                .filter(|(irn, earlier)| {
                    after
                        .get(*irn)
                        .is_some_and(|later| later.career_technical == earlier.career_technical)
                })
                .count() as f64
        },
    },
    Figure {
        key: "project/career-technical-fte-statewide-fy2027",
        owner: "crates/project",
        unit: Unit::Pupils,
        label: "Career-technical full-time equivalents statewide in the FY2027 model, against \
                28,558 in the FY2026 one",
        pinned: 28_641.919_504,
        tolerance: 0.001,
        compute: |_| {
            project::counts::year(2027)
                .values()
                .map(|row| row.career_technical.iter().sum::<f64>())
                .sum()
        },
    },
    Figure {
        key: "project/school-building-count-districts-unchanged",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts whose school building count is identical across the two models \
                \u{2014} every one of them, and the only column on the sheet that manages it",
        pinned: 611.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| {
            project::counts::stability(project::counts::Series::SchoolBuildings).unchanged as f64
        },
    },
    // The parameter the corpus calls the most consequential number in Ohio school funding, read
    // off twelve committed greenbooks and put through the deflator this workspace has had all
    // along. Every real figure is in FY2026 dollars, the last June the Bureau has published.
    Figure {
        key: "project/base-cost-real-peak",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The most Ohio's statewide base cost per pupil has ever been worth, in FY2026 \
                dollars, which it reached in FY2003",
        pinned: 8_996.888_666_303_757,
        tolerance: 0.01,
        compute: |_| project::base_cost::peak().1,
    },
    Figure {
        key: "project/base-cost-nominal-rise-since-fy2002",
        owner: "crates/project",
        unit: Unit::Share,
        label: "How far the statewide base cost per pupil rises from FY2002 to FY2026 in the \
                dollars of each year",
        pinned: 0.712_089_738_263_398_4,
        tolerance: 0.000_001,
        compute: |_| base_cost_growth(2002, 2026).0,
    },
    Figure {
        key: "project/base-cost-real-fall-since-fy2002-negative",
        owner: "crates/project",
        unit: Unit::Share,
        label: "And how far it falls across the same span once the change in prices is removed",
        pinned: 0.077_696_962_696_479_19,
        tolerance: 0.000_001,
        compute: |_| -base_cost_growth(2002, 2026).1,
    },
    Figure {
        key: "project/base-cost-years-with-no-statewide-amount",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Fiscal years in which Ohio's formula has no statewide base cost per pupil at all, \
                which are FY2010 through FY2013",
        pinned: 4.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| project::base_cost::gap().len() as f64,
    },
    Figure {
        key: "project/base-cost-real-fall-across-the-gap-negative",
        owner: "crates/project",
        unit: Unit::Share,
        label: "How much less the base cost per pupil was worth when it returned in FY2014 than \
                when it was abandoned in FY2009",
        pinned: 0.092_978_668_972_085_07,
        tolerance: 0.000_001,
        compute: |_| -base_cost_growth(2009, 2014).1,
    },
    Figure {
        key: "project/base-cost-opportunity-grant-era-real-fall-negative",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The real fall in the opportunity grant's formula amount from FY2014 to FY2021, \
                across which the printed figure rose every year",
        pinned: 0.080_766_982_747_163_98,
        tolerance: 0.000_001,
        compute: |_| -base_cost_growth(2014, 2021).1,
    },
    Figure {
        key: "project/base-cost-fy2024-amount-in-fy2026-dollars",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the FY2024 statewide average base cost per pupil, which two acts have frozen \
                through FY2027, is worth in FY2026 dollars",
        pinned: 8_760.825_603_564_892,
        tolerance: 0.01,
        compute: |_| base_cost_freeze().0,
    },
    Figure {
        key: "project/base-cost-freeze-shortfall-per-pupil",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And the gap between that and the frozen amount the formula actually pays in \
                FY2026, per pupil",
        pinned: 518.825_603_564_892,
        tolerance: 0.01,
        compute: |_| {
            let (restated, paid, _) = base_cost_freeze();
            restated - paid
        },
    },
    Figure {
        key: "project/base-cost-fy2022-estimate-miss",
        owner: "crates/project",
        unit: Unit::Share,
        label: "How far the Legislative Service Commission's estimate of the FY2022 statewide \
                average was below the amount the year produced",
        pinned: 0.020_827_547_903_360_178,
        tolerance: 0.000_001,
        compute: |_| {
            let year = project::base_cost::year(2022).expect("FY2022 is in the series");
            let estimate = year.stated.dollars.expect("an estimate is an amount");
            project::base_cost::nominal(2022).expect("FY2022 has an amount") / estimate - 1.0
        },
    },
    // The metric Ohio publishes twice. Every figure on `dispersion::valuation::TAX_YEAR`, because
    // the node that published them first did not fix a year and the table came out three.
    Figure {
        key: "dispersion/valuation-numerator-agreement",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts on which the Department of Taxation's total taxable value and the \
                District Profile Report's valuation per pupil reproduce each other exactly, of 606",
        pinned: 606.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| dispersion::valuation::numerators_agree().0 as f64,
    },
    Figure {
        key: "dispersion/valuation-agreement-within-two-per-cent",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "And districts whose two published per-pupil figures agree within 2% \u{2014} the \
                numerators are identical, so the rest is the pupil count",
        pinned: 63.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |_| dispersion::valuation::agreement().0 as f64,
    },
    Figure {
        key: "dispersion/valuation-median-enrolled",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Median assessed valuation per pupil on the count of children a district teaches",
        pinned: 248_096.51,
        tolerance: 0.01,
        compute: |_| dispersion::valuation::medians().0,
    },
    Figure {
        key: "dispersion/valuation-median-resident",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "And on the count of children resident in it \u{2014} six per cent apart at the \
                median, and more than two to one in the districts that matter",
        pinned: 233_319.65,
        tolerance: 0.01,
        compute: |_| dispersion::valuation::medians().1,
    },
    Figure {
        key: "dispersion/youngstown-denominator-ratio",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Resident pupils per pupil taught, Youngstown City \u{2014} the widest in the state",
        pinned: 2.2337,
        tolerance: 0.0005,
        compute: |_| named_ratio("045161"),
    },
    Figure {
        key: "dispersion/columbus-denominator-ratio",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Resident pupils per pupil taught, Columbus City \u{2014} the largest district \
                with the divergence",
        pinned: 1.6725,
        tolerance: 0.0005,
        compute: |_| named_ratio("043802"),
    },
    Figure {
        key: "dispersion/youngstown-valuation-per-pupil-enrolled",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Youngstown City's assessed valuation per pupil on the Department of Education's \
                count",
        pinned: 172_999.40,
        tolerance: 0.01,
        compute: |_| {
            dispersion::valuation::district("045161")
                .expect("Youngstown joins both tables")
                .on_enrolled()
                .value
        },
    },
    Figure {
        key: "dispersion/youngstown-valuation-per-pupil-resident",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "And on the Department of Taxation's, which is the same property",
        pinned: 77_450.72,
        tolerance: 0.01,
        compute: |_| {
            dispersion::valuation::district("045161")
                .expect("Youngstown joins both tables")
                .on_resident()
                .value
        },
    },
    // Ohio's school construction program, which the corpus had no series for because it looked for
    // one on the paying side. `Ratio` for the correlations; magnitudes with the direction in the
    // key, as everywhere here.
    Figure {
        key: "dispersion/facilities-state-money",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "State capital money reaching Ohio's school districts, FY2009 to FY2024",
        pinned: 12_651_245_000.0,
        tolerance: 1000.0,
        compute: |i| i.facilities.districts.iter().map(|d| d.received).sum(),
    },
    Figure {
        key: "dispersion/facilities-capital-outlay",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "What those districts spent on capital over the same span, from every source",
        pinned: 33_283_507_000.0,
        tolerance: 1000.0,
        compute: |i| i.facilities.districts.iter().map(|d| d.spent).sum(),
    },
    Figure {
        key: "dispersion/facilities-state-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "So the state's share of what Ohio's districts spent on buildings",
        pinned: 0.3801,
        tolerance: 0.0005,
        compute: |i| {
            let received: f64 = i.facilities.districts.iter().map(|d| d.received).sum();
            let spent: f64 = i.facilities.districts.iter().map(|d| d.spent).sum();
            received / spent
        },
    },
    Figure {
        key: "dispersion/facilities-share-poorest-quintile",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The state's share for the fifth of districts with the least assessed value per \
                pupil",
        pinned: 0.5756,
        tolerance: 0.0005,
        compute: |i| i.facilities.by_wealth[0],
    },
    Figure {
        key: "dispersion/facilities-share-wealthiest-quintile",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "And for the fifth with the most, which is the program's own rule measured rather \
                than described",
        pinned: 0.1895,
        tolerance: 0.0005,
        compute: |i| i.facilities.by_wealth[4],
    },
    Figure {
        key: "dispersion/facilities-share-against-wealth-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "A district's fifteen-year state share against the log of its assessed value per \
                pupil \u{2014} negative",
        pinned: 0.4405,
        tolerance: 0.0005,
        compute: |_| dispersion::facilities::share_against_wealth().abs(),
    },
    Figure {
        key: "dispersion/facilities-smallest-district-total",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "What the least-funded of the 606 districts received across the whole span \u{2014} \
                the program is a queue ordered by poverty and it still reached every district",
        pinned: 730_000.0,
        tolerance: 1000.0,
        compute: |i| {
            i.facilities
                .districts
                .iter()
                .map(|d| d.received)
                .fold(f64::INFINITY, f64::min)
        },
    },
    Figure {
        key: "dispersion/facilities-fy2009",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "State capital money in FY2009, which is the largest year the panel holds",
        pinned: 1_356_609_000.0,
        tolerance: 1000.0,
        compute: |i| i.facilities.by_year[&2009].0,
    },
    Figure {
        key: "dispersion/facilities-fy2013",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "And in FY2013, four years later",
        pinned: 492_539_000.0,
        tolerance: 1000.0,
        compute: |i| i.facilities.by_year[&2013].0,
    },
    Figure {
        key: "dispersion/facilities-nonspecified-against-capital-lowest",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The weakest of fifteen yearly correlations between nonspecified state revenue and \
                capital outlay per pupil",
        pinned: 0.4733,
        tolerance: 0.0005,
        compute: |i| {
            i.facilities
                .against_capital
                .iter()
                .map(|(_, r)| *r)
                .fold(f64::INFINITY, f64::min)
        },
    },
    Figure {
        key: "dispersion/facilities-formula-against-capital-highest",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "And the strongest the same correlation reaches for general formula assistance, \
                which is the control the identification rests on",
        pinned: 0.1122,
        tolerance: 0.0005,
        compute: |i| {
            i.facilities
                .formula_against_capital
                .iter()
                .map(|(_, r)| *r)
                .fold(f64::NEG_INFINITY, f64::max)
        },
    },
    Figure {
        key: "dispersion/facilities-peaks-that-are-build-years",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Districts whose largest year of state capital money is a year their own capital \
                spending beat their own median",
        pinned: 0.818,
        tolerance: 0.0005,
        #[allow(clippy::cast_precision_loss)]
        compute: |i| i.facilities.peaks.0 as f64 / i.facilities.peaks.1 as f64,
    },
    // The break the same column has at FY2016, which is what put every figure above on a basis
    // rather than on the column as published. `Ratio` for the correlations, so the prose bound to
    // them carries no per-cent sign; magnitudes with the direction in the key, as everywhere here.
    Figure {
        key: "dispersion/state-column-break-year",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "The fiscal year the survey begins netting Ohio's community-school deduct out of \
                district state revenue, located from the data alone",
        pinned: 2016.0,
        tolerance: 0.0,
        compute: |_| f64::from(dispersion::survey_basis::break_year().expect("one break")),
    },
    Figure {
        key: "dispersion/fy2016-deduct-against-move-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "A district's community-school deduct against its FY2015-to-FY2016 change in \
                state revenue per pupil, on the column as published \u{2014} negative",
        pinned: 0.2506,
        tolerance: 0.0005,
        compute: |i| {
            i.basis
                .transitions
                .iter()
                .find(|t| t.to == 2016)
                .expect("the break")
                .as_published
                .abs()
        },
    },
    Figure {
        key: "dispersion/fy2016-deduct-against-move-corrected-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The same correlation with both years on one basis \u{2014} negative, and what is \
                left of it",
        pinned: 0.0183,
        tolerance: 0.0005,
        compute: |i| {
            i.basis
                .transitions
                .iter()
                .find(|t| t.to == 2016)
                .expect("the break")
                .corrected
                .abs()
        },
    },
    Figure {
        key: "dispersion/deduct-against-move-loudest-other-year",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "And the largest the same correlation reaches in any of the other ten transitions \
                the panel holds",
        pinned: 0.1026,
        tolerance: 0.0005,
        compute: |i| {
            i.basis
                .transitions
                .iter()
                .filter(|t| t.to != 2016)
                .map(|t| t.as_published.abs())
                .fold(0.0_f64, f64::max)
        },
    },
    Figure {
        key: "dispersion/fy2016-step-districts-corrected",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts falling more than a fifth across FY2016 once both eras are on one \
                basis, against the twenty-seven the published column shows",
        pinned: 19.0,
        tolerance: 0.0,
        #[allow(clippy::cast_precision_loss)]
        compute: |i| i.basis.corrected_fallers.0 as f64,
    },
    Figure {
        key: "dispersion/fy2016-step-pupil-share-corrected",
        owner: "crates/dispersion",
        // A `Ratio` and not a `Share`: 0.0166 is under this manifest's share floor, so the prose
        // bound to it carries the bare numeral and no per-cent sign.
        unit: Unit::Ratio,
        label: "And the share of the panel's pupils they hold \u{2014} a quarter of the published \
                figure",
        pinned: 0.0166,
        tolerance: 0.0005,
        compute: |i| i.basis.corrected_fallers.1,
    },
    Figure {
        key: "dispersion/fy2016-step-median-corrected",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The median district's FY2016 step on one basis",
        pinned: 0.1374,
        tolerance: 0.0005,
        compute: |i| i.basis.corrected_step[i.basis.corrected_step.len() / 2].1,
    },
    Figure {
        key: "dispersion/columbus-fy2016-step-corrected",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Columbus City's FY2016 step on one basis, against the 29.22% fall the published \
                column shows",
        pinned: 0.1493,
        tolerance: 0.0005,
        compute: |i| step_of(i, "043802"),
    },
    Figure {
        key: "dispersion/toledo-fy2016-step-corrected",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Toledo City's, against a published 20.49% fall",
        pinned: 0.1388,
        tolerance: 0.0005,
        compute: |i| step_of(i, "044909"),
    },
    Figure {
        key: "dispersion/dayton-fy2016-step-corrected",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "And Dayton City's, the largest gain among Ohio's fourteen largest districts",
        pinned: 0.3214,
        tolerance: 0.0005,
        compute: |i| step_of(i, "043844"),
    },
    Figure {
        key: "dispersion/fy2016-district-state-revenue-fall-negative",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "What Ohio's districts' state revenue fell across FY2016 on the published column \
                \u{2014} negative, as a magnitude",
        pinned: 849_921_000.0,
        tolerance: 1000.0,
        compute: |i| i.basis.fall_and_deduct.0.abs(),
    },
    Figure {
        key: "dispersion/fy2016-community-school-deduct",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "And what those districts paid community schools that year, which the survey \
                stopped crediting them with",
        pinned: 919_992_000.0,
        tolerance: 1000.0,
        compute: |i| i.basis.fall_and_deduct.1,
    },
    Figure {
        key: "dispersion/toledo-deduct-share-fy2015",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Toledo City's community-school deduct as a share of the state revenue the survey \
                credited it with in FY2015",
        pinned: 0.3529,
        tolerance: 0.0005,
        compute: |_| dispersion::survey_basis::deduct_share("044909", 2015).expect("FY2015"),
    },
    Figure {
        key: "dispersion/ohio-local-share-fy2015-published",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Ohio districts' local revenue share in FY2015 on the column as published",
        pinned: 0.5019,
        tolerance: 0.0005,
        compute: |i| i.basis.shares.0[&2015].0,
    },
    Figure {
        key: "dispersion/ohio-local-share-fy2015-corrected",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "And on the same basis FY2016 is already on, which removes five sixths of the \
                jump between them",
        pinned: 0.5239,
        tolerance: 0.0005,
        compute: |i| i.basis.shares.1[&2015].0,
    },
    // ECOT and Cleveland, on the series their nodes said were unpopulated.
    Figure {
        key: "dispersion/ecot-peak-enrolment",
        owner: "crates/dispersion",
        // A `Count` rather than `Pupils`: `V33` is fall membership, a headcount of children on a
        // day, not an average daily membership. The survey publishes it whole and it compares
        // exactly.
        unit: Unit::Count,
        label: "Electronic Classroom of Tomorrow peak enrolment, FY2016, on the Census count",
        pinned: 14_153.0,
        tolerance: 0.0,
        compute: |i| {
            i.closed
                .ecot
                .iter()
                .map(|y| y.enrolment)
                .fold(0.0, f64::max)
        },
    },
    Figure {
        key: "dispersion/ecot-peak-state-revenue",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "And the state money it received that year",
        pinned: 109_371_000.0,
        tolerance: 1000.0,
        compute: |i| {
            i.closed
                .ecot
                .iter()
                .map(|y| y.state * y.enrolment)
                .fold(0.0, f64::max)
        },
    },
    Figure {
        key: "dispersion/ecot-state-revenue-total",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "State money ECOT received across the eight years the survey holds it, FY2009 to \
                FY2017",
        pinned: 666_576_000.0,
        tolerance: 1000.0,
        compute: |i| i.closed.ecot.iter().map(|y| y.state * y.enrolment).sum(),
    },
    Figure {
        key: "dispersion/ecot-funding-against-the-average-district",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "ECOT's state revenue per pupil over the average Ohio district's, FY2016",
        pinned: 1.3995,
        tolerance: 0.0005,
        compute: |i| {
            let year = i
                .closed
                .ecot
                .iter()
                .find(|y| y.fiscal_year == 2016)
                .expect("the panel holds ECOT in FY2016");
            let (total, enrolment) = i.closed.districts[&2016];
            year.state / (total / enrolment)
        },
    },
    Figure {
        key: "dispersion/ecot-share-of-district-state-revenue",
        owner: "crates/dispersion",
        // A `Ratio` and not a `Share`, because 0.0124 is under the floor a share is allowed: at
        // that size the manifest cannot tell a fraction from a percentage typed by mistake. Prose
        // bound to this may not carry a per-cent sign.
        unit: Unit::Ratio,
        label: "ECOT's draw as a fraction of what Ohio's districts received in state money, FY2016",
        pinned: 0.0124,
        tolerance: 0.0005,
        compute: |i| {
            let year = i
                .closed
                .ecot
                .iter()
                .find(|y| y.fiscal_year == 2016)
                .expect("the panel holds ECOT in FY2016");
            year.state * year.enrolment / i.closed.districts[&2016].0
        },
    },
    Figure {
        key: "dispersion/cleveland-state-revenue-real-change-negative",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Cleveland Municipal state revenue per pupil, FY2010 to FY2024, net of the \
                community-school deduct in both years, in constant dollars \u{2014} negative",
        pinned: 0.12912,
        tolerance: 0.0005,
        compute: |i| i.basis.restated["043786"].real.abs(),
    },
    Figure {
        key: "dispersion/cleveland-enrolment-change-negative",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Cleveland Municipal enrolment over the same span \u{2014} negative",
        pinned: 0.3225,
        tolerance: 0.0005,
        compute: |i| i.closed.cleveland_enrolment.nominal.abs(),
    },
    // The pair's fifteen years, and the four measures its two nodes recorded as not held. Real
    // changes are exported as magnitudes with the direction in the key, except Perrysburg's local
    // revenue, which is the one that rises and whose sign is the finding.
    Figure {
        key: "dispersion/toledo-state-revenue-real-change-negative",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Toledo City state revenue per pupil, FY2010 to FY2024, net of the \
                community-school deduct in both years, in constant dollars \u{2014} negative",
        pinned: 0.15062,
        tolerance: 0.0005,
        compute: |i| i.basis.restated["044909"].real.abs(),
    },
    Figure {
        key: "dispersion/perrysburg-state-revenue-real-change-negative",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Perrysburg Exempted Village state revenue per pupil over the same span and on \
                the same basis \u{2014} negative, and deeper than Toledo's",
        pinned: 0.20286,
        tolerance: 0.0005,
        compute: |i| i.basis.restated["045583"].real.abs(),
    },
    Figure {
        key: "dispersion/toledo-local-revenue-real-change-negative",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Toledo City local revenue per pupil over the same span, in constant dollars \
                \u{2014} negative",
        pinned: 0.08141,
        tolerance: 0.0005,
        compute: |i| i.pair.toledo_local.real.abs(),
    },
    Figure {
        key: "dispersion/perrysburg-local-revenue-real-change",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Perrysburg Exempted Village local revenue per pupil over the same span \u{2014} \
                the one series of the four that rises",
        pinned: 0.22336,
        tolerance: 0.0005,
        compute: |i| i.pair.perrysburg_local.real,
    },
    Figure {
        key: "dispersion/toledo-valuation-per-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Toledo City assessed valuation per pupil, TY2023",
        pinned: 135_506.32,
        tolerance: 1.0,
        compute: |i| i.pair.toledo.valuation_per_pupil,
    },
    Figure {
        key: "dispersion/perrysburg-valuation-per-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Perrysburg Exempted Village assessed valuation per pupil, TY2023",
        pinned: 270_202.33,
        tolerance: 1.0,
        compute: |i| i.pair.perrysburg.valuation_per_pupil,
    },
    Figure {
        key: "dispersion/toledo-effective-millage",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Toledo City effective Class I operating millage, TY2023",
        pinned: 35.5476,
        tolerance: 0.0005,
        compute: |i| i.pair.toledo.effective_millage,
    },
    Figure {
        key: "dispersion/perrysburg-effective-millage",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Perrysburg Exempted Village effective Class I operating millage, TY2023 \u{2014} \
                the higher of the two",
        pinned: 39.15,
        tolerance: 0.0005,
        compute: |i| i.pair.perrysburg.effective_millage,
    },
    Figure {
        key: "dispersion/toledo-classroom-share-percentile",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Where Toledo City's classroom share sits among the 606 districts, as a percentile",
        pinned: 18.512,
        tolerance: 0.05,
        compute: |i| i.pair.toledo.classroom_share_percentile,
    },
    Figure {
        key: "dispersion/perrysburg-classroom-share-percentile",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "And where Perrysburg Exempted Village's sits",
        pinned: 95.041,
        tolerance: 0.05,
        compute: |i| i.pair.perrysburg.classroom_share_percentile,
    },
    Figure {
        key: "dispersion/fy2016-step-districts",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts whose state revenue per pupil fell more than a fifth across FY2016 and \
                stayed down, on the column as published \u{2014} which charges each district its \
                own community-school deduct",
        pinned: 27.0,
        tolerance: 0.0,
        compute: |i| i.pair.step.iter().filter(|s| s.1 < -0.20).count() as f64,
    },
    Figure {
        key: "dispersion/fy2016-step-pupil-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The share of the panel's pupils those districts hold, on the same published \
                column",
        pinned: 0.06700,
        tolerance: 0.0005,
        compute: |i| {
            let all: f64 = i.pair.step.iter().map(|s| s.2).sum();
            i.pair
                .step
                .iter()
                .filter(|s| s.1 < -0.20)
                .map(|s| s.2)
                .sum::<f64>()
                / all
        },
    },
    // What the money was spent on, entered into the model that only knew how much. The coefficients
    // are standardised — standard deviations of outcome per standard deviation of predictor — so
    // that a dollar figure and a percentage-point share can be read against each other at all. The
    // negative ones follow this manifest's convention: exported as magnitudes with the direction in
    // the key, because `numerals()` does not read a sign.
    Figure {
        key: "dispersion/classroom-spending-on-growth",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Classroom instruction per pupil against the Progress effect size, standardised, \
                with non-classroom spending and five controls",
        pinned: 0.2432,
        tolerance: 0.0005,
        compute: |i| i.composition.split.standardized[0],
    },
    Figure {
        key: "dispersion/nonclassroom-spending-on-growth-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Non-classroom spending per pupil against the Progress effect size in the same \
                model \u{2014} negative, and not distinguishable from zero",
        pinned: 0.0320,
        tolerance: 0.0005,
        compute: |i| i.composition.split.standardized[1].abs(),
    },
    Figure {
        key: "dispersion/classroom-share-on-growth",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Classroom share of operating spending against the Progress effect size, holding \
                total spending and five controls fixed",
        pinned: 0.1215,
        tolerance: 0.0005,
        compute: |i| i.composition.on_growth.standardized[1],
    },
    Figure {
        key: "dispersion/classroom-share-on-level",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The same share against the Performance Index, where the level of spending runs \
                the other way",
        pinned: 0.0948,
        tolerance: 0.0005,
        compute: |i| i.composition.on_level.standardized[1],
    },
    Figure {
        key: "dispersion/spending-level-on-level-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Operating spending per pupil against the Performance Index in the model that also \
                carries the classroom share \u{2014} negative",
        pinned: 0.0562,
        tolerance: 0.0005,
        compute: |i| i.composition.on_level.standardized[0].abs(),
    },
    Figure {
        key: "dispersion/instruction-on-growth",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Instruction per pupil against the Progress effect size, all nine published \
                functions entered at once",
        pinned: 0.1944,
        tolerance: 0.0005,
        compute: |i| i.composition.functions.standardized[0],
    },
    Figure {
        key: "dispersion/plant-maintenance-on-growth-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Operations and maintenance of plant in the same model \u{2014} negative, and the \
                only function that is reliably so",
        pinned: 0.1259,
        tolerance: 0.0005,
        compute: |i| i.composition.functions.standardized[5].abs(),
    },
    Figure {
        key: "dispersion/pupil-transportation-on-growth-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Pupil transportation in the same model \u{2014} the corpus's own example, and not \
                distinguishable from zero",
        pinned: 0.0346,
        tolerance: 0.0005,
        compute: |i| i.composition.functions.standardized[6].abs(),
    },
    Figure {
        key: "dispersion/median-classroom-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Median district's classroom share of operating spending, FY2025",
        pinned: 0.6700,
        tolerance: 0.0005,
        compute: |i| {
            let shares: Vec<f64> = i
                .composition
                .districts
                .iter()
                .map(|d| d.classroom_share() / 100.0)
                .collect();
            dispersion::median(&{
                let mut sorted = shares;
                sorted.sort_by(f64::total_cmp);
                sorted
            })
            .expect("districts report a classroom share")
        },
    },
    Figure {
        key: "dispersion/classroom-share-against-spending-negative",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Correlation between the classroom share and operating spending per pupil \
                \u{2014} negative: spending more buys a smaller classroom fraction",
        pinned: 0.3371,
        tolerance: 0.0005,
        compute: |i| {
            let shares: Vec<f64> = i
                .composition
                .districts
                .iter()
                .map(dispersion::composition::District::classroom_share)
                .collect();
            let totals: Vec<f64> = i.composition.districts.iter().map(|d| d.operating).collect();
            dispersion::wealth_neutrality(&shares, &totals)
                .expect("paired series")
                .correlation
                .abs()
        },
    },
    // The same statistic as the line above, computed the way 20 U.S.C. 6337 computes it, so that
    // 0.2016 finally has something to be read against. Enrolment-weighted, on the Census Bureau's
    // current spending for FY2022 rather than the department's operating expenditure for FY2024,
    // and over the comparable agencies only — see `dispersion::equity_factor`, which says at
    // length why that filter is not optional here.
    Figure {
        key: "dispersion/ohio-equity-factor",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Ohio's Education Finance Incentive Grant equity factor, FY2022",
        pinned: 0.19873,
        tolerance: 0.0005,
        compute: |i| equity_of(i, "OH").factor,
    },
    Figure {
        key: "dispersion/ohio-equity-factor-rank",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Where Ohio's equity factor ranks among the 51 jurisdictions, widest spread first",
        pinned: 9.0,
        tolerance: 0.0,
        compute: |i| {
            i.equity
                .iter()
                .position(|s| s.state == "OH")
                .expect("the national panel carries Ohio") as f64
                + 1.0
        },
    },
    Figure {
        key: "dispersion/median-state-equity-factor",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The median jurisdiction's equity factor, FY2022",
        pinned: 0.15818,
        tolerance: 0.0005,
        compute: |i| {
            let mut factors: Vec<f64> = i.equity.iter().map(|s| s.factor).collect();
            factors.sort_by(f64::total_cmp);
            factors[factors.len() / 2]
        },
    },
    Figure {
        key: "dispersion/ohio-equity-factor-distance-to-band-edge",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How far Ohio's equity factor sits below the 0.20 edge where the within-state \
                poverty ladder steepens",
        pinned: 0.00127,
        tolerance: 0.0005,
        compute: |i| equity_of(i, "OH").distance_to_edge(),
    },
    Figure {
        key: "dispersion/ohio-equity-factor-poverty-adjusted",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Ohio's equity factor with the statutory 1.4 poverty weight over-applied to its \
                economically disadvantaged count — the low end of the bracket",
        pinned: 0.16840,
        tolerance: 0.0005,
        compute: |i| i.equity_bracket.0,
    },
    Figure {
        key: "dispersion/ohio-efig-rate-against-a-median-state",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "How far Ohio's `1.30 minus equity factor` per-child rate falls below a median \
                jurisdiction's",
        pinned: 0.0355,
        tolerance: 0.0005,
        compute: |i| {
            let mut factors: Vec<f64> = i.equity.iter().map(|s| s.factor).collect();
            factors.sort_by(f64::total_cmp);
            let median = dispersion::equity_factor::STATE_RATE_CONSTANT - factors[factors.len() / 2];
            1.0 - equity_of(i, "OH").state_rate() / median
        },
    },
    Figure {
        key: "dispersion/operating-expenditure-restricted-range-ratio",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "P95 over P5 of operating expenditure per pupil, FY2024 — the restricted range \
                ratio, which is the federal range ratio plus one",
        pinned: 1.8436,
        tolerance: 0.0005,
        compute: |i| operating_dispersion(i).restricted_range_ratio,
    },
    Figure {
        key: "dispersion/state-aid-falls-with-property-wealth",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How strongly state aid per pupil falls as valuation per pupil rises, FY2024 — \
                the magnitude of a negative correlation",
        pinned: 0.5483,
        tolerance: 0.0005,
        compute: |i| {
            let (wealth, aid) = dispersion::profile::paired(
                &i.profile,
                |d| d.valuation_per_pupil,
                |d| d.state_revenue_per_pupil,
            );
            dispersion::wealth_neutrality(&wealth, &aid)
                .expect("a paired series")
                .correlation
                .abs()
        },
    },
    Figure {
        key: "dispersion/state-aid-rises-with-poverty",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How strongly state aid per pupil rises with the economically disadvantaged \
                share, FY2024 — the finding that Ohio targets poverty better than property",
        pinned: 0.6323,
        tolerance: 0.0005,
        compute: |i| {
            let (poverty, aid) = dispersion::profile::paired(
                &i.profile,
                |d| d.economically_disadvantaged,
                |d| d.state_revenue_per_pupil,
            );
            dispersion::wealth_neutrality(&poverty, &aid)
                .expect("a paired series")
                .correlation
        },
    },
    // `metric/performance-index`. The dominant fact about Ohio's attainment measure is poverty,
    // and the node's whole argument is a table of correlations — every one of which was computed
    // in a test file on a private parser until #157.
    Figure {
        key: "dispersion/performance-index-tracks-poverty",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How strongly the Performance Index falls with the profile report's economically \
                disadvantaged share — the magnitude of a negative correlation",
        pinned: 0.84596,
        tolerance: 0.0005,
        compute: |i| {
            outcome_correlation(
                i,
                |_, p| p?.economically_disadvantaged,
                |c| c.performance_index,
            )
        },
    },
    Figure {
        key: "dispersion/performance-index-variance-poverty-explains",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The share of cross-district variance in the Performance Index that the \
                disadvantaged share accounts for",
        pinned: 0.71564,
        tolerance: 0.0005,
        compute: |i| {
            let r = outcome_correlation(
                i,
                |_, p| p?.economically_disadvantaged,
                |c| c.performance_index,
            );
            r * r
        },
    },
    Figure {
        key: "dispersion/performance-index-tracks-the-top-coded-poverty-measure",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The same against the report card's own disadvantaged share, which community \
                eligibility top-codes — the weaker association censoring predicts",
        pinned: 0.73434,
        tolerance: 0.0005,
        compute: |i| {
            outcome_correlation(
                i,
                |c, _| c.economically_disadvantaged,
                |c| c.performance_index,
            )
        },
    },
    // `accountability-regime/ohio-report-card`. The achievement component is awarded on the Index
    // over a statewide maximum refitted every year, so the rating is a share and the Index is a
    // level. Bound here because the whole finding is a comparison of two years of a denominator,
    // and a report-card refresh moves both ends of it without touching a word of the prose.
    Figure {
        key: "dispersion/achievement-maximum-2425",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The statewide achievement denominator for 2024-25 \u{2014} the average of the \
                highest two per cent of district Performance Index scores, as published",
        pinned: 109.8,
        tolerance: 0.05,
        compute: |_| {
            let published: Vec<f64> = dispersion::report_card::report_cards()
                .iter()
                .filter_map(|c| c.performance_index_maximum)
                .collect();
            assert!(
                published.windows(2).all(|w| w[0] == w[1]),
                "the maximum is one statewide number and this file no longer says so"
            );
            published[0]
        },
    },
    Figure {
        key: "dispersion/achievement-maximum-2223",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The same denominator two years earlier, recomputed because the department \
                publishes the maximum for the current year alone",
        pinned: 108.8,
        tolerance: 0.05,
        compute: |_| {
            achievement_denominator(
                dispersion::report_card::report_cards()
                    .iter()
                    .filter_map(|c| c.performance_index_earliest)
                    .collect(),
            )
        },
    },
    Figure {
        key: "dispersion/achievement-top-districts-averaged",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "How many districts the statute\u{2019}s highest two per cent resolves to \u{2014} \
                the ceiling of two per cent of 607, and the only count that reproduces the \
                published maximum",
        pinned: 13.0,
        tolerance: 0.0,
        compute: |_| (dispersion::report_card::report_cards().len() * 2).div_ceil(100) as f64,
    },
    Figure {
        key: "dispersion/districts-rising-on-index-and-falling-on-share",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts that improved on the Performance Index between 2022-23 and 2024-25 and \
                declined as a share of the year's maximum \u{2014} the cost of an annually \
                refitted denominator",
        pinned: 121.0,
        tolerance: 0.0,
        compute: |_| {
            let rows = dispersion::report_card::report_cards();
            // The published maximum for the current year and the recomputed one for the earlier
            // year, which is the only pair available: the department publishes the maximum for
            // the year the card covers and no other. Recomputing *both* is the obvious
            // alternative and is wrong here — it would divide 2024-25 by 109.8077 where the
            // department divided by 109.8, and the count is decided at the fifth decimal place.
            // `crates/dispersion/tests/the_weights_behind_the_index.rs` makes the same choice and
            // pins how little room there is.
            let then = achievement_denominator(
                rows.iter().filter_map(|c| c.performance_index_earliest).collect(),
            );
            rows.iter()
                .filter(|c| {
                    match (
                        c.performance_index_earliest,
                        c.performance_index,
                        c.performance_index_maximum,
                    ) {
                        (Some(before), Some(after), Some(now)) => {
                            after > before && after / now <= before / then
                        }
                        _ => false,
                    }
                })
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/achievement-denominator-drift",
        owner: "crates/dispersion",
        // Percentage points as a `Ratio`, and the two guards between them left no other choice.
        // `Share` refuses anything under 0.02, because a share that small would survive its own
        // hundredfold typo — and 0.00919 is under it. `Ratio` then refuses any prose numeral
        // carrying a `%`, on the reading that a percent sign means the author meant a share. So a
        // growth rate below two per cent, written with the sign, fits neither unit. The prose
        // spells the words instead, which is this node's style for the statute's own "two per
        // cent" anyway. If a third sub-percent rate wants binding, the gap is worth a unit rather
        // than a third comment like this one.
        unit: Unit::Ratio,
        label: "How far the achievement denominator rose between 2022-23 and 2024-25, in per \
                cent \u{2014} the amount every district\u{2019}s share fell relative to its own \
                index, and the robust half of the finding the 121 counts",
        pinned: 0.919,
        tolerance: 0.005,
        compute: |_| {
            let rows = dispersion::report_card::report_cards();
            let then = achievement_denominator(
                rows.iter().filter_map(|c| c.performance_index_earliest).collect(),
            );
            let now = rows
                .iter()
                .find_map(|c| c.performance_index_maximum)
                .expect("a published maximum");
            (now / then - 1.0) * 100.0
        },
    },
    Figure {
        key: "dispersion/achievement-modal-rating-districts",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts holding the most common achievement star rating \u{2014} the number \
                R.C. 3302.03(D)(4)(b) requires stay under half the state",
        pinned: 227.0,
        tolerance: 0.0,
        compute: |_| modal_achievement_rating().0 as f64,
    },
    Figure {
        key: "dispersion/achievement-modal-rating-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The same as a share of the rated districts \u{2014} how much room the statute's \
                distributional constraint actually has",
        pinned: 0.37397,
        tolerance: 0.0005,
        compute: |_| {
            let (modal, rated) = modal_achievement_rating();
            modal as f64 / rated as f64
        },
    },
    Figure {
        key: "dispersion/report-card-buildings",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Buildings on the 2024-25 report card \u{2014} the population federal \
                identification selects from, and the denominator every share below is taken over",
        pinned: 3318.0,
        tolerance: 0.0,
        compute: |_| dispersion::building::buildings().len() as f64,
    },
    Figure {
        key: "dispersion/buildings-rated-on-achievement",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Buildings the department gave a Performance Index and a star \u{2014} 167 fewer \
                than it lists, and the population a rank order can be taken over",
        pinned: 3151.0,
        tolerance: 0.0,
        compute: |_| dispersion::building::by_index().len() as f64,
    },
    Figure {
        key: "dispersion/identifications",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Federal identifications across the three tiers, every one of which names a \
                building the report card carries",
        pinned: 408.0,
        tolerance: 0.0,
        compute: |_| identified_buildings().len() as f64,
    },
    Figure {
        key: "dispersion/csi-identified-since-2018",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Comprehensive Support schools still carrying a 2018 identification \u{2014} \
                seven years against exit criteria that allow three",
        pinned: 100.0,
        tolerance: 0.0,
        compute: |_| {
            comprehensive_support()
                .iter()
                .filter(|(i, _)| i.year_identified == "2018")
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/csi-2018-cohort-chronic-absenteeism-median",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Median chronic absenteeism in that cohort, in percentage points \u{2014} what \
                distinguishes the schools that have not exited, and it is not the index",
        pinned: 80.7,
        tolerance: 0.05,
        compute: |_| {
            median(
                comprehensive_support()
                    .iter()
                    .filter(|(i, _)| i.year_identified == "2018")
                    .filter_map(|(_, b)| b.chronic_absenteeism)
                    .collect(),
            )
        },
    },
    Figure {
        key: "dispersion/building-chronic-absenteeism-median",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Median chronic absenteeism across every building in the state, in percentage \
                points \u{2014} the comparison the cohort figure is only meaningful against",
        pinned: 20.8,
        tolerance: 0.05,
        compute: |_| {
            median(
                dispersion::building::buildings()
                    .iter()
                    .filter_map(|b| b.chronic_absenteeism)
                    .collect(),
            )
        },
    },
    Figure {
        key: "dispersion/csi-escalated-above-two-stars",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Comprehensive Support schools rated three stars or better \u{2014} every one of \
                which arrived through a subgroup rather than by performing in the bottom 5%",
        pinned: 11.0,
        tolerance: 0.0,
        compute: |_| {
            comprehensive_support()
                .iter()
                .filter(|(_, b)| b.achievement_stars.is_some_and(|s| s >= 3.0))
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/csi-identification-overlap-band",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Buildings between the lowest Performance Index on no federal list and the \
                highest on the Comprehensive Support list \u{2014} the band inside which the \
                published rating does not say who is identified",
        pinned: 1958.0,
        tolerance: 0.0,
        compute: |_| {
            let listed = federally_listed();
            let in_csi: BTreeSet<String> =
                comprehensive_support().into_iter().map(|(_, b)| b.irn).collect();
            let rated = dispersion::building::by_index();
            let floor = rated
                .iter()
                .filter(|b| !listed.contains(&b.irn))
                .map(building_index)
                .fold(f64::MAX, f64::min);
            let ceiling = rated
                .iter()
                .filter(|b| in_csi.contains(&b.irn))
                .map(building_index)
                .fold(f64::MIN, f64::max);
            rated.iter().filter(|b| (floor..=ceiling).contains(&building_index(b))).count() as f64
        },
    },
    Figure {
        key: "dispersion/csi-contested-index-values",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Published Performance Index values carrying both a Comprehensive Support school \
                and a building on no federal list \u{2014} exact agreement no threshold on the \
                index can separate",
        pinned: 113.0,
        tolerance: 0.0,
        compute: |_| {
            let listed = federally_listed();
            let in_csi: BTreeSet<String> =
                comprehensive_support().into_iter().map(|(_, b)| b.irn).collect();
            let mut at_value: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for building in dispersion::building::by_index() {
                let index = building_index(&building);
                at_value.entry(format!("{index:.1}")).or_default().push(building.irn);
            }
            at_value
                .values()
                .filter(|group| {
                    group.iter().any(|irn| in_csi.contains(irn))
                        && group.iter().any(|irn| !listed.contains(irn))
                })
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/one-star-buildings",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Buildings Ohio rates one star on achievement \u{2014} the state's own \
                worst-rated population, and not the population the federal list holds",
        pinned: 347.0,
        tolerance: 0.0,
        compute: |_| {
            dispersion::building::buildings()
                .iter()
                .filter(|b| b.achievement_stars == Some(1.0))
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/one-star-buildings-on-no-federal-list",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "One-star buildings on none of the three federal lists \u{2014} nearly half the \
                state's worst-rated buildings, outside the regime that identifies on that rating",
        pinned: 164.0,
        tolerance: 0.0,
        compute: |_| {
            let listed = federally_listed();
            dispersion::building::buildings()
                .iter()
                .filter(|b| b.achievement_stars == Some(1.0) && !listed.contains(&b.irn))
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/anton-grdina-rank",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Where the corpus's worst-performing exemplar building ranks on the published \
                Performance Index \u{2014} the bottom one per cent, and on no federal list",
        pinned: 23.0,
        tolerance: 0.0,
        compute: |_| {
            dispersion::building::by_index()
                .iter()
                .position(|b| b.irn == "000828")
                .expect("Anton Grdina is rated") as f64
                + 1.0
        },
    },
    Figure {
        key: "dispersion/buildings-below-anton-grdina-identified",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "How many of the 22 buildings scoring below it are federally identified \u{2014} \
                what makes its own absence a question about eligibility rather than performance",
        pinned: 18.0,
        tolerance: 0.0,
        compute: |_| {
            let listed = federally_listed();
            let rated = dispersion::building::by_index();
            let rank = rated
                .iter()
                .position(|b| b.irn == "000828")
                .expect("Anton Grdina is rated");
            rated[..rank].iter().filter(|b| listed.contains(&b.irn)).count() as f64
        },
    },
    Figure {
        key: "dispersion/csi-past-the-three-year-exit",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Comprehensive Support schools identified more than three years ago \u{2014} the \
                population the sixteen-rung intervention ladder can reach today",
        pinned: 127.0,
        tolerance: 0.0,
        compute: |_| past_the_three_year_exit().len() as f64,
    },
    Figure {
        key: "dispersion/csi-past-the-exit-pupils",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Pupils enrolled in those schools \u{2014} a headcount off the report card, not an \
                average daily membership",
        pinned: 50086.0,
        tolerance: 0.0,
        compute: |_| {
            past_the_three_year_exit().iter().filter_map(|(_, b)| b.enrollment).sum()
        },
    },
    Figure {
        key: "dispersion/csi-past-the-exit-as-their-own-agency",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "How many of them are their own local education agency \u{2014} already outside a \
                school district, so the conversion and merger rungs have nowhere to move them",
        pinned: 82.0,
        tolerance: 0.0,
        compute: |_| {
            past_the_three_year_exit()
                .iter()
                .filter(|(i, _)| i.building_irn == i.lea_irn)
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/csi-past-the-exit-pupils-outside-a-district",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Pupils in those self-operating schools \u{2014} three fifths of the population \
                the ladder reaches",
        pinned: 30085.0,
        tolerance: 0.0,
        compute: |_| {
            past_the_three_year_exit()
                .iter()
                .filter(|(i, _)| i.building_irn == i.lea_irn)
                .filter_map(|(_, b)| b.enrollment)
                .sum()
        },
    },
    Figure {
        key: "project/lsc-education-greenbooks",
        owner: "crates/project",
        unit: Unit::Count,
        label: "LSC education analyses committed \u{2014} one per enacted budget act from the \
                124th General Assembly to the 135th, and the only source the charge-off's last \
                two eras are stated in",
        pinned: 12.0,
        tolerance: 0.0,
        compute: |_| project::greenbook::greenbooks().len() as f64,
    },
    Figure {
        key: "project/evidence-based-model-local-share-mills",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The local share the Evidence-Based Model charged, in mills \u{2014} one rate \
                against two bases, split by whether a district sits at the twenty-mill floor",
        pinned: 22.0,
        tolerance: 0.0005,
        compute: |_| {
            mills_before(
                &evidence_based_local_share(),
                "(2.2%) of total taxable valuation",
            )
        },
    },
    Figure {
        key: "project/prior-law-non-base-cost-charge-off-mills",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The second charge-off before FY2010, in mills \u{2014} special education, \
                career-technical education and transportation, which the corpus's series omits",
        pinned: 3.3,
        tolerance: 0.0005,
        compute: |_| {
            mills_before(
                &evidence_based_local_share(),
                "(0.33%) of their recognized",
            )
        },
    },
    Figure {
        key: "project/bridge-formula-implied-charge-off-low",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The lowest charge-off rate the Bridge formula's state share index implies, in \
                mills, at FY2014 \u{2014} the bottom of a spread that replaced a uniform rate",
        pinned: 11.3,
        tolerance: 0.0005,
        compute: |_| mills_before(&bridge_implied_charge_off(), "to 22.9 mills"),
    },
    Figure {
        key: "project/bridge-formula-implied-charge-off-high",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The highest, excluding outlier districts \u{2014} twice the lowest, which is what \
                stops the average below from being a rate",
        pinned: 22.9,
        tolerance: 0.0005,
        compute: |_| {
            mills_before(
                &bridge_implied_charge_off(),
                "(excluding several outlier districts)",
            )
        },
    },
    Figure {
        key: "project/bridge-formula-implied-charge-off-average",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statewide average of that spread at FY2014, in mills \u{2014} the number \
                secondary reporting rounds to twenty and states as though it were legislated",
        pinned: 20.6,
        tolerance: 0.0005,
        compute: |_| mills_before(&bridge_implied_charge_off(), "Targeted Assistance"),
    },
    Figure {
        key: "project/edchoice-full-award-ceiling",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "Family income, as a multiple of the federal poverty guidelines, at or below which \
                the EdChoice Expansion pays its full base amount",
        pinned: 4.5,
        tolerance: 0.0,
        compute: |_| project::scholarship::FULL_AWARD_CEILING,
    },
    Figure {
        key: "project/edchoice-minimum-award-threshold",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "And where the award stops falling \u{2014} 7.82 times poverty, derived from the \
                statute's own decay rate and its ten per cent floor, and stated nowhere in it",
        pinned: 7.82193,
        tolerance: 0.00001,
        compute: |_| project::scholarship::minimum_award_threshold(),
    },
    Figure {
        key: "project/edchoice-award-at-five-hundred-and-fifty-per-cent",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What a kindergarten-through-eight student at 550% of poverty is awarded \u{2014} \
                half the base, because the curve halves for every further hundred points",
        pinned: 2750.0,
        tolerance: 0.005,
        compute: |_| {
            project::scholarship::expansion_award(project::scholarship::EDCHOICE_BASE_K8, 5.5)
        },
    },
    Figure {
        key: "project/edchoice-minimum-award-k8",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The smallest EdChoice Expansion award for grades kindergarten through eight \
                \u{2014} a tenth of the base, and what every family above 782% of poverty gets",
        pinned: 550.0,
        tolerance: 0.005,
        compute: |_| {
            project::scholarship::expansion_award(project::scholarship::EDCHOICE_BASE_K8, 100.0)
        },
    },
    Figure {
        key: "project/jon-peterson-award-severest-category",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the Jon Peterson formula computes for a category six student before its own \
                ceiling \u{2014} $7,190 plus the largest supplement",
        pinned: 39122.0,
        tolerance: 0.005,
        compute: |_| {
            project::scholarship::JON_PETERSON_BASE + project::scholarship::JON_PETERSON_SUPPLEMENTS[5]
        },
    },
    Figure {
        key: "project/jon-peterson-ceiling-haircut",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "How much the unindexed $34,000 ceiling takes off that award \u{2014} the ceiling \
                already binds the severest category, before any year of indexing",
        pinned: 5122.0,
        tolerance: 0.005,
        compute: |_| {
            project::scholarship::JON_PETERSON_BASE
                + project::scholarship::JON_PETERSON_SUPPLEMENTS[5]
                - project::scholarship::jon_peterson_award(6)
        },
    },
    Figure {
        key: "project/dpia-amount-before-the-plan",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The per-pupil economically disadvantaged amount the Fair School Funding Plan \
                replaced \u{2014} what $422 was raised from, and the only prior value the corpus \
                holds",
        pinned: 272.0,
        tolerance: 0.005,
        compute: |_| {
            amount_after(
                &project::greenbook::greenbook("hb110").flat(),
                "with the base per-pupil amount increased from ",
            )
        },
    },
    Figure {
        key: "project/gifted-identification-before-the-plan",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The per-pupil gifted identification rate before the plan raised it to $24 \
                \u{2014} a more-than-fourfold increase, and the rate a series would start from",
        pinned: 5.5,
        tolerance: 0.005,
        compute: |_| {
            amount_after(
                &project::greenbook::greenbook("hb110").flat(),
                "gifted identification funds from ",
            )
        },
    },
    Figure {
        key: "project/districts-at-the-five-per-cent-floor-fy2024",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts the minimum state share reached in FY2024, the first year at ten per \
                cent \u{2014} what makes the floor a live parameter rather than a formality",
        pinned: 55.0,
        tolerance: 0.0,
        compute: |_| {
            amount_after(
                &project::greenbook::greenbook("hb33").flat(),
                "The minimum state share percentage applies to an estimated ",
            )
        },
    },
    Figure {
        key: "project/districts-at-the-five-per-cent-floor-fy2025",
        owner: "crates/project",
        unit: Unit::Count,
        label: "And in FY2025 \u{2014} thirteen more districts in one year, on a floor that did \
                not move",
        pinned: 68.0,
        tolerance: 0.0,
        compute: |_| {
            amount_after(
                &project::greenbook::greenbook("hb33").flat(),
                "districts in FY 2024 and ",
            )
        },
    },
    Figure {
        key: "project/transportation-floor-fy2023",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The minimum transportation state share in FY2023, in percentage points \u{2014} \
                the first term of a schedule that reaches fifty per cent in FY2027",
        pinned: 33.33,
        tolerance: 0.005,
        compute: |_| {
            amount_after(
                &project::greenbook::greenbook("hb33").flat(),
                "The minimum state share for transportation increases from ",
            )
        },
    },
    Figure {
        key: "project/analyses-stating-the-preschool-flat-grant",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Consecutive enacted budget analyses stating the $4,000 preschool grant and its \
                half-day multiplier \u{2014} FY2014 through FY2025, across a change of funding \
                regime",
        pinned: 6.0,
        tolerance: 0.0,
        compute: |_| {
            project::greenbook::greenbooks()
                .iter()
                .filter(|book| book.flat().contains("$4,000 for each preschool"))
                .count() as f64
        },
    },
    Figure {
        key: "project/foundation-funding-missed-in-fy2020",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "How far foundation funding fell below its enacted FY2020 appropriation \u{2014} \
                the year the guarantee's base is defined to exclude the reductions from",
        pinned: 254956619.56,
        tolerance: 0.005,
        compute: |_| {
            let (enacted, actual) = foundation_in(2020);
            enacted - actual
        },
    },
    Figure {
        key: "project/foundation-funding-missed-in-fy2020-share",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The same as a share of the appropriation, in percentage points \u{2014} against a \
                band of one point either way in every ordinary year of the two formula regimes",
        pinned: 3.67,
        tolerance: 0.005,
        compute: |_| {
            let (enacted, actual) = foundation_in(2020);
            (enacted - actual) / enacted * 100.0
        },
    },
    Figure {
        key: "dispersion/the-two-poverty-measures-against-each-other",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How closely Ohio's two economically-disadvantaged shares track each other — \
                closely enough to look interchangeable and not closely enough to be",
        pinned: 0.82273,
        tolerance: 0.0005,
        compute: |i| {
            outcome_correlation(
                i,
                |_, p| p?.economically_disadvantaged,
                |c| c.economically_disadvantaged,
            )
        },
    },
    Figure {
        key: "dispersion/districts-at-the-report-card-poverty-ceiling",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts the report card places at exactly 100% economically disadvantaged",
        pinned: 87.0,
        tolerance: 0.0,
        compute: |i| {
            i.outcomes
                .iter()
                .filter(|(c, _)| c.economically_disadvantaged.is_some_and(|v| v >= 99.95))
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/districts-at-the-profile-poverty-ceiling",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts the profile report places at 100% — the same ceiling on the \
                uncensored measure, and a third as many",
        pinned: 37.0,
        tolerance: 0.0,
        compute: |i| {
            i.profile
                .iter()
                .filter(|d| d.economically_disadvantaged.is_some_and(|v| v >= 0.9995))
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/performance-index-tracks-the-adm-weight-ratio",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How strongly the Performance Index falls with the weighted-over-headcount ADM \
                ratio — which is why dividing spending by the weighted count removes the signal",
        pinned: 0.74455,
        tolerance: 0.0005,
        compute: |i| outcome_correlation(i, |c, _| c.weight_ratio(), |c| c.performance_index),
    },
    Figure {
        key: "dispersion/performance-index-tracks-spending-per-headcount-pupil",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The Performance Index against FY2025 operating expenditure per *headcount* \
                pupil — the divisor that leaves the association visible",
        pinned: 0.33652,
        tolerance: 0.0005,
        compute: |i| outcome_correlation(i, |c, _| c.per_enrolled_pupil(), |c| c.performance_index),
    },
    Figure {
        key: "dispersion/performance-index-tracks-spending-per-weighted-pupil",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The same on the department's published divisor, the weighted count — near zero, \
                which is the published finding this corpus disagrees with",
        pinned: 0.01552,
        tolerance: 0.0005,
        compute: |i| outcome_correlation(i, |c, _| c.per_equivalent_pupil, |c| c.performance_index),
    },
    Figure {
        key: "dispersion/performance-index-tracks-federal-spending-per-pupil",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The Performance Index against federal expenditure per equivalent pupil — OCG \
                White Paper 013's strongest reported finding",
        pinned: 0.55764,
        tolerance: 0.0005,
        compute: |i| {
            outcome_correlation(
                i,
                |c, _| c.per_equivalent_pupil_federal,
                |c| c.performance_index,
            )
        },
    },
    Figure {
        key: "dispersion/attainment-against-growth",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The Performance Index against the value-added effect size — related, and far \
                from interchangeable",
        pinned: 0.37525,
        tolerance: 0.0005,
        compute: |i| outcome_correlation(i, |c, _| c.progress_effect_size, |c| c.performance_index),
    },
    // The twenty-mill floor, from the two tables that measure it. `parameter/twenty-mill-floor`
    // is the node; the count of districts sitting on the floor is the corpus's answer to what it
    // recorded for a long time as a first-order open question.
    Figure {
        key: "dispersion/profile-districts",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Traditional districts in the FY2024 District Profile Report",
        pinned: 606.0,
        tolerance: 0.0,
        compute: |i| i.profile.len() as f64,
    },
    Figure {
        key: "dispersion/districts-at-the-twenty-mill-floor",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts whose effective Class I rate sits exactly on the twenty-mill floor, \
                TY2023",
        pinned: 170.0,
        tolerance: 0.0,
        compute: |i| {
            i.profile
                .iter()
                .filter(|d| d.at_twenty_mill_floor())
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/share-at-the-twenty-mill-floor",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The same, as a share of the districts reporting a rate",
        pinned: 0.2805,
        tolerance: 0.0001,
        compute: |i| {
            let at = i
                .profile
                .iter()
                .filter(|d| d.at_twenty_mill_floor())
                .count();
            at as f64 / i.profile.len() as f64
        },
    },
    Figure {
        // `< 20.5` and not `|m - 20| < 0.5`, which is what the prose's "within half a mill" reads
        // as and gives 60 rather than 63: three of the twenty districts *below* the floor sit
        // more than half a mill under it. The count the node states is this one, so this is what
        // is exported — the looser sentence is the thing that would have to move, not the figure.
        key: "dispersion/districts-at-or-against-the-twenty-mill-floor",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts whose effective Class I rate is below 20.5 mills — at the floor or \
                close enough that reduction factors barely operate",
        pinned: 233.0,
        tolerance: 0.0,
        compute: |i| {
            i.profile
                .iter()
                .filter(|d| d.effective_class1_millage.is_some_and(|m| m < 20.5))
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/districts-below-the-twenty-mill-floor",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts reporting an effective Class I rate below twenty mills, TY2023",
        pinned: 20.0,
        tolerance: 0.0,
        compute: |i| {
            i.profile
                .iter()
                .filter(|d| d.below_twenty_mill_floor())
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/districts-that-never-voted-twenty-mills",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts below the floor because their voters never approved twenty mills — the \
                condition `millage`'s own guard encodes",
        pinned: 6.0,
        tolerance: 0.0,
        compute: |i| {
            i.profile
                .iter()
                .filter(|d| d.below_twenty_mill_floor() && d.never_voted_twenty_mills())
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/districts-just-below-the-floor-unexplained",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts that voted well above twenty mills and still report a Class I rate just \
                under it — the anomaly the corpus's model of the floor does not explain",
        pinned: 14.0,
        tolerance: 0.0,
        compute: |i| {
            i.profile
                .iter()
                .filter(|d| d.below_twenty_mill_floor() && !d.never_voted_twenty_mills())
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/median-effective-class1-millage",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The statewide median effective Class I operating rate, in mills, TY2023",
        pinned: 23.40,
        tolerance: 0.005,
        compute: |i| profile_median(i, |d| d.effective_class1_millage),
    },
    Figure {
        key: "dispersion/median-voted-operating-millage",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The statewide median voted current operating rate, in mills, TY2023",
        pinned: 42.31,
        tolerance: 0.005,
        compute: |i| profile_median(i, |d| d.current_operating_millage),
    },
    Figure {
        key: "dispersion/maximum-effective-class1-millage",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The highest effective Class I operating rate in the state, in mills, TY2023",
        pinned: 84.29,
        tolerance: 0.005,
        compute: |i| {
            dispersion::profile::column(&i.profile, |d| d.effective_class1_millage)
                .into_iter()
                .fold(f64::MIN, f64::max)
        },
    },
    Figure {
        key: "dispersion/median-millage-reduction",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "How much of its voted millage the median district has lost to H.B. 920 reduction \
                factors — the median of the per-district ratio",
        pinned: 0.4245,
        tolerance: 0.0001,
        compute: |i| {
            let r = millage_reductions(i);
            r[r.len() / 2]
        },
    },
    Figure {
        key: "dispersion/largest-millage-reduction",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The largest reduction in the state, as a share of voted millage",
        pinned: 0.7467,
        tolerance: 0.0001,
        compute: |i| {
            *millage_reductions(i)
                .last()
                .expect("districts report a reduction")
        },
    },
    Figure {
        key: "dispersion/sd1-districts",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts in one tax year of Table SD-1 — a different population from the \
                profile report's 606 and from the funding model's 609",
        pinned: 611.0,
        tolerance: 0.0,
        compute: |i| i.sd1.iter().filter(|r| r.tax_year == 2024).count() as f64,
    },
    Figure {
        key: "dispersion/districts-at-twenty-mills-on-class-one-ty2024",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts at exactly twenty mills on the Class I rate, TY2024 Table SD-1",
        pinned: 155.0,
        tolerance: 0.0,
        compute: |i| sd1_at_twenty(i, 2024, |r| r.class1_rate) as f64,
    },
    // `parameter/twenty-mill-floor`. R.C. 319.301(E)(2) measures the floor on a district's own
    // current-expense taxes *combined with* the pre-1982 joint vocational taxes, and the corpus's
    // model carries only the first term. Read backwards from the fourteen districts sitting just
    // under the floor, the omitted term implies 1981 joint vocational rates a hair above the two
    // mills the same section names — which is what says the term is not the wrong size.
    Figure {
        key: "dispersion/implied-1981-joint-vocational-rate-low",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The smallest TY1981 joint vocational current-expense rate, in mills, implied by \
                a district sitting under the twenty-mill floor having voted past it",
        pinned: 2.006,
        tolerance: 0.0005,
        compute: |i| implied_1981_rates(i).0,
    },
    Figure {
        key: "dispersion/implied-1981-joint-vocational-rate-high",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The largest of the same \u{2014} Bradford Exempted Village, the district a \
                rounding account of the shortfalls cannot reach",
        pinned: 2.278,
        tolerance: 0.0005,
        compute: |i| implied_1981_rates(i).1,
    },
    // And the term the two figures above size is disconfirmed. It belongs to the joint vocational
    // district rather than to the member, so it binds every member alike; these are the counts and
    // rates that show it does not.
    // Ohio's joint vocational sector, which nothing here could find until the directory's agency
    // type was read for what it actually holds.
    // The relief cliff, read against the fund the five-year forecasts report. The first two are
    // the control: the grant never passed through the general fund, which is what makes the
    // absence of a signal after it ended mean something.
    Figure {
        key: "project/relief-federal-revenue-fy2019",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Federal revenue to the 607 districts the relief join reaches, FY2019 \u{2014} the \
                last year before the pandemic grants",
        pinned: 1_579_183_000.0,
        tolerance: 1.0,
        compute: |_| relief_aggregate(project::esser::BASELINE_YEAR, |y| y.federal_revenue),
    },
    Figure {
        key: "project/relief-federal-revenue-at-the-peak",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The same at the FY2022 peak \u{2014} a rise of 139% in three years",
        pinned: 3_775_608_000.0,
        tolerance: 1.0,
        compute: |_| relief_aggregate(2022, |y| y.federal_revenue),
    },
    Figure {
        key: "project/general-fund-revenue-at-the-relief-peak",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Their general fund revenue in the same year \u{2014} below what it was in FY2020, \
                which is how the relief money is known never to have entered it",
        pinned: 20_564_598_850.0,
        tolerance: 1.0,
        compute: |_| relief_aggregate(2022, |y| y.general_fund_revenue),
    },
    Figure {
        key: "project/relief-exposure-against-later-spending",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "Correlation between a district's relief per pupil and its general fund spending \
                growth after the cliff \u{2014} the recurring-cost reading's prediction, tested",
        pinned: 0.0274,
        tolerance: 0.000_05,
        compute: |_| project::esser::exposure_against_spending_growth(),
    },
    Figure {
        key: "project/relief-cliff-districts",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts the Census panel and the five-year forecasts both reach in every year \
                the comparison needs",
        pinned: 607.0,
        tolerance: 0.0,
        compute: |_| project::esser::districts().len() as f64,
    },
    Figure {
        key: "project/districts-spending-less-after-the-cliff",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Of those, the ones spending less from the general fund in FY2025 than FY2023 \
                \u{2014} spread across every exposure quartile, not concentrated in the exposed one",
        pinned: 27.0,
        tolerance: 0.0,
        compute: |_| {
            project::esser::districts()
                .iter()
                .filter(|d| d.spending_growth < 0.0)
                .count() as f64
        },
    },
    Figure {
        key: "dispersion/regional-service-agencies",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Ohio agencies the federal directory types as regional education service agencies \
                \u{2014} joint vocational districts and educational service centres together",
        pinned: 100.0,
        tolerance: 0.0,
        compute: |_| dispersion::lea_directory::service_agencies().len() as f64,
    },
    Figure {
        key: "dispersion/joint-vocational-districts",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Of those, the joint vocational school districts \u{2014} the same 49 whether they \
                are picked out by name or by whether they levy property tax",
        pinned: 49.0,
        tolerance: 0.0,
        compute: |_| dispersion::lea_directory::joint_vocational_districts().len() as f64,
    },
    Figure {
        key: "dispersion/joint-vocational-rows-outside-the-comparable-set",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Joint vocational rows in the F-33 Ohio panel, every one of them marked outside \
                the comparable set \u{2014} the peer-grouping question, already answered",
        pinned: 121.0,
        tolerance: 0.0,
        compute: |_| joint_vocational_panel_rows() as f64,
    },
    Figure {
        key: "dispersion/eastland-fairfield-enrolment-fy2023",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Eastland-Fairfield's enrolment on the Census count, FY2023 \u{2014} a head count \
                and not a full-time equivalent, which is what makes a ratio off it a trap",
        pinned: 1_160.0,
        tolerance: 0.0,
        compute: |_| eastland_fairfield(2023).map_or(f64::NAN, |row| row.enrollment),
    },
    Figure {
        key: "dispersion/eastland-fairfield-spending-per-pupil-fy2024",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Its current spending over that count, FY2024 \u{2014} the double count the node's \
                own description warns about, in one number",
        pinned: 20_869.0,
        tolerance: 1.0,
        compute: |_| {
            eastland_fairfield(2024)
                .and_then(|row| Some(row.current_spending? / row.enrollment))
                .unwrap_or(f64::NAN)
        },
    },
    Figure {
        key: "dispersion/comparable-median-spending-per-pupil-fy2024",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The median of the comparable districts it is set against, FY2024",
        pinned: 15_426.0,
        tolerance: 1.0,
        compute: |_| comparable_median_spending_per_pupil(2024),
    },
    Figure {
        key: "project/eastland-fairfield-property-tax-fy2025",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Eastland-Fairfield's property tax in FY2025, the last closed year of the \
                Auditor's finances \u{2014} the funding series the node recorded as unpopulated",
        pinned: 25_891_768.0,
        tolerance: 1.0,
        compute: |_| eastland_fairfield_finances(|y| y.property_tax),
    },
    Figure {
        key: "project/eastland-fairfield-state-aid-fy2025",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Its unrestricted state aid in the same year, which more than doubled over the six \
                while the levy rose by three fifths",
        pinned: 12_088_067.0,
        tolerance: 1.0,
        compute: |_| eastland_fairfield_finances(|y| y.unrestricted_aid),
    },
    Figure {
        key: "project/joint-vocational-districts-named-for-one-county",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Joint vocational districts whose own name contains exactly one Ohio county, which \
                is the only candidate roster a name gives you",
        pinned: 24.0,
        tolerance: 0.0,
        compute: |_| project::joint_vocational::county_named().len() as f64,
    },
    Figure {
        key: "project/joint-vocational-rosters-reconciling-with-their-county",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Of those, the ones whose levies reconcile against the district's own books \
                \u{2014} the county is their whole membership and for the other twelve it is not",
        pinned: 12.0,
        tolerance: 0.0,
        compute: |_| project::joint_vocational::reconciling().len() as f64,
    },
    Figure {
        key: "dispersion/districts-in-a-joint-vocational-district",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts carrying a joint vocational operating levy in Table SD-1, TY2024 \u{2014} \
                the membership the abstract never names and the column pair settles",
        pinned: 501.0,
        tolerance: 0.0,
        compute: |i| joint_vocational_membership(i, 2024).0 as f64,
    },
    Figure {
        key: "dispersion/districts-in-no-joint-vocational-district",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts carrying none, TY2024 \u{2014} for which R.C. 319.301(E)(1) has no \
                referent at all",
        pinned: 110.0,
        tolerance: 0.0,
        compute: |i| joint_vocational_membership(i, 2024).1 as f64,
    },
    Figure {
        key: "dispersion/shelby-joint-vocational-class1-mills",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The Class I rate, in mills, that Shelby County's eight districts jointly fit in \
                TY2024 \u{2014} two numbers explaining eight levies, which is co-membership",
        pinned: 2.672_22,
        tolerance: 0.000_005,
        compute: |i| {
            county_joint_vocational_fit(i, "Shelby", 2024).map_or(f64::NAN, |fit| fit.class1_mills)
        },
    },
    Figure {
        key: "dispersion/shelby-joint-vocational-class2-mills",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "The Class II rate of the same fit, in mills \u{2014} both far enough above the \
                two-mill floor that the fit is not the trivial one",
        pinned: 4.693_03,
        tolerance: 0.000_005,
        compute: |i| {
            county_joint_vocational_fit(i, "Shelby", 2024).map_or(f64::NAN, |fit| fit.class2_mills)
        },
    },
    Figure {
        key: "dispersion/botkins-floor-shortfall-mills",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How far Botkins Local sits below the twenty-mill floor, TY2024 \u{2014} in the \
                same joint vocational district as Anna Local, which sits on it",
        pinned: 0.0705,
        tolerance: 0.000_05,
        compute: |i| floor_shortfall(i, "049767", 2024),
    },
    Figure {
        key: "dispersion/mccomb-floor-shortfall-mills",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How far McComb Local sits below it \u{2014} the second largest shortfall of the \
                fourteen, in a district that is part of no joint vocational district",
        pinned: 0.1076,
        tolerance: 0.000_05,
        compute: |i| floor_shortfall(i, "047456", 2024),
    },
    Figure {
        key: "dispersion/districts-below-the-floor-in-every-tax-year",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts below the twenty-mill floor on the computed Class I rate in all four \
                tax years \u{2014} a shortfall that survives four annual recomputations is not a \
                rounding residual",
        pinned: 17.0,
        tolerance: 0.0,
        compute: |i| below_the_floor_in_every_tax_year(i) as f64,
    },
    Figure {
        key: "dispersion/districts-at-twenty-mills-on-the-combined-base-ty2024",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Districts at exactly twenty mills on the value-weighted real property rate, \
                TY2024 — the combined-base hypothesis, disconfirmed",
        pinned: 62.0,
        tolerance: 0.0,
        compute: |i| sd1_at_twenty(i, 2024, |r| r.real_property_millage) as f64,
    },
    // `formula-component/fsfp-enrolment-supplements`, `fsfp-performance-supplement` and
    // `fsfp-gifted-units`. All three compute through `project::panel`'s public API and always
    // did — what was missing was the binding, which is what #158 is about rather than #157.
    Figure {
        key: "project/base-funding-supplement-total",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The base funding supplement statewide — $40 a pupil, every district, no test",
        pinned: 56_079_575.0,
        tolerance: 1.0,
        compute: |i| i.panel.iter().map(|r| r.supplements.base_funding).sum(),
    },
    Figure {
        key: "project/base-funding-supplement-per-district",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The same, per district",
        pinned: 92_084.69,
        tolerance: 0.01,
        compute: |i| {
            i.panel
                .iter()
                .map(|r| r.supplements.base_funding)
                .sum::<f64>()
                / i.panel.len() as f64
        },
    },
    Figure {
        key: "project/enrolment-growth-supplement-total",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The enrolment growth supplement statewide — $250 a pupil on the whole roll, for \
                a district whose enrolment rose 3% over three years",
        pinned: 39_379_553.0,
        tolerance: 1.0,
        compute: |i| i.panel.iter().map(|r| r.supplements.growth).sum(),
    },
    Figure {
        key: "project/enrolment-growth-supplement-districts",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts that cleared the three-per-cent cliff",
        pinned: 43.0,
        tolerance: 0.0,
        compute: |i| {
            i.panel
                .iter()
                .filter(|r| r.supplements.growth_eligible)
                .count() as f64
        },
    },
    Figure {
        key: "project/enrolment-growth-supplement-per-district",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The same, per district that draws it — the comparison that makes the two \
                supplements different in kind rather than in size",
        pinned: 915_803.56,
        tolerance: 0.01,
        compute: |i| {
            let total: f64 = i.panel.iter().map(|r| r.supplements.growth).sum();
            let n = i
                .panel
                .iter()
                .filter(|r| r.supplements.growth_eligible)
                .count();
            total / n as f64
        },
    },
    Figure {
        // A magnitude, with the direction in the key: Ohio's median district is shrinking.
        key: "project/median-three-year-enrolment-decline",
        owner: "crates/project",
        unit: Unit::Share,
        label: "How far the median district's enrolment fell over the three years the growth \
                supplement measures",
        pinned: 0.048417,
        tolerance: 0.00001,
        compute: |i| {
            let mut changes: Vec<f64> = i
                .panel
                .iter()
                .map(|r| r.supplements.enrollment_change)
                .collect();
            changes.sort_by(f64::total_cmp);
            -changes[changes.len() / 2]
        },
    },
    Figure {
        key: "project/largest-forgone-growth-supplement",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What three hundredths of a percentage point cost the district that came closest \
                to the cliff and missed",
        pinned: 430_476.80,
        tolerance: 0.01,
        compute: |i| {
            just_below_the_growth_cliff(i)
                .first()
                .map_or(0.0, |(_, _, forgone)| *forgone)
        },
    },
    Figure {
        // *Besides* the nearest miss, which the node names and prices separately. Four districts
        // sit in the band; the sentence is about the other three, and a figure of four bound to
        // "three other districts" would be the check passing on a definition nobody stated.
        key: "project/other-districts-just-below-the-growth-cliff",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts between 2.7% and 3% enrolment growth besides the nearest miss — near \
                the cliff, and paid nothing for it",
        pinned: 3.0,
        tolerance: 0.0,
        compute: |i| just_below_the_growth_cliff(i).len().saturating_sub(1) as f64,
    },
    Figure {
        key: "project/performance-supplement-total",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The performance supplement statewide",
        pinned: 55_676_980.0,
        tolerance: 1.0,
        compute: |i| i.panel.iter().map(|r| r.performance.amount).sum(),
    },
    // `intervention/academic-distress-commission` and the performance supplement's own node.
    // The supplement reaches districts the same report card put under state control,
    // which is checkable only because the FY2027 model carries the ratings and the payment on one
    // row. Bound in both nodes that state it.
    Figure {
        key: "project/performance-supplement-to-youngstown",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the FY2027 model pays the one district still under an academic distress \
                commission",
        pinned: 162_207.67,
        tolerance: 0.01,
        compute: |i| {
            i.panel
                .iter()
                .find(|r| r.irn == "045161")
                .map_or(0.0, |r| r.performance.amount)
        },
    },
    Figure {
        key: "project/districts-below-the-release-threshold",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts rated below the three stars R.C. 3302.10(N)(1) requires to begin a \
                transition out of an academic distress commission",
        pinned: 59.0,
        tolerance: 0.0,
        compute: |i| below_the_release_threshold(i).0 as f64,
    },
    Figure {
        key: "project/districts-below-the-release-threshold-paid",
        owner: "crates/project",
        unit: Unit::Count,
        label: "How many of those are paid a performance supplement anyway \u{2014} none of them \
                through the star route",
        pinned: 15.0,
        tolerance: 0.0,
        compute: |i| below_the_release_threshold(i).1 as f64,
    },
    Figure {
        key: "project/performance-supplement-below-the-release-threshold",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What those districts receive between them",
        pinned: 3_036_977.90,
        tolerance: 0.01,
        compute: |i| below_the_release_threshold(i).2,
    },
    Figure {
        key: "project/performance-supplement-least-poor-quintile",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Mean performance supplement per pupil in the least-poor quintile",
        pinned: 55.92,
        tolerance: 0.005,
        compute: |i| performance_quintiles(i)[0].0,
    },
    Figure {
        key: "project/performance-supplement-second-quintile",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The same, second quintile",
        pinned: 43.51,
        tolerance: 0.005,
        compute: |i| performance_quintiles(i)[1].0,
    },
    Figure {
        key: "project/performance-supplement-third-quintile",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The same, third quintile",
        pinned: 35.62,
        tolerance: 0.005,
        compute: |i| performance_quintiles(i)[2].0,
    },
    Figure {
        key: "project/performance-supplement-fourth-quintile",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The same, fourth quintile",
        pinned: 30.62,
        tolerance: 0.005,
        compute: |i| performance_quintiles(i)[3].0,
    },
    Figure {
        key: "project/performance-supplement-poorest-quintile",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The same, poorest quintile — the other end of a gradient that runs against need",
        pinned: 21.84,
        tolerance: 0.005,
        compute: |i| performance_quintiles(i)[4].0,
    },
    Figure {
        key: "project/performance-supplement-gradient",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "How many times more per pupil the least-poor quintile receives than the poorest",
        pinned: 2.5605,
        tolerance: 0.0005,
        compute: |i| {
            let q = performance_quintiles(i);
            q[0].0 / q[4].0
        },
    },
    Figure {
        key: "project/performance-supplement-qualifying-least-poor",
        owner: "crates/project",
        unit: Unit::Share,
        label: "Share of the least-poor quintile that qualifies for the supplement at all",
        pinned: 0.9421,
        tolerance: 0.0001,
        compute: |i| performance_quintiles(i)[0].1,
    },
    Figure {
        key: "project/performance-supplement-qualifying-poorest",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The same for the poorest quintile",
        pinned: 0.4720,
        tolerance: 0.0001,
        compute: |i| performance_quintiles(i)[4].1,
    },
    Figure {
        key: "project/gifted-identification-falls-with-poverty",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "How strongly the share of pupils a district identifies as gifted falls as its \
                disadvantaged share rises — the magnitude of a negative correlation",
        pinned: 0.6726,
        tolerance: 0.0005,
        compute: |i| {
            let (x, y): (Vec<f64>, Vec<f64>) = sizeable(i)
                .iter()
                .map(|r| (r.dpia.percentage, identified_share(r)))
                .unzip();
            dispersion::wealth_neutrality(&x, &y)
                .expect("a paired series")
                .correlation
                .abs()
        },
    },
    Figure {
        key: "project/gifted-identification-rises-with-property-wealth",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The same against valuation per pupil — real, monotone, and half the strength of \
                the poverty gradient",
        pinned: 0.3483,
        tolerance: 0.0005,
        compute: |i| {
            let (x, y): (Vec<f64>, Vec<f64>) = sizeable(i)
                .iter()
                .filter_map(|r| Some((r.valuation_per_pupil?, identified_share(r))))
                .unzip();
            dispersion::wealth_neutrality(&x, &y)
                .expect("a paired series")
                .correlation
        },
    },
    // `formula-component/fsfp-disadvantaged-pupil-impact-aid`. The two counts DPIA blends, summed
    // over the panel, because the node's claim about their *vintages* rests on their sizes: the
    // greenbook publishes both statewide for FY2025, the term the workbook labels FY25 lands on
    // its figure and the term it labels FY26 lands 15% away from it. See #174.
    //
    // Rounded to a whole pupil inside `compute`. An ADM is a full-time equivalent and sums to a
    // fraction; a `Count` is compared exactly, and the corpus quotes these as pupils.
    Figure {
        key: "project/dpia-economically-disadvantaged-adm-statewide",
        owner: "crates/project",
        unit: Unit::Count,
        label: "`d1a` summed over the panel — the FY2025 economically disadvantaged ADM the \
                FY2027 model is still funding on",
        pinned: 856_236.0,
        tolerance: 0.0,
        compute: |i| {
            i.panel
                .iter()
                .map(|r| r.dpia.economically_disadvantaged_adm)
                .sum::<f64>()
                .round()
        },
    },
    Figure {
        key: "project/dpia-directly-certified-adm-statewide",
        owner: "crates/project",
        unit: Unit::Count,
        label: "`d1b` summed over the panel — the directly certified ADM, which the workbook \
                heads FY26 and the act would have on FY2027",
        pinned: 474_197.0,
        tolerance: 0.0,
        compute: |i| {
            i.panel
                .iter()
                .map(|r| r.dpia.directly_certified_adm)
                .sum::<f64>()
                .round()
        },
    },
    // `formula-component/temporary-transitional-aid-guarantee`. The finding is a negative and the
    // pair of figures is the whole of it: guarantee status looks like it predicts achievement and
    // stops doing so the moment poverty is held constant.
    Figure {
        key: "project/guarantee-predicts-achievement-raw",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "How strongly being on the guarantee predicts a district's Performance Index, \
                before any control",
        pinned: 0.18686,
        tolerance: 0.0005,
        compute: |i| {
            let (guarantee, index, _) = guarantee_against_achievement(i);
            dispersion::wealth_neutrality(&guarantee, &index)
                .expect("a paired series")
                .correlation
        },
    },
    Figure {
        key: "project/guarantee-predicts-achievement-holding-poverty",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The same, holding economic disadvantage constant — what is left of it, which is \
                nothing",
        pinned: 0.03457,
        tolerance: 0.0005,
        compute: |i| {
            let (guarantee, index, poverty) = guarantee_against_achievement(i);
            let pair = |x: &[f64], y: &[f64]| {
                dispersion::wealth_neutrality(x, y)
                    .expect("a paired series")
                    .correlation
            };
            dispersion::partial_correlation(
                pair(&guarantee, &index),
                pair(&guarantee, &poverty),
                pair(&index, &poverty),
            )
            .expect("a defined partial")
        },
    },
    Figure {
        key: "project/median-performance-index-on-the-guarantee",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "Median Performance Index among districts the guarantee holds up",
        // 89.85 and not the 89.9 two places carried: `dispersion::median` averages the two middle
        // observations of an even series and the figure was taken as the upper of them, which is
        // the same local-median defect this workspace corrected in three other files.
        pinned: 89.85,
        tolerance: 0.005,
        compute: |i| median_index(i, true),
    },
    Figure {
        key: "project/median-performance-index-on-the-formula",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "Median Performance Index among districts the formula funds — the raw gap the \
                control above dissolves",
        pinned: 85.6,
        tolerance: 0.005,
        compute: |i| median_index(i, false),
    },
    Figure {
        key: "project/model-districts",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts in the department's FY2027 funding model",
        pinned: 609.0,
        tolerance: 0.0,
        compute: |i| i.panel.len() as f64,
    },
    Figure {
        key: "project/districts-at-the-minimum-state-share",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts whose state share is the 10% floor rather than the computed share",
        pinned: 138.0,
        tolerance: 0.0,
        compute: |i| {
            i.panel
                .iter()
                .filter(|r| r.at_minimum_state_share())
                .count() as f64
        },
    },
    Figure {
        key: "project/open-enrolment-clawback-districts",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts whose guarantee is reduced by the open-enrolment clawback, FY2027",
        pinned: 43.0,
        tolerance: 0.0,
        compute: |i| clawback(&i.panel).0 as f64,
    },
    Figure {
        key: "project/open-enrolment-clawback-withheld",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Total withheld by the open-enrolment clawback, FY2027",
        pinned: 5_100_000.0,
        tolerance: 50_000.0,
        compute: |i| clawback(&i.panel).1,
    },
    Figure {
        key: "project/open-enrolment-clawback-largest",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The largest single open-enrolment clawback, FY2027 — Columbus City",
        pinned: 674_561.0,
        tolerance: 0.5,
        compute: |i| clawback(&i.panel).2,
    },
    Figure {
        key: "project/open-enrolment-clawback-second-largest",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The second largest open-enrolment clawback, FY2027 — Cuyahoga Falls",
        pinned: 640_025.0,
        tolerance: 0.5,
        compute: |i| clawback(&i.panel).3,
    },
    Figure {
        key: "project/statewide-average-base-cost-per-pupil",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "FY2027 statewide average base cost per pupil, the rate the clawback is charged at",
        pinned: 8_241.61,
        tolerance: 0.005,
        compute: |_| project::panel::AVERAGE_BASE_COST_PER_PUPIL,
    },
    Figure {
        key: "project/targeted-assistance-median-weighted-wealth",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "FY2027 statewide median weighted wealth, the targeted assistance capacity median",
        pinned: 392_151_306.63,
        tolerance: 0.005,
        compute: |_| project::panel::TA_MEDIAN_WEIGHTED_WEALTH,
    },
    Figure {
        key: "project/targeted-assistance-median-wealth-per-pupil",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "FY2027 statewide median weighted wealth per pupil, the wealth-tier median",
        pinned: 276_708.97,
        tolerance: 0.005,
        compute: |_| project::panel::TA_MEDIAN_WEALTH_PER_PUPIL,
    },
    Figure {
        key: "project/targeted-assistance-statewide-total",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "FY2027 targeted assistance, summed over the panel",
        pinned: 1_364_333_154.32,
        tolerance: 0.005,
        compute: |i| {
            i.panel
                .iter()
                .map(|r| r.categoricals.targeted_assistance)
                .sum()
        },
    },
    Figure {
        key: "project/targeted-assistance-wealth-millage",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The wealth tier's rate against the median district, as LSC states it: 14 mills",
        pinned: 14.0,
        tolerance: 0.0005,
        compute: |_| {
            mills_before(
                &plan_targeted_assistance(),
                "and the district\u{2019}s weighted wealth per pupil multiplied by 11.2 mills",
            )
        },
    },
    Figure {
        key: "project/targeted-assistance-district-millage",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The wealth tier's rate against the district's own wealth: 11.2 mills",
        pinned: 11.2,
        tolerance: 0.0005,
        compute: |_| {
            mills_before(
                &plan_targeted_assistance(),
                "This calculation is summarized below. Before any guarantees or phase-ins, the \
                 wealth amount",
            )
        },
    },
    Figure {
        key: "project/targeted-assistance-capacity-millage",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The capacity tier's rate against the gap to median weighted wealth: 8 mills",
        pinned: 8.0,
        tolerance: 0.0005,
        compute: |_| {
            mills_before(
                &plan_targeted_assistance(),
                "multiplied by the difference between the statewide median district\u{2019}s \
                 weighted wealth and the district\u{2019}s weighted wealth",
            )
        },
    },
    Figure {
        key: "project/first-targeted-assistance-millage",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The target millage of the FY2014 targeted assistance this one replaced: 6 mills",
        pinned: 6.0,
        tolerance: 0.0005,
        compute: |_| mills_before(&first_targeted_assistance(), "in each fiscal year. As a result"),
    },
    Figure {
        key: "project/capacity-aid-multiplier-fy2016",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "Capacity aid's multiplier in FY2016, the first of the three it had",
        pinned: 2.75,
        tolerance: 0.0005,
        compute: |_| amount_after(&capacity_aid("hb64"), "Capacity aid multiplier = "),
    },
    Figure {
        key: "project/capacity-aid-multiplier-fy2017",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "Capacity aid's multiplier in FY2017",
        pinned: 3.5,
        tolerance: 0.0005,
        compute: |_| {
            amount_after(
                &capacity_aid("hb49"),
                "computing capacity aid funds from ",
            )
        },
    },
    Figure {
        key: "project/capacity-aid-multiplier-fy2018",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "Capacity aid's multiplier in FY2018 and FY2019, the last of the three",
        pinned: 4.0,
        tolerance: 0.0005,
        compute: |_| {
            amount_after(
                &capacity_aid("hb49"),
                "capacity aid funds from 3.5 in FY 2017 to ",
            )
        },
    },
    Figure {
        key: "project/targeted-assistance-fy2022-estimate",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Targeted assistance in FY2022 as LSC estimated it, before guarantees or phase-ins",
        pinned: 988_400_000.0,
        tolerance: 0.005,
        compute: |_| {
            amount_after(
                &plan_targeted_assistance(),
                "targeted assistance is estimated to be ",
            ) * 1_000_000.0
        },
    },
    Figure {
        key: "project/targeted-assistance-growth-since-the-plan",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The growth in targeted assistance from FY2022 to FY2027, with no rate changed",
        pinned: 0.380_345_158_154_592_9,
        tolerance: 0.000_005,
        compute: |i| {
            let fy2027: f64 = i
                .panel
                .iter()
                .map(|r| r.categoricals.targeted_assistance)
                .sum();
            let fy2022 = amount_after(
                &plan_targeted_assistance(),
                "targeted assistance is estimated to be ",
            ) * 1_000_000.0;
            fy2027 / fy2022 - 1.0
        },
    },
    Figure {
        key: "project/special-education-weight-rescaling",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "What the plan multiplied the 2009 special education weights by, all six alike",
        pinned: 0.837_925_779_893_096_6,
        tolerance: 0.000_005,
        compute: |_| {
            let then = evidence_based_weights();
            project::panel::SPECIAL_EDUCATION_WEIGHTS
                .iter()
                .zip(then)
                .map(|(now, was)| now / was)
                .sum::<f64>()
                / 6.0
        },
    },
    Figure {
        key: "project/special-education-raise-at-the-plan",
        owner: "crates/project",
        unit: Unit::Share,
        label: "What the plan raised every special education category by, over the FY2017 amounts",
        pinned: 0.111_183_608_345_807_1,
        tolerance: 0.000_005,
        compute: |_| {
            let base = base_cost_fy2022();
            let before = implied_divisor(
                &special_education_amounts_fy2017(),
                &evidence_based_weights(),
            );
            let rescaling = {
                let then = evidence_based_weights();
                project::panel::SPECIAL_EDUCATION_WEIGHTS
                    .iter()
                    .zip(then)
                    .map(|(now, was)| now / was)
                    .sum::<f64>()
                    / 6.0
            };
            rescaling * base / before - 1.0
        },
    },
    Figure {
        key: "project/english-learner-amount-carried-into-the-plan",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The FY2015 English learner category 1 amount the plan's first weight re-expresses",
        pinned: 1_515.0,
        tolerance: 0.005,
        compute: |_| {
            dollars_after(
                &project::greenbook::greenbook("hb59").flat(),
                "1 LEP students in U.S. schools for no more than 180 days and not",
                2,
            )[1]
        },
    },
    Figure {
        key: "project/career-technical-base-cost-implied-fy2022",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The FY2022 career-technical base cost the plan's six weights imply, never stated",
        pinned: 8_333.115_691_651_989,
        tolerance: 0.005,
        compute: |_| {
            let weights: Vec<f64> = project::panel::CTE_WEIGHTS
                .iter()
                .copied()
                .chain(std::iter::once(project::panel::CTE_ASSOCIATED_WEIGHT))
                .collect();
            implied_divisor(&career_technical_amounts_fy2017(), &weights)
        },
    },
    Figure {
        key: "project/growth-supplement-appropriation-fy2021",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "ALI 200636 in FY2021, the last year the enrollment growth supplement was paid",
        pinned: 23_000_000.0,
        tolerance: 0.005,
        compute: |_| enacted_line("200636", 2021),
    },
    Figure {
        key: "project/growth-supplement-appropriation-fy2022",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "ALI 200636 in FY2022, when the plan cut the supplement to nothing",
        pinned: 0.0,
        tolerance: 0.005,
        compute: |_| enacted_line("200636", 2022),
    },
    Figure {
        key: "project/base-funding-supplements-fy2006",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The four FY2006 base funding supplements summed, which is the amount paid flat now",
        pinned: 40.0,
        tolerance: 0.005,
        compute: |_| base_funding_supplements_fy2006(),
    },
    Figure {
        key: "project/outcome-bonus-share-of-the-formula-amount",
        owner: "crates/project",
        unit: Unit::Share,
        label: "What the FY2016-19 graduation and reading bonuses paid, as a share of base cost",
        pinned: 0.075,
        tolerance: 0.000_005,
        compute: |_| {
            let lsc = project::greenbook::greenbook("hb64").flat();
            assert!(lsc.contains(
                "Graduation bonus = Graduation rate x 0.075 x Formula amount x Graduate count x \
                 State share index"
            ));
            0.075
        },
    },
    Figure {
        key: "dispersion/panel-real-per-pupil-growth-fy2010-fy2024",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Real spending per pupil on the comparable district panel, FY2010 to FY2024",
        pinned: 0.088_586_501_832_213_17,
        tolerance: 0.000_005,
        compute: |_| {
            let series = dispersion::ohio_panel::spending_by_year();
            series[&2024].real_per_pupil / series[&2010].real_per_pupil - 1.0
        },
    },
    Figure {
        key: "dispersion/panel-real-total-fy2024-over-fy2010",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Real total spending on the same panel in FY2024 as a multiple of FY2010's",
        pinned: 0.988_911_040_817_362_5,
        tolerance: 0.000_005,
        compute: |_| {
            let series = dispersion::ohio_panel::spending_by_year();
            series[&2024].real_total / series[&2010].real_total
        },
    },
    Figure {
        key: "dispersion/panel-enrolment-decline-fy2010-fy2024",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Fall membership lost on the same panel over the same years, which is the difference",
        pinned: 0.091_564_116_261_855_12,
        tolerance: 0.000_005,
        compute: |_| {
            let series = dispersion::ohio_panel::spending_by_year();
            1.0 - series[&2024].enrollment / series[&2010].enrollment
        },
    },
    Figure {
        key: "dispersion/panel-real-total-fall-fy2020-fy2021",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "The pandemic year's fall in real total district spending, which per pupil hides",
        pinned: 0.033_342_033_569_011_8,
        tolerance: 0.000_005,
        compute: |_| {
            let series = dispersion::ohio_panel::spending_by_year();
            1.0 - series[&2021].real_total / series[&2020].real_total
        },
    },
    Figure {
        key: "dispersion/panel-pupils-lost-fy2021",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Fall membership the comparable panel lost in one year, FY2020 to FY2021",
        pinned: 54_777.0,
        tolerance: 0.0,
        compute: |_| {
            let series = dispersion::ohio_panel::spending_by_year();
            series[&2020].enrollment - series[&2021].enrollment
        },
    },
    Figure {
        key: "project/parity-aid-millage-fy2002",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "Parity aid's millage in FY2002, the first ancestor of targeted assistance",
        pinned: 9.5,
        tolerance: 0.0005,
        compute: |_| {
            mills_before(
                &project::greenbook::greenbook("hb94").flat(),
                "(above the adequacy level) to the 80th percentile",
            )
        },
    },
    Figure {
        key: "project/parity-aid-districts-fy2002",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts eligible for parity aid in FY2002, of about 612",
        pinned: 492.0,
        tolerance: 0.0,
        compute: |_| {
            amount_after(
                &project::greenbook::greenbook("hb94").flat(),
                "Overall, about ",
            )
        },
    },
    Figure {
        key: "project/parity-aid-districts-fy2009",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts eligible for parity aid in FY2009, its last year as a district payment",
        pinned: 367.0,
        tolerance: 0.0,
        compute: |_| {
            amount_after(
                &project::greenbook::greenbook("hb119").flat(),
                "lowest wealth districts in FY 2008 and the ",
            )
        },
    },
    Figure {
        key: "project/the-threshold-targeted-assistance-inherited",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The 490th lowest district of the era's panel, which is parity aid's own threshold",
        pinned: 0.800_653_594_771_241_9,
        tolerance: 0.000_005,
        compute: |_| {
            let districts = dispersion::ohio_panel::spending_by_year()[&2013].districts;
            490.0 / districts as f64
        },
    },
    Figure {
        key: "project/transportation-proration-earmark-fy2014",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The earmark the transportation proration divides by, FY2014",
        pinned: 413_385_915.0,
        tolerance: 0.005,
        compute: |_| {
            amount_after(
                &project::greenbook::greenbook("hb59").flat(),
                "Prorated Transportation Aid $ ",
            )
        },
    },
    Figure {
        key: "project/transportation-supplement-earmark-fy2014",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The earmark for the payment that undoes the proration, FY2014",
        pinned: 25_300_000.0,
        tolerance: 0.005,
        compute: |_| {
            amount_after(
                &project::greenbook::greenbook("hb59").flat(),
                "Supplemental Transportation Aid $ ",
            )
        },
    },
    Figure {
        key: "project/transportation-proration-share-of-the-line",
        owner: "crates/project",
        unit: Unit::Share,
        label: "How much of ALI 200502 the proration's own denominator is, FY2014",
        pinned: 0.818_564_044_127_079_4,
        tolerance: 0.000_005,
        compute: |_| {
            let earmark = amount_after(
                &project::greenbook::greenbook("hb59").flat(),
                "Prorated Transportation Aid $ ",
            );
            let line = project::ledger::appropriations::lines()
                .into_iter()
                .find(|l| l.line_item == "200502" && l.fiscal_year == 2014 && l.kind == "enacted")
                .expect("the ledger carries FY2014 pupil transportation")
                .amount;
            earmark / line
        },
    },
    Figure {
        key: "project/sped-transportation-implied-allocation",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The statewide special education transportation allocation FY2027\u{2019}s \
                proration factor implies \u{2014} the enacted earmark divided by the factor the \
                calculator states",
        pinned: 212_348_136.16,
        tolerance: 0.01,
        compute: |_| {
            project::transport::special_education(2027)
                .expect("FY2027 is modelled")
                .implied_statewide()
        },
    },
    Figure {
        key: "project/sped-transportation-districts-allocation",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the 611 districts in the FY2027 calculator are allocated before proration \
                \u{2014} the part of that denominator the model can see",
        pinned: 199_061_038.65,
        tolerance: 0.01,
        compute: |_| {
            project::transport::special_education(2027)
                .expect("FY2027 is modelled")
                .districts
        },
    },
    Figure {
        key: "project/sped-transportation-outside-the-model",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The part of the FY2027 denominator no district in the calculator accounts for, \
                which the greenbook attributes to county DD boards and ESCs",
        pinned: 13_287_097.51,
        tolerance: 0.01,
        compute: |_| {
            project::transport::special_education(2027)
                .expect("FY2027 is modelled")
                .outside_the_model()
        },
    },
    Figure {
        key: "project/sped-transportation-outside-the-model-share",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The same residual as a share of the implied statewide allocation",
        pinned: 0.062_571_930_7,
        tolerance: 0.000_005,
        compute: |_| {
            let allocation =
                project::transport::special_education(2027).expect("FY2027 is modelled");
            allocation.outside_the_model() / allocation.implied_statewide()
        },
    },
    Figure {
        key: "project/sped-transportation-districts-only-factor",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The FY2027 proration factor a reader would get by dividing the earmark by the \
                districts the calculator holds, against the 0.91746 it states",
        pinned: 0.978_699_133,
        tolerance: 0.000_000_5,
        compute: |_| {
            project::transport::special_education(2027)
                .expect("FY2027 is modelled")
                .districts_only_factor()
        },
    },
    Figure {
        key: "project/sped-transportation-fy2026-ceiling",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What FY2026\u{2019}s factor of 1.0 leaves for everything outside the calculator \
                \u{2014} the earmark less what its districts are allocated",
        pinned: 11_778_159.68,
        tolerance: 0.01,
        compute: |_| {
            project::transport::special_education(2026)
                .expect("FY2026 is modelled")
                .headroom()
        },
    },
    Figure {
        key: "project/sped-transportation-cost-step",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the prior year\u{2019}s reported costs add to the special education \
                transportation allocation between FY2026 and FY2027",
        pinned: 26_808_906.80,
        tolerance: 0.01,
        compute: |_| {
            project::transport::movement()
                .expect("both years are modelled")
                .cost()
        },
    },
    Figure {
        key: "project/sped-transportation-state-share-fall",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What falling state share percentages take back off that allocation over the same \
                interval \u{2014} the one term of the three that moves against the proration",
        pinned: 4_207_654.26,
        tolerance: 0.01,
        compute: |_| {
            -project::transport::movement()
                .expect("both years are modelled")
                .state_share()
        },
    },
    Figure {
        key: "project/sped-transportation-floor-step",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the floor under the state share adds by stepping from 45.83% to 50%, the \
                term that puts the allocation past its earmark",
        pinned: 11_340_267.79,
        tolerance: 0.01,
        compute: |_| {
            project::transport::movement()
                .expect("both years are modelled")
                .floor()
        },
    },
    Figure {
        key: "project/sped-transportation-at-the-old-floor",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What would have been left of the FY2027 earmark if the floor had stayed at \
                45.83% \u{2014} against the $4.2m it is short at 50%",
        pinned: 7_100_095.14,
        tolerance: 0.01,
        compute: |inputs| {
            let _ = inputs;
            let allocation =
                project::transport::special_education(2027).expect("FY2027 is modelled");
            let movement = project::transport::movement().expect("both years are modelled");
            allocation.appropriation - movement.after_state_share
        },
    },
    Figure {
        key: "project/general-transportation-headroom-fy2026",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "How much of ALI 200502\u{2019}s unearmarked remainder the general transportation \
                formula left unspent in FY2026 \u{2014} the width of a proration factor of 1.0",
        pinned: 5_594_475.23,
        tolerance: 0.01,
        compute: |_| {
            let (paid, line) = project::transport::general_headroom(2026).expect("FY2026");
            line - paid
        },
    },
    Figure {
        key: "project/general-transportation-headroom-fy2027",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The same headroom a year later, on a remainder the legislature raised",
        pinned: 36_760_324.87,
        tolerance: 0.01,
        compute: |_| {
            let (paid, line) = project::transport::general_headroom(2027).expect("FY2027");
            line - paid
        },
    },
    Figure {
        key: "project/sped-transportation-floor-districts",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts whose FY2027 special education transportation is paid at the statutory \
                floor rather than at their own state share, of the 563 that report a cost",
        pinned: 411.0,
        tolerance: 0.0,
        compute: |_| {
            project::transport::special_education(2027)
                .expect("FY2027 is modelled")
                .at_the_floor as f64
        },
    },
    Figure {
        key: "regime-diff/walter-in-derolph-i",
        owner: "crates/regime-diff",
        unit: Unit::Count,
        label: "Times DeRolph I names Walter, in an opinion that never says overrule",
        pinned: 76.0,
        tolerance: 0.0,
        compute: |_| {
            let opinion = edfund_core::records::record(
                include_str!("../../regime-diff/fixtures/derolph-opinions.txt"),
                "derolph-i",
            );
            let flat = opinion.body.split_whitespace().collect::<Vec<_>>().join(" ");
            assert_eq!(flat.to_lowercase().matches("overrul").count(), 0);
            flat.matches("Walter").count() as f64
        },
    },
    Figure {
        key: "project/gifted-statewide-total",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "FY2027 gifted funding as the department pays it, summed over the panel",
        pinned: 54_445_264.51,
        tolerance: 0.005,
        // The published column, not `Gifted::total()`. The six parts reconstruct it to 19 cents
        // across 609 districts — the department's own rounding — so this is what it pays and the
        // parts are how it got there. See `gifted-funding-rates.yml`.
        compute: |i| i.panel.iter().map(|r| r.categoricals.gifted).sum(),
    },
    Figure {
        key: "project/gifted-coordinator-aid-statewide",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "FY2027 gifted coordinator units, what they pay statewide",
        pinned: 13_995_587.23,
        tolerance: 0.005,
        compute: |i| i.panel.iter().map(|r| r.gifted.coordinator_aid).sum(),
    },
    Figure {
        key: "project/gifted-specialist-k8-aid-statewide",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "FY2027 gifted K-8 intervention specialist units, what they pay statewide",
        pinned: 20_980_411.17,
        tolerance: 0.005,
        compute: |i| i.panel.iter().map(|r| r.gifted.specialist_k8_aid).sum(),
    },
    Figure {
        key: "project/gifted-specialist-9-12-aid-statewide",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "FY2027 gifted 9-12 intervention specialist units, what they pay statewide",
        pinned: 12_633_770.59,
        tolerance: 0.005,
        compute: |i| i.panel.iter().map(|r| r.gifted.specialist_9_12_aid).sum(),
    },
    // ---- crates/regime-diff ------------------------------------------------------------------
    Figure {
        key: "regime-diff/charge-off-districts-with-a-difference",
        owner: "crates/regime-diff",
        unit: Unit::Count,
        label: "Districts the charge-off counterfactual can value under both regimes",
        pinned: 606.0,
        tolerance: 0.0,
        compute: |i| with_a_total(&i.at_recognized) as f64,
    },
    Figure {
        key: "regime-diff/charge-off-districts-censored",
        owner: "crates/regime-diff",
        unit: Unit::Count,
        label: "Districts whose local capacity the minimum state share censors, so the component \
                row is absent while the total is not",
        pinned: 138.0,
        tolerance: 0.0,
        compute: |i| {
            i.at_recognized
                .iter()
                .filter(|d| d.components[0].successor.is_none())
                .count() as f64
        },
    },
    Figure {
        key: "regime-diff/charge-off-zeroes-base-cost-aid",
        owner: "crates/regime-diff",
        unit: Unit::Count,
        label: "Districts a 23-mill charge-off on recognized valuation would leave with no base \
                cost aid at all",
        pinned: 65.0,
        tolerance: 0.0,
        compute: |i| zeroed(&i.at_recognized) as f64,
    },
    Figure {
        key: "regime-diff/charge-off-zeroes-base-cost-aid-on-total-taxable",
        owner: "crates/regime-diff",
        unit: Unit::Count,
        label: "The same count on total taxable value — the base the corpus wrongly assumed, and \
                the measure of how much of that finding was the base",
        pinned: 81.0,
        tolerance: 0.0,
        compute: |i| zeroed(&i.at_total_taxable) as f64,
    },
    Figure {
        key: "regime-diff/districts-the-plan-pays-more",
        owner: "crates/regime-diff",
        unit: Unit::Count,
        label: "Districts better off under the Fair School Funding Plan than under the charge-off",
        pinned: 290.0,
        tolerance: 0.0,
        compute: |i| {
            i.at_recognized
                .iter()
                .filter(|d| d.total_difference().is_some_and(|t| t > 0.01))
                .count() as f64
        },
    },
    Figure {
        key: "regime-diff/districts-the-charge-off-pays-more",
        owner: "crates/regime-diff",
        unit: Unit::Count,
        label: "Districts better off under the charge-off than under the plan",
        pinned: 316.0,
        tolerance: 0.0,
        compute: |i| {
            i.at_recognized
                .iter()
                .filter(|d| d.total_difference().is_some_and(|t| t < -0.01))
                .count() as f64
        },
    },
    Figure {
        // Positive, and named for the direction it runs in, because that is how the three nodes
        // carrying it write it: "the median district is $45 per pupil worse off under the plan".
        // A figure exported as `-44.62` could not be bound to that sentence without the check
        // either ignoring the sign — which is the one thing #120 was about — or demanding prose
        // nobody would write.
        key: "regime-diff/median-shortfall-under-the-plan",
        owner: "crates/regime-diff",
        unit: Unit::Dollars,
        label: "How much less the median district receives per pupil under the plan than it would \
                have under a 23-mill charge-off on recognized valuation",
        pinned: 44.62,
        tolerance: 0.005,
        compute: |i| -median_difference(&i.at_recognized),
    },
    Figure {
        key: "regime-diff/recognized-valuation-deferred-share",
        owner: "crates/regime-diff",
        unit: Unit::Share,
        label: "How far recognized valuation sits below total taxable value statewide at TY2024",
        pinned: 0.082,
        tolerance: 0.00005,
        compute: |i| 1.0 - i.recognized_total / i.actual_total,
    },
    Figure {
        key: "regime-diff/recognized-valuation-deferred-value",
        owner: "crates/regime-diff",
        unit: Unit::Dollars,
        label: "The taxable value TY2024's reappraisals have not yet phased in, statewide",
        pinned: 34_500_000_000.0,
        tolerance: 50_000_000.0,
        compute: |i| i.actual_total - i.recognized_total,
    },
    Figure {
        key: "regime-diff/deferred-charge-off",
        owner: "crates/regime-diff",
        unit: Unit::Dollars,
        label: "The charge-off that deferral removes — deferred value at the terminal 23 mills",
        pinned: 793_000_000.0,
        tolerance: 500_000.0,
        compute: |i| (i.actual_total - i.recognized_total) * TERMINAL_MILLS / 1000.0,
    },
    Figure {
        key: "project/prek-sped-remainder-fy2027",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The FY2027 appropriation for preschool special education — the residual earmark of GRF \
                ALI 200540, and the limit its proration is set against",
        pinned: 153_976_832.0,
        tolerance: 0.0,
        compute: |_| enacted_200540(PRESCHOOL_REMAINDER),
    },
    Figure {
        key: "project/ali-200540-total-fy2027",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "GRF ALI 200540, Special Education Enhancements, as enacted for FY2027",
        pinned: 193_272_426.0,
        tolerance: 0.0,
        compute: |_| enacted_200540(ALI_200540_TOTAL),
    },
    Figure {
        key: "project/ali-200540-actual-fy2025",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The same line in FY2025, as an actual rather than an appropriation — the year the \
                calculator's stale limit came from",
        pinned: 195_160_040.0,
        tolerance: 0.0,
        compute: |_| budget_analysis::special_education_enhancements(Edition::Enacted, ALI_200540_TOTAL).prior,
    },
    Figure {
        key: "project/prek-sped-program-total-fy2027",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What preschool special education totals across the FY2027 model at the department's own \
                stated proration factor",
        pinned: 148_408_183.76,
        tolerance: 0.005,
        compute: prek_sped_program,
    },
    Figure {
        key: "project/prek-sped-headroom-fy2027",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "How far that total sits under the appropriation that governs it — the reason no proration \
                arises, against a calculator cell that says one does",
        pinned: 5_568_648.24,
        tolerance: 0.005,
        compute: |i| enacted_200540(PRESCHOOL_REMAINDER) - prek_sped_program(i),
    },
    Figure {
        key: "project/prek-sped-over-the-stale-limit",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And how far it sits over the $147,500,000 the calculator prints beside the factor, which \
                is the FY2025 estimate carried into an FY2027 sheet",
        pinned: 908_183.76,
        tolerance: 0.005,
        compute: |i| prek_sped_program(i) - project::panel::supplements::PREK_SPED_APPROPRIATION,
    },
    Figure {
        key: "project/enacted-foundation-aid-fy2026",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Total foundation aid appropriated for FY2026, as enacted — not the $11.15 billion the \
                redbook proposed, which two nodes published under [verified]",
        pinned: 11_230_057_557.0,
        tolerance: 0.0,
        compute: |_| budget_analysis::foundation_aid(Edition::Enacted, TOTAL_FOUNDATION_AID).first,
    },
    Figure {
        key: "project/enacted-lottery-line-fy2026",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Fund 7017 ALI 200612 inside that total, as enacted",
        pinned: 1_436_583_202.0,
        tolerance: 0.0,
        compute: |_| budget_analysis::foundation_aid(Edition::Enacted, LOTTERY_LINE).first,
    },
    Figure {
        key: "project/lottery-line-rose-at-enactment",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "How much the lottery's line rose between the bill as introduced and the bill as enacted",
        pinned: 97_638_202.0,
        tolerance: 0.0,
        compute: |_| budget_analysis::enactment_movement(LOTTERY_LINE),
    },
    Figure {
        key: "project/foundation-aid-rose-at-enactment",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "How much the whole foundation aid total rose over the same passage — less than the \
                lottery line alone did",
        pinned: 82_062_286.0,
        tolerance: 0.0,
        compute: |_| budget_analysis::enactment_movement(TOTAL_FOUNDATION_AID),
    },
    Figure {
        // Positive, with the direction in the key, on the convention
        // `regime-diff/median-shortfall-under-the-plan` set: prose writes "the other four lines
        // together fell $15,575,916", and the consumer's numeral reader does not read a sign.
        key: "project/foundation-lines-off-the-lottery-fell-at-enactment",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And how much the other four lines fell, which is the difference between those two: the \
                substitution argument as one subtraction",
        pinned: 15_575_916.0,
        tolerance: 0.0,
        compute: |_| -budget_analysis::enactment_movement_off_the_lottery_line(),
    },
    Figure {
        key: "project/foundation-appropriation-annual-movements",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Year-over-year movements in the foundation aid appropriation — the length of the series \
                every figure below is a summary of",
        pinned: 24.0,
        tolerance: 0.0,
        compute: |i| i.movements.len() as f64,
    },
    Figure {
        key: "project/foundation-appropriation-noise-floor-real",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The median absolute annual movement in foundation aid, in constant FY2025 dollars — the \
                floor below which a substitution cannot be read off the total at all",
        pinned: 235_900_000.0,
        tolerance: 100_000.0,
        compute: |_| project::appropriations::foundation_noise_floor(BASE),
    },
    Figure {
        key: "project/foundation-appropriation-mean-movement-real",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The mean of the same movements, which two years dominate and which is why the floor is \
                stated as a median",
        pinned: 349_300_000.0,
        tolerance: 100_000.0,
        compute: |i| { let m = movement_magnitudes(i); m.iter().sum::<f64>() / m.len() as f64 },
    },
    Figure {
        key: "project/foundation-appropriation-smallest-movement-real",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The quietest year in the series",
        pinned: 10_512_598.0,
        tolerance: 1.0,
        compute: |i| movement_magnitudes(i)[0],
    },
    Figure {
        key: "project/foundation-appropriation-largest-movement-real",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And the loudest — the spread is the other half of why inference from the total fails",
        pinned: 1_514_161_257.0,
        tolerance: 1.0,
        compute: |i| *movement_magnitudes(i).last().expect("a non-empty series"),
    },
    Figure {
        key: "project/foundation-appropriation-rose-fy2012-real",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "How much foundation aid moved in FY2012, the year the county student fund began \
                distributing casino money",
        pinned: 182_809_660.0,
        tolerance: 1.0,
        compute: |i| movement(i, 2012),
    },
    Figure {
        key: "project/foundation-appropriation-fell-fy2013-real",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And in FY2013 — both unremarkable among their neighbours, which is the null result",
        pinned: 100_531_931.0,
        tolerance: 1.0,
        compute: |i| movement(i, 2013),
    },
    // ── the parameters that do not index ──────────────────────────────────
    //
    // `project::indexation`. Two freeze vintages, three components, and one gradient. The
    // erosions are properties of the vintage alone, so one figure serves every FY2022 amount.
    Figure {
        key: "project/preschool-grant-real-fy2026",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The preschool special education flat grant in FY2026 dollars — $4,000 nominal, \
                unmoved since FY2014",
        pinned: 5_604.56,
        tolerance: 0.01,
        compute: |_| frozen("preschool flat grant").real().expect("frozen"),
    },
    Figure {
        key: "project/preschool-grant-erosion",
        owner: "crates/project",
        unit: Unit::Share,
        label: "What the preschool flat grant has lost in real terms across thirteen years of \
                nominal freeze",
        pinned: 0.2863,
        tolerance: 0.0001,
        compute: |_| frozen("preschool flat grant").erosion().expect("frozen"),
    },
    Figure {
        key: "project/dpia-per-pupil-real-fy2026",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Disadvantaged Pupil Impact Aid's $422 in FY2026 dollars",
        pinned: 475.61,
        tolerance: 0.01,
        compute: |_| frozen("DPIA per pupil").real().expect("frozen"),
    },
    Figure {
        key: "project/gifted-coordinator-unit-real-fy2026",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The gifted coordinator unit's legislated salary of $85,776 in FY2026 dollars",
        pinned: 96_672.30,
        tolerance: 0.01,
        compute: |_| frozen("gifted coordinator unit").real().expect("frozen"),
    },
    Figure {
        key: "project/erosion-since-fy2022",
        owner: "crates/project",
        unit: Unit::Share,
        label: "What every amount set in FY2022 and unmoved since has lost — DPIA, and all five \
                gifted figures",
        pinned: 0.1127,
        tolerance: 0.0001,
        compute: |_| frozen("DPIA per pupil").erosion().expect("frozen"),
    },
    Figure {
        key: "project/frozen-parameters-shortfall-computed",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the three frozen components would pay across the FY2027 model had they held \
                their real value, less what they pay — before the guarantee",
        pinned: 133_152_626.93,
        tolerance: 1.0,
        compute: |i| project::indexation::statewide(&i.panel).computed,
    },
    Figure {
        key: "project/frozen-parameters-shortfall-delivered",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And how much of it would reach districts once the guarantee is applied",
        pinned: 104_298_429.45,
        tolerance: 1.0,
        compute: |i| project::indexation::statewide(&i.panel).delivered,
    },
    Figure {
        key: "project/frozen-parameters-shortfall-absorbed",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The difference: what guarantee floors absorb, which is the part of the freeze a \
                held district would not feel if it were reversed",
        pinned: 28_854_197.47,
        tolerance: 1.0,
        compute: |i| {
            let statewide = project::indexation::statewide(&i.panel);
            statewide.computed - statewide.delivered
        },
    },
    Figure {
        key: "project/frozen-parameters-shortfall-dpia",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The DPIA share of the shortfall — half of it, on size rather than on erosion",
        pinned: 66_703_817.98,
        tolerance: 1.0,
        compute: |i| {
            project::indexation::by_component(&i.panel)
                [&project::indexation::Component::Dpia]
        },
    },
    Figure {
        key: "project/frozen-parameters-shortfall-preschool",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The preschool share — 45% of the total, and the part no guarantee floor can \
                absorb, because it sits outside [H] Foundation Funding",
        pinned: 59_532_514.24,
        tolerance: 1.0,
        compute: |i| {
            project::indexation::by_component(&i.panel)
                [&project::indexation::Component::PreschoolSpecialEducation]
        },
    },
    Figure {
        key: "project/frozen-parameters-shortfall-gifted",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And the gifted share, the smallest of the three",
        pinned: 6_916_294.71,
        tolerance: 1.0,
        compute: |i| {
            project::indexation::by_component(&i.panel)
                [&project::indexation::Component::Gifted]
        },
    },
    Figure {
        key: "project/frozen-parameters-poorest-quintile-per-pupil",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the freeze costs the poorest fifth of districts per pupil",
        pinned: 141.66,
        tolerance: 0.01,
        compute: |i| quintile_shortfall(&i.panel, 4),
    },
    Figure {
        key: "project/frozen-parameters-wealthiest-quintile-per-pupil",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "And the wealthiest fifth — a four-to-one gradient nothing in the statute chose",
        pinned: 34.69,
        tolerance: 0.01,
        compute: |i| quintile_shortfall(&i.panel, 0),
    },
    Figure {
        key: "dispersion/casino-distributed-fy2016",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The casino channel distributed to school districts in FY2016, the first complete \
                state fiscal year the per-district series reaches",
        pinned: 90_832_043.0,
        tolerance: 1.0,
        compute: |i| i.casino[&2016],
    },
    Figure {
        key: "dispersion/casino-distributed-fy2017",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The casino channel distributed to school districts in FY2017",
        pinned: 89_356_178.0,
        tolerance: 1.0,
        compute: |i| i.casino[&2017],
    },
    Figure {
        key: "dispersion/casino-distributed-fy2018",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The casino channel distributed to school districts in FY2018",
        pinned: 92_029_468.0,
        tolerance: 1.0,
        compute: |i| i.casino[&2018],
    },
    Figure {
        key: "dispersion/casino-distributed-fy2019",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The casino channel distributed to school districts in FY2019",
        pinned: 93_928_002.0,
        tolerance: 1.0,
        compute: |i| i.casino[&2019],
    },
    Figure {
        key: "dispersion/casino-distributed-fy2020",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The casino channel distributed to school districts in FY2020",
        pinned: 95_985_938.0,
        tolerance: 1.0,
        compute: |i| i.casino[&2020],
    },
    Figure {
        key: "dispersion/casino-distributed-fy2021",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The same for FY2021 — the closure year, and the trough of the series",
        pinned: 73_873_805.0,
        tolerance: 1.0,
        compute: |i| i.casino[&2021],
    },
    Figure {
        key: "dispersion/casino-distributed-fy2022",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The same for FY2022, the first year the channel exceeds the lottery movement that \
                was legible",
        pinned: 109_385_275.0,
        tolerance: 1.0,
        compute: |i| i.casino[&2022],
    },
    Figure {
        key: "dispersion/casino-distributed-fy2023",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The casino channel distributed to school districts in FY2023",
        pinned: 113_107_108.0,
        tolerance: 1.0,
        compute: |i| i.casino[&2023],
    },
    Figure {
        key: "dispersion/casino-distributed-fy2024",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "The same for FY2024 — the largest year, and still under half the noise floor",
        pinned: 114_177_214.0,
        tolerance: 1.0,
        compute: |i| i.casino[&2024],
    },
    Figure {
        key: "project/hb-96-refresh-run-cost",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What moving the base cost reference year from FY2022 to FY2024 costs across the FY2027 \
                model — the draft's only provision, so the draft's cost is its cost",
        pinned: 220_525_319.16,
        tolerance: 0.005,
        compute: |i| i.refresh.cost().expect("the refresh prices"),
    },
    Figure {
        key: "project/hb-96-refresh-districts-lifted-off-the-guarantee",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts the refresh lifts onto the formula — the only districts for which it changes \
                the kind of thing that determines their aid rather than the amount",
        pinned: 41.0,
        tolerance: 0.0,
        compute: |i| lifted_off_the_guarantee(&i.refresh),
    },
    Figure {
        // Positive, direction in the key: the prose is "the bill cuts $143.9 million".
        key: "project/fund-the-plan-run-cut",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the refresh and a half-retired guarantee cost when they are run together",
        pinned: 143_877_698.81,
        tolerance: 0.005,
        compute: |i| -i.fund_the_plan.cost().expect("the draft prices"),
    },
    Figure {
        key: "project/fund-the-plan-provisions-costed-separately",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the same two provisions come to when each is priced alone and the two are added",
        pinned: 218_951_994.53,
        tolerance: 0.005,
        compute: |i| -i.fund_the_plan.attribution().iter().map(|a| a.cost).sum::<f64>(),
    },
    Figure {
        key: "project/fund-the-plan-interaction-residual",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The difference between those two — what the guarantee's `max` double-counts, and the \
                reason a draft's cost is one combined run",
        pinned: 75_074_295.72,
        tolerance: 0.005,
        compute: |i| i.fund_the_plan.residual().expect("the draft prices"),
    },
    Figure {
        key: "project/school-districts-in-two-or-more-house-districts",
        owner: "crates/project",
        unit: Unit::Count,
        label: "School districts with population in more than one House district — the reason there is no \
                published crosswalk and no clean one to publish",
        pinned: 339.0,
        tolerance: 0.0,
        compute: |i| house_reach(i).values().filter(|seats| **seats >= 2).count() as f64,
    },
    Figure {
        key: "project/house-districts-columbus-city-reaches",
        owner: "crates/project",
        unit: Unit::Count,
        label: "House seats the most-divided school district has population in",
        pinned: 11.0,
        tolerance: 0.0,
        compute: |i| *house_reach(i).values().max().expect("a non-empty crosswalk") as f64,
    },
    Figure {
        key: "project/ohio-under-eighteen-share",
        owner: "crates/project",
        unit: Unit::Share,
        label: "Ohio's under-18 share of population, which is the weight school funding is apportioned \
                across House seats by — not total population, which would weight a seat full of \
                retirees the same as one full of families",
        pinned: 0.2197,
        tolerance: 0.00005,
        compute: |i| { let pop: f64 = i.house.iter().map(|o| o.population).sum(); i.house.iter().map(|o| o.population_under_18).sum::<f64>() / pop },
    },
    // `formula-component/temporary-transitional-aid-guarantee` and the eight other nodes that
    // state its count. This is the single most-repeated computed number in the corpus -- nine
    // nodes across five classes write "294 of 609" -- and until #158 not one of them was bound,
    // which is the shape of the failure #120 was filed about rather than a near miss.
    Figure {
        key: "project/districts-on-the-guarantee",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts the temporary transitional aid guarantee pays in FY2027 -- the terminal \
                year, at a phase-in of 100%",
        pinned: 294.0,
        tolerance: 0.0,
        compute: |i| on_the_guarantee(i).len() as f64,
    },
    // The projected block. `tolerance` is how far the calculator may drift from the pin, not how
    // precisely the corpus writes the figure — the prose comparison is a separate half-step check
    // in `corpusFigures.ts`. `report::forecast` is deterministic over a committed panel, so the
    // only slack these need is the pin's own written precision, and a cent is the convention every
    // other exact figure here uses.
    Figure {
        key: "project/statewide-adm-fy2032",
        owner: "crates/project",
        unit: Unit::Pupils,
        label: "Statewide enrolled ADM projected to FY2032 by the shipped method",
        pinned: 1_384_244.862_5,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2032.adm,
    },
    Figure {
        key: "project/statewide-adm-fy2036",
        owner: "crates/project",
        unit: Unit::Pupils,
        label: "Statewide enrolled ADM projected to FY2036, the feed's horizon \u{2014} twelve \
                pupils below the FY2032 figure, four years earlier",
        pinned: 1_384_232.417_4,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2036.adm,
    },
    Figure {
        key: "project/statewide-adm-fy2036-undamped",
        owner: "crates/project",
        unit: Unit::Pupils,
        label: "The same projection undamped, which the backtest finds very nearly unbiased",
        pinned: 1_290_679.717_1,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2036_undamped.adm,
    },
    Figure {
        key: "project/damping-holds-adm-above-the-undamped-trend",
        owner: "crates/project",
        unit: Unit::Pupils,
        label: "Pupils the damping holds the FY2036 projection above the undamped trend",
        pinned: 93_552.700_2,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2036.adm - i.forecasts.fy2036_undamped.adm,
    },
    Figure {
        key: "project/damping-move-statewide-adm-fy2036",
        owner: "crates/project",
        unit: Unit::Pupils,
        label: "Pupils that moving the damping from 0.85 to 0.30 adds to the FY2036 projection, \
                under the shrunk rate the feed runs",
        pinned: 45_293.897_0,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2036.adm - i.forecasts.fy2036_at_the_old_convention.adm,
    },
    Figure {
        key: "project/damping-move-realized-aid-fy2036",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the same move does to realized state aid at FY2036 \u{2014} proportionally \
                about two fifths of the enrollment move, because the guarantee absorbs most of it",
        pinned: 101_033_524.939_1,
        tolerance: 0.01,
        compute: |i| {
            i.forecasts.fy2036.realized_aid - i.forecasts.fy2036_at_the_old_convention.realized_aid
        },
    },
    Figure {
        key: "project/districts-on-the-guarantee-fy2036",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts the guarantee pays at FY2036 under the shipped projection",
        pinned: 312.0,
        tolerance: 0.0,
        compute: |i| i.forecasts.fy2036.on_guarantee as f64,
    },
    Figure {
        key: "project/districts-on-the-guarantee-fy2036-at-the-old-damping",
        owner: "crates/project",
        unit: Unit::Count,
        label: "The same count at the 0.85 damping, which is the other end of the corpus's \
                44-district move",
        pinned: 356.0,
        tolerance: 0.0,
        compute: |i| i.forecasts.fy2036_at_the_old_convention.on_guarantee as f64,
    },
    // The FY2032 aid band, which two nodes state and three paragraphs carry. It is the shape the
    // whole mechanism was built for and it had no key: `scenario/guarantee-phase-out` says of its
    // own guarantee count that it "is the one figure on this node that no test pinned", having
    // already been corrected once from a wrong number nobody caught. The band beside it was in
    // exactly the same position.
    Figure {
        key: "project/fy2032-aid-current-law",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Total realized state aid at FY2032 enrollment under current law, central estimate",
        pinned: 7_233_134_576.709_7,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2032.realized_aid,
    },
    Figure {
        key: "project/fy2032-aid-current-law-low",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The low end of the FY2032 current-law aid band, from the enrollment \
                projection interval",
        pinned: 6_957_859_359.941_2,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2032.low,
    },
    Figure {
        key: "project/fy2032-aid-current-law-high",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The high end of the FY2032 current-law aid band \u{2014} asymmetric against \
                the low end because the interval is multiplicative, which is why the corpus \
                writes the half-width. Widened when `HORIZON_EXPONENT` was fitted",
        pinned: 7_581_865_913.570_1,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2032.high,
    },
    Figure {
        key: "project/fy2032-aid-guarantee-removed",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Total realized state aid at FY2032 enrollment with the guarantee removed, \
                central estimate",
        pinned: 6_301_076_001.003_5,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2032_guarantee_removed.realized_aid,
    },
    Figure {
        key: "project/fy2032-aid-guarantee-removed-low",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The low end of the FY2032 guarantee-removed aid band, which the corpus \
                reports as nearly twice as wide as the current-law one",
        pinned: 5_830_740_283.837_9,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2032_guarantee_removed.low,
    },
    Figure {
        key: "project/fy2032-aid-guarantee-removed-high",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The high end of the FY2032 guarantee-removed aid band",
        pinned: 6_809_351_272.337_8,
        tolerance: 0.01,
        compute: |i| i.forecasts.fy2032_guarantee_removed.high,
    },
    Figure {
        key: "project/districts-on-the-guarantee-fy2032",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts the guarantee pays at FY2032 enrollment \u{2014} 294 at FY2027, so \
                eighteen move onto it",
        pinned: 312.0,
        tolerance: 0.0,
        compute: |i| i.forecasts.fy2032.on_guarantee as f64,
    },
    // `formula-component/fsfp-formula-transition-supplement`. The second hold-harmless, and the
    // node's point is that it is not nested inside the first.
    Figure {
        key: "project/transition-supplement-total",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Line [K], the formula transition supplement, statewide",
        pinned: 63_578_629.47,
        tolerance: 0.01,
        compute: |i| i.panel.iter().map(|r| r.transition.transition_supplement).sum(),
    },
    Figure {
        key: "project/transition-supplement-districts",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts line [K] pays",
        pinned: 144.0,
        tolerance: 0.0,
        compute: |i| {
            i.panel
                .iter()
                .filter(|r| r.transition.transition_supplement > 0.0)
                .count() as f64
        },
    },
    Figure {
        // The definition is in the key, per the convention #171 recorded: this is the subset that
        // draws the second hold-harmless while drawing nothing from the first, and a figure of 144
        // bound to "17 of the 144" would be the check passing on a definition nobody wrote down.
        key: "project/transition-supplement-districts-off-the-guarantee",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts drawing line [K] that the guarantee does not pay -- the two \
                hold-harmlesses are not nested",
        pinned: 17.0,
        tolerance: 0.0,
        compute: |i| {
            i.panel
                .iter()
                .filter(|r| r.transition.transition_supplement > 0.0 && !r.on_guarantee())
                .count() as f64
        },
    },
    Figure {
        // The panel's population, said so in the key. The department's 611-row count is 440, and
        // the two figures are both right about different populations -- which is the seam
        // `crates/project/tests/the_population_the_panel_speaks_for.rs` exists to hold open.
        key: "project/panel-districts-under-the-transportation-floor",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts of the 609-row panel whose base cost state share sits below the 50% \
                transportation minimum -- the department's 611-row count is 440",
        pinned: 438.0,
        tolerance: 0.0,
        compute: |i| {
            i.panel
                .iter()
                .filter(|r| r.state_share_fraction() < 0.5)
                .count() as f64
        },
    },
    Figure {
        key: "project/districts-on-the-transportation-guarantee",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts held harmless by transportation's own guarantee -- R.C. 3317.019(A)(2), \
                on the same FY2020 base as the guarantee itself and not on the FY2021 its column \
                is headed",
        pinned: 38.0,
        tolerance: 0.0,
        compute: |i| {
            i.panel
                .iter()
                .filter(|r| r.transportation.guarantee > 0.0)
                .count() as f64
        },
    },
    // `parameter/guarantee-funding-base`. Column [H2], which is both the floor under the guarantee
    // and the origin the phase-in interpolates from.
    Figure {
        // **Positive**, not merely present. All 609 rows carry a figure; one of them is negative,
        // and the sentence this binds is about the 608 that are not. Naming that in the key is the
        // convention #171 arrived at after a count bound cleanly to a number meaning something else.
        key: "project/districts-with-a-positive-guarantee-funding-base",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts whose published [H2] guarantee funding base is above zero",
        pinned: 608.0,
        tolerance: 0.0,
        compute: |i| {
            i.panel
                .iter()
                .filter(|r| r.transition.funding_base > 0.0)
                .count() as f64
        },
    },
    Figure {
        key: "project/districts-off-the-guarantee-with-a-positive-funding-base",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Districts the guarantee does not pay that carry a positive [H2] anyway -- being \
                held at the floor is not what reveals the figure",
        pinned: 314.0,
        tolerance: 0.0,
        compute: |i| {
            i.panel
                .iter()
                .filter(|r| !r.on_guarantee() && r.transition.funding_base > 0.0)
                .count() as f64
        },
    },
    Figure {
        // A magnitude with the direction in the key, per the convention #171 recorded: the corpus
        // writes this as `-$40,179.23` and the numeral reader cannot see the sign.
        key: "project/largest-negative-guarantee-funding-base",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "How far below zero the one negative [H2] sits -- Richmond Heights Local, where the \
                FY2020 deductions R.C. 3317.02(N)(1)(b) subtracts exceeded the funding",
        pinned: 40_179.23,
        tolerance: 0.01,
        compute: |i| {
            -i.panel
                .iter()
                .map(|r| r.transition.funding_base)
                .fold(f64::INFINITY, f64::min)
        },
    },
    Figure {
        key: "project/guaranteed-districts-aggregate-funding-base",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The [H2] base summed over the districts the guarantee pays -- what Ohio is \
                committed to for them regardless of what the formula computes",
        pinned: 3_032_797_430.41,
        tolerance: 0.01,
        compute: |i| on_the_guarantee(i).iter().map(|r| r.transition.funding_base).sum(),
    },
    Figure {
        key: "project/guaranteed-districts-formula-amount",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What the formula computes for those same districts -- the quantity the base is \
                compared against and found larger than",
        pinned: 2_150_812_372.08,
        tolerance: 0.01,
        compute: |i| {
            on_the_guarantee(i)
                .iter()
                .map(|r| r.core_foundation_funding)
                .sum()
        },
    },
    Figure {
        key: "project/guaranteed-districts-formula-share-of-base",
        owner: "crates/project",
        unit: Unit::Share,
        label: "The formula amount as a share of the base, over the guaranteed districts -- 70.9%, \
                where the node published 71.0% from dividing the two rounded billions",
        pinned: 0.7092,
        tolerance: 0.00005,
        compute: |i| {
            let base: f64 = on_the_guarantee(i).iter().map(|r| r.transition.funding_base).sum();
            on_the_guarantee(i)
                .iter()
                .map(|r| r.core_foundation_funding)
                .sum::<f64>()
                / base
        },
    },
    // `formula-component/fsfp-special-education-weights`. The distribution runs against the
    // weights: the category carrying the most money is not the one carrying the most pupils.
    Figure {
        key: "project/special-education-total",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Special education aid statewide, the second-largest categorical",
        pinned: 722_177_050.74,
        tolerance: 0.01,
        compute: |i| i.panel.iter().map(|r| r.special_education.total()).sum(),
    },
    Figure {
        key: "project/special-education-category-six-share-of-pupils",
        owner: "crates/project",
        unit: Unit::Share,
        label: "Category 6's share of special education pupils -- few pupils at the highest weight",
        pinned: 0.1508,
        tolerance: 0.00005,
        compute: |i| special_education_shares(i, 6).0,
    },
    Figure {
        key: "project/special-education-category-six-share-of-money",
        owner: "crates/project",
        unit: Unit::Share,
        label: "Category 6's share of special education money, against a sixth of the pupils",
        pinned: 0.4810,
        tolerance: 0.00005,
        compute: |i| special_education_shares(i, 6).1,
    },
    Figure {
        key: "project/special-education-category-two-share-of-pupils",
        owner: "crates/project",
        unit: Unit::Share,
        label: "Category 2's share of special education pupils -- the opposite shape, many pupils \
                at a low weight",
        pinned: 0.6500,
        tolerance: 0.00005,
        compute: |i| special_education_shares(i, 2).0,
    },
    Figure {
        key: "project/special-education-category-two-share-of-money",
        owner: "crates/project",
        unit: Unit::Share,
        label: "Category 2's share of special education money",
        pinned: 0.3385,
        tolerance: 0.00005,
        compute: |i| special_education_shares(i, 2).1,
    },
    Figure {
        key: "project/special-education-categories-two-and-six-share-of-money",
        owner: "crates/project",
        unit: Unit::Share,
        label: "What the two together are of the programme, by money -- the pair is 82% of the \
                spending and 80% of the pupils, which is why the sentence has to say which",
        pinned: 0.8195,
        tolerance: 0.00005,
        compute: |i| special_education_shares(i, 2).1 + special_education_shares(i, 6).1,
    },
    Figure {
        key: "project/special-education-category-four-pupils",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Category 4 pupils statewide, the smallest of the six",
        pinned: 1_060.0,
        tolerance: 0.0,
        compute: |i| special_education_totals(i).0[3].round(),
    },
    Figure {
        key: "project/special-education-category-four-aid",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "What Category 4 generates statewide",
        pinned: 5_675_479.07,
        tolerance: 0.01,
        compute: |i| special_education_totals(i).1[3],
    },
    // `formula-component/fsfp-career-technical-weights`. Mechanically special education's shape,
    // and the difference is entirely the multiplicand.
    Figure {
        key: "project/career-technical-total",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "Career-technical aid statewide, the five categories plus associated services",
        pinned: 53_875_678.32,
        tolerance: 0.01,
        compute: |i| i.panel.iter().map(|r| r.career_technical.total()).sum(),
    },
    Figure {
        key: "project/career-technical-associated-services",
        owner: "crates/project",
        unit: Unit::Dollars,
        label: "The sixth weight, applied to total FTE rather than to any category -- services \
                rather than instruction",
        pinned: 3_234_233.67,
        tolerance: 0.01,
        compute: |i| {
            i.panel
                .iter()
                .map(|r| r.career_technical.associated_services)
                .sum()
        },
    },
    Figure {
        key: "project/career-technical-base-cost-premium",
        owner: "crates/project",
        unit: Unit::Share,
        label: "How far the career-technical base cost per pupil sits above the statewide average \
                the other weighted categoricals multiply -- a CTE pupil starts higher before any \
                weight is applied",
        pinned: 0.1958,
        tolerance: 0.00005,
        compute: |_| {
            project::panel::CTE_BASE_COST_PER_PUPIL / project::panel::AVERAGE_BASE_COST_PER_PUPIL
                - 1.0
        },
    },

    // The eleven statutory multiples, which two nodes state and a test checks against the Revised
    // Code. Bound as figures rather than left to that test alone because the test compares the
    // crate against `ohio-laws`, and this compares the *corpus* against the crate: an edit to
    // either constant array reddens every sentence that quotes it, which is the whole mechanism.
    Figure {
        key: "project/special-education-weight-category-one",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory special education weight on Category one, R.C. 3317.013(A)",
        pinned: 0.2435,
        tolerance: 0.0,
        compute: |_| project::panel::SPECIAL_EDUCATION_WEIGHTS[0],
    },
    Figure {
        key: "project/special-education-weight-category-two",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory special education weight on Category two, R.C. 3317.013(A)",
        pinned: 0.6179,
        tolerance: 0.0,
        compute: |_| project::panel::SPECIAL_EDUCATION_WEIGHTS[1],
    },
    Figure {
        key: "project/special-education-weight-category-three",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory special education weight on Category three, R.C. 3317.013(A)",
        pinned: 1.4845,
        tolerance: 0.0,
        compute: |_| project::panel::SPECIAL_EDUCATION_WEIGHTS[2],
    },
    Figure {
        key: "project/special-education-weight-category-four",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory special education weight on Category four, R.C. 3317.013(A)",
        pinned: 1.9812,
        tolerance: 0.0,
        compute: |_| project::panel::SPECIAL_EDUCATION_WEIGHTS[3],
    },
    Figure {
        key: "project/special-education-weight-category-five",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory special education weight on Category five, R.C. 3317.013(A)",
        pinned: 2.6830,
        tolerance: 0.0,
        compute: |_| project::panel::SPECIAL_EDUCATION_WEIGHTS[4],
    },
    Figure {
        key: "project/special-education-weight-category-six",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory special education weight on Category six, R.C. 3317.013(A)",
        pinned: 3.9554,
        tolerance: 0.0,
        compute: |_| project::panel::SPECIAL_EDUCATION_WEIGHTS[5],
    },
    Figure {
        key: "project/career-technical-weight-category-one",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory career-technical weight on Category one, R.C. 3317.014(A)",
        pinned: 0.6230,
        tolerance: 0.0,
        compute: |_| project::panel::CTE_WEIGHTS[0],
    },
    Figure {
        key: "project/career-technical-weight-category-two",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory career-technical weight on Category two, R.C. 3317.014(A)",
        pinned: 0.5905,
        tolerance: 0.0,
        compute: |_| project::panel::CTE_WEIGHTS[1],
    },
    Figure {
        key: "project/career-technical-weight-category-three",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory career-technical weight on Category three, R.C. 3317.014(A)",
        pinned: 0.2154,
        tolerance: 0.0,
        compute: |_| project::panel::CTE_WEIGHTS[2],
    },
    Figure {
        key: "project/career-technical-weight-category-four",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory career-technical weight on Category four, R.C. 3317.014(A)",
        pinned: 0.1830,
        tolerance: 0.0,
        compute: |_| project::panel::CTE_WEIGHTS[3],
    },
    Figure {
        key: "project/career-technical-weight-category-five",
        owner: "crates/project",
        unit: Unit::Ratio,
        label: "The statutory career-technical weight on Category five, R.C. 3317.014(A)",
        pinned: 0.1570,
        tolerance: 0.0,
        compute: |_| project::panel::CTE_WEIGHTS[4],
    },
    // --- The Toledo–Perrysburg cross-section, FY2025 ------------------------------------
    Figure {
        key: "dispersion/toledo-operating-per-headcount-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Toledo operating spending per headcount pupil, FY2025",
        pinned: 20_804.57,
        tolerance: 0.01,
        compute: |i| i.toledo.operating(),
    },
    Figure {
        key: "dispersion/perrysburg-operating-per-headcount-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Perrysburg operating spending per headcount pupil, FY2025",
        pinned: 14_632.16,
        tolerance: 0.01,
        compute: |i| i.perrysburg.operating(),
    },
    Figure {
        key: "dispersion/median-operating-per-headcount-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Median operating spending per headcount pupil, over the 607 districts rated",
        pinned: 16_289.44,
        tolerance: 0.01,
        compute: |i| i.median_operating_per_pupil,
    },
    Figure {
        key: "dispersion/toledo-perrysburg-operating-gap",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "What Toledo spends per pupil over Perrysburg, FY2025",
        pinned: 6_172.41,
        tolerance: 0.01,
        compute: |i| i.toledo.operating() - i.perrysburg.operating(),
    },
    Figure {
        key: "dispersion/toledo-instruction-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Instruction as a share of Toledo operating spending, FY2025",
        pinned: 0.512_640_251_6,
        tolerance: 0.000_001,
        compute: |i| i.toledo.share(|f| f.instruction),
    },
    Figure {
        key: "dispersion/perrysburg-instruction-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Instruction as a share of Perrysburg operating spending, FY2025",
        pinned: 0.626_336_098,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.share(|f| f.instruction),
    },
    Figure {
        key: "dispersion/toledo-classroom-instruction-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Classroom instruction as a share of Toledo operating spending, FY2025",
        pinned: 0.634_294_772_7,
        tolerance: 0.000_001,
        compute: |i| i.toledo.share(|f| f.classroom_instruction),
    },
    Figure {
        key: "dispersion/perrysburg-classroom-instruction-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Classroom instruction as a share of Perrysburg operating spending, FY2025",
        pinned: 0.731_052_011_5,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.share(|f| f.classroom_instruction),
    },
    Figure {
        key: "dispersion/toledo-operations-maintenance-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Operations and maintenance as a share of Toledo operating spending, FY2025",
        pinned: 0.147_772_821_1,
        tolerance: 0.000_001,
        compute: |i| i.toledo.share(|f| f.operations_maintenance),
    },
    Figure {
        key: "dispersion/perrysburg-operations-maintenance-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Operations and maintenance as a share of Perrysburg operating spending, FY2025",
        pinned: 0.077_453_363,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.share(|f| f.operations_maintenance),
    },
    Figure {
        key: "dispersion/toledo-school-admin-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Building administration as a share of Toledo operating spending, FY2025",
        pinned: 0.085_123_124_39,
        tolerance: 0.000_001,
        compute: |i| i.toledo.share(|f| f.school_admin),
    },
    Figure {
        key: "dispersion/perrysburg-school-admin-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Building administration as a share of Perrysburg operating spending, FY2025",
        pinned: 0.048_491_815_29,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.share(|f| f.school_admin),
    },
    Figure {
        key: "dispersion/toledo-pupil-support-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Student support as a share of Toledo operating spending, FY2025",
        pinned: 0.081_496_998_02,
        tolerance: 0.000_001,
        compute: |i| i.toledo.share(|f| f.pupil_support),
    },
    Figure {
        key: "dispersion/perrysburg-pupil-support-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Student support as a share of Perrysburg operating spending, FY2025",
        pinned: 0.084_976_517_48,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.share(|f| f.pupil_support),
    },
    Figure {
        key: "dispersion/perrysburg-pupil-transportation-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Pupil transportation as a share of Perrysburg operating spending, FY2025",
        pinned: 0.033_671_036_95,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.share(|f| f.pupil_transportation),
    },

    // --- The same two districts on the 2024-25 report card -------------------------------
    Figure {
        key: "dispersion/toledo-operating-expenditure",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Toledo total operating expenditure, FY2025",
        pinned: 440_219_539.58,
        tolerance: 0.01,
        compute: |i| i.toledo.card(|c| c.operating_expenditure),
    },
    Figure {
        key: "dispersion/toledo-per-equivalent-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Toledo operating spending per equivalent pupil, as the report card prints it",
        pinned: 14_312.0,
        tolerance: 0.01,
        compute: |i| i.toledo.card(|c| c.per_equivalent_pupil),
    },
    Figure {
        key: "dispersion/toledo-enrolled-adm",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Toledo enrolled ADM, FY2025, a headcount and not a whole number of pupils",
        pinned: 21_159.753_2,
        tolerance: 0.000_001,
        compute: |i| i.toledo.card(|c| c.unweighted_adm),
    },
    Figure {
        key: "dispersion/toledo-weighted-adm",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Toledo weighted ADM, FY2025, as the report card publishes it",
        pinned: 30_758.0,
        tolerance: 0.0,
        compute: |i| i.toledo.card(|c| c.weighted_adm),
    },
    Figure {
        key: "dispersion/toledo-weight-ratio",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Toledo weighted ADM over its headcount, FY2025",
        pinned: 1.453_608_637,
        tolerance: 0.000_001,
        compute: |i| i.toledo.card(|c| c.weighted_adm) / i.toledo.card(|c| c.unweighted_adm),
    },
    Figure {
        key: "dispersion/toledo-performance-index",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Toledo Performance Index, 2024-25",
        pinned: 60.4,
        tolerance: 0.000_001,
        compute: |i| i.toledo.card(|c| c.performance_index),
    },
    Figure {
        key: "dispersion/toledo-performance-index-prior",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Toledo Performance Index, 2023-24",
        pinned: 57.9,
        tolerance: 0.000_001,
        compute: |i| i.toledo.card(|c| c.performance_index_prior),
    },
    Figure {
        key: "dispersion/toledo-performance-index-earliest",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Toledo Performance Index, 2022-23",
        pinned: 58.8,
        tolerance: 0.000_001,
        compute: |i| i.toledo.card(|c| c.performance_index_earliest),
    },
    Figure {
        key: "dispersion/toledo-economically-disadvantaged-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Toledo: Share of pupils economically disadvantaged, 2024-25",
        pinned: 0.989,
        tolerance: 0.000_001,
        compute: |i| i.toledo.card(|c| c.economically_disadvantaged) / 100.0,
    },
    Figure {
        key: "dispersion/toledo-students-with-disabilities-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Toledo: Share of pupils with a disability, 2024-25",
        pinned: 0.219,
        tolerance: 0.000_001,
        compute: |i| i.toledo.card(|c| c.students_with_disabilities) / 100.0,
    },
    Figure {
        key: "dispersion/toledo-english-learner-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Toledo: Share of pupils who are English learners, 2024-25",
        pinned: 0.024,
        tolerance: 0.000_001,
        compute: |i| i.toledo.card(|c| c.english_learner) / 100.0,
    },
    Figure {
        key: "dispersion/perrysburg-operating-expenditure",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Perrysburg total operating expenditure, FY2025",
        pinned: 79_735_653.02,
        tolerance: 0.01,
        compute: |i| i.perrysburg.card(|c| c.operating_expenditure),
    },
    Figure {
        key: "dispersion/perrysburg-per-equivalent-pupil",
        owner: "crates/dispersion",
        unit: Unit::Dollars,
        label: "Perrysburg operating spending per equivalent pupil, as the report card prints it",
        pinned: 12_392.0,
        tolerance: 0.01,
        compute: |i| i.perrysburg.card(|c| c.per_equivalent_pupil),
    },
    Figure {
        key: "dispersion/perrysburg-enrolled-adm",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Perrysburg enrolled ADM, FY2025, a headcount and not a whole number of pupils",
        pinned: 5_449.342_8,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.card(|c| c.unweighted_adm),
    },
    Figure {
        key: "dispersion/perrysburg-weighted-adm",
        owner: "crates/dispersion",
        unit: Unit::Count,
        label: "Perrysburg weighted ADM, FY2025, as the report card publishes it",
        pinned: 6_435.0,
        tolerance: 0.0,
        compute: |i| i.perrysburg.card(|c| c.weighted_adm),
    },
    Figure {
        key: "dispersion/perrysburg-weight-ratio",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Perrysburg weighted ADM over its headcount, FY2025",
        pinned: 1.180_876_343,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.card(|c| c.weighted_adm) / i.perrysburg.card(|c| c.unweighted_adm),
    },
    Figure {
        key: "dispersion/perrysburg-performance-index",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Perrysburg Performance Index, 2024-25",
        pinned: 103.5,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.card(|c| c.performance_index),
    },
    Figure {
        key: "dispersion/perrysburg-performance-index-prior",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Perrysburg Performance Index, 2023-24",
        pinned: 102.8,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.card(|c| c.performance_index_prior),
    },
    Figure {
        key: "dispersion/perrysburg-performance-index-earliest",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "Perrysburg Performance Index, 2022-23",
        pinned: 102.7,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.card(|c| c.performance_index_earliest),
    },
    Figure {
        key: "dispersion/perrysburg-economically-disadvantaged-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Perrysburg: Share of pupils economically disadvantaged, 2024-25",
        pinned: 0.138,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.card(|c| c.economically_disadvantaged) / 100.0,
    },
    Figure {
        key: "dispersion/perrysburg-students-with-disabilities-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Perrysburg: Share of pupils with a disability, 2024-25",
        pinned: 0.113,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.card(|c| c.students_with_disabilities) / 100.0,
    },
    Figure {
        key: "dispersion/perrysburg-english-learner-share",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "Perrysburg: Share of pupils who are English learners, 2024-25",
        pinned: 0.023,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.card(|c| c.english_learner) / 100.0,
    },
    Figure {
        key: "dispersion/toledo-progress-effect-size-below-zero",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How far Toledo value-added effect size sits below zero, 2024-25",
        pinned: 0.02,
        tolerance: 0.000_001,
        compute: |i| -i.toledo.card(|c| c.progress_effect_size),
    },
    Figure {
        key: "dispersion/toledo-progress-composite-below-zero",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How far Toledo value-added composite sits below zero, 2024-25",
        pinned: 5.07,
        tolerance: 0.000_001,
        compute: |i| -i.toledo.card(|c| c.progress_composite),
    },
    Figure {
        key: "dispersion/perrysburg-progress-effect-size-above-zero",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How far Perrysburg value-added effect size sits above zero, 2024-25",
        pinned: 0.17,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.card(|c| c.progress_effect_size),
    },
    Figure {
        key: "dispersion/perrysburg-progress-composite-above-zero",
        owner: "crates/dispersion",
        unit: Unit::Ratio,
        label: "How far Perrysburg value-added composite sits above zero, 2024-25",
        pinned: 23.76,
        tolerance: 0.000_001,
        compute: |i| i.perrysburg.card(|c| c.progress_composite),
    },
    Figure {
        key: "dispersion/toledo-denominator-spread",
        owner: "crates/dispersion",
        unit: Unit::Share,
        label: "How much higher Toledo spending reads per headcount pupil than per weighted one",
        pinned: 0.453_608_636_6,
        tolerance: 0.000_001,
        compute: |i| {
            let dollars = i.toledo.card(|c| c.operating_expenditure);
            let headcount = dollars / i.toledo.card(|c| c.unweighted_adm);
            let weighted = dollars / i.toledo.card(|c| c.weighted_adm);
            headcount / weighted - 1.0
        },
    },
    // --- Sub. H.B. 583 of the 134th, read from the enrolled act ---------------------------
    Figure {
        key: "project/hb583-sections-of-chapter-3317-reprinted",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Sections of R.C. 3317 that H.B. 583 reprints, and therefore amends",
        pinned: 13.0,
        tolerance: 0.0,
        compute: |_| reprinted_in(project::act::HB583, "3317."),
    },
    Figure {
        key: "project/hb583-uncodified-sections-reopened",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Uncodified sections of H.B. 110 that H.B. 583 reopens",
        pinned: 4.0,
        tolerance: 0.0,
        compute: |_| reprinted_in(project::act::HB583, "265."),
    },
    Figure {
        key: "project/hb583-sections-citing-the-assembly-section",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Sections of H.B. 583 that cite R.C. 3317.022, the section it does not reopen",
        pinned: 10.0,
        tolerance: 0.0,
        compute: |_| project::act::sections_citing(project::act::HB583, "3317.022").len() as f64,
    },
    // --- H.B. 643 of the 136th, the draft the model cannot price ---------------------------
    Figure {
        key: "project/hb-643-provisions",
        owner: "crates/project",
        unit: Unit::Count,
        label: "Provisions in H.B. 643 of the 136th General Assembly, as introduced",
        pinned: 1.0,
        tolerance: 0.0,
        compute: |_| hb643().provisions.len() as f64,
    },
    Figure {
        key: "project/hb-643-provisions-priced",
        owner: "crates/project",
        unit: Unit::Count,
        label: "How many of them this repository can price, which is none of them",
        pinned: 0.0,
        tolerance: 0.0,
        compute: |_| hb643().priced().len() as f64,
    },
];

/// The narrowest and widest share of the local gap state aid closes across FY2012-FY2024.
///
/// The window opens at FY2012 because FY2010 and FY2011 are ARRA years and sit below it, and it
/// closes at FY2024 because that is where the survey stops. It used to close at FY2022: see
/// `dispersion::ohio_panel::MIN_ENROLMENT` for why the two years after it read as a break.
fn band(i: &Inputs) -> (f64, f64) {
    let shares: Vec<f64> = i
        .equalization
        .iter()
        .filter(|(year, _)| (2012..=2024).contains(*year))
        .map(|(_, e)| e.state_share())
        .collect();
    (
        shares.iter().copied().fold(f64::MAX, f64::min),
        shares.iter().copied().fold(f64::MIN, f64::max),
    )
}
