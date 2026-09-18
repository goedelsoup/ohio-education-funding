//! What changed between FY2025 and the end of the biennium, per district, on a stated measure.
//!
//! # Why the measure is a type
//!
//! Two different quantities in this workspace are called a district's state funding, and they
//! differ by about 15%:
//!
//! - [`Measure::FoundationAid`] is `core foundation funding + guarantee` — what
//!   [`crate::policy::apply`] computes and [`crate::report::forecast`] projects. $7.281B in FY2027.
//! - [`Measure::TotalStateSupport`] is the department's `[R]` line, which adds transportation,
//!   special education transportation, preschool special education and the supplements. $8.553B.
//!
//! **The $1.272B between them is where the interesting variation is.** Delphos City and Shawnee
//! Local are both held at their funding base throughout the biennium, so their foundation aid is
//! identical in every year and contributes exactly nothing to a comparison; everything that
//! separates them is in the 15%. A comparison that silently picked the narrow measure would
//! report the two districts as alike, which is the question answered backwards.
//!
//! So a measure is carried on the value rather than chosen by a caller and remembered. Nothing
//! here returns a bare `Dollars` that could be added to a figure on the other measure.
//!
//! # Why this cannot be projected
//!
//! Every year here is [`Basis::Observed`]: three published department files, no model. It stops at
//! [`crate::panel::MODEL_YEAR`] and [`Change::at`] answers `None` for anything past it.
//!
//! That refusal is deliberate and is not a gap waiting to be filled. `forecast` produces
//! `FoundationAid` only, because the model has no transportation, supplement or preschool
//! projection to produce the wider measure with. Letting a projected year join this series would
//! append the narrow measure to the wide one and call the join a trend.
//!
//! # And the population is fixed at 609
//!
//! The FY2025 and FY2026 files carry **611** districts and the FY2027 model **609** — Middle Bass
//! Local and North Bass Local are paid in both payment years and modelled in neither. This takes
//! the intersection, so no total here changes population between its years. See
//! [`crate::baseline`] for the two districts and `the_population_the_panel_speaks_for.rs` for what
//! the 609 excludes.

use std::collections::HashMap;

use edfund_core::{Dollars, FiscalYear};

use crate::baseline::{self, Fy2025};
use crate::panel::{panel, DistrictRecord, MODEL_YEAR};
use crate::prior_model::{self, Prior};
use crate::series::Basis;

/// The first year of the comparison — the year a "change in funding" claim is measured from.
pub const BASELINE_YEAR: FiscalYear = FiscalYear(2025);
/// The middle year.
pub const MIDDLE_YEAR: FiscalYear = FiscalYear(2026);

/// Which money a figure is.
///
/// Carried on every value in this module so that two figures on different measures cannot be
/// added, subtracted or charted together without the difference being visible at the call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measure {
    /// `core foundation funding + guarantee`. What the model computes and the only measure it can
    /// project. Excludes transportation, the supplements and preschool special education.
    FoundationAid,
    /// The department's `[R] Total State Support`. What a district is paid before transfers, and
    /// the measure a "change in funding" claim is ordinarily about.
    TotalStateSupport,
}

impl Measure {
    /// How the measure reads in prose, for a caller that has to label a figure.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::FoundationAid => "foundation aid",
            Self::TotalStateSupport => "total state support",
        }
    }
}

/// One district's three observed years on one measure.
///
/// The years are private and there are exactly three. A caller reads them through [`Change::at`],
/// which knows only the observed window — so a projected year cannot be appended, and asking for
/// one returns `None` rather than a number on a different measure.
#[derive(Debug, Clone)]
pub struct Change {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name, as the FY2027 model spells it.
    pub name: String,
    /// Which money these amounts are.
    pub measure: Measure,
    fy2025: Dollars,
    fy2026: Dollars,
    fy2027: Dollars,
}

impl Change {
    /// Every year here is observed. There is no constructor that produces anything else.
    pub const BASIS: Basis = Basis::Observed;

    /// The amount in one fiscal year, or `None` outside the observed window.
    #[must_use]
    pub fn at(&self, year: FiscalYear) -> Option<Dollars> {
        match year {
            BASELINE_YEAR => Some(self.fy2025),
            MIDDLE_YEAR => Some(self.fy2026),
            MODEL_YEAR => Some(self.fy2027),
            _ => None,
        }
    }

    /// The change from FY2025 to one later year, or `None` outside the window.
    #[must_use]
    pub fn against_baseline(&self, year: FiscalYear) -> Option<Dollars> {
        Some(self.at(year)? - self.fy2025)
    }

    /// `(FY2026 − FY2025) + (FY2027 − FY2025)` — the biennium's total change against the year it
    /// is measured from.
    ///
    /// Both years counted against the same baseline, which is what a two-year budget's effect on a
    /// district is: each year of it is paid, and each is a departure from the last settled year.
    /// Summing year-over-year steps instead would give `FY2027 − FY2025` and count the first
    /// year's change once rather than twice.
    #[must_use]
    pub fn biennium(&self) -> Dollars {
        (self.fy2026 - self.fy2025) + (self.fy2027 - self.fy2025)
    }
}

