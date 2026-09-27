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
//!
//! # The series manifest
//!
//! A figure is a scalar, and a chart is a column. [`SERIES`] is the second registry: each entry
//! is a short column of labelled values the wiki draws rather than quotes, computed from the same
//! [`Inputs`] and written as [`series_manifest`] to `crates/series.json`. It has its own contract
//! version, [`SERIES_CONTRACT_VERSION`], because its consumer (`web/src/lib/corpusSeries.ts`) reads
//! a different shape and should refuse a document it does not recognise on its own terms.
//!
//! What keeps a chart honest is the **endpoint rule**: the largest and smallest row of every
//! series name an ordinary figure in [`FIGURES`], and that figure reproduces the row. A reader
//! quotes a chart by its extremes, so the extremes are the values the corpus already binds and
//! pins, and a series cannot draw a number the figure manifest has not stood behind. Rows are
//! **signed** — a chart needs the sign to draw a bar below the zero rule — while the figure
//! manifest's correlations are magnitudes with the direction in the key, so the rule compares the
//! row's magnitude against the figure's pin.
//!
//! The same document carries a second drawing. [`SCATTERS`] is a registry of *clouds*: a whole
//! population on a continuous predictor, under one or more panels, each with a line the owning
//! crate fitted. A column has endpoints; a cloud does not, and what stands in for the rule there
//! is the **fit** — a panel names the figures that pin its slope and its r-squared, and those two
//! numbers are the only ones printed on it. The rule underneath is the same one in both cases:
//! whatever a reader could quote off the picture is a number the figure manifest has pinned.
//!
//! And a fourth. [`CURVES`] is a registry of *curves*: two or more named lines over one ordered
//! index, drawn against a **reference** the reader is asked to compare them to. A column's
//! categories can be reordered without changing what it says and a cloud's points have no order at
//! all; a curve's index is monotone and the claim is a *shape* along it — that one line sinks and
//! the other stays flat. So neither extreme of a curve is the thing a reader quotes: both ends of
//! *each line* are, and so is the worst departure from the reference, which is a maximum over the
//! index rather than a value at any point on it. The rule is therefore **three figures per line**
//! — first point, last point, worst departure — and that is what makes "flat the whole way" a
//! claim the figure manifest has stood behind rather than an impression from a picture.
//!
//! The reference itself names no figure, and cannot: 68.3% is the area under a normal curve
//! inside one standard deviation. No crate computes it, nothing would go stale if it moved, and a
//! pin duplicating a definition checks nothing. It is the one number on a curve that is not a
//! measurement, which is exactly why the lines are read against it.
//!
//! And a third. [`PLANES`] is a registry of *planes*: a handful of named alternatives placed on
//! two signed axes at once, where the finding is that the axes **disagree**. A cloud is six
//! hundred subjects whose shape is the claim and no one of which a reader quotes; a plane is
//! seven policies, each of which a reader quotes by name and by both coordinates. So neither the
//! endpoint rule nor the fit rule fits it, and what stands in for them is the strictest form of
//! the same rule: **every** position names a figure for each of its two coordinates. Seven points
//! is few enough that all fourteen numbers can be pinned, so all fourteen are.
//!
//! And a fifth. [`SPREADS`] is a registry of *spreads*: a whole population on two signed axes
//! that sum to a third quantity, where the finding is **which regions of the plane are full and
//! which are empty**. It is the one shape whose claim a list cannot carry at all. "Nobody is off
//! the floor because the formula pays less per child" is a count of four districts in one wedge
//! of six hundred, and a reader will take a full region on trust from prose long before they take
//! an empty one. A cloud's rule is its fit and a spread has nothing fitted, so what stands in is
//! the **census**: every region a reader could count names a figure, and — the second half of the
//! same rule — the subjects the computation could not place are named on the picture rather than
//! dropped out of the denominator. A population is the thing that silently changes when a fixture
//! is extended, and a spread of 607 captioned 609 is a small lie that is easy to ship.
//!
//! And a sixth. [`BANDS`] is a registry of *bands*: one row per step of an ordered index, each
//! running between two values whose **ratio** is the finding. Drawn on a log axis, where a row's
//! length is that ratio and its position is its level — which is why a set of gaps that a column
//! chart would turn into a statement about district size stays a statement about the gap. What
//! stands in for the endpoint rule is the two ends of the **first and last row**, which are what
//! a reader quotes off an ordered picture, plus every [`Marker`] — a position in the plan drawn
//! on the index, which is a parameter the prose quotes rather than an arithmetic definition —
//! plus the [`Aggregate`] the whole picture is captioned with. Six numbers on the one band chart
//! this manifest carries, and all six are pinned.
//!
//! [`Ranges::unreached`] is the same second half of the rule a spread carries, put to a second
//! use: there it is the subjects a computation could not place, and here it is the **floors the
//! source cannot see**. One of R.C. 3317.011's seven binding staffing floors meets a column of
//! the District Profile Report and six meet none, so a picture of the one presented as a picture
//! of staffing would overstate what the report supports. The negative is checkable because
//! `dispersion::profile::EXPECTED_HEADER` is asserted on, which is what makes it worth drawing.
//!
//! # Where the registries are
//!
//! One module each — `figure.rs`, `series.rs`, `scatter.rs`, `plane.rs`, `curve.rs`,
//! `spread.rs`, `band.rs` — holding that registry's form, its static, its `compute_all_*`
//! entry point and the computations nothing else calls. What they share stays in this file:
//! [`Unit`], [`Axis`], and [`Inputs`], which is the one build of the fixtures and both
//! counterfactuals that all seven read.
//!
//! The division is by use and not by taste. A helper two registries call is a helper this file
//! keeps, and each module reads its parent rather than its siblings, so there is one direction
//! to look in when a figure and the chart drawn from it disagree.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::OnceLock;

use edfund_core::FiscalYear;
use project::budget_analysis::{
    self, Edition, ALI_200540_TOTAL, LOTTERY_LINE, PRESCHOOL_REMAINDER, TOTAL_FOUNDATION_AID,
};
use project::panel::{panel, DistrictRecord};
use regime_diff::recognized_valuation::{self, Recognition};
use regime_diff::{panel_at_fy2027, ChargeOffBase, RegimeDiff, TERMINAL_MILLS};

mod band;
mod curve;
mod figure;
mod json;
mod plane;
mod scatter;
mod series;
mod spread;

pub use band::{compute_all_bands, Aggregate, Band, ComputedBand, Marker, Ranges, Span, BANDS};
pub use curve::{compute_all_curves, ComputedCurve, Curve, Reference, Traces, CURVES};
pub use figure::{compute_all, Computed, Figure, FIGURES};
pub use json::{manifest, series_manifest};
pub use plane::{compute_all_planes, ComputedPlane, Plane, PLANES};
pub use scatter::{
    compute_all_scatters, Cloud, ComputedCloud, Fit, Panel, PanelSpec, Point, Scatter, SCATTERS,
};
pub use series::{compute_all_series, ComputedSeries, Row, Series, SERIES};
pub use spread::{
    compute_all_spreads, Boundary, ComputedSpread, Mark, Regions, Slice, Spread, Tally, SPREADS,
};

/// The damping `DEFAULT_DAMPING` replaced, restated rather than remembered.
///
/// Every "moving the damping from 0.85 to 0.30 moves" figure in [`FIGURES`] is a difference
/// against a run at this value. Not imported from anywhere, because there is nowhere to import
/// it from: it is a number the repository no longer uses, kept because the corpus quotes
/// differences against it.
const THE_DAMPING_BEFORE_IT_WAS_FITTED: f64 = 0.85;

/// The manifest schema version. Bump on any change to the fields an entry carries.
///
/// Read by the consumer before it reads anything else, on the same rule
/// [`bundle::CONTRACT_VERSION`](../../bundle/index.html) states: a check that does not recognise
/// the document should refuse to run rather than compare fields it may be misreading. A gate that
/// silently passes because it could not find what it was looking for is the failure mode #125
/// catalogued sixteen times.
pub const CONTRACT_VERSION: &str = "1.2.0";

