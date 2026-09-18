//! The phase-in, at the one year in the workspace where it can be seen doing anything.
//!
//! R.C. 3317.022 interpolates: a district is funded at `base + pct × (computed − base)`. The
//! FY2027 panel is at `pct = 1`, where that expression collapses to `computed` and a wrong
//! multiplier is invisible. FY2026 is at 0.8333 and FY2025 at its own schedule, and both carry
//! the department's `[Ha]` base, `[Hb]` computed, `[Hc]` step and `[Hd]` funded columns, so the
//! multiplier is recoverable per district rather than assumed.
//!
//! The guarantee sits on top of that and is the reason the interpolation is so often invisible in
//! a payment: for a district held at its base, every dollar the formula moves is offset, and the
//! foundation contributes exactly nothing to a change between years.

use project::{baseline, prior_model};

const DELPHOS: &str = "043885";
const SHAWNEE: &str = "045799";

/// The department's stated general phase-in, recovered from every district's own three columns.
///
/// `Summary_SFPR` carries `0.8333` once, in a header cell above the table. This does not read that
/// cell: it solves `[Hc] / ([Hb] − [Ha])` on each district and asserts every answer is the same
/// number. A multiplier applied to the wrong base, or to the wrong sign of the span, fails here
/// and cannot fail on the FY2027 panel.
#[test]
fn fy2026_interpolates_at_exactly_the_stated_percentage() {
    let mut seen = 0;
    for row in prior_model::frame() {
        let span = row.foundation_calculated - row.funding_base;
        // A span near zero divides badly and says nothing about the multiplier either way.
        if span.abs() < 1_000.0 {
            continue;
        }
        seen += 1;
        let implied = row.foundation_phase_in / span;
        assert!(
            (implied - 0.8333).abs() < 5e-7,
            "{} ({}) implies a phase-in of {implied}",
            row.name,
            row.irn
        );
    }
    assert_eq!(seen, 610, "districts with a span wide enough to divide");
}

/// `[Hd] = [Ha] + [Hc]` — the funded amount is the base plus the step, in both years.
///
/// The identity that makes the four columns one quantity rather than four. It holds to the cent
/// on every row of both fixtures, which is what licenses reading a shortfall against `[Ha]` as a
/// fact about the guarantee rather than about the arithmetic.
#[test]
fn the_funded_amount_is_the_base_plus_the_step_in_both_years() {
    for row in baseline::frame() {
        let closes = row.funding_base + row.foundation_phase_in - row.foundation;
        assert!(
            closes.abs() < 0.005,
            "FY2025 {} is off by {closes}",
            row.irn
        );
    }
    for row in prior_model::frame() {
        let closes = row.funding_base + row.foundation_phase_in - row.foundation_funding;
        assert!(
            closes.abs() < 0.005,
            "FY2026 {} is off by {closes}",
            row.irn
        );
    }
}

/// FY2026 states transportation twice, on two sheets, and the two agree.
///
/// `[G]` on `Transportation` and `[J]` on `Summary_SFPR`. Carrying both is deliberate: the
/// extractor reads them through different headers, so agreement is evidence the tag-matching
/// found the columns it meant to and not their neighbours.
#[test]
fn the_two_transportation_columns_agree_on_every_district() {
    for row in prior_model::frame() {
        let gap = row.transportation_total - row.transportation;
        assert!(
            gap.abs() < 0.005,
            "{} ({}): [G] {} against [J] {}",
            row.name,
            row.irn,
            row.transportation_total,
            row.transportation
        );
    }
}

/// Sixteen FY2025 districts are paid less than their base, and the clawback is the whole of it.
///
/// **The identity is the statute's, not an approximation of it.**
/// `formula-component/temporary-transitional-aid-guarantee` writes the guarantee as
/// `max([H2] − [I1] − [H], 0)`, so a guaranteed district satisfies `[Hd] + [I] + [I1] = [Ha]` and
/// only satisfies `[Hd] + [I] = [Ha]` where the open-enrolment clawback is zero.
///
/// Sixteen districts here have a non-zero clawback, and for every one of them it accounts for the
/// entire distance to the funding base — to the cent, not approximately. The count is pinned
/// because it is a fact about this year; the *mechanism* is asserted because it is a fact about
/// the formula, and asserting only the count is what let a wrong explanation stand.
#[test]
fn the_sixteen_districts_short_of_their_base_are_short_by_exactly_the_clawback() {
    let paid = baseline::frame();
    let held: Vec<_> = paid.iter().filter(|row| row.on_guarantee()).collect();
    let short: Vec<_> = held
        .iter()
        .filter(|row| !row.fully_on_guarantee())
        .collect();

    assert_eq!(held.len(), 185, "FY2025 districts drawing a guarantee");
    assert_eq!(short.len(), 16, "of which are paid less than their base");

    for row in &short {
        let gap = row.funding_base - row.foundation - row.guarantee;
        assert!(
            (gap - row.open_enrollment_adjustment).abs() < 0.005,
            "{} ({}) is short {gap} against a clawback of {}",
            row.name,
            row.irn,
            row.open_enrollment_adjustment
        );
    }

    // And with the clawback in hand every guaranteed district closes, so the weak test and the
    // full identity agree on the whole population.
    assert!(
        held.iter().all(|row| row.held_at_its_base()),
        "every guaranteed district closes once the clawback is counted"
    );
    // A clawback is recorded for more districts than it reduces. 41 carry one; the 25 that are
    // not on the guarantee are unaffected by it, because the guarantee is a `max(.., 0)` and a
    // district the formula already pays has nothing for the clawback to come out of. So the
    // sixteen are the intersection, not the population — which is why a count of clawbacks would
    // not have predicted them.
    let with_clawback = paid
        .iter()
        .filter(|row| row.open_enrollment_adjustment > 0.0)
        .count();
    assert_eq!(with_clawback, 41, "districts carrying a clawback at all");
    assert!(
        short.iter().all(|row| row.open_enrollment_adjustment > 0.0),
        "every short district carries one"
    );
    let clawed_and_unguaranteed = paid
        .iter()
        .filter(|row| row.open_enrollment_adjustment > 0.0 && !row.on_guarantee())
        .count();
    assert_eq!(clawed_and_unguaranteed, 41 - 16);
}

