//! The Jon Peterson Special Needs Scholarship annual report, one row per edition and one row per
//! category series.
//!
//! Three editions are read by this one function: the standalone FY2023 and FY2024 JPSN annual
//! reports, and the Jon Peterson section of the consolidated 2025 Scholarship Annual Report that
//! replaced them. They are structurally the same document — the same participation sentence, the
//! same award-category table, the same two pie charts — so reading them with one parser is what
//! makes the three years comparable, and what makes a fourth edition that changes shape fail here
//! rather than arrive quietly.
//!
//! # What is deliberately not extracted
//!
//! Each edition opens its participation section with a bar chart of total applications by fiscal
//! year, covering FY2014 onward, and a second of students by grade level. Neither carries a data
//! label in the text layer — only the axis gridlines do — so `pdftotext` reaches the axis and not
//! the bars. Those two series are the ones that would fill the FY2014-FY2023 participation hole
//! rather than bound it, and recovering them means reading the chart's geometry, which is a
//! different instrument from this one.

use super::text::{count_before, flatten};

/// One row per edition: participation, reach, and the only expenditure total the department ever
/// states in words.
pub const JPSN_HEADER: &[&str] = &[
    "fiscal_year",
    "students",
    "providers",
    "districts",
    "stated_total",
    "stated_total_fiscal_year",
];

/// One row per edition per series per position. Six of each series, in every edition so far.
pub const JPSN_CATEGORY_HEADER: &[&str] = &["fiscal_year", "series", "position", "value"];

/// The six funding levels R.C. 3317.022 organizes the thirteen IDEA disability categories into.
const CATEGORIES: usize = 6;

/// What one edition of the report yields: its summary row, and its per-category rows.
#[derive(Debug)]
pub struct Edition {
    /// One row, in [`JPSN_HEADER`]'s columns.
    pub summary: Vec<String>,
    /// Eighteen rows, in [`JPSN_CATEGORY_HEADER`]'s columns.
    pub categories: Vec<Vec<String>>,
}

/// The tokens of `flat` from the first occurrence of `from` up to the next `to` after it.
fn between<'a>(flat: &'a str, from: &str, to: &str) -> Result<&'a str, String> {
    let at = flat
        .find(from)
        .ok_or_else(|| format!("no `{from}` in the report"))?
        + from.len();
    let rest = &flat[at..];
    let end = rest
        .find(to)
        .ok_or_else(|| format!("no `{to}` after `{from}`"))?;
    Ok(&rest[..end])
}

/// A figure as the report prints it, with the currency, grouping and percent marks taken off and
/// every digit it published kept.
///
/// The precision is part of the quotation: the FY2023 edition gives its disability shares to one
/// decimal place and both later editions give two. Normalizing them to a common width would assert
/// a precision the FY2023 edition does not publish.
fn figure(token: &str) -> Result<String, String> {
    // The stop at the end of "sum to $81,773,133.70." survives the filter and would fail the
    // parse, so it is taken off after it rather than guessed at before.
    let bare = token
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .trim_end_matches('.')
        .to_string();
    if bare.is_empty() || bare.parse::<f64>().is_err() {
        return Err(format!("`{token}` is not a figure"));
    }
    Ok(bare)
}

