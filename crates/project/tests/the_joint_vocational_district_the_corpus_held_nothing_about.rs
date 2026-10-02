//! Four fields `education-agency/eastland-fairfield-ctc` records as unpopulated, three of which
//! were already committed.
//!
//! The node's blocker reads: *"IRN, member district list, enrollment, and funding series remain
//! unpopulated."* The IRN is in the federal directory, the funding series is six closed years of
//! the Auditor's district finances, and the enrolment is two years of the Census F-33 panel. All
//! three fixtures were committed and read for other purposes; none of them had ever been asked
//! for a joint vocational district, because nothing here knew how to find one.
//!
//! # Why nothing found them
//!
//! The CCD does not type a joint vocational district as a district. It types it `4`, a *regional
//! education service agency* — the same code Ohio's educational service centres carry. A hundred
//! agencies hold that code and only forty-nine are joint vocational districts, so a reader
//! filtering on type gets half a list of something else, and a reader filtering on "district"
//! gets none of them.
//!
//! Two independent routes split the hundred, and they agree exactly: the name (a service centre
//! says so) and the levy (a joint vocational district charges property tax and a service centre
//! does not, so only the forty-nine appear in the Auditor's finances at all).
//!
//! # The fourth field, narrowed and then filled
//!
//! The roster was first narrowed rather than recovered, by an instrument and its calibration: a joint vocational district's members are exactly the
//! districts whose Table SD-1 rows carry its levy, so summing a candidate roster's levies and
//! comparing against the district's own published property tax says whether the roster is
//! complete. Against the whole of the county its name points at, **twelve of twenty-four
//! reconcile and twelve do not** — and the sign of the miss says which way the roster crosses
//! the county line. [`project::joint_vocational`] is where that lives.
//!
//! Eastland-Fairfield's county pairing left a ninth of its levy unexplained. Its career-technical
//! planning district's roster closes the gap: fifteen of the sixteen districts on it carry the
//! levy, the one outside Franklin and Fairfield is Teays Valley in Pickaway County, and the
//! fifteen reconcile inside the band (#533).

use std::collections::{BTreeMap, BTreeSet};

use dispersion::lea_directory;
use dispersion::sd1;
use project::finances;
use project::joint_vocational::{self, QUIET_TAX_YEARS};

/// Eastland-Fairfield Career & Technical Schools.
const EASTLAND_FAIRFIELD: &str = "051003";

/// The directory files joint vocational districts and service centres under one code.
#[test]
fn a_hundred_agencies_carry_one_code_and_half_of_them_are_not_districts() {
    let all = lea_directory::service_agencies();
    assert_eq!(
        all.len(),
        100,
        "agencies typed as regional service agencies"
    );

    let joint_vocational = lea_directory::joint_vocational_districts();
    assert_eq!(
        joint_vocational.len(),
        49,
        "joint vocational school districts"
    );
    assert_eq!(
        all.len() - joint_vocational.len(),
        51,
        "educational service centres"
    );
    assert_eq!(
        joint_vocational.get(EASTLAND_FAIRFIELD).map(String::as_str),
        Some("Eastland-Fairfield Career & Technical Schools"),
        "the node's own agency, by the IRN it records as unpopulated"
    );
}

/// The name test and the levy test select the same forty-nine.
///
/// This is what makes the name test usable. A joint vocational district levies property tax and
/// an educational service centre does not, so an agency's presence in the Auditor's district
/// finances is a fact about it rather than a spelling of it. If Ohio ever names a career centre
/// in a form the name test misses, it will appear here as a fiftieth agency with a levy.
#[test]
fn the_two_routes_to_the_forty_nine_agree_exactly() {
    let by_name: BTreeSet<String> = lea_directory::joint_vocational_districts()
        .into_keys()
        .collect();

    let panel = finances::finances();
    let by_levy: BTreeSet<String> = lea_directory::service_agencies()
        .into_keys()
        .filter(|irn| {
            finances::for_district(&panel, irn).is_some_and(|d| {
                d.years
                    .iter()
                    .any(|y| y.property_tax.is_some_and(|tax| tax > 0.0))
            })
        })
        .collect();

    assert_eq!(by_levy.len(), 49, "service agencies that levy property tax");
    assert_eq!(by_name, by_levy, "the two routes disagree about membership");
}

