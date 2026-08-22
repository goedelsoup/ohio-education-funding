//! The appropriation tables printed in the acts themselves, FY1998 to FY2001.
//!
//! Where the workbook series does not reach. These are printed columns in a legislative document
//! rather than a data file, so the reader finds the column boundaries from the numbers' own
//! alignment, and then reconciles each group against the total the act prints for it — which is
//! the only check available on a column read from the wrong offset.
//!
//! Two footing defects in the acts as printed are recorded rather than corrected: the act says
//! what it says, and a total that does not foot is a fact about the act.

use super::format::clean_name;

/// Columns of the session-law extract.
pub const SESSION_LAW_HEADER: &[&str] = &[
    "general_assembly",
    "bill",
    "fiscal_year",
    "fund_group",
    "fund",
    "line_item",
    "title",
    "amount",
];

/// One enrolled act, with the two fiscal years its money columns stand for.
#[derive(Debug, Clone, Copy)]
pub struct ActText<'a> {
    /// The General Assembly that passed it.
    pub general_assembly: u16,
    /// The bill, as the registry keys it: `hb215`.
    pub bill: &'a str,
    /// The heading that opens the Department of Education appropriation table.
    pub heading: &'a str,
    /// The fiscal year the first money column stands for. The second is the year after.
    pub first_year: u16,
    /// `pdftotext -layout` output for the whole act.
    pub text: &'a str,
}

/// The fund-group totals an act prints, which are what the parsed rows must reproduce.
struct GroupTotal {
    label: String,
    first: i64,
    second: i64,
}

