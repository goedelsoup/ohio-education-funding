//! The appropriation series: what the General Assembly set aside, by line item and year.
//!
//! Two publishers for one series. The LSC workbooks carry FY2010 onward as data; the greenbooks
//! carry the printed line-item detail tables, which are read by column position because that is
//! the only form they exist in.
//!
//! [`reconcile`] is what makes the two comparable, and it is the reason the series can be quoted
//! at all: an amount published as enacted and the same amount published as an actual are
//! different claims about the same line.

use super::delimited::column_named;

/// Columns of the appropriation series.
pub const APPROPRIATION_HEADER: &[&str] = &[
    "general_assembly",
    "bill",
    "fiscal_year",
    "kind",
    "source",
    "documents",
    "fund_group",
    "fund",
    "line_item",
    "title",
    "amount",
];

/// One budget workbook, with the biennium it appropriates for.
#[derive(Debug, Clone, Copy)]
pub struct AppropriationBook<'a> {
    /// The General Assembly that enacted it.
    pub general_assembly: u16,
    /// The bill, as the registry keys it — `hb96`.
    pub bill: &'a str,
    /// The registry key of the document, which is the provenance of every figure from it.
    pub source: &'a str,
    /// Which of the two documents this is: `enacted` or `actuals`.
    ///
    /// It decides what an unlabelled column means, and nothing else does. The `actuals` workbook
    /// heads its prior years and its closed biennium year alike with a bare `FY 2014`, and marks
    /// only the year still open as `Adj. Appr.`; the `enacted` one labels every biennium column
    /// with the stage it belongs to and leaves only genuine prior years bare.
    pub variant: &'a str,
    /// The first fiscal year of the biennium this act appropriates for.
    pub first_year: u16,
    /// The workbook's first sheet, as rows.
    pub rows: &'a [Vec<String>],
}

/// What a column of amounts is a claim about, or why it is not one.
///
/// # Why this is a whitelist and not a set of exclusions
///
/// The first attempt classified a column by looking for `Actual` or `Approp` in its label and
/// treating everything else as an amount for the year it named. That silently swept in `$ Change`
/// and `% Change` — so `200550 Foundation Funding` in FY2019 arrived three times, as
/// $6,970,372,221.42, $167,292,415.57 and $0.02, with nothing to say which was the appropriation.
///
/// The `as enacted` workbooks make the point sharper still: they carry the bill's entire path,
/// with an amount per fiscal year for *introduced*, each chamber's substitute, each committee
/// report, conference, and finally as enacted. Every one of those is a real dollar figure for a
/// real fiscal year, and all but the last are proposals that never became law. A classifier that
/// recognises what it wants and refuses the rest is the only shape that survives this source.
fn amount_kind(label: &str) -> Option<&'static str> {
    // Collapse the newlines LSC wraps its headers on, and the year, leaving the kind.
    let stripped: String = label
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let mut without_year = String::new();
    let chars: Vec<char> = stripped.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let is_year = index + 4 <= chars.len()
            && chars[index..index + 4].iter().all(char::is_ascii_digit)
            && matches!(
                chars[index..index + 2].iter().collect::<String>().as_str(),
                "19" | "20"
            );
        if is_year {
            index += 4;
        } else {
            without_year.push(chars[index]);
            index += 1;
        }
    }
    let key = without_year
        .to_uppercase()
        .replace("FY", " ")
        .replace(['.', ',', '/'], " ");
    let key = key.split_whitespace().collect::<Vec<_>>().join(" ");

    // Order matters: "ADJUSTED APPROPRIATIONS AS OF 9 30 22" must not fall through to the plain
    // appropriation arm, and "AS ENACTED AFTER GOVERNOR'S VETOES" is still the enacted figure.
    if key.is_empty() {
        // A bare `FY 2024` column. Always a closed prior year in this series — but the caller
        // checks that against the biennium rather than trusting it here.
        Some("bare")
    } else if key.starts_with("ADJ") {
        Some("adjusted")
    } else if key.contains("ACTUAL") {
        Some("actual")
    } else if key.contains("ENACTED") || key == "APPROPRIATION" {
        Some("enacted")
    } else {
        // Introduced, OBM Estimate, House Substitute, Senate Reported, Conference Report, and
        // both change columns. Real figures, and none of them law.
        None
    }
}