/// The funding series the node records as unpopulated is six closed fiscal years.
#[test]
fn the_funding_series_is_six_closed_years_of_the_auditors_finances() {
    let panel = finances::finances();
    let district = finances::for_district(&panel, EASTLAND_FAIRFIELD).expect("in the finances");
    assert_eq!(
        district.name,
        "Eastland-Fairfield Career & Technical Schools"
    );
    assert_eq!(district.county, "Franklin");

    let years: Vec<u16> = district.years.iter().map(|y| y.fiscal_year.0).collect();
    assert_eq!(years, vec![2020, 2021, 2022, 2023, 2024, 2025]);

    let first = &district.years[0];
    let last = district.years.last().expect("a last year");
    assert_eq!(first.property_tax, Some(16_036_688.0));
    assert_eq!(last.property_tax, Some(25_891_768.0));
    assert_eq!(first.unrestricted_aid, Some(5_736_601.0));
    assert_eq!(last.unrestricted_aid, Some(12_088_067.0));

    // State aid more than doubled over the six years while the levy rose by three fifths, so the
    // state share of this district's revenue rose. Both halves are needed to say that, and the
    // node held neither.
    let share = |y: &finances::YearRecord| {
        y.unrestricted_aid.expect("aid") / y.total_revenue.expect("revenue")
    };
    assert!(
        share(last) - share(first) > 0.04,
        "state share moved from {:.3} to {:.3}",
        share(first),
        share(last)
    );
}

/// The enrolment the node records as unpopulated is two years of the F-33 panel.
#[test]
fn the_enrolment_is_two_years_of_the_census_panel() {
    let rows: Vec<_> = dispersion::ohio_panel::panel()
        .into_iter()
        .filter(|r| r.irn == EASTLAND_FAIRFIELD)
        .collect();
    let years: Vec<u16> = rows.iter().map(|r| r.fiscal_year).collect();
    assert_eq!(years, vec![2023, 2024], "years the panel covers it for");

    assert!((rows[0].enrollment - 1_160.0).abs() < 0.5);
    assert!((rows[1].enrollment - 1_354.0).abs() < 0.5);
    assert_eq!(rows[1].property_tax, Some(23_093_000.0));
}

/// Every joint vocational row in the panel is marked outside the comparable set.
///
/// The node asks how joint vocational districts are grouped for peer comparison and records it
/// as unresolved. The corpus's own panel answers it and has since it was built: they are not
/// grouped, they are excluded, on all 121 rows.
#[test]
fn every_joint_vocational_row_in_the_panel_is_outside_the_comparable_set() {
    let joint_vocational: BTreeSet<String> = lea_directory::joint_vocational_districts()
        .into_keys()
        .collect();
    let rows: Vec<_> = dispersion::ohio_panel::panel()
        .into_iter()
        .filter(|r| joint_vocational.contains(&r.irn))
        .collect();

    assert_eq!(rows.len(), 121, "joint vocational rows in the panel");
    assert_eq!(
        rows.iter()
            .map(|r| r.irn.clone())
            .collect::<BTreeSet<_>>()
            .len(),
        46,
        "of the 49, how many the panel reaches"
    );
    assert!(
        rows.iter().all(|r| !r.comparable),
        "a joint vocational district is inside the comparable set"
    );
}

/// A per-pupil figure off these rows is the double count the node already warns about.
///
/// The node's description says a statewide per-pupil figure that sums joint vocational and member
/// district amounts is double counting, because a joint vocational district's students are
/// enrolled in their home district too and attend part-time. The panel's own numbers show what
/// that does to a ratio: spending per pupil comes out a third above the comparable median, on an
/// enrolment count that is not full-time equivalent.
#[test]
fn a_per_pupil_figure_off_these_rows_is_the_double_count() {
    let panel = dispersion::ohio_panel::panel();
    let row = panel
        .iter()
        .find(|r| r.irn == EASTLAND_FAIRFIELD && r.fiscal_year == 2024)
        .expect("FY2024");
    let per_pupil = row.current_spending.expect("spending") / row.enrollment;
    assert!(
        (per_pupil - 20_869.0).abs() < 1.0,
        "Eastland-Fairfield FY2024 spending per pupil is {per_pupil}"
    );

    let mut comparable: Vec<f64> = panel
        .iter()
        .filter(|r| r.comparable && r.fiscal_year == 2024)
        .filter(|r| r.enrollment > 0.0)
        .filter_map(|r| Some(r.current_spending? / r.enrollment))
        .collect();
    comparable.sort_by(f64::total_cmp);
    let median = comparable[comparable.len() / 2];
    assert!(
        (median - 15_426.0).abs() < 1.0,
        "comparable median is {median}"
    );
    assert!(
        per_pupil / median > 1.3,
        "the ratio the exclusion exists to prevent is {}",
        per_pupil / median
    );
}

