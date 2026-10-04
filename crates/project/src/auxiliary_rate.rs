//! The auxiliary services rate for every year it can be computed, against every rate LSC printed.
//!
//! R.C. 3317.024(E)(2)(d) sets the per-pupil amount as the appropriation for sections 3317.06 and
//! 3317.062 over the chartered nonpublic schools' in-state October membership.
//! [`crate::ledger::nonpublic_support::auxiliary_rate_fy2025`] reproduces it for one year; this
//! module does it for FY2008 through FY2026, with the numerator read off each greenbook's earmark
//! table by [`remainders`] rather than subtracted, and [`published_rates`] holding every per-pupil
//! figure LSC has printed. Pairing the two is what `the_rate_the_greenbooks_published` does: five
//! of seven close, and the two that do not predate any exact October.
//!
//! It sits outside [`mod@crate::ledger`] because it reads [`mod@crate::greenbook`], and the
//! ledger reaches nothing outside itself.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use deflator::CpiSeries;
use edfund_core::FiscalYear;

use crate::ledger::budget_analysis;
use crate::ledger::nonpublic_enrolment;

/// The line [`remainders`] reads, as every greenbook since H.B. 119 prints it.
const REMAINDER_LABEL: &str = "Remainder – Auxiliary Services";

/// The H.B. 96 greenbook's record id, which its fixture does not carry because it is not a record
/// file — see [`budget_analysis::GREENBOOK`].
const HB96_GREENBOOK: &str = "hb96-greenbook";

/// The H.B. 96 redbook's, likewise.
const HB96_REDBOOK: &str = "hb96-redbook";

/// What one greenbook's earmark table leaves of [`AUXILIARY_SERVICES`](crate::ledger::nonpublic_support::AUXILIARY_SERVICES) for the quotient.
///
/// R.C. 3317.024(E)(2)(d) divides "the total amount appropriated for the implementation of
/// sections 3317.06 and 3317.062", and every enacted budget since H.B. 66 has earmarked part of
/// the line for something that is neither — the postsecondary enrollment options programme for
/// nonpublic pupils, which College Credit Plus replaced from FY2016. LSC prints the rest as its
/// own row, so the numerator is read rather than subtracted.
#[derive(Debug, Clone, PartialEq)]
pub struct Remainder {
    /// The greenbook, as `hb64-greenbook`.
    pub edition: &'static str,
    /// The fiscal year.
    pub fiscal_year: u16,
    /// `actual` where the table's column says so, `appropriation` otherwise.
    pub kind: &'static str,
    /// The amount left for the quotient, in the dollars of the year.
    pub amount: f64,
}

/// Every remainder row of every enacted budget's earmark table, oldest edition first.
///
/// Read from the greenbooks only. The redbook prints the same table over the executive proposal,
/// which is not what was appropriated. H.B. 66's greenbook states its $2,000,000 earmark in prose
/// and prints no table, so the series this reads begins at FY2008.
#[must_use]
pub fn remainders() -> Vec<Remainder> {
    static REMAINDERS: OnceLock<Vec<Remainder>> = OnceLock::new();
    REMAINDERS.get_or_init(read_remainders).clone()
}

/// The greenbooks, read.
///
/// `OnceLock` for the reason `project::panel`'s reader has one: the parse is pure and the files
/// are compiled in, so a second read could only reproduce the first.
fn read_remainders() -> Vec<Remainder> {
    crate::greenbook::greenbooks()
        .into_iter()
        .map(|book| (book.id, book.body))
        .chain([(HB96_GREENBOOK, budget_analysis::GREENBOOK)])
        .flat_map(|(edition, body)| remainder_rows(edition, body))
        .collect()
}

/// The remainder rows of one document, each paired with the fiscal years of the header above it.
///
/// The header is the nearest line above that names a fiscal year, and the row's amounts are its
/// *last* columns — H.B. 166 onward print a closed year to the left of the two appropriated ones.
/// A line between the two that says `Actual` marks the first of three columns as closed.
fn remainder_rows(edition: &'static str, body: &'static str) -> Vec<Remainder> {
    let lines: Vec<&str> = body.lines().collect();
    let mut out = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        if !line.trim_start().starts_with(REMAINDER_LABEL) || !line.contains('$') {
            continue;
        }
        let amounts = dollar_amounts(line);
        let above = &lines[at.saturating_sub(8)..at];
        let Some(header) = above.iter().rposition(|l| !fiscal_years(l).is_empty()) else {
            continue;
        };
        let years = fiscal_years(above[header]);
        let closed = above[header..].iter().any(|l| l.contains("Actual"));
        let Some(first) = years.len().checked_sub(amounts.len()) else {
            continue;
        };
        for (column, (fiscal_year, amount)) in years[first..].iter().zip(amounts).enumerate() {
            out.push(Remainder {
                edition,
                fiscal_year: *fiscal_year,
                kind: if closed && column == 0 {
                    "actual"
                } else {
                    "appropriation"
                },
                amount,
            });
        }
    }
    out
}