/// Every Department of Education appropriation line in the acts this repository holds.
///
/// # Why the acts and not an analysis of them
///
/// Everything else in the appropriation series comes from the Legislative Service Commission —
/// greenbooks, budget workbooks, the Catalog of Budget Line Items — and all three stop at FY2002
/// or later. The acts stop where the legislature's own archive stops, which is the 122nd General
/// Assembly, and that is four fiscal years earlier.
///
/// It is also a different kind of document. A greenbook is LSC describing what an act did; this is
/// the act. Where they can be compared they agree, and where they cannot the act is the thing that
/// was voted on.
///
/// # The reconciliation is the whole quality bar
///
/// Each fund group's table ends with a printed total, and the act closes with a grand total across
/// groups. Every parsed row is summed back against them, in both fiscal-year columns, and a
/// mismatch of one dollar fails the rebuild. That check is not decoration: it is the only thing
/// standing between this and a plausible table with a row missing, which is a failure this
/// repository has shipped before and could not see afterwards.
///
/// # What the FY1999 column means, and why no reader should average it
///
/// [`crate::registry`]'s note on `hb215-122-enrolled` has the detail. In short: fifty-one of
/// H.B. 215's fifty-three GRF education lines are appropriated **zero** for FY1999, and the whole
/// year sits in one line, `200-405`. That is not a defect in this reader and it is not a cut. Nine
/// months after the Supreme Court held the funding system unconstitutional, the General Assembly
/// passed a budget that declined to itemise the second year, and said so in the act:
///
/// > By January 15, 1998, the General Assembly shall develop a plan to provide itemized
/// > appropriations for the Department of Education for fiscal year 1999.
///
/// The rows are emitted as the act states them, zeros included, and the consumer is expected to
/// read `line_item` `200405` as the marker it is. See `project::session_laws`.
///
/// # Errors
///
/// Returns a description if an act's education heading is absent, if a fund group's rows do not
/// sum to its printed total, or if the group totals do not sum to the printed grand total.
pub fn build_session_laws(acts: &[ActText<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for act in acts {
        let label = format!(
            "{} of the {}th General Assembly",
            act.bill, act.general_assembly
        );
        let start = act
            .text
            .find(act.heading)
            .ok_or_else(|| format!("{label} has no {:?} heading", act.heading))?;
        // The table ends at its own grand total; everything after is the earmark prose.
        let rest = &act.text[start..];
        let end = rest
            .find("TOTAL ALL BUDGET FUND GROUPS")
            .ok_or_else(|| format!("{label} prints no grand total to check against"))?;
        let table = &rest[..end];
        let closing = &rest[end..];

        let (rows, totals) = session_law_rows(table, act)?;
        reconcile_act(&rows, &totals, closing, &label, act)?;
        out.extend(rows.into_iter().flat_map(|row| row.into_fields(act)));
    }
    Ok(out)
}

/// One appropriation line in one act, before it becomes two fixture rows.
struct ActLine {
    group: String,
    fund: String,
    item: String,
    title: String,
    first: i64,
    second: i64,
}

impl ActLine {
    /// One fixture row per fiscal year, because every consumer of the appropriation series reads
    /// a row as one year's claim about one line.
    fn into_fields(self, act: &ActText<'_>) -> [Vec<String>; 2] {
        let row = |year: u16, amount: i64| {
            vec![
                act.general_assembly.to_string(),
                act.bill.to_string(),
                year.to_string(),
                self.group.clone(),
                self.fund.clone(),
                self.item.clone(),
                self.title.clone(),
                amount.to_string(),
            ]
        };
        [
            row(act.first_year, self.first),
            row(act.first_year + 1, self.second),
        ]
    }
}

/// Read the rows and the printed fund-group totals out of one act's education table.
///
/// The layout is fixed and has been since at least 1997: a fund code, the hyphenated line item, a
/// title that may wrap onto the following lines, and two right-aligned dollar amounts. A wrapped
/// title carries no amounts, which is how the continuation is recognised without measuring
/// columns.
fn session_law_rows(
    table: &str,
    act: &ActText<'_>,
) -> Result<(Vec<ActLine>, Vec<GroupTotal>), String> {
    let mut rows: Vec<ActLine> = Vec::new();
    let mut totals: Vec<GroupTotal> = Vec::new();
    let mut group = String::new();
    let mut pending: Option<String> = None;
    // Where the last row or total put its two amounts, so a replacement printed beneath it can be
    // assigned to the year it replaces. `None` until the first row of the table.
    let mut last: Option<Columns> = None;
    // Whether the open thing is a row or a total, because an amending act corrects both.
    let mut last_was_total = false;

    for line in table.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // An amendment binds to whatever it sits under, and it may sit two lines under: the
        // amended figure for `200-545 Vocational Education Enhancements` follows the wrapped
        // second half of its own title. So this is tried before the row and title cases and the
        // open row stays open across a wrap.
        if let Some(columns) = last {
            if let Some([first, second]) = amendment(line, columns) {
                if last_was_total {
                    if let Some(total) = totals.last_mut() {
                        total.first = first.unwrap_or(total.first);
                        total.second = second.unwrap_or(total.second);
                    }
                } else if let Some(row) = rows.last_mut() {
                    row.first = first.unwrap_or(row.first);
                    row.second = second.unwrap_or(row.second);
                }
                continue;
            }
        }
        // A total's label wraps as often as not — `TOTAL GSF General Services` on one line and
        // `Fund Group   $ 12,695,447 $ 13,033,594` on the next — so a `TOTAL` with no money on it
        // opens a label that the following line closes.
        if let Some(total) = printed_total(trimmed) {
            pending = None;
            last = columns_of(line);
            last_was_total = true;
            totals.push(total);
            continue;
        }
        if trimmed.len() > 6 && trimmed[..6].eq_ignore_ascii_case("total ") {
            pending = Some(trimmed.to_string());
            continue;
        }
        // The wrap can fall mid-word — `TOTAL LPE Lottery Pr` / `ofits Education` / `Fund Group
        // $ 699,892,200 ...` — so the label accumulates until a line carries the amounts rather
        // than for a fixed number of lines.
        if let Some(head) = pending.as_mut() {
            head.push(' ');
            head.push_str(trimmed);
            if let Some(total) = printed_total(head) {
                last = columns_of(line);
                last_was_total = true;
                totals.push(total);
                pending = None;
            }
            continue;
        }
        match appropriation_row(trimmed) {
            Some((fund, item, title, first, second)) => {
                last = columns_of(line);
                last_was_total = false;
                rows.push(ActLine {
                    group: group.clone(),
                    fund,
                    item,
                    title,
                    first,
                    second,
                });
            }
            None => {
                // A line with no amounts is either a fund-group heading or a wrapped title. The
                // difference is whether a row is open: the heading always precedes the first row
                // of its group.
                if trimmed.ends_with("Fund Group") || trimmed.ends_with("Fund") {
                    group = trimmed.to_string();
                } else if let Some(last) = rows.last_mut() {
                    if !trimmed.starts_with("H. B. No.")
                        && !trimmed.chars().all(|c| c.is_ascii_digit())
                    {
                        last.title.push(' ');
                        last.title.push_str(trimmed);
                    }
                }
            }
        }
    }

    if rows.is_empty() {
        return Err(format!(
            "{} of the {}th yielded no appropriation rows",
            act.bill, act.general_assembly
        ));
    }
    Ok((rows, totals))
}

