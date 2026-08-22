//! The Census Bureau's F-33 Annual Survey of School System Finances.
//!
//! Three fixtures out of one survey: a state comparison, a district panel for Ohio, and the
//! national district file the panel is cut from. The survey covers every school system in the
//! United States, and what the corpus needs from it is the one thing nothing in Ohio can supply
//! — where Ohio sits against other states on the same definitions.
//!
//! Joining it to Ohio districts needs the NCES-to-IRN map from the CCD directory, which is why
//! [`super::ccd`]'s output is an input here.

use std::collections::BTreeMap;

use super::delimited::{column, delimited_fields};
use super::format::format_value;

/// Header of the state comparison: one row per state, plus the District of Columbia.
///
/// Fifty-one rows out of fourteen thousand. The survey covers every school system in the United
/// States and the corpus needs exactly one thing from it that nothing else can supply — where
/// Ohio sits among the states — so the fixture is the aggregate rather than the panel.
pub const F33_HEADER: &[&str] = &[
    "fips",
    "state",
    "systems",
    "enrollment",
    "total_revenue",
    "federal_revenue",
    "state_revenue",
    "local_revenue",
    "property_tax_revenue",
    "parent_government_revenue",
    "current_spending",
];

/// FIPS code to state name. The survey carries codes only.
const STATE_NAMES: &[(&str, &str)] = &[
    ("01", "Alabama"),
    ("02", "Alaska"),
    ("04", "Arizona"),
    ("05", "Arkansas"),
    ("06", "California"),
    ("08", "Colorado"),
    ("09", "Connecticut"),
    ("10", "Delaware"),
    ("11", "District of Columbia"),
    ("12", "Florida"),
    ("13", "Georgia"),
    ("15", "Hawaii"),
    ("16", "Idaho"),
    ("17", "Illinois"),
    ("18", "Indiana"),
    ("19", "Iowa"),
    ("20", "Kansas"),
    ("21", "Kentucky"),
    ("22", "Louisiana"),
    ("23", "Maine"),
    ("24", "Maryland"),
    ("25", "Massachusetts"),
    ("26", "Michigan"),
    ("27", "Minnesota"),
    ("28", "Mississippi"),
    ("29", "Missouri"),
    ("30", "Montana"),
    ("31", "Nebraska"),
    ("32", "Nevada"),
    ("33", "New Hampshire"),
    ("34", "New Jersey"),
    ("35", "New Mexico"),
    ("36", "New York"),
    ("37", "North Carolina"),
    ("38", "North Dakota"),
    ("39", "Ohio"),
    ("40", "Oklahoma"),
    ("41", "Oregon"),
    ("42", "Pennsylvania"),
    ("44", "Rhode Island"),
    ("45", "South Carolina"),
    ("46", "South Dakota"),
    ("47", "Tennessee"),
    ("48", "Texas"),
    ("49", "Utah"),
    ("50", "Vermont"),
    ("51", "Virginia"),
    ("53", "Washington"),
    ("54", "West Virginia"),
    ("55", "Wisconsin"),
    ("56", "Wyoming"),
];

