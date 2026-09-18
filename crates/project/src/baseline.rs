//! FY2025, the year a biennium comparison is measured from.
//!
//! # Why this year is a payment report and the others are calculators
//!
//! [`mod@crate::panel`] is the department's FY2027 model and [`crate::prior_model`] its FY2026 one.
//! This is neither: FY2025's calculator survives only in Internet Archive captures truncated at
//! exactly 1 MiB, so the workbook cannot be read. What is open is the department's **final**
//! payment report, published after the year closed.
//!
//! That is the better artefact for a baseline and not merely the available one. A calculator
//! states what a model projected before a year; this states what was paid after it. "Change in
//! funding from FY2025 levels" is a claim about money that moved, so the paid figure is the one
//! the claim is about.
//!
//! # It carries a line no later year has
//!
//! [`Fy2025::supplemental_targeted_assistance`] is the tier H.B. 96 repealed.
//! `formula-component/fsfp-targeted-assistance` records the repeal as "a live zero rather than an
//! absence" — the eligibility column still runs and still names the districts, while the payment
//! was never computed from its rate anywhere in this workspace. This is the last year it was
//! paid, per district, and the only place that amount survives.
//!
//! # The guarantee does not always close, and the reason is a clawback rather than a shortfall
//!
//! `[Hd] + [I] = [Ha]` holds for most guaranteed districts and fails for **16 of the 611** here,
//! which are paid less than their funding base by between $3,024 (Ross Local) and $668,301
//! (Lakota Local). The whole of every gap is `[I1] Open Enrollment Adjustment`, on the
//! `Detailed SFPR` sheet — **16 of 16, to the cent**.
//!
//! So the identity the guarantee actually satisfies is the one
//! `formula-component/temporary-transitional-aid-guarantee` already writes down:
//!
//! ```text
//! [I] = max( [H2] Funding Base − [I1] Open Enrolment Adjustment − [H] Foundation Funding, 0 )
//! ```
//!
//! [`Fy2025::open_enrollment_adjustment`] is carried for exactly this reason. Nothing on the
//! summary sheet explains the shortfall, and a reader who has only that sheet will reach for an
//! explanation the formula does not need — this module first attributed it to a payment report
//! having a year of reconciliation behind it, which was wrong and which the corpus could have
//! corrected on sight.
//!
//! # And 611 rows, not 609
//!
//! The payment report carries **Middle Bass Local** and **North Bass Local**, two Lake Erie island
//! districts the FY2027 model does not. Every one of the panel's 609 is here; these two are extra.
//! See `the_year_the_biennium_is_measured_from.rs`, which is where that is asserted rather than
//! assumed.

use edfund_core::Dollars;

/// The committed extract of the department's final FY2025 payment report.
const FIXTURE: &str = include_str!("../fixtures/fy25-payment-report.csv");

const EXPECTED_HEADER: &str = "irn,district,county,foundation_base,foundation_calculated,\
     foundation_phase_in,foundation_state_funding,\
     temporary_transitional_aid_guarantee,supplemental_targeted_assistance,transportation,\
     formula_transition_supplement,total_formula_funding,preschool_special_education,\
     special_education_transportation,total_state_support,net_state_funding,\
     open_enrollment_adjustment";

/// One district's FY2025 payment, on the department's own lines.
#[derive(Debug, Clone)]
pub struct Fy2025 {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name, as the payment report spells it.
    pub name: String,
    /// County the department attributes the district to.
    pub county: String,
    /// `[Ha]` — the FY2020 funding base the guarantee holds a district at, and the base the
    /// phase-in interpolates away from.
    pub funding_base: Dollars,
    /// `[Hb]` — base cost and the six categoricals as the formula computes them: after the state
    /// share, *before* the phase-in and *before* the guarantee. The closest published figure to
    /// the plan's own output.
    pub foundation_calculated: Dollars,
    /// `[Hc]` — the interpolated step from [`Self::funding_base`] toward
    /// [`Self::foundation_calculated`]. Negative wherever the formula computes less than the base,
    /// which is the ordinary case for a guaranteed district.
    pub foundation_phase_in: Dollars,
    /// `[Hd]` — base cost and the six categoricals, after the phase-in.
    /// Equal to [`Self::funding_base`] plus [`Self::foundation_phase_in`].
    pub foundation: Dollars,
    /// `[I]` — the top-up toward the FY2020 funding base, net of the open-enrolment clawback.
    ///
    /// *Toward*, not *to*: 16 of the 611 rows here are paid less than [`Self::funding_base`] even
    /// with this line, by exactly [`Self::open_enrollment_adjustment`].
    pub guarantee: Dollars,
    /// `[J]` — the tier H.B. 96 repealed. Paid to 36 districts in this year and to none since.
    pub supplemental_targeted_assistance: Dollars,
    /// `[K]`.
    pub transportation: Dollars,
    /// `[L]` — the second hold-harmless, on an FY2021 base.
    pub formula_transition_supplement: Dollars,
    /// `[M]`. Note the letter: in FY2026 this quantity is `[N]`.
    pub total_formula_funding: Dollars,
    /// `[N]` in this year's scheme.
    pub preschool_special_education: Dollars,
    /// `[O]` in this year's scheme.
    pub special_education_transportation: Dollars,
    /// `[P]` — the widest measure of state support, and the one a "change in funding" claim is
    /// ordinarily about. Wider than [`Self::foundation`], which is all `crate::policy` computes.
    pub total_state_support: Dollars,
    /// `[T]` — total state support net of transfers.
    pub net_state_funding: Dollars,
    /// `[I1]` — the reduction applied to a guaranteed district that has shed entering open
    /// enrolment beyond a threshold. From `Detailed SFPR`; the summary sheet does not carry it.
    ///
    /// Zero for all but 16 districts, and for those it is the whole of the distance between what
    /// they were paid and [`Self::funding_base`].
    pub open_enrollment_adjustment: Dollars,
}

