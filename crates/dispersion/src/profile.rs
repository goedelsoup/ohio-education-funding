//! The FY2024 District Profile Report — the "Cupp Report" — one row per district.
//!
//! # Why this module exists
//!
//! This fixture is the corpus's district cross-section: 606 traditional districts with the
//! enrolment, poverty, valuation, millage, spending and revenue columns that most findings here
//! are computed against. It was read by **four separate parsers**, none of them public — the
//! equity suite's, the report-card suite's, the F-33 trough suite's and the `millage` suite's,
//! the last two reaching across crate boundaries with `../../dispersion/fixtures/`. Each carried
//! its own column table and its own convention for an unparseable cell, and nothing anywhere
//! checked that they agreed about what a row is. See issue #157.
//!
//! One reader now, in the crate that owns the file.
//!
//! # Two things this file is easy to be wrong about
//!
//! **Its poverty column is not the report card's.** `econ_disadvantaged_pct_fy24` is a headcount
//! share for FY2024 and runs 0.0-1.0. The 2024-25 report card publishes a share for the same
//! quantity that is **top-coded by community eligibility** and runs 0-100. Against the
//! Performance Index the first gives -0.846 and the second -0.734, so they are not substitutes
//! and swapping one for the other looks like a vintage correction while being a change of
//! variable. [`crate::report_card::ReportCard::economically_disadvantaged`] is the other one.
//!
//! **Its per-pupil denominators are ADM, not headcount and not weighted ADM.** A figure from
//! this file must not be shown beside one from [`crate::report_card`] without saying which
//! denominator each used.
//!
//! # What its personnel block is, and is not
//!
//! Seven of the report's sixty columns are personnel, and the extract has carried them since
//! #408: a teachers' average salary, three experience shares, and three administrator figures.
//! **One of the seven is a count.** `FTE Number of Administrators` sums twelve EMIS position
//! codes — superintendent, deputy superintendent, treasurer, principal, assistant principal,
//! director, supervisor, coordinator, education administration specialist, administrative
//! assistant (the licensed 101 code, not clerical), community school administrator, and other
//! officials. There is no teacher FTE, no counselor, no wellness, fiscal, EMIS or clerical
//! count, and no category finer than "administrator". So the report can be set against the
//! administrator elements of R.C. 3317.011 and against nothing else in that section; a floor on
//! six special teachers meets no column here. Where the category detail would live is EMIS staff
//! reporting, which the corpus has not catalogued.

use std::sync::OnceLock;

/// The committed profile report.
pub const FIXTURE: &str = include_str!("../fixtures/cupp-fy24-district-data.csv");

/// The header this reader was written against.
pub const EXPECTED_HEADER: &str = "irn,district,enrolled_adm_fy24,econ_disadvantaged_pct_fy24,\
assessed_valuation_per_pupil_fy23,current_operating_millage_ty23,\
effective_class1_millage_ty23,operating_expenditure_per_pupil_fy24,\
state_revenue_per_pupil_fy24,local_revenue_per_pupil_fy24,teacher_average_salary_fy24,\
teachers_0_4_years_pct_fy24,teachers_4_10_years_pct_fy24,teachers_10_plus_years_pct_fy24,\
fte_administrators_fy24,administrator_average_salary_fy24,pupil_administrator_ratio_fy24";

