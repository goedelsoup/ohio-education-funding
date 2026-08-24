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
//! Roughly a fifth of the corpus's crate-attributed numeric claims. A figure earns a place here by
//! being computable through a crate's **public** API; the rest are computed inside a test file, on
//! a fixture parser that test declares privately, and exporting them means moving that parser into
//! the library. That is worth doing and is not done here. `crates/figures.json` is a floor that
//! only rises: `web/tests/unit/corpusFigures.spec.ts` pins the bound count at its value, so the
//! next figure exported is a figure that cannot quietly go unbound.

#![forbid(unsafe_code)]

use std::collections::HashMap;

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
            actual_total,
            recognized_total,
        }
    }
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
];