/// The first four-digit fiscal year named in a header cell.
fn header_year(label: &str) -> Option<u16> {
    let chars: Vec<char> = label.chars().collect();
    for window in chars.windows(4) {
        if window.iter().all(char::is_ascii_digit) {
            let year: u16 = window.iter().collect::<String>().parse().ok()?;
            if (1990..=2100).contains(&year) {
                return Some(year);
            }
        }
    }
    None
}

/// Build the appropriation-line series from LSC's budget workbooks.
///
/// # Errors
///
/// Returns an error naming the workbook and the column when a header cannot be mapped, when a
/// bare year column falls inside the biennium — where it could be either the enacted amount or a
/// revised one and the label does not say — or when two accepted columns in one document claim
/// the same fiscal year and kind. Each of those is a figure this series would otherwise carry
/// under a label it cannot support.
pub fn build_appropriations(books: &[AppropriationBook<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for book in books {
        let label = format!("{} ({})", book.bill, book.source);
        let header_index = book
            .rows
            .iter()
            .take(8)
            .position(|row| column_named(row, &["ALI"]).is_some())
            .ok_or_else(|| format!("{label}: no header row carrying an `ALI` column"))?;
        let header = &book.rows[header_index];

        let ali =
            column_named(header, &["ALI"]).ok_or_else(|| format!("{label}: no `ALI` column"))?;
        let title = column_named(header, &["ALI Title", "ALIName", "Title"])
            .ok_or_else(|| format!("{label}: no line-item title column"))?;
        let fund_group = column_named(header, &["Fund Group", "FundGroup"]);
        let fund = column_named(header, &["Fund", "Fund Number"]);

        let mut columns: Vec<(usize, u16, &'static str)> = Vec::new();
        let mut seen: Vec<(u16, &'static str)> = Vec::new();
        for (index, cell) in header.iter().enumerate() {
            let Some(year) = header_year(cell) else {
                continue;
            };
            let Some(kind) = amount_kind(cell) else {
                continue;
            };
            let kind = if kind == "bare" {
                // In the revised workbook an unlabelled column is spending: that document exists
                // to report what was spent, it marks the year still open as `Adj. Appr.`, and
                // where a biennium year is genuinely an appropriation — the current biennium, in
                // the 136th — it says `Appropriation` on the column. In the enacted workbook a
                // bare column is only ever a prior year, because every biennium column there
                // carries the legislative stage it belongs to.
                if book.variant == "actuals" || year < book.first_year {
                    "actual"
                } else {
                    return Err(format!(
                        "{label}: column {index} is headed `{}`, which names a biennium year with \
                         no word saying whether it is the enacted amount or a later revision",
                        cell.trim()
                    ));
                }
            } else {
                kind
            };
            if seen.contains(&(year, kind)) {
                return Err(format!(
                    "{label}: two columns claim FY{year} {kind}; the second is `{}`",
                    cell.trim()
                ));
            }
            seen.push((year, kind));
            columns.push((index, year, kind));
        }
        if columns.is_empty() {
            return Err(format!("{label}: no column header names a fiscal year"));
        }

        for row in book.rows.iter().skip(header_index + 1) {
            let Some(line_item) = row.get(ali).map(|cell| cell.trim()) else {
                continue;
            };
            // Line items are numbered by agency and 200 is the Department of Education's.
            // Filtering on the number rather than an agency column is deliberate: three of the
            // sixteen carry no agency column, and two that do spell the agency differently.
            if line_item.len() != 6
                || !line_item.starts_with("200")
                || !line_item.chars().all(|c| c.is_ascii_digit())
            {
                continue;
            }
            let name = row.get(title).map(|cell| cell.trim()).unwrap_or_default();
            let at = |column: Option<usize>| -> String {
                column
                    .and_then(|index| row.get(index))
                    .map(|cell| cell.trim())
                    .unwrap_or_default()
                    .to_string()
            };
            for (index, year, kind) in &columns {
                let Some(raw) = row.get(*index).map(|cell| cell.trim()) else {
                    continue;
                };
                let Some(amount) = crate::conventions::number(raw) else {
                    continue;
                };
                out.push(vec![
                    book.general_assembly.to_string(),
                    book.bill.to_string(),
                    year.to_string(),
                    (*kind).to_string(),
                    book.source.to_string(),
                    at(fund_group),
                    at(fund),
                    line_item.to_string(),
                    name.to_string(),
                    format!("{amount:.2}"),
                ]);
            }
        }
    }
    if out.is_empty() {
        return Err("no appropriation lines were extracted from any workbook".to_string());
    }
    Ok(out)
}

/// Collapse every document into one row per claim, refusing if any two disagree.
///
/// # Errors
///
/// Returns an error naming the claim when two documents give it different amounts.
///
/// # Why the fixture is deduplicated rather than left as it was read
///
/// The documents overlap heavily and that is the source of confidence here: a fiscal year is
/// reported as an enacted amount by the act that made it and as a prior-year actual by the next
/// act's workbook, across layouts that share no conventions. Agreement across those is worth more
/// than any check this crate could write.
///
/// It is also, left alone, a way to be confidently wrong. Summing the raw extract to get "what
/// Ohio appropriated for schools in FY2027" counts every figure twice, because two documents
/// report it — which is how `200550 Foundation Funding` first came out at $17.5 billion against a
/// true $8.7 billion, on a total budget of $15.3 billion. A number that is exactly double is the
/// kind that survives a sanity check.
///
/// So agreement is asserted here, at build time, and the fixture carries one row per claim with
/// `documents` recording how many said it. Provenance is kept: `source` names the document the
/// figure was taken from, preferring the act whose own biennium the year falls in.
pub fn reconcile(rows: Vec<Vec<String>>) -> Result<Vec<Vec<String>>, String> {
    use std::collections::BTreeMap;
    // (fiscal_year, kind, line_item) -> the rows claiming it.
    let mut claims: BTreeMap<(String, String, String), Vec<Vec<String>>> = BTreeMap::new();
    for row in rows {
        claims
            .entry((row[2].clone(), row[3].clone(), row[7].clone()))
            .or_default()
            .push(row);
    }

    let mut out = Vec::with_capacity(claims.len());
    for ((year, kind, item), reports) in claims {
        let first: f64 = reports[0][9].parse().map_err(|_| {
            format!(
                "FY{year} {kind} {item}: {:?} is not a number",
                reports[0][9]
            )
        })?;
        for report in &reports {
            let amount: f64 = report[9]
                .parse()
                .map_err(|_| format!("FY{year} {kind} {item}: {:?} is not a number", report[9]))?;
            if (amount - first).abs() >= 0.005 {
                return Err(format!(
                    "FY{year} {kind} for line item {item} is {} in {} and {} in {}; two \
                     documents disagree about the same figure",
                    report[9], report[4], reports[0][9], reports[0][4]
                ));
            }
        }
        // The act whose own biennium the year falls in is the one that decided it; anything else
        // is reporting it second-hand. `general_assembly` and the year are both on the row.
        let ga_of = |row: &Vec<String>| -> u16 { row[0].parse().unwrap_or(0) };
        let year_number: u16 = year.parse().unwrap_or(0);
        let own = |row: &Vec<String>| -> bool {
            // The Nth General Assembly appropriates for the biennium starting FY(2N + 1754):
            // the 136th for FY2026-27, the 129th for FY2012-13. Derived rather than tabled so it
            // cannot fall out of step with the table in `rebuild`.
            let first_year = 2 * ga_of(row) + 1754;
            year_number == first_year || year_number == first_year + 1
        };
        let chosen = reports
            .iter()
            .find(|row| own(row) && row[4].ends_with("-enacted"))
            .or_else(|| reports.iter().find(|row| own(row)))
            .unwrap_or(&reports[0]);

        let mut row = chosen.clone();
        row.insert(5, reports.len().to_string());
        out.push(row);
    }
    out.sort();
    Ok(out)
}

/// One greenbook, with the biennium it appropriates for.
#[derive(Debug, Clone, Copy)]
pub struct Greenbook<'a> {
    /// The General Assembly that enacted it.
    pub general_assembly: u16,
    /// The bill, as the registry keys it.
    pub bill: &'a str,
    /// The registry key of the document.
    pub source: &'a str,
    /// The first fiscal year of the biennium it appropriates for.
    pub first_year: u16,
    /// `pdftotext -layout` output.
    pub text: &'a str,
}

/// The heading above each page of the line-item table.
const DETAIL_MARKER: &str = "Line Item Detail by Agency";

/// How far apart two right edges may be and still be the same column.
///
/// Amounts are right-aligned, so within a column their ends land within a character or two of
/// each other, while the gap between columns is a dozen or more. Four is comfortably inside that
/// margin in both directions.
const COLUMN_GAP: usize = 4;

/// The fiscal years a page's header names, left to right.
fn header_years(above: &str, below: &str) -> Vec<u16> {
    let mut found: Vec<(usize, u16)> = Vec::new();
    for line in [above, below] {
        let bytes: Vec<char> = line.chars().collect();
        let mut index = 0;
        while index + 3 <= bytes.len() {
            // `FY 1999` or `FY1999`. The bare years in a `2001 to 2002:` span label are
            // deliberately not matched: they are the `% Change` column's caption, not a column.
            if bytes[index] == 'F' && bytes.get(index + 1) == Some(&'Y') {
                let mut cursor = index + 2;
                if bytes.get(cursor) == Some(&' ') {
                    cursor += 1;
                }
                if cursor + 4 <= bytes.len()
                    && bytes[cursor..cursor + 4].iter().all(char::is_ascii_digit)
                {
                    if let Ok(year) = bytes[cursor..cursor + 4].iter().collect::<String>().parse() {
                        if (1990..=2100).contains(&year) && !found.iter().any(|(_, y)| *y == year) {
                            found.push((index, year));
                        }
                    }
                }
            }
            index += 1;
        }
    }
    found.sort_unstable();
    found.into_iter().map(|(_, year)| year).collect()
}

/// The end position of every dollar amount on a line, with its value.
fn amounts(line: &str) -> Vec<(usize, f64)> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '$' {
            index += 1;
            continue;
        }
        let mut cursor = index + 1;
        while chars.get(cursor) == Some(&' ') {
            cursor += 1;
        }
        let start = cursor;
        while cursor < chars.len() && (chars[cursor].is_ascii_digit() || chars[cursor] == ',') {
            cursor += 1;
        }
        if cursor > start {
            let digits: String = chars[start..cursor].iter().filter(|c| **c != ',').collect();
            if let Ok(value) = digits.parse::<f64>() {
                out.push((cursor, value));
            }
        }
        index = cursor.max(index + 1);
    }
    out
}

