//! The Catalog of Budget Line Items, which reaches back further than the workbook series.
//!
//! Used only for enacted figures in years the workbooks do not cover — the Catalog restates an
//! actual as a year closes, so mixing the two would build a series whose meaning changes partway
//! through. Headings, amounts, and the labels above the amount columns are each read separately,
//! because a line item's title may contain a comma and its amounts may not all be present.

/// One appropriation line item in one edition, before its years are split out.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogItem {
    /// The fund the line is paid from: `GRF`, `3AF`, `5VS0`.
    pub fund: String,
    /// The line item number, hyphens removed: `200550`.
    pub ali: String,
    /// The line's name as printed.
    pub name: String,
    /// The act that authorises it this biennium, and the act that established it.
    pub legal_basis: String,
    /// `(fiscal year, kind, amount)`, where `amount` is `None` for a printed `---`.
    pub years: Vec<(u16, String, Option<f64>)>,
}

/// A line-item heading: fund, number, and name.
///
/// The number is printed `200-100` in the 2006 edition and `200100` from 2008, so the hyphen is
/// optional here and stripped from the value. Without that the same line item reads as two
/// different ones across the series, which is the join this whole extract exists to support.
fn catalog_heading(line: &str) -> Option<(String, String, String)> {
    let mut parts = line.split_whitespace();
    let fund = parts.next()?;
    // Funds are short alphanumeric codes. Anything longer is prose that happens to start a line.
    if fund.len() > 5 || !fund.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let raw = parts.next()?;
    let ali: String = raw.chars().filter(char::is_ascii_digit).collect();
    if ali.len() != 6 || raw.chars().any(|c| !c.is_ascii_digit() && c != '-') {
        return None;
    }
    let name = parts.collect::<Vec<_>>().join(" ");
    // The 2025 layout appends a cross-reference like `IId-3470` to the name.
    let name = name
        .rsplit_once(" IId-")
        .map_or(name.as_str(), |(head, _)| head)
        .trim()
        .to_string();
    if name.is_empty() {
        return None;
    }
    Some((fund.to_string(), ali, name))
}

/// The amounts on one line, in order, with `---` carried as a stated absence.
///
/// # Why `---` is a value rather than a gap
///
/// Because it is the difference between an extraction that aligns and one that does not. Measured
/// across five editions spanning the whole series, 566 of 566 line items have exactly as many
/// amount tokens as year headings **once `---` counts as a token**; without it, seven items in the
/// 2006 edition come up short and every year after the gap is misdated by one column.
///
/// That is the failure the greenbook attempt was reverted for, arriving from the other direction:
/// there an unfunded year left an empty column, here the publisher prints a mark for it. A mark
/// that is read is a column that stays aligned.
fn catalog_amounts(line: &str) -> Vec<Option<f64>> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find(['$', '-']) {
        let tail = &rest[at..];
        if tail.starts_with("--") {
            out.push(None);
            let end = tail.find(|c: char| c != '-').unwrap_or(tail.len());
            rest = &tail[end..];
        } else {
            let after = &tail[1..];
            let end = after
                .find(|c: char| !c.is_ascii_digit() && c != ',')
                .unwrap_or(after.len());
            let digits: String = after[..end].chars().filter(|c| *c != ',').collect();
            if digits.is_empty() {
                rest = after;
                continue;
            }
            out.push(digits.parse().ok());
            rest = &after[end..];
        }
    }
    out
}

/// The column kinds on a labels row, in order.
///
/// # The distinction this exists to preserve
///
/// Editions come in two shapes, and the difference is not cosmetic. An edition published at the
/// start of a biennium carries four actuals and **two `Appropriation` columns** — the enacted
/// amounts for the biennium just begun. An edition published a year later carries five actuals and
/// **one `Adj. Approp.`**: the biennium's first year has closed into an actual, and the year that
/// remains is stated as *adjusted*, not as enacted.
///
/// That is the same trap that reverted the workbook attempt on the greenbook series, where the
/// with-actuals variant silently superseded the enacted column and the extraction produced zero
/// appropriation rows for a whole biennium without failing. Here the publisher labels it, so the
/// only way to reproduce that failure is to throw the label away. These are therefore three kinds
/// and never two, and `adjusted` is never written as `appropriation`.
fn catalog_labels(line: &str) -> Vec<&'static str> {
    // Longest first, so `Adj. Approp.` is not read as a bare `Approp.`.
    const KINDS: [(&str, &str); 6] = [
        ("Adjusted Approp.", "adjusted"),
        ("Adj. Approp.", "adjusted"),
        ("Appropriation", "appropriation"),
        ("Approp.", "appropriation"),
        ("Estimate", "estimate"),
        ("Actual", "actual"),
    ];
    let mut out = Vec::new();
    let mut at = 0;
    while at < line.len() {
        let rest = &line[at..];
        if let Some((token, kind)) = KINDS
            .iter()
            .find(|(token, _)| rest.starts_with(token))
            .copied()
        {
            out.push(kind);
            at += token.len();
        } else {
            at += rest.chars().next().map_or(1, char::len_utf8);
        }
    }
    out
}

