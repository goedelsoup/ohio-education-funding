//! Who belongs to each career-technical planning district, and how much of that is outside the panel.
//!
//! Ohio's 92 planning districts are the recipient of the career awareness and exploration payment
//! under R.C. 3317.014(E)(1)(a): the **lead** of each is paid $3 for every pupil in the summed
//! enrolled ADM of that planning district's members. The catalog's CTPD table gives the 92 leads.
//! This gives the members, from the only place they are published — the report card's data API.
//!
//! # Why it is here
//!
//! The same reason as [`super::community_school_funding`] and [`super::jvsd_funding`]: the
//! population is not the 609-district panel. It is that panel **plus** community and STEM
//! schools, which is precisely what `project::drafts` names as the reason this payment reaches no
//! lever — "the sum spans two populations this panel does not carry and needs a membership map no
//! source this corpus has found publishes."
//!
//! The second half of that sentence is now false, and this module is the map. The first half is
//! still true and is what [`outside_the_panel`] measures.
//!
//! # The 92, the 90, and the two
//!
//! **Ninety planning districts answer.** The other two are the correctional institutions,
//! `200600` and `200602`: they return HTTP 204 and appear in no organisation index, because they
//! are rated by nobody. 90 + 2 is the catalog table's 92, which is where the legend that
//! "disagrees with itself" resolves — [`RATED_CTPDS`] is the rated population and not a shortfall.
//!
//! # What the measure is, and what it is not
//!
//! [`Member::career_technical_enrolment`] is the API's `enrlCtpd`, which is **career-technical
//! enrolment**. The statute pays on **enrolled ADM**, which this source does not publish. So the
//! shares below describe who does career-technical education through each planning district, and
//! **not** the base the payment multiplies. Treating one for the other would overstate the
//! precision of anything computed here, which is why no function in this module returns a dollar.
//!
//! On the measure that is published: of 954 members, 607 are districts and 347 are not — 36.4% of
//! the membership — but the non-districts are **4.75%** of career-technical enrolment, and 277 of
//! the 347 carry **zero**. A population large in members and small in pupils is a different fact
//! from either number alone, and a reader given only the first would price it very differently.

use std::collections::BTreeMap;

const FIXTURE: &str = include_str!("../fixtures/ctpd-membership.csv");

const EXPECTED_HEADER: &str =
    "school_year,ctpd_irn,ctpd,member,district_irn,career_technical_enrolment";

/// How many columns a row of the extract has.
const WIDTH: usize = 6;

/// The school year the membership is retrieved for.
pub const SCHOOL_YEAR: u16 = 2025;

/// How many planning districts the report card rates, and therefore how many are held here.
///
/// Ninety. The catalog's table has ninety-two; the two absent are correctional institutions.
pub const RATED_CTPDS: usize = 90;

/// How many districts the report card rates, every one of which belongs to a planning district.
pub const RATED_DISTRICTS: usize = 607;

/// One organisation's membership of one planning district.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// The school year.
    pub school_year: u16,
    /// The planning district's number. Not an IRN in the district sense — the numbers run
    /// `200001`-`200121` with one outlier at `021357`.
    pub ctpd_irn: String,
    /// The planning district's name, as the department writes it.
    pub ctpd: String,
    /// The member's name, as the roster writes it. The roster gives no identifier.
    pub member: String,
    /// The member's district IRN, if it is one of the 607 districts.
    ///
    /// `None` for a community or STEM school. **That is the measurement rather than a gap**: the
    /// payment's base spans both populations and only the first is in the panel every lever is
    /// priced over.
    pub district_irn: Option<String>,
    /// Career-technical enrolment through this planning district.
    ///
    /// **Not enrolled ADM**, which is what the statute pays on and what this source does not
    /// publish. See the module note.
    pub career_technical_enrolment: u32,
}

impl Member {
    /// Whether this member is one of the 607 districts the panel could carry.
    #[must_use]
    pub const fn is_district(&self) -> bool {
        self.district_irn.is_some()
    }
}

/// Every membership row.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or a row is not the full
/// width — either of which means the extract is not the file this reader expects.
#[must_use]
pub fn members() -> Vec<Member> {
    let mut lines = FIXTURE.lines();
    let header = lines.next().unwrap_or_default().trim();
    assert_eq!(
        header, EXPECTED_HEADER,
        "the CTPD membership fixture's columns have moved"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let cells: Vec<&str> = line.split(',').collect();
            assert_eq!(
                cells.len(),
                WIDTH,
                "the CTPD membership fixture is uniform-width; this row is not: {line}"
            );
            let district = cells[4].trim();
            Member {
                school_year: cells[0].trim().parse().unwrap_or(SCHOOL_YEAR),
                ctpd_irn: cells[1].trim().to_string(),
                ctpd: cells[2].trim().to_string(),
                member: cells[3].trim().to_string(),
                district_irn: (!district.is_empty()).then(|| district.to_string()),
                career_technical_enrolment: cells[5].trim().parse().unwrap_or(0),
            }
        })
        .collect()
}

/// The members of one planning district.
#[must_use]
pub fn roster(ctpd_irn: &str) -> Vec<Member> {
    members()
        .into_iter()
        .filter(|m| m.ctpd_irn == ctpd_irn)
        .collect()
}

