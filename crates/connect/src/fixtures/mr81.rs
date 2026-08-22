//! MR-81, the child nutrition sponsor panel, across four published site layouts.
//!
//! The longest-running series the corpus retrieves and the one with the least stable format: four
//! distinct site layouts over the period, two of them printed reports read by character position
//! rather than by delimiter. Each site's reported share is checked against the totals its own
//! report prints, because a printed column read one position off still parses.

use std::collections::BTreeMap;

use super::delimited::{column, delimited_fields};
use super::text::fixed;

/// Columns of the MR-81 sponsor panel.
///
/// `stream`, `identified` and `claimable` exist for the same reason `enrollment_basis` does: the
/// report stops being one report. From FY2012 it is published as three, and the three do not
/// count the same thing. See [`build_mr81`].
pub const MR81_HEADER: &[&str] = &[
    "fiscal_year",
    "sponsor_irn",
    "sponsor_name",
    "county",
    "sponsor_type",
    "stream",
    "sites",
    "enrollment",
    "enrollment_basis",
    "free_lunch",
    "reduced_lunch",
    "identified",
    "claimable",
];

/// Which of the report's publications a row came from.
///
/// Not cosmetic. Each stream reports a different quantity in the same columns, and a consumer
/// that adds them without reading this gets a number that looks like a poverty rate and is not
/// one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// FY1998 through FY2011, when the report was one file covering every sponsor.
    Single,
    /// From FY2012: sponsors still collecting meal applications. The header says outright that it
    /// *"Excludes Provision 2 and Community Eligibility Option (CEO) sponsors."*
    Traditional,
    /// From FY2012: sponsors serving under Provision 2, whose approvals are **frozen at a base
    /// year**. `ProvisionYear` names it, and the same sponsor reports the same free and reduced
    /// counts in FY2012, FY2013 and FY2014 while its enrolment moves underneath them.
    Provision2,
    /// From FY2012: community eligibility. **No applications are collected at all**, so free and
    /// reduced are structurally zero and the comparable quantity is `identified`.
    Community,
}

impl Stream {
    /// The value written to the fixture.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Traditional => "traditional",
            Self::Provision2 => "provision2",
            Self::Community => "community",
        }
    }
}

/// How a year's file is laid out, which decides which reader runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mr81Layout {
    /// FY1998 through FY2000: one row per **school**, 170 characters wide, with the district
    /// named and numbered on every row.
    ///
    /// Nominally comma-separated and actually fixed-width, which is the whole reason it is read
    /// by offset. Seven district names a year carry a comma — `Graham School, The`,
    /// `Holy Trinity, Swanton` — and splitting on it shifts those rows past the last column. The
    /// separators sit at the same thirteen offsets in all 4,246 rows of all three files, so
    /// slicing never meets the problem.
    SchoolCentric,
    /// FY2001 through FY2014: one row per school **site**, with its sponsor named on the row.
    Delimited,
    /// The printed report, for the one stream-year posted no other way.
    ///
    /// Sponsor blocks with a repeated column heading, a `SPONSOR TOTAL` after each and a `STATE
    /// TOTAL` at the end. Fixed-width as well, and for the same reason — five site names a year
    /// run flush into the IRN beside them with no space between.
    Printed,
}

/// One published MR-81 file: which October, which stream, which layout, and its text.
#[derive(Debug, Clone, Copy)]
pub struct Mr81Report<'a> {
    /// The October the report counts.
    pub year: u16,
    /// Which of the three publications this is.
    pub stream: Stream,
    /// Which reader this file needs.
    pub layout: Mr81Layout,
    /// The file's text.
    pub text: &'a str,
}

/// One sponsor's running totals while its school sites are being summed.
#[derive(Debug, Default)]
struct SponsorTotal {
    name: String,
    county: String,
    kind: String,
    sites: usize,
    enrollment: i64,
    free: i64,
    reduced: i64,
    identified: i64,
    claimable: i64,
}

/// One school site, whichever layout it was read from.
struct Site {
    sponsor_irn: String,
    sponsor_name: String,
    county: String,
    kind: String,
    enrollment: i64,
    free: i64,
    reduced: i64,
    identified: i64,
    /// The percentage the report itself prints in its free-and-reduced-of-enrolment column.
    reported_share: f64,
}