/// The series manifest's schema version, on the same rule as [`CONTRACT_VERSION`] and versioned
/// separately from it: the two documents are read by two consumers, and a field added to a row
/// is not a reason for the scalar check to refuse a manifest it still reads correctly.
///
/// 2.0.0 was the second drawing, [`SCATTERS`], arriving beside the first. 3.0.0 is the four
/// optional members a row gained for the census of the plan's bounds — [`Row::group`],
/// [`Row::of`], [`Row::cites`] and [`Row::marked`] — which are what let a long ranked column be
/// read as a table as well as a chart. 4.0.0 is the third drawing, [`PLANES`], 5.0.0 the fourth,
/// [`CURVES`], 6.0.0 the fifth, [`SPREADS`], and 7.0.0 the sixth, [`BANDS`]. Its consumer
/// compares the version exactly, so all six were breaking whether or not any existing entry
/// moved, which is the behaviour wanted: a reader that has not been taught what a member means
/// should stop rather than draw half of one.
pub const SERIES_CONTRACT_VERSION: &str = "7.0.0";

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
    /// A number of staff positions, in full-time equivalents.
    ///
    /// Not a [`Count`](Unit::Count) for the reason [`Pupils`](Unit::Pupils) is not: a district
    /// reports 6.25 FTE administrators and R.C. 3317.011 funds it 5.23, because both sides are a
    /// share of a person's time and neither is a tally of people. Unlike a pupil count it is
    /// often whole anyway — thirty-nine administrators is thirty-nine — so what marks it out is
    /// the *tolerance* rather than the fraction, and `a_position_is_an_fte_rather_than_a_head`
    /// says so. Written bare, like a count, and compared to the hundredth the report states it
    /// to.
    Positions,
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
            Self::Positions => "positions",
        }
    }
}

/// One continuous axis: what it measures, and the domain it is drawn on.
pub struct Axis {
    /// What the axis measures, in words.
    pub label: &'static str,
    /// What it is measured in, so that a consumer can write a coordinate the way the rest of the
    /// site writes that quantity. A figure carries one for the same reason and it is the same
    /// enumeration: a cloud whose axes were bare numbers would be a chart formatting dollars by
    /// guessing from their size.
    pub unit: Unit,
    /// The low end of the drawn domain, in the values' own units.
    pub min: f64,
    /// The high end.
    pub max: f64,
    /// Drawn on a log scale, which every axis of the first cloud is: the fits are in logs, and a
    /// line fitted in logs is a line only where the axes are.
    pub log: bool,
}

/// How much room a computed domain leaves between the extreme point and the frame, at each end,
/// as a share of the span.
///
/// The same 4% the scatter form pads an axis it fits itself by, restated here because these
/// domains are not fitted by the renderer: the manifest carries the frame, so the frame is what
/// has to leave the room.
const CLOUD_PAD: f64 = 0.04;

/// One subject of a [`Plane`], placed on both axes, with both coordinates pinned.
pub struct Position {
    /// What the point is called, on the chart and in its tooltip. Carries the frame it was
    /// measured in, because the subjects here were not all priced at the same horizon.
    pub label: String,
    /// Its place on the horizontal axis, signed.
    pub x: f64,
    /// Its place on the vertical axis, signed.
    pub y: f64,
    /// The [`Figure`] whose pin [`Self::x`] reproduces in magnitude.
    pub x_figure: &'static str,
    /// And the one [`Self::y`] reproduces.
    pub y_figure: &'static str,
}

/// The subjects of a [`Plane`] and the frame they are drawn in.
pub struct Positions {
    /// The horizontal axis.
    pub x: Axis,
    /// The vertical axis.
    pub y: Axis,
    /// The subjects, in the order the owning crate returned them.
    pub points: Vec<Position>,
}

impl Positions {
    /// Frame a set of positions around their own extremes, with **zero inside both axes**.
    ///
    /// Zero is not a data value here and it is the most important place on the picture: it is the
    /// rule in force, the line each axis is read against, and the boundary between the quadrant
    /// where a rule is cheaper and the quadrant where it is dearer. A domain fitted to the points
    /// alone would drop it whenever every rule falls on one side, and then the chart would draw a
    /// spread where the claim is a sign.
    ///
    /// Both axes are padded by the same `CLOUD_PAD` the other drawings leave, and neither is
    /// matched to the other: they measure different quantities, they are not being compared as
    /// lengths, and a shared scale would flatten whichever of them is smaller.
    ///
    /// # Panics
    ///
    /// If there are no points, or a coordinate is not finite.
    #[must_use]
    pub fn framed(x: &'static str, y: &'static str, unit: Unit, points: Vec<Position>) -> Self {
        assert!(!points.is_empty(), "a plane with no points");
        let frame = |values: &[f64], label: &'static str| {
            let (low, high) = values.iter().fold((0.0_f64, 0.0_f64), |(lo, hi), v| {
                assert!(v.is_finite(), "{label}: {v} cannot be placed");
                (lo.min(*v), hi.max(*v))
            });
            let pad = (high - low) * CLOUD_PAD;
            Axis {
                label,
                unit,
                min: low - pad,
                max: high + pad,
                log: false,
            }
        };
        let xs: Vec<f64> = points.iter().map(|p| p.x).collect();
        let ys: Vec<f64> = points.iter().map(|p| p.y).collect();
        Self {
            x: frame(&xs, x),
            y: frame(&ys, y),
            points,
        }
    }
}

/// One point on a [`Line`].
pub struct Coordinate {
    /// Its place on the shared index.
    pub x: f64,
    /// Its value.
    pub y: f64,
}

/// The worst a [`Line`] departs from its curve's reference, and where.
///
/// A maximum over the index rather than a value at a point on it, which is why it is computed
/// here and pinned as a figure of its own: "within three points the whole way" is the sentence
/// the flat line is drawn to support, and no single coordinate states it. Ties go to the
/// shallower end of the index, which is the one a reader would quote.
pub struct Departure {
    /// The index position the worst gap is at.
    pub at: f64,
    /// Its size, unsigned — a distance from the reference, so a line above and a line below by
    /// the same amount have departed equally.
    pub gap: f64,
    /// The [`Figure`] that pins [`Self::gap`].
    pub figure: &'static str,
}

/// One line of a [`Curve`]: a value at every position on the shared index.
///
/// Its three pinned numbers are the ones a reader can take off the picture. Both ends, because a
/// line is quoted by where it starts and where it finishes; and [`Self::worst`], because the
/// claim a curve is drawn for is usually about the whole line rather than any point on it.
pub struct Line {
    /// What this line is, in words — the legend entry.
    pub label: &'static str,
    /// The points, in index order.
    pub points: Vec<Coordinate>,
    /// The [`Figure`] whose pin the first point's value reproduces.
    pub first_figure: &'static str,
    /// And the one the last point's value reproduces.
    pub last_figure: &'static str,
    /// The worst departure from [`Traces::reference`].
    pub worst: Departure,
}

/// A subject the computation could not place, named rather than dropped.
pub struct Missing {
    /// What it is called, named unambiguously enough that a reader cannot mistake it for
    /// something drawn — see `origin_unreached` for the case that made that worth saying.
    pub label: String,
    /// Which input is missing, in words a reader of the picture can act on.
    pub why: String,
}