/// Where a line's two money columns end, so an amendment beneath it can be assigned to one.
///
/// By position because that is the only thing that says which column an amending act is changing:
/// H.B. 770 prints the amended figure on its own line under the column it replaces, and a line
/// carrying one number is otherwise silent about which of the two years it belongs to.
#[derive(Debug, Clone, Copy)]
struct Columns {
    first: usize,
    second: usize,
}

/// `GRF 200-501 School Foundation Basic   $  2,202,851,688 $  0` — or `None` if the line is not one.
fn appropriation_row(line: &str) -> Option<(String, String, String, i64, i64)> {
    let mut parts = line.split_whitespace();
    let fund = parts.next()?.to_string();
    // Fund codes are three characters: `GRF`, `017`, `4D1`, `3L6`.
    if fund.len() != 3 || !fund.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let item = parts.next()?;
    let (head, tail) = item.split_once('-')?;
    if head.len() != 3 || tail.len() != 3 || !item.chars().all(|c| c.is_ascii_digit() || c == '-') {
        return None;
    }
    let rest: Vec<&str> = parts.collect();
    // Two dollar signs close the row and everything between the item and the first is the title.
    let mut money = rest.iter().enumerate().filter(|(_, w)| **w == "$");
    let (first_at, _) = money.next()?;
    let (second_at, _) = money.next()?;
    let amount =
        |at: usize| -> Option<i64> { rest.get(at + 1)?.replace(',', "").parse::<i64>().ok() };
    Some((
        fund,
        item.replace('-', ""),
        clean_name(&rest[..first_at].join(" ")),
        amount(first_at)?,
        amount(second_at)?,
    ))
}

/// `TOTAL GRF General Revenue Fund   $ 4,899,708,534 $ 5,134,145,592`, or `None`.
/// The end positions of the last two comma-grouped numbers on a line.
fn columns_of(line: &str) -> Option<Columns> {
    let ends: Vec<usize> = number_spans(line)
        .into_iter()
        .map(|(_, end, _)| end)
        .collect();
    match ends[..] {
        [.., first, second] => Some(Columns { first, second }),
        _ => None,
    }
}

/// Every run of digits and commas on a line, with where it starts and ends.
fn number_spans(line: &str) -> Vec<(usize, usize, i64)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut at = 0usize;
    while at < bytes.len() {
        if !bytes[at].is_ascii_digit() {
            at += 1;
            continue;
        }
        let start = at;
        while at < bytes.len() && (bytes[at].is_ascii_digit() || bytes[at] == b',') {
            at += 1;
        }
        // A trailing comma belongs to the prose, not to the number.
        let end = line[start..at].trim_end_matches(',').len() + start;
        if let Ok(value) = line[start..end].replace(',', "").parse::<i64>() {
            out.push((start, end, value));
        }
    }
    out
}