/// Sponsor number and type by name, for the printed report that carries neither.
#[derive(Debug, Default)]
struct Identity {
    /// Keyed on name and county together, which is how all but one sponsor resolves.
    by_place: BTreeMap<(String, String), (String, String)>,
    /// Keyed on name alone, `None` where the name is not unique.
    by_name: BTreeMap<String, Option<(String, String)>>,
}

impl Identity {
    /// Both sides of the lookup pass through here, because they do not arrive alike: names taken
    /// from a delimited file have had their commas substituted for the fixture's sake and names
    /// read off the printed report have not. `Virtual Schoolhouse, Inc.` is one of them.
    fn key(raw: &str) -> String {
        without_comma(raw).to_lowercase()
    }

    fn insert(&mut self, name: &str, county: &str, irn: &str, kind: &str) {
        let value = (irn.to_string(), kind.to_string());
        self.by_place
            .insert((Self::key(name), Self::key(county)), value.clone());
        match self.by_name.entry(Self::key(name)) {
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(Some(value));
            }
            std::collections::btree_map::Entry::Occupied(mut slot) => {
                if slot.get().as_ref().is_some_and(|held| held.0 != value.0) {
                    slot.insert(None);
                }
            }
        }
    }

    /// The county is tried first and the name alone second.
    ///
    /// The fallback is not tidiness. `Believe to Achieve-Canton` is filed under Cuyahoga in the
    /// FY2012 file and under Stark in the FY2013 one — the publisher moved it, and on a
    /// name-and-county key it would be a sponsor this repository had never seen.
    fn get(&self, name: &str, county: &str) -> Option<&(String, String)> {
        self.by_place
            .get(&(Self::key(name), Self::key(county)))
            .or_else(|| self.by_name.get(&Self::key(name))?.as_ref())
    }
}

/// The sponsor types the report uses, which is also how a shifted row is put back together.
const SPONSOR_TYPES: [&str; 4] = [
    "Public",
    "Non-Public",
    "Residential Child Care Institution",
    "Camp",
];