/// What the profile report publishes for one district.
///
/// Every figure is `Option` because the department suppresses and omits: one district reports no
/// operating expenditure at all, and a suppressed cell is an absence rather than a zero. The
/// four parsers this replaces disagreed on exactly this point — two returned `None`, one
/// returned `0.0`, and one panicked.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileDistrict {
    /// Information Retrieval Number, the department's district key.
    pub irn: String,
    /// The district's published name, which carries its IRN and county: `Ada Exempted Village
    /// (045187) - Hardin County`.
    pub name: String,
    /// Enrolled ADM, FY2024. The denominator the department's own per-pupil columns use.
    pub enrolled_adm: Option<f64>,
    /// Economically disadvantaged share, FY2024, as a **fraction**. See the module note.
    pub economically_disadvantaged: Option<f64>,
    /// Assessed valuation per pupil, TY2023 — the wealth measure every equity finding here
    /// regresses against.
    pub valuation_per_pupil: Option<f64>,
    /// Total current operating millage, TY2023: the rate voters approved.
    pub current_operating_millage: Option<f64>,
    /// Effective Class I operating millage, TY2023: the rate anyone actually pays, after H.B.
    /// 920 reduction factors and the twenty-mill floor.
    pub effective_class1_millage: Option<f64>,
    /// Operating expenditure per pupil, FY2024.
    pub operating_expenditure_per_pupil: Option<f64>,
    /// State revenue per pupil, FY2024.
    pub state_revenue_per_pupil: Option<f64>,
    /// Local revenue per pupil, FY2024.
    pub local_revenue_per_pupil: Option<f64>,
    /// Classroom teachers' average salary, FY2024.
    pub teacher_average_salary: Option<f64>,
    /// Share of teachers with 0-4 years' experience, FY2024, as a fraction.
    pub teachers_0_4_years: Option<f64>,
    /// Share of teachers with 4-10 years' experience, FY2024, as a fraction.
    pub teachers_4_10_years: Option<f64>,
    /// Share of teachers with more than 10 years' experience, FY2024, as a fraction.
    pub teachers_10_plus_years: Option<f64>,
    /// Full-time-equivalent administrators, FY2024 — the one **count** of staff the report
    /// carries. See the module note on what it sums and what it cannot say.
    pub fte_administrators: Option<f64>,
    /// Administrators' average salary, FY2024.
    pub administrator_average_salary: Option<f64>,
    /// Enrolled ADM per FTE administrator, FY2024. The department's own quotient of two columns
    /// above, and asserted to be that below.
    pub pupil_administrator_ratio: Option<f64>,
}

impl ProfileDistrict {
    /// Whether the district's effective Class I rate sits exactly on the twenty-mill floor.
    ///
    /// Half a hundredth of a mill, because the department publishes the rate to four decimals
    /// and a district held at the floor reads as `20` rather than `20.0000`.
    #[must_use]
    pub fn at_twenty_mill_floor(&self) -> bool {
        self.effective_class1_millage
            .is_some_and(|m| (m - 20.0).abs() < 0.005)
    }

    /// Whether the district's effective Class I rate is **below** twenty mills.
    ///
    /// Twenty of the 606 are, which reads as a violation of the floor and is not. Six never
    /// voted twenty mills, and a floor cannot lift a rate above what voters approved — see
    /// [`ProfileDistrict::never_voted_twenty_mills`], which is the condition `millage`'s own
    /// guard encodes. The other fourteen sit between 19.7 and 20.0 and the corpus has no
    /// account of them.
    #[must_use]
    pub fn below_twenty_mill_floor(&self) -> bool {
        self.effective_class1_millage.is_some_and(|m| m < 19.995)
    }

    /// Whether voters never approved twenty mills of current operating levy.
    #[must_use]
    pub fn never_voted_twenty_mills(&self) -> bool {
        self.current_operating_millage.is_some_and(|m| m < 20.0)
    }
}

/// Every district in the profile report.
///
/// # Panics
///
/// If the fixture's header is not the one this reader was written against, or if a row's width
/// differs from the header's — both by way of [`edfund_core::csv::rows`], which holds the
/// uniform-width invariant these fixtures are written under. The parsers this replaces skipped
/// the header line and indexed by position, so a column inserted upstream would have shifted
/// every field and the file would still have parsed.
#[must_use]
pub fn districts() -> Vec<ProfileDistrict> {
    cached().clone()
}

/// The fixture, parsed once.
///
/// `OnceLock` for the reason `project::panel`'s reader has one: the parse is pure and
/// the file is compiled in, and a lookup helper that re-read it per call turned a scan
/// over districts into a quadratic one.
fn cached() -> &'static Vec<ProfileDistrict> {
    static ROWS: OnceLock<Vec<ProfileDistrict>> = OnceLock::new();
    ROWS.get_or_init(parse)
}

fn parse() -> Vec<ProfileDistrict> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            let irn = row.str(0);
            if irn.is_empty() {
                return None;
            }
            Some(ProfileDistrict {
                irn: irn.to_string(),
                name: row.str(1).to_string(),
                enrolled_adm: row.num(2),
                economically_disadvantaged: row.num(3),
                valuation_per_pupil: row.num(4),
                current_operating_millage: row.num(5),
                effective_class1_millage: row.num(6),
                operating_expenditure_per_pupil: row.num(7),
                state_revenue_per_pupil: row.num(8),
                local_revenue_per_pupil: row.num(9),
                teacher_average_salary: row.num(10),
                teachers_0_4_years: row.num(11),
                teachers_4_10_years: row.num(12),
                teachers_10_plus_years: row.num(13),
                fte_administrators: row.num(14),
                administrator_average_salary: row.num(15),
                pupil_administrator_ratio: row.num(16),
            })
        })
        .collect()
}