/// The shared inputs. Built once per process by [`Inputs::shared`], because the FY2027 panel and
/// the county abstract are read from fixtures and every regime-diff figure in [`FIGURES`] wants
/// the same two runs over them.
pub struct Inputs {
    /// The 609 districts of the FY2027 department model.
    pub panel: Vec<DistrictRecord>,
    /// The charge-off counterfactual against recognized valuation at TY2024 — the base the
    /// mechanism actually used, and the one the corpus was wrong about for fourteen phases.
    pub at_recognized: Vec<RegimeDiff>,
    /// The same counterfactual against total taxable value, which is what the corpus assumed.
    /// Kept because two of the figures in [`FIGURES`] are the size of that error.
    pub at_total_taxable: Vec<RegimeDiff>,
    /// Ohio's comparable districts quartiled by local revenue per pupil, FY2022, with what each
    /// level of government pays into each quartile.
    pub quartiles: [dispersion::national_peers::WealthQuartile; 4],
    /// What the community school equity supplement costs, and what the two rules it was measured
    /// against would have cost. The only figure block here computed over a population that is not
    /// the 609-district panel, and the only one no district appears in.
    pub community_school_equity: dispersion::community_school_funding::EquityCost,
    /// The same 355 schools split site-based, e-school and STEM — which is the split the
    /// supplement's eligibility rule makes and the money cannot show, since two of the three are
    /// zero.
    pub community_school_segments: Vec<dispersion::community_school_funding::Segment>,
    /// Total state support across those 355 schools, which is what the supplement is a share of.
    pub community_school_state_support: f64,
    /// The same block for FY2025, the other year of the rate schedule with a department model
    /// behind it. Held to verify a rate rather than to be differenced against FY2027 — the two
    /// years are two apart over a population that opened and closed schools in between.
    pub community_school_equity_fy2025: dispersion::community_school_funding::EquityCost,
    /// FY2025's 343 schools split site-based, e-school and STEM, which is how many schools the
    /// $650 was actually paid to.
    pub community_school_segments_fy2025: Vec<dispersion::community_school_funding::Segment>,
    /// Who belongs to each career-technical planning district, FY2025.
    ///
    /// The third population outside the 609-district panel in this manifest, and the only one
    /// that *contains* the panel: a planning district's members are the districts plus community
    /// and STEM schools.
    pub ctpd: dispersion::ctpd_membership::OutsideThePanel,
    /// Six years of the department's joint vocational district payment reports, FY2022-FY2027.
    ///
    /// The second population here outside the 609-district panel, and the only one the corpus
    /// holds across an amendment to its own method: H.B. 96 item 7 rewrote the JVSD state share
    /// of base cost, and FY2022-FY2025 sit under the law it replaced.
    pub jvsd: Vec<dispersion::jvsd_funding::District>,
    /// What item 7 moved in FY2026, the one year that is both under the amendment and closed.
    ///
    /// `None` would mean the fixture had stopped carrying a priceable FY2026, which is a fixture
    /// failure rather than a figure of zero.
    pub jvsd_item_7: Option<dispersion::jvsd_funding::Item7Incidence>,
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
    /// The guarantee scenario, every run of it, on both answers to what §265.225 does.
    pub guarantee: Guarantee,
    /// What the formula pays a district to do, at the margin.
    pub margins: Margins,
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
    /// The same refresh on the department's published worked-example district — the one the
    /// scenario's incidence table sweeps capacity across.
    pub refresh_worked_example: foundation::WorkedExampleRefresh,
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
    /// The guarantee's anchor replaced, at the modelled year and walked forward.
    ///
    /// Here for the same reason `forecasts` is: `formula-component/temporary-transitional-aid-guarantee`
    /// states what a rolling anchor costs, and a rule whose price is quoted in prose and computed
    /// nowhere the corpus can see is the #120 shape.
    pub anchors: Anchors,
    /// Who each anchor rule and each of #400's shapes reaches outside the enrollment cluster.
    ///
    /// Here because the same node states a gradient on every one of them, and a gradient quoted
    /// in prose from a walk nobody re-runs is the #120 shape twice over: it is a delta, and the
    /// measure it is taken on has already widened once under a bound level.
    pub reach: Reach,
    /// The two terms for every district, the wealth gradient among the ones that lost pupils, and
    /// what a fitted partition recovers — [`project::lost_pupils`], for #396.
    ///
    /// Here because `formula-component/temporary-transitional-aid-guarantee` states the gradient
    /// and the siblings' distance from the floor, and because the fitted partition's ceilings are
    /// the evidence for a decision record: a negative result quoted from a run nobody re-runs is
    /// a number that cannot go stale and cannot be checked either.
    pub lost_pupils: LostPupils,
    /// What the published band held at every horizon the panel reaches, both ways of asking.
    ///
    /// Here because the shape is the claim and the shape was a table in a doc comment. `/method`
    /// argued in prose that the band's width is right out to thirteen years while the level is
    /// not, and the evidence was thirteen pairs of percentages nothing recomputed — quoted into
    /// `scenario/guarantee-phase-out` and `.yidam/decisions/the-widening-rule.yml` from a test
    /// file, which is the #120 shape exactly. [`project::backtest::profile`] runs it once for the
    /// six pinned figures and the curve alike.
    pub coverage: Vec<project::backtest::Held>,
    /// The bias in the two published quantities, at every horizon, in both populations.
    ///
    /// Beside [`Self::coverage`] because it is the other half of the same question and the other
    /// half of the same backtest: the band is checked for its *width* there and for its *level*
    /// here, and a reader told the band holds what it claims is owed the second answer too. Four
    /// lines rather than two, because the mean district's bias and the whole state's are
    /// different numbers with different signs — see
    /// `.yidam/decisions/the-bias-published-beside-the-point.yml`. Run once, like the coverage
    /// profile, so the ten pins and the two curves are one computation.
    pub bias: Vec<project::backtest::Drift>,
    /// The three Category 3 lines that pay the chartered nonpublic schools outside the
    /// scholarships, and what the department's scholarship report puts beside them.
    ///
    /// Here because the two nodes for those lines are the corpus's only account of a quarter of a
    /// billion dollars a year, and a node with no `figures:` block has verified nothing. Computed
    /// once because every figure reading it takes one of four series off the same catalog
    /// fixture.
    pub nonpublic: Nonpublic,
}

/// Category 3 of the department's budget, as [`project::ledger::nonpublic_support`] reads it.
pub struct Nonpublic {
    /// GRF 200511 Auxiliary Services, newest vintage per year, deflated to FY2025 dollars.
    pub auxiliary: Vec<NonpublicYear>,
    /// GRF 200532 Nonpublic Administrative Cost Reimbursement, the same.
    pub administrative: Vec<NonpublicYear>,
    /// 5980 200659 Auxiliary Services Reimbursement — the mobile unit line, the same.
    pub reimbursement: Vec<NonpublicYear>,
    /// The three of them summed.
    pub category: Vec<NonpublicYear>,
    /// FY2020 to FY2025 on 200511, nominal and real. The window ends at FY2025 because the CPI
    /// series cannot reach FY2027 and a growth rate wants both endpoints on one footing.
    pub auxiliary_growth: (f64, f64),
    /// The same window on 200532.
    pub administrative_growth: (f64, f64),
    /// And on the category.
    pub category_growth: (f64, f64),
    /// What the four scholarship programmes that publish an expenditure paid in 2024-25.
    ///
    /// The denominator of the scale claim, and short by one programme: Jon Peterson's
    /// expenditure is not published, so the ratio built on this is an upper bound.
    pub scholarship_expenditure: f64,
}

/// One year of one line, as this manifest holds it.
pub type NonpublicYear = project::ledger::nonpublic_support::Year;

impl Nonpublic {
    /// Read the four series and the scholarship report.
    #[must_use]
    pub fn build() -> Self {
        use project::ledger::nonpublic_support as nonpublic;

        let base = edfund_core::FiscalYear(2025);
        let auxiliary = nonpublic::series(nonpublic::AUXILIARY_SERVICES, base);
        let administrative = nonpublic::series(nonpublic::ADMINISTRATIVE_COST_REIMBURSEMENT, base);
        let reimbursement = nonpublic::series(nonpublic::AUXILIARY_SERVICES_REIMBURSEMENT, base);
        let category = nonpublic::category_three(base);

        let window = |history: &[NonpublicYear]| -> (f64, f64) {
            let span: Vec<NonpublicYear> = history
                .iter()
                .filter(|year| (2020..=2025).contains(&year.fiscal_year))
                .cloned()
                .collect();
            nonpublic::growth(&span).expect("FY2020 and FY2025 both deflate")
        };

        Self {
            auxiliary_growth: window(&auxiliary),
            administrative_growth: window(&administrative),
            category_growth: window(&category),
            scholarship_expenditure: project::scholarship::report::programmes()
                .values()
                .filter_map(|programme| programme.expenditure)
                .sum(),
            auxiliary,
            administrative,
            reimbursement,
            category,
        }
    }

    /// The nominal amount one series carries for one year.
    ///
    /// Panics rather than returning an option: every year these figures ask for is inside the
    /// catalog's span, and a missing one is a fixture that stopped carrying a line rather than a
    /// figure of zero.
    #[must_use]
    pub fn nominal(history: &[NonpublicYear], fiscal_year: u16) -> f64 {
        project::ledger::nonpublic_support::at(history, fiscal_year)
            .unwrap_or_else(|| panic!("the catalog no longer carries FY{fiscal_year}"))
            .nominal
    }
}

