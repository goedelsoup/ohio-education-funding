//! The department's similar-district grouping: which of eight kinds of district a district is.
//!
//! Ohio has classified its districts into similar-district groups since 1996, revised in 2007 for
//! the 2000 census and again in 2013 for the 2010 one. The 2013 scheme is the one in force.
//!
//! # Why this is a type and not a number
//!
//! Because the codes run 0 through 8 and **only 1 through 8 are ordered**. The department orders
//! those by a location composite — least urban to most — with student poverty as the secondary
//! term inside each locale, and `0` is not a ninth position on that scale: it is the five
//! districts the methodology removes from the analysis entirely.
//!
//! A bare `u8` invites two errors that look identical to correct code. Treating `0` as "least
//! urban" puts three islands and a border district at the rural end of a ranking they were
//! excluded from. And treating `1..=8` as a poverty band asserts a monotone the scheme does not
//! have: median student poverty runs **46.4% at code 1, 36.7% at 2, 29.0% at 3, and back up to
//! 51.5% at code 4** — above codes 1, 2, 3 and 5 alike. [`Typology::urbanicity`] exists; a
//! `poverty_rank` beside it deliberately does not, and cannot be linked here because there is
//! nothing to link to.
//!
//! # The assignment is current and its measures are not
//!
//! The workbook's own `docProps/core.xml` gives `dcterms:modified` as 6 January 2015, matching the
//! department's "amended January 2015" and meaning the file has been untouched for eleven years.
//! The classification is still the one in force — there is nothing newer — but a district coded
//! *High Student Poverty* was one **in 2013**. So [`District`] carries the four measures the
//! assignment was made on, each named for the year it was measured in, and a caller can set
//! `student_poverty_2013` beside the report card's current share rather than assume the drift away.

use std::collections::BTreeMap;

const FIXTURE: &str = include_str!("../fixtures/district-typology.csv");

const EXPECTED_HEADER: &str =
    "irn,district,county,typology,enrollment_2013,median_income_2013,student_poverty_2013,pct_minority_2013";

/// The fiscal year the assignment was made in, and the census it was made from.
pub const VINTAGE: u16 = 2013;

/// The year the department last amended the file, from the workbook's own properties.
pub const AMENDED: u16 = 2015;

/// One of the department's eight kinds of district, or its refusal to classify one.
///
/// The variants carry the department's own wording. `Excluded` is code `0` and is not a kind of
/// district — see the module note on why this is not a `u8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Typology {
    /// Code 0. *Removed from the analysis*: three islands, a border district, and one more.
    Excluded,
    /// Code 1. Rural — High Student Poverty & Small Student Population.
    RuralHighPoverty,
    /// Code 2. Rural — Average Student Poverty & Very Small Student Population.
    RuralAveragePoverty,
    /// Code 3. Small Town — Low Student Poverty & Small Student Population.
    SmallTownLowPoverty,
    /// Code 4. Small Town — High Student Poverty & Average Student Population Size.
    SmallTownHighPoverty,
    /// Code 5. Suburban — Low Student Poverty & Average Student Population Size.
    SuburbanLowPoverty,
    /// Code 6. Suburban — Very Low Student Poverty & Large Student Population.
    SuburbanVeryLowPoverty,
    /// Code 7. Urban — High Student Poverty & Average Student Population.
    UrbanHighPoverty,
    /// Code 8. Urban — Very High Student Poverty & Very Large Student Population.
    UrbanVeryHighPoverty,
}

/// The four locales the eight codes fall into, which is the axis the department actually orders on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Locale {
    /// Codes 1 and 2. The least urban end of the department's location composite.
    Rural,
    /// Codes 3 and 4.
    SmallTown,
    /// Codes 5 and 6.
    Suburban,
    /// Codes 7 and 8. The most urban end.
    Urban,
}

impl Locale {
    /// The department's own word for it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Rural => "Rural",
            Self::SmallTown => "Small town",
            Self::Suburban => "Suburban",
            Self::Urban => "Urban",
        }
    }
}