/// Which planning district each district belongs to, by district IRN.
///
/// Every one of the 607 appears exactly once, which is the department's own statement that every
/// district belongs to a planning district — checked here rather than quoted.
#[must_use]
pub fn planning_district_of() -> BTreeMap<String, String> {
    members()
        .into_iter()
        .filter_map(|m| Some((m.district_irn?, m.ctpd_irn)))
        .collect()
}

/// How much of the payment's population sits outside the 609-district panel.
///
/// The quantity `project::drafts` could name and not measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutsideThePanel {
    /// Members that are districts.
    pub districts: usize,
    /// Members that are not — community schools under R.C. Chapter 3314 and STEM schools under
    /// Chapter 3326.
    pub other: usize,
    /// Career-technical enrolment attributed to the districts.
    pub district_enrolment: u32,
    /// Career-technical enrolment attributed to everyone else.
    pub other_enrolment: u32,
    /// How many of the non-district members carry no career-technical enrolment at all.
    pub other_with_no_enrolment: usize,
}

impl OutsideThePanel {
    /// The non-district share of the membership, by count.
    #[must_use]
    pub fn share_of_members(&self) -> f64 {
        let total = self.districts + self.other;
        if total == 0 {
            return 0.0;
        }
        self.other as f64 / total as f64
    }

    /// The non-district share of career-technical enrolment.
    ///
    /// **Not of enrolled ADM**, which is the base the statute pays on. The two differ by more
    /// than rounding: a member's career-technical enrolment is a subset of its ADM, and the
    /// subset is not a constant fraction across the two populations.
    #[must_use]
    pub fn share_of_career_technical_enrolment(&self) -> f64 {
        let total = self.district_enrolment + self.other_enrolment;
        if total == 0 {
            return 0.0;
        }
        f64::from(self.other_enrolment) / f64::from(total)
    }
}

/// Split the membership by whether the panel could carry it.
#[must_use]
pub fn outside_the_panel() -> OutsideThePanel {
    let mut out = OutsideThePanel {
        districts: 0,
        other: 0,
        district_enrolment: 0,
        other_enrolment: 0,
        other_with_no_enrolment: 0,
    };
    for member in members() {
        if member.is_district() {
            out.districts += 1;
            out.district_enrolment += member.career_technical_enrolment;
        } else {
            out.other += 1;
            out.other_enrolment += member.career_technical_enrolment;
            if member.career_technical_enrolment == 0 {
                out.other_with_no_enrolment += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ninety planning districts, and the membership is the whole of both populations.
    #[test]
    fn the_extract_is_ninety_planning_districts() {
        let all = members();
        assert_eq!(
            all.iter()
                .map(|m| &m.ctpd_irn)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            RATED_CTPDS
        );
        assert_eq!(all.len(), 954, "607 districts and 347 that are not");
    }

    /// Every district belongs to exactly one planning district. The department says so; this is
    /// where it is checked.
    #[test]
    fn every_district_is_in_exactly_one_planning_district() {
        let placed = planning_district_of();
        assert_eq!(placed.len(), RATED_DISTRICTS);
        let rows = members().into_iter().filter(Member::is_district).count();
        assert_eq!(
            rows, RATED_DISTRICTS,
            "a district in two rosters would make these disagree"
        );
    }

    /// The forty-seven whose names collide each landed in a different planning district, which is
    /// what the extractor's per-district placement is for.
    #[test]
    fn same_named_districts_are_placed_apart() {
        let mut by_name: BTreeMap<String, std::collections::BTreeSet<(String, String)>> =
            BTreeMap::new();
        for member in members().into_iter().filter(Member::is_district) {
            by_name
                .entry(member.member)
                .or_default()
                .insert((member.district_irn.expect("a district"), member.ctpd_irn));
        }
        let shared: Vec<_> = by_name.iter().filter(|(_, v)| v.len() > 1).collect();
        assert!(!shared.is_empty(), "nineteen names are shared; none found");
        for (name, entries) in shared {
            let ctpds: std::collections::BTreeSet<&String> =
                entries.iter().map(|(_, c)| c).collect();
            assert_eq!(
                ctpds.len(),
                entries.len(),
                "`{name}` has two districts in one planning district, so a name match could not \
                 have told them apart"
            );
        }
    }

    /// The split the module exists for, and the gap between its two measures.
    #[test]
    fn the_population_outside_the_panel_is_large_in_members_and_small_in_pupils() {
        let split = outside_the_panel();
        assert_eq!(split.districts, RATED_DISTRICTS);
        assert_eq!(split.other, 347);
        assert!(
            (split.share_of_members() - 0.3637).abs() < 0.001,
            "36.4% of members: {}",
            split.share_of_members()
        );
        assert!(
            (split.share_of_career_technical_enrolment() - 0.0475).abs() < 0.001,
            "4.75% of career-technical enrolment: {}",
            split.share_of_career_technical_enrolment()
        );
        assert!(
            split.share_of_members() > 7.0 * split.share_of_career_technical_enrolment(),
            "the two measures differ by more than sevenfold, which is the finding"
        );
        assert_eq!(split.other_with_no_enrolment, 277);
    }

    /// A roster is retrievable by its planning district's number.
    #[test]
    fn a_roster_reads_back_by_ctpd() {
        let six = roster("200097");
        assert_eq!(six.len(), 7);
        assert!(six.iter().all(|m| m.ctpd.contains("Six District")));
        assert_eq!(six.iter().filter(|m| !m.is_district()).count(), 1);
    }
}