/// What `project::lost_pupils` establishes, computed once over the 607 districts the identity
/// reaches.
pub struct LostPupils {
    /// Every district the identity reaches, in the order `project::lost_pupils` returned them.
    ///
    /// The rows themselves and not only what they aggregate to, because [`SPREADS`] draws one
    /// mark per district and the counts a figure takes off it are regions of that same picture.
    /// Computing the standings twice would be two populations that happen to agree today.
    pub standings: Vec<project::lost_pupils::Standing>,
    /// The districts it does not, carried rather than dropped: they are named on the drawing.
    pub unreached: Vec<project::lost_pupils::Unreached>,
    /// Formula districts the identity reaches.
    pub formula: usize,
    /// Of them, the ones with fewer pupils than in FY2020.
    pub formula_that_lost_pupils: usize,
    /// Of them, the ones the FY2027 formula pays more per pupil than the FY2020 regime did.
    pub formula_paid_more_per_pupil: usize,
    /// Every district that lost pupils, held or not, cut into fifths by capacity per pupil.
    pub lost_by_capacity: [project::lost_pupils::Fifth; 5],
    /// Every district that did not, the same cut.
    pub grew_by_capacity: [project::lost_pupils::Fifth; 5],
    /// The enrollment cluster, described on the axes the siblings are compared on.
    pub cluster: project::lost_pupils::Profile,
    /// The formula districts that lost pupils at the cluster's median rate or faster.
    pub siblings: project::lost_pupils::Siblings,
    /// How many of the siblings the enacted anchor holds by FY2032, at the shipped damping.
    pub siblings_held_fy2032: usize,
    /// How many formula districts it holds by then — the 18 that make the corpus's 312.
    pub formula_held_fy2032: usize,
    /// The fitted partition at every `k` from 2 to 9, from farthest-first seeds.
    pub fitted: Vec<project::lost_pupils::Fitted>,
    /// The typology scored as a partition.
    pub typology: project::lost_pupils::Ceilings,
    /// The fitted partition started from the typology's own centres.
    pub seeded: project::lost_pupils::Fitted,
    /// The nearest-neighbour question, nothing fitted.
    pub neighbours: project::lost_pupils::Neighbours,
    /// What the plan's two size-dependent terms do to the siblings — [`project::size_incidence`],
    /// for #435.
    ///
    /// Here rather than computed per figure because each row re-runs
    /// `size_terms::striking_out` over the whole panel, and because the same node states the
    /// counts, the per-pupil worth and the size prediction together: three numbers from one
    /// counterfactual, which is exactly the shape that goes stale in halves.
    pub size_terms: Vec<project::size_incidence::Row>,
    /// The newly guaranteed by ADM band, siblings against the rest, with both terms struck out.
    pub size_bands: [project::size_incidence::Band; project::size_terms::BANDS],
    /// What those bands predict for the siblings from size alone.
    pub expected_from_size: f64,
    /// The siblings the two terms leave off the floor, against each component of `[H]`.
    pub unmoved: Vec<project::size_incidence::Carried>,
    /// Their median base cost enrolled ADM, and the moved siblings' — the 29 are the larger ones.
    pub unmoved_median_adm: f64,
    /// The same for the 35 the terms move.
    pub moved_median_adm: f64,
    /// The cluster and the siblings unioned: the arc's "153".
    pub cluster_and_siblings: usize,
    /// Every district shrinking at the cluster's median rate or faster, which is a different
    /// population and holds 169.
    pub shrinking_at_the_cluster_rate: usize,
}

impl LostPupils {
    /// The most any of the fitted partitions scored on the cluster against the rest.
    #[must_use]
    pub fn best_cluster_ceiling(&self) -> usize {
        self.fitted
            .iter()
            .map(|f| f.ceilings.cluster)
            .chain(std::iter::once(self.seeded.ceilings.cluster))
            .max()
            .unwrap_or(0)
    }

    /// What one cell scores on the cluster: everyone who is not in it.
    #[must_use]
    pub fn trivial_cluster_ceiling(&self) -> usize {
        self.typology.districts - self.typology.cluster_members
    }

    /// The fitted partition at `k`.
    ///
    /// # Panics
    ///
    /// If `k` was not run.
    #[must_use]
    pub fn at(&self, k: usize) -> &project::lost_pupils::Fitted {
        self.fitted
            .iter()
            .find(|f| f.k == k)
            .unwrap_or_else(|| panic!("k = {k} was run"))
    }

    /// One fifth of the districts that lost pupils, 1 the least wealthy.
    #[must_use]
    pub fn lost_fifth(&self, rank: usize) -> &project::lost_pupils::Fifth {
        &self.lost_by_capacity[rank - 1]
    }
}

/// `rolling_anchor`'s runs, computed once because each walks all 609 districts for five years.
pub struct Anchors {
    /// FY2027 as enacted — the control, and the published guarantee.
    pub fy2027_as_enacted: project::rolling_anchor::Held,
    /// FY2027 with the floor set to each district's own FY2026 realized aid.
    pub fy2027_prior_year: project::rolling_anchor::Held,
    /// FY2032 as enacted, walked rather than computed in one step.
    pub fy2032_as_enacted: project::rolling_anchor::Held,
    /// FY2032 under a prior-year ratchet.
    pub fy2032_prior_year: project::rolling_anchor::Held,
    /// FY2032 with each district's annual fall capped at 2%.
    pub fy2032_capped: project::rolling_anchor::Held,
    /// The 89 of `enrollment_decline`'s cluster, at FY2032 as enacted.
    pub fy2032_cluster_as_enacted: project::rolling_anchor::Held,
    /// And under the 2% cap — the pair the absorption share is taken over.
    pub fy2032_cluster_capped: project::rolling_anchor::Held,
    /// The 89 at FY2036 undamped as enacted, which is where the held share is quoted.
    pub fy2036_cluster_as_enacted: project::rolling_anchor::Held,
    /// And under the 2% cap, which holds the same share on a different line.
    pub fy2036_cluster_capped: project::rolling_anchor::Held,
}

/// `anchor_incidence`'s frames, one per rule, each against the rule in force.
///
/// A frame is every district's movement in total state support; the figures cut it into fifths
/// on an axis at compute time, which is cheap, and leave the cluster out, which is the question.
pub struct Reach {
    /// The 89, as the set every cut leaves out.
    pub cluster: BTreeSet<String>,
    /// A prior-year ratchet against the enacted anchor, walked to FY2032 at the shipped damping.
    pub ratchet_fy2032: Vec<project::anchor_incidence::Movement>,
    /// A 2% cap on the annual fall against the enacted anchor, the same walk.
    pub capped_fy2032: Vec<project::anchor_incidence::Movement>,
    /// A 1% cap against the enacted anchor, walked to FY2036 undamped — where the sign flips.
    pub gentle_cap_fy2036: Vec<project::anchor_incidence::Movement>,
    /// The rolling pupil count against current law, at the modelled year.
    pub rolling_count: Vec<project::anchor_incidence::Movement>,
    /// `[M]` mirrored inside `[H]`.
    pub mirror_inside: Vec<project::anchor_incidence::Movement>,
    /// `[M]` mirrored beside `[L]`, `[M]` and `[O]`.
    pub mirror_beside: Vec<project::anchor_incidence::Movement>,
    /// The FY2027 guarantee list rolled down on a date -- the shape keyed to the floor.
    pub dated_phase_down: Vec<project::anchor_incidence::Movement>,
    /// The 64 siblings, the second population every row of #436 is read over.
    pub siblings: BTreeSet<String>,
    /// The cluster and the siblings unioned: the arc's 153.
    pub union: BTreeSet<String>,
    /// Districts shrinking at the cluster's median rate or faster: a different 169.
    pub shrinking: BTreeSet<String>,
    /// The mirror inside `[H]`, gross against what arrives -- the siblings, then the cluster.
    pub absorbed_inside: (
        project::decline_reach::Absorption,
        project::decline_reach::Absorption,
    ),
    /// The siblings split at the median years-to-the-floor, nearest half first, inside `[H]`.
    ///
    /// Here rather than per figure because each half re-prices the whole panel, and because the
    /// pair only means anything together: one half's absorbed share against the other's is the
    /// measurement, and a figure holding one alone would read as a level.
    pub absorbed_by_headroom: (
        project::decline_reach::Absorption,
        project::decline_reach::Absorption,
    ),
    /// The siblings' clock to their own guarantee floors.
    pub sibling_clock: project::decline_reach::Clock,
    /// Siblings on the floor by FY2036, at the shipped damping and undamped.
    pub siblings_on_the_floor_fy2036: (usize, usize),
}

