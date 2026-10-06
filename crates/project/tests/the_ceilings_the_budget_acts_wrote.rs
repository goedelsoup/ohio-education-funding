//! The Autism and Jon Peterson ceilings year by year, and the vintage of the text they were read
//! from.
//!
//! Issue #779. `program/autism-scholarship` set the FY2025 average award, $26,598.88, beside a
//! "statutory ceiling" of $34,000 read off R.C. 3317.022(A)(12) — and left open "whether it has
//! been raised by amendment rather than by index". The section in `revised-code.txt` is the text
//! H.B. 96 left, effective 30 September 2025. In FY2025 the ceiling was $32,445, and LSC says so in
//! the H.B. 96 analysis itself.
//!
//! # What these tests settle
//!
//! - **Every step is an amendment.** Each ceiling below is a sentence in LSC's analysis of the act
//!   that set it, and no act's analysis describes either one as indexed. The `[open]` closes.
//! - **The statute held is H.B. 96's, and only H.B. 96's schedule matches it.** The award
//!   `scholarship::jon_peterson_award` computes from the vendored text equals LSC's FY2026-FY2027
//!   table at all six positions and no earlier year's at any.
//! - **The department's award tables are enacted schedules, and the FY2023 edition prints
//!   FY2024's.** `the_three_years_the_jon_peterson_report_gives` read the tables as a uniform
//!   fraction of the statutory vector, recomputed by index each year, and so read two editions
//!   printing one table as an index that stood still. Two LSC analyses state the FY2023 schedule —
//!   $7,976 to $27,000 — and it is not what the FY2023 edition prints.

use edfund_core::FiscalYear;
use project::scholarship::ceilings::{self, Programme};
use project::scholarship::jpsn::{self, Series};
use project::scholarship::{self, report};
use project::statute;

fn year(fiscal_year: u16) -> FiscalYear {
    FiscalYear(fiscal_year)
}

/// `$32,445`, the way LSC prints a whole-dollar amount.
fn dollars(amount: f64) -> String {
    let whole = format!("{amount:.0}");
    let digits: Vec<char> = whole.chars().collect();
    let mut out = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*c);
    }
    format!("${out}")
}

#[test]
fn every_ceiling_is_a_sentence_in_the_analysis_of_the_act_that_set_it() {
    for programme in [Programme::Autism, Programme::JonPeterson] {
        for step in ceilings::steps(programme) {
            assert!(
                ceilings::analysis(step.act).contains(step.quote),
                "{} no longer says: {}",
                step.act,
                step.quote
            );
            assert!(
                step.quote.contains(&dollars(step.amount)),
                "{programme:?} FY{}: the quote does not state {}",
                step.from,
                dollars(step.amount)
            );
        }
    }

    // Three raises each since $20,000, none of them by index: two ceilings that moved together at
    // H.B. 64, apart at H.B. 110 and H.B. 33, and together again at H.B. 96.
    let acts = |p| -> Vec<&str> {
        let mut a: Vec<&str> = ceilings::steps(p).iter().map(|s| s.act).collect();
        a.dedup();
        a
    };
    assert_eq!(acts(Programme::Autism), ["hb64", "hb110", "hb96"]);
    assert_eq!(acts(Programme::JonPeterson), ["hb64", "hb33", "hb96"]);
}

