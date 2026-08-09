//! Real property taxes charged, TY2023 and TY2024, and the property-tax-share replication.
//!
//! Two jobs, as with the other fixture suites here.
//!
//! The first is replication. Policy Matters Ohio's August 2025 brief on the property tax repeal
//! amendment divided each district's TY2024 real property taxes charged for current expenses by
//! its FY2025 operating expenditure, and published a median, a fiftieth-ranked threshold, three
//! big-city figures and a statewide JVSD total. All of them reproduce here from the department
//! of taxation's own table and the department of education's own spending file. That the
//! replication is exact is what makes the caveats below worth stating: they are corrections to a
//! number this repository can produce, not objections to one it cannot.
//!
//! The second is the part the brief's own methodology names and does not act on. Taxes *charged*
//! for a tax year is not money *received* in a fiscal year, and the gap between the two is
//! largest exactly where a levy has just passed — which is the district the brief singles out.
//!
//! Cited by `catalog/dot-sd1-school-district-taxes.md`.

const SD1: &str = include_str!("../fixtures/sd1-district-taxes.csv");
const FUNCTIONS: &str = include_str!("../fixtures/expenditure-functions-fy25.csv");

/// Column indices in the SD-1 fixture.
mod sd1 {
    pub const IRN: usize = 0;
    pub const TAX_YEAR: usize = 3;
    pub const CLASS1_VALUE: usize = 6;
    pub const CLASS2_VALUE: usize = 11;
    pub const REAL_VALUE: usize = 12;
    pub const CHARGED: usize = 17;
    pub const CHARGED_WITH_JVSD: usize = 18;
}

/// Column indices in the FY2025 expenditure-function fixture.
mod functions {
    pub const IRN: usize = 0;
    pub const ADM: usize = 2;
    pub const OPERATING_PER_PUPIL: usize = 3;
}

const DANBURY: &str = "048934";
const PUT_IN_BAY: &str = "048975";
const MAYFIELD: &str = "044370";
const COLUMBUS: &str = "043802";
const CLEVELAND: &str = "043786";
const CINCINNATI: &str = "043752";

fn rows(csv: &str) -> impl Iterator<Item = Vec<&str>> {
    csv.lines().skip(1).map(|line| line.split(',').collect())
}

fn number(parts: &[&str], column: usize) -> Option<f64> {
    parts.get(column)?.trim().parse::<f64>().ok()
}

/// Taxes charged for one district and tax year, excluding JVSD operating levies.
fn charged(irn: &str, tax_year: &str, column: usize) -> f64 {
    rows(SD1)
        .find(|p| p[sd1::IRN] == irn && p[sd1::TAX_YEAR] == tax_year)
        .and_then(|p| number(&p, column))
        .expect("district present for that tax year")
}

/// FY2025 operating expenditure: per-pupil times ADM, which is the brief's own construction.
fn operating(irn: &str) -> f64 {
    rows(FUNCTIONS)
        .find(|p| p[functions::IRN] == irn)
        .map(|p| {
            number(&p, functions::ADM).expect("adm")
                * number(&p, functions::OPERATING_PER_PUPIL).expect("per pupil")
        })
        .expect("district present in the function fixture")
}

/// The published ratio: TY2024 charge over FY2025 operating expenditure.
fn share(irn: &str) -> f64 {
    charged(irn, "2024", sd1::CHARGED) / operating(irn)
}

fn close(actual: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{what}: got {actual:.4}, expected {expected:.4}"
    );
}

fn total(tax_year: &str, column: usize) -> f64 {
    rows(SD1)
        .filter(|p| p[sd1::TAX_YEAR] == tax_year)
        .filter_map(|p| number(&p, column))
        .sum()
}

/// Four tax years, balanced — every district present in each.
///
/// The window is TY2021 through TY2024 and its width is not arbitrary. Ohio's counties reappraise
/// or update on a staggered three-year cycle, so TY2022–TY2024 holds exactly one valuation event
/// per county and TY2021 makes the earliest of those a change rather than a level. A missing year
/// would leave some county's event unmeasurable — see `regime_diff::recognized_valuation`.
#[test]
fn the_fixture_carries_every_district_in_all_four_tax_years() {
    const YEARS: [&str; 4] = ["2021", "2022", "2023", "2024"];
    assert_eq!(SD1.lines().count() - 1, 611 * YEARS.len());
    for tax_year in YEARS {
        assert_eq!(
            rows(SD1).filter(|p| p[sd1::TAX_YEAR] == tax_year).count(),
            611,
            "tax year {tax_year}"
        );
    }
}

/// Class I plus Class II is total real property value, for every district and year. Checked
/// because the two classes carry different reduction factors and are easy to mix up, and a
/// fixture whose parts do not sum to its whole invites shares that do not add to one.
#[test]
fn the_two_property_classes_partition_real_property_value() {
    for parts in rows(SD1) {
        let (Some(one), Some(two), Some(total)) = (
            number(&parts, sd1::CLASS1_VALUE),
            number(&parts, sd1::CLASS2_VALUE),
            number(&parts, sd1::REAL_VALUE),
        ) else {
            continue;
        };
        assert!(
            (one + two - total).abs() < 1.0,
            "{}: {one} + {two} against {total}",
            parts[sd1::IRN]
        );
    }
}

// ---------------------------------------------------------------------------------------
// Replicating the brief
// ---------------------------------------------------------------------------------------

