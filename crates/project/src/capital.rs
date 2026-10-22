//! How a school district builds without a ballot, read from the sections that let it.
//!
//! A bond issue goes to the voters under R.C. 133.18. Three other routes do not: an installment
//! contract or notes for energy conservation under R.C. 3313.372, the parallel energy securities
//! of R.C. 133.06(G), and a lease-purchase agreement under R.C. 3313.375. The
//! `revenue-stream/capital-finance-without-a-vote` node states their limits, and each limit the
//! statute writes as a number word is read here, so that an amendment fails a binding instead of
//! leaving the prose behind. The same goes for the competitive-bidding threshold R.C. 3313.46
//! borrows from R.C. 9.17, and for the one place a school bidding rule says who finds an urgent
//! necessity.

use crate::levy::per_cent_in_words;
use crate::statute::{prior, section};

/// A whole number written in words, as these sections write their terms and amounts.
///
/// Reads one to nineteen, a round ten up to ninety, a hyphenated compound such as
/// "seventy-five", and any of those followed by " thousand". A phrase in no such shape is refused
/// rather than guessed at.
#[must_use]
pub fn number_in_words(words: &str) -> Option<u64> {
    const UNITS: [&str; 19] = [
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];
    const TENS: [&str; 8] = [
        "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];
    let unit = |w: &str| UNITS.iter().position(|u| *u == w).map(|i| i as u64 + 1);
    let ten = |w: &str| {
        TENS.iter()
            .position(|t| *t == w)
            .map(|i| (i as u64 + 2) * 10)
    };
    let below_hundred = |w: &str| match w.split_once('-') {
        Some((t, u)) => Some(ten(t)? + unit(u).filter(|u| *u < 10)?),
        None => unit(w).or_else(|| ten(w)),
    };
    let words = words.trim();
    match words.strip_suffix(" thousand") {
        Some(rest) => below_hundred(rest).map(|n| n * 1_000),
        None => below_hundred(words),
    }
}

/// The words between `lead` and the next `tail` in `body`, if both occur in that order.
fn between<'a>(body: &'a str, lead: &str, tail: &str) -> Option<&'a str> {
    let start = body.find(lead)? + lead.len();
    let end = start + body[start..].find(tail)?;
    Some(&body[start..end])
}

/// The energy conservation measures R.C. 3313.372(A) enumerates in a body of the section.
///
/// Counts the numbered paragraphs between the opening of division (A) and the opening of
/// division (B). The last of them is always the catch-all "any other modification ... approved
/// by the Ohio facilities construction commission", so the count is the named measures plus one.
#[must_use]
pub fn energy_measures(body: &str) -> usize {
    body.lines()
        .skip_while(|l| !l.starts_with("(A) "))
        .skip(1)
        .take_while(|l| !l.starts_with("(B) "))
        .filter(|l| {
            l.strip_prefix('(')
                .and_then(|r| r.split_once(") "))
                .is_some_and(|(n, _)| n.parse::<u32>().is_ok())
        })
        .count()
}

/// [`energy_measures`] in R.C. 3313.372 as it reads now, after H.B. 15 of the 136th General
/// Assembly.
#[must_use]
pub fn energy_measures_in_force() -> usize {
    energy_measures(section("3313.372").body)
}

/// [`energy_measures`] in R.C. 3313.372 as it read from September 29, 2017, before H.B. 15.
#[must_use]
pub fn energy_measures_before_hb15() -> usize {
    energy_measures(prior("3313.372", "9-29-2017").body)
}

/// Whether a body of R.C. 3313.372(A) names solar panels as a measure.
#[must_use]
pub fn names_solar_panels(body: &str) -> bool {
    body.lines().any(|l| l.ends_with(" Solar panels;"))
}

/// The most years an energy conservation installment contract may run: R.C. 3313.372(B)(2),
/// "The remaining balance of the costs thereof shall be paid within ... years".
#[must_use]
pub fn energy_contract_term_years() -> Option<u64> {
    let words = between(
        section("3313.372").body,
        "The remaining balance of the costs thereof shall be paid within ",
        " years",
    )?;
    number_in_words(words)
}

/// The fraction of the cost an energy conservation installment contract must pay within two
/// years: R.C. 3313.372(B)(1), "Not less than one-... of the costs".
#[must_use]
pub fn energy_contract_first_payment_share() -> Option<f64> {
    let words = between(
        section("3313.372").body,
        "Not less than one-",
        " of the costs thereof shall be paid within two years",
    )?;
    let denominator = number_in_words(words.strip_suffix("th")?)?;
    Some(1.0 / denominator as f64)
}

/// The ceiling on all net indebtedness a district incurs without a vote, as a share of tax
/// valuation, as R.C. 3313.372(D) writes it: "shall not exceed ... of the district's tax
/// valuation".
#[must_use]
pub fn energy_notes_unvoted_ceiling() -> Option<f64> {
    let words = between(
        section("3313.372").body,
        "except section 3318.052 of the Revised Code, shall not exceed ",
        " of the district's tax valuation",
    )?;
    per_cent_in_words(words)
}

