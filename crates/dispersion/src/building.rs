//! The 2024-25 Ohio School Report Card, one row per building.
//!
//! # Why this reader did not exist
//!
//! The extract has been committed and written by `connect` since the building file was wired,
//! and nothing read it. Four corpus nodes quote figures off it by filename — `school/anton-grdina`
//! states a Performance Index of 36.8 and 66.7% chronic absenteeism as `[verified]` against
//! `report-card-2425-buildings.csv` — and no test held any of them, because the only path to the
//! file was the path constant `connect` writes it through.
//!
//! It is also the other half of a join the corpus was reasoning about without: [`super::identified`]
//! lists 408 federal identifications keyed on building IRN, and every one of them is a row here.
//!
//! # The unit is not the district
//!
//! 3,318 buildings across 941 operating agencies, which is more agencies than there are school
//! districts — community schools, STEM schools and dropout recovery schools each operate as their
//! own LEA, and a building's `district_irn` is its operator rather than the district whose
//! territory it sits in. Summing anything here by `district_name` and comparing it against a
//! district panel compares two different populations.
//!
//! # What the file does not carry
//!
//! **The overall rating.** Ohio's report card publishes one, R.C. 3302.03 defines it, and both
//! federal identification and the academic distress trigger key on it — and neither published
//! building file has the column. What is here is the *achievement* component: a star rating and
//! the Performance Index it is computed from. A reader ranking buildings on
//! [`Building::performance_index`] is ranking them on one of the six components division (D)(3)
//! names, and `accountability-regime/ohio-report-card` records that even the six is unsettled.
//!
//! **A Title I service flag.** CSI selects among Title I served schools, so absence from the
//! identification lists cannot be read off performance here. [`super::identified`] says this from
//! the other side and it is the same limit.

/// The committed extract.
pub const FIXTURE: &str = include_str!("../fixtures/report-card-2425-buildings.csv");

/// The header this reader was written against, asserted on every read.
pub const EXPECTED_HEADER: &str =
    "building_irn,building_name,district_irn,district_name,county,enrollment,\
chronic_absenteeism,achievement_star_rating,performance_index_2425,performance_index_2324,\
performance_index_2223";

/// What the report card publishes for one building.
#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    /// The building's IRN. The key [`super::identified`] joins on.
    pub irn: String,
    /// The building's published name, commas replaced with semicolons by the extractor.
    pub name: String,
    /// The operating agency's IRN — a district, a community school, or a STEM school.
    pub district_irn: String,
    /// The operating agency's name.
    pub district_name: String,
    /// The county the building reports under.
    pub county: String,
    /// Enrolled pupils, 2024-25.
    pub enrollment: Option<f64>,
    /// The share of pupils chronically absent, as a percentage. Five buildings publish none.
    pub chronic_absenteeism: Option<f64>,
    /// The achievement component's star rating, 1 to 5. Whole stars at building grain.
    ///
    /// Absent for exactly the 167 buildings with no Performance Index, which is what says the
    /// two absences are one absence: a building the department did not rate on achievement.
    pub achievement_stars: Option<f64>,
    /// Performance Index, 2024-25.
    pub performance_index: Option<f64>,
    /// Performance Index, 2023-24.
    pub performance_index_prior: Option<f64>,
    /// Performance Index, 2022-23.
    pub performance_index_earliest: Option<f64>,
}

impl Building {
    /// Whether the department rated the building on achievement at all.
    ///
    /// 167 of 3,318 are unrated in 2024-25 — the file writes `NC`, and
    /// [`edfund_core::conventions::MISSING`] carries it so the cell reads as absent rather than
    /// failing to parse by accident.
    #[must_use]
    pub fn rated(&self) -> bool {
        self.performance_index.is_some()
    }
}

/// Which column holds what.
mod column {
    pub const IRN: usize = 0;
    pub const NAME: usize = 1;
    pub const DISTRICT_IRN: usize = 2;
    pub const DISTRICT_NAME: usize = 3;
    pub const COUNTY: usize = 4;
    pub const ENROLLMENT: usize = 5;
    pub const CHRONIC_ABSENTEEISM: usize = 6;
    pub const STARS: usize = 7;
    pub const INDEX_2425: usize = 8;
    pub const INDEX_2324: usize = 9;
    pub const INDEX_2223: usize = 10;
}

