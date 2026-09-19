//! Every device that holds an Ohio district above what the FY2027 formula computes for it.
//!
//! The corpus has treated the Fair School Funding Plan as having **one** hold-harmless — the
//! temporary transitional aid guarantee, `[I]`, $879.0m across 294 districts — and has stated
//! shares against it as though that were the whole of the money. There are **four** devices, and
//! the guarantee is 90.9% of them.
//!
//! | Device | Line | Base | Anchor | Dollars | Districts | Placement | `policy::apply` |
//! |---|---|---|--:|--:|--:|---|---|
//! | Temporary transitional aid guarantee | `[I]` | `[H2]` | FY2020 | $878,954,627 | 294 | inside foundation aid | recomputed |
//! | Formula transition supplement | `[K]` | `[L1]` | FY2021 | $63,578,629 | 144 | **outside foundation aid** | **omitted** |
//! | Transitional transportation aid | `[F]` | `[F1]` | FY2020 | $24,780,627 | 38 | inside `[J]` transportation | passed through |
//! | Open-enrolment clawback | `[I1]` | — | prior year | −$5,110,050 | 43 | reduces `[I]` | passed through |
//! | The phase-in | — | `[H2]`/`[H3]` | FY2020/FY2019 | $0 | 0 | inside `[H]` | recomputed |
//!
//! Combined, the three that pay are **$967.3m to 326 districts, 58.0% of ADM** — against the
//! $879.0m and 294 the corpus has been dividing by.
//!
//! # The decisive question was the wrong boundary
//!
//! [`crate::supplement_reach`] settled the performance supplement `[O]` by asking whether it sits
//! inside `[H] Foundation Funding`, and that is the right question for a payment the guarantee
//! might absorb. It is the wrong question here, because **`[I]` is outside `[H]` too**: the
//! department's `[H]` is `[A]`–`[G]`, the seven phased-in components, and the guarantee is a
//! separate line added at `[N] Total Formula Funding`. The boundary that decides what absorbs
//! what is [`crate::biennium::Measure::FoundationAid`] — `[H] + [I]`, which is what
//! [`crate::panel::DistrictRecord::realized_aid`] returns and what every published share is
//! computed on.
//!
//! `[K]` is outside *that*. It is invisible to `realized_aid` and present in `[R]`, which is the
//! distinction [`crate::biennium::Measure`] exists to force and the reason this went unnoticed.
//!
//! # The devices are a ladder, not a set
//!
//! `[I]` tops `[H]` up to `[H2]`. `[K]` tops `[H] + [I] + [J]` up to `[L1]` — a total that
//! *already contains the guarantee*. So `[K]` is evaluated above `[I]` in the stack, and for the
//! 144 districts drawing it the identity
//!
//! ```text
//! [H] + [I] + [J] + [K] = [L1]
//! ```
//!
//! holds to the cent. Those districts are held at a **total**, which makes `[K]` a strictly
//! stronger instrument than the guarantee: a cut to foundation funding, to the guarantee itself,
//! or to transportation is offset pound for pound by `[K]` rising, and the district's cheque does
//! not move at all. See [`transition_supplement_under`] and [`insulated`].
//!
//! `[I]` is a term in `[K]`'s own subtrahend, so the supplement **backstops the guarantee**:
//! retiring `[I]` moves 90.9% of it onto `[K]` rather than saving it, and 127 of the 294
//! guaranteed districts are made whole. Most of that is latent — a district drawing no `[K]`
//! today. See [`absorption`], and `the_floors_the_guarantee_denominator_left_out.rs` for what it
//! means for every scenario this crate runs.
//!
//! `[L] Base Funding Supplement`, `[M] Enrollment Growth Supplement` and `[O] Performance
//! Supplement` are outside the closure — the 144 are paid $13.3m of the first two on top of a
//! total that is otherwise frozen.
//!
//! # Two of the four are uncodified, and one act carries them both
//!
//! `[I]` and `[F]` are codified at R.C. 3317.019, and the section **names its own years**: as
//! amended by H.B. 96 it reads "for fiscal years 2026 and 2027" at both (A)(1) and (A)(2), so the
//! guarantee is temporary in the Revised Code rather than merely in practice.
//!
//! `[K]` is not codified at all. Section 265.225 is uncodified law of H.B. 110, and the newest
//! text this repository holds — H.B. 583's amendment — reads "for fiscal years 2022 and 2023".
//! What pays it in FY2027 is H.B. 96, whose own analysis extends **both** devices in a single
//! clause: it "extends to FY 2026 and FY 2027 the payment of temporary transitional aid to school
//! districts based on an FY 2020 funding base *and a formula transition supplement based on an FY
//! 2021 funding base*". The department's payment report cites only the originating section, which
//! is why [`Device::authority`] names the act that keeps each one alive as well.
//!
//! That single clause is the legislative fact behind [`absorption`]: the two devices the backstop
//! links have now been carried forward together, twice, by the same acts.
//!
//! # `[K]` reaches further than this inventory does
//!
//! H.B. 96 extends the supplement "to districts, community schools, and STEM schools". Every
//! figure here is over the **609 city, local and exempted village districts the FY2027 model
//! carries**, so the $63,578,629.47 is the district share and not the programme. The community
//! school and STEM school halves are paid under the same section and are not in this fixture; a
//! statewide cost of repealing `[K]` would be larger than anything computed here.
//!
//! # Three anchor years, not one
//!
//! Two of the four bases carry a `FY21` heading and only one of those headings is right.
//! `the_year_the_column_header_names.rs` retired the reading for `[F1]`, which is the FY2020
//! amount before Executive Order 2020-19D under R.C. 3317.019(A)(2); `[H2]` is the same year
//! under (A)(1), established at `the_year_the_guarantee_holds_to.rs`; and `[H3]` — the DPIA slice
//! that partitions `[H2]` — reaches back to the FY2019 DPIA payment. Only `[L1]` is FY2021. See
//! [`bases`].
//!
//! # What is not a hold-harmless
//!
//! `[L]` pays $40 a pupil to all 609 districts with no test of any kind: a flat grant with no
//! historical anchor and no shortfall to make good. `[M]` pays a district whose enrolment
//! **rose**; it points the opposite way and is a need adjustment, not a floor. Neither is in the
//! inventory, and the reason is the same for both — a hold-harmless is defined here by comparing
//! a computed amount against a historical one and paying the difference, which is what all four
//! devices below do and what neither supplement does.