/// Aggregate the Census F-33 district panel to one row per state.
///
/// # Which systems are counted, and why the rule is enrollment rather than school level
///
/// The survey covers 14,106 school systems at six school levels: elementary-only, secondary-only,
/// unified, vocational and special, nonoperating, and education service agencies. States organise
/// differently — Ohio has 609 unified districts and no elementary-only ones, Illinois has
/// hundreds of both — so any rule stated in terms of school level would count different things in
/// different states, which is precisely what an interstate comparison must not do.
///
/// The rule here is **enrolment above zero**. It admits every system that teaches somebody and
/// excludes the two that would double count: 691 education service agencies, which report revenue
/// received *from* the districts they serve, and 121 nonoperating systems, which levy tax and pay
/// tuition elsewhere. Ohio's own ESCs are in the first group, and the corpus already knows that
/// channel — it is the `total_transfers` line that had to be ruled out as the voucher deduction.
///
/// # Property tax cannot be ranked across states, and local revenue can
///
/// Nine states — Alaska, Connecticut, the District of Columbia, Maryland, Massachusetts, North
/// Carolina, Rhode Island, Tennessee and Virginia — report **zero** school property tax revenue
/// while raising billions locally. They levy plenty; their school districts are dependent
/// agencies of a city or county, so the tax belongs to the parent government and reaches the
/// district as an appropriation. Virginia's parent contributions are 94% of its local school
/// revenue, the District of Columbia's 99%.
///
/// So `property_tax_revenue / total_revenue` is not a national ranking: it silently compares
/// states that report their own levy against states structurally unable to. Massachusetts funds
/// schools from property tax about as heavily as anywhere and scores zero.
///
/// `local_revenue` is the aggregate that survives the difference, because parent contributions
/// are inside it. Rank on that. `parent_government_revenue` travels beside the property tax
/// column so a consumer can see which structure a state has rather than having to know.
///
/// # Dollars are thousands, and enrolment is not
///
/// Every money column in the F-33 is reported in thousands of dollars. The fixture keeps the
/// survey's own unit rather than converting, so a reader comparing it against the published
/// tables is comparing like with like; consumers multiply. Enrolment is a headcount.
///
/// # Read by header, because the layout moved once already
///
/// This builder used a fixed column map, which was accurate for FY2022 and silently wrong for
/// FY2024: the Bureau dropped the `IDCENSUS` column, shifting every index after it by one. Every
/// name the builder wants still exists and still means the same thing, so positional indices
/// would have produced a complete, plausible, entirely wrong fixture — state codes read as unit
/// types, revenue read as enrolment. Names are looked up per file instead.
///
/// Note that these are the Bureau's `elsec` tables, whose names differ from the NCES `sdf`
/// school-district files the Ohio panel reads: property tax is `LOCRPROP` here and `T06` there.
///
/// # Errors
///
/// Returns a message naming the missing column if the layout moves again.
pub fn build_f33_states(rows: &[Vec<String>], label: &str) -> Result<Vec<Vec<String>>, String> {
    let head = rows.first().cloned().unwrap_or_default();
    let at = |name: &str| column(&head, name, label);
    let fips = at("FIPST")?;
    let school_level = at("SCHLEV")?;
    let enrolment = at("ENROLL")?;
    let spending = at("TCURSPND")?;
    let columns = [
        at("TOTALREV")?,
        at("TFEDREV")?,
        at("TSTREV")?,
        at("TLOCREV")?,
        at("LOCRPROP")?,
        at("LOCRPAR")?,
    ];

    let number = |row: &[String], index: usize| -> f64 {
        row.get(index)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .unwrap_or(0.0)
    };

    let mut totals: std::collections::BTreeMap<String, [f64; 8]> =
        std::collections::BTreeMap::new();
    let mut systems: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();

    for row in rows.iter().skip(1) {
        let enrolled = number(row, enrolment);
        if enrolled <= 0.0 {
            continue;
        }
        let Some(code) = row.get(fips).map(|s| s.trim().to_string()) else {
            continue;
        };
        // A school level the survey does not use would mean the layout has moved under us.
        debug_assert!(matches!(
            row.get(school_level).map(String::as_str),
            Some("01" | "02" | "03" | "05" | "06" | "07")
        ));

        *systems.entry(code.clone()).or_default() += 1;
        let entry = totals.entry(code).or_insert([0.0; 8]);
        entry[0] += enrolled;
        for (slot, index) in columns.iter().enumerate() {
            entry[slot + 1] += number(row, *index);
        }
        entry[7] += number(row, spending);
    }

    Ok(totals
        .into_iter()
        .filter_map(|(fips, totals)| {
            // A FIPS code with no name is a territory. The survey carries Puerto Rico and the
            // outlying areas in some years and they are not states, so they are dropped rather
            // than ranked against them.
            let name = STATE_NAMES.iter().find(|(code, _)| *code == fips)?.1;
            let count = systems.get(&fips).copied().unwrap_or(0);
            let mut row = vec![fips, name.to_string(), count.to_string()];
            row.extend(totals.iter().map(|value| format_value(Some(*value), 0)));
            Some(row)
        })
        .collect())
}

