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
//! - Enrollment: the worked example published in the FY2022 payment report — 20,342.13 base cost
//!   enrolled ADM, 43 open buildings. `[verified]` Only the enrollment is that report's; the
//!   prices below are not, so the frozen totals here are **not** the FY2022 report's totals.
//!   Those are reproduced in `foundation`'s own unit tests, against
//!   [`StatewideFactors::fy2022`].
//! - Price vector: [`StatewideFactors::fy2027`] — the FY2022 averages H.B. 96 carries forward,
//!   which is what "frozen" means for this scenario's FY2026–FY2027 horizon. `[verified]`
//! - FY2022 reference salary $68,022.22 — that same factor set's `teacher_salary`, read off the
//!   department's own `FY27 TRAD State Foundation Funding Calculator`. `[verified]`
//! - FY2024 refreshed salary $73,777.08 — ADM-weighted average of classroom teachers' average
//!   salary across all 606 districts in the FY2024 District Profile Report. `[verified]`
//! - Minimum state share: `local_capacity::MINIMUM_STATE_SHARE_FY2027`, 10%. The department's FY2027
//!   calculator states `0.1` for both FY2026 and FY2027 on its `Notes` sheet. `[verified]`
//!
//! One vintage throughout, which this run did not previously have: it priced the FY2027 teacher
//! salary into the FY2022 factor set and floored the result at the FY2022 5% minimum. The
//! *delta* is unaffected — it is positions times the salary gap times the benefit multiplier,
//! and none of those moved — but the levels are, and so is where the floor binds.
//!
//! # The population the incidence table answers for
//!
//! One district is not a state, and the share column below is a share *among the districts the
//! formula pays*. A district the temporary transitional aid guarantee holds is off-formula: its
//! aid is its FY2020 floor, so a rise in its computed base cost shrinks the top-up by the same
//! sum and reaches it as nothing at all. Printing 100% without that population beside it reads as
//! statewide coverage, and it is not.
//!
//! So the run counts the guarantee off [`foundation::department_model`] — the department's own
//! FY2027 calculator, the same fixture the base cost check runs against — and states both halves.
//! What a refresh actually *delivers* across all 609, after the guarantee, the minimum state
//! share and the three-year ADM denominator, is `project::refresh` and
//! `crates/scenario-delta/tests/who_a_change_reaches.rs`; this run has one district's capacity
//! and cannot say which side of the floor that district falls on.

use foundation::department_model;
use foundation::{
    aggregate_base_cost, published_worked_example, refresh_worked_example, StatewideFactors,
    FY2024_TEACHER_SALARY,
};
use local_capacity::state_share;

/// The FY2022 reference-year teacher salary carried forward by H.B. 96. [verified]
const FY2022_TEACHER_SALARY: f64 = StatewideFactors::fy2027().teacher_salary;

/// The capacities the incidence table is sampled at.
///
/// The illustrative points are a fixed sweep across the wealth scale. The three derived ones are
/// the band this perturbation opens: the frozen run's floor threshold, the refreshed run's, and
/// the midpoint between them. They are **computed** rather than written down because the
/// published table's headline — a clean jump from 100% to the floor rate — was an artifact of a
/// grid that stepped over that band. A grid that names its own thresholds cannot step over them
/// again when a later factor vintage moves base cost per pupil.
fn capacity_grid(run: &foundation::WorkedExampleRefresh) -> Vec<f64> {
    let (lower, upper) = (
        run.floor_binds_above_frozen,
        run.floor_binds_above_refreshed,
    );
    let mut grid = vec![
        500.0,
        1_500.0,
        2_027.0,
        4_000.0,
        6_000.0,
        7_000.0,
        lower,
        f64::midpoint(lower, upper),
        upper,
        12_000.0,
    ];
    grid.sort_by(f64::total_cmp);
    grid
}