/// Group right edges into columns.
fn columns(mut ends: Vec<usize>) -> Vec<usize> {
    ends.sort_unstable();
    let mut centres = Vec::new();
    let mut group: Vec<usize> = Vec::new();
    for end in ends {
        if group.last().is_some_and(|last| end - last > COLUMN_GAP) {
            centres.push(group[group.len() / 2]);
            group.clear();
        }
        group.push(end);
    }
    if !group.is_empty() {
        centres.push(group[group.len() / 2]);
    }
    centres
}

/// Whether a line is a line-item row, and its fund, item and title if so.
fn detail_row(line: &str) -> Option<(String, String, String)> {
    let trimmed = line.trim_start();
    let mut parts = trimmed.split_whitespace();
    let fund = parts.next()?.to_string();
    if fund.len() < 3 || fund.len() > 4 || !fund.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let raw = parts.next()?;
    let item: String = raw.chars().filter(char::is_ascii_digit).collect();
    if item.len() != 6
        || !item.starts_with("200")
        || raw.trim_matches(['-'; 1].as_slice()).len() < 6
    {
        return None;
    }
    // The title runs to the first run of two spaces, which is where the figures begin.
    let after = trimmed.split_once(raw)?.1;
    let title = after
        .split("  ")
        .find(|piece| !piece.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_string();
    Some((fund, item, title))
}

/// Build appropriation lines from the greenbook PDFs of the 124th to 128th General Assemblies.
///
/// # Why the columns are calibrated from the figures and not from the header
///
/// The obvious source of column positions is the table header, and the header is not where the
/// data is. Two things had to be measured before this worked. Each table repeats per page and
/// `pdftotext -layout` lays every page out independently, so the same five columns sit at
/// character 60/75/91/107/137 on one page of the 124th and 64/82/100/119/148 on another. And the
/// header *labels* are narrower than the columns of figures beneath them: in the 127th they land
/// at 67, 81 and 91 while the amounts spread far wider, so assigning an amount to its nearest
/// label misdates 16 rows there and 6 in the 124th.
///
/// So the header is used for **which years**, in order, and the figures for **where the columns
/// are**: amounts are right-aligned, so their end positions cluster, and the number of clusters is
/// the number of columns. Across the four documents this assigns 2,185 figures with no row ever
/// claiming one column twice.
///
/// # Errors
///
/// Returns an error naming the page when its cluster count and year count disagree, or when a row
/// puts two amounts in one column. Either means the layout is not the one measured, and a misdated
/// appropriation is a plausible figure in the wrong year rather than a parse failure — these years
/// have no later document to be reconciled against, so refusing is the only guard available.
pub fn build_greenbook_appropriations(books: &[Greenbook<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for book in books {
        let lines: Vec<&str> = book.text.lines().collect();
        let marks: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains(DETAIL_MARKER))
            .map(|(index, _)| index)
            .collect();
        if marks.is_empty() {
            return Err(format!(
                "{}: no `{DETAIL_MARKER}` page; this document keeps its appropriations only in \
                 per-category tables",
                book.source
            ));
        }

        for (position, start) in marks.iter().enumerate() {
            let end = marks.get(position + 1).copied().unwrap_or(lines.len());
            let years = header_years(
                start.checked_sub(1).map(|i| lines[i]).unwrap_or_default(),
                lines.get(start + 1).copied().unwrap_or_default(),
            );
            if years.is_empty() {
                continue;
            }
            let rows: Vec<&str> = lines[*start..end]
                .iter()
                .copied()
                .filter(|line| detail_row(line).is_some())
                .collect();
            if rows.is_empty() {
                continue;
            }
            let centres = columns(
                rows.iter()
                    .flat_map(|line| amounts(line))
                    .map(|(end, _)| end)
                    .collect(),
            );
            if centres.len() != years.len() {
                return Err(format!(
                    "{}: the page at line {start} has {} columns of figures and names {} years",
                    book.source,
                    centres.len(),
                    years.len()
                ));
            }

            for line in rows {
                let Some((fund, item, title)) = detail_row(line) else {
                    continue;
                };
                let mut claimed: Vec<usize> = Vec::new();
                for (position, value) in amounts(line) {
                    let column = centres
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, centre)| centre.abs_diff(position))
                        .map(|(index, _)| index)
                        .unwrap_or_default();
                    if claimed.contains(&column) {
                        return Err(format!(
                            "{}: line item {item} puts two figures in the FY{} column",
                            book.source, years[column]
                        ));
                    }
                    claimed.push(column);
                    // The last two columns are the biennium this act appropriates for; everything
                    // to their left is a closed year, reported as spent.
                    let year = years[column];
                    let kind = if year >= book.first_year {
                        "enacted"
                    } else {
                        "actual"
                    };
                    out.push(vec![
                        book.general_assembly.to_string(),
                        book.bill.to_string(),
                        year.to_string(),
                        kind.to_string(),
                        book.source.to_string(),
                        String::new(),
                        fund.clone(),
                        item.clone(),
                        title.clone(),
                        format!("{value:.2}"),
                    ]);
                }
            }
        }
    }
    if out.is_empty() {
        return Err("no appropriation lines were extracted from any greenbook".to_string());
    }
    Ok(out)
}