/// Columns of the per-district F-33 fixture.
pub const F33_DISTRICTS_HEADER: &[&str] = &[
    "leaid",
    "irn",
    "state",
    "comparable",
    "enrollment",
    "total_revenue",
    "federal_revenue",
    "state_revenue",
    "local_revenue",
    "property_tax",
    "current_spending",
];

/// Agency identifier and school year to Ohio IRN, from the rows [`build_ccd_directory`] wrote.
///
/// Keyed on the year as well as the agency, which is the whole reason the directory is held for
/// thirty years rather than one. Asking the 2022-23 file for a district that closed in 2015 gets
/// nothing back, and the panel then carried an empty identifier for 124 agencies in FY2012 and
/// described the count as a consolidation history. See `dispersion::lea_directory`.
fn directory_irn_map(rows: &[Vec<String>]) -> BTreeMap<(u16, String), String> {
    rows.iter()
        .filter_map(|row| {
            let opens = row.first()?.parse().ok()?;
            Some(((opens, row.get(1)?.clone()), row.get(2)?.clone()))
        })
        .collect()
}

/// `LEAID` to Ohio IRN, from the CCD LEA directory.
///
/// Shared by the national cross-section and the Ohio panel so the join cannot be written twice
/// and drift — the failure the F-33 district panel and the legislative crosswalk both had, where
/// a derivation existed only as prose in two places that could not check each other.
fn ccd_irn_map(directory: &str) -> Result<BTreeMap<String, String>, String> {
    const DIRECTORY: &str = "the CCD LEA directory";
    let mut rows = directory.lines();
    let head = delimited_fields(rows.next().unwrap_or_default(), ',');
    let (key, st_leaid) = (
        column(&head, "LEAID", DIRECTORY)?,
        column(&head, "ST_LEAID", DIRECTORY)?,
    );
    let mut irn_of = BTreeMap::new();
    for line in rows {
        if line.trim().is_empty() {
            continue;
        }
        let f = delimited_fields(line, ',');
        if let (Some(leaid), Some(irn)) = (f.get(key), f.get(st_leaid)) {
            irn_of.insert(leaid.trim().to_string(), irn.trim().to_string());
        }
    }
    Ok(irn_of)
}