/// One column of the report, over the districts that report it.
///
/// The shape every dispersion statistic here takes: `Dispersion::of(&column(&ds, |d|
/// d.operating_expenditure_per_pupil))`. Districts with no value are dropped rather than
/// counted as zero, so the resulting `n` is the reporting population and not the panel.
pub fn column<F>(districts: &[ProfileDistrict], pick: F) -> Vec<f64>
where
    F: Fn(&ProfileDistrict) -> Option<f64>,
{
    districts.iter().filter_map(pick).collect()
}

/// Two columns over the districts reporting **both**, which is what a correlation needs.
pub fn paired<A, B>(districts: &[ProfileDistrict], a: A, b: B) -> (Vec<f64>, Vec<f64>)
where
    A: Fn(&ProfileDistrict) -> Option<f64>,
    B: Fn(&ProfileDistrict) -> Option<f64>,
{
    districts
        .iter()
        .filter_map(|d| Some((a(d)?, b(d)?)))
        .unzip()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_covers_every_traditional_district() {
        assert_eq!(districts().len(), 606);
    }

    /// The key is the department's IRN and it is unique, which every join in this workspace
    /// assumes and nothing asserted.
    #[test]
    fn every_district_is_keyed_once() {
        let ds = districts();
        let keys: std::collections::BTreeSet<&str> = ds.iter().map(|d| d.irn.as_str()).collect();
        assert_eq!(keys.len(), ds.len());
    }

    /// The poverty column is a fraction. The report card's is a percentage, and reading one as
    /// the other is a hundredfold error that no downstream assertion would catch.
    ///
    /// The 607-row report card carries one district this file does not, which is where the
    /// "one district is missing a disadvantage share" in the report-card suite comes from — it
    /// is a fact about the join and not about this column, which is complete.
    #[test]
    fn the_disadvantage_share_is_a_fraction_and_not_a_percentage() {
        let shares = column(&districts(), |d| d.economically_disadvantaged);
        assert_eq!(shares.len(), 606, "every district publishes one");
        assert!(shares.iter().all(|s| (0.0..=1.0).contains(s)));
    }

    /// The one staff count is complete, and the department's ratio column is its quotient with
    /// enrolled ADM — so a reader who wants the ratio can take either and get the same number.
    #[test]
    fn every_district_reports_administrators_and_the_ratio_is_their_quotient() {
        let ds = districts();
        let counted = column(&ds, |d| d.fte_administrators);
        assert_eq!(counted.len(), 606);
        assert!(counted.iter().all(|c| *c > 0.0));
        for d in &ds {
            let (adm, fte, ratio) = (
                d.enrolled_adm.expect("every district has ADM"),
                d.fte_administrators
                    .expect("every district has administrators"),
                d.pupil_administrator_ratio
                    .expect("every district has the ratio"),
            );
            assert!(
                (adm / fte - ratio).abs() < 0.01,
                "{}: {adm} / {fte} is not {ratio}",
                d.name
            );
        }
    }

    /// The personnel block carries exactly one count, which is the whole of what #408 found
    /// when it opened the sheet. If the department adds a teacher FTE this fails, and the
    /// question that issue moved to EMIS staff reporting comes back here.
    #[test]
    fn the_personnel_block_has_one_count_and_it_is_administrators() {
        let counts: Vec<&str> = EXPECTED_HEADER
            .split(',')
            .filter(|column| column.starts_with("fte_"))
            .collect();
        assert_eq!(counts, vec!["fte_administrators_fy24"]);
    }

    /// An absent figure is absent. The parser this replaced read it as `0.0` in one of its four
    /// copies, which put a district at zero operating expenditure.
    #[test]
    fn a_missing_figure_is_none_rather_than_zero() {
        let ds = districts();
        let missing: Vec<&str> = ds
            .iter()
            .filter(|d| d.operating_expenditure_per_pupil.is_none())
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(missing.len(), 1);
        assert!(missing[0].starts_with("Southern Local (046441)"));
    }
}