#[test]
fn the_autism_ceiling_in_fy2025_was_32445_and_34000_is_hb96s() {
    let autism = |y| ceilings::ceiling(Programme::Autism, year(y));
    assert_eq!(autism(2015), None, "no committed step dates the $20,000");
    assert_eq!(autism(2021), Some(27_000.0));
    assert_eq!(autism(2022), Some(31_500.0));
    for held in 2023..=2025 {
        assert_eq!(autism(held), Some(32_445.0), "FY{held}");
    }
    assert_eq!(autism(2026), Some(34_000.0));
    assert_eq!(autism(2027), Some(34_000.0));
    assert_eq!(autism(2028), None, "no analysis reaches FY2028");

    // The vendored ceiling is the FY2026 one, and the section says which act wrote it.
    let section = statute::section("3317.022");
    assert!(
        section.legislation.contains("House Bill 96"),
        "{}",
        section.legislation
    );
    assert!((scholarship::AUTISM_CEILING - autism(2026).unwrap()).abs() < f64::EPSILON);
    assert!(scholarship::AUTISM_CEILING > autism(2025).unwrap());

    // The FY2025 average award against the ceiling in force that year, and against the one the
    // node used to quote.
    let average = report::programmes()["autism"]
        .published_average
        .expect("the report prints autism's average");
    let in_force = average / autism(2025).unwrap();
    let anachronistic = average / scholarship::AUTISM_CEILING;
    assert!((0.8198..0.8199).contains(&in_force), "{in_force:.5}");
    assert!(
        (0.7822..0.7824).contains(&anachronistic),
        "{anachronistic:.5}"
    );

    // H.B. 96's own words for the move: parity with Jon Peterson, which was already level.
    assert_eq!(
        ceilings::ceiling(Programme::JonPeterson, year(2025)),
        autism(2025)
    );
    assert!(ceilings::analysis("hb96").contains("providing parity with the JPSN capped amount"));
}

#[test]
fn every_schedule_lsc_prints_is_in_the_analysis_and_two_analyses_agree_on_each_shared_year() {
    // H.B. 110's table opens with a "Prior Law" column no fiscal year is attached to.
    let prior_law = [7_598.0, 10_025.0, 15_682.0, 18_861.0, 23_410.0, 27_000.0];
    // (act, the label its table gives the categories, years the table prints in column order)
    for (act, label, years) in [
        ("hb110", "Disability", vec![2022u16, 2023]),
        ("hb33", "Disability", vec![2023, 2024, 2025]),
        ("hb96", "Special Education", vec![2025, 2026]),
    ] {
        let text = ceilings::analysis(act);
        assert!(
            text.contains(label),
            "{act} no longer labels its categories {label}"
        );
        for category in 1..=6 {
            let mut cells: Vec<String> = years
                .iter()
                .map(|y| dollars(ceilings::jpsn_maxima(year(*y)).unwrap()[category - 1]))
                .collect();
            if act == "hb110" {
                cells.insert(0, dollars(prior_law[category - 1]));
            }
            let row = format!("Category {category} {}", cells.join(" "));
            assert!(text.contains(&row), "{act}: no row reads {row}");
        }
    }

    // The bases the category amounts sit on.
    assert!(ceilings::analysis("hb33")
        .contains("increases the base amount from $6,414 to $7,190 in FY 2024 and FY 2025"));
    assert!(ceilings::analysis("hb96")
        .contains("keeps the base per-pupil award amount flat at $7,190 in each fiscal year"));
    assert_eq!(ceilings::jpsn_base(year(2025)), Some(7_190.0));
    assert_eq!(ceilings::jpsn_base(year(2026)), Some(7_190.0));
}

#[test]
fn the_vendored_statute_computes_hb96s_schedule_and_no_earlier_one() {
    let statutory: Vec<f64> = (1..=6).map(scholarship::jon_peterson_award).collect();
    assert_eq!(
        statutory,
        ceilings::jpsn_maxima(year(2026)).unwrap(),
        "R.C. 3317.022(A)(13) as held is not LSC's FY2026-FY2027 table"
    );
    for earlier in 2022..=2025 {
        let table = ceilings::jpsn_maxima(year(earlier)).unwrap();
        assert!(
            statutory
                .iter()
                .zip(table)
                .all(|(s, t)| (s - t).abs() > 0.5),
            "FY{earlier} shares a position with the H.B. 96 text"
        );
    }
    assert!(
        (scholarship::JON_PETERSON_CEILING
            - ceilings::ceiling(Programme::JonPeterson, year(2026)).unwrap())
        .abs()
            < f64::EPSILON
    );
}