impl Typology {
    /// Read a code as the department writes it. `None` for anything outside `0..=8`.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code.trim() {
            "0" => Self::Excluded,
            "1" => Self::RuralHighPoverty,
            "2" => Self::RuralAveragePoverty,
            "3" => Self::SmallTownLowPoverty,
            "4" => Self::SmallTownHighPoverty,
            "5" => Self::SuburbanLowPoverty,
            "6" => Self::SuburbanVeryLowPoverty,
            "7" => Self::UrbanHighPoverty,
            "8" => Self::UrbanVeryHighPoverty,
            _ => return None,
        })
    }

    /// The code, as the department writes it.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Excluded => 0,
            Self::RuralHighPoverty => 1,
            Self::RuralAveragePoverty => 2,
            Self::SmallTownLowPoverty => 3,
            Self::SmallTownHighPoverty => 4,
            Self::SuburbanLowPoverty => 5,
            Self::SuburbanVeryLowPoverty => 6,
            Self::UrbanHighPoverty => 7,
            Self::UrbanVeryHighPoverty => 8,
        }
    }

    /// The department's full category name, verbatim.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Excluded => "Removed from the analysis",
            Self::RuralHighPoverty => "Rural — High Student Poverty & Small Student Population",
            Self::RuralAveragePoverty => {
                "Rural — Average Student Poverty & Very Small Student Population"
            }
            Self::SmallTownLowPoverty => {
                "Small Town — Low Student Poverty & Small Student Population"
            }
            Self::SmallTownHighPoverty => {
                "Small Town — High Student Poverty & Average Student Population Size"
            }
            Self::SuburbanLowPoverty => {
                "Suburban — Low Student Poverty & Average Student Population Size"
            }
            Self::SuburbanVeryLowPoverty => {
                "Suburban — Very Low Student Poverty & Large Student Population"
            }
            Self::UrbanHighPoverty => "Urban — High Student Poverty & Average Student Population",
            Self::UrbanVeryHighPoverty => {
                "Urban — Very High Student Poverty & Very Large Student Population"
            }
        }
    }

    /// A short name, for a chart legend that cannot hold the full one.
    #[must_use]
    pub const fn short(self) -> &'static str {
        match self {
            Self::Excluded => "Not classified",
            Self::RuralHighPoverty => "Rural, high poverty",
            Self::RuralAveragePoverty => "Rural, average poverty",
            Self::SmallTownLowPoverty => "Small town, low poverty",
            Self::SmallTownHighPoverty => "Small town, high poverty",
            Self::SuburbanLowPoverty => "Suburban, low poverty",
            Self::SuburbanVeryLowPoverty => "Suburban, very low poverty",
            Self::UrbanHighPoverty => "Urban, high poverty",
            Self::UrbanVeryHighPoverty => "Urban, very high poverty",
        }
    }

    /// Which of the four locales this code belongs to. `None` for [`Self::Excluded`].
    #[must_use]
    pub const fn locale(self) -> Option<Locale> {
        Some(match self {
            Self::Excluded => return None,
            Self::RuralHighPoverty | Self::RuralAveragePoverty => Locale::Rural,
            Self::SmallTownLowPoverty | Self::SmallTownHighPoverty => Locale::SmallTown,
            Self::SuburbanLowPoverty | Self::SuburbanVeryLowPoverty => Locale::Suburban,
            Self::UrbanHighPoverty | Self::UrbanVeryHighPoverty => Locale::Urban,
        })
    }

    /// The rank on the department's location composite, 1 least urban to 8 most.
    ///
    /// `None` for [`Self::Excluded`], which is the whole reason this is a method and not
    /// [`Self::code`]: code `0` is a refusal to classify, not the bottom of the scale, and a
    /// caller that sorted on the raw code would put three islands at the rural end of a ranking
    /// the department removed them from.
    ///
    /// There is deliberately no `poverty_rank`. Poverty is the secondary term inside each locale,
    /// so the sequence is not monotone in it — see the module note.
    #[must_use]
    pub const fn urbanicity(self) -> Option<u8> {
        match self {
            Self::Excluded => None,
            other => Some(other.code()),
        }
    }

    /// Every code the department assigns, in its own order.
    pub const ALL: [Self; 9] = [
        Self::Excluded,
        Self::RuralHighPoverty,
        Self::RuralAveragePoverty,
        Self::SmallTownLowPoverty,
        Self::SmallTownHighPoverty,
        Self::SuburbanLowPoverty,
        Self::SuburbanVeryLowPoverty,
        Self::UrbanHighPoverty,
        Self::UrbanVeryHighPoverty,
    ];
}

/// One district's assignment, and the measures it was assigned on.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// The district's IRN, which is the only key that is unique — 28 panel names are not.
    pub irn: String,
    /// The district's name as the 2013 roster wrote it, which may not be its name now.
    pub name: String,
    /// The county the roster attributes it to.
    pub county: String,
    /// Its assignment, or the department's refusal to make one.
    pub typology: Typology,
    /// Enrollment as the department weighted it in 2013. `None` where the workbook suppresses it.
    pub enrollment_2013: Option<f64>,
    /// Median household income, 2013. `None` where suppressed.
    pub median_income_2013: Option<f64>,
    /// Student poverty share, 2013. `None` where suppressed.
    pub student_poverty_2013: Option<f64>,
    /// Minority share, 2013. `None` where suppressed.
    pub pct_minority_2013: Option<f64>,
}