/// Twelve rosters reconcile against their county and twelve do not.
///
/// The reconciliation is the instrument and this is its calibration. A candidate roster's levies
/// summed out of Table SD-1 should equal what the joint vocational district booked, so a roster
/// that reconciles is complete and one that does not is not — and the sign says which way. A
/// positive miss means the county holds districts belonging to some other joint vocational
/// district; a negative one means this district's members reach outside the county.
///
/// The county is taken from the district's own name, which is a weak pairing and is why twelve
/// fail. Portage Lakes is the instrument catching itself: the name carries a county word that is
/// not its county, and the reconciliation misses by 88%.
#[test]
fn twelve_rosters_reconcile_against_their_county_and_twelve_do_not() {
    let candidates = joint_vocational::county_named();
    assert_eq!(candidates.len(), 24, "districts naming exactly one county");

    let reconciling: Vec<&str> = candidates
        .iter()
        .filter(|r| r.whole())
        .map(|r| r.county.as_str())
        .collect();
    assert_eq!(
        reconciling,
        vec![
            "ASHTABULA",
            "CLARK",
            "COLUMBIANA",
            "DELAWARE",
            "GREENE",
            "LORAIN",
            "MAHONING",
            "MEDINA",
            "PIKE",
            "TRUMBULL",
            "WASHINGTON",
            "WAYNE",
        ],
        "counties that are their joint vocational district's whole membership"
    );

    let portage = candidates
        .iter()
        .find(|r| r.county == "PORTAGE")
        .expect("Portage Lakes");
    assert_eq!(portage.name, "Portage Lakes");
    assert!(
        portage.misses.iter().all(|m| *m > 0.85),
        "the name test's own false positive misses by {:?}",
        portage.misses
    );
}

/// Eastland-Fairfield's roster reaches past both counties in its name.
///
/// Against Fairfield County alone the reconciliation misses by 71%; adding the seven Franklin
/// County districts that carry a levy at the two-mill floor brings it to within a tenth, which
/// is close enough to say those fourteen are members and too far to say they are all of them.
/// The missing member is Teays Valley; see `the_planning_districts_roster_is_the_whole_levy`.
#[test]
fn the_roster_reaches_past_both_counties_in_the_name() {
    /// The seven Fairfield County districts that carry a joint vocational levy, and the seven
    /// Franklin County districts that carry one at the same two-mill rate. Franklin's other two
    /// levy-payers are at 1.8 mills, which is a different joint vocational district.
    const CANDIDATE: [&str; 14] = [
        "046847", "046854", "046862", "046870", "046888", "046896", "046904", "045070", "046946",
        "046953", "046961", "046979", "046995", "047001",
    ];

    for tax_year in QUIET_TAX_YEARS {
        let booked =
            joint_vocational::gross_property_tax(EASTLAND_FAIRFIELD, tax_year + 1).expect("booked");

        let fairfield: f64 = sd1::county("Fairfield", tax_year)
            .into_iter()
            .map(joint_vocational::levy)
            .sum::<f64>()
            / booked
            - 1.0;
        assert!(
            fairfield < -0.6,
            "TY{tax_year}: Fairfield County alone misses by {fairfield}"
        );

        let charged: f64 = sd1::rows()
            .iter()
            .filter(|row| row.tax_year == tax_year && CANDIDATE.contains(&row.irn.as_str()))
            .map(joint_vocational::levy)
            .sum();
        let miss = charged / booked - 1.0;
        assert!(
            (-0.14..-0.08).contains(&miss),
            "TY{tax_year}: the fourteen miss by {miss}"
        );
    }
}