impl Reach {
    /// One fifth of one frame on one axis, outside the cluster. `rank` is 1 to 5, lowest first.
    fn fifth(
        &self,
        frame: &[project::anchor_incidence::Movement],
        axis: project::anchor_incidence::Axis,
        rank: usize,
    ) -> project::anchor_incidence::Band {
        project::anchor_incidence::by(frame, axis, 5, &self.cluster)
            .into_iter()
            .find(|band| band.rank == rank)
            .expect("five bands, ranks 1 to 5")
    }

    /// One frame summed over one population -- the 64, the 89, the 153 or the 169.
    fn on(
        &self,
        frame: &[project::anchor_incidence::Movement],
        population: &BTreeSet<String>,
    ) -> project::decline_reach::Reach {
        project::decline_reach::over(frame, population)
    }

    /// One band of one frame cut into the siblings and everyone else.
    fn split(
        &self,
        frame: &[project::anchor_incidence::Movement],
        axis: project::anchor_incidence::Axis,
        rank: usize,
    ) -> project::decline_reach::Split {
        project::decline_reach::split_by(frame, axis, 5, &self.cluster, &self.siblings)
            .into_iter()
            .find(|split| split.rank == rank)
            .expect("five bands, ranks 1 to 5")
    }

    /// The spread of one frame outside the cluster.
    fn spread(
        &self,
        frame: &[project::anchor_incidence::Movement],
    ) -> project::anchor_incidence::Spread {
        project::anchor_incidence::spread(frame, &self.cluster)
    }

    /// One frame totalled on both axes at once, outside the cluster.
    fn aggregate(
        &self,
        frame: &[project::anchor_incidence::Movement],
    ) -> project::anchor_incidence::Aggregate {
        project::anchor_incidence::aggregate(frame, &self.cluster)
    }

    /// The seven rules, in the order `who_a_capped_or_rolling_anchor_reaches` compares them.
    ///
    /// Each label carries the frame it was priced in, because they were not all priced at one
    /// horizon: three are anchor rules walked to a year and four are #400's supplements at the
    /// modelled year. A drawing that hid that would be comparing four things and calling them
    /// seven. The crate's own test already asserts a direction across exactly this list, which
    /// is the precedent for treating them as one comparable set.
    fn rules(&self) -> [(&'static str, &[project::anchor_incidence::Movement]); 7] {
        [
            ("A ratchet, FY2032", &self.ratchet_fy2032),
            ("A 2% cap, FY2032", &self.capped_fy2032),
            ("A 1% cap, FY2036 undamped", &self.gentle_cap_fy2036),
            ("A rolling pupil count", &self.rolling_count),
            ("[M] mirrored inside [H]", &self.mirror_inside),
            ("[M] mirrored beside [L], [M], [O]", &self.mirror_beside),
            ("A dated phase-down", &self.dated_phase_down),
        ]
    }
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
    /// FY2032 with the guarantee removed **and** §265.225 repealed beside it.
    ///
    /// The run that recovers the doubling. On foundation aid it is indistinguishable from the one
    /// above — `[K]` is outside that measure — and on total state support the two are 4.16% and
    /// 7.77%, which is the whole of why this third forecast exists.
    pub fy2032_guarantee_and_backstop_removed: project::report::EnrollmentEffect,
    /// FY2032 with both of the per-pupil terms `policy::apply` freezes recomputed at the
    /// projected count, which is the other end of the interval a projected local share sits in.
    pub fy2032_two_terms_recomputed: project::report::EnrollmentEffect,
    /// The same, with R.C. 3317.011's staffing floors recomputed and the wealth denominator left
    /// frozen. The smaller term, and the one it pays more aid for.
    pub fy2032_base_cost_recomputed: project::report::EnrollmentEffect,
    /// And with the wealth denominator alone, which is the term that carries the sign.
    pub fy2032_capacity_recomputed: project::report::EnrollmentEffect,
    /// Every district's movement between the linear run and the recomputed one, at FY2032.
    pub fy2032_movements: Vec<project::projected_base_cost::Movement>,
    /// Realized state aid at **observed** enrollment, which is what every projected total above
    /// is a movement from. Carried here rather than recomputed at each call site so that a
    /// forecast's effect and a correction to it are differences against one number.
    pub observed_realized_aid: edfund_core::Dollars,
}

/// `scenario/guarantee-phase-out`, every run of it.
///
/// # Why each guarantee rule is priced twice
///
/// `[I]` is a term in `[K]`'s own subtrahend, so uncodified §265.225 **backstops the guarantee**
/// and decides what retiring it costs. Left standing it absorbs 90.9% of the apparent saving; the
/// same rule therefore prices an order of magnitude apart depending on an answer the lever does not
/// contain. The node states both columns, so this block holds both.
///
/// Every run is [`project::report::simulate`] at observed enrollment against the department's own
/// FY2027 model, which `Policy::current_law` reproduces to the cent.
pub struct Guarantee {
    /// Removal, §265.225 standing — the honest model of current law.
    pub removal: project::report::PolicyEffect,
    /// Removal, §265.225 repealed alongside — what "retiring the guarantee" means as a policy.
    pub removal_and_backstop: project::report::PolicyEffect,
    /// A half phase-out, §265.225 standing.
    pub half: project::report::PolicyEffect,
    /// A half phase-out, §265.225 repealed alongside.
    pub half_and_backstop: project::report::PolicyEffect,
    /// A rebase of the floor to 90%, §265.225 standing.
    pub rebase: project::report::PolicyEffect,
    /// A rebase of the floor to 90%, §265.225 repealed alongside.
    pub rebase_and_backstop: project::report::PolicyEffect,
    /// Removal, §265.225 standing but measured against a base with no guarantee inside it.
    ///
    /// The third reading, and the one the node carried as `[open]` until the guarantee inside
    /// `[L1]` could be split out of two other published files — see `project::transition_base`.
    pub removal_and_a_guarantee_free_base: project::report::PolicyEffect,
    /// A half phase-out, on the same guarantee-free base.
    pub half_and_a_guarantee_free_base: project::report::PolicyEffect,
    /// A rebase of the floor to 90%, on the same guarantee-free base.
    pub rebase_and_a_guarantee_free_base: project::report::PolicyEffect,
    /// The guarantee-free base with the guarantee itself left entirely alone.
    ///
    /// Not a null run: `[K]` holds a district at a FY2021 total that contains a guarantee it is
    /// also still being paid, so taking the guarantee out of the base cuts the supplement before
    /// any bill touches `[I]`.
    pub a_guarantee_free_base_alone: project::report::PolicyEffect,
    /// What the previous formula's guarantee contributes to `[L1]`, across the 611 districts the
    /// department publishes a base for.
    pub guarantee_inside_the_base: f64,
    /// Whom a guarantee-only retirement reaches, and whom the backstop makes whole.
    ///
    /// Measured on total state support rather than on `scenario-delta`'s table, which is built from
    /// realized aid and therefore cannot see the backstop at all.
    pub retirement: project::hold_harmless::Retirement,
    /// The base cost sweep the same node states, at +2%, +5%, +10% and +20%.
    pub base_cost: [project::report::PolicyEffect; 4],
}

impl Guarantee {
    /// Marginal cost per point of base cost increase, between two of the four runs.
    ///
    /// The node's fourth column, and it is genuinely marginal rather than average — a difference of
    /// two runs over the points between them. `from` of `None` is the increase from current law.
    #[must_use]
    pub fn marginal_per_point(&self, from: Option<usize>, to: usize, points: f64) -> f64 {
        let at = |index: Option<usize>| index.map_or(0.0, |i| self.base_cost[i].cost());
        (at(Some(to)) - at(from)) / points
    }
}

/// What the formula pays a district **to do**, from [`project::margin`].
///
/// Built once because every member walks the 609-district panel at least twice — a margin is the
/// difference between two runs of `policy::apply` — and ten figures read them.
///
/// The one thing to hold while reading these: three of the margins are priced at zero or at
/// transportation alone, and they are the ones no district chooses. `project::margin`'s own docs
/// carry the ordering.
pub struct Margins {
    /// One pupil fewer, per district, at current law.
    pub pupil: Vec<project::margin::Pupil>,
    /// Every district's enrolment down one per cent at once, which is where the floors show.
    pub statewide: project::margin::Aggregate,
    /// Each formula district's distance to its own guarantee floor.
    pub headroom: Vec<project::margin::Headroom>,
    /// The districts short of `[M]`'s threshold, and what clearing it would pay.
    pub cliff: Vec<project::margin::Cliff>,
    /// The clawback margin, for every district `[I1]` is charged against.
    pub clawback: Vec<project::margin::OpenEnrolment>,
    /// The charge on a dollar of assessed valuation, where no floor already zeroes it.
    pub wealth: Vec<project::margin::Wealth>,
}

impl Margins {
    /// How many districts are in one marginal regime.
    #[must_use]
    pub fn districts(&self, response: project::margin::Response) -> f64 {
        self.pupil.iter().filter(|p| p.response == response).count() as f64
    }