/// FY2026's summary sheet closes for every guaranteed district without needing the clawback.
#[test]
fn every_guaranteed_district_in_fy2026_is_restored_to_its_base_exactly() {
    let prior = prior_model::frame();
    let held: Vec<_> = prior.iter().filter(|row| row.guarantee > 0.0).collect();
    assert_eq!(held.len(), 235, "FY2026 districts drawing a guarantee");
    for row in &held {
        let closes = row.foundation_funding + row.guarantee - row.funding_base;
        assert!(
            closes.abs() < 0.005,
            "{} ({}) is off by {closes}",
            row.name,
            row.irn
        );
    }
}

/// Delphos's foundation line does not move at all; Shawnee's falls by its own FY2025 phase-in.
///
/// Both are restored to their base in FY2026, and the base is the same number in both years, so
/// from FY2026 onward the formula pays each of them a fixed amount whatever it computes — and
/// Delphos's `[Hb]` falls 20% between these two years while its payment does not move a cent.
///
/// They differ in FY2025. Delphos was already held at its base. Shawnee was **above** it, funded
/// at `[Ha] + [Hc]` with `[Hc]` positive, and drew no guarantee; arriving on the guarantee in
/// FY2026 costs it exactly that step back. So the foundation line contributes nothing for Delphos
/// across the biennium and a **loss** for Shawnee — in neither case a gain, which is why the
/// difference between how the two fare has to be looked for outside the formula.
#[test]
fn the_foundation_line_is_flat_for_delphos_and_falls_for_shawnee() {
    let fy26: Vec<_> = prior_model::frame();
    let at_fy26 = |irn: &str| {
        fy26.iter()
            .find(|row| row.irn == irn)
            .expect("in the FY2026 model")
    };

    for irn in [DELPHOS, SHAWNEE] {
        let (before, after) = (baseline::at(irn).expect("in the report"), at_fy26(irn));
        assert!(
            (after.foundation_funding + after.guarantee - after.funding_base).abs() < 0.005,
            "{irn} is restored to its base in FY2026"
        );
        assert!(
            (before.funding_base - after.funding_base).abs() < 0.005,
            "{irn}: the base is the same number in both years"
        );
        // The formula computes a great deal less while the guarantee is what pays.
        assert!(
            after.foundation_calculated < before.foundation_calculated * 0.85,
            "{irn}: [Hb] {} against {}",
            after.foundation_calculated,
            before.foundation_calculated
        );
    }

    let (delphos, shawnee) = (
        baseline::at(DELPHOS).expect("Delphos"),
        baseline::at(SHAWNEE).expect("Shawnee"),
    );
    assert!(delphos.fully_on_guarantee(), "Delphos was already restored");
    assert!(!shawnee.on_guarantee(), "Shawnee drew nothing in FY2025");

    // Delphos: paid its base in both years, so the line is flat.
    let delphos_moved = at_fy26(DELPHOS).foundation_funding + at_fy26(DELPHOS).guarantee
        - delphos.foundation
        - delphos.guarantee;
    assert!(
        delphos_moved.abs() < 0.005,
        "Delphos's foundation line moved by {delphos_moved}"
    );

    // Shawnee: paid above its base in FY2025 by exactly [Hc], and restored to the base after.
    let shawnee_moved =
        at_fy26(SHAWNEE).foundation_funding + at_fy26(SHAWNEE).guarantee - shawnee.foundation;
    assert!(
        (shawnee_moved + shawnee.foundation_phase_in).abs() < 0.005,
        "Shawnee gave back {} against an FY2025 step up of {}",
        -shawnee_moved,
        shawnee.foundation_phase_in
    );
    assert!(
        shawnee_moved < -263_000.0,
        "and it is a real sum: {shawnee_moved}"
    );
}
