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

use edfund_core::{Adm, Dollars, FiscalYear};

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

/// The supplements one year paid, by name.
///
/// [`Lines::supplements`] stays a residual so the five lines are exhaustive whatever an act does;
/// this is the same money named, so a reader can see *which* supplement moved. The two agree to
/// the cent on every district — `the_named_supplements_are_the_whole_of_the_residual` — and if an
/// act adds a supplement that this does not name, that test is what fails.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Supplements {
    /// Supplemental targeted assistance. Paid in FY2025 only: H.B. 96 repealed it.
    pub targeted_assistance: Dollars,
    /// The formula transition supplement, the second hold-harmless on an FY2021 base.
    pub formula_transition: Dollars,
    /// The base funding supplement, which H.B. 96 created.
    pub base_funding: Dollars,
    /// The enrollment growth supplement, which H.B. 96 created.
    pub enrollment_growth: Dollars,
    /// The performance supplement, which H.B. 96 created.
    pub performance: Dollars,
}

impl Supplements {
    /// Every named supplement summed.
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.targeted_assistance
            + self.formula_transition
            + self.base_funding
            + self.enrollment_growth
            + self.performance
    }
}

/// The foundation formula's published stages in one year: the base it interpolates from, what it
/// computes, and what it pays before the guarantee.
///
/// R.C. 3317.022 pays `base + rate × (calculated − base)`, so a district whose formula computes
/// about its FY2020 base gains nothing from a rising rate, and one far above it gains most. That is
/// the whole of why two neighbours can move in opposite directions under the same act.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseIn {
    /// `[Ha]`/`[H2]` — the FY2020 funding base. The same figure in all three years.
    pub funding_base: Dollars,
    /// `[Hb]` — the formula's own output, before the phase-in and the guarantee.
    pub calculated: Dollars,
    /// `[Hd]` — foundation funding as paid, `base + rate × (calculated − base)`.
    pub paid: Dollars,
}

impl PhaseIn {
    /// The step the phase-in took from the base. Negative where the formula computes below it.
    #[must_use]
    pub fn step(&self) -> Dollars {
        self.paid - self.funding_base
    }

    /// What the phase-in pays above the base: [`Self::step`] where the formula calculates above
    /// it, and nothing where it does not, because there the guarantee pays the shortfall.
    ///
    /// Foundation aid is `funding_base + above_base()` for every district the open enrollment
    /// clawback does not reach, and the base is the same in all three years. So a change in
    /// foundation aid between years is a change in this — `the_fall_in_foundation_aid_is_the_phase_ins.rs`.
    #[must_use]
    pub fn above_base(&self) -> Dollars {
        self.step().max(0.0)
    }
}

/// What the department subtracts from total state support to reach net state funding.
///
/// Every component is a deduction, and none is the voucher or community-school channel: under the
/// Fair School Funding Plan those are funded directly rather than deducted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transfers {
    /// The educational service center charge. `None` where the year publishes only the total.
    pub service_center: Option<Dollars>,
    /// The residual adjustments line. `None` where the year publishes only the total.
    pub other: Option<Dollars>,
    /// All transfers, as a signed amount — negative is money withheld.
    pub total: Dollars,
    /// Total state support after transfers.
    pub net_state_funding: Dollars,
}

/// The three payment lines outside both the formula and the supplements, at their level in one
/// year.
///
/// [`Lines`] carries these as the FY2025→FY2027 change; this is each year's figure, so a page can
/// set the three years side by side and a reader can see which year a line moved in. With
/// foundation aid and [`Supplements::total`] they make up total state support to within cent
/// rounding — `every_year_is_its_named_lines`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Paid {
    /// Transportation, before the special education line.
    pub transportation: Dollars,
    /// Special education transportation, after the year's proration.
    pub special_education_transportation: Dollars,
    /// Preschool special education.
    pub preschool_special_education: Dollars,
}

/// One observed year of one district, beyond its two measures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Year {
    /// The supplements paid, by name.
    pub supplements: Supplements,
    /// The other three payment lines outside foundation aid.
    pub paid: Paid,
    /// Enrolled ADM as the year's file states it. `None` for FY2025, whose payment report carries
    /// no capacity or enrollment inputs.
    pub enrolled_adm: Option<Adm>,
    /// The foundation formula's stages.
    pub phase_in: PhaseIn,
    /// Transfers and net funding. `None` for FY2026, whose model publishes no transfer columns —
    /// absent, which is not the same as zero.
    pub transfers: Option<Transfers>,
    /// `[b4]` the published state share of base cost. `None` for FY2025, whose payment report
    /// does not carry it.
    pub state_share: Option<f64>,
    /// Disadvantaged Pupil Impact Aid as the year's formula computes it, before the phase-in and
    /// the guarantee. `None` for FY2025, whose payment report does not itemize the formula.
    ///
    /// The one categorical carried by year, because it is the one H.B. 96 changed the count of:
    /// FY2026 blends the disadvantaged and directly certified counts 75/25 and FY2027 65/35. A
    /// component of what the formula calculates, not a payment line — it reaches foundation aid
    /// through the phase-in, and not at all for a district held at its base.
    pub dpia: Option<Dollars>,
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
    /// FY2025, FY2026 and FY2027, in that order.
    pub years: [Year; 3],
    /// Assessed valuation for tax years 2023, 2024 and 2025, oldest first — the three the FY2027
    /// capacity measure blends. A county reappraisal shows up here a year before the state share
    /// falls under it.
    pub valuation: [Dollars; 3],
}