/// Whether R.C. 3313.372 leaves its debt out of the R.C. 133.06 count of net indebtedness.
#[must_use]
pub fn energy_debt_excluded_from_net_indebtedness() -> bool {
    section("3313.372").body.lines().any(|l| {
        l.starts_with("(E) ")
            && l.contains("shall not be included in the calculation of the net indebtedness")
            && l.contains("section 133.06")
    })
}

/// The principal of energy securities a district may issue without a vote, as a share of tax
/// valuation: R.C. 133.06(G)(3), "in a principal amount not to exceed ... of its tax valuation".
#[must_use]
pub fn energy_securities_principal_limit() -> Option<f64> {
    let words = between(
        section("133.06").body,
        "may issue securities without a vote of the electors in a principal amount not to exceed ",
        " of its tax valuation",
    )?;
    per_cent_in_words(words)
}

/// The ceiling R.C. 133.06(G)(3) sets on all of a district's unvoted net indebtedness once it
/// issues energy securities, as a share of tax valuation.
#[must_use]
pub fn energy_securities_unvoted_ceiling() -> Option<f64> {
    let body = section("133.06").body;
    let paragraph = body
        .lines()
        .find(|l| l.contains("may issue securities without a vote of the electors"))?;
    let words = between(
        paragraph,
        "except section 3318.052 of the Revised Code, shall not exceed ",
        " of the district's tax valuation",
    )?;
    per_cent_in_words(words)
}

/// The longest a lease-purchase agreement may run, in years: R.C. 3313.375(A), "in no event
/// more than ... years".
#[must_use]
pub fn lease_purchase_term_ceiling_years() -> Option<u64> {
    let words = between(section("3313.375").body, "in no event more than ", " years")?;
    number_in_words(words)
}

/// Whether R.C. 3313.375 says lease-purchase obligations are not net indebtedness under R.C.
/// 133.06.
#[must_use]
pub fn lease_purchase_is_outside_net_indebtedness() -> bool {
    section("3313.375").body.contains(
        "shall not be considered to be net indebtedness of a school district under section 133.06",
    )
}

/// Whether R.C. 3313.375 lets lease-purchase payments be paid as current operating expenses.
#[must_use]
pub fn lease_purchase_paid_as_operating_expense() -> bool {
    section("3313.375")
        .body
        .contains("may be deemed to be, and paid as, current operating expenses")
}

/// The competitive-bidding threshold R.C. 9.17(A)(1) set through calendar year 2024, in dollars.
#[must_use]
pub fn bidding_threshold_through_2024() -> Option<u64> {
    let words = between(
        section("9.17").body,
        "through calendar year 2024, ",
        " dollars",
    )?;
    number_in_words(words)
}

/// The yearly increase R.C. 9.17(A)(2) applies to the threshold after 2024, as a share.
#[must_use]
pub fn bidding_threshold_escalator() -> Option<f64> {
    let words = between(
        section("9.17").body,
        "the amount for the previous calendar year increased by ",
        " as determined",
    )?;
    per_cent_in_words(words)
}

/// Who decides that a case is one of urgent necessity, where a section says.
///
/// Reads the words after "urgent necessity as determined by " up to the next semicolon. R.C.
/// 3311.75 writes "two-thirds vote of the board". R.C. 3313.46 writes the exception, "except in
/// cases of urgent necessity", and names no one, so it gives `None`.
#[must_use]
pub fn urgent_necessity_determiner(number: &str) -> Option<&'static str> {
    let body = section(number).body;
    let start = body.find("urgent necessity as determined by ")?
        + "urgent necessity as determined by ".len();
    let end = start + body[start..].find(';')?;
    Some(&body[start..end])
}

/// Whether a section's text contains the phrase "urgent necessity" at all.
#[must_use]
pub fn mentions_urgent_necessity(number: &str) -> bool {
    section(number).body.contains("urgent necessity")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_words_are_read_in_every_shape_the_sections_use() {
        assert_eq!(number_in_words("fifteen"), Some(15));
        assert_eq!(number_in_words("thirty"), Some(30));
        assert_eq!(number_in_words("seventy-five thousand"), Some(75_000));
        assert_eq!(number_in_words("fifty thousand"), Some(50_000));
    }

    #[test]
    fn a_number_word_in_no_shape_is_refused() {
        assert_eq!(number_in_words("fifteen years"), None);
        assert_eq!(number_in_words("seventy-fifteen"), None);
        assert_eq!(number_in_words("one hundred"), None);
        assert_eq!(number_in_words("15"), None);
    }

    #[test]
    fn the_measure_count_reads_only_division_a() {
        let body = "(A) Defined. It includes:\n(1) One;\n(2) Two;\n(B) Then:\n(1) Not a measure.";
        assert_eq!(energy_measures(body), 2);
        assert!(!names_solar_panels(body));
        assert!(names_solar_panels("(A) x\n(9) Solar panels;\n(B) y"));
    }

    #[test]
    fn the_first_payment_is_one_fifteenth() {
        let share = energy_contract_first_payment_share().expect("reads");
        assert!((share - 1.0 / 15.0).abs() < 1e-15, "{share}");
    }

    #[test]
    fn a_section_without_a_determiner_gives_none() {
        assert_eq!(urgent_necessity_determiner("3313.46"), None);
        assert_eq!(
            urgent_necessity_determiner("3311.75"),
            Some("two-thirds vote of the board")
        );
    }
}
