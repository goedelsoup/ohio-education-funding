//! Whether Ohio's community schools are over-represented in federal accountability, and what the
//! answer turns out to be about.
//!
//! Community schools operate **310 of the 3,318 buildings** the report card carries, 9.3%. They
//! hold **45.5% of the comprehensive support list**. Four and a nine-tenths times their share, and
//! it looks like a finding about school quality.
//!
//! It is not, and the thing that says so is inside the same file: **the other two lists tilt the
//! other way.** Community schools are 4.3% of ATSI and 5.0% of TSI against 9.3% of buildings —
//! under-represented by half. A sector that simply performed worse would be over-represented on
//! all three. Two opposite signs in one accountability regime are a fact about the doors, not
//! about what is behind them.
//!
//! # The doors
//!
//! **Into CSI: the graduation rate.** Ohio's plan identifies every high school with a federal
//! graduation rate at or below 67%, with no Title I gate on that criterion. Dropout recovery
//! schools enrol pupils who have already left school once, and **73 of the 76 dropout recovery
//! identifications in the state are community-operated**. The designation is 75 of the 231 CSI
//! schools, 1 of 60 TSI and none of 117 ATSI, which is what a criterion that only high schools can
//! fail looks like from the side.
//!
//! **Into TSI and ATSI: a subgroup.** `Students with Disabilities` alone puts **86 of 177**
//! subgroup-tier buildings on a list, and **not one of them is a community school**. Take that one
//! door out of both sides and the sector comparison inverts: 3.90% against 2.88%.
//!
//! # What survives
//!
//! Removing dropout recovery from both sides of the comprehensive list still leaves community
//! schools at **13.9% against 4.1%** — a floor, for a reason the denominator forces, and still
//! more than three times. So the over-representation is not only the definitional door. It is
//! smaller than the raw figure and it does not disappear.
//!
//! # Three things this cannot establish, and one hypothesis it refutes
//!
//! **Title I eligibility.** CSI's main criterion selects among Title I served schools and this
//! repository holds no service flag — `crate::identified`'s own module note says an absence here
//! cannot be read as "not failing".
//!
//! **Why no community school enters through the disability subgroup.** Whether they enrol fewer
//! such pupils or merely too few to make a reportable subgroup needs building-level counts, and
//! the report card publishes those only at district grain for the 607 traditional districts.
//!
//! **Refuted: that the reverse tilt is about size.** Community school buildings are half the size
//! of the rest — a median of 206 pupils against 411 — and a subgroup needs pupils to be
//! reportable, so smallness was the obvious explanation. It is wrong: the under-representation
//! holds inside every enrolment band, including buildings of 700 and over. See
//! `the_reverse_tilt_is_not_explained_by_size`.

use dispersion::building;
use dispersion::identified::{self, Sector, DISABILITY_SUBGROUP, DROPOUT_RECOVERY};

/// Community schools' share of the buildings the report card carries.
const SECTOR_SHARE: f64 = 310.0 / 3318.0;

fn rate(incidence: identified::Incidence) -> f64 {
    incidence.rate().expect("the sector has buildings")
}

/// **The control.** One regime, three lists, two signs.
///
/// Over-represented by 4.87 on comprehensive support and under-represented by half on both
/// subgroup tiers. No account of community schools as a sector explains both; an account of the
/// criteria does.
#[test]
fn the_three_lists_do_not_tilt_the_same_way() {
    let community = identified::incidence("csi", Sector::Community);
    let other = identified::incidence("csi", Sector::Other);
    assert_eq!((community.identified, community.buildings), (105, 310));
    assert_eq!((other.identified, other.buildings), (126, 3008));

    let share = f64::from(u32::try_from(community.buildings).expect("small")) / 3318.0;
    assert!(
        (share - SECTOR_SHARE).abs() < 1e-12,
        "the denominator moved"
    );

    let tilt =
        |status: &str| identified::tilt(status, Sector::Community).expect("a non-empty list");
    let comprehensive = tilt("csi");
    let additional = tilt("atsi");
    let targeted = tilt("tsi");

    assert!(
        (comprehensive - 4.8651).abs() < 0.001,
        "CSI tilt {comprehensive}"
    );
    assert!(
        (additional - 0.4574).abs() < 0.001,
        "ATSI tilt {additional}"
    );
    assert!((targeted - 0.5352).abs() < 0.001, "TSI tilt {targeted}");

    // The signs, which is the whole argument and is worth asserting as such.
    assert!(
        comprehensive > 4.0,
        "comprehensive support should be far over"
    );
    assert!(
        additional < 1.0 && targeted < 1.0,
        "both subgroup tiers should be under"
    );
}