/// The fiscal year the report states, from the heading that gives its July-to-June span.
///
/// # Why the span and not the cover
///
/// Every edition puts its year on the cover and in a sentence — "for fiscal year 2024" — but only
/// the award-category heading says which twelve months that is, and the consolidated edition has
/// no cover sentence at all. Requiring the span also makes the label check itself: the year after
/// "June 30" has to be the year in "FY", so a heading carried forward from the previous edition
/// without being updated is a parse failure rather than a year that is quietly wrong.
///
/// # Errors
///
/// If no such heading exists, or if two of them disagree.
fn fiscal_year(flat: &str) -> Result<u16, String> {
    let mut found: Vec<u16> = Vec::new();
    for (at, _) in flat.match_indices("FY ") {
        let rest = &flat[at + "FY ".len()..];
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if digits.len() != 4 {
            continue;
        }
        let Ok(year) = digits.parse::<u16>() else {
            continue;
        };
        let tail = rest[digits.len()..].trim_start();
        if !tail.starts_with('(') {
            continue;
        }
        let Some(close) = tail.find(')') else {
            continue;
        };
        let span = tail[1..close].to_uppercase();
        let Some(ends) = span.split("JUNE 30,").nth(1) else {
            continue;
        };
        let closing: String = ends
            .trim_start()
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if !span.contains("JULY 1,") || closing != digits {
            return Err(format!(
                "the heading `FY {year} ({})` does not close in its own fiscal year",
                tail[1..close].trim()
            ));
        }
        found.push(year);
    }
    found.sort_unstable();
    found.dedup();
    match found.as_slice() {
        [] => Err("no `FY <year> (July 1 - June 30)` heading in the report".to_string()),
        [only] => Ok(*only),
        many => Err(format!("the report states {many:?} as its fiscal year")),
    }
}

/// The school year a fiscal year covers, as the report writes it.
fn school_year(fiscal_year: u16) -> String {
    format!("{}-{}", fiscal_year - 1, fiscal_year)
}

/// The fiscal year a `2021-2022` school-year label names.
fn fiscal_year_of(label: &str) -> Result<u16, String> {
    let (first, second) = label
        .split_once('-')
        .ok_or_else(|| format!("`{label}` is not a school year"))?;
    let (first, second) = (
        first
            .parse::<u16>()
            .map_err(|_| format!("`{label}` is not a school year"))?,
        second
            .parse::<u16>()
            .map_err(|_| format!("`{label}` is not a school year"))?,
    );
    if second != first + 1 {
        return Err(format!("`{label}` spans {} years, not one", second - first));
    }
    Ok(second)
}

/// The six per-year maxima, each against the category number the table prints beside it.
///
/// # Why the printed number and not the order
///
/// Because the pie charts below it cannot be read that way, and the difference is the point. This
/// is a table with a `#` column, so every amount has its category stated; the charts put their
/// labels wherever the slice falls, and [`chart`] records a position rather than a category for
/// exactly that reason.
fn maxima(flat: &str) -> Result<Vec<(u8, String)>, String> {
    let window = between(flat, "Amount per Year", "*Category")?;
    let mut out: Vec<(u8, String)> = Vec::new();
    let mut number: Option<u8> = None;
    for token in window.split_whitespace() {
        if let Ok(n) = token.parse::<u8>() {
            if (1..=CATEGORIES as u8).contains(&n) {
                number = Some(n);
            }
        } else if token.starts_with('$') {
            let n = number
                .take()
                .ok_or_else(|| format!("the maximum `{token}` names no category"))?;
            out.push((n, figure(token)?));
        }
    }
    let numbered: Vec<u8> = out.iter().map(|(n, _)| *n).collect();
    if numbered != (1..=CATEGORIES as u8).collect::<Vec<u8>>() {
        return Err(format!(
            "the award-category table numbers its rows {numbered:?}, not 1 through {CATEGORIES}"
        ));
    }
    Ok(out)
}

/// The six labels of one pie chart, in the order the text layer gives them.
///
/// # Why a position and not a category
///
/// A chart's labels are laid out around the slices, and `pdftotext` reads them in that layout
/// order. Matching the disability shares by magnitude across the three editions shows the order
/// moving: the smallest slice, near 0.7% in every edition, is read first in FY2023 and fourth in
/// both later ones. A fixed category cannot change position, so the position is not the category,
/// and this reader does not claim it is. What the order does support is a sum, which does not
/// depend on it, and the largest value, which is identifiable by size.
///
/// The spending chart's order happens to be stable across all three editions. That is not a reason
/// to trust it: it is the same chart type laid out by the same tool, and the shares prove the tool
/// does not preserve category order.
fn chart(flat: &str, heading: &str, labelled: fn(&str) -> bool) -> Result<Vec<String>, String> {
    let window = between(flat, heading, "Category 1")?;
    let values: Vec<&str> = window
        .split_whitespace()
        .filter(|token| labelled(token))
        .collect();
    if values.len() != CATEGORIES {
        return Err(format!(
            "`{heading}` carries {} labels, not {CATEGORIES}",
            values.len()
        ));
    }
    values.iter().map(|t| figure(t)).collect()
}

