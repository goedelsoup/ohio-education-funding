//! Where every committed fixture is written, and the manifest of the ones the rebuild produces.
//!
//! Thirty-five path constants and [`REBUILT`], which lists the subset a full rebuild regenerates.
//!
//! # Why they are together
//!
//! They used to be scattered across 5,500 lines of parsing code — `FY27_FIXTURE` at line 1,711,
//! `CPI_FIXTURE` at 2,415, `F33_FIXTURE` at 3,112, `MR81_FIXTURE` at 5,280 — and `REBUILT`
//! forward-referenced eight constants defined up to 2,900 lines *later*. Nothing about the
//! manifest could be checked by reading it, which is the one thing a manifest is for.
//!
//! A new fixture registers in one place now, and `registry`'s test that every rebuilt path is
//! claimed by a connector has a list it can be read against.

/// Where each fixture is written, relative to the repository root.
pub const FY27_FIXTURE: &str = "crates/foundation/fixtures/fy27-department-model.csv";
/// The FY2026 department model, in the sixteen columns that let it be compared to FY2027.
///
/// Narrow on purpose. [`FY27_FIXTURE`] is 162 columns because every calculator crate reads it;
/// this is the two sides of the state share, the local capacity build-up and the disadvantaged
/// pupil blend, because what it exists for is to say what moved. See
/// [`super::fy26`] for why the catalog's decision not to take these tables at all was wrong.
pub const FY26_FIXTURE: &str = "crates/project/fixtures/fy26-department-model.csv";

/// The FY2025 baseline: what each district was actually paid, from the department's final
/// payment report rather than from a calculator.
///
/// # Why the baseline is a payment report and not a model
///
/// Every other year here comes from a `State Foundation Funding Calculator`. FY2025's is archived
/// only in captures truncated at exactly 1 MiB — 1,033,192 bytes stored against FY2026's
/// 4,187,092 — so the workbook cannot be read from the archive at all. The department's **final**
/// payment report is open, complete, and is in one respect the better source: it is what was
/// paid, after the year closed, rather than what a model projected before it.
///
/// # And its letters are not the other years' letters
///
/// `Summary SFPR` labels its columns `[Aa]`…`[T]`, and the scheme **shifted** when H.B. 96
/// removed supplemental targeted assistance and added two supplements. In FY2025 `[M]` is
/// `Total Formula Funding`; in FY2026 `[M]` is the enrollment growth supplement and `[N]` is the
/// total. A reader keyed on the letter takes the wrong column and reports a plausible number, so
/// [`super::fy25`] reads by the label's **text** and errors on a tag it cannot find.
pub const FY25_FIXTURE: &str = "crates/project/fixtures/fy25-payment-report.csv";

/// The FY2027 community and STEM school model, per school.
///
/// In `dispersion` rather than beside the district models in `foundation` and `project` because
/// it is a different population, not a further year of the same one. Nothing in the calculator
/// crates can join it: a community school has no IRN in the 609-district panel, no valuation and
/// no local share, and the one line it carries that no district does — the equity supplement —
/// is paid to 324 of its 355 schools and to nobody else in Ohio.
pub const COMMUNITY_SCHOOL_FUNDING_FIXTURE: &str =
    "crates/dispersion/fixtures/fy27-community-school-funding.csv";

/// Six years of the department's joint vocational school district foundation payment reports,
/// one row per district per year.
///
/// In `dispersion` beside the community school extract rather than in `foundation` or `project`
/// for the same reason that one is: a second population, not a further year of the 609-district
/// panel. The 49 JVSDs are typed `4` by the federal directory — *regional education service
/// agency*, the code Ohio's educational service centres also carry — and none of them appears in
/// the panel the calculator crates read.
///
/// # Why six years of payment reports and not a calculator
///
/// There is no JVSD calculator. The department publishes a simulator for traditional districts
/// and another for community and STEM schools, and for this population it publishes payment
/// reports instead — what was paid, after the year closed. For FY2022 through FY2026 that is the
/// better artefact, and for FY2027 it is the only one.
///
/// # What the series is for
///
/// Item 7 of H.B. 96's list rewrote the JVSD state share of base cost into per-pupil form. The
/// span is chosen so the amendment has a before and an after inside one fixture: FY2022-FY2025
/// under prior law, FY2026-FY2027 under item 7. See [`super::jvsd`] for what the rewrite turns
/// out to change, which is not the percentage.
pub const JVSD_FUNDING_FIXTURE: &str = "crates/dispersion/fixtures/jvsd-foundation-payments.csv";