/// The comprehensive tilt is mostly one designation, and that designation is almost entirely one
/// sector.
///
/// A criterion that identifies a high school for graduating under 67% of its cohort will identify
/// a school whose pupils enrolled having already dropped out. That is close to definitional, and it
/// reaches the community sector because that is where Ohio's dropout recovery schools are.
#[test]
fn the_comprehensive_tilt_is_carried_by_dropout_recovery_schools() {
    let all = identified::identifications();
    let recovery: Vec<_> = all
        .iter()
        .filter(|one| one.school_type == DROPOUT_RECOVERY)
        .collect();
    assert_eq!(recovery.len(), 76);

    let schools = dispersion::community_schools::schools();
    let operators = recovery
        .iter()
        .filter(|one| {
            building::by_irn(&one.building_irn)
                .is_some_and(|b| schools.iter().any(|school| school.irn == b.district_irn))
        })
        .count();
    assert_eq!(
        operators, 73,
        "dropout recovery is a community-sector designation"
    );

    // And it lands on one list and essentially not the others.
    let on = |status: &str| {
        all.iter()
            .filter(|one| one.status == status && one.school_type == DROPOUT_RECOVERY)
            .count()
    };
    assert_eq!((on("csi"), on("tsi"), on("atsi")), (75, 1, 0));
}

/// **It does not explain all of it.** Take dropout recovery off both sides and a gap over three
/// times remains.
///
/// The figure is a floor rather than a rate, and the reason is worth keeping in view: the dropout
/// recovery label exists only on the identification lists. The report card names a building's
/// operator, not its designation, so the schools that carry the designation and were *not*
/// identified cannot be counted and cannot be removed from the denominator. Removing only the
/// identified ones leaves the denominator as large as it can be, so the rate is as low as it can
/// be.
#[test]
fn removing_dropout_recovery_leaves_a_gap_of_more_than_three_times() {
    let community = identified::comprehensive_without_dropout_recovery(Sector::Community);
    let other = identified::comprehensive_without_dropout_recovery(Sector::Other);

    assert_eq!((community.identified, community.buildings), (33, 237));
    assert_eq!((other.identified, other.buildings), (123, 3005));

    let theirs = rate(community);
    let rest = rate(other);
    assert!(
        (theirs - 0.139_241).abs() < 0.0005,
        "community rate {theirs}"
    );
    assert!((rest - 0.040_932).abs() < 0.0005, "other rate {rest}");
    assert!(
        (theirs / rest - 3.402).abs() < 0.01,
        "the remaining ratio is {}",
        theirs / rest
    );

    // Removing schools from a list and from its denominator should shrink the gap, not widen it.
    let raw = rate(identified::incidence("csi", Sector::Community))
        / rate(identified::incidence("csi", Sector::Other));
    assert!(
        raw > theirs / rest,
        "the exclusion should narrow the gap, from {raw}"
    );
}

/// **The reverse tilt is one subgroup.** Excluding it turns under-representation into over.
///
/// `Students with Disabilities` is the single largest door into the subgroup tiers and no community
/// school goes through it. Holding the comprehensive list constant — the three lists are disjoint,
/// so a sector heavily identified there has a smaller residue left — and then removing that one
/// subgroup moves the comparison from 3.90 against 5.86 to 3.90 against 2.88.
///
/// The community figure does not move at all, because there is nothing there to remove. That is
/// the finding stated as arithmetic.
#[test]
fn the_reverse_tilt_is_one_subgroup() {
    let with = |sector| identified::subgroup_tiers_outside_comprehensive(sector, false);
    let without = |sector| identified::subgroup_tiers_outside_comprehensive(sector, true);

    let community_with = with(Sector::Community);
    let community_without = without(Sector::Community);
    assert_eq!(
        community_with, community_without,
        "no community school enters by disability"
    );
    assert_eq!(
        (community_with.identified, community_with.buildings),
        (8, 205)
    );

    let other_with = with(Sector::Other);
    let other_without = without(Sector::Other);
    assert_eq!((other_with.identified, other_with.buildings), (169, 2882));
    assert_eq!(
        (other_without.identified, other_without.buildings),
        (83, 2882)
    );

    // Under before, over after.
    assert!(
        rate(community_with) < rate(other_with),
        "under-represented as published"
    );
    assert!(
        rate(community_without) > rate(other_without),
        "and over-represented once the disability door is closed on both sides"
    );
    let flipped = rate(community_without) / rate(other_without);
    assert!(
        (flipped - 1.355).abs() < 0.01,
        "the inverted ratio is {flipped}"
    );

    // The door's size, which is what makes one subgroup able to do this.
    let tiers: Vec<_> = identified::identifications()
        .into_iter()
        .filter(|one| one.status == "tsi" || one.status == "atsi")
        .collect();
    let only_disability = tiers
        .iter()
        .filter(|one| one.subgroups.as_slice() == [DISABILITY_SUBGROUP.to_string()])
        .count();
    assert_eq!((tiers.len(), only_disability), (177, 86));
}