/// An amending act's replacement figures, aligned under the columns they replace.
///
/// `None` unless the line is nothing but numbers **and** at least one of them ends where one of
/// the row above's amounts ends. That second condition is what keeps page furniture out: the
/// enrolled acts print a bare page number on its own line, and `247` under a row whose columns
/// end at 59 and 74 is not an amendment to anything.
fn amendment(line: &str, columns: Columns) -> Option<[Option<i64>; 2]> {
    if line.trim().is_empty() || line.chars().any(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let mut out = [None, None];
    let mut aligned = false;
    for (_, end, value) in number_spans(line) {
        // Two characters of slack: the columns are right-aligned but the amending line is set
        // separately and the two do not always land on exactly the same byte.
        if end.abs_diff(columns.first) <= 2 {
            out[0] = Some(value);
            aligned = true;
        } else if end.abs_diff(columns.second) <= 2 {
            out[1] = Some(value);
            aligned = true;
        }
    }
    aligned.then_some(out)
}

/// The prefix is matched case-insensitively because one group's is not like the others: H.B. 215
/// closes the lottery funds with `Total 017 and 018 LPE Lottery Profits Education Fund Group`,
/// lowercase and named by fund number rather than by group. It carries $699,892,200, and the
/// reconciliation is what noticed it was missing.
fn printed_total(line: &str) -> Option<GroupTotal> {
    if line.len() <= 6 || !line[..6].eq_ignore_ascii_case("total ") {
        return None;
    }
    let rest = &line[6..];
    let words: Vec<&str> = rest.split_whitespace().collect();
    let mut money = words.iter().enumerate().filter(|(_, w)| **w == "$");
    let (first_at, _) = money.next()?;
    let (second_at, _) = money.next()?;
    let amount =
        |at: usize| -> Option<i64> { words.get(at + 1)?.replace(',', "").parse::<i64>().ok() };
    Some(GroupTotal {
        label: words[..first_at].join(" "),
        first: amount(first_at)?,
        second: amount(second_at)?,
    })
}

/// Discrepancies in an act's own arithmetic, each one named and its size fixed.
///
/// Not a tolerance. A tolerance would let any small error through and would grow silently; this
/// is a list of specific known defects, and a difference that is not on it — or is on it at a
/// different size — still fails the rebuild.
///
/// Each entry is the amount by which the act's own rows **exceed** what it foots them to, and
/// `at_grand_total` says which of the two additions is wrong: the rows against their fund-group
/// totals, or those totals against the grand total. An act is wrong in one place and right in the
/// other, so the allowance has to be aimed.
///
/// **H.B. 282, FY2001.** Its printed GRF total is one dollar *more* than its own fifty-two rows,
/// while its five group totals sum to its grand total exactly. The dollar sits between the rows
/// and the GRF footing and there is nothing to correct it against. Carried rather than absorbed
/// because $1 on $7.98 billion is exactly what a tolerance wide enough to hide a missing row
/// would swallow.
///
/// **H.B. 770, FY1999.** Its fund-group totals sum to $1,443,401 *more* than its printed grand
/// total, and this one is resolvable in the rows' favour: the missing amount is the Education
/// Improvement Fund's `006 200-689 Hazardous Waste Removal`, which the act's FY1998 grand total
/// includes and its FY1999 grand total does not. `appropriation-lines.csv` already carries that
/// line as a FY1999 **actual** of exactly $1,443,401, from the H.B. 94 greenbook — so the money
/// was appropriated and spent, and it is the act's footing that is wrong rather than its rows.
const ACT_FOOTING_DEFECTS: [(&str, u16, i64, bool); 2] =
    [("hb282", 2001, -1, false), ("hb770", 1999, 1_443_401, true)];

/// Sum the rows back against every total the act prints, in both columns.
fn reconcile_act(
    rows: &[ActLine],
    totals: &[GroupTotal],
    closing: &str,
    label: &str,
    act: &ActText<'_>,
) -> Result<(), String> {
    if totals.is_empty() {
        return Err(format!("{label} prints no fund-group totals"));
    }
    let summed = |k: usize| -> i64 {
        rows.iter()
            .map(|r| if k == 0 { r.first } else { r.second })
            .sum()
    };
    let printed = |k: usize| -> i64 {
        totals
            .iter()
            .map(|t| if k == 0 { t.first } else { t.second })
            .sum()
    };
    let allowed = |year: u16, at_grand_total: bool| -> i64 {
        ACT_FOOTING_DEFECTS
            .iter()
            .find(|(bill, defect_year, _, at)| {
                *bill == act.bill && *defect_year == year && *at == at_grand_total
            })
            .map_or(0, |(_, _, by, _)| *by)
    };
    for (k, year) in [(0usize, act.first_year), (1, act.first_year + 1)] {
        if summed(k) - printed(k) != allowed(year, false) {
            let names: Vec<&str> = totals.iter().map(|t| t.label.as_str()).collect();
            return Err(format!(
                "{label}: FY{year} rows sum to {} against the {} its fund-group totals print \
                 ({names:?}); a row is missing or double-counted",
                summed(k),
                printed(k)
            ));
        }
    }
    // And the groups against the act's own grand total, which is a separate assertion: the groups
    // can each be internally consistent and still not be all of them.
    // The grand total wraps like any other and is amended like any other: H.B. 770 prints its
    // own replacement beneath it, and reading only the first line gets the figure the act is
    // superseding rather than the one it enacts.
    let mut head = String::new();
    let mut grand = None;
    let mut at = 0usize;
    for (i, line) in closing.lines().take(4).enumerate() {
        head.push(' ');
        head.push_str(line.trim());
        if let Some(total) = printed_total(head.trim()) {
            grand = Some((total, columns_of(line)));
            at = i;
            break;
        }
    }
    let (mut grand, columns) =
        grand.ok_or_else(|| format!("{label}: the grand total does not parse"))?;
    if let (Some(columns), Some(next)) = (columns, closing.lines().nth(at + 1)) {
        if let Some([first, second]) = amendment(next, columns) {
            grand.first = first.unwrap_or(grand.first);
            grand.second = second.unwrap_or(grand.second);
        }
    }
    for (k, year, stated) in [
        (0usize, act.first_year, grand.first),
        (1, act.first_year + 1, grand.second),
    ] {
        if printed(k) - stated != allowed(year, true) {
            return Err(format!(
                "{label}: FY{year} fund-group totals sum to {} against the grand total's {stated}",
                printed(k)
            ));
        }
    }
    Ok(())
}