/// The integer that opens `rest`, written with thousands separators.
fn count_after(rest: &str) -> Option<f64> {
    let digits: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .filter(|c| *c != ',')
        .collect();
    digits.parse().ok()
}

/// One edition of the report, as its summary row and its eighteen category rows.
///
/// # How the section is found
///
/// By the participation sentence — "N students received services from M approved providers
/// through the Jon Peterson Special Needs Scholarship" — which every edition carries once and
/// which the consolidated edition carries once among five programmes. Everything but the fiscal
/// year is read from that sentence forward, so the consolidated edition's other four programmes
/// cannot contribute a figure. The fiscal year is read from the whole document because the
/// consolidated edition prints its award-category heading *above* the sentence.
///
/// # What is cross-checked rather than trusted
///
/// The school year in the participation sentence has to be the fiscal year's, so an edition that
/// updates one label and not the other fails. The provider count appears twice, in the sentence
/// and in the providers section, and both have to agree. The district count appears two or three
/// times and all of them have to agree. The disability shares have to sum to 100%.
///
/// The one figure that is **not** made to agree is the expenditure total. The FY2023 edition
/// states one in words and attributes it to a school year that is not its own; the two later
/// editions state none at all. The stated total and the year it names are carried as published, in
/// their own columns, because the disagreement is the finding.
///
/// # Errors
///
/// If any of the above fails, which means this edition is not the document this reader was written
/// against and every figure it would have produced should be treated as unread.
pub fn jpsn_edition(text: &str) -> Result<Edition, String> {
    const PARTICIPATION: &str = "students received services from ";
    let flat = flatten(text);
    let fiscal_year = fiscal_year(&flat)?;

    let starts: Vec<usize> = flat
        .match_indices(PARTICIPATION)
        .map(|(at, _)| at)
        .filter(|at| {
            let after = &flat[*at..];
            after[..after.len().min(160)].contains("Jon Peterson Special Needs Scholarship")
        })
        .collect();
    let [at] = starts.as_slice() else {
        return Err(format!(
            "{} Jon Peterson participation sentences in the report, not one",
            starts.len()
        ));
    };
    let window = &flat[*at..];

    let students = count_before(&flat, *at)
        .ok_or_else(|| "the participation sentence states no student count".to_string())?;
    let providers = count_after(&window[PARTICIPATION.len()..])
        .ok_or_else(|| "the participation sentence states no provider count".to_string())?;

    // The school year the sentence opens with, which has to be the fiscal year's.
    let stated = flat[..*at]
        .rfind(" school year")
        .and_then(|end| flat[..end].split_whitespace().next_back())
        .ok_or_else(|| "the participation sentence names no school year".to_string())?;
    if stated != school_year(fiscal_year) {
        return Err(format!(
            "the report is FY{fiscal_year} and its participation sentence says {stated}"
        ));
    }

    // The providers section restates the count. Both readings have to agree.
    let restated = between(window, "The JPSN program had ", " approved providers")
        .ok()
        .and_then(count_after);
    if restated != Some(providers) {
        return Err(format!(
            "the report gives {providers} approved providers and then {restated:?}"
        ));
    }

    // Every district count in the section, of which there are two or three. The editions say
    // "487 districts across Ohio", "495 school districts in Ohio" and "represented 494 school
    // districts" in various combinations, so an optional "school" is stepped over before reading
    // the number — which is also what leaves "over 80% of Ohio's school districts" carrying none.
    let counts: Vec<f64> = window
        .match_indices("districts")
        .filter_map(|(at, _)| {
            let head = &window[..at];
            count_before(
                window,
                at - usize::from(head.ends_with("school ")) * "school ".len(),
            )
        })
        .collect();
    let [districts, rest @ ..] = counts.as_slice() else {
        return Err("no district count in the Jon Peterson section".to_string());
    };
    if rest.iter().any(|other| other != districts) {
        return Err(format!(
            "the section gives {districts} districts and also {rest:?}"
        ));
    }

    // The FY2023 edition's total, and the school year it attributes it to. Absent from both later
    // editions, and left empty rather than derived when it is.
    let (stated_total, stated_total_year) = match between(
        window,
        "Total expenditures for the JPSN program during the ",
        " school year sum to ",
    ) {
        Ok(label) => {
            let year = fiscal_year_of(label.trim())?;
            let at = window.find(" school year sum to ").unwrap_or(0);
            let amount = window[at..]
                .split_whitespace()
                .find(|t| t.starts_with('$'))
                .ok_or_else(|| "the total sentence states no amount".to_string())?;
            (figure(amount)?, year.to_string())
        }
        Err(_) => (String::new(), String::new()),
    };

    let shares = chart(window, "PERCENTAGE OF STUDENTS PER DISABILITY", |t| {
        t.ends_with('%')
    })?;
    let summed: f64 = shares
        .iter()
        .map(|s| s.parse::<f64>().unwrap_or_default())
        .sum();
    if (summed - 100.0).abs() > 0.1 {
        return Err(format!("the disability shares sum to {summed}%, not 100%"));
    }
    let spending = chart(window, "FUNDS SPENT PER", |t| t.starts_with('$'))?;

    let year = fiscal_year.to_string();
    let mut categories = Vec::new();
    for (number, value) in maxima(&flat)? {
        categories.push(vec![
            year.clone(),
            "maximum".to_string(),
            number.to_string(),
            value,
        ]);
    }
    for (series, values) in [("share", shares), ("spending", spending)] {
        for (position, value) in values.into_iter().enumerate() {
            categories.push(vec![
                year.clone(),
                series.to_string(),
                (position + 1).to_string(),
                value,
            ]);
        }
    }

    Ok(Edition {
        summary: vec![
            year,
            format!("{students:.0}"),
            format!("{providers:.0}"),
            format!("{districts:.0}"),
            stated_total,
            stated_total_year,
        ],
        categories,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The FY2023 edition, abridged to the sentences and charts this reader touches.
    ///
    /// Everything here is verbatim from the document. What is cut is the executive summary's prose,
    /// the two unlabelled bar charts, and the page footers — none of which the reader reads. The
    /// award table keeps its `#` column, both district sentences keep their different wordings, and
    /// the expenditure sentence keeps the school year it names, which is not this edition's.
    const FY2023: &str = r#"FY 2023 (July 1, 2022 – June 30, 2023)
                                                                          Maximum Dollar
 #                              Category Name
                                                                          Amount per Year
 1   Speech or Language Impairment*                                             $8,941
 2   Specific Learning Disability, Intellectual Disability, or OHI-minor        $11,632
 3   Hearing Impaired or Emotional Disturbance                                 $17,863
 4   Vision Impaired or OHI-major                                              $21,433
 5   Orthopedic Impairment or Multi-handicapped                                $26,480
 6   Autism, Traumatic Brain Injury, or Hearing and Vision Impaired            $30,000
*Category 1 (Speech or Language Impairment) students may only utilize their scholarship to pay for
related and therapy services.

2022-2023 Student Participation
During the 2022-2023 school year, 8,186 students received services from 439 approved
providers through the Jon Peterson Special Needs Scholarship program. The program drew
students from 487 districts across Ohio.

                      PERCENTAGE OF STUDENTS PER DISABILITY CATEGORY

                             0.7%       3.8%
                                                7.9%   7.8%
                                 2.6%

                                                        77.2%

                          JPSN Category 1      JPSN Category 2   JPSN Category 3
                          JPSN Category 4      JPSN Category 5   JPSN Category 6

DISTRICT OF RESIDENCE
Students enrolled in the JPSN scholarship program during the 2022-2023 school year covered
487 school districts in Ohio. This represents nearly 80 percent of Ohio’s school districts. A
breakdown of students per school district utilizing JPSN can be found here.

Statewide Expenditures
Prior to the 2021-2022 school year, funds used to award scholarships were deducted from the
district in which the student was entitled to attend school, known as the district of residence.
House Bill 110, passed in 2021, eliminated the “district deduction” mechanism and
established direct funding of scholarship programs through the Department of Education and
Workforce. Total expenditures for the JPSN program during the 2021-2022 school year sum to
$81,773,133.70.

                                 FUNDS SPENT PER CATEGORY
                                                   $2,847,779.10

                                        $12,489,008.78

                  $6,064,060.61

                  $750,180.88
                 $2,801,012.05

                                                           $56,821,092.28

             Category 1    Category 2       Category 3    Category 4    Category 5   Category 6

Jon Peterson Special Needs Program
Providers
The JPSN program had 439 approved providers in the 2022-2023 school year. Organizations
that are represented in the program include chartered nonpublic schools and private
providers.
"#;

    /// The FY2024 edition, abridged the same way.
    ///
    /// Two differences from FY2023 are load-bearing and kept: there is no expenditure sentence at
    /// all, and the shares are printed to two decimal places rather than one.
    const FY2024: &str = r#"FY 2024 (July 1, 2023 – June 30, 2024)
                                                                        Maximum Dollar
 #                              Category Name
                                                                        Amount per Year
 1   Speech or Language Impairment*                                            $8,941
 2   Specific Learning Disability, Intellectual Disability, or OHI-minor       $11,632
 3   Hearing Impaired or Emotional Disturbance                                $17,863
 4   Vision Impaired or OHI-major                                             $21,433
 5   Orthopedic Impairment or Multi-handicapped                               $26,480
 6   Autism, Traumatic Brain Injury, or Hearing and Vision Impaired           $30,000
*Category 1 (Speech or Language Impairment) students may only utilize their scholarship to pay for
related and therapy services.

2023-2024 Student Participation
During the 2023-2024 school year, 8,551 students received services from 500 approved
providers through the Jon Peterson Special Needs Scholarship program. The program drew
students from 495 districts across Ohio.

                   PERCENTAGE OF STUDENTS PER DISABILITY CATEGORY
                                 3.63%
                                               7.51% 6.79%
                                 0.71%
                                  2.69%

                                                      78.67%

                             JPSN Category 1   JPSN Category 2   JPSN Category 3
                             JPSN Category 4   JPSN Category 5   JPSN Category 6

DISTRICT OF RESIDENCE
Students enrolled in the JPSN scholarship program during the 2023-2024 school year
represented 495 school districts, covering over 80 percent of Ohio’s school districts. A
breakdown of students per school district utilizing JPSN can be found here.

Statewide Expenditures
Prior to the 2021-2022 school year, funds used to award scholarships were deducted from the
district in which the student was entitled to attend school, known as the district of residence.
House Bill 110, passed in 2021, eliminated the “district deduction” mechanism and
established direct funding of scholarship programs through the Department of Education and
Workforce.

                            FUNDS SPENT PER CATEGORY
                                                  $2,991,497.19
                                $13,702,663.43

                     $6,704,739.83
                     $892,350.40
                   $3,308,496.37

                                                        $67,763,210.31

                          JPSN Category 1        JPSN Category 2   JPSN Category 3
                          JPSN Category 4        JPSN Category 5   JPSN Category 6

Jon Peterson Special Needs Program
Providers
The JPSN program had 500 approved providers in the 2023-2024 school year. Organizations
that are represented in the program include chartered nonpublic schools and private
providers.
"#;

    /// The Jon Peterson section of the consolidated 2025 report, with the table-of-contents line
    /// that repeats its fiscal-year heading and the tail of the Autism section that precedes it.
    ///
    /// Both of those are here on purpose. The contents line means the fiscal year is stated twice
    /// and has to be deduplicated rather than read as a disagreement; the Autism figures sit where
    /// a reader anchored on document order rather than on the participation sentence would pick
    /// them up.
    const FY2025: &str = r#"     FY 2025 (July 1, 2024-June 30, 2025) ................................................ 17

During the 2024-2025 school year, 3,850 students utilized the Autism Scholarship Program.
Expenditures for the Autism Scholarship Program totaled $154,477,262.16, and the average
scholarship amount per participant was $40,124.00.

Jon Peterson Special Needs Scholarship
FY 2025 (JULY 1, 2024-JUNE 30, 2025)
                                                                                 Maximum Dollar
 #                             Category Name
                                                                                 Amount per Year
 1   Speech or Language Impairment*                                                   $9,585
 2   Specific Learning Disability, Intellectual Disability, or OHI-minor              $12,470
 3   Hearing Impaired or Emotional Disturbance                                        $19,150
 4   Vision Impaired or OHI-major                                                     $22,977
 5   Orthopedic Impairment or Multi-Handicapped                                       $28,387
 6   Autism, Traumatic Brain Injury, or Hearing and Vision Impaired                   $32,445
*Category 1 (Speech or Language Impairment) students may only utilize their scholarship to pay for
related and therapy services.

2024-2025 STUDENT PARTICIPATION
During the 2024-2025 school year, 8,680 students received services from 530 approved
providers through the Jon Peterson Special Needs Scholarship Program. Below are displays of
student participation by grade level and students per disability category.

                    PERCENTAGE OF STUDENTS PER DISABILITY
                                 CATEGORY
                                                7.33% 6.43%
                                      3.54%
                          0.65%
                          2.49%

                                                        79.57%

                         JPSN Category 1        JPSN Category 2    JPSN Category 3
                         JPSN Category 4        JPSN Category 5    JPSN Category 6

DISTRICT OF RESIDENCE
Students enrolled in the JPSN Scholarship Program during the 2024-2025 school year
represented 494 school districts, covering over 80% of Ohio’s school districts. A breakdown of
students per school district utilizing JPSN is available here.

                  FY 2025 FUNDS SPENT PER DISABILITY CATEGORY
                                                  $3,192,938.55
                               $14,691,869.72

                      $7,015,533.82

                     $936,465.75
                    $3,160,794.90

                                                         $74,946,785.41

                        JPSN Category 1         JPSN Category 2     JPSN Category 3
                        JPSN Category 4         JPSN Category 5     JPSN Category 6

Jon Peterson Special Needs Program Providers
The JPSN program had 530 approved providers in the 2024-2025 school year. Organizations
that are represented in the program include chartered nonpublic schools and private
providers.
"#;

    /// One series of one edition, in the order the reader emitted it.
    fn series(edition: &Edition, name: &str) -> Vec<f64> {
        edition
            .categories
            .iter()
            .filter(|row| row[1] == name)
            .map(|row| row[3].parse().expect("a figure"))
            .collect()
    }

    /// Which position of `values` carries the smallest one, counting from one.
    fn smallest(values: &[f64]) -> usize {
        let least = values
            .iter()
            .copied()
            .fold(f64::INFINITY, |a: f64, b| a.min(b));
        values.iter().position(|v| *v == least).expect("a minimum") + 1
    }

    fn read(text: &str) -> Edition {
        jpsn_edition(text).expect("the edition reads")
    }

    #[test]
    fn the_three_editions_read_as_one_series() {
        /*
         * The point of the issue this reader answers. JPSN's participation series was published one
         * year at a time and the department deleted each edition as it posted the next, so the two
         * that survive plus the consolidated report's section are three consecutive years — and the
         * programme's first year-on-year change from anything the department itself published.
         */
        let editions = [read(FY2023), read(FY2024), read(FY2025)];
        let summaries: Vec<&[String]> = editions.iter().map(|e| e.summary.as_slice()).collect();
        assert_eq!(summaries[0][..4], ["2023", "8186", "439", "487"]);
        assert_eq!(summaries[1][..4], ["2024", "8551", "500", "495"]);
        assert_eq!(summaries[2][..4], ["2025", "8680", "530", "494"]);
    }

    #[test]
    fn only_the_first_edition_states_a_total_and_it_names_another_year() {
        /*
         * The FY2023 edition is the only one in the lineage that writes an expenditure total in
         * words, and it attributes it to the 2021-2022 school year while reporting FY2023
         * throughout. The two columns carry the figure and the year it names separately, so nothing
         * downstream can read the total as this edition's own without seeing the year beside it.
         */
        let fy2023 = read(FY2023);
        assert_eq!(fy2023.summary[4], "81773133.70");
        assert_eq!(fy2023.summary[5], "2022", "and the edition is FY2023");

        for later in [read(FY2024), read(FY2025)] {
            assert_eq!(
                later.summary[4..],
                ["", ""],
                "the {} edition gained a stated total; the catalog record says it has none",
                later.summary[0]
            );
        }
    }

    #[test]
    fn the_stated_total_equals_the_sum_of_the_chart_it_sits_above() {
        /*
         * What licenses the derivation everywhere else. Only FY2023 states a total, and it equals
         * the sum of its own six category figures to the cent — so the department itself treats
         * that sum as "total expenditures for the JPSN program", and summing the chart in an
         * edition that states nothing is quoting its arithmetic rather than inventing one.
         */
        let fy2023 = read(FY2023);
        let summed: f64 = series(&fy2023, "spending").iter().sum();
        let stated: f64 = fy2023.summary[4].parse().expect("a figure");
        assert!(
            (summed - stated).abs() < 0.005,
            "{summed:.2} vs {stated:.2}"
        );
    }

    #[test]
    fn the_award_table_numbers_its_rows_and_the_pie_charts_do_not() {
        /*
         * Why `position` is not a category. The award table prints a `#` column, so every maximum
         * arrives with its category stated and the reader can check the numbering runs 1 through 6.
         * The charts print their labels around the slices, and matching the shares by magnitude
         * across the three editions shows the order moving: the smallest slice, near 0.7% in every
         * edition, is read first in FY2023 and fourth in both later ones.
         *
         * The dollar chart's order happens to hold still across all three. That is a coincidence of
         * layout, not a guarantee, and it is recorded here so nobody reads the stability as one.
         */
        for text in [FY2023, FY2024, FY2025] {
            let edition = read(text);
            let numbered: Vec<&str> = edition
                .categories
                .iter()
                .filter(|row| row[1] == "maximum")
                .map(|row| row[2].as_str())
                .collect();
            assert_eq!(numbered, ["1", "2", "3", "4", "5", "6"]);
        }

        let positions: Vec<usize> = [FY2023, FY2024, FY2025]
            .iter()
            .map(|text| smallest(&series(&read(text), "share")))
            .collect();
        assert_eq!(
            positions,
            [1, 4, 4],
            "the smallest disability share moved position between editions; if it has stopped \
             moving that is not evidence the position is a category"
        );
    }

    #[test]
    fn the_two_editions_that_share_a_table_do_not_share_a_per_student_spend() {
        /*
         * The tension the fixture exists to hold rather than settle. FY2023 and FY2024 print
         * identical award maxima, so the statutory ceiling did not move between them — and yet the
         * spending chart rises 11.6% per student. FY2024 to FY2025 behaves the other way: the
         * maxima rise 7.2% and the per-student spend 7.4%, which is what a year of the same
         * programme under a raised ceiling looks like.
         *
         * Either the FY2023 chart is not FY2023's, which would fit the expenditure sentence naming
         * the 2021-2022 school year, or the programme's mix shifted sharply in one year. This test
         * pins the arithmetic both readings have to explain.
         */
        let editions: Vec<Edition> = [FY2023, FY2024, FY2025].iter().map(|t| read(t)).collect();
        let spend: Vec<f64> = editions
            .iter()
            .map(|e| {
                series(e, "spending").iter().sum::<f64>()
                    / e.summary[1].parse::<f64>().expect("a count")
            })
            .collect();
        // Category 1, which both later editions prorate off the same statutory supplement.
        // Category 6 is the wrong column to measure the table's movement on: FY2023 and FY2024
        // both print a round $30,000 there where the same proration would give more.
        let maxima: Vec<f64> = editions.iter().map(|e| series(e, "maximum")[0]).collect();

        assert_eq!(maxima[0], maxima[1], "FY2023 and FY2024 print one table");
        assert!(
            (spend[1] / spend[0] - 1.1164).abs() < 0.0005,
            "{:.4}",
            spend[1] / spend[0]
        );
        assert!(
            (maxima[2] / maxima[1] - 1.0720).abs() < 0.0005,
            "{:.4}",
            maxima[2] / maxima[1]
        );
        assert!(
            (spend[2] / spend[1] - 1.0738).abs() < 0.0005,
            "{:.4}",
            spend[2] / spend[1]
        );
    }

    #[test]
    fn a_heading_that_closes_outside_its_fiscal_year_is_an_error() {
        let doctored = FY2023.replace("– June 30, 2023)", "– June 30, 2024)");
        let error = jpsn_edition(&doctored).expect_err("the span is wrong");
        assert!(
            error.contains("does not close in its own fiscal year"),
            "{error}"
        );
    }

    #[test]
    fn a_participation_sentence_in_another_school_year_is_an_error() {
        let doctored = FY2023.replace(
            "During the 2022-2023 school year, 8,186",
            "During the 2021-2022 school year, 8,186",
        );
        let error = jpsn_edition(&doctored).expect_err("the school year is wrong");
        assert!(error.contains("FY2023"), "{error}");
        assert!(error.contains("2021-2022"), "{error}");
    }

    #[test]
    fn a_provider_count_the_report_restates_differently_is_an_error() {
        let doctored = FY2024.replace("The JPSN program had 500", "The JPSN program had 501");
        let error = jpsn_edition(&doctored).expect_err("the counts disagree");
        assert!(error.contains("approved providers"), "{error}");
    }

    #[test]
    fn a_district_count_that_disagrees_with_itself_is_an_error() {
        let doctored = FY2023.replace("487 districts across Ohio", "486 districts across Ohio");
        let error = jpsn_edition(&doctored).expect_err("the counts disagree");
        assert!(error.contains("486") && error.contains("487"), "{error}");
    }

    #[test]
    fn shares_that_do_not_sum_to_a_hundred_are_an_error() {
        let doctored = FY2025.replace("79.57%", "70.57%");
        let error = jpsn_edition(&doctored).expect_err("the slices do not close");
        assert!(error.contains("not 100%"), "{error}");
    }

    #[test]
    fn an_award_table_that_skips_a_category_is_an_error() {
        // The dollar sign off one row, which is what a re-typeset table looks like: the amount is
        // still there and still beside its number, and the reader no longer sees it.
        let doctored = FY2023.replace("$21,433", "21,433");
        let error = jpsn_edition(&doctored).expect_err("a category is missing");
        assert!(error.contains("[1, 2, 3, 5, 6]"), "{error}");
    }

    #[test]
    fn a_second_jon_peterson_section_is_an_error() {
        /*
         * The consolidated edition carries the sentence once among five programmes. If a future one
         * carried it twice — a programme split, or a section repeated in an appendix — reading the
         * first would silently pick a side.
         */
        let doctored = format!(
            "{FY2023}\nDuring the 2022-2023 school year, 12 students received services from 3 \
             approved providers through the Jon Peterson Special Needs Scholarship program."
        );
        let error = jpsn_edition(&doctored).expect_err("two sentences");
        assert!(
            error.contains("2 Jon Peterson participation sentences"),
            "{error}"
        );
    }
}