/// Every `$` figure on a line, read past the space some editions print after the sign.
fn dollar_amounts(line: &str) -> Vec<f64> {
    line.split('$')
        .skip(1)
        .filter_map(|cell| {
            let digits: String = cell
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == ',')
                .filter(char::is_ascii_digit)
                .collect();
            digits.parse().ok()
        })
        .collect()
}

/// Every `FY 2008` on a line, as `2008`.
fn fiscal_years(line: &str) -> Vec<u16> {
    line.split("FY ")
        .skip(1)
        .filter_map(|cell| cell.get(..4)?.parse().ok())
        .collect()
}

/// The newest greenbook's remainder for `fiscal_year`.
///
/// The same rule [`latest`](crate::ledger::nonpublic_support::latest) applies to the catalog, and for the same reason: a year is answered by
/// one edition. A closed year is answered by the edition that printed its actual where one did;
/// FY2015 and FY2017 never got one, because the greenbooks of H.B. 64 and H.B. 49 print no closed
/// column.
#[must_use]
pub fn remainder(fiscal_year: u16) -> Option<Remainder> {
    remainders()
        .into_iter()
        .rfind(|row| row.fiscal_year == fiscal_year)
}

/// A per-pupil auxiliary services rate as LSC states it in prose.
#[derive(Debug, Clone, PartialEq)]
pub struct PublishedRate {
    /// The analysis stating it, as `hb64-greenbook` or `hb96-redbook`.
    pub edition: &'static str,
    /// The fiscal year it is stated for.
    pub fiscal_year: u16,
    /// The rate, in whole dollars as printed.
    pub rate: f64,
    /// Whether LSC qualified it: H.B. 59's greenbook says FY2013's was "about $710", H.B. 166's
    /// that FY2019's "was roughly $900".
    pub roughly: bool,
}

impl PublishedRate {
    /// Whether `quotient` is the figure LSC would have printed.
    ///
    /// To the dollar, or to the ten dollars where LSC said "roughly" or "about". Rounding and not truncation:
    /// FY2023's quotient is $926.52 and LSC prints $927.
    #[must_use]
    pub fn reproduced_by(&self, quotient: f64) -> bool {
        let step = if self.roughly { 10.0 } else { 1.0 };
        ((quotient / step).round() * step - self.rate).abs() < 0.5
    }
}

/// Every per-pupil auxiliary services rate LSC states, oldest edition first.
///
/// A sentence opening "In FY" that states a dollar figure per pupil, where it or the sentence
/// before it is about auxiliary services. The second condition is what reaches H.B. 49's "In FY
/// 2017, the per-pupil amount was $867", which does not name the programme.
#[must_use]
pub fn published_rates() -> Vec<PublishedRate> {
    static PUBLISHED: OnceLock<Vec<PublishedRate>> = OnceLock::new();
    PUBLISHED.get_or_init(read_published_rates).clone()
}

/// The analyses, read.
///
/// `OnceLock` for the reason `project::panel`'s reader has one: the parse is pure and the files
/// are compiled in, so a second read could only reproduce the first.
fn read_published_rates() -> Vec<PublishedRate> {
    crate::greenbook::greenbooks()
        .into_iter()
        .map(|book| (book.id, book.body))
        .chain([
            (HB96_REDBOOK, budget_analysis::REDBOOK),
            (HB96_GREENBOOK, budget_analysis::GREENBOOK),
        ])
        .flat_map(|(edition, body)| {
            let flat = body.split_whitespace().collect::<Vec<_>>().join(" ");
            rate_sentences(&flat)
                .into_iter()
                .map(move |(fiscal_year, rate, roughly)| PublishedRate {
                    edition,
                    fiscal_year,
                    rate,
                    roughly,
                })
        })
        .collect()
}