use edfund_core::{Adm, Dollars};

use crate::panel::DistrictRecord;

/// The fiscal year a device reaches back to.
///
/// Three, against the one the corpus recorded. The workbook labels `[H2]`, `[F1]` and `[L1]` alike
/// and only the last label is right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// The FY2019 DPIA payment, which `[H3]` is built from at R.C. 3317.02(N)(2).
    Fiscal2019,
    /// The FY2020 amount **before** Executive Order 2020-19D's pandemic reductions.
    Fiscal2020,
    /// FY2021, the only year a `FY21` column heading names correctly.
    Fiscal2021,
    /// The prior year, for a device that compares against last year rather than a frozen one.
    PriorYear,
}

/// Where a device sits relative to the boundary that decides what absorbs what.
///
/// Not `[H]`, which is only the seven phased-in components: the guarantee is outside `[H]` as a
/// line and inside the measure every published share is computed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Inside [`crate::biennium::Measure::FoundationAid`] — `[H] + [I]`.
    FoundationAid,
    /// Outside it, inside `[N] Total Formula Funding`. Visible only in `[R]`.
    FormulaFunding,
    /// Inside `[J] Transportation`, which is itself outside foundation aid.
    Transportation,
    /// A term in the `[H]` computation rather than a line beside it.
    WithinTheComputation,
}

