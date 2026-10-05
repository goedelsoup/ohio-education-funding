//! The scholarship donation credit, R.C. 5747.73, as `program/scholarship-donation-credit`
//! states it: what taxpayers claimed, what Taxation estimated, and how many organizations could
//! receive a gift.

use project::donation_credit::{
    average_claim, certified_on, estimate, estimates, ever_certified, total, RETRIEVED,
};

fn claimed(year: u16) -> f64 {
    total(year).and_then(|t| t.claimed).expect("a total")
}

fn returns(year: u16) -> u32 {
    total(year).and_then(|t| t.returns).expect("a total")
}

#[test]
fn what_taxpayers_claimed() {
    assert!((claimed(2021) - 268_616.0).abs() < 0.005);
    assert!((claimed(2022) - 11_924_685.0).abs() < 0.005);
    assert!((claimed(2023) - 26_808_131.10).abs() < 0.005);
    assert!((claimed(2024) - 35_509_908.41).abs() < 0.005);
    assert_eq!(
        [2021, 2022, 2023, 2024].map(returns),
        [523, 10_927, 23_860, 30_584]
    );
}

#[test]
fn the_tax_year_2022_average_is_between_the_single_and_joint_caps() {
    // The $750 cap was the only cap in R.C. 5747.73 for tax year 2022 as enacted; H.B. 66 set
    // $1,500 for a joint return in April 2023 with no applicability section, during that year's
    // filing season. An average over $750 cannot come from single-cap returns alone. Taxation's
    // tax year 2022 instructions, printed in December 2022, already told joint filers $1,500, so
    // the average sits under the cap the forms administered.
    let average = average_claim(2022).expect("TY2022");
    assert!(average > 750.0 && average < 1_500.0, "{average}");
    assert!((average - 1_091.30).abs() < 0.01, "{average}");
    assert!(average_claim(2021).expect("TY2021") < 750.0);
}

#[test]
fn every_estimate_is_an_estimate_and_the_editions_disagree() {
    assert_eq!(estimates().len(), 8);
    assert!(estimates().iter().all(|e| e.data_sources == "B+C"));
    for year in 2022..=2025 {
        assert_eq!(estimate("2024-25", year), Some(50.5), "FY{year}");
    }
    assert_eq!(estimate("2026-27", 2024), Some(21.0));
    assert_eq!(estimate("2026-27", 2025), Some(23.1));
    assert_eq!(estimate("2026-27", 2026), Some(24.3));
    assert_eq!(estimate("2026-27", 2027), Some(25.5));
}

#[test]
fn claims_ran_over_the_later_estimate() {
    // Tax year 2024's returns were filed in FY2025; the later edition put FY2025 at $23.1
    // million and the returns claimed $35.5 million. Y-1 is an upper bound, so this says the two
    // are measuring differently, not that one is wrong.
    let fy2025 = estimate("2026-27", 2025).expect("FY2025") * 1e6;
    assert!(claimed(2024) > fy2025);
}

#[test]
fn how_many_organizations_could_receive_a_gift() {
    assert_eq!(certified_on(RETRIEVED), 81);
    assert_eq!(ever_certified(), 89);
    assert_eq!(
        [
            "2022-01-01",
            "2023-01-01",
            "2024-01-01",
            "2025-01-01",
            "2026-01-01"
        ]
        .map(certified_on),
        [7, 33, 62, 63, 72]
    );
}
