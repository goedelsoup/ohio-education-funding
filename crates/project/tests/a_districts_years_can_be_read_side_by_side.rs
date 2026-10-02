//! The year-over-year detail behind the biennium's two measures, pinned on Allen County.
//!
//! Lima City was one of two Allen County districts paid less in FY2026 than in FY2025 while its
//! neighbours gained. The detail here is what tells a reader why, and each reason is a different
//! line: Lima lost a repealed supplement, Elida a phase-in step its reappraisal took away. Every
//! figure is observed; none is modelled.

use project::biennium::{self, Row, BASELINE_YEAR, MIDDLE_YEAR, VALUATION_TAX_YEARS};
use project::panel::MODEL_YEAR;

const LIMA: &str = "044222";
const ELIDA: &str = "045773";
const DELPHOS: &str = "043885";

/// The nine districts the department attributes to Allen County.
const ALLEN: [&str; 9] = [
    "045757", "045807", "045211", DELPHOS, "045781", "045765", ELIDA, "045799", LIMA,
];

fn row(irn: &str) -> Row {
    biennium::at(irn).unwrap_or_else(|| panic!("{irn} is in the comparison"))
}

fn close(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

/// The named supplements are the residual [`biennium::Lines::supplements`] already carried.
///
/// To within cent rounding: five lines each rounded to the cent can differ from their rounded sum
/// by a few cents, and that is the whole gap. Should an act add a supplement this does not name,
/// the gap is that supplement and this fails.
#[test]
fn the_named_supplements_are_the_whole_of_the_residual() {
    for row in biennium::frame() {
        let named = row.years[2].supplements.total() - row.years[0].supplements.total();
        assert!(
            close(named, row.lines.supplements, 0.03),
            "{} ({}): named supplements moved {named}, the residual {}",
            row.total.name,
            row.total.irn,
            row.lines.supplements
        );
    }
}

/// Net funding is total state support plus transfers, in every year that publishes them.
#[test]
fn transfers_close_total_state_support_onto_net_funding() {
    for row in biennium::frame() {
        for (year, detail) in [BASELINE_YEAR, MIDDLE_YEAR, MODEL_YEAR]
            .into_iter()
            .zip(row.years)
        {
            let Some(transfers) = detail.transfers else {
                assert_eq!(year, MIDDLE_YEAR, "only FY2026 publishes no transfers");
                continue;
            };
            let total = row.total.at(year).expect("observed");
            assert!(close(
                total + transfers.total,
                transfers.net_state_funding,
                0.005
            ));
            if let (Some(charge), Some(other)) = (transfers.service_center, transfers.other) {
                assert!(close(charge + other, transfers.total, 0.005));
            }
        }
    }
}

/// FY2026's transfers are absent, not zero, and FY2025's are a total with no parts.
#[test]
fn a_year_that_does_not_publish_transfers_says_so_rather_than_reporting_none() {
    let lima = row(LIMA);
    let [fy25, fy26, fy27] = lima.years;
    assert!(fy26.transfers.is_none());
    let fy25 = fy25.transfers.expect("FY2025 states net funding");
    assert!(fy25.service_center.is_none() && fy25.other.is_none());
    assert!(close(fy25.total, -616_425.89, 0.005));
    let fy27 = fy27.transfers.expect("FY2027 itemizes");
    assert_eq!(fy27.service_center, Some(-269_453.0));
}

/// **Lima and Elida are the only Allen County districts paid less in FY2026 than FY2025.**
#[test]
fn two_of_allen_countys_nine_were_cut_in_the_first_year_of_the_biennium() {
    let mut cut: Vec<String> = ALLEN
        .iter()
        .map(|irn| row(irn))
        .filter(|row| row.total.against_baseline(MIDDLE_YEAR).expect("observed") < 0.0)
        .map(|row| row.total.irn)
        .collect();
    cut.sort();
    assert_eq!(cut, [LIMA, ELIDA]);

    let lima = row(LIMA);
    assert!(close(
        lima.total.at(BASELINE_YEAR).unwrap(),
        39_036_842.48,
        1.0
    ));
    assert!(close(
        lima.total.at(MIDDLE_YEAR).unwrap(),
        37_900_888.0,
        1.0
    ));
}

/// Lima's cut is the repealed supplement, and its phase-in could not offset it.
///
/// Lima's formula computes within 0.3% of its FY2020 base in all three years, so the phase-in
/// paid it $11,435 in FY2026 however fast the rate rose. The $1.86m of targeted assistance it lost
/// had nothing to replace it.
#[test]
fn lima_lost_targeted_assistance_and_the_phase_in_had_nothing_to_give_it() {
    let [fy25, fy26, fy27] = row(LIMA).years;
    assert!(close(
        fy25.supplements.targeted_assistance,
        1_859_366.55,
        0.005
    ));
    assert_eq!(fy26.supplements.targeted_assistance, 0.0);
    assert_eq!(fy27.supplements.targeted_assistance, 0.0);
    assert!(close(fy26.phase_in.step(), 11_434.71, 0.005));
    for year in [fy25, fy26, fy27] {
        assert!(close(
            year.phase_in.calculated,
            year.phase_in.funding_base,
            110_000.0
        ));
    }
}

/// The repeal is a category, not a district: about half its recipients were cut, against under a
/// quarter of everyone else.
#[test]
fn the_repeal_cut_half_its_recipients_and_under_a_quarter_of_the_rest() {
    let (mut recipients, mut recipients_cut, mut rest, mut rest_cut) = (0, 0, 0, 0);
    for row in biennium::frame() {
        let cut = row.total.against_baseline(MIDDLE_YEAR).expect("observed") < 0.0;
        if row.years[0].supplements.targeted_assistance > 0.0 {
            recipients += 1;
            recipients_cut += usize::from(cut);
        } else {
            rest += 1;
            rest_cut += usize::from(cut);
        }
    }
    assert_eq!((recipients_cut, recipients), (19, 36));
    assert_eq!((rest_cut, rest), (135, 573));
}

/// Elida's cut is the phase-in shrinking under a reappraisal.
///
/// Its valuation rose 27% from tax year 2023 to 2024, its state share fell, and the formula's
/// output fell toward its base faster than the rate rose to meet it.
#[test]
fn elidas_phase_in_step_shrank_as_its_valuation_rose() {
    let elida = row(ELIDA);
    let [fy25, fy26, fy27] = elida.years;
    assert!(close(fy25.phase_in.step(), 976_918.97, 0.005));
    assert!(close(fy26.phase_in.step(), 412_217.63, 0.005));
    assert!(
        fy27.phase_in.step() < 0.0,
        "below its base by FY2027; the guarantee holds it"
    );

    assert_eq!(VALUATION_TAX_YEARS, [2023, 2024, 2025]);
    let [ty23, ty24, _] = elida.valuation;
    assert!(ty24 / ty23 > 1.27);
    assert!(fy27.state_share < fy26.state_share);
}

/// State share is published for FY2026 and FY2027, and Delphos is on the 10% minimum.
#[test]
fn state_share_is_absent_in_fy2025_and_delphos_reaches_the_minimum() {
    let [fy25, fy26, fy27] = row(DELPHOS).years;
    assert!(fy25.state_share.is_none());
    assert!(close(
        fy26.state_share.expect("published"),
        0.165_446_2,
        1e-7
    ));
    assert_eq!(fy27.state_share, Some(0.1));
}

/// The FY2027 phase-in is complete, so calculated and paid are one figure.
#[test]
fn the_final_year_pays_what_it_calculates() {
    for row in biennium::frame() {
        let fy27 = row.years[2].phase_in;
        assert_eq!(fy27.calculated, fy27.paid, "{}", row.total.irn);
    }
}

/// The phase-in interpolates from one base, and the three files agree on it.
///
/// Each year's file states the FY2020 funding base separately, so this is three publications
/// agreeing rather than one figure copied — and a page may draw a single base line under all three.
#[test]
fn every_year_states_the_same_funding_base() {
    for row in biennium::frame() {
        let [fy25, fy26, fy27] = row.years.map(|year| year.phase_in.funding_base);
        assert!(
            close(fy25, fy26, 0.005) && close(fy26, fy27, 0.005),
            "{}: {fy25} / {fy26} / {fy27}",
            row.total.irn
        );
    }
}

/// Each year is its named lines: foundation aid, the three paid lines and the named supplements
/// sum to total state support, to within cent rounding, in all three years and every district.
///
/// This is what lets a page draw the three years line by line with a total under them, rather
/// than only the FY2025→FY2027 change [`biennium::Lines`] carries.
#[test]
fn every_year_is_its_named_lines() {
    for row in biennium::frame() {
        for (year, detail) in [BASELINE_YEAR, MIDDLE_YEAR, MODEL_YEAR]
            .into_iter()
            .zip(row.years)
        {
            let named = row.foundation.at(year).expect("observed")
                + detail.paid.transportation
                + detail.paid.special_education_transportation
                + detail.paid.preschool_special_education
                + detail.supplements.total();
            let total = row.total.at(year).expect("observed");
            assert!(
                close(named, total, 0.03),
                "{} ({}) FY{}: lines sum to {named}, total state support is {total}",
                row.total.name,
                row.total.irn,
                year.0
            );
        }
    }
}

/// The levels agree with the change [`biennium::Lines`] already carried.
#[test]
fn the_levels_difference_to_the_lines() {
    for row in biennium::frame() {
        let [first, _, last] = row.years.map(|year| year.paid);
        for (change, line) in [
            (
                last.transportation - first.transportation,
                row.lines.transportation,
            ),
            (
                last.special_education_transportation - first.special_education_transportation,
                row.lines.special_education_transportation,
            ),
            (
                last.preschool_special_education - first.preschool_special_education,
                row.lines.preschool_special_education,
            ),
        ] {
            assert!(close(change, line, 0.005), "{}", row.total.irn);
        }
    }
}

/// Enrolled ADM is published for the two model years and not for the payment report.
#[test]
fn enrolled_adm_is_absent_in_fy2025() {
    for row in biennium::frame() {
        let [fy25, fy26, fy27] = row.years.map(|year| year.enrolled_adm);
        assert!(fy25.is_none());
        assert!(fy26.is_some_and(|adm| adm > 0.0) && fy27.is_some_and(|adm| adm > 0.0));
    }
}

/// DPIA is carried for the two model years, and over the 609 it is the program's whole fall.
///
/// $567.7M in the FY2026 model and $525.1M in the FY2027 one, 7.5% lower, in 562 districts: the
/// blend moving from 75/25 to 65/35 toward a directly certified count that was itself restated
/// lower. The figures `the_two_clocks_the_plan_runs_on.rs` pins over each file's own rows, read
/// here over the comparison's population.
#[test]
fn dpia_is_absent_in_fy2025_and_falls_across_the_model_years() {
    let frame = biennium::frame();
    let (mut fy26, mut fy27, mut fell) = (0.0, 0.0, 0);
    for row in &frame {
        let [a, b, c] = row.years.map(|year| year.dpia);
        assert!(a.is_none(), "{}", row.total.irn);
        let (b, c) = (
            b.expect("FY2026 models DPIA"),
            c.expect("FY2027 models DPIA"),
        );
        fy26 += b;
        fy27 += c;
        fell += usize::from(c < b - 0.005);
    }
    assert!(close(fy26, 567_673_868.76, 1.0), "{fy26}");
    assert!(close(fy27, 525_094_312.31, 1.0), "{fy27}");
    assert_eq!(fell, 562);
}