/// Which way a device moves the cheque.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Pays the shortfall between a historical amount and a computed one.
    HoldsUp,
    /// Reduces a hold-harmless a district would otherwise have drawn.
    ClawsBack,
}

/// What [`crate::policy::apply`] does with a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modelled {
    /// Re-derived against the policy's own output, so a lever moves it.
    Recomputed,
    /// Carried at its published value and added, so a lever cannot move it.
    PassedThrough,
    /// Absent from the sum entirely.
    Omitted,
}

/// One base a device measures against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base {
    /// The workbook column.
    pub column: &'static str,
    /// What it holds.
    pub label: &'static str,
    /// The year it reaches back to, which is not always the year its heading names.
    pub anchor: Anchor,
    /// The section that defines it.
    pub authority: &'static str,
}

/// One device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Device {
    /// The line on the FY2026-and-later payment report, whose lettering the FY2027 model shares.
    pub line: &'static str,
    /// What the department calls it.
    pub label: &'static str,
    /// The column of [`bases`] it measures against, or `None` for one that compares two years.
    pub base: Option<&'static str>,
    /// Where it sits relative to [`crate::biennium::Measure::FoundationAid`].
    pub placement: Placement,
    /// Whether it pays or claws back.
    pub direction: Direction,
    /// What [`crate::policy::apply`] does with it.
    pub modelled: Modelled,
    /// The section or uncodified item that authorises it, and — where the section is temporary
    /// law that names its own years — the act that extends it to FY2027.
    pub authority: &'static str,
}

/// The four bases, and the three years they reach back to.
///
/// `[H3]` is not a fifth device — it is a slice of `[H2]`, which it partitions exactly — but it
/// carries its own anchor year, so a table of devices cannot hold it and a table of bases must.
#[must_use]
pub const fn bases() -> [Base; 4] {
    [
        Base {
            column: "[H2]",
            label: "Funding base",
            anchor: Anchor::Fiscal2020,
            authority: "R.C. 3317.02(N)(1)",
        },
        Base {
            column: "[H3]",
            label: "Funding base, economically disadvantaged slice",
            anchor: Anchor::Fiscal2019,
            authority: "R.C. 3317.02(N)(2)",
        },
        Base {
            column: "[L1]",
            label: "FY21 funding base, which includes transportation",
            anchor: Anchor::Fiscal2021,
            authority: "H.B. 110 Section 265.225, extended to FY2027 by H.B. 96",
        },
        Base {
            column: "[F1]",
            label: "FY21 transportation funding base, whose heading is stale",
            anchor: Anchor::Fiscal2020,
            authority: "R.C. 3317.019(A)(2)",
        },
    ]
}

/// Every hold-harmless device in the FY2027 formula, and the one that runs the other way.
///
/// The phase-in is not here. R.C. 3317.022 interpolates from `[H2]`, so below 100% it is a
/// hold-harmless in substance — but FY2027 is the terminal year, both dials are at 100%, the
/// bases cancel, and it binds on nothing. [`crate::policy::Policy`] carries it as a lever and
/// `the_interpolation_the_terminal_year_cannot_show.rs` is where it is measured.
#[must_use]
pub const fn inventory() -> [Device; 4] {
    [
        Device {
            line: "[I]",
            label: "Temporary transitional aid guarantee",
            base: Some("[H2]"),
            placement: Placement::FoundationAid,
            direction: Direction::HoldsUp,
            modelled: Modelled::Recomputed,
            authority: "R.C. 3317.019(A)(1)",
        },
        Device {
            line: "[K]",
            label: "Formula transition supplement",
            base: Some("[L1]"),
            placement: Placement::FormulaFunding,
            direction: Direction::HoldsUp,
            modelled: Modelled::Omitted,
            authority: "H.B. 110 Section 265.225, extended to FY2027 by H.B. 96",
        },
        Device {
            line: "[F]",
            label: "Temporary transitional aid guarantee for transportation",
            base: Some("[F1]"),
            placement: Placement::Transportation,
            direction: Direction::HoldsUp,
            modelled: Modelled::PassedThrough,
            authority: "R.C. 3317.019(A)(2)",
        },
        Device {
            line: "[I1]",
            label: "Open-enrolment adjustment",
            base: None,
            placement: Placement::FoundationAid,
            direction: Direction::ClawsBack,
            modelled: Modelled::PassedThrough,
            authority: "R.C. 3317.019(C)",
        },
    ]
}