/// Every line item in one edition of the Catalog's education volume.
///
/// # How a block is found
///
/// By the **labels** row — the one reading `Actual Actual Actual Actual Appropriation
/// Appropriation` — rather than by the year row or the heading. The year row is printed `2002` in
/// the 2006 and 2009 editions and `FY 2022` from later ones, and a bare four-digit number is not a
/// distinctive thing to search a budget document for. The labels row is, and it fixes the year row
/// as the line above it and the amounts as the next line carrying figures.
///
/// # Why the years are paired ordinally rather than by column position
///
/// The greenbook attempt failed reading columns by nearest header label, because the labels are
/// narrower than the figures beneath them. The lesson recorded there was to calibrate from the
/// amounts. Here that reduces to something simpler and stronger: when the count of years, the
/// count of labels and the count of amounts all agree, left-to-right order is the pairing, and no
/// position arithmetic is involved at all.
///
/// So the guard is the count, and a block whose three counts disagree is **refused** rather than
/// guessed at. Across the sampled editions nothing is refused, which is the point — the rule costs
/// nothing when the document is well formed and it is the only thing standing between a misread
/// page and a plausible figure in the wrong year.
///
/// # Errors
///
/// If no line item parses at all, which means the edition's layout is one this has not seen.
pub fn catalog_items(text: &str) -> Result<(Vec<CatalogItem>, Vec<String>), String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<CatalogItem> = Vec::new();
    let mut refused: Vec<String> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let labels = catalog_labels(line);
        if labels.len() < 4 || i == 0 {
            continue;
        }

        let years: Vec<u16> = {
            let mut found = Vec::new();
            let row = lines[i - 1];
            let bytes = row.as_bytes();
            let mut k = 0;
            while k < bytes.len() {
                if bytes[k].is_ascii_digit() {
                    let start = k;
                    while k < bytes.len() && bytes[k].is_ascii_digit() {
                        k += 1;
                    }
                    if k - start == 4 {
                        if let Ok(y) = row[start..k].parse::<u16>() {
                            if (1990..2100).contains(&y) {
                                found.push(y);
                            }
                        }
                    }
                } else {
                    k += 1;
                }
            }
            found
        };
        if years.is_empty() {
            continue;
        }

        let Some((amounts, _)) = lines[i + 1..lines.len().min(i + 4)]
            .iter()
            .enumerate()
            .map(|(offset, l)| (catalog_amounts(l), offset))
            .find(|(a, _)| !a.is_empty())
        else {
            continue;
        };

        // The heading is the nearest line above the year row that parses as one.
        let Some((fund, ali, name)) = (i.saturating_sub(6)..i - 1)
            .rev()
            .find_map(|k| catalog_heading(lines[k]))
        else {
            continue;
        };

        if years.len() != labels.len() || years.len() != amounts.len() {
            refused.push(format!(
                "{fund} {ali} {name}: {} years, {} labels, {} amounts",
                years.len(),
                labels.len(),
                amounts.len()
            ));
            continue;
        }

        // `Legal Basis:` runs to the next labelled field, wrapping across lines.
        let mut legal_basis = String::new();
        for l in lines[i..lines.len().min(i + 30)].iter() {
            if let Some(rest) = l.trim_start().strip_prefix("Legal Basis:") {
                legal_basis = rest.trim().to_string();
                continue;
            }
            if !legal_basis.is_empty() {
                let t = l.trim();
                if t.is_empty() || t.contains(':') {
                    break;
                }
                legal_basis.push(' ');
                legal_basis.push_str(t);
            }
        }

        out.push(CatalogItem {
            fund,
            ali,
            name,
            legal_basis: legal_basis.split_whitespace().collect::<Vec<_>>().join(" "),
            years: years
                .into_iter()
                .zip(labels)
                .zip(amounts)
                .map(|((y, kind), amount)| (y, kind.to_string(), amount))
                .collect(),
        });
    }

    if out.is_empty() {
        return Err("no line items parsed; the edition's layout is unrecognised".to_string());
    }
    Ok((out, refused))
}