/// The two Franklin County districts the candidate roster leaves out pay a different rate.
///
/// This is what makes the exclusion a reading rather than a convenience: Hilliard and Dublin
/// carry 1.8 mills where the other seven carry 2.0, and a joint vocational district levies one
/// rate across the whole of its territory.
#[test]
fn the_two_franklin_districts_left_out_are_on_a_different_rate() {
    let mut rates: BTreeMap<&str, f64> = BTreeMap::new();
    for row in sd1::county("Franklin", 2024) {
        let (Some(value), tax) = (row.real_property_value, joint_vocational::levy(row)) else {
            continue;
        };
        if tax > 0.0 && value > 0.0 {
            rates.insert(row.irn.as_str(), tax / value * 1000.0);
        }
    }
    assert_eq!(rates.len(), 9, "Franklin districts carrying a levy");

    let at_two = rates.values().filter(|r| (**r - 2.0).abs() < 5e-4).count();
    let at_one_eight = rates.values().filter(|r| (**r - 1.8).abs() < 5e-4).count();
    assert_eq!(
        (at_two, at_one_eight),
        (7, 2),
        "the rates in Franklin: {rates:?}"
    );
}

/// Eastland's planning district.
const EASTLAND_PLANNING_DISTRICT: &str = "200036";

/// Bexley City, which the planning district lists and the levy does not reach.
const BEXLEY: &str = "043620";

/// Teays Valley Local, the member the county pairing could not see.
const TEAYS_VALLEY: &str = "049098";

/// The districts on the planning district's roster, by IRN.
fn planning_district_roster() -> Vec<String> {
    dispersion::ctpd_membership::roster(EASTLAND_PLANNING_DISTRICT)
        .into_iter()
        .filter_map(|member| member.district_irn)
        .collect()
}

/// The planning district's roster is the whole of the levy.
///
/// The fourteen above leave a ninth unexplained. The report card's planning-district roster names
/// sixteen districts, and against Eastland-Fairfield's own books it lands inside the band in both
/// quiet years. Fifteen of the sixteen carry the levy; the sixteenth is Bexley, which carries none. That is
/// the Auditor's own count: the district's audits describe it as serving sixteen districts,
/// fifteen by statute and one by contract.
#[test]
fn the_planning_districts_roster_is_the_whole_levy() {
    let roster = planning_district_roster();
    assert_eq!(roster.len(), 16, "districts on the roster: {roster:?}");

    let members: Vec<&str> = roster.iter().map(String::as_str).collect();
    let misses = joint_vocational::roster_misses(EASTLAND_FAIRFIELD, &members);
    assert_eq!(misses.len(), QUIET_TAX_YEARS.len());
    assert!(
        misses.iter().all(|m| m.abs() < joint_vocational::TOLERANCE),
        "the planning district's roster misses by {misses:?}"
    );

    let rows = sd1::rows();
    for tax_year in QUIET_TAX_YEARS {
        let levying: BTreeSet<&str> = rows
            .iter()
            .filter(|row| row.tax_year == tax_year && members.contains(&row.irn.as_str()))
            .filter(|row| joint_vocational::levy(row) > 0.0)
            .map(|row| row.irn.as_str())
            .collect();
        assert_eq!(
            levying.len(),
            15,
            "TY{tax_year}: members levying {levying:?}"
        );
        assert!(
            !levying.contains(BEXLEY),
            "TY{tax_year}: Bexley carries the levy"
        );
    }
}

/// Teays Valley is the one statutory member outside Franklin and Fairfield, at the same rate.
///
/// So the node's unfilled field — the members outside the two counties in the name — is one
/// district in Pickaway County, at the two mills the seven Franklin members carry. A joint
/// vocational district levies one rate across its territory, which is what makes the rate a
/// second witness and not a coincidence.
#[test]
fn teays_valley_is_the_one_member_outside_the_two_counties() {
    let roster = planning_district_roster();
    let rows = sd1::rows();
    for tax_year in QUIET_TAX_YEARS {
        let outside: Vec<(&str, &str, f64)> = rows
            .iter()
            .filter(|row| row.tax_year == tax_year && roster.contains(&row.irn))
            .filter(|row| joint_vocational::levy(row) > 0.0)
            .filter(|row| {
                !row.county.eq_ignore_ascii_case("Franklin")
                    && !row.county.eq_ignore_ascii_case("Fairfield")
            })
            .map(|row| {
                let value = row.real_property_value.unwrap_or(f64::NAN);
                (
                    row.irn.as_str(),
                    row.county.as_str(),
                    joint_vocational::levy(row) / value * 1000.0,
                )
            })
            .collect();
        let [(irn, county, mills)] = outside.as_slice() else {
            panic!("TY{tax_year}: members outside the two counties: {outside:?}");
        };
        assert_eq!((*irn, *county), (TEAYS_VALLEY, "Pickaway"));
        assert!(
            (mills - 2.0).abs() < 5e-4,
            "TY{tax_year}: Teays Valley at {mills} mills"
        );
    }
}