/// Sponsors across every October of MR-81 this repository holds, aggregated from school sites.
///
/// # What this is, and what the catalog said it was
///
/// MR-81 is the Office for Child Nutrition's free and reduced-price lunch report, one row per
/// school site grouped by sponsor. It is not an enrollment archive; it carries an enrolment
/// column because a lunch claim needs a denominator. Its value here is the **free and
/// reduced-price counts**, which are the closest available long series of the measure Ohio's
/// disadvantaged pupil funding is paid on — R.C. 3317.03(B)(21) hands the definition of
/// "economically disadvantaged" to the department, and free-lunch eligibility has been the
/// department's operative test.
///
/// # Three breaks, each carried on the row rather than annotated
///
/// **The denominator is renamed in FY2010.** Through FY2009 the column is `AdmCount`. From
/// FY2010 it is `CECount`, and the report's own header defines CE as the *"highest daily number
/// of students with access to the program"* — not average daily membership and not the same
/// quantity. `enrollment_basis` says which.
///
/// **The report splits into three in FY2012**, and the three count differently. Traditional is
/// current-year applications. Provision 2 is applications **frozen at a base year** — the same
/// sponsor reports the same free and reduced counts three years running while its enrolment moves
/// underneath them. Community eligibility collects **no applications at all**: its free and
/// reduced columns are zero by construction, and what it publishes instead is
/// `CEOEligibleStudents`, the directly-certified count. `stream` says which, and a consumer that
/// sums the three without reading it produces a poverty rate that falls because the poorest
/// sponsors stopped filling in forms.
///
/// `identified` and `claimable` bound what the community stream can be read as. `identified` is
/// the published directly-certified count — a floor, because direct certification reaches SNAP,
/// TANF, foster and homeless children and nobody else. `claimable` is that count run through
/// USDA's 1.6 multiplier and capped at enrolment, which is the ceiling the programme itself uses
/// and which reproduces the report's own printed percentage in all 735 of the FY2014 rows. For
/// the two application streams both equal the approvals, so a consumer can carry either bound
/// across the whole panel without a special case.
///
/// **The grain changes in FY2001.** FY1998-FY2000 are one row per school with the district on it;
/// everything later is one row per site with the sponsor on it. Both aggregate to the same shape,
/// but a district is not a sponsor: the earlier files have no sponsor-type column at all, so
/// their rows are written `Unknown` rather than guessed at.
///
/// # Sponsors are not districts
///
/// `SponsorType` is carried rather than filtered on. "Public" includes county boards of
/// developmental disabilities and community schools alongside traditional districts, and the
/// report also covers non-public schools, residential child care institutions and camps.
/// Deciding which sponsors are districts needs a join this function does not have; emitting the
/// type lets the consumer draw the line and lets a test count what was drawn.
///
/// # Every row is checked against the percentage printed beside it
///
/// The FY2001 file is the only comma-delimited one, and nine of its rows carry a comma inside a
/// school name. Split positionally, those rows put a **site IRN into the enrolment column** —
/// `00026450` and `00093153` for two Cleveland City schools, adding 119,603 students to a
/// district of 73,562 and understating the statewide FY2001 poverty share by 1.8 points. The
/// figure was plausible, the row count was right, and the panel shipped that way.
///
/// So the columns are checked against each other: the report prints free-and-reduced as a share
/// of enrolment beside the counts, and a shifted row fails that arithmetic by orders of
/// magnitude. Rows reporting **no applications at all** are exempt, because that is the
/// community-eligibility signature rather than a parse failure — and the exemption is itself
/// informative, since those are precisely the rows whose printed percentage is a claiming rate.
///
/// # Errors
///
/// Returns the missing column's name if any year's layout has moved, or a description of the
/// arithmetic that failed.
pub fn build_mr81(reports: &[Mr81Report<'_>]) -> Result<Vec<Vec<String>>, String> {
    // The printed report names its sponsors and neither numbers nor types them. Built in full
    // before anything is read, so the lookup does not depend on the order reports arrive in.
    let mut identity = Identity::default();
    for report in reports {
        if report.layout != Mr81Layout::Delimited {
            continue;
        }
        let label = format!("the MR-81 report for October {}", report.year);
        for site in delimited_sites(report.text, &label)?.0 {
            identity.insert(
                &site.sponsor_name,
                &site.county,
                &site.sponsor_irn,
                &site.kind,
            );
        }
    }

    // Sponsor totals, keyed so the output is stable without a sort: year, stream, then IRN.
    let mut totals: BTreeMap<(u16, &'static str, String), SponsorTotal> = BTreeMap::new();
    let mut basis: BTreeMap<u16, &'static str> = BTreeMap::new();

    for report in reports {
        let label = format!(
            "the MR-81 {} report for October {}",
            report.stream.label(),
            report.year
        );
        let (sites, which) = match report.layout {
            Mr81Layout::SchoolCentric => (school_centric_sites(report.text, &label)?, "adm"),
            Mr81Layout::Delimited => delimited_sites(report.text, &label)?,
            // The one printed file is FY2013's, which is well inside the CE era. Stated rather
            // than sniffed because the printed heading spells CE out in prose and not in a
            // column name.
            Mr81Layout::Printed => (printed_sites(report.text, &identity, &label)?, "ce"),
        };
        // Every stream of one October shares a denominator, and they agree.
        if let Some(prior) = basis.insert(report.year, which) {
            if prior != which {
                return Err(format!(
                    "October {} is published on both {prior} and {which}",
                    report.year
                ));
            }
        }

        // The traditional and single-stream files have never carried fewer than three thousand
        // sites; the other two are small by construction and only have to be non-empty.
        let floor = match report.stream {
            Stream::Single | Stream::Traditional => 2000,
            Stream::Provision2 | Stream::Community => 1,
        };
        if sites.len() < floor {
            return Err(format!(
                "{label} yielded {} sites against a floor of {floor}, so the delimiter or the \
                 layout is wrong",
                sites.len()
            ));
        }

        for site in sites {
            check_reported_share(&site, &label)?;
            let entry = totals
                .entry((report.year, report.stream.label(), site.sponsor_irn))
                .or_insert_with(|| SponsorTotal {
                    name: site.sponsor_name,
                    county: site.county,
                    kind: site.kind,
                    ..SponsorTotal::default()
                });
            entry.sites += 1;
            entry.enrollment += site.enrollment;
            entry.free += site.free;
            entry.reduced += site.reduced;
            entry.identified += site.identified;
            // Capped at enrolment site by site, because that is where the programme caps it.
            // Rolled up first, the cap would let one site's slack forgive another's overshoot.
            entry.claimable += if report.stream == Stream::Community {
                (site.identified * 8 / 5).min(site.enrollment)
            } else {
                site.free + site.reduced
            };
        }
    }

    Ok(totals
        .into_iter()
        .map(|((year, stream, irn), t)| {
            vec![
                year.to_string(),
                irn,
                t.name,
                t.county,
                t.kind,
                stream.to_string(),
                t.sites.to_string(),
                t.enrollment.to_string(),
                (*basis.get(&year).unwrap_or(&"adm")).to_string(),
                t.free.to_string(),
                t.reduced.to_string(),
                t.identified.to_string(),
                t.claimable.to_string(),
            ]
        })
        .collect())
}

/// Zero-padded to six, because the years disagree: FY2004 writes `00043786` and FY2005 writes
/// `43786` for Cleveland. Joining the panel to itself across years — or to anything else in this
/// repository, which uses the padded form — silently matches nothing for whichever half is
/// written the other way.
fn sponsor_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_start_matches('0');
    (!trimmed.is_empty()).then(|| format!("{trimmed:0>6}"))
}

/// `write_csv` does not quote and every consumer splits on the comma, so a sponsor called "Edge
/// Academy, The" would shift its own row's remaining columns. Ninety-nine rows were written that
/// way before the writer learned to refuse them. Substituted rather than dropped, because the
/// name is how a reader recognises the sponsor and the IRN beside it is the key anything joins on.
fn without_comma(raw: &str) -> String {
    raw.trim().replace(',', ";")
}

/// The printed percentage has to reproduce from the counts beside it, or the row is misread.
fn check_reported_share(site: &Site, label: &str) -> Result<(), String> {
    // No applications at all is the community-eligibility signature, not a shifted row: those
    // files publish a claiming rate in this column and nothing to reproduce it from.
    if site.enrollment <= 0 || (site.free == 0 && site.reduced == 0) {
        return Ok(());
    }
    let computed = 100.0 * (site.free + site.reduced) as f64 / site.enrollment as f64;
    if (computed - site.reported_share).abs() > 0.02 {
        return Err(format!(
            "in {label}, a site of {} in {} reports {} and {} approvals against {} enrolled, \
             which is {computed:.2}% and not the {:.2}% printed beside it — the columns have \
             shifted",
            site.sponsor_name,
            site.county,
            site.free,
            site.reduced,
            site.enrollment,
            site.reported_share,
        ));
    }
    Ok(())
}

/// FY2001-FY2014, one row per site, and which denominator the file is on.
fn delimited_sites(text: &str, label: &str) -> Result<(Vec<Site>, &'static str), String> {
    let mut lines = text.lines();
    let header_line = lines.next().unwrap_or_default();
    // FY2001 is comma-delimited and every later year is tab. Sniffed rather than tabulated,
    // because the delimiter is visible in the file and a table is another thing to maintain.
    let delimiter = if header_line.contains('\t') {
        '\t'
    } else {
        ','
    };
    // The FY2012 community file's header carries an empty column before `CEOEligibleStudents`
    // that its data rows do not, so the header is one wider than every row beneath it. Dropped
    // here, because a name resolved against it would point one past the end of the data.
    let head: Vec<String> = delimited_fields(header_line, delimiter)
        .into_iter()
        .filter(|h| !h.trim().is_empty())
        .collect();
    let at = |name: &str| column(&head, name, label);

    let (county, irn, name, kind) = (
        at("County")?,
        at("SponsorIRN")?,
        at("SponsorName")?,
        at("SponsorType")?,
    );
    let free = at("FreeLunchApps")?;
    let reduced = at("RedLunchApps")?;
    let (enrolment, which, share) = match at("AdmCount") {
        Ok(i) => (i, "adm", at("PctFreeRedAdm")?),
        Err(_) => (at("CECount")?, "ce", at("PctFreeRedCE")?),
    };
    // Only the community files carry it, so its absence is a fact about the file rather than an
    // error: a stream that still collects applications has no directly-certified column because
    // it does not run on one.
    let identified = at("CEOEligibleStudents").ok();

    let mut sites = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let f = rejoin(delimited_fields(line, delimiter), head.len());
        let field = |i: usize| f.get(i).map(|v| v.trim()).unwrap_or_default();
        let number = |i: usize| field(i).parse::<i64>().unwrap_or(0);
        let Some(key) = sponsor_key(field(irn)) else {
            continue;
        };
        sites.push(Site {
            sponsor_irn: key,
            sponsor_name: without_comma(field(name)),
            county: without_comma(field(county)),
            kind: without_comma(field(kind)),
            enrollment: number(enrolment),
            free: number(free),
            reduced: number(reduced),
            identified: identified.map_or(0, number),
            reported_share: field(share).parse::<f64>().unwrap_or(0.0),
        });
    }
    Ok((sites, which))
}

