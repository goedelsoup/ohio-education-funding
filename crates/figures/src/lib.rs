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

use std::collections::{BTreeMap, HashMap};

use edfund_core::FiscalYear;
use project::budget_analysis::{
    self, Edition, ALI_200540_TOTAL, LOTTERY_LINE, PRESCHOOL_REMAINDER, TOTAL_FOUNDATION_AID,
};
use project::panel::{panel, DistrictRecord};
use regime_diff::recognized_valuation::{self, Recognition};
use regime_diff::{panel_at_fy2027, ChargeOffBase, RegimeDiff, TERMINAL_MILLS};

mod json;

pub use json::manifest;

/// The manifest schema version. Bump on any change to the fields an entry carries.
///
/// Read by the consumer before it reads anything else, on the same rule
/// [`bundle::CONTRACT_VERSION`](../../bundle/index.html) states: a check that does not recognise
/// the document should refuse to run rather than compare fields it may be misreading. A gate that
/// silently passes because it could not find what it was looking for is the failure mode #125
/// catalogued sixteen times.
pub const CONTRACT_VERSION: &str = "1.0.0";

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
}

impl Inputs {
    /// Read the fixtures and run both counterfactuals.
    #[must_use]
    pub fn build() -> Self {
        let panel = panel();
        // Cloned rather than borrowed: `panel` moves into the struct literal below, and the two
        // draft runs need it before that happens.
        let panel_for_drafts = panel.clone();
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
            joined: project::outcomes::joined(),
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
        }
    }
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
];
