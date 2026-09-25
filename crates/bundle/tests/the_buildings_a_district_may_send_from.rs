//! The EdChoice designated list, per district, in the feed.
//!
//! Eligibility and not participation — a designated building is one whose students *may* apply,
//! and nothing published says which district a scholarship was charged against. This is as close
//! as the per-district question gets, and the distance is the point: the four scholarship units
//! in `funding_units` are statewide because they cannot be anything else.

use std::collections::BTreeMap;

fn feed() -> bundle::Bundle {
    bundle::build::build()
}

#[test]
fn every_district_either_has_a_row_on_the_list_or_is_absent_from_it() {
    let feed = feed();
    let carried = feed
        .districts
        .iter()
        .filter(|d| d.designated.is_some())
        .count();
    let absent = feed.districts.len() - carried;

    // The department's file covers 606 of the 609 districts in this panel. Three is a small
    // number and a real one: a district with no row is not a district with no designated
    // building, and the feed says `null` rather than zero so a consumer cannot conflate them.
    assert_eq!(
        carried + absent,
        feed.districts.len(),
        "every district is either on the list or off it"
    );
    assert!(
        absent < 10,
        "the list covers nearly the whole panel; {absent} districts absent is a join defect, \
         not a property of the file"
    );
}

#[test]
fn the_count_in_the_feed_is_the_count_in_the_fixture() {
    let feed = feed();
    // Against `dispersion` rather than against a literal: the figure moves when the department
    // publishes the next edition, and a literal here would fail for the right reason at the
    // wrong moment.
    let fixture: BTreeMap<String, (usize, usize)> = dispersion::designated::by_district();
    let mut compared = 0;
    for district in &feed.districts {
        let Some(carried) = district.designated else {
            assert!(
                !fixture.contains_key(&district.irn),
                "{} is null in the feed and present in the fixture",
                district.irn
            );
            continue;
        };
        let (listed, designated) = fixture[&district.irn];
        assert_eq!(carried.listed, listed, "{} listed", district.irn);
        assert_eq!(
            carried.designated, designated,
            "{} designated",
            district.irn
        );
        compared += 1;
    }
    assert!(compared > 500, "only {compared} districts compared");
}

#[test]
fn most_districts_hold_no_designated_building_and_the_feed_says_so_without_saying_null() {
    let feed = feed();
    let none = feed
        .districts
        .iter()
        .filter(|d| d.designated.is_some_and(|e| e.designated == 0))
        .count();
    let some = feed
        .districts
        .iter()
        .filter(|d| d.designated.is_some_and(|e| e.designated > 0))
        .count();
    let buildings: usize = feed
        .districts
        .iter()
        .filter_map(|d| d.designated)
        .map(|e| e.designated)
        .sum();

    // The shape of the finding: designation is concentrated. A minority of districts hold every
    // designated building in the state, and the majority row reads zero rather than absent.
    assert!(
        none > some * 5,
        "{none} districts with none against {some} with at least one — the concentration is the \
         finding and this assertion is what would catch it inverting"
    );
    assert!(
        buildings > some,
        "{buildings} designated buildings across {some} districts: more buildings than districts"
    );
    assert_eq!(
        none + some,
        feed.districts
            .iter()
            .filter(|d| d.designated.is_some())
            .count(),
        "every carried district is in exactly one of the two groups"
    );
}

#[test]
fn the_list_dates_itself_and_is_ahead_of_the_rest_of_the_feed() {
    let feed = feed();
    let year = feed
        .series_years
        .iter()
        .find(|y| y.series == "designated")
        .expect("the designated list names its year");

    // A school year, because R.C. 3310.03 designates a building for a year of enrolment, and a
    // school year is named for the September it opens in while a fiscal year is named for the
    // June it closes in. The two reckonings cover the same twelve months one number apart, which
    // is exactly the confusion the `series_years` index exists to stop — `2026-27` and `FY2027`
    // are the same year and `2027-28` would not be.
    assert_eq!(year.kind, bundle::YearKind::School);
    assert_eq!(year.label, dispersion::designated::SCHOOL_YEAR);
    let opens: u16 = year.label[..4]
        .parse()
        .expect("a school year opens with its first year");
    assert!(
        opens + 1 >= feed.fiscal_year,
        "the designated list ({}) is behind the formula year (FY{}); the department publishes \
         the list the autumn before the year it governs, so it should never be",
        year.label,
        feed.fiscal_year
    );
}