/// Put a row back together when an unquoted name has split across the delimiter.
///
/// Nine FY2001 rows do this and nothing else in the panel does. The repair anchors from the
/// right, where the shape is fixed — site IRN, kitchen type, base IRN and six figures close every
/// row — and finds the sponsor type in what is left, since it comes from a four-word vocabulary
/// no school name collides with. What precedes the type is the sponsor's name and what follows it
/// is the breakfast flag and the school's name, each rejoined with the comma that split it.
///
/// A row this cannot read is returned untouched, so it fails the arithmetic check rather than
/// being quietly reshaped into something plausible.
fn rejoin(fields: Vec<String>, expected: usize) -> Vec<String> {
    /// Site IRN, kitchen type, base IRN, and the six figures that close every row.
    const TAIL: usize = 9;
    if fields.len() <= expected || expected < TAIL + 2 || fields.len() <= TAIL + 3 {
        return fields;
    }
    let middle = &fields[2..fields.len() - TAIL];
    let mut found = middle
        .iter()
        .enumerate()
        .filter(|(_, v)| SPONSOR_TYPES.contains(&v.trim()));
    let Some((at, _)) = found.next() else {
        return fields;
    };
    // Two candidates means the vocabulary matched something that is not the type column, and
    // guessing between them is exactly what this is meant not to do.
    if found.next().is_some() || at + 2 > middle.len() {
        return fields;
    }
    let mut out = fields[..2].to_vec();
    out.push(middle[..at].join(","));
    out.push(middle[at].clone());
    // Between the type and the school name sits the one-character breakfast flag; everything
    // after it belongs to the name, however many commas it was written with.
    out.push(middle[at + 1].clone());
    out.push(middle[at + 2..].join(","));
    out.extend_from_slice(&fields[fields.len() - TAIL..]);
    if out.len() == expected {
        out
    } else {
        fields
    }
}