/// The statewide scalars each year's funding calculator states once, one row per fiscal year.
///
/// The file a `parameter` node's `series:` should be read off. Several of them said "no
/// prior-year values are held" while two years of the calculator sat in the cache.
pub const CALCULATOR_SCALARS_FIXTURE: &str = "crates/project/fixtures/calculator-parameters.csv";

/// Every count the plan multiplies, both published years, one row per district per year.
///
/// Built to settle a claim four nodes carried on the strength of a column header: that the
/// career-technical and English learner counts are frozen at FY2021. See [`super::counts`].
pub const CALCULATOR_COUNTS_FIXTURE: &str = "crates/project/fixtures/calculator-counts.csv";

/// The department's own table of where each of its inputs came from, both workbooks.
///
/// Tab-separated: a `data_from` reads "Greater of FY26 (Nov #2) or average of (FY26, 25, 24)"
/// and the commas are the department's own.
pub const CALCULATOR_VINTAGES_FIXTURE: &str = "crates/project/fixtures/calculator-vintages.tsv";

/// Enrolled ADM as each workbook publishes it: three labelled prior years plus the model's own
/// `[a]` column, long-form, both workbooks.
///
/// The two windows overlap by two years, which is what makes a fourth observation and a
/// restatement measurable at once. See [`super::counts`].
pub const CALCULATOR_ADM_FIXTURE: &str = "crates/project/fixtures/calculator-adm-series.csv";

/// Where the district-profile fixture is written, relative to the repository root.
pub const PROFILE_FIXTURE: &str = "crates/dispersion/fixtures/cupp-fy24-district-data.csv";
/// Where the grade-band headcount fixture is written, relative to the repository root.
pub const GRADE_BANDS_FIXTURE: &str = "crates/foundation/fixtures/fy24-district-grade-bands.csv";

/// Where the per-district financial panel is written, relative to the repository root.
pub const FINANCE_FIXTURE: &str = "crates/project/fixtures/district-finances.csv";

/// Every way an Ohio child is schooled outside a traditional district, as the department counts
/// them, relative to the repository root.
///
/// Statewide rather than per-agency, which is unusual here and is the point: the sheet prints
/// home education, vouchers, community schools, chartered private schools and open enrolment in
/// one table, so the channels can be read against each other. The department publishes a
/// home-education count nowhere else in machine-reachable form.
pub const LANDSCAPE_FIXTURE: &str = "crates/project/fixtures/education-landscape-channels.csv";

/// Where the SD-1 taxable value and taxes charged panel is written, relative to the root.
pub const SD1_FIXTURE: &str = "crates/dispersion/fixtures/sd1-district-taxes.csv";

/// Where the per-district casino distribution panel is written, relative to the root.
pub const CASINO_FIXTURE: &str = "crates/dispersion/fixtures/casino-district-distributions.csv";