impl Row {
    /// Total state support's FY2025→FY2026 change with supplemental targeted assistance taken out
    /// of both years: the dollars, and that change as a share of FY2025 without it. `None` for a
    /// district paid under a cent of it in FY2025.
    ///
    /// H.B. 96 repealed the supplement after FY2025, so a recipient's first step carries all of it
    /// as a loss. This is what the rest of the payment did. The county page's who-gained card
    /// prints the same figure, and `withoutTargetedAssistance` in `web/src/lib/countyChange.ts`
    /// must agree with it.
    #[must_use]
    pub fn without_targeted_assistance(&self) -> Option<(Dollars, f64)> {
        let [fy25, fy26, _] = self.years;
        if fy25.supplements.targeted_assistance < 0.01 {
            return None;
        }
        let before = self.total.at(BASELINE_YEAR)? - fy25.supplements.targeted_assistance;
        let after = self.total.at(MIDDLE_YEAR)? - fy26.supplements.targeted_assistance;
        let change = after - before;
        Some((change, if before > 0.0 { change / before } else { 0.0 }))
    }
}

/// The tax years [`Row::valuation`] covers, oldest first.
pub const VALUATION_TAX_YEARS: [u16; 3] = [2023, 2024, 2025];

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
/// [`Lines`] is stated as the FY2025→FY2027 split — the biennium's endpoints — and a middle-year
/// decomposition would be a second, differently-shaped object rather than more of this one. The
/// middle year's lines are read as levels instead, into [`Year::paid`], where all three years
/// carry them alike.
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

/// The FY2025 year beyond its measures.
fn year_2025(row: &Fy2025) -> Year {
    Year {
        supplements: Supplements {
            targeted_assistance: row.supplemental_targeted_assistance,
            formula_transition: row.formula_transition_supplement,
            ..Supplements::default()
        },
        paid: Paid {
            transportation: row.transportation,
            special_education_transportation: row.special_education_transportation,
            preschool_special_education: row.preschool_special_education,
        },
        enrolled_adm: None,
        phase_in: PhaseIn {
            funding_base: row.funding_base,
            calculated: row.foundation_calculated,
            paid: row.foundation,
        },
        // The payment report states net funding and not its parts.
        transfers: Some(Transfers {
            service_center: None,
            other: None,
            total: row.net_state_funding - row.total_state_support,
            net_state_funding: row.net_state_funding,
        }),
        state_share: None,
        dpia: None,
    }
}

/// The FY2026 year beyond its measures.
fn year_2026(row: &Prior) -> Year {
    Year {
        supplements: Supplements {
            targeted_assistance: 0.0,
            formula_transition: row.formula_transition_supplement,
            base_funding: row.base_funding_supplement,
            enrollment_growth: row.enrollment_growth_supplement,
            performance: row.performance_supplement,
        },
        paid: Paid {
            transportation: row.transportation,
            special_education_transportation: row.summary_special_education_transportation,
            preschool_special_education: row.preschool_special_education,
        },
        enrolled_adm: Some(row.enrolled_adm),
        phase_in: PhaseIn {
            funding_base: row.funding_base,
            calculated: row.foundation_calculated,
            paid: row.foundation_funding,
        },
        transfers: None,
        state_share: Some(row.state_share_percentage),
        dpia: Some(row.dpia_aid),
    }
}

/// The FY2027 year beyond its measures.
///
/// The phase-in is at 100% in this year, so what the formula calculates is what it pays.
fn year_2027(record: &DistrictRecord) -> Year {
    Year {
        supplements: Supplements {
            targeted_assistance: 0.0,
            formula_transition: record.transition.transition_supplement,
            base_funding: record.supplements.base_funding,
            enrollment_growth: record.supplements.growth,
            performance: record.performance.amount,
        },
        paid: Paid {
            transportation: record.transportation.total,
            special_education_transportation: record.transportation.special_education,
            preschool_special_education: record.preschool_special_education.total,
        },
        enrolled_adm: Some(record.current_year_adm),
        phase_in: PhaseIn {
            funding_base: record.transition.funding_base,
            calculated: record.core_foundation_funding,
            paid: record.core_foundation_funding,
        },
        transfers: Some(Transfers {
            service_center: Some(record.service_center_charge),
            other: Some(record.other_adjustments),
            total: record.total_transfers,
            net_state_funding: record.net_state_funding,
        }),
        state_share: record.published_state_share,
        dpia: Some(record.categoricals.dpia),
    }
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
            years: [year_2025(before), year_2026(middle), year_2027(&record)],
            valuation: {
                let [newest, middle, oldest] = record.valuation_three_year;
                [oldest, middle, newest]
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