/// FY1998-FY2000, one fixed-width row per school with its district named on it.
///
/// The header names fourteen columns and the rows are 170 characters with the separators at the
/// same offsets throughout, so the fields are taken by offset. `% REDU` is misnamed in that
/// header: it holds free **and** reduced as a share of ADM, which every row's arithmetic confirms
/// and which is what the later files call `PctFreeRedAdm`.
///
/// The three files were regenerated together in 2004 from one roster — all three carry the same
/// 4,246 schools, with the ones not participating in a given October written as zeros. Sites are
/// therefore counted only where the school reported an enrolment, or a district's site count
/// would include schools that were not open.
fn school_centric_sites(text: &str, label: &str) -> Result<Vec<Site>, String> {
    /// Start and end of each field, from the separator offsets shared by every row.
    const CUTS: [(usize, usize); 14] = [
        (0, 1),
        (2, 32),
        (33, 39),
        (40, 50),
        (51, 57),
        (58, 67),
        (68, 77),
        (78, 87),
        (88, 97),
        (98, 107),
        (108, 117),
        (118, 152),
        (153, 163),
        (164, 170),
    ];
    const WIDTH: usize = 170;
    const ADM: usize = 5;
    const FREE: usize = 6;
    const REDUCED: usize = 7;
    const SHARE: usize = 10;
    const DISTRICT: usize = 11;
    const COUNTY: usize = 12;
    const DISTRICT_IRN: usize = 13;

    let mut lines = text.lines();
    let header = lines.next().unwrap_or_default();
    for name in ["SCHOOL IRN", "ADM", "FREE", "REDUCED", "DISTRICT IRN"] {
        if !header.contains(name) {
            return Err(format!(
                "{label} has no {name} column; its layout has moved"
            ));
        }
    }

    let mut sites = Vec::new();
    for line in lines {
        let row: Vec<char> = line.chars().collect();
        if row.len() != WIDTH {
            continue;
        }
        let at = |i: usize| fixed(&row, CUTS[i].0, CUTS[i].1);
        let number = |i: usize| at(i).parse::<i64>().unwrap_or(0);
        let Some(key) = sponsor_key(&at(DISTRICT_IRN)) else {
            continue;
        };
        if number(ADM) <= 0 {
            continue;
        }
        sites.push(Site {
            sponsor_irn: key,
            sponsor_name: without_comma(at(DISTRICT).trim_end_matches('.')),
            county: without_comma(&at(COUNTY)),
            // These files have no sponsor type. Resolved from FY2001 downstream, where the same
            // district carries the same number; left visible rather than guessed at here.
            kind: "Unknown".to_string(),
            enrollment: number(ADM),
            free: number(FREE),
            reduced: number(REDUCED),
            identified: 0,
            reported_share: at(SHARE).parse::<f64>().unwrap_or(0.0),
        });
    }
    if sites.is_empty() {
        return Err(format!("{label} yielded no rows {WIDTH} characters wide"));
    }
    Ok(sites)
}

