//! Every count the plan multiplies, both published years, against the department's own account
//! of where each one came from.
//!
//! # What a column header is not
//!
//! Four corpus nodes recorded that the career-technical and English learner counts are **frozen
//! at FY2021**, and all four cited the same thing: the `CTE` sheet heads its five count columns
//! `Category 1 Career Tech FTE-FY21` and the `EL` sheet heads its three `Category 1 EL
//! ADM-FY21`. From there the nodes drew the obvious consequence — a district whose programme
//! has grown since 2021 is funded for the programme it had, and for English learners, *"the
//! FY2021 Category 1 learners are not the FY2027 Category 1 learners"*.
//!
//! Nothing in the Revised Code says it. R.C. 3317.02(E) and (F) define both as "the enrollment
//! of students **during the school year**", with none of the explicit look-back the same section
//! writes for base cost enrolled ADM in division (C). No greenbook says it. And the counts
//! themselves say otherwise: English learner Category 1 falls **21.97%** between the two
//! published models and moves in **439 of 611 districts**. A count taken in FY2021 cannot do
//! that.
//!
//! What the department *does* say is in [`vintages`], its own table of what each variable is
//! drawn from, on the `Directions` sheet of each workbook and the `Notes` sheet for the two
//! models before them. Four models, four answers, and **FY2021 is not among them**:
//!
//! | model | categorical FTEs drawn from |
//! | --- | --- |
//! | FY2024 | FY23 |
//! | FY2025 | FY23 |
//! | FY2026 | FY26 (Aug #1) |
//! | FY2027 | FY26 (Nov #2) |
//!
//! # Stable is not frozen, and the workbook shows both
//!
//! The reading was believable because career-technical really does barely move: 589 of 611
//! districts carry an identical Category 1 FTE across the two models and the statewide total
//! moves **+0.29%**. Gifted moves less still. Special education moves in nearly every district.
//!
//! That is a fact about the counts rather than about the calendar. A career-technical FTE is
//! fixed by a course schedule and a gifted identification is cumulative, so neither moves much
//! between the August and November collections of one year; a special education ADM is
//! re-evaluated continuously and an English learner's category turns on a proficiency score and
//! a 180-day clock, so both do.
//!
//! And the workbook supplies the control. `[a] School Building Count - FY25` names a year in its
//! header, the `Directions` table names the same year, and it is **identical for all 611
//! districts in both models**. That is what a held-back column looks like. Nothing else on the
//! sheet does.

use std::collections::BTreeMap;

use edfund_core::Adm;

use crate::panel::categoricals::ENGLISH_LEARNER_WEIGHTS;

/// Every count the plan multiplies, one row per district per published year.
const COUNTS: &str = include_str!("../fixtures/calculator-counts.csv");

/// The department's own table of where each of its inputs came from.
const VINTAGES: &str = include_str!("../fixtures/calculator-vintages.tsv");

/// The header this reader was written against.
const EXPECTED_HEADER: &str = "fiscal_year,irn,district,enrolled_adm,special_education_1,\
                               special_education_2,special_education_3,special_education_4,\
                               special_education_5,special_education_6,english_learner_1,\
                               english_learner_2,english_learner_3,gifted_k_8,gifted_9_12,\
                               career_technical_1,career_technical_2,career_technical_3,\
                               career_technical_4,career_technical_5,school_buildings";

/// The two years the department has published a model for.
pub const YEARS: (u16, u16) = (2026, 2027);

/// One district's counts in one published model.
#[derive(Debug, Clone)]
pub struct Row {
    /// The fiscal year the model computes.
    pub fiscal_year: u16,
    /// Information Retrieval Number, the department's district key.
    pub irn: String,
    /// District name, which is not unique across the 611.
    pub name: String,
    /// `[a]` enrolled ADM.
    pub enrolled_adm: Adm,
    /// `[c1]`-`[c6]`, the six special education categories.
    pub special_education: [Adm; 6],
    /// `[e1]`-`[e3]`, the three English learner categories.
    pub english_learner: [Adm; 3],
    /// `[f1]` and `[f2]`, gifted K-8 and 9-12.
    pub gifted: [Adm; 2],
    /// `[g1]`-`[g5]`, the five career-technical categories.
    pub career_technical: [Adm; 5],
    /// `[a]` school building count — the control, which really is held at FY2025.
    pub school_buildings: f64,
}

/// A count the two models can be compared on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Series {
    /// Enrolled ADM, which drives everything else.
    EnrolledAdm,
    /// One of the six special education categories, 1-indexed.
    SpecialEducation(usize),
    /// One of the three English learner categories, 1-indexed.
    EnglishLearner(usize),
    /// Gifted, 1 for K-8 and 2 for grades 9-12.
    Gifted(usize),
    /// One of the five career-technical categories, 1-indexed.
    CareerTechnical(usize),
    /// The school building count, held at FY2025 in both models.
    SchoolBuildings,
}

