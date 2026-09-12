//! Who is actually in Ohio's federal accountability system.
//!
//! The corpus modelled the regime, the trigger and the intervention before it could name a single
//! identified school. This is the list.
//!
//! # What identification does and does not mean
//!
//! **CSI selects among Title I served schools**, so a building can perform badly and never appear
//! here because it is outside the eligible population. This module holds no Title I service flag,
//! which means an absence cannot be read as "not failing" — only as "not on this list". The
//! distinction matters most for exactly the buildings a reader would look up first.
//!
//! **The lists are a snapshot, not a history.** The department republishes each file in place
//! under a dated filename rather than archiving prior cycles, so a school that exited before this
//! file was written is indistinguishable from one never identified.
//!
//! # The escalation path is legible in one column
//!
//! `year_identified` is not always a year. For 42 of the 231 CSI schools it carries a history —
//! `ATSI 2022 ; CSI 2025` — which is the three-year ATSI-to-CSI escalation written into a cell.
//! Those schools were identified through a *subgroup*, failed to exit within three years, and
//! became whole-school identifications.

use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = include_str!("../fixtures/identified-schools-2026.csv");

const EXPECTED_HEADER: &str = "status,building_irn,building_name,lea_irn,lea_name,school_type,\
subgroups,year_identified,open_closed,cycle,school_year";

/// One identification of one building.
#[derive(Debug, Clone, PartialEq)]
pub struct Identified {
    /// `csi`, `tsi` or `atsi`.
    pub status: String,
    /// The building's IRN.
    pub building_irn: String,
    /// The building's name, commas replaced with semicolons by the extractor.
    pub building_name: String,
    /// The operating agency's IRN.
    pub lea_irn: String,
    /// The operating agency's name.
    pub lea_name: String,
    /// Traditional School, Community School, and so on.
    pub school_type: String,
    /// For TSI and ATSI, the subgroups that triggered it. Empty for CSI.
    pub subgroups: Vec<String>,
    /// The year, or an escalation history. Empty for TSI and ATSI, which publish none.
    pub year_identified: String,
    /// Open or Closed, where the list says.
    pub open_closed: String,
}

impl Identified {
    /// Whether this identification came by escalation from a subgroup rather than directly.
    #[must_use]
    pub fn escalated(&self) -> bool {
        self.year_identified.contains(';')
    }
}

/// Every identification in the three lists.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or if a row's width differs
/// from the header's — both by way of [`edfund_core::csv::rows`], which is what holds the
/// uniform-width invariant these fixtures are written under. A hand-rolled `split(',')` here
/// checked the header and then indexed by position, so a cell that grew a comma shifted every
/// field after it and the row still parsed.
#[must_use]
pub fn identifications() -> Vec<Identified> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .map(|row| Identified {
            status: row.str(0).to_string(),
            building_irn: row.str(1).to_string(),
            building_name: row.str(2).to_string(),
            lea_irn: row.str(3).to_string(),
            lea_name: row.str(4).to_string(),
            school_type: row.str(5).to_string(),
            subgroups: row
                .str(6)
                .split(';')
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect(),
            year_identified: row.str(7).to_string(),
            open_closed: row.str(8).to_string(),
        })
        .collect()
}

/// Every identification of one building, by IRN. Empty where the building is on no list.
#[must_use]
pub fn status_of(building_irn: &str) -> Vec<Identified> {
    identifications()
        .into_iter()
        .filter(|i| i.building_irn == building_irn)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_lists_are_all_present_and_sized() {
        let all = identifications();
        let count = |s: &str| all.iter().filter(|i| i.status == s).count();
        assert_eq!(count("csi"), 231);
        assert_eq!(count("atsi"), 117);
        assert_eq!(count("tsi"), 60);
    }

    /// The building Ohio's constitutional litigation is named for is on the CSI list.
    ///
    /// Pinned because it is the corpus's sharpest single fact and because it arrived by the
    /// escalation route rather than by performing in the bottom 5% — which is a different claim
    /// and would be easy to lose.
    #[test]
    fn sheridan_high_school_is_csi_by_escalation() {
        let status = status_of("034363");
        assert_eq!(status.len(), 1, "Sheridan appears {} times", status.len());
        assert_eq!(status[0].status, "csi");
        assert_eq!(status[0].lea_name, "Northern Local");
        assert!(
            status[0].escalated(),
            "Sheridan's identification is {:?}, not an escalation",
            status[0].year_identified
        );
        assert!(status[0].year_identified.contains("ATSI 2022"));
        assert!(status[0].year_identified.contains("CSI 2025"));
    }

    /// The corpus's worst-performing exemplar building is on no list, and that is not a bug.
    ///
    /// Anton Grdina has a Performance Index of 36.8 and two-thirds of its students chronically
    /// absent, and appears in none of the three files. CSI selects among **Title I served**
    /// schools and this fixture carries no Title I flag, so the absence cannot be read. Asserted
    /// so that the day it changes, somebody notices.
    #[test]
    fn a_low_performing_building_can_be_absent_from_every_list() {
        assert!(status_of("000828").is_empty());
    }

    /// Escalation is a real and sizeable route into CSI, not an edge case.
    #[test]
    fn a_sixth_of_csi_schools_arrived_by_failing_to_exit_atsi() {
        let csi: Vec<Identified> = identifications()
            .into_iter()
            .filter(|i| i.status == "csi")
            .collect();
        let escalated = csi.iter().filter(|i| i.escalated()).count();
        assert_eq!(escalated, 42);
        assert!(
            escalated * 4 < csi.len() && escalated * 8 > csi.len(),
            "{escalated} of {} is outside the band the corpus records",
            csi.len()
        );
    }

    /// Identification outlasts the three years the exit criteria allow.
    #[test]
    fn some_schools_have_been_identified_since_2018() {
        let stale = identifications()
            .into_iter()
            .filter(|i| i.status == "csi" && i.year_identified.contains("2018"))
            .count();
        assert!(
            stale > 0,
            "no CSI school still carries a 2018 identification; the exit story has changed"
        );
    }

    /// TSI and ATSI identify through named subgroups; CSI does not.
    #[test]
    fn only_the_subgroup_tiers_carry_subgroups() {
        for row in identifications() {
            if row.status == "csi" {
                assert!(
                    row.subgroups.is_empty(),
                    "{} carries subgroups",
                    row.building_name
                );
            }
        }
        let with_groups = identifications()
            .into_iter()
            .filter(|i| i.status != "csi" && !i.subgroups.is_empty())
            .count();
        assert!(
            with_groups > 100,
            "only {with_groups} subgroup rows are marked"
        );
    }
}