/// Every district the department classifies.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or a row carries a code
/// outside `0..=8` — either of which means the extract is not the file this reader expects, and
/// both of which would otherwise produce a plausible classification.
#[must_use]
pub fn districts() -> Vec<District> {
    let mut lines = FIXTURE.lines();
    let header = lines.next().unwrap_or_default().trim();
    assert_eq!(
        header, EXPECTED_HEADER,
        "the typology fixture's columns have moved"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let cells: Vec<&str> = line.split(',').collect();
            assert_eq!(
                cells.len(),
                8,
                "the typology fixture is uniform-width; this row is not: {line}"
            );
            let number = |index: usize| cells[index].trim().parse::<f64>().ok();
            District {
                irn: cells[0].trim().to_string(),
                name: cells[1].trim().to_string(),
                county: cells[2].trim().to_string(),
                typology: Typology::from_code(cells[3]).unwrap_or_else(|| {
                    panic!(
                        "the typology fixture gives {} the code {:?}",
                        cells[0], cells[3]
                    )
                }),
                enrollment_2013: number(4),
                median_income_2013: number(5),
                student_poverty_2013: number(6),
                pct_minority_2013: number(7),
            }
        })
        .collect()
}

/// The assignment, by IRN. District names are not unique; IRNs are.
#[must_use]
pub fn by_irn() -> BTreeMap<String, District> {
    districts()
        .into_iter()
        .map(|district| (district.irn.clone(), district))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_district_the_department_classifies_is_read() {
        let all = districts();
        assert_eq!(all.len(), 614, "the 2013 assignment covers 614 districts");
        assert_eq!(
            by_irn().len(),
            all.len(),
            "and each of them once, keyed on an IRN that is unique where the name is not"
        );
    }

    #[test]
    fn the_excluded_are_the_five_the_methodology_removes() {
        let excluded: Vec<String> = districts()
            .into_iter()
            .filter(|d| d.typology == Typology::Excluded)
            .map(|d| d.name)
            .collect();
        assert_eq!(
            excluded,
            [
                "College Corner Local",
                "Kelleys Island Local",
                "Middle Bass Local",
                "North Bass Local",
                "Put-In-Bay Local",
            ],
            "three islands, a border district, and one more"
        );
    }

    #[test]
    fn a_suppressed_measure_is_absent_rather_than_zero() {
        let all = by_irn();
        // Middle Bass Local: the department suppresses all four of its census measures.
        let bass = &all["048959"];
        assert_eq!(bass.enrollment_2013, None);
        assert_eq!(bass.median_income_2013, None);
        assert_eq!(bass.student_poverty_2013, None);
        assert_eq!(bass.pct_minority_2013, None);
        // And a classified district carries all four.
        let adams = &all["061903"];
        assert!(adams.student_poverty_2013.is_some());
    }

    #[test]
    fn the_excluded_have_no_place_on_the_urbanicity_scale() {
        assert_eq!(Typology::Excluded.urbanicity(), None);
        assert_eq!(Typology::Excluded.locale(), None);
        assert_eq!(Typology::RuralHighPoverty.urbanicity(), Some(1));
        assert_eq!(Typology::UrbanVeryHighPoverty.urbanicity(), Some(8));
    }

    /// The reason there is no `poverty_rank`, asserted against the fixture rather than argued.
    #[test]
    fn the_codes_are_not_ordered_by_poverty() {
        let median = |code: Typology| {
            let mut values: Vec<f64> = districts()
                .into_iter()
                .filter(|d| d.typology == code)
                .filter_map(|d| d.student_poverty_2013)
                .collect();
            values.sort_by(f64::total_cmp);
            values[values.len() / 2]
        };
        let small_town_high = median(Typology::SmallTownHighPoverty);
        // Code 4 sits above codes 1, 2, 3 and 5, so a monotone read of 1..=8 as a poverty band is
        // wrong at four of its eight steps.
        for lower in [
            Typology::RuralHighPoverty,
            Typology::RuralAveragePoverty,
            Typology::SmallTownLowPoverty,
            Typology::SuburbanLowPoverty,
        ] {
            assert!(
                small_town_high > median(lower),
                "code 4's median poverty should exceed code {}'s",
                lower.code()
            );
        }
    }

    #[test]
    fn every_code_has_a_label_and_a_locale_unless_it_is_the_refusal() {
        for code in Typology::ALL {
            assert!(!code.label().is_empty());
            assert!(!code.short().is_empty());
            assert_eq!(
                code.locale().is_none(),
                code == Typology::Excluded,
                "only the refusal has no locale"
            );
        }
    }
}