/// Every building the 2024-25 report card rates.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or if a row's width differs
/// from the header's — both by way of [`edfund_core::csv::rows`].
#[must_use]
pub fn buildings() -> Vec<Building> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .map(|row| Building {
            irn: row.str(column::IRN).to_string(),
            name: row.str(column::NAME).to_string(),
            district_irn: row.str(column::DISTRICT_IRN).to_string(),
            district_name: row.str(column::DISTRICT_NAME).to_string(),
            county: row.str(column::COUNTY).to_string(),
            enrollment: row.num(column::ENROLLMENT),
            chronic_absenteeism: row.num(column::CHRONIC_ABSENTEEISM),
            achievement_stars: row.num(column::STARS),
            performance_index: row.num(column::INDEX_2425),
            performance_index_prior: row.num(column::INDEX_2324),
            performance_index_earliest: row.num(column::INDEX_2223),
        })
        .collect()
}

/// One building by IRN, or `None` where the report card does not rate it.
#[must_use]
pub fn by_irn(irn: &str) -> Option<Building> {
    buildings().into_iter().find(|b| b.irn == irn)
}

/// Every rated building, worst Performance Index first.
///
/// The ordering the identification question is asked in. Unrated buildings are dropped rather
/// than sorted to one end, because "no index" is not a low index — 167 buildings would otherwise
/// arrive at the bottom of a list about the bottom.
#[must_use]
pub fn by_index() -> Vec<Building> {
    let mut rated: Vec<Building> = buildings().into_iter().filter(Building::rated).collect();
    rated.sort_by(|a, b| {
        a.performance_index
            .expect("filtered to rated")
            .total_cmp(&b.performance_index.expect("filtered to rated"))
            .then_with(|| a.irn.cmp(&b.irn))
    });
    rated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_building_carries_a_key_and_an_operator() {
        let all = buildings();
        assert_eq!(all.len(), 3318);
        assert_eq!(
            all.iter()
                .map(|b| &b.irn)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            3318,
            "building IRNs are the join key and are not unique"
        );
        for building in &all {
            assert!(
                !building.district_irn.is_empty(),
                "{} has no operator",
                building.name
            );
            assert!(
                building.enrollment.is_some(),
                "{} has no enrollment",
                building.name
            );
        }
    }

    /// More operating agencies than Ohio has school districts.
    #[test]
    fn the_operator_is_not_always_a_school_district() {
        let operators: std::collections::BTreeSet<String> =
            buildings().into_iter().map(|b| b.district_irn).collect();
        assert_eq!(operators.len(), 941);
        assert!(
            operators.len() > 607,
            "{} operators against 607 traditional districts; the excess is the community, STEM \
             and dropout recovery sector and is what stops this file being a district panel",
            operators.len()
        );
    }

    /// The unrated buildings are one population, not two.
    #[test]
    fn a_building_without_an_index_has_no_star_either() {
        let all = buildings();
        let unrated = all.iter().filter(|b| !b.rated()).count();
        assert_eq!(unrated, 167);
        for building in &all {
            assert_eq!(
                building.performance_index.is_some(),
                building.achievement_stars.is_some(),
                "{} carries one of an index and a star and not the other",
                building.name
            );
        }
    }

    /// `NC` is the department's marker and it must read as absent, not as a parse accident.
    #[test]
    fn the_not_calculated_marker_is_a_recorded_convention() {
        assert!(
            FIXTURE.contains(",NC,"),
            "the fixture no longer writes NC and this reader's absence handling is untested"
        );
        assert!(edfund_core::conventions::MISSING.contains(&"NC"));
        assert_eq!(edfund_core::conventions::number("NC"), None);
    }

    /// The building the corpus quotes, pinned. Nothing held these figures.
    #[test]
    fn anton_grdina_is_the_building_the_corpus_states_it_is() {
        let grdina = by_irn("000828").expect("Anton Grdina is on the report card");
        assert_eq!(grdina.district_name, "Cleveland Municipal");
        assert_eq!(grdina.enrollment, Some(314.0));
        assert_eq!(grdina.performance_index, Some(36.8));
        assert_eq!(grdina.performance_index_prior, Some(39.1));
        assert_eq!(grdina.performance_index_earliest, Some(38.9));
        assert_eq!(grdina.chronic_absenteeism, Some(66.7));
        assert_eq!(grdina.achievement_stars, Some(1.0));
    }

    #[test]
    fn the_worst_index_in_the_state_is_at_a_sixth_of_the_scale() {
        let ranked = by_index();
        assert_eq!(ranked.len(), 3151);
        assert_eq!(ranked[0].performance_index, Some(16.4));
        assert_eq!(ranked[0].name, "Lake Erie International High School");
        assert_eq!(ranked.last().expect("rated").performance_index, Some(116.9));
    }
}