    /// The median marginal pupil in one regime, on total state support.
    #[must_use]
    pub fn median_marginal(&self, response: project::margin::Response) -> f64 {
        median(
            self.pupil
                .iter()
                .filter(|p| p.response == response)
                .map(|p| p.marginal)
                .collect(),
        )
    }

    /// Districts that lose no foundation aid at all when a pupil leaves.
    ///
    /// The guaranteed ones, both regimes together — the guarantee holds `[H] + [I]` level, and it is
    /// `[J]`, outside the measure, that separates the two.
    #[must_use]
    pub fn no_foundation_aid(&self) -> f64 {
        self.pupil
            .iter()
            .filter(|p| p.foundation.abs() < project::margin::CENT)
            .count() as f64
    }

    /// Formula districts whose own enrolment trend reaches their floor inside a year.
    #[must_use]
    pub fn within_a_year(&self) -> f64 {
        self.headroom
            .iter()
            .filter(|h| h.years.is_some_and(|years| years <= 1.0))
            .count() as f64
    }

    /// What the pupil at the top of `[M]`'s cliff is worth.
    ///
    /// The supplement pays on the whole roll and tests on the increment, so the district closest to
    /// the threshold carries every dollar the other pupils do not.
    #[must_use]
    pub fn steepest_cliff(&self) -> f64 {
        self.cliff
            .iter()
            .map(project::margin::Cliff::per_pupil)
            .fold(0.0, f64::max)
    }

    /// Districts for which one open-enrolment FTE is worth the whole statewide average base cost.
    ///
    /// The guaranteed districts `[I1]` reaches that `[K]` does not backstop.
    #[must_use]
    pub fn clawback_exposed(&self) -> f64 {
        self.clawback
            .iter()
            .filter(|m| m.marginal.abs() > project::margin::CENT)
            .count() as f64
    }

    /// Districts charged for valuation that H.B. 920 lets them collect nothing on.
    #[must_use]
    pub fn charged_for_nothing(&self) -> f64 {
        self.wealth
            .iter()
            .filter(|w| !w.status.valuation_growth_reaches_revenue())
            .count() as f64
    }

    /// The median share of a new valuation dollar's local yield the state takes back.
    ///
    /// Over the districts at the twenty-mill floor, which are the only ones the dollar yields
    /// anything to at all.
    #[must_use]
    pub fn recapture(&self) -> f64 {
        median(
            self.wealth
                .iter()
                .filter_map(project::margin::Wealth::recapture)
                .collect(),
        )
    }
}

impl Inputs {
    /// The one build of this process, read the first time a registry is run and borrowed after.
    ///
    /// Every `compute_all_*` entry point wants the whole struct, and there are seven of them;
    /// `json::manifest` and `json::series_manifest` between them call all seven, so a
    /// per-entry-point build ran `Inputs::build` — two full `project` regime runs, a thirteen
    /// origin backtest and `project::outcomes::joined()` — seven times to produce one manifest.
    /// Nothing about the result can differ between builds: every input is a committed fixture or
    /// a compile-time constant, so the second run could only ever reproduce the first. What it
    /// could do is take four minutes.
    ///
    /// Borrowed rather than cloned. The struct is a few megabytes of panels and standings and
    /// every consumer is a `fn(&Inputs)`, so there is nothing for a clone to buy.
    #[must_use]
    pub fn shared() -> &'static Self {
        static INPUTS: OnceLock<Inputs> = OnceLock::new();
        INPUTS.get_or_init(Self::build)
    }