/// The printed report, for the one stream-year published no other way.
///
/// FY2013's community file is posted rendered and not delimited, so this reads the rendering.
/// Both the per-sponsor and the statewide totals it prints are recomputed against what was
/// parsed, so a row this misses is an error rather than a quietly smaller number — which is how
/// the two sites whose IRN is printed seven digits wide instead of eight were found.
///
/// The sponsor is named and neither numbered nor typed, so [`Identity`] supplies both from the
/// delimited files of the years either side. A sponsor it cannot name is an error: an unnumbered
/// row cannot be joined to anything and would sit in the panel as a sponsor of its own.
fn printed_sites(text: &str, identity: &Identity, label: &str) -> Result<Vec<Site>, String> {
    /// The site IRN column, right-aligned within it.
    const IRN: (usize, usize) = (39, 47);

    let mut sites: Vec<Site> = Vec::new();
    let (mut name, mut county) = (String::new(), String::new());
    let mut running = [0i64; 3];
    let mut counted = 0usize;
    let mut state: Option<[i64; 3]> = None;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("SPONSOR: ") {
            name = rest.trim().to_string();
            running = [0; 3];
            counted = 0;
            continue;
        }
        if let Some(rest) = line.strip_prefix("COUNTY:") {
            county = rest.trim().to_string();
            continue;
        }
        if let Some(rest) = line.strip_prefix("SPONSOR TOTAL") {
            let stated = printed_totals(rest);
            if stated != running {
                return Err(format!(
                    "in {label}, {name} of {county} sums to {running:?} across {counted} sites \
                     against the {stated:?} it prints"
                ));
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("STATE TOTAL") {
            state = Some(printed_totals(rest));
            continue;
        }
        let row: Vec<char> = line.chars().collect();
        let site_irn = fixed(&row, IRN.0, IRN.1);
        if site_irn.is_empty() || !site_irn.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let figures: Vec<String> = line
            .chars()
            .skip(IRN.1)
            .collect::<String>()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        // Enrolment, free, reduced, their total, two percentages, and the identified count.
        if figures.len() < 7 {
            continue;
        }
        let number = |i: usize| figures[i].parse::<i64>().unwrap_or(0);
        let Some((irn, kind)) = identity.get(&name, &county) else {
            return Err(format!(
                "in {label}, {name} of {county} is named by no delimited file, so it has no IRN"
            ));
        };
        running[0] += number(0);
        running[1] += number(1);
        running[2] += number(2);
        counted += 1;
        sites.push(Site {
            sponsor_irn: irn.clone(),
            sponsor_name: without_comma(&name),
            county: without_comma(&county),
            kind: kind.clone(),
            enrollment: number(0),
            free: number(1),
            reduced: number(2),
            identified: number(6),
            reported_share: figures[5].parse::<f64>().unwrap_or(0.0),
        });
    }

    let Some(stated) = state else {
        return Err(format!("{label} prints no state total to check against"));
    };
    let summed = [
        sites.iter().map(|s| s.enrollment).sum::<i64>(),
        sites.iter().map(|s| s.free).sum::<i64>(),
        sites.iter().map(|s| s.reduced).sum::<i64>(),
    ];
    if summed != stated {
        return Err(format!(
            "{label} sums to {summed:?} across {} sites against the {stated:?} it prints",
            sites.len()
        ));
    }
    Ok(sites)
}

/// Enrolment, free and reduced off a printed total line.
fn printed_totals(rest: &str) -> [i64; 3] {
    let figures: Vec<i64> = rest
        .split_whitespace()
        .map(|f| f.parse::<i64>().unwrap_or(0))
        .collect();
    [
        figures.first().copied().unwrap_or(0),
        figures.get(1).copied().unwrap_or(0),
        figures.get(2).copied().unwrap_or(0),
    ]
}
