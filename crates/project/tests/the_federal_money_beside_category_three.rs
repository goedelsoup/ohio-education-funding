//! The federal money that reached chartered nonpublic schools, measured beside Category 3 and
//! never added to it.
//!
//! [`project::ledger::nonpublic_support`] is the state's three Category 3 lines. Two federal
//! channels serve the same pupils: Emergency Assistance to Non-Public Schools, a pandemic grant
//! that ran FY2022 to FY2025, and the proportionate share of each district's IDEA Part B grant.
//! These tests pin both sizes and keep EANS's catalog reader from picking up the other two
//! programs that have used its line number.

use edfund_core::FiscalYear;
use project::ledger::nonpublic_federal::{
    against_category_three, eans, eans_adjusted, eans_claims, eans_total, idea_part_b,
    ALI_PROGRAMS, EANS_ALI, EANS_FUND,
};
use project::ledger::nonpublic_support;

#[test]
fn the_eans_line_number_has_been_three_programs() {
    /*
     * ALI 200651 is Child Nutrition Services in fund 5B1, Title IID Technology in 3DM0, then EANS
     * in 3HQ0. A reader keyed on the ALI alone gets ten years of spending from three programs,
     * FY2002 to FY2025, and only the last four are EANS.
     */
    let claims = nonpublic_support::claims(EANS_ALI);
    for (fund, title) in ALI_PROGRAMS {
        assert!(
            claims
                .iter()
                .any(|claim| claim.fund == fund && claim.name.starts_with(title)),
            "fund {fund} prints `{title}` under {EANS_ALI}"
        );
    }
    let funds: std::collections::BTreeSet<&str> =
        claims.iter().map(|claim| claim.fund.as_str()).collect();
    assert_eq!(funds.len(), ALI_PROGRAMS.len(), "no fourth fund: {funds:?}");

    let spent = |years: Vec<nonpublic_support::Year>| -> Vec<u16> {
        years
            .into_iter()
            .filter(|year| year.nominal > 0.0)
            .map(|year| year.fiscal_year)
            .collect()
    };
    assert_eq!(
        spent(nonpublic_support::series(EANS_ALI, FiscalYear(2025))),
        [2002, 2003, 2004, 2010, 2011, 2012, 2022, 2023, 2024, 2025],
        "the ALI alone"
    );
    assert_eq!(
        spent(nonpublic_support::series_in(
            EANS_FUND,
            EANS_ALI,
            FiscalYear(2025)
        )),
        [2022, 2023, 2024, 2025],
        "the ALI in fund 3HQ0"
    );
    assert!(eans_claims().iter().all(|claim| claim.fund == EANS_FUND));
}

#[test]
fn eans_spent_285_million_over_four_years() {
    let years = eans(FiscalYear(2025));
    let nominal: Vec<(u16, f64)> = years
        .iter()
        .map(|year| (year.fiscal_year, year.nominal))
        .collect();
    assert_eq!(
        nominal,
        [
            (2022, 55_331_436.0),
            (2023, 95_051_480.0),
            (2024, 86_446_473.0),
            (2025, 48_578_449.0),
        ]
    );
    assert!(years
        .iter()
        .all(|year| year.edition == 2025 && year.kind == "actual"));
    assert!((eans_total() - 285_407_838.0).abs() < 0.5);
}

#[test]
fn eans_was_never_enacted_and_two_years_were_raised_mid_biennium() {
    /*
     * Every enacted appropriation is zero; the only appropriations are the Controlling Board's
     * adjusted ones, and the catalog states two. FY2023's was $254.8 million against $95.1
     * million spent.
     */
    assert!(eans_claims()
        .iter()
        .filter(|claim| claim.kind == "appropriation")
        .all(|claim| claim.amount == Some(0.0)));
    assert_eq!(
        eans_adjusted(),
        [(2023, 2022, 254_755_326.0), (2025, 2024, 64_585_482.0)]
    );
}

#[test]
fn eans_was_a_fifth_to_two_fifths_of_category_three_and_never_part_of_it() {
    /*
     * Year by year EANS ran from 20% to 41% of Category 3; over the four years together it was
     * 30%. Its four-year total is more than Category 3 spent in FY2025, its largest year.
     */
    let years = against_category_three();
    let ratios: Vec<(u16, String)> = years
        .iter()
        .map(|year| (year.fiscal_year, format!("{:.3}", year.ratio())))
        .collect();
    assert_eq!(
        ratios,
        [
            (2022, "0.243".to_string()),
            (2023, "0.413".to_string()),
            (2024, "0.365".to_string()),
            (2025, "0.200".to_string()),
        ]
    );
    let state: f64 = years.iter().map(|year| year.category_three).sum();
    assert_eq!(format!("{:.3}", eans_total() / state), "0.305");
    let fy2025 = years.last().expect("FY2025").category_three;
    assert!((fy2025 - 242_688_919.0).abs() < 0.5);
    assert!(eans_total() > fy2025);

    // Category 3 is the three state lines and nothing else.
    assert!(!nonpublic_support::CATEGORY_THREE.contains(&EANS_ALI));
}

#[test]
fn the_idea_proportionate_share_is_under_four_percent_of_the_grant() {
    /*
     * About $15 million to $16.5 million a year, owed by 225 to 232 agencies. The editions print
     * no total, so these sums are this repository's.
     */
    let years = idea_part_b();
    let summary: Vec<(u16, usize, usize, String, String)> = years
        .iter()
        .map(|year| {
            (
                year.fiscal_year,
                year.agencies,
                year.sharing_agencies,
                format!("{:.2}", year.proportionate_share),
                format!("{:.4}", year.share()),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (2021, 997, 225, "15174643.47".into(), "0.0368".into()),
            (2022, 995, 226, "15560688.50".into(), "0.0372".into()),
            (2023, 1007, 230, "16267189.29".into(), "0.0381".into()),
            (2024, 1024, 232, "16433446.71".into(), "0.0365".into()),
        ]
    );
}

#[test]
fn the_idea_nonpublic_count_is_a_floor_only_in_fy2021() {
    let counts: Vec<(u16, u32, usize, usize)> = idea_part_b()
        .iter()
        .map(|year| {
            (
                year.fiscal_year,
                year.nonpublic_swd,
                year.nonpublic_suppressed,
                year.blank_shares,
            )
        })
        .collect();
    assert_eq!(
        counts,
        [
            (2021, 9_868, 57, 1),
            (2022, 9_890, 0, 0),
            (2023, 10_322, 0, 0),
            (2024, 10_245, 0, 0),
        ]
    );
}
