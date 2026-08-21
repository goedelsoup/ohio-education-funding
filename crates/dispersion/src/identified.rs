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
/// If the fixture's header is not the one this was written against.
#[must_use]
pub fn identifications() -> Vec<Identified> {
    let mut lines = FIXTURE.lines();
    assert_eq!(
        lines.next().unwrap_or_default().trim(),
        EXPECTED_HEADER,
        "the identified-schools fixture header changed; update dispersion::identified"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            Some(Identified {
                status: f.first()?.to_string(),
                building_irn: f.get(1)?.to_string(),
                building_name: f.get(2)?.to_string(),
                lea_irn: f.get(3)?.to_string(),
                lea_name: f.get(4)?.to_string(),
                school_type: f.get(5)?.to_string(),
                subgroups: f
                    .get(6)?
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect(),
                year_identified: f.get(7)?.to_string(),
                open_closed: f.get(8)?.to_string(),
            })
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