/// Which sector operates a building, for the incidence question.
///
/// The distinction the identification lists do not draw and the report card does not carry: the
/// lists label a building `Community School` or `Dropout Recovery School` without saying that
/// almost every dropout recovery school is a community school, and the report card names only the
/// operating agency. Both are resolved by joining the operating IRN against
/// [`crate::community_schools`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sector {
    /// Operated by a community school, dropout recovery designation or not.
    Community,
    /// Everything else — traditional districts, STEM schools, joint vocational districts.
    Other,
}

/// The label the lists give a school whose pupils have already left school once.
pub const DROPOUT_RECOVERY: &str = "Dropout Recovery School";

/// The subgroup that triggers half the subgroup tier, and no community school in it.
pub const DISABILITY_SUBGROUP: &str = "Students with Disabilities";

/// Every community school's operating IRN, as [`crate::community_schools`] resolves the type.
fn community_operators() -> BTreeSet<String> {
    crate::community_schools::schools()
        .into_iter()
        .map(|school| school.irn)
        .collect()
}

/// How many buildings each sector operates, and how many of them are identified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Incidence {
    /// Buildings the report card carries for this sector.
    pub buildings: usize,
    /// Those of them on the list being asked about.
    pub identified: usize,
}

impl Incidence {
    /// The share identified, or `None` where the sector has no buildings.
    #[must_use]
    pub fn rate(&self) -> Option<f64> {
        #[expect(
            clippy::cast_precision_loss,
            reason = "counts of buildings, far under 2^53"
        )]
        (self.buildings > 0).then(|| self.identified as f64 / self.buildings as f64)
    }
}

/// Buildings by sector, and how many carry an identification matching `wanted`.
///
/// `wanted` is given the identifications attached to a building, so a caller can ask about one
/// list, about the subgroup tiers together, or about a subgroup.
fn incidence_where(sector: Sector, wanted: &dyn Fn(&[Identified]) -> bool) -> Incidence {
    let operators = community_operators();
    let mut lists: BTreeMap<String, Vec<Identified>> = BTreeMap::new();
    for found in identifications() {
        lists
            .entry(found.building_irn.clone())
            .or_default()
            .push(found);
    }

    let mut incidence = Incidence {
        buildings: 0,
        identified: 0,
    };
    for building in crate::building::buildings() {
        let operated = if operators.contains(&building.district_irn) {
            Sector::Community
        } else {
            Sector::Other
        };
        if operated != sector {
            continue;
        }
        incidence.buildings += 1;
        if wanted(lists.get(&building.irn).map_or(&[], Vec::as_slice)) {
            incidence.identified += 1;
        }
    }
    incidence
}

/// Buildings of one sector carrying an identification of one status.
///
/// `status` is `csi`, `tsi` or `atsi`, as the lists spell it.
#[must_use]
pub fn incidence(status: &str, sector: Sector) -> Incidence {
    incidence_where(sector, &|found: &[Identified]| {
        found.iter().any(|one| one.status == status)
    })
}

