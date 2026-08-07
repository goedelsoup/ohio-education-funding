//! Run the seeded scenario: FSFP cost input refresh versus freeze.
//!
//! Answers `.yidam/corpus/scenario/fsfp-input-year-refresh.yml`. Run from `crates/`:
//!
//! ```text
//! cargo run --example input_year_refresh -p foundation
//! ```
//!
//! # What is perturbed
//!
//! Only the statewide average **classroom teacher salary**, from the FY2022 reference year
//! H.B. 96 carries forward to an FY2024 refresh. Everything else is held at enacted values:
//! staffing ratios, minimums, ADM definitions, the other nine salary categories, per-pupil
//! amounts, and the phase-in percentage.
//!
//! Holding the other nine salaries fixed makes this a **lower bound** on the effect of a real
//! refresh. Administrator, counselor, librarian, and clerical salaries have all risen too;
//! this run measures the teacher term alone.
//!
//! # Inputs and their provenance
//!
//! - District: the worked example published in the FY2022 payment report — 20,342.13 base cost
//!   enrolled ADM, 43 open buildings. `[verified]`
//! - FY2022 reference salary $67,654 — secondary reporting, not the department's own figure.
//!   `[inference]`
//! - FY2024 refreshed salary $73,777.08 — ADM-weighted average of classroom teachers' average
//!   salary across all 606 districts in the FY2024 District Profile Report. `[verified]`

use foundation::{aggregate_base_cost, DistrictEnrollment, StatewideFactors};
use local_capacity::{state_share, MINIMUM_STATE_SHARE_FY2022};

/// The FY2022 reference-year teacher salary carried forward by H.B. 96. [inference]
const FY2022_TEACHER_SALARY: f64 = 67_654.0;
/// ADM-weighted statewide average classroom teacher salary, FY2024. [verified]
const FY2024_TEACHER_SALARY: f64 = 73_777.08;

fn published_district() -> DistrictEnrollment {
    DistrictEnrollment {
        kindergarten: 1_630.68,
        grades_1_3: 4_797.69,
        grades_4_8: 7_559.90,
        grades_9_12: 5_075.82,
        career_technical: 1_278.03,
        grades_9_12_total: 6_090.96,
        base_cost_enrolled_adm: 20_342.13,
        open_buildings: 43.0,
        athletics_eligible: true,
    }
}

fn main() {
    let district = published_district();
    let adm = district.base_cost_enrolled_adm;

    let frozen = StatewideFactors {
        teacher_salary: FY2022_TEACHER_SALARY,
        ..StatewideFactors::fy2022()
    };
    let refreshed = StatewideFactors {
        teacher_salary: FY2024_TEACHER_SALARY,
        ..StatewideFactors::fy2022()
    };

    let a = aggregate_base_cost(&district, &frozen);
    let b = aggregate_base_cost(&district, &refreshed);

    println!("SCENARIO  fsfp-input-year-refresh");
    println!("District  worked example, {adm:.2} base cost enrolled ADM, 43 buildings");
    println!();
    println!(
        "Teacher salary input   FY2022 ${FY2022_TEACHER_SALARY:>10.2}  ->  FY2024 ${FY2024_TEACHER_SALARY:>10.2}   ({:+.1}%)",
        (FY2024_TEACHER_SALARY / FY2022_TEACHER_SALARY - 1.0) * 100.0
    );
    println!();
    println!(
        "{:<38} {:>16} {:>16} {:>14}",
        "component", "frozen", "refreshed", "delta"
    );
    let rows: [(&str, f64, f64); 6] = [
        ("A teacher", a.teacher.total, b.teacher.total),
        (
            "B student support",
            a.student_support.total,
            b.student_support.total,
        ),
        (
            "C district leadership",
            a.district_leadership.total,
            b.district_leadership.total,
        ),
        (
            "D building leadership and operation",
            a.building_leadership.total,
            b.building_leadership.total,
        ),
        (
            "E athletic co-curricular",
            a.athletic_cocurricular,
            b.athletic_cocurricular,
        ),
        ("   aggregate base cost", a.aggregate, b.aggregate),
    ];
    for (label, frozen_v, refreshed_v) in rows {
        println!(
            "{label:<38} {:>16} {:>16} {:>14}",
            format!("${frozen_v:>14.0}"),
            format!("${refreshed_v:>14.0}"),
            format!("${:>+12.0}", refreshed_v - frozen_v)
        );
    }
    println!();
    println!(
        "base cost per pupil                    ${:>13.2}  ${:>13.2}  ${:>+11.2}  ({:+.2}%)",
        a.per_pupil,
        b.per_pupil,
        b.per_pupil - a.per_pupil,
        (b.per_pupil / a.per_pupil - 1.0) * 100.0
    );

    // Incidence: the state pays the residual after local capacity, so who gains depends
    // entirely on where a district sits on the capacity scale.
    println!();
    println!("INCIDENCE — who receives the increase, by local capacity per pupil");
    println!();
    println!(
        "{:>14} {:>14} {:>14} {:>16} {:>10}",
        "capacity/pupil", "state frozen", "state refreshed", "gain/pupil", "share"
    );
    for capacity in [
        500.0_f64, 1_500.0, 2_027.0, 4_000.0, 6_000.0, 7_000.0, 12_000.0,
    ] {
        let s_frozen = state_share(a.per_pupil, capacity, adm, MINIMUM_STATE_SHARE_FY2022).unwrap();
        let s_refreshed =
            state_share(b.per_pupil, capacity, adm, MINIMUM_STATE_SHARE_FY2022).unwrap();
        let gain = s_refreshed.percentage * b.per_pupil - s_frozen.percentage * a.per_pupil;
        let captured = gain / (b.per_pupil - a.per_pupil);
        let marker = if s_frozen.at_minimum {
            "  (5% floor)"
        } else {
            ""
        };
        println!(
            "{:>14} {:>14} {:>14} {:>16} {:>9.0}%{marker}",
            format!("${capacity:>12.0}"),
            format!("${:>12.0}", s_frozen.percentage * a.per_pupil),
            format!("${:>12.0}", s_refreshed.percentage * b.per_pupil),
            format!("${gain:>+14.2}"),
            captured * 100.0
        );
    }

    println!();
    println!(
        "The state pays the residual, so a district on formula receives 100% of any base cost\n\
         increase. A district held at the {:.0}% floor receives {:.0}% of it. Refreshing the\n\
         input year is therefore progressive, and freezing it is regressive — uniform in method,\n\
         not in effect.",
        MINIMUM_STATE_SHARE_FY2022 * 100.0,
        MINIMUM_STATE_SHARE_FY2022 * 100.0
    );
    println!();
    println!(
        "This district's computed base cost rises ${:.0}. Ohio's FY2022 base cost enrolled ADM\n\
         was 1,499,918.26, so at ${:.2} per pupil a comparable refresh raises COMPUTED BASE COST\n\
         statewide by roughly ${:.0} million a year — from the teacher salary term alone.",
        b.aggregate - a.aggregate,
        b.per_pupil - a.per_pupil,
        (b.per_pupil - a.per_pupil) * 1_499_918.26 / 1_000_000.0
    );
    println!(
        "The state's share of that is NOT the same number: it is the full amount for districts\n\
         on formula and 5% for districts at the floor, before the phase-in percentage is applied.\n\
         Converting it to a state appropriation needs per-district capacity, which this run does\n\
         not have."
    );
}