/// LSC's education analysis of every enacted budget act the corpus describes from one.
///
/// One file rather than one per act, on the same ground [`OPINIONS_FIXTURE`] holds four opinions:
/// they are the same kind of document about the same question, read together or not at all. A file
/// that reads as a series and quietly omits the biennium whose PDF did not download is worse than
/// no file, because a gap in the record and a gap in the cache look identical from the fixture.
///
/// **Named for what it holds rather than for when.** It was `bridge-era-greenbooks` for exactly one
/// commit, which was long enough to be wrong: H.B. 95 and H.B. 119 are the two budgets before the
/// Evidence-Based Model and are not Bridge era at all. A fixture named after the first thing put
/// in it has to be renamed or lied about the second time, and this repository has a README's worth
/// of evidence about which of those actually happens.
///
/// **As enrolled, which is the enacted document.** Not a redbook. That distinction has cost this
/// repository a `[verified]` before: a redbook figure is the executive's proposal, and marking one
/// verified reads as though the act had been checked. Every analysis here is LSC's account of what
/// passed.
///
/// **Twelve records and about 2.6 MB, which is a deliberate choice and the largest fixture here.**
/// It is one document per General Assembly from the 124th to the 135th, unbroken — every act that
/// set Ohio's school funding formula from FY2002 to FY2025. Committed whole rather than sliced for
/// the reason the H.B. 96 greenbook is: a greenbook is about one agency and there is nothing in it
/// to cut away. The alternative to the size is that no claim about any of these acts can be
/// `[verified]` at all, which is what the class looked like three phases ago.
///
/// The **136th is absent on purpose**. H.B. 96's greenbook is committed on its own at
/// [`GREENBOOK_FIXTURE`], where corpus nodes and tests already cite it; duplicating a quarter of a
/// megabyte to make one file look complete would be paying in bytes for a tidier heading.
pub const LSC_GREENBOOK_FIXTURE: &str = "crates/project/fixtures/lsc-education-greenbooks.txt";

/// Where the deflator's check fixture is written, relative to the repository root.
pub const CPI_FIXTURE: &str = "crates/connect/fixtures/cpi-u-june.tsv";
/// Where the Midwest diesel price series is written, relative to the repository root.
pub const DIESEL_FIXTURE: &str = "crates/connect/fixtures/midwest-diesel-monthly.csv";

/// The department's own line-by-line specification of the payment report, one record per year.
///
/// A text fixture rather than a table, on the same ground [`LSC_GREENBOOK_FIXTURE`] is one: it is
/// prose about a method, and the figures in it are scattered through sentences rather than laid
/// out in columns. What it is *for* is the method — it is the only published series that says how
/// the department computed a component in a year whose calculator the department has since
/// replaced.
pub const SFPR_FIXTURE: &str = "crates/project/fixtures/dew-sfpr-line-by-line.txt";

/// The statewide transportation factors, one row per fiscal year.
///
/// Two rows, which is the whole of what is recoverable: the department replaces the calculator
/// rather than archiving it, and only FY2026 survives anywhere besides the current year. Small,
/// and the smallness is the finding — a parameter that moves without an act, observable across
/// exactly one interval.
pub const TRANSPORT_RATES_FIXTURE: &str = "crates/project/fixtures/transportation-rates.csv";
/// Where the 2024-25 report card fixture is written, relative to the repository root.
pub const REPORT_CARD_FIXTURE: &str =
    "crates/dispersion/fixtures/report-card-2425-district-data.csv";
/// Where the FY2025 expenditure-function fixture is written, relative to the repository root.
pub const FUNCTIONS_FIXTURE: &str = "crates/dispersion/fixtures/expenditure-functions-fy25.csv";

/// Where the Census F-33 state comparison is written, relative to the root.
pub const F33_FIXTURE: &str = "crates/dispersion/fixtures/census-f33-states.csv";

/// Where the FY2024 state cross-section is written, relative to the repository root.
///
/// A second fixture rather than a year column on the first, because the FY2022 file is what the
/// corpus's published interstate figures were computed from and a test pins them. Two years side
/// by side is also the only way to see that the Bureau's own per-pupil figures are not
/// reconstructible from these files — see the WP015 catalog entry.
pub const F33_FY2024_FIXTURE: &str = "crates/dispersion/fixtures/census-f33-states-fy2024.csv";

/// Where the per-district F-33 extract is written, relative to the repository root.
pub const F33_DISTRICTS_FIXTURE: &str = "crates/dispersion/fixtures/f33-districts-fy2022.csv";