fn main() {
    // The run itself is `foundation::refresh_worked_example`, so `crates/figures` binds exactly
    // what this prints. What is left here is the printing, and the component build-up that only
    // the table needs.
    let run = refresh_worked_example();
    let floor = run.minimum_state_share;
    let district = published_worked_example();
    let adm = district.base_cost_enrolled_adm;

    // Both runs are priced at the FY2027 factor set, which *is* the frozen policy: it is the
    // FY2022 price vector H.B. 96 carries forward, and its `teacher_salary` is the reference
    // this scenario perturbs away from. The frozen run therefore needs no splice at all.
    let frozen = StatewideFactors::fy2027();
    let refreshed = StatewideFactors {
        teacher_salary: FY2024_TEACHER_SALARY,
        ..StatewideFactors::fy2027()
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
    // entirely on where a district sits on the capacity scale — *given* that the formula is
    // what pays the district at all. The guarantee block below is the other axis, and the
    // column header carries its own population so the table cannot be read as statewide.
    println!();
    println!("INCIDENCE — who receives the increase, by local capacity per pupil");
    println!("            among the districts the formula pays; see GUARANTEE below");
    println!();
    println!(
        "{:>14} {:>14} {:>14} {:>16} {:>20}",
        "capacity/pupil", "state frozen", "state refreshed", "gain/pupil", "share, if on formula"
    );
    for capacity in capacity_grid(&run) {
        let s_frozen = state_share(a.per_pupil, capacity, adm, floor).unwrap();
        let s_refreshed = state_share(b.per_pupil, capacity, adm, floor).unwrap();
        let gain = s_refreshed.percentage * b.per_pupil - s_frozen.percentage * a.per_pupil;
        let captured = run.captured_at(capacity);
        let marker = match (s_frozen.at_minimum, s_refreshed.at_minimum) {
            (true, true) => format!("  ({:.0}% floor)", floor * 100.0),
            (true, false) => "  (straddles)".to_string(),
            _ => String::new(),
        };
        println!(
            "{:>14} {:>14} {:>14} {:>16} {:>19.1}%{marker}",
            format!("${capacity:>12.2}"),
            format!("${:>12.0}", s_frozen.percentage * a.per_pupil),
            format!("${:>12.0}", s_refreshed.percentage * b.per_pupil),
            format!("${gain:>+14.2}"),
            captured * 100.0
        );
    }

    println!();
    println!(
        "The state pays the residual, so a district on formula receives 100% of any base cost\n\
         increase, and a district the floor holds in both runs receives {:.0}% of it. Between\n\
         them lies a band ${:.2} wide — ${:.2} to ${:.2} of capacity per pupil — where the floor\n\
         binds before the refresh and the formula pays after it. A district there captures\n\
         something strictly between the two, falling continuously from 100% to {:.0}% across the\n\
         band. The transition is not the floor itself: the floor is a share OF base cost per\n\
         pupil, and this perturbation moves base cost per pupil, so the two runs reach it at\n\
         different capacities. Refreshing the input year is progressive and freezing it is\n\
         regressive — uniform in method, not in effect.",
        floor * 100.0,
        run.straddling_band_width(),
        run.floor_binds_above_frozen,
        run.floor_binds_above_refreshed,
        floor * 100.0
    );

    guarantee_population();

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
        "The state's share of that is NOT the same number: it is the full amount for districts on\n\
         formula, {:.0}% for districts the floor holds in both runs, a share between the two for\n\
         the band that straddles it, and nothing at all for the ones the guarantee holds, before\n\
         the phase-in percentage is applied. Converting it to a state appropriation needs\n\
         per-district capacity and guarantee status; this run sweeps the first across one district\n\
         and has the second for none of them.",
        floor * 100.0
    );
}

/// How many districts the column above answers for, counted off the department's own model.
///
/// The capacity table is a function of one axis and the guarantee is a second one. A district the
/// guarantee holds is paid its FY2020 floor rather than its formula amount: when the formula
/// amount rises the top-up shrinks by the same sum, so the district captures none of the increase
/// unless the rise carries it past the floor. Its share is zero at every capacity in that table.
///
/// This run has one district and cannot say which side of that floor it falls on — so what it can
/// honestly do is name the population, which is what this prints. The counts are the department's
/// [`department_model`] fixture, the same one `tests/department_model_fy27.rs` asserts them from,
/// counted here rather than restated as literals.
fn guarantee_population() {
    let districts = department_model::districts();
    let guaranteed: Vec<_> = districts.iter().filter(|d| d.on_guarantee()).collect();
    let on_formula = districts.len() - guaranteed.len();
    let share = |n: usize| 100.0 * n as f64 / districts.len() as f64;

    let all_adm: f64 = districts.iter().map(|d| d.base_cost_adm).sum();
    let guaranteed_adm: f64 = guaranteed.iter().map(|d| d.base_cost_adm).sum();

    println!();
    println!("GUARANTEE — the population that share column describes");
    println!();
    println!(
        "{:<48} {:>4}",
        "districts in the department's FY2027 model",
        districts.len()
    );
    println!(
        "{:<48} {:>4}   ({:.1}%)",
        "  paid by the formula, so the column answers for",
        on_formula,
        share(on_formula)
    );
    println!(
        "{:<48} {:>4}   ({:.1}%, {:.1}% of ADM)",
        "  held on the guarantee, so it does not",
        guaranteed.len(),
        share(guaranteed.len()),
        100.0 * guaranteed_adm / all_adm
    );
    println!();
    println!(
        "A guaranteed district is paid its FY2020 floor, not its formula amount, so a rise in\n\
         computed base cost shrinks its top-up by the same sum and reaches it as nothing — at\n\
         every capacity in the table above. The share column is a share among the {on_formula}\n\
         districts the formula pays, not statewide coverage. What a refresh delivers across all\n\
         {}, after the guarantee, the minimum state share and the three-year ADM denominator, is\n\
         `project::refresh` and `crates/scenario-delta/tests/who_a_change_reaches.rs`.",
        districts.len()
    );
}