/// **A refuted hypothesis, kept.** The reverse tilt is not about community schools being small.
///
/// They are small — a median of 206 pupils against 411 — and a subgroup needs enough pupils to be
/// reportable, so size was the obvious explanation and it is wrong. The under-representation holds
/// in every enrolment band, including buildings of 700 and over where the size argument has no
/// force at all.
///
/// Kept as a test rather than deleted, so the explanation is not reached for again.
#[test]
fn the_reverse_tilt_is_not_explained_by_size() {
    let schools = dispersion::community_schools::schools();
    let is_community = |district_irn: &str| schools.iter().any(|s| s.irn == district_irn);
    let tiers: Vec<String> = identified::identifications()
        .into_iter()
        .filter(|one| one.status == "tsi" || one.status == "atsi")
        .map(|one| one.building_irn)
        .collect();

    let mut bands_checked = 0;
    for (low, high) in [
        (0.0, 200.0),
        (200.0, 400.0),
        (400.0, 700.0),
        (700.0, f64::MAX),
    ] {
        let in_band = |b: &building::Building| b.enrollment.is_some_and(|n| n >= low && n < high);
        let mut counts = [(0usize, 0usize); 2];
        for b in building::buildings().iter().filter(|b| in_band(b)) {
            let slot = usize::from(is_community(&b.district_irn));
            counts[slot].0 += 1;
            if tiers.contains(&b.irn) {
                counts[slot].1 += 1;
            }
        }
        let (other_n, other_hit) = counts[0];
        let (community_n, community_hit) = counts[1];
        assert!(community_n > 20, "band {low}-{high} has too few to read");

        #[expect(clippy::cast_precision_loss, reason = "counts of buildings")]
        let share = |hit: usize, n: usize| hit as f64 / n as f64;
        assert!(
            share(community_hit, community_n) < share(other_hit, other_n),
            "community schools are not under-represented in band {low}-{high}: {} vs {}",
            share(community_hit, community_n),
            share(other_hit, other_n)
        );
        bands_checked += 1;
    }
    assert_eq!(bands_checked, 4, "every band should have been read");

    // The size difference itself is real, which is why the hypothesis was worth testing.
    let median = |mut values: Vec<f64>| {
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    };
    let enrolment = |community: bool| {
        median(
            building::buildings()
                .iter()
                .filter(|b| is_community(&b.district_irn) == community)
                .filter_map(|b| b.enrollment)
                .collect(),
        )
    };
    assert!(
        (enrolment(true) - 206.0).abs() < 1.0,
        "community median {}",
        enrolment(true)
    );
    assert!(
        (enrolment(false) - 411.0).abs() < 1.0,
        "other median {}",
        enrolment(false)
    );
}

/// The lists are disjoint, which is what the comparisons above rely on.
#[test]
fn no_building_appears_on_two_of_the_three_lists() {
    let all = identified::identifications();
    let mut seen: Vec<&str> = all.iter().map(|one| one.building_irn.as_str()).collect();
    seen.sort_unstable();
    let total = seen.len();
    seen.dedup();
    assert_eq!(seen.len(), 408, "distinct buildings");
    assert_eq!(
        total, 408,
        "identifications, one per building and no building twice"
    );

    // Asserted anyway rather than inferred from the counts above, because the residue argument
    // needs the tiers to be disjoint and a future file could carry a building on two of them
    // without changing either total.
    let statuses: Vec<Vec<&str>> = seen
        .iter()
        .map(|irn| {
            all.iter()
                .filter(|one| one.building_irn == *irn)
                .map(|one| one.status.as_str())
                .collect()
        })
        .collect();
    for row in &statuses {
        let mut kinds: Vec<&str> = row.clone();
        kinds.sort_unstable();
        kinds.dedup();
        assert_eq!(
            kinds.len(),
            1,
            "a building on two tiers would break the residue argument"
        );
    }
}