/// Where the multi-year Ohio F-33 panel is written, relative to the repository root.
pub const F33_OHIO_PANEL_FIXTURE: &str = "crates/dispersion/fixtures/f33-ohio-panel.csv";

/// Where the statute extract is written, relative to the repository root.
pub const STATUTE_FIXTURE: &str = "crates/project/fixtures/revised-code.txt";

/// Where the enacted-act extract is written, relative to the repository root.
pub const ENACTED_FIXTURE: &str = "crates/project/fixtures/enacted-school-funding.txt";

/// Where the department's redbook extract is written, relative to the repository root.
pub const REDBOOK_FIXTURE: &str = "crates/project/fixtures/dew-redbook.txt";

/// The same analysis as enacted. See [`REDBOOK_FIXTURE`], which is the introduced one.
pub const GREENBOOK_FIXTURE: &str = "crates/project/fixtures/dew-greenbook.txt";

/// Where the court opinion extract is written, relative to the repository root.
pub const OPINIONS_FIXTURE: &str = "crates/regime-diff/fixtures/derolph-opinions.txt";

/// Where the EdChoice challenge's appellate record is written, relative to the repository root.
///
/// A second file rather than a fifth record in [`OPINIONS_FIXTURE`], because that file's own first
/// line says *"DeRolph v. State, the four opinions this corpus cites"* and this is neither. The
/// rebuild now selects a connector's sources by the fixture each one declares it feeds, rather
/// than assuming every source of a connector feeds the same file — an assumption that was true
/// while `ohio-courts` held one case and would have quietly stopped being true here.
pub const EDCHOICE_FIXTURE: &str = "crates/regime-diff/fixtures/edchoice-appellate-record.txt";

/// Where the legislative-district crosswalk is written, relative to the repository root.
pub const CROSSWALK_FIXTURE: &str = "crates/project/fixtures/legislative-district-crosswalk.csv";

/// Where the recited transfer orders are written, relative to the repository root.
pub const TRANSFER_FIXTURE: &str = "crates/dispersion/fixtures/territory-transfers.tsv";

/// Where the pre-FY2002 enacted appropriation lines are written, relative to the repository root.
pub const SESSION_LAW_FIXTURE: &str = "crates/project/fixtures/session-law-lines.csv";

/// Where the corrective act's own text is written, relative to the repository root.
///
/// # Why this is a second file and not eight more rows in [`SESSION_LAW_FIXTURE`]
///
/// That fixture is a table, built by finding a department heading and reading the money columns
/// under it. Sub. H.B. 583 of the 134th appropriates almost nothing and amends a great deal, so
/// there is no such table in it to read. What it carries is its own amending title — every
/// Revised Code section it changed, and every uncodified section of H.B. 110 it reopened — and
/// that is prose, which is the form the question about it was asked in.
///
/// Committed whole, for the reason [`GREENBOOK_FIXTURE`] is: it is an education act by its own
/// title, and the retirement, nursing and tax-credit sections sitting beside the school funding
/// ones are a few pages against ninety-four.
pub const CORRECTIONS_FIXTURE: &str = "crates/project/fixtures/hb583-corrections.txt";

/// Where the text of the bill the Fair School Funding Plan was drafted in is written, relative to
/// the repository root.
///
/// # Why an un-enacted bill is committed at all
///
/// H.B. 1 of the 134th General Assembly never became law. H.B. 110 did, and where the two differ
/// the act governs — nothing here is authority for what the law *is*. What it is authority for is
/// what the plan's own drafters wrote, which is a different question and the one
/// [issue #399](https://github.com/goedelsoup/ohio-education-funding/issues/399) asks: whether a
/// choice visible in the enacted text was made in the workgroup bill or in the budget.
///
/// A budget act cannot answer that, because it carries the provision without its history. This
/// bill enacts R.C. 3317.0217 and R.C. 3317.03 in the same document, which makes the two readable
/// against each other at the moment they were written.
///
/// Committed whole, on the same ground [`CORRECTIONS_FIXTURE`] is: it is an education bill by its
/// own title — "Create new school financing system" — and its non-education sections are a few
/// pages against three hundred.
pub const PLAN_BILL_FIXTURE: &str = "crates/project/fixtures/hb1-134-as-introduced.txt";