impl Series {
    /// The fixture column this series is written in.
    #[must_use]
    pub fn column(self) -> String {
        match self {
            Self::EnrolledAdm => "enrolled_adm".to_string(),
            Self::SpecialEducation(k) => format!("special_education_{k}"),
            Self::EnglishLearner(k) => format!("english_learner_{k}"),
            Self::Gifted(1) => "gifted_k_8".to_string(),
            Self::Gifted(_) => "gifted_9_12".to_string(),
            Self::CareerTechnical(k) => format!("career_technical_{k}"),
            Self::SchoolBuildings => "school_buildings".to_string(),
        }
    }

    /// One district's value of this series, out of a row.
    #[must_use]
    pub fn of(self, row: &Row) -> f64 {
        match self {
            Self::EnrolledAdm => row.enrolled_adm,
            Self::SpecialEducation(k) => row.special_education[k - 1],
            Self::EnglishLearner(k) => row.english_learner[k - 1],
            Self::Gifted(k) => row.gifted[k - 1],
            Self::CareerTechnical(k) => row.career_technical[k - 1],
            Self::SchoolBuildings => row.school_buildings,
        }
    }

    /// Every series in the fixture, in column order.
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut out = vec![Self::EnrolledAdm];
        out.extend((1..=6).map(Self::SpecialEducation));
        out.extend((1..=3).map(Self::EnglishLearner));
        out.extend((1..=2).map(Self::Gifted));
        out.extend((1..=5).map(Self::CareerTechnical));
        out.push(Self::SchoolBuildings);
        out
    }
}

/// How a count behaves between the two published models.
#[derive(Debug, Clone, Copy)]
pub struct Stability {
    /// Districts present in both models.
    pub paired: usize,
    /// Of those, how many carry an identical value.
    pub unchanged: usize,
    /// The statewide total in the earlier model.
    pub before: f64,
    /// The statewide total in the later one.
    pub after: f64,
}

impl Stability {
    /// The share of paired districts whose value did not move at all.
    #[must_use]
    pub fn held(&self) -> f64 {
        if self.paired == 0 {
            return 0.0;
        }
        self.unchanged as f64 / self.paired as f64
    }

    /// Relative change in the statewide total, as a fraction.
    #[must_use]
    pub fn change(&self) -> f64 {
        if self.before == 0.0 {
            return 0.0;
        }
        (self.after - self.before) / self.before
    }
}

/// Every row of the counts fixture, oldest year first and by IRN within a year.
///
/// # Panics
///
/// If the fixture's header is not the one this reader was written against, or a row's width
/// differs from it. Both mean a column moved, and every field below is read by position.
#[must_use]
pub fn frame() -> Vec<Row> {
    let mut lines = COUNTS.lines();
    let header = lines.next().unwrap_or_default();
    assert_eq!(
        header, EXPECTED_HEADER,
        "calculator-counts.csv header changed under a positional reader"
    );
    let width = header.split(',').count();
    let mut out = Vec::new();
    for line in lines {
        let cells: Vec<&str> = line.split(',').collect();
        assert_eq!(
            cells.len(),
            width,
            "calculator-counts.csv row is {} wide against a {width}-wide header",
            cells.len()
        );
        let number = |at: usize| cells[at].trim().parse::<f64>().unwrap_or(0.0);
        let Ok(fiscal_year) = cells[0].parse::<u16>() else {
            continue;
        };
        out.push(Row {
            fiscal_year,
            irn: cells[1].to_string(),
            name: cells[2].to_string(),
            enrolled_adm: number(3),
            special_education: std::array::from_fn(|k| number(4 + k)),
            english_learner: std::array::from_fn(|k| number(10 + k)),
            gifted: std::array::from_fn(|k| number(13 + k)),
            career_technical: std::array::from_fn(|k| number(15 + k)),
            school_buildings: number(20),
        });
    }
    out
}

/// One published year's rows, by IRN.
///
/// IRN rather than name, because 611 districts carry 580-odd distinct names — see
/// [`mod@crate::panel`].
#[must_use]
pub fn year(fiscal_year: u16) -> BTreeMap<String, Row> {
    frame()
        .into_iter()
        .filter(|row| row.fiscal_year == fiscal_year)
        .map(|row| (row.irn.clone(), row))
        .collect()
}

/// How one count behaves across the two published models.
#[must_use]
pub fn stability(series: Series) -> Stability {
    let (before, after) = (year(YEARS.0), year(YEARS.1));
    let mut out = Stability {
        paired: 0,
        unchanged: 0,
        before: 0.0,
        after: 0.0,
    };
    for (irn, earlier) in &before {
        let Some(later) = after.get(irn) else {
            continue;
        };
        let (a, b) = (series.of(earlier), series.of(later));
        out.paired += 1;
        out.before += a;
        out.after += b;
        // Exact, not within a tolerance. The question is whether the department re-collected the
        // column, and a re-collection that happened to land on the same value is not a thing
        // that happens to a six-decimal ADM.
        if (a - b).abs() < 1e-9 {
            out.unchanged += 1;
        }
    }
    out
}

