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
        label: "Districts held harmless by transportation's own FY2021 guarantee -- the third of \
                the three mechanisms anchored to that year",
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