/// What one district draws from one device, by the device's line.
#[must_use]
pub fn paid(record: &DistrictRecord, line: &str) -> Dollars {
    match line {
        "[I]" => record.guarantee,
        "[K]" => record.transition.transition_supplement,
        "[F]" => record.transportation.guarantee,
        "[I1]" => record.transition.open_enrollment_adjustment,
        _ => 0.0,
    }
}

/// How far one device reaches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reach {
    /// The device's line.
    pub line: &'static str,
    /// What it pays, or withholds where the device claws back.
    pub dollars: Dollars,
    /// How many districts it touches.
    pub districts: usize,
    /// Their share of the panel's enrolled ADM.
    pub adm_share: f64,
}

/// Every device measured across the panel, in the order [`inventory`] gives them.
///
/// Over the **609** districts the department's FY2027 model carries, which is not the 611 its
/// payment reports do: Middle Bass and North Bass are not costed here.
#[must_use]
pub fn reach(panel: &[DistrictRecord]) -> Vec<Reach> {
    let total_adm = enrolled_adm(panel);
    inventory()
        .into_iter()
        .map(|device| {
            let drawing: Vec<&DistrictRecord> = panel
                .iter()
                .filter(|record| paid(record, device.line) > 0.0)
                .collect();
            Reach {
                line: device.line,
                dollars: drawing.iter().map(|record| paid(record, device.line)).sum(),
                districts: drawing.len(),
                adm_share: share_of(&drawing, total_adm),
            }
        })
        .collect()
}

/// The panel's enrolled ADM, which every share here is taken against.
#[must_use]
pub fn enrolled_adm(panel: &[DistrictRecord]) -> Adm {
    panel
        .iter()
        .map(|record| record.categorical_enrolled_adm)
        .sum()
}

/// What a subset of the panel is worth as a share of its ADM.
fn share_of(subset: &[&DistrictRecord], total_adm: Adm) -> f64 {
    if total_adm <= 0.0 {
        return 0.0;
    }
    subset
        .iter()
        .map(|record| record.categorical_enrolled_adm)
        .sum::<Adm>()
        / total_adm
}

/// Whether any hold-harmless device pays this district.
///
/// The clawback is excluded: it reduces a device rather than being one, and a district it touches
/// is already counted through `[I]`.
#[must_use]
pub fn held(record: &DistrictRecord) -> bool {
    record.guarantee > 0.0
        || record.transition.transition_supplement > 0.0
        || record.transportation.guarantee > 0.0
}

/// The denominator every share of "the guarantee" should have been taken against.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Combined {
    /// What the three paying devices are worth together.
    pub dollars: Dollars,
    /// How many districts at least one of them reaches.
    pub districts: usize,
    /// Their share of enrolled ADM.
    pub adm_share: f64,
}

/// The three paying devices together, over the districts at least one of them reaches.
#[must_use]
pub fn combined(panel: &[DistrictRecord]) -> Combined {
    let drawing: Vec<&DistrictRecord> = panel.iter().filter(|record| held(record)).collect();
    Combined {
        dollars: drawing
            .iter()
            .map(|record| {
                record.guarantee
                    + record.transition.transition_supplement
                    + record.transportation.guarantee
            })
            .sum(),
        districts: drawing.len(),
        adm_share: share_of(&drawing, enrolled_adm(panel)),
    }
}