/// Districts whose value of a series is identical across the two models.
#[must_use]
pub fn unchanged(series: Series) -> Vec<String> {
    let (before, after) = (year(YEARS.0), year(YEARS.1));
    let mut out: Vec<String> = before
        .iter()
        .filter(|(irn, earlier)| {
            after
                .get(*irn)
                .is_some_and(|later| (series.of(earlier) - series.of(later)).abs() < 1e-9)
        })
        .map(|(irn, _)| irn.clone())
        .collect();
    out.sort();
    out
}

/// One line of the department's own vintage table.
#[derive(Debug, Clone)]
pub struct Vintage {
    /// The workbook the line was read out of.
    pub workbook: u16,
    /// The model the line describes, which is the workbook's own year on the `Directions` sheet
    /// and FY2024 or FY2025 on the carried-forward `Notes` sheet.
    pub model: u16,
    /// The report the variable belongs to — `Base Cost`, `Detailed SFPR`, `Transportation`.
    pub calculation: String,
    /// The department's name for the variable.
    pub variable: String,
    /// The department's own answer to "data from", verbatim.
    pub data_from: String,
}

/// The department's table of where each of its inputs came from, both workbooks.
///
/// # Panics
///
/// If a line does not have the five tab-separated fields the header names.
#[must_use]
pub fn vintages() -> Vec<Vintage> {
    let mut lines = VINTAGES.lines();
    let header = lines.next().unwrap_or_default();
    assert_eq!(header, "workbook\tmodel\tcalculation\tvariable\tdata_from");
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let cells: Vec<&str> = line.split('\t').collect();
            assert_eq!(cells.len(), 5, "calculator-vintages.tsv row: {line}");
            Vintage {
                workbook: cells[0].parse().unwrap_or_default(),
                model: cells[1].parse().unwrap_or_default(),
                calculation: cells[2].to_string(),
                variable: cells[3].to_string(),
                data_from: cells[4].to_string(),
            }
        })
        .collect()
}

/// What the department says one variable of one model was drawn from.
///
/// `None` if no workbook states it. Where both workbooks carry the same model — the `Notes`
/// sheet is identical in the two — the answers agree, which [`carried_forward`] is the check on.
#[must_use]
pub fn vintage(model: u16, variable: &str) -> Option<String> {
    vintages()
        .into_iter()
        .find(|row| row.model == model && row.variable == variable)
        .map(|row| row.data_from)
}

/// Every model the vintage tables describe, oldest first, with what each drew its categorical
/// FTEs from.
#[must_use]
pub fn categorical_vintages() -> Vec<(u16, String)> {
    let mut out: Vec<(u16, String)> = Vec::new();
    for row in vintages() {
        if row.variable != "Categorical FTEs (b-g)" {
            continue;
        }
        if !out.iter().any(|(model, _)| *model == row.model) {
            out.push((row.model, row.data_from));
        }
    }
    out.sort_by_key(|(model, _)| *model);
    out
}

/// The lines both workbooks carry for the same model, and whether they agree.
///
/// The `Notes` sheet is the same table in both files, describing the FY2024 and FY2025 models —
/// two biennia stale in the later one. That it is carried forward *unchanged* is the point: this
/// workbook copies its documentation along with its sheets, which is the same habit that left
/// `-FY21` on two sets of column headers.
#[must_use]
pub fn carried_forward() -> (usize, usize) {
    let rows = vintages();
    let mut shared = 0;
    let mut agreeing = 0;
    for row in rows.iter().filter(|row| row.workbook == YEARS.0) {
        let Some(other) = rows.iter().find(|it| {
            it.workbook == YEARS.1 && it.model == row.model && it.variable == row.variable
        }) else {
            continue;
        };
        shared += 1;
        if other.data_from == row.data_from {
            agreeing += 1;
        }
    }
    (agreeing, shared)
}

/// What the English learner programme actually pays on: the three counts against their weights.
///
/// R.C. 3317.016's schedule descends — 0.2104, 0.1577, 0.1053 — so a pupil who advances a
/// category costs the district money without leaving the population.
#[must_use]
pub fn english_learner_weighted(row: &Row) -> f64 {
    row.english_learner
        .iter()
        .zip(ENGLISH_LEARNER_WEIGHTS)
        .map(|(count, weight)| count * weight)
        .sum()
}

/// Districts whose English learner headcount did not fall between the two published models and
/// whose weighted count fell anyway, against the districts that had any English learner at all.
///
/// This is "composition rather than size" made checkable. The node it belongs to reasoned its
/// way there from a freeze that is not real; the conclusion survives the correction, for the
/// opposite reason. Pupils advancing down the taper is exactly what makes it true.
#[must_use]
pub fn composition_losers() -> (usize, usize) {
    let (before, after) = (year(YEARS.0), year(YEARS.1));
    let mut any = 0;
    let mut losers = 0;
    for (irn, earlier) in &before {
        let Some(later) = after.get(irn) else {
            continue;
        };
        if earlier.english_learner.iter().sum::<f64>() <= 0.0 {
            continue;
        }
        any += 1;
        let held = later.english_learner.iter().sum::<f64>()
            >= earlier.english_learner.iter().sum::<f64>();
        if held && english_learner_weighted(later) < english_learner_weighted(earlier) {
            losers += 1;
        }
    }
    (losers, any)
}