/// Where the Ohio slice of the CCD agency directory is written, relative to the repository root.
pub const CCD_DIRECTORY_FIXTURE: &str = "crates/dispersion/fixtures/ccd-lea-directory.csv";

/// The department's similar-district classification, one row per district.
///
/// Beside the CUPP profile in `dispersion` because that is where this repository's peer
/// comparison lives, and because the typology is the peer grouping the department itself uses —
/// the CUPP workbook's `Similar District Data` sheet reports a district's group *averages* and
/// never names the group.
///
/// The assignment is 2013 and has not been revised since, so the four measures it was made on are
/// carried beside it under their own vintage. See [`super::typology`].
pub const TYPOLOGY_FIXTURE: &str = "crates/dispersion/fixtures/district-typology.csv";

/// Where the MR-81 sponsor panel is written, relative to the repository root.
pub const MR81_FIXTURE: &str = "crates/dispersion/fixtures/mr81-sponsor-panel.csv";

/// Where the building-level report card extract is written, relative to the repository root.
pub const BUILDING_FIXTURE: &str = "crates/dispersion/fixtures/report-card-2425-buildings.csv";

/// Where the identified-schools extract is written, relative to the repository root.
pub const IDENTIFIED_FIXTURE: &str = "crates/dispersion/fixtures/identified-schools-2026.csv";

/// The appropriation-line series: every Department of Education line item, by fiscal year.
pub const APPROPRIATION_FIXTURE: &str = "crates/project/fixtures/appropriation-lines.csv";

/// Where the scholarship programme summary is written, relative to the repository root.
pub const SCHOLARSHIP_FIXTURE: &str = "crates/project/fixtures/scholarship-programs.csv";

/// Where the archived scholarship participation series is written, relative to the repository
/// root.
///
/// # Why this is a second file and not more rows in [`SCHOLARSHIP_FIXTURE`]
///
/// They are not the same table. That one is one row per programme for a single year, with
/// dollars in it; this is one row per programme per fiscal year per *denominator*, with no
/// dollars anywhere — the department's archive counts applications and payments and never
/// prices them. Merging them would mean a column that is empty for seventeen years out of
/// eighteen, in both directions.
pub const SCHOLARSHIP_HISTORY_FIXTURE: &str = "crates/project/fixtures/scholarship-history.csv";

/// Where the EdChoice designated list is written, relative to the repository root.
///
/// In `dispersion` rather than `project` because it is a per-building panel over every district,
/// which is what the other files in that directory are and what a question about it will join
/// against.
pub const EDCHOICE_DESIGNATED_FIXTURE: &str =
    "crates/dispersion/fixtures/edchoice-designated-2627.csv";

/// The three building-level Performance Index rankings the designation is cut from.
///
/// Beside [`EDCHOICE_DESIGNATED_FIXTURE`] because it is that file's first input, and in
/// `dispersion` for the same reason: a building panel over every district. The corpus has a
/// building Performance Index already, in [`REPORT_CARD_FIXTURE`]'s companion
/// [`BUILDING_FIXTURE`] — for one year, at one decimal place. This is three years at the
/// precision the ranking is actually decided on.
pub const PI_RANKING_FIXTURE: &str = "crates/dispersion/fixtures/edchoice-pi-rankings.csv";

/// The three years of district Title I formula counts the designation's other criterion is cut
/// from, decomposed into the five categories the formula adds.
///
/// The first held source in this repository for Ohio's Title I formula counts. The corpus holds
/// the *shares* these produce — [`EDCHOICE_DESIGNATED_FIXTURE`] carries three years of them —
/// and nothing of the numerator or the denominator until now.
pub const TITLE1_FIXTURE: &str = "crates/dispersion/fixtures/title1-formula-counts.csv";