/// The statewide totals, on both JVSD bases.
///
/// The brief derives its current-expense total from LSC as $13.61bn less $1.44bn of bond and
/// permanent-improvement levies less $513m of JVSD operating levies. The last term is a
/// difference this table computes directly, and it agrees to the million.
#[test]
fn the_statewide_totals_and_the_jvsd_difference_reproduce() {
    let excluding = total("2024", sd1::CHARGED);
    let including = total("2024", sd1::CHARGED_WITH_JVSD);
    close(
        excluding / 1e9,
        11.657,
        0.005,
        "TY2024 charged for current expenses, excluding JVSD",
    );
    close(
        (including - excluding) / 1e6,
        513.0,
        1.0,
        "TY2024 JVSD operating levies",
    );
}

/// Every district-level figure the brief published, to the point it published it at.
#[test]
fn the_published_district_shares_reproduce() {
    close(share(COLUMBUS), 0.48, 0.005, "Columbus");
    close(share(CLEVELAND), 0.39, 0.005, "Cleveland");
    close(share(CINCINNATI), 0.50, 0.005, "Cincinnati");
}

/// The distribution, on the brief's own exclusions.
///
/// Five districts are dropped for want of a Cupp report row — Kelleys Island, Middle Bass, North
/// Bass, Put-in-Bay and College Corner — which the brief states in a footnote. Four of the five
/// are Lake Erie island districts, and they are the same mechanism as the district the brief
/// leads with, so the exclusion removes the corroborating cases rather than outlying ones.
#[test]
fn the_median_and_the_fiftieth_ranked_district_reproduce() {
    let mut shares: Vec<f64> = rows(FUNCTIONS)
        .map(|p| p[functions::IRN].to_string())
        .filter(|irn| irn != PUT_IN_BAY)
        .filter(|irn| rows(SD1).any(|p| p[sd1::IRN] == *irn))
        .map(|irn| share(&irn))
        .collect();
    shares.sort_by(|a, b| b.partial_cmp(a).expect("no NaN"));

    close(shares[49], 0.74, 0.005, "fiftieth-ranked district");
    close(shares[shares.len() / 2], 0.36, 0.005, "median district");
}

/// Danbury Local is first and Put-in-Bay second — and Put-in-Bay is one of the five the brief
/// excludes, which is why the brief's second place is Mayfield City.
///
/// Both leaders are charged more in real property tax than they spend on operations. That is not
/// an error: a district whose valuation per pupil pins it to the minimum state share has to
/// raise essentially all of its own cost, and the residual definition of state aid makes that
/// arithmetic rather than policy.
#[test]
fn the_two_highest_shares_are_the_lake_erie_districts_and_both_exceed_one() {
    assert!(share(DANBURY) > share(PUT_IN_BAY));
    assert!(share(PUT_IN_BAY) > share(MAYFIELD));
    assert!(share(DANBURY) > 1.0, "{}", share(DANBURY));
    assert!(share(PUT_IN_BAY) > 1.0, "{}", share(PUT_IN_BAY));
}

// ---------------------------------------------------------------------------------------
// What a tax year is not
// ---------------------------------------------------------------------------------------

/// **The levy the brief credits for Mayfield's rank is mostly not in the denominator's year.**
///
/// Mayfield passed a levy in November 2024. It is charged in full for TY2024 — the numerator
/// rises 14.7% — but TY2024 is collected across calendar 2025, half early and half in July, so
/// only about half of it reached the district during FY2025, the year whose spending is the
/// denominator. The brief's own methodology states the collection split and then divides anyway,
/// calling it "the closest alignment possible from available data."
///
/// The size of the effect is the point: charged-basis growth against receipts growth, for the
/// one district whose ranking is explained by a levy.
#[test]
fn mayfields_charge_outruns_its_receipts_in_the_levy_year() {
    let growth =
        charged(MAYFIELD, "2024", sd1::CHARGED) / charged(MAYFIELD, "2023", sd1::CHARGED) - 1.0;
    close(growth, 0.147, 0.002, "Mayfield TY2023 to TY2024 charge");

    // Statewide over the same two years, for scale: Mayfield's charge grew about three times
    // as fast as the state's.
    let statewide = total("2024", sd1::CHARGED) / total("2023", sd1::CHARGED) - 1.0;
    close(statewide, 0.0549, 0.002, "statewide charge growth");
    assert!(growth > statewide * 2.5);
}

/// The half of the new levy that the denominator's year never saw, in points of the ratio.
///
/// About **6.6 points** of Mayfield's published share is charge whose second-half settlement
/// arrived in July 2025 — FY2026. Netting it out takes the district from 103% to roughly 96%,
/// which is no longer above every district but Danbury and Put-in-Bay.
///
/// The halving is the department of taxation's own split, not an estimate: first half early in
/// the collection year, second half in July. It assumes the levy is spread evenly across the
/// two settlements, which is how a full-year charge is collected.
#[test]
fn the_timing_gap_is_about_seven_points_of_mayfields_share() {
    let increase =
        charged(MAYFIELD, "2024", sd1::CHARGED) - charged(MAYFIELD, "2023", sd1::CHARGED);
    let uncollected_by_fiscal_year_end = increase / 2.0;
    let points = uncollected_by_fiscal_year_end / operating(MAYFIELD);
    close(
        points,
        0.066,
        0.005,
        "half the levy over operating spending",
    );
    close(
        share(MAYFIELD) - points,
        0.96,
        0.01,
        "Mayfield, timing netted out",
    );
}
