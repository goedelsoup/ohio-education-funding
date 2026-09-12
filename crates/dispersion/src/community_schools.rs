//! Ohio's community schools as a population: who opened, who never did, and who lasted.
//!
//! # The succession that has to be resolved first
//!
//! [`crate::lea_directory`] files Ohio's community schools under CCD agency type 7, and says so.
//! The type is also a catch-all in the directory's early editions, and there it names something
//! else entirely: **twenty-four regional data-processing co-operatives** — `SUMMIT CO ED COMP
//! ASSOC.`, `LAKE ERIE ED COMP ASSOC`, `METROPOLITAN EDUC COUNCIL`. They are typed 7 from
//! 1994-95, which is three years before Ohio had a community school law and four before it had a
//! community school.
//!
//! A survival curve built on the bare type would book those as Ohio's founding charter cohort,
//! and their 1994 cohort would read 0% surviving — a clean, false finding about a sector that did
//! not exist yet. This is the shape `project::appropriations` met in ALI 200604 and resolved by
//! title: **an identifier that names more than one thing needs its succession established before
//! a series built on it means anything.**
//!
//! # The partition is not a judgement call
//!
//! [`is_predecessor`] excludes an agency typed 7 in any edition before [`FIRST_POSSIBLE`]. That is
//! a year, and a year is the weakest kind of rule — so what licenses it here is that **no agency
//! straddles**. Zero agencies are typed 7 both before 1997-98 and after 2000-01; the two
//! populations share no identifier at all, and the cut falls in a gap rather than through a group.
//! `the_two_populations_do_not_overlap_on_any_axis` holds that, and three further discriminators
//! that give the identical partition:
//!
//! | discriminator | co-operatives | community schools |
//! |---|--:|--:|
//! | IRN block | all in `08xxxx`/`09xxxx` | none in either |
//! | ever in the meal-programme report | 0 of 24 | 490 of 934 |
//! | ever in the school finance survey | 0 of 24 | 539 of 934 |
//!
//! Four rules, one partition. That is what makes it a fact about the source rather than a choice.
//!
//! # The second trap, which the directory's own notes already carry
//!
//! Ohio's first 47 community schools are filed as **type 1, regular school districts**, in the
//! 1999-2000 edition, and recoded to type 7 the next year. `lea_directory` records the recode.
//! The consequence for this module is that an opening year taken from the first *type-7* edition
//! dates all 47 a year late, so [`School::listed_from`] is the first edition naming the agency at
//! all, whatever type it carries there. All 47 are filed `3 New` in that first edition, so nothing
//! is being inferred from the type.
//!
//! # Two findings the raw directory hides
//!
//! **Nearly three in ten community schools never opened.** 934 agencies are listed; **667 ever
//! reach an operating status** and **267 do not**. Those 267 appear under status 7, *scheduled to
//! be operational within 2 years*, and then stop appearing. A count of community schools that
//! counts rows counts a great many schools that never taught anybody.
//!
//! **The sector did not peak in 2005.** Listed agencies peak at **504 in 2005-06** and the number
//! actually open peaks at **391 in 2013-14**, eight years later. In 2005 the directory lists 504
//! and 318 of them are open: the other 186 are announcements and closures. A sector history read
//! off the row count gets both the height and the date wrong.
//!
//! # What survival here is and is not
//!
//! [`survival`] is the share of schools observed for at least `k` years that were still open in
//! their `k`th. It is not a hazard model and it does not adjust for anything. A school open in the
//! directory's last edition is still open as far as this source goes, so every span is a floor.

use std::collections::{BTreeMap, BTreeSet};

use crate::lea_directory::{self, Agency, LAST_YEAR};

/// The CCD agency type Ohio files community schools under.
pub const CHARTER_TYPE: &str = "7";

/// The first edition a community school could appear in.
///
/// Ohio's community school law is H.B. 215 of 1997 and the first schools opened in 1998-99, so an
/// agency typed 7 in an earlier edition is typed 7 for some other reason. See the module docs for
/// why a year is safe here and what would make it unsafe.
pub const FIRST_POSSIBLE: u16 = 1998;

/// The status codes that mean the agency was running a school that year.
///
/// `1` open, `3` new, `4` added to the universe having been part of another agency, `8` reopened.
/// Every one of the six agencies filed `4` is filed `1` the following year, so it is an operating
/// first year rather than a paper one.
///
/// The codes deliberately left out are `2` closed, `6` inactive and `7` scheduled — and `7` is the
/// one that matters, because it is what the 267 schools that never opened are filed under.
pub const OPERATING: [&str; 4] = ["1", "3", "4", "8"];

/// Whether an agency typed 7 is one of the regional co-operatives rather than a community school.
///
/// True where the directory types it 7 in an edition before [`FIRST_POSSIBLE`].
#[must_use]
pub fn is_predecessor(irn: &str) -> bool {
    lea_directory::panel().iter().any(|agency| {
        agency.irn == irn && agency.agency_type == CHARTER_TYPE && agency.opens < FIRST_POSSIBLE
    })
}

/// The twenty-four agencies the type names before Ohio had community schools.
///
/// Held as a list rather than filtered away silently, so that the day the directory changes shape
/// this module fails rather than quietly reclassifies a co-operative as a school.
#[must_use]
pub fn predecessors() -> Vec<Agency> {
    let mut found: Vec<Agency> = lea_directory::panel()
        .into_iter()
        .filter(|agency| agency.agency_type == CHARTER_TYPE && agency.opens < FIRST_POSSIBLE)
        .collect();
    found.sort_by(|a, b| a.irn.cmp(&b.irn).then(a.opens.cmp(&b.opens)));
    found.dedup_by(|a, b| a.irn == b.irn);
    found
}