#[test]
fn hb96_raised_every_maximum_alike_and_so_raised_the_category_amounts_unevenly() {
    // LSC: "increases of about 4.8% in effective award amounts for each category".
    let rise = ceilings::jpsn_rise(year(2025), year(2026)).unwrap();
    for (i, r) in rise.iter().enumerate() {
        assert!((0.0479..0.0481).contains(r), "category {}: {r:.5}", i + 1);
    }
    assert!(ceilings::analysis("hb96").contains("increases of about 4.8% in effective award"));
    assert!((rise[0] - 0.047_992).abs() < 0.000_001, "{:.6}", rise[0]);

    // With the base held flat, a uniform rise in base-plus-amount is a rise in the amount alone,
    // and it is steepest where the amount is smallest.
    let before = ceilings::jpsn_supplements(year(2025)).unwrap();
    let after = ceilings::jpsn_supplements(year(2026)).unwrap();
    assert_eq!(before[5], None, "category six was capped in FY2025");
    assert_eq!(after[5], None, "and is capped under H.B. 96");
    let amount_rise: Vec<f64> = (0..5)
        .map(|i| after[i].unwrap() / before[i].unwrap() - 1.0)
        .collect();
    assert!(
        amount_rise.windows(2).all(|w| w[0] > w[1]),
        "{amount_rise:?}"
    );
    assert!(
        (0.1920..0.1921).contains(&amount_rise[0]),
        "{:.5}",
        amount_rise[0]
    );
    assert!(
        (0.0643..0.0644).contains(&amount_rise[4]),
        "{:.5}",
        amount_rise[4]
    );

    // The H.B. 96 amounts are the statute's.
    for (i, amount) in after.iter().take(5).enumerate() {
        assert!((amount.unwrap() - scholarship::JON_PETERSON_SUPPLEMENTS[i]).abs() < 0.5);
    }
}

#[test]
fn the_department_prints_the_enacted_schedule_and_its_fy2023_edition_prints_fy2024s() {
    let printed = |y| jpsn::series(year(y), Series::Maximum);
    let enacted = |y| ceilings::jpsn_maxima(year(y)).unwrap().to_vec();

    // The two later editions print their own year's schedule to the dollar.
    assert_eq!(printed(2024), enacted(2024));
    assert_eq!(printed(2025), enacted(2025));

    // The FY2023 edition prints FY2024's, and the FY2023 schedule two analyses state is not it.
    assert_eq!(printed(2023), enacted(2024));
    assert_ne!(printed(2023), enacted(2023));

    // So the "uniform fraction of the statutory vector" was a ratio of two enacted tables. FY2025's
    // fraction is the ratio of the two ceilings, because H.B. 96 raised every position alike.
    let fraction = printed(2025)[0] / scholarship::jon_peterson_award(1);
    assert!(
        ((32_445.0 / 34_000.0) - fraction).abs() < 0.0001,
        "{fraction:.5}"
    );

    // And the year the table was read as standing still is a year it rose by an eighth: H.B. 33
    // raised categories one to five a uniform 12.09% and the ceiling 11.1%.
    let rise = ceilings::jpsn_rise(year(2023), year(2024)).unwrap();
    for r in &rise[..5] {
        assert!((0.1208..0.1210).contains(r), "{r:.5}");
    }
    assert!((rise[5] - 1.0 / 9.0).abs() < 1e-9);
    assert!((rise[0] - 0.120_988).abs() < 0.000_001, "{:.6}", rise[0]);

    // Against the per-student spend the two editions' charts give, 11.6%: the spend tracks the
    // schedule in both years rather than outrunning a flat one in the first.
    let editions = jpsn::editions();
    let per_student: Vec<f64> = editions
        .values()
        .map(|e| {
            e.spending_per_student()
                .expect("every edition charts spending")
        })
        .collect();
    let spend_first = per_student[1] / per_student[0] - 1.0;
    assert!(
        (rise[0] - spend_first).abs() < 0.006,
        "{spend_first:.4} against {:.4}",
        rise[0]
    );
}