/// How far a sector is over- or under-represented on one list, against its share of buildings.
///
/// One where the list matches the sector's share of the state's buildings, above one where the
/// sector is over-represented. The three lists do not agree on the sign, which is the finding
/// `the_three_lists_do_not_tilt_the_same_way` holds.
///
/// `None` where the sector has no buildings or the list is empty.
#[must_use]
pub fn tilt(status: &str, sector: Sector) -> Option<f64> {
    let here = incidence(status, sector);
    let there = incidence(
        status,
        match sector {
            Sector::Community => Sector::Other,
            Sector::Other => Sector::Community,
        },
    );
    let listed = here.identified + there.identified;
    let buildings = here.buildings + there.buildings;
    if listed == 0 || buildings == 0 || here.buildings == 0 {
        return None;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "counts of buildings, far under 2^53"
    )]
    let share = |part: usize, whole: usize| part as f64 / whole as f64;
    let of_list = share(here.identified, listed);
    let of_buildings = share(here.buildings, buildings);
    (of_buildings > 0.0).then_some(of_list / of_buildings)
}

/// The subgroup tiers together, optionally without the schools that only a disability subgroup put
/// there.
///
/// `Students with Disabilities` is the single largest door into TSI and ATSI and **no community
/// school enters through it**, so excluding it is what turns the sector comparison from a
/// statement about community schools into a statement about that subgroup. See
/// `the_reverse_tilt_is_one_subgroup`.
#[must_use]
pub fn subgroup_tiers(sector: Sector, without_disability_only: bool) -> Incidence {
    incidence_where(sector, &move |found: &[Identified]| {
        let tiers: Vec<&Identified> = found
            .iter()
            .filter(|one| one.status == "tsi" || one.status == "atsi")
            .collect();
        if tiers.is_empty() {
            return false;
        }
        if !without_disability_only {
            return true;
        }
        tiers
            .iter()
            .any(|one| one.subgroups.as_slice() != [DISABILITY_SUBGROUP.to_string()])
    })
}

/// Buildings already on the CSI list, which the other two lists do not also carry.
///
/// The lists are disjoint — no building appears on two — so a sector heavily represented in CSI has
/// a smaller and better-performing residue left to be identified by subgroup. Comparisons of the
/// subgroup tiers hold this constant by excluding CSI buildings from both denominators.
#[must_use]
pub fn comprehensive_buildings() -> BTreeSet<String> {
    identifications()
        .into_iter()
        .filter(|one| one.status == "csi")
        .map(|one| one.building_irn)
        .collect()
}

/// The subgroup tiers among buildings not already identified for comprehensive support.
///
/// The comparison that survives the exhaustion objection above.
#[must_use]
pub fn subgroup_tiers_outside_comprehensive(
    sector: Sector,
    without_disability_only: bool,
) -> Incidence {
    let comprehensive = comprehensive_buildings();
    let operators = community_operators();
    let mut lists: BTreeMap<String, Vec<Identified>> = BTreeMap::new();
    for found in identifications() {
        lists
            .entry(found.building_irn.clone())
            .or_default()
            .push(found);
    }

    let mut incidence = Incidence {
        buildings: 0,
        identified: 0,
    };
    for building in crate::building::buildings() {
        if comprehensive.contains(&building.irn) {
            continue;
        }
        let operated = if operators.contains(&building.district_irn) {
            Sector::Community
        } else {
            Sector::Other
        };
        if operated != sector {
            continue;
        }
        incidence.buildings += 1;
        let tiers: Vec<&Identified> = lists
            .get(&building.irn)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .filter(|one| one.status == "tsi" || one.status == "atsi")
            .collect();
        let counts = if tiers.is_empty() {
            false
        } else if without_disability_only {
            tiers
                .iter()
                .any(|one| one.subgroups.as_slice() != [DISABILITY_SUBGROUP.to_string()])
        } else {
            true
        };
        if counts {
            incidence.identified += 1;
        }
    }
    incidence
}

/// The comprehensive-support rate for a sector with dropout recovery schools taken off both sides.
///
/// **A floor, not a rate**, and the reason is the denominator. The dropout recovery label exists
/// only on the identification lists: the report card names a building's operator and not its
/// designation, so there is no way to count the dropout recovery schools that were *not*
/// identified. Removing only the identified ones removes the fewest that could be there, which
/// makes the denominator the largest it could be and the resulting rate the lowest the true one
/// can be. A sector with unidentified dropout recovery schools has a higher rate than this, never
/// a lower one.
#[must_use]
pub fn comprehensive_without_dropout_recovery(sector: Sector) -> Incidence {
    let operators = community_operators();
    let of_sector = |irn: &str| {
        let operated = crate::building::by_irn(irn)
            .is_some_and(|building| operators.contains(&building.district_irn));
        (operated && sector == Sector::Community) || (!operated && sector == Sector::Other)
    };

    let recovery: BTreeSet<String> = identifications()
        .into_iter()
        .filter(|one| one.school_type == DROPOUT_RECOVERY)
        .map(|one| one.building_irn)
        .filter(|irn| of_sector(irn))
        .collect();

    let whole = incidence("csi", sector);
    let recovering_and_listed = identifications()
        .into_iter()
        .filter(|one| one.status == "csi" && recovery.contains(&one.building_irn))
        .count();

    Incidence {
        buildings: whole.buildings - recovery.len(),
        identified: whole.identified - recovering_and_listed,
    }
}