/// `(fiscal year, rate, roughly)` for every rate sentence in one flattened analysis.
fn rate_sentences(flat: &str) -> Vec<(u16, f64, bool)> {
    let sentences: Vec<&str> = flat.split(". ").collect();
    let mut out = Vec::new();
    for (at, sentence) in sentences.iter().enumerate() {
        let Some(rest) = sentence.strip_prefix("In FY ") else {
            continue;
        };
        let lower = sentence.to_lowercase();
        if !(lower.contains("per pupil") || lower.contains("per-pupil")) {
            continue;
        }
        let previous = at.checked_sub(1).map_or("", |i| sentences[i]);
        if !(lower.contains("auxiliary") || previous.to_lowercase().contains("auxiliary")) {
            continue;
        }
        let (Some(fiscal_year), Some(rate)) = (
            rest.get(..4).and_then(|year| year.parse().ok()),
            dollar_amounts(sentence).first().copied(),
        ) else {
            continue;
        };
        let roughly = lower.contains("roughly") || lower.contains("about $");
        out.push((fiscal_year, rate, roughly));
    }
    out
}

/// One fiscal year's auxiliary services quotient, on every denominator the department published.
///
/// The numerator is [`remainder`]. The denominators are three because the department's files
/// give up to three answers for one October, and which one reproduces LSC is the finding rather
/// than an input — see `the_rate_the_greenbooks_published` for the year-by-year pairing.
#[derive(Debug, Clone, PartialEq)]
pub struct Rate {
    /// The fiscal year.
    pub fiscal_year: u16,
    /// The numerator: the line net of its earmark, from the newest greenbook that prints it.
    pub remainder: Remainder,
    /// Over the in-state total of the October's own annual file, where it states one.
    pub on_the_annual_file: Option<f64>,
    /// Over the in-state total of the 2014-2019 compilation, for the six Octobers it restates.
    pub on_the_compilation: Option<f64>,
    /// Over the masked kindergarten-through-twelve building sheet: `(lowest, highest)`, the
    /// remainder divided by the sheet's ceiling and by its floor.
    pub bound: Option<(f64, f64)>,
}

impl Rate {
    /// The one quotient a series of this rate should carry: the compilation where it restates the
    /// October, the annual file otherwise.
    ///
    /// The compilation first because it is the only one of the two that is right everywhere it
    /// can be checked. October 2015's annual file states a total below its own building sheet's
    /// floor; October 2016's states as in-state the figure the compilation states as the whole,
    /// and only the compilation's in-state count reproduces the $867 LSC published for FY2017.
    #[must_use]
    pub fn statutory(&self) -> Option<f64> {
        self.on_the_compilation.or(self.on_the_annual_file)
    }
}

/// The auxiliary services quotient for every fiscal year that has both a remainder and an October.
///
/// FY2008 through FY2026. Before FY2008 no greenbook prints the remainder; FY2027 divides by
/// October 2026.
#[must_use]
pub fn rates() -> Vec<Rate> {
    let years: BTreeSet<u16> = remainders().iter().map(|row| row.fiscal_year).collect();
    years
        .into_iter()
        .filter_map(|fiscal_year| {
            let remainder = remainder(fiscal_year)?;
            let annual = nonpublic_enrolment::october(fiscal_year)?;
            let over = |count: Option<f64>| count.map(|pupils| remainder.amount / pupils);
            Some(Rate {
                fiscal_year,
                on_the_annual_file: over(annual.published_in_state),
                on_the_compilation: over(
                    nonpublic_enrolment::restatement(fiscal_year)
                        .and_then(|restated| restated.published_in_state),
                ),
                bound: annual
                    .k12
                    .map(|k12| (remainder.amount / k12.ceiling, remainder.amount / k12.floor)),
                remainder,
            })
        })
        .collect()
}

/// How [`Rate::statutory`] moved from `from` to `to`, nominal and in `to`'s dollars, as fractions.
///
/// `None` where either year has no statutory quotient or the index cannot reach it.
#[must_use]
pub fn rate_growth(from: u16, to: u16) -> Option<(f64, f64)> {
    let all = rates();
    let quotient = |fiscal_year: u16| {
        all.iter()
            .find(|rate| rate.fiscal_year == fiscal_year)
            .and_then(Rate::statutory)
    };
    let (first, last) = (quotient(from)?, quotient(to)?);
    let restated = CpiSeries::cpi_u_june()
        .convert(first, FiscalYear(from), FiscalYear(to))
        .ok()?
        .value;
    Some((last / first - 1.0, last / restated - 1.0))
}