/// One community school's whole life, as the directory records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct School {
    /// Ohio's identifier.
    pub irn: String,
    /// The last name the directory gives it, which is not always the first.
    pub name: String,
    /// The first edition naming it at all, whatever type that edition files it under.
    ///
    /// Not the first edition typing it 7 — see the module docs on the 47 schools filed as regular
    /// districts in 1999-2000.
    pub listed_from: u16,
    /// The first edition filing it under an [`OPERATING`] status. `None` if it never opened.
    pub opened: Option<u16>,
    /// The last such edition. `None` if it never opened.
    pub last_open: Option<u16>,
    /// How many editions file it open, which may be fewer than the span if it went inactive.
    pub open_years: usize,
    /// Whether it is open in the directory's last edition, and so right-censored rather than
    /// closed.
    pub still_open: bool,
}

impl School {
    /// First open edition to last, inclusive. `None` if it never opened.
    ///
    /// A floor where [`Self::still_open`], because the directory stops rather than the school.
    #[must_use]
    pub fn span(&self) -> Option<u16> {
        Some(self.last_open? - self.opened? + 1)
    }
}

/// Every community school the directory holds, by identifier.
///
/// Excludes [`predecessors`]. Includes schools that never opened — [`School::opened`] is `None`
/// for those, and they are 267 of the 934.
#[must_use]
pub fn schools() -> Vec<School> {
    let excluded: BTreeSet<String> = predecessors()
        .into_iter()
        .map(|agency| agency.irn)
        .collect();

    let mut editions: BTreeMap<String, Vec<Agency>> = BTreeMap::new();
    let mut charters: BTreeSet<String> = BTreeSet::new();
    for agency in lea_directory::panel() {
        if agency.agency_type == CHARTER_TYPE && !excluded.contains(&agency.irn) {
            charters.insert(agency.irn.clone());
        }
        editions.entry(agency.irn.clone()).or_default().push(agency);
    }

    let mut found: Vec<School> = charters
        .into_iter()
        .filter_map(|irn| {
            let mut rows = editions.remove(&irn)?;
            rows.sort_by_key(|agency| agency.opens);
            let open: Vec<u16> = rows
                .iter()
                .filter(|agency| OPERATING.contains(&agency.status.as_str()))
                .map(|agency| agency.opens)
                .collect();
            Some(School {
                name: rows.last()?.name.clone(),
                listed_from: rows.first()?.opens,
                opened: open.first().copied(),
                last_open: open.last().copied(),
                open_years: open.len(),
                still_open: open.last().copied() == Some(LAST_YEAR),
                irn,
            })
        })
        .collect();
    found.sort_by(|a, b| a.irn.cmp(&b.irn));
    found
}

/// The sector in one edition: how many agencies are listed, and how many are running a school.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sector {
    /// The school year, as the calendar year it opens in.
    pub year: u16,
    /// Agencies the edition names, under any status.
    pub listed: usize,
    /// Those of them filed under an [`OPERATING`] status.
    pub open: usize,
}

/// Every edition of it, oldest first.
///
/// The two columns are the module's second finding: they peak eight years apart.
#[must_use]
pub fn sector_by_year() -> BTreeMap<u16, Sector> {
    let excluded: BTreeSet<String> = predecessors()
        .into_iter()
        .map(|agency| agency.irn)
        .collect();
    let charters: BTreeSet<String> = lea_directory::panel()
        .into_iter()
        .filter(|agency| agency.agency_type == CHARTER_TYPE && !excluded.contains(&agency.irn))
        .map(|agency| agency.irn)
        .collect();

    let mut years: BTreeMap<u16, Sector> = BTreeMap::new();
    for agency in lea_directory::panel() {
        if !charters.contains(&agency.irn) {
            continue;
        }
        let sector = years.entry(agency.opens).or_insert(Sector {
            year: agency.opens,
            listed: 0,
            open: 0,
        });
        sector.listed += 1;
        if OPERATING.contains(&agency.status.as_str()) {
            sector.open += 1;
        }
    }
    years
}

/// How many schools observable for `years` years were still open in their last one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Survival {
    /// The length of life being asked about.
    pub years: usize,
    /// Schools that opened early enough for the directory to have observed them that long.
    pub at_risk: usize,
    /// Those of them whose span reaches it.
    pub surviving: usize,
}

impl Survival {
    /// The share, or `None` where nothing was at risk.
    #[must_use]
    pub fn rate(&self) -> Option<f64> {
        #[expect(
            clippy::cast_precision_loss,
            reason = "counts of schools, well under 2^53"
        )]
        (self.at_risk > 0).then(|| self.surviving as f64 / self.at_risk as f64)
    }
}

/// The share of community schools that reached their `years`th year open.
///
/// Counts only schools that opened, and only those the directory could have observed that long —
/// a school opening in 2020 is not evidence about ten-year survival and is left out rather than
/// counted as a failure. See the module docs for what this is not.
#[must_use]
pub fn survival(years: usize) -> Survival {
    let span = u16::try_from(years).unwrap_or(u16::MAX);
    let mut at_risk = 0;
    let mut surviving = 0;
    for school in schools() {
        let Some(opened) = school.opened else {
            continue;
        };
        if opened + span.saturating_sub(1) > LAST_YEAR {
            continue;
        }
        at_risk += 1;
        if school.span().is_some_and(|reached| reached >= span) {
            surviving += 1;
        }
    }
    Survival {
        years,
        at_risk,
        surviving,
    }
}

/// The schools the directory announced and never recorded as open.
///
/// Filed under status `7`, *scheduled to be operational within 2 years*, and then absent. 267 of
/// 934, which is the module's first finding.
#[must_use]
pub fn never_opened() -> Vec<School> {
    schools()
        .into_iter()
        .filter(|school| school.opened.is_none())
        .collect()
}