    /// Read the fixtures and run both counterfactuals.
    fn build() -> Self {
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
        let panel_for_guarantee = panel.clone();
        let panel_for_margins = panel.clone();
        // Eight origins over 602 complete histories, thirteen times: about two seconds in debug,
        // and run here rather than per figure so the six pins and the curve are one backtest.
        let coverage = project::backtest::profile(
            project::backtest::DEEPEST_HORIZON,
            *project::backtest::PANEL_YEARS
                .last()
                .expect("the panel carries years"),
        );
        let bias = project::backtest::bias_profile(
            project::backtest::DEEPEST_HORIZON,
            *project::backtest::PANEL_YEARS
                .last()
                .expect("the panel carries years"),
        );
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
        let (anchors, reach, fy2032_enacted) = {
            use project::anchor_incidence::{self, Supplement};
            use project::decline_adjustment::Line;
            use project::rolling_anchor::{self, Anchor, Observed};
            // `shrunk` is local to the `forecasts` block below; the same construction, so that a
            // walk and a forecast are the same projection at the same parameters.
            let shrunk = |damping: f64| project::series::Method::Shrunk {
                rate: 0.0,
                damping,
                weight: project::series::DEFAULT_SHRINK_WEIGHT,
                toward: 0.0,
            };
            let cluster: BTreeSet<String> =
                project::enrollment_decline::cluster(&panel_for_forecasts)
                    .into_iter()
                    .map(|record| record.irn.clone())
                    .collect();
            // Every walk is kept as its standings, because the incidence frames are joins of two
            // of them and a walk is the expensive half of a figure here.
            let walk = |through: u16, damping: f64, anchor: Anchor| {
                rolling_anchor::walk(
                    &panel_for_forecasts,
                    FiscalYear(through),
                    shrunk(damping),
                    anchor,
                )
            };
            let summarize =
                |through: u16, standings: &BTreeMap<String, rolling_anchor::Standing>| {
                    let year = FiscalYear(through);
                    (
                        rolling_anchor::summarize(year, standings.values()),
                        rolling_anchor::summarize(
                            year,
                            standings.values().filter(|s| cluster.contains(&s.irn)),
                        ),
                    )
                };
            let capped = Anchor::Decayed { factor: 0.98 };
            let damping = project::series::DEFAULT_DAMPING;
            let fy2032_enacted = walk(2032, damping, Anchor::Fixed);
            let fy2032_ratchet = walk(2032, damping, Anchor::PriorYear);
            let fy2032_capped = walk(2032, damping, capped);
            let fy2036_enacted = walk(2036, 1.0, Anchor::Fixed);
            let fy2036_capped = walk(2036, 1.0, capped);
            let fy2036_gentle = walk(2036, 1.0, Anchor::Decayed { factor: 0.99 });
            let (fy2032_as_enacted, fy2032_cluster_as_enacted) = summarize(2032, &fy2032_enacted);
            let (fy2032_prior_year, _) = summarize(2032, &fy2032_ratchet);
            let (fy2032_capped_all, fy2032_cluster_capped) = summarize(2032, &fy2032_capped);
            let (_, fy2036_cluster_as_enacted) = summarize(2036, &fy2036_enacted);
            let (_, fy2036_cluster_capped) = summarize(2036, &fy2036_capped);
            let between =
                |enacted: &BTreeMap<String, rolling_anchor::Standing>,
                 alternative: &BTreeMap<String, rolling_anchor::Standing>| {
                    anchor_incidence::between_walks(&panel_for_forecasts, enacted, alternative)
                };
            let under =
                |shape: Supplement| anchor_incidence::under_supplement(&panel_for_forecasts, shape);
            // #436 reads every frame above over a second population as well. The standings are
            // recomputed here rather than shared with the `lost_pupils` block below because a
            // `Reach` that borrowed them would tie two blocks' lifetimes together for one set of
            // IRNs.
            let (siblings, union, shrinking) = {
                let (rows, _) = project::lost_pupils::standings(&panel_for_forecasts);
                (
                    project::decline_reach::irns(&project::lost_pupils::siblings(&rows)),
                    project::decline_reach::irns(&project::size_incidence::cluster_and_siblings(
                        &rows,
                    )),
                    project::decline_reach::irns(
                        &project::size_incidence::shrinking_at_the_cluster_rate(&rows),
                    ),
                )
            };
            (
                Anchors {
                    fy2027_as_enacted: rolling_anchor::at_the_modelled_year(
                        &panel_for_forecasts,
                        Observed::Fy2020Base,
                    ),
                    fy2027_prior_year: rolling_anchor::at_the_modelled_year(
                        &panel_for_forecasts,
                        Observed::PriorYearRealized,
                    ),
                    fy2032_as_enacted,
                    fy2032_prior_year,
                    fy2032_capped: fy2032_capped_all,
                    fy2032_cluster_as_enacted,
                    fy2032_cluster_capped,
                    fy2036_cluster_as_enacted,
                    fy2036_cluster_capped,
                },
                Reach {
                    ratchet_fy2032: between(&fy2032_enacted, &fy2032_ratchet),
                    capped_fy2032: between(&fy2032_enacted, &fy2032_capped),
                    gentle_cap_fy2036: between(&fy2036_enacted, &fy2036_gentle),
                    rolling_count: under(Supplement::RollingCount),
                    mirror_inside: under(Supplement::Mirror(Line::Foundation)),
                    mirror_beside: under(Supplement::Mirror(Line::Beside)),
                    dated_phase_down: under(Supplement::DatedPhaseDown),
                    absorbed_inside: (
                        project::decline_reach::absorption(
                            &panel_for_forecasts,
                            Line::Foundation,
                            &siblings,
                        ),
                        project::decline_reach::absorption(
                            &panel_for_forecasts,
                            Line::Foundation,
                            &cluster,
                        ),
                    ),
                    absorbed_by_headroom: project::decline_reach::by_headroom(
                        &panel_for_forecasts,
                        Line::Foundation,
                        &siblings,
                    ),
                    sibling_clock: project::decline_reach::clock(&panel_for_forecasts, &siblings),
                    siblings_on_the_floor_fy2036: (
                        project::decline_reach::on_the_floor(
                            &panel_for_forecasts,
                            edfund_core::FiscalYear(2036),
                            project::series::Method::Shrunk {
                                rate: 0.0,
                                damping,
                                weight: project::series::DEFAULT_SHRINK_WEIGHT,
                                toward: 0.0,
                            },
                            &siblings,
                        ),
                        project::decline_reach::on_the_floor(
                            &panel_for_forecasts,
                            edfund_core::FiscalYear(2036),
                            project::series::Method::Shrunk {
                                rate: 0.0,
                                damping: 1.0,
                                weight: project::series::DEFAULT_SHRINK_WEIGHT,
                                toward: 0.0,
                            },
                            &siblings,
                        ),
                    ),
                    siblings,
                    union,
                    shrinking,
                    cluster,
                },
                fy2032_enacted,
            )
        };
        let lost_pupils = {
            use project::lost_pupils::{self, Population};
            let (rows, unreached) = lost_pupils::standings(&panel_for_forecasts);
            let formula: Vec<&lost_pupils::Standing> = rows
                .iter()
                .filter(|s| s.population == Population::Formula)
                .collect();
            let siblings = lost_pupils::siblings(&rows);
            // Both terms struck out, computed once: every reader of it re-runs the whole panel.
            let both = [
                project::size_terms::Change::StaffingFloorsStruckOut,
                project::size_terms::Change::CapacityTierStruckOut,
            ];
            let size_bands =
                project::size_incidence::by_sextile(&panel_for_forecasts, &rows, &both);
            let unmoved = project::size_incidence::unmoved(&panel_for_forecasts, &rows, &both);
            let left_behind: std::collections::BTreeSet<String> =
                unmoved.iter().map(|s| s.irn.clone()).collect();
            let held_by_fy2032 = |set: &[&lost_pupils::Standing]| {
                set.iter()
                    .filter(|s| fy2032_enacted.get(&s.irn).is_some_and(|w| w.on_the_floor()))
                    .count()
            };
            LostPupils {
                standings: rows.clone(),
                unreached,
                formula: formula.len(),
                formula_that_lost_pupils: formula.iter().filter(|s| s.lost_pupils()).count(),
                formula_paid_more_per_pupil: formula
                    .iter()
                    .filter(|s| s.terms.per_pupil_term < 0.0)
                    .count(),
                lost_by_capacity: lost_pupils::by_capacity(&lost_pupils::who(&rows, true)),
                grew_by_capacity: lost_pupils::by_capacity(&lost_pupils::who(&rows, false)),
                cluster: lost_pupils::profile(
                    &rows
                        .iter()
                        .filter(|s| s.population == Population::EnrollmentLoss)
                        .collect::<Vec<_>>(),
                ),
                siblings_held_fy2032: held_by_fy2032(&siblings),
                formula_held_fy2032: held_by_fy2032(&formula),
                siblings: lost_pupils::siblings_headroom(&panel_for_forecasts, &siblings),
                fitted: (2..=9).map(|k| lost_pupils::fitted(&rows, k)).collect(),
                typology: lost_pupils::typology_ceilings(&rows),
                seeded: lost_pupils::fitted_from_typology(&rows),
                neighbours: lost_pupils::neighbours(&rows),
                size_terms: project::size_incidence::table(&panel_for_forecasts, &rows),
                size_bands,
                expected_from_size: project::size_incidence::expected_from_size(&size_bands),
                unmoved: project::size_incidence::carried(&panel_for_forecasts, &unmoved),
                unmoved_median_adm: median(unmoved.iter().map(|s| s.adm).collect()),
                moved_median_adm: median(
                    siblings
                        .iter()
                        .filter(|s| !left_behind.contains(&s.irn))
                        .map(|s| s.adm)
                        .collect(),
                ),
                cluster_and_siblings: project::size_incidence::cluster_and_siblings(&rows).len(),
                shrinking_at_the_cluster_rate:
                    project::size_incidence::shrinking_at_the_cluster_rate(&rows).len(),
            }
        };
        Self {
            panel,
            at_recognized,
            at_total_taxable,
            quartiles: dispersion::national_peers::ohio_by_local_wealth(),
            community_school_equity: dispersion::community_school_funding::equity_cost(
                dispersion::community_school_funding::Vintage::CURRENT,
            ),
            community_school_segments: dispersion::community_school_funding::segments(
                dispersion::community_school_funding::Vintage::CURRENT,
            ),
            ctpd: dispersion::ctpd_membership::outside_the_panel(),
            jvsd: dispersion::jvsd_funding::districts(),
            jvsd_item_7: dispersion::jvsd_funding::item_7_incidence(
                dispersion::jvsd_funding::FIRST_ITEM_7_YEAR,
            ),
            community_school_state_support:
                dispersion::community_school_funding::total_state_support(
                    dispersion::community_school_funding::Vintage::CURRENT,
                ),
            community_school_equity_fy2025: dispersion::community_school_funding::equity_cost(
                dispersion::community_school_funding::Vintage::Fy2025,
            ),
            community_school_segments_fy2025: dispersion::community_school_funding::segments(
                dispersion::community_school_funding::Vintage::Fy2025,
            ),
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
            guarantee: {
                use project::policy::{Backstop, GuaranteeRule};
                let at = |guarantee: GuaranteeRule, backstop: Backstop| {
                    project::report::simulate(
                        &panel_for_guarantee,
                        &project::policy::Policy {
                            guarantee,
                            backstop,
                            ..project::policy::Policy::current_law()
                        },
                    )
                };
                let scaled = |base_cost_scale: f64| {
                    project::report::simulate(
                        &panel_for_guarantee,
                        &project::policy::Policy {
                            base_cost_scale,
                            ..project::policy::Policy::current_law()
                        },
                    )
                };
                let half = GuaranteeRule::PhasedOut { remaining: 0.5 };
                let rebase = GuaranteeRule::Rebased { factor: 0.9 };
                Guarantee {
                    removal: at(GuaranteeRule::Removed, Backstop::AsEnacted),
                    removal_and_backstop: at(GuaranteeRule::Removed, Backstop::Repealed),
                    half: at(half, Backstop::AsEnacted),
                    half_and_backstop: at(half, Backstop::Repealed),
                    rebase: at(rebase, Backstop::AsEnacted),
                    rebase_and_backstop: at(rebase, Backstop::Repealed),
                    removal_and_a_guarantee_free_base: at(
                        GuaranteeRule::Removed,
                        Backstop::Rebased,
                    ),
                    half_and_a_guarantee_free_base: at(half, Backstop::Rebased),
                    rebase_and_a_guarantee_free_base: at(rebase, Backstop::Rebased),
                    a_guarantee_free_base_alone: project::report::simulate(
                        &panel_for_guarantee,
                        &project::policy::Policy {
                            backstop: Backstop::Rebased,
                            ..project::policy::Policy::current_law()
                        },
                    ),
                    guarantee_inside_the_base: project::transition_base::guarantee_in_fy21_base()
                        .values()
                        .sum(),
                    retirement: project::hold_harmless::retirement(
                        &panel_for_guarantee,
                        GuaranteeRule::Removed,
                    ),
                    base_cost: [scaled(1.02), scaled(1.05), scaled(1.10), scaled(1.20)],
                }
            },
            margins: {
                let law = project::policy::Policy::current_law();
                Margins {
                    pupil: project::margin::pupil(&panel_for_margins, &law),
                    statewide: project::margin::statewide(
                        &panel_for_margins,
                        &law,
                        project::margin::SHOCK,
                    ),
                    headroom: project::margin::headroom(&panel_for_margins, &law),
                    cliff: project::margin::growth_cliff(&panel_for_margins),
                    clawback: project::margin::open_enrolment(&panel_for_margins, &law),
                    wealth: project::margin::wealth(&panel_for_margins),
                }
            },
            actual_total,
            recognized_total,
            // Computed once here rather than per figure: each of the three walks the whole
            // ten-year panel and deflates it, and fourteen figures below read them.
            transfers: project::transfers::by_year(),
            nonpublic: Nonpublic::build(),
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
            refresh_worked_example: foundation::refresh_worked_example(),
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
                // The same run with the per-pupil terms `apply` freezes recomputed at the
                // projected count. `Terms::NEITHER` would reproduce `fy2032` exactly, which is
                // what `what_a_linear_projection_holds_per_pupil` asserts, so it is not exported.
                let recomputed =
                    |panel: &[project::panel::DistrictRecord],
                     prior: project::series::Prior,
                     terms: project::projected_base_cost::Terms| {
                        project::projected_base_cost::forecast(
                            panel,
                            &project::policy::Policy::current_law(),
                            FiscalYear(2032),
                            shrunk(project::series::DEFAULT_DAMPING),
                            prior,
                            terms,
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
                    fy2032_guarantee_and_backstop_removed: project::report::forecast(
                        &panel_for_forecasts,
                        &project::policy::Policy {
                            guarantee: project::policy::GuaranteeRule::Removed,
                            backstop: project::policy::Backstop::Repealed,
                            ..project::policy::Policy::current_law()
                        },
                        FiscalYear(2032),
                        shrunk(project::series::DEFAULT_DAMPING),
                        prior,
                    ),
                    fy2032_two_terms_recomputed: recomputed(
                        &panel_for_forecasts,
                        prior,
                        project::projected_base_cost::Terms::BOTH,
                    ),
                    fy2032_base_cost_recomputed: recomputed(
                        &panel_for_forecasts,
                        prior,
                        project::projected_base_cost::Terms::BASE_COST,
                    ),
                    fy2032_capacity_recomputed: recomputed(
                        &panel_for_forecasts,
                        prior,
                        project::projected_base_cost::Terms::CAPACITY,
                    ),
                    fy2032_movements: project::projected_base_cost::movements(
                        &panel_for_forecasts,
                        &project::policy::Policy::current_law(),
                        FiscalYear(2032),
                        shrunk(project::series::DEFAULT_DAMPING),
                        prior,
                    ),
                    observed_realized_aid: project::report::Totals::of(
                        &project::policy::apply_all(
                            &panel_for_forecasts,
                            &project::policy::Policy::current_law(),
                        ),
                    )
                    .realized_aid,
                }
            },
            anchors,
            reach,
            lost_pupils,
            coverage,
            bias,
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

/// The correlation of the FY2016 step with one district property share, signed.
///
/// Shared by the four `fy2016-step-against-*` figures and the series that draws them, so the
/// bars and the numbers under them are one computation rather than two that agree today.
fn fy2016_step_against(i: &Inputs, share: fn(&dispersion::fy2016::District) -> f64) -> f64 {
    let steps: Vec<f64> = i.fy2016.districts.iter().map(|d| d.step).collect();
    let against: Vec<f64> = i.fy2016.districts.iter().map(share).collect();
    dispersion::wealth_neutrality(&against, &steps)
        .expect("paired")
        .correlation
}

/// Designated buildings in one edition, and every building that edition lists.
fn edchoice_edition_counts(which: dispersion::designated_editions::Edition) -> (usize, usize) {
    let rows = dispersion::designated_editions::edition(which);
    (rows.iter().filter(|d| d.designated).count(), rows.len())
}

/// The census's rows, ranked by how many districts each bound is the operative term for.
///
/// Descending, and stably, so a tie keeps the statutory order [`project::bounds::Bound::all`]
/// returns — which is the order the corpus's table is written in, and the order a reader who
/// went looking for one of these bounds in the Revised Code would find them.
fn census_ranked(i: &Inputs) -> Vec<project::bounds::Row> {
    let mut rows = project::bounds::census(&i.panel);
    rows.sort_by_key(|row| core::cmp::Reverse(row.operative));
    rows
}

/// The median of a sample. Sorted here rather than by the caller, because every call site wants
/// the same thing and one of them getting the sort wrong would move a figure quietly.
fn median(mut sample: Vec<f64>) -> f64 {
    sample.sort_by(f64::total_cmp);
    sample[sample.len() / 2]
}

/// Every district carrying an open-enrolment clawback, as `(count, total, largest, second)`.
/// The open-enrolment clawback, on both of the quantities it has.
///
/// # Why two and not one
///
/// Because a district can be *charged* the adjustment without *losing* anything to it.
/// `[I] = max([H2] − [I1] − [H], 0)` clamps at zero, so an adjustment that meets a district with
/// no guarantee takes nothing: **43 are charged and 22 lose**, and the two totals differ by $2.07
/// million. A single figure reported as "withheld" was the wider one, which is the incidence of
/// the mechanism rather than its cost.
///
/// `largest` and `second` are the largest amounts actually withheld, not the largest charged. On
/// this panel they are the same two districts either way — Columbus City and Cuyahoga Falls City
/// are both on the guarantee and both lose the whole charge — but the figures are labelled as
/// reductions, so they are computed as reductions.
struct Clawback {
    charged: usize,
    adjustment: f64,
    reduced: usize,
    withheld: f64,
    largest: f64,
    second: f64,
}

fn clawback(panel: &[DistrictRecord]) -> Clawback {
    let charged: Vec<&DistrictRecord> = panel
        .iter()
        .filter(|r| r.transition.open_enrollment_adjustment > 0.0)
        .collect();
    let mut taken: Vec<f64> = charged
        .iter()
        .map(|r| r.open_enrollment_withheld())
        .filter(|w| *w > 0.005)
        .collect();
    taken.sort_by(|a, b| b.partial_cmp(a).expect("no NaN in a clawback"));
    let at = |i: usize| taken.get(i).copied().unwrap_or(0.0);
    Clawback {
        charged: charged.len(),
        adjustment: charged
            .iter()
            .map(|r| r.transition.open_enrollment_adjustment)
            .sum(),
        reduced: taken.len(),
        withheld: taken.iter().sum(),
        largest: at(0),
        second: at(1),
    }
}

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