/// `[K]` re-derived against a policy's own output, which is what makes its omission measurable.
///
/// `max([L1] − (realized aid + transportation), 0)`, where realized aid is `[H] + [I]` under the
/// policy being run. At current law — [`DistrictRecord::realized_aid`] and the published
/// transportation total — this reproduces the department's column for all 609 districts, which is
/// what licenses using it on an output the department never published.
///
/// Supplemental targeted assistance is a term in the department's formula and pays zero for every
/// district in FY2027, so it drops out rather than being dropped.
#[must_use]
pub fn transition_supplement_under(
    record: &DistrictRecord,
    realized_aid: Dollars,
    transportation: Dollars,
) -> Dollars {
    (record.transition.fy21_funding_base - (realized_aid + transportation)).max(0.0)
}

/// The districts `[K]` insulates against any cut to `[H]`, `[I]` or `[J]`.
///
/// A district already drawing the supplement is held at `[L1]` in total, so a dollar taken off
/// foundation funding is a dollar `[K]` puts back. Every scenario this crate runs reports those
/// dollars as taken.
#[must_use]
pub fn insulated(panel: &[DistrictRecord]) -> usize {
    panel
        .iter()
        .filter(|record| record.transition.transition_supplement > 0.0)
        .count()
}

/// One district's position on the two quantities `[K]` is measured against.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    /// `[H] + [I]` — [`crate::biennium::Measure::FoundationAid`].
    pub realized_aid: Dollars,
    /// `[J]`, which `[K]` counts and `realized_aid` does not.
    pub transportation: Dollars,
}

impl Position {
    /// The total `[K]` tops up to `[L1]` from.
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.realized_aid + self.transportation
    }
}

/// What `[K]` would absorb of a cut, against what the cut is reported as.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Absorption {
    /// The loss after the guarantee, which is what a scenario run reports.
    pub reported: Dollars,
    /// The loss after `[K]` has risen to meet it.
    pub actual: Dollars,
    /// Districts the cut reaches as reported.
    pub reported_districts: usize,
    /// Districts it actually reaches.
    pub actual_districts: usize,
}

impl Absorption {
    /// What `[K]` puts back.
    #[must_use]
    pub fn absorbed(&self) -> Dollars {
        self.reported - self.actual
    }

    /// That, as a share of what the run reported.
    #[must_use]
    pub fn share(&self) -> f64 {
        if self.reported <= 0.0 {
            return 0.0;
        }
        self.absorbed() / self.reported
    }
}

/// A cent, which is the tolerance every figure here is measured at.
///
/// [`crate::policy::apply`] recovers transportation to its pre-share amount by dividing by the
/// share applied and multiplying back, and on fifteen districts that round-trip lands a cent away
/// from the published total. Counting a district as reached at anything finer turns that artefact
/// into twenty-one districts of phantom loss, which is how this threshold came to be named rather
/// than assumed.
pub const CENT: Dollars = 0.01;

/// Measure a policy's cut against the one `[K]` would leave, district by district.
///
/// `under` supplies each district's [`Position`] twice — under the policy and under the baseline
/// the same run computed, which are [`crate::policy::Outcome`]'s four aid fields. Taking the
/// baseline from the run rather than from the record is what keeps the cent artefact above from
/// being counted as a loss.
#[must_use]
pub fn absorption<F>(panel: &[DistrictRecord], under: F) -> Absorption
where
    F: Fn(&DistrictRecord) -> (Position, Position),
{
    let mut out = Absorption::default();
    for record in panel {
        let (moved, baseline) = under(record);
        let reported = (baseline.total() - moved.total()).max(0.0);
        let supplement =
            transition_supplement_under(record, moved.realized_aid, moved.transportation);
        let actual = (reported - (supplement - record.transition.transition_supplement)).max(0.0);
        out.reported += reported;
        out.actual += actual;
        if reported > CENT {
            out.reported_districts += 1;
        }
        if actual > CENT {
            out.actual_districts += 1;
        }
    }
    out
}