/// The FY2025 payment report, in IRN order.
///
/// # Panics
///
/// If the fixture's header is not the one this reader was written against, or a row's width
/// differs from the header's — both by way of [`edfund_core::csv::rows`].
#[must_use]
pub fn frame() -> Vec<Fy2025> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .map(|row| Fy2025 {
            irn: row.str(0).to_string(),
            name: row.str(1).to_string(),
            county: row.str(2).to_string(),
            funding_base: row.num(3).unwrap_or(0.0),
            foundation_calculated: row.num(4).unwrap_or(0.0),
            foundation_phase_in: row.num(5).unwrap_or(0.0),
            foundation: row.num(6).unwrap_or(0.0),
            guarantee: row.num(7).unwrap_or(0.0),
            supplemental_targeted_assistance: row.num(8).unwrap_or(0.0),
            transportation: row.num(9).unwrap_or(0.0),
            formula_transition_supplement: row.num(10).unwrap_or(0.0),
            total_formula_funding: row.num(11).unwrap_or(0.0),
            preschool_special_education: row.num(12).unwrap_or(0.0),
            special_education_transportation: row.num(13).unwrap_or(0.0),
            total_state_support: row.num(14).unwrap_or(0.0),
            net_state_funding: row.num(15).unwrap_or(0.0),
            open_enrollment_adjustment: row.num(16).unwrap_or(0.0),
        })
        .collect()
}

/// One district's FY2025 payment, by IRN.
#[must_use]
pub fn at(irn: &str) -> Option<Fy2025> {
    frame().into_iter().find(|row| row.irn == irn)
}

/// Whether this district was held at its funding base in FY2025.
///
/// Read off the guarantee line rather than compared against a floor, because the payment report
/// states the top-up directly and a positive top-up *is* the condition.
impl Fy2025 {
    /// Whether the guarantee paid this district anything in FY2025.
    #[must_use]
    pub fn on_guarantee(&self) -> bool {
        self.guarantee > 0.0
    }

    /// Whether the guarantee absorbed the formula entirely — `[Hd] + [I] == [Ha]`.
    ///
    /// The strong condition, and the one a comparison needs. A district that satisfies it is paid
    /// its funding base whatever the formula computes, so its foundation aid contributes **zero**
    /// to any change between years. [`Self::on_guarantee`] is the weak condition and is true of
    /// districts the formula still moves.
    ///
    /// To the cent, because the department's arithmetic closes exactly. Where it appears not to —
    /// 16 rows in this year — the missing term is [`Self::open_enrollment_adjustment`] rather
    /// than an error, and [`Self::held_at_its_base`] is the test that accounts for it.
    #[must_use]
    pub fn fully_on_guarantee(&self) -> bool {
        self.guarantee > 0.0 && (self.foundation + self.guarantee - self.funding_base).abs() < 0.005
    }

    /// Whether the guarantee determines this district's aid, clawback included.
    ///
    /// `[Hd] + [I] + [I1] == [Ha]`, which is the statute's identity rearranged and is true of
    /// every guaranteed district in the file. Prefer this to [`Self::fully_on_guarantee`] when
    /// the question is *what sets this district's aid*; the narrower one is for when the question
    /// is whether the district was paid its base.
    #[must_use]
    pub fn held_at_its_base(&self) -> bool {
        self.guarantee > 0.0
            && (self.foundation + self.guarantee + self.open_enrollment_adjustment
                - self.funding_base)
                .abs()
                < 0.005
    }
}