/// Where the per-line-item money series is written, relative to the repository root.
pub const CATALOG_FIXTURE: &str = "crates/project/fixtures/catalog-line-items.csv";

/// Where the per-line-item legal basis is written.
pub const CATALOG_BASIS_FIXTURE: &str = "crates/project/fixtures/catalog-line-item-basis.tsv";

/// Every fixture [`crate::rebuild`] writes.
///
/// The list exists so that "a source declares a fixture" and "something regenerates it" can be
/// checked against each other without a populated cache, which CI does not have. It caught the
/// F-33 district panel: 754 KB committed, read by a calculator, declared by a `Source`, and
/// produced by nothing — the digest manifest could not see it, because a digest pins the input
/// and says nothing about whether the derivation from it still exists.
pub const REBUILT: &[&str] = &[
    FY27_FIXTURE,
    FY26_FIXTURE,
    FY25_FIXTURE,
    CALCULATOR_SCALARS_FIXTURE,
    CALCULATOR_COUNTS_FIXTURE,
    CALCULATOR_VINTAGES_FIXTURE,
    CALCULATOR_ADM_FIXTURE,
    COMMUNITY_SCHOOL_FUNDING_FIXTURE,
    JVSD_FUNDING_FIXTURE,
    PROFILE_FIXTURE,
    GRADE_BANDS_FIXTURE,
    REPORT_CARD_FIXTURE,
    FUNCTIONS_FIXTURE,
    FINANCE_FIXTURE,
    LANDSCAPE_FIXTURE,
    SD1_FIXTURE,
    CASINO_FIXTURE,
    LSC_GREENBOOK_FIXTURE,
    CPI_FIXTURE,
    DIESEL_FIXTURE,
    SFPR_FIXTURE,
    TRANSPORT_RATES_FIXTURE,
    F33_FIXTURE,
    F33_FY2024_FIXTURE,
    F33_DISTRICTS_FIXTURE,
    F33_OHIO_PANEL_FIXTURE,
    MR81_FIXTURE,
    CCD_DIRECTORY_FIXTURE,
    TYPOLOGY_FIXTURE,
    SESSION_LAW_FIXTURE,
    PLAN_BILL_FIXTURE,
    CORRECTIONS_FIXTURE,
    TRANSFER_FIXTURE,
    BUILDING_FIXTURE,
    IDENTIFIED_FIXTURE,
    EDCHOICE_DESIGNATED_FIXTURE,
    PI_RANKING_FIXTURE,
    TITLE1_FIXTURE,
    CROSSWALK_FIXTURE,
    STATUTE_FIXTURE,
    ENACTED_FIXTURE,
    REDBOOK_FIXTURE,
    GREENBOOK_FIXTURE,
    OPINIONS_FIXTURE,
    EDCHOICE_FIXTURE,
    APPROPRIATION_FIXTURE,
    SCHOLARSHIP_FIXTURE,
    SCHOLARSHIP_HISTORY_FIXTURE,
    CATALOG_FIXTURE,
    CATALOG_BASIS_FIXTURE,
];

/// Fixtures a [`crate::registry::Source`] declares that [`crate::rebuild`] does not produce.
///
/// **An entry here is a debt with a name on it, not a category of fixture.** The pairing is the
/// path and what would have to be written to retire it.
///
/// The list is empty, and the two entries it held are the reason it exists: the F-33 district
/// panel and the legislative-district crosswalk were both committed, both read by a calculator,
/// both declared by a `Source`, and both produced by nothing. Neither was discoverable — a digest
/// pins the bytes that went in and cannot notice that the derivation from them was never written.
///
/// The test holds this list to both directions — an unlisted gap fails, and so does an entry that
/// has quietly been fixed — so it cannot grow silently or rot after the work is done.
pub const NOT_REGENERATED: &[(&str, &str)] = &[];