/// The FY2025→FY2027 change split across the department's own payment lines.
///
/// Only meaningful on [`Measure::TotalStateSupport`], because on the narrow measure four of the
/// five lines are zero by construction. [`Lines::total`] equals the wide measure's
/// `against_baseline(MODEL_YEAR)` to the cent, which is the check that the split is exhaustive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lines {
    /// `core foundation funding + guarantee` — zero for any district held at its base throughout.
    pub foundation: Dollars,
    /// `[G]`/`[J]` transportation.
    pub transportation: Dollars,
    /// `[Q]` special education transportation.
    pub special_education_transportation: Dollars,
    /// `[P]` preschool special education.
    pub preschool_special_education: Dollars,
    /// Everything else, as a residual so the five lines are exhaustive by construction.
    ///
    /// The formula transition supplement in both years; supplemental targeted assistance in
    /// FY2025, which H.B. 96 repealed; the base funding, enrollment growth and performance
    /// supplements in FY2027, which it created. A residual rather than a sum because the set of
    /// supplements is not the same in the two years, and a sum would have to be edited every time
    /// an act adds one.
    pub supplements: Dollars,
}

impl Lines {
    /// The five lines summed — equal to the total state support change.
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.foundation
            + self.transportation
            + self.special_education_transportation
            + self.preschool_special_education
            + self.supplements
    }
}

/// One district's three years, its measure, and the lines the change is made of.
#[derive(Debug, Clone)]
pub struct Row {
    /// Total state support, the wide measure.
    pub total: Change,
    /// Foundation aid, the narrow one the model projects.
    pub foundation: Change,
    /// What the wide measure's FY2025→FY2027 change is made of.
    pub lines: Lines,
}

/// A district's FY2027 preschool, transportation and foundation lines, from the panel.
fn at_fy2027(record: &DistrictRecord) -> (Dollars, Dollars, Dollars, Dollars, Dollars) {
    (
        record.realized_aid(),
        record.transportation.total,
        record.transportation.special_education,
        record.preschool_special_education.total,
        record.total_state_support,
    )
}

/// The FY2026 reading, on the two measures only.
///
/// The middle year carries the same five payment lines and this deliberately does not read them.
/// [`Lines`] is stated as the FY2025→FY2027 split — the biennium's endpoints — and a middle-year
/// decomposition would be a second, differently-shaped object rather than more of this one. The
/// year is fully present in both level series and in [`Change::biennium`]; what it is not is a
/// third column of [`Lines`].
fn at_fy2026(row: &Prior) -> (Dollars, Dollars) {
    (
        row.foundation_funding + row.guarantee,
        row.total_state_support,
    )
}

/// The FY2025 reading.
fn at_fy2025(row: &Fy2025) -> (Dollars, Dollars, Dollars, Dollars, Dollars) {
    (
        row.foundation + row.guarantee,
        row.transportation,
        row.special_education_transportation,
        row.preschool_special_education,
        row.total_state_support,
    )
}

/// Every district the three files agree on, in IRN order.
///
/// # Panics
///
/// If any of the three fixtures fails its header check, by way of its own reader.
#[must_use]
pub fn frame() -> Vec<Row> {
    let paid: HashMap<String, Fy2025> = baseline::frame()
        .into_iter()
        .map(|row| (row.irn.clone(), row))
        .collect();
    let prior: HashMap<String, Prior> = prior_model::frame()
        .into_iter()
        .map(|row| (row.irn.clone(), row))
        .collect();

    let mut out = Vec::new();
    for record in panel() {
        let (Some(before), Some(middle)) = (paid.get(&record.irn), prior.get(&record.irn)) else {
            continue;
        };
        let (f25, t25, s25, p25, w25) = at_fy2025(before);
        let (f26, w26) = at_fy2026(middle);
        let (f27, t27, s27, p27, w27) = at_fy2027(&record);

        let named = |measure, a, b, c| Change {
            irn: record.irn.clone(),
            name: record.name.clone(),
            measure,
            fy2025: a,
            fy2026: b,
            fy2027: c,
        };
        let foundation = f27 - f25;
        out.push(Row {
            total: named(Measure::TotalStateSupport, w25, w26, w27),
            foundation: named(Measure::FoundationAid, f25, f26, f27),
            lines: Lines {
                foundation,
                transportation: t27 - t25,
                special_education_transportation: s27 - s25,
                preschool_special_education: p27 - p25,
                // The residual, so the five are exhaustive whatever an act does to the supplements.
                supplements: (w27 - w25) - foundation - (t27 - t25) - (s27 - s25) - (p27 - p25),
            },
        });
    }
    out.sort_by(|a, b| a.total.irn.cmp(&b.total.irn));
    out
}

/// One district's biennium, by IRN.
#[must_use]
pub fn at(irn: &str) -> Option<Row> {
    frame().into_iter().find(|row| row.total.irn == irn)
}