/// The per-district F-33 panel: one row per agency the national comparison can use.
///
/// # Why two files
///
/// The Bureau keys the F-33 on `IDCENSUS` and Ohio keys everything on IRN, and the corpus
/// recorded that join as unavailable for as long as `census-f33` has been wired. It was not:
/// NCES publishes the same survey keyed on `LEAID`, and the CCD directory carries `ST_LEAID`,
/// which for Ohio is `OH-` followed by the IRN. `survey` is NCES's `sdf22_1a.txt` and
/// `directory` is the CCD LEA file; neither alone is enough.
///
/// # What is kept
///
/// **Comparable** is the survey's own distinction: `AGCHRT != 1` — no associated charter schools
/// — and `SCHLEV == 03`, a unified elementary-and-secondary agency. Every Ohio agency is kept
/// whether or not it is comparable, flagged in the `comparable` column, because the corpus needs
/// figures for all 968 of them and a national position for only the 611 that have one. Leaving
/// charters in the distribution put Ohio's 200 smallest agencies at an 8% local share, which is a
/// fact about charter finance and not about school districts — see
/// [`dispersion::national_peers`](../../dispersion/src/national_peers.rs).
///
/// A row also needs **enrolment and total revenue above zero**. The survey reports `-1` and `-2`
/// for missing and not-applicable, and 190 agencies carry those in every money column; admitting
/// them would put a district with no reported revenue at a 0% local share, at the bottom of a
/// distribution it is simply absent from. `T06` and `TCURELSC` are blanked rather than dropped
/// when negative, because a district that reports no property tax is usually fiscally dependent
/// rather than untaxed, which is a distinction the consumer draws.
///
/// Money is in thousands of dollars, as the survey publishes it and as
/// [`build_f33_states`] keeps it.
///
/// # Errors
///
/// Returns the missing column's name if either file's layout has moved.
pub fn build_f33_districts(survey: &str, directory: &str) -> Result<Vec<Vec<String>>, String> {
    const SURVEY: &str = "the F-33 district survey";
    let irn_of = ccd_irn_map(directory)?;

    let mut rows = survey.lines();
    let head = delimited_fields(rows.next().unwrap_or_default(), '\t');
    let at = |name: &str| column(&head, name, SURVEY);
    let (leaid, state, charter, level) =
        (at("LEAID")?, at("STABBR")?, at("AGCHRT")?, at("SCHLEV")?);
    let enrolment = at("V33")?;
    let revenue = [
        at("TOTALREV")?,
        at("TFEDREV")?,
        at("TSTREV")?,
        at("TLOCREV")?,
    ];
    let (property_tax, spending) = (at("T06")?, at("TCURELSC")?);

    let mut out = Vec::new();
    for line in rows {
        if line.trim().is_empty() {
            continue;
        }
        let f = delimited_fields(line, '\t');
        let field = |i: usize| f.get(i).map(|v| v.trim()).unwrap_or_default();
        let number = |i: usize| field(i).parse::<i64>().ok();

        let state = field(state).to_string();
        let comparable = field(charter) != "1" && field(level) == "03";
        if !comparable && state != "OH" {
            continue;
        }
        // Missing data is coded negative, not blank, so an unreported agency would otherwise
        // enter the distribution as a real zero.
        if number(enrolment).unwrap_or(-1) <= 0 || number(revenue[0]).unwrap_or(-1) <= 0 {
            continue;
        }

        let key = field(leaid).to_string();
        let irn = if state == "OH" {
            irn_of
                .get(&key)
                .map(|v| v.strip_prefix("OH-").unwrap_or(v).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        // Reported as-is; blanked only where the survey's negative codes mean "not reported".
        let plain = |i: usize| number(i).map(|v| v.to_string()).unwrap_or_default();
        let unreported_is_blank = |i: usize| match number(i) {
            Some(v) if v >= 0 => v.to_string(),
            _ => String::new(),
        };

        let mut row = vec![
            key,
            irn,
            state,
            if comparable { "1" } else { "0" }.to_string(),
            plain(enrolment),
        ];
        row.extend(revenue.iter().map(|i| plain(*i)));
        row.push(unreported_is_blank(property_tax));
        row.push(unreported_is_blank(spending));
        out.push(row);
    }
    Ok(out)
}

/// Columns of the Ohio panel. `fiscal_year` leads because the file's whole purpose is the series.
pub const F33_OHIO_PANEL_HEADER: &[&str] = &[
    "fiscal_year",
    "leaid",
    "irn",
    "comparable",
    "enrollment",
    "total_revenue",
    "federal_revenue",
    "state_revenue",
    "local_revenue",
    "property_tax",
    "current_spending",
];

/// One year of the survey, paired with the fiscal year it reports.
#[derive(Debug, Clone, Copy)]
pub struct PanelYear<'a> {
    /// The fiscal year the file reports, which is not derivable from the file itself.
    pub fiscal_year: u16,
    /// The survey member's text.
    pub survey: &'a str,
}

/// Ohio across every year of the survey this repository holds.
///
/// # Why a second F-33 fixture rather than more rows in the first
///
/// The two answer different questions and need different populations. The national cross-section
/// exists to place one Ohio district among America's — 16,872 agencies, one year — and is the
/// only way to say whether Ohio is unusual. This panel exists to say how Ohio *changed*, which
/// needs many years and only Ohio. Putting ten years of the national file in one fixture would
/// commit roughly 7 MB to answer a question that needs 968 rows a year.
///
/// # The layout genuinely is per-era, and only the column names survive
///
/// The three eras this reads across carry **256, 260 and 354 columns**, the archive member is
/// `sdf121a.txt` in one year and `Sdf16_1a.txt` in another, and the file is tab-delimited
/// throughout. Every column this needs is present in all of them under the same name, which is
/// the entire reason every column is resolved by header: a positional map written against FY2022 would
/// read the wrong field in FY2012 and report it as a number rather than as an error.
///
/// # What is kept, and the one difference from the cross-section
///
/// The same comparability flag and the same non-negative enrolment and revenue test, so a figure
/// here and a figure there mean the same thing. The difference is that **every Ohio agency is
/// kept whatever its flag** — as in the cross-section — but nothing outside Ohio is, and the year
/// is carried on the row rather than in the filename.
///
/// # The join is to one directory, and that is a real limitation
///
/// `LEAID` to IRN comes from the FY2022-23 CCD directory, because that is the directory this
/// repository holds. An agency that existed in FY2012 and had closed or merged by FY2023 is in
/// the survey and not in the directory, so it keeps its `LEAID` and gets an empty `irn`. That is
/// the consolidation problem `nces-ccd` was approved to solve and has not, recorded on the rows
/// it affects rather than in prose: count the blank IRNs per year and the panel tells you how
/// much of Ohio's agency population it cannot name.
///
/// # Errors
///
/// Returns the missing column's name if any year's layout has moved.
pub fn build_f33_ohio_panel(
    years: &[PanelYear<'_>],
    directory: &[Vec<String>],
) -> Result<Vec<Vec<String>>, String> {
    let irn_of = directory_irn_map(directory);
    let mut out = Vec::new();

    for year in years {
        let label = format!("the F-33 district survey for FY{}", year.fiscal_year);
        let mut rows = year.survey.lines();
        let head = delimited_fields(rows.next().unwrap_or_default(), '\t');
        let at = |name: &str| column(&head, name, &label);
        let (leaid, state, charter, level) =
            (at("LEAID")?, at("STABBR")?, at("AGCHRT")?, at("SCHLEV")?);
        let enrolment = at("V33")?;
        let revenue = [
            at("TOTALREV")?,
            at("TFEDREV")?,
            at("TSTREV")?,
            at("TLOCREV")?,
        ];
        let (property_tax, spending) = (at("T06")?, at("TCURELSC")?);

        let mut kept = 0usize;
        for line in rows {
            if line.trim().is_empty() {
                continue;
            }
            let f = delimited_fields(line, '\t');
            let field = |i: usize| f.get(i).map(|v| v.trim()).unwrap_or_default();
            let number = |i: usize| field(i).parse::<i64>().ok();

            if field(state) != "OH" {
                continue;
            }
            if number(enrolment).unwrap_or(-1) <= 0 || number(revenue[0]).unwrap_or(-1) <= 0 {
                continue;
            }

            let key = field(leaid).to_string();
            // The survey's fiscal year and the directory's school year name the same year: FY2012
            // finance sits beside the 2011-12 directory. Resolved against that year where it is
            // held and against the nearest later one otherwise, so an agency is named by a file
            // written while it existed rather than by one written a decade after it closed.
            let irn = irn_of
                .get(&(year.fiscal_year.saturating_sub(1), key.clone()))
                .or_else(|| {
                    irn_of
                        .range((year.fiscal_year.saturating_sub(1), key.clone())..)
                        .find(|((_, leaid), _)| *leaid == key)
                        .map(|(_, irn)| irn)
                })
                .cloned()
                .unwrap_or_default();
            let plain = |i: usize| number(i).map(|v| v.to_string()).unwrap_or_default();
            let unreported_is_blank = |i: usize| match number(i) {
                Some(v) if v >= 0 => v.to_string(),
                _ => String::new(),
            };

            let mut row = vec![
                year.fiscal_year.to_string(),
                key,
                irn,
                if field(charter) != "1" && field(level) == "03" {
                    "1"
                } else {
                    "0"
                }
                .to_string(),
                plain(enrolment),
            ];
            row.extend(revenue.iter().map(|i| plain(*i)));
            row.push(unreported_is_blank(property_tax));
            row.push(unreported_is_blank(spending));
            out.push(row);
            kept += 1;
        }

        // A year that contributes nothing is a layout that parsed but matched no Ohio row, which
        // reads as a shorter series rather than as a failure. Ohio has never had fewer than 900
        // reporting agencies in any year the survey covers.
        if kept < 500 {
            return Err(format!(
                "FY{} yielded {kept} Ohio agencies; the survey has never had fewer than 900, so \
                 the state filter or the file is wrong",
                year.fiscal_year
            ));
        }
    }
    Ok(out)
}
