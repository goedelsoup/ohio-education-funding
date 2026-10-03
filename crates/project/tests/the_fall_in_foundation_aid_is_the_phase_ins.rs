//! The fall in foundation aid across the biennium is the phase-in's (#695).
//!
//! `/what-changed` prints −$114.5M twice: foundation aid fell that much from FY2025 to FY2027,
//! and the phase-in paid that much less above bases. They are the same money. The FY2020 funding
//! base is the same figure in all three years, so foundation aid can only move through what is
//! paid above it:
//!
//! ```text
//! foundation aid = Σ funding base + Σ paid above base − Σ open enrollment clawback
//! ```
//!
//! The last term is the whole of the residual: 16 districts in FY2025 and 22 in FY2027 are paid
//! below their base by `[I1]`, and the FY2026 model pays none below it. The clawback is about $3M
//! in each end year, so it nearly cancels out of the change.

use std::collections::HashMap;

use edfund_core::{Dollars, FiscalYear};
use project::baseline;
use project::biennium::{self, BASELINE_YEAR, MIDDLE_YEAR};
use project::panel::{panel, MODEL_YEAR};

const YEARS: [FiscalYear; 3] = [BASELINE_YEAR, MIDDLE_YEAR, MODEL_YEAR];

fn close(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

/// Per year: Σ funding base, Σ foundation aid, Σ paid above base.
fn sums() -> [(Dollars, Dollars, Dollars); 3] {
    let mut out = [(0.0, 0.0, 0.0); 3];
    for row in biennium::frame() {
        for (i, year) in YEARS.into_iter().enumerate() {
            let phase_in = row.years[i].phase_in;
            out[i].0 += phase_in.funding_base;
            out[i].1 += row.foundation.at(year).expect("observed");
            out[i].2 += phase_in.above_base();
        }
    }
    out
}

/// $6,395.28M in every year: three files agreeing on one base, summed over the 609.
#[test]
fn the_bases_sum_to_one_figure_in_every_year() {
    let [a, b, c] = sums().map(|(base, _, _)| base);
    assert!(close(a, 6_395_281_649.73, 1.0), "{a}");
    assert!(close(a, b, 1.0) && close(b, c, 1.0), "{a} / {b} / {c}");
}

/// Foundation aid is base plus what the phase-in pays above it, less the clawback, in every year
/// and every district, to the cent.
///
/// FY2025's clawback is read off the payment report's `[I1]`: the guarantee is
/// `max([Ha] − [I1] − [Hd], 0)`, so a district loses the whole adjustment or, where the guarantee
/// was smaller, the whole guarantee. FY2027's is the panel's own
/// [`project::panel::DistrictRecord::open_enrollment_withheld`]. FY2026 publishes no `[I1]`
/// column and pays no district below its base.
#[test]
fn foundation_aid_is_the_base_plus_what_is_paid_above_it_less_the_clawback() {
    let fy2025: HashMap<_, _> = baseline::frame()
        .into_iter()
        .map(|row| {
            let shortfall = (row.funding_base - row.foundation).max(0.0);
            (row.irn, shortfall.min(row.open_enrollment_adjustment))
        })
        .collect();
    let fy2027: HashMap<_, _> = panel()
        .into_iter()
        .map(|record| (record.irn.clone(), record.open_enrollment_withheld()))
        .collect();

    let mut reached = [0; 3];
    let mut withheld = [0.0; 3];
    for row in biennium::frame() {
        let irn = &row.total.irn;
        let clawback = [fy2025[irn], 0.0, fy2027[irn]];
        for (i, year) in YEARS.into_iter().enumerate() {
            let phase_in = row.years[i].phase_in;
            let aid = row.foundation.at(year).expect("observed");
            let residual = aid - phase_in.funding_base - phase_in.above_base();
            assert!(
                close(residual, -clawback[i], 0.02),
                "{} ({irn}) FY{}: {residual} against a clawback of {}",
                row.total.name,
                year.0,
                clawback[i]
            );
            reached[i] += usize::from(clawback[i] >= 0.01);
            withheld[i] += clawback[i];
        }
    }
    assert_eq!(reached, [16, 0, 22]);
    assert!(close(withheld[0], 3_073_454.87, 1.0), "{}", withheld[0]);
    assert!(close(withheld[2], 3_037_536.64, 1.0), "{}", withheld[2]);
}

/// So the change in foundation aid is the change paid above base, to within the clawback's own
/// change: −$114.50M against −$114.53M, $35,918 apart.
#[test]
fn the_change_in_foundation_aid_is_the_change_paid_above_base() {
    let [first, _, last] = sums();
    let aid = last.1 - first.1;
    let above = last.2 - first.2;
    assert!(close(first.2, 1_003_514_918.64, 1.0), "{}", first.2);
    assert!(close(last.2, 888_983_478.56, 1.0), "{}", last.2);
    assert!(close(aid, -114_495_521.85, 1.0), "{aid}");
    assert!(close(aid, above, 50_000.0), "{aid} against {above}");
}
