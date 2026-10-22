//! How a school levy is voted, renewed and bonded against, read from the sections that say so.
//!
//! The levy modality nodes under `revenue-stream/` cite five sections of the Revised Code and one
//! superseded version. Two of their claims are numbers the statute writes in words — R.C.
//! 133.06's net indebtedness limits, "one-tenth of one per cent" and "nine per cent" of tax
//! valuation — and one is a count of what an amendment removed. Reading them here, rather than
//! typing them into a node, is what lets a later amendment fail a binding instead of leaving the
//! prose quietly behind.

use crate::statute::{prior, section};

/// A per cent of tax valuation as R.C. 133.06 writes one, as a fraction of one.
///
/// The section writes its limits as words: "nine per cent", "four per cent", "three per cent",
/// and the unvoted limit as "one-tenth of one per cent". Those are the only two shapes, and a
/// phrase in neither is refused rather than guessed at.
#[must_use]
pub fn per_cent_in_words(words: &str) -> Option<f64> {
    const UNITS: [&str; 10] = [
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    ];
    let words = words.trim();
    let whole = |w: &str| UNITS.iter().position(|u| *u == w).map(|i| (i + 1) as f64);
    if let Some(rest) = words.strip_suffix(" per cent") {
        if let Some(of) = rest.strip_suffix(" of one") {
            // "one-tenth of one per cent".
            let (numerator, denominator) = of.split_once('-')?;
            let denominator = match denominator {
                "tenth" => 10.0,
                "hundredth" => 100.0,
                _ => return None,
            };
            return Some(whole(numerator)? / denominator / 100.0);
        }
        return Some(whole(rest)? / 100.0);
    }
    None
}

/// The per cent a division of R.C. 133.06 limits net indebtedness to, read from the division.
///
/// Finds the paragraph that opens `(<division>)`, then the phrase ending "per cent of its tax
/// valuation" inside it, and reads the words before it back to "equal to" or "exceed". `None`
/// if the division states no such limit.
#[must_use]
pub fn indebtedness_limit(division: &str) -> Option<f64> {
    const TAIL: &str = " per cent of its tax valuation";
    let opening = format!("({division}) ");
    let paragraph = section("133.06")
        .body
        .lines()
        .find(|l| l.starts_with(&opening))?;
    let end = paragraph.find(TAIL)?;
    let before = &paragraph[..end];
    let start = ["an amount equal to ", "exceed "]
        .iter()
        .filter_map(|lead| before.rfind(lead).map(|i| i + lead.len()))
        .max()?;
    per_cent_in_words(&format!("{} per cent", &before[start..]))
}

/// The net indebtedness a school district may incur without a vote, as a share of tax
/// valuation: R.C. 133.06(A).
#[must_use]
pub fn unvoted_limit() -> f64 {
    indebtedness_limit("A").unwrap_or(f64::NAN)
}

/// The net indebtedness a school district may incur in all, voted and unvoted: R.C. 133.06(B).
#[must_use]
pub fn total_limit() -> f64 {
    indebtedness_limit("B").unwrap_or(f64::NAN)
}

/// The net indebtedness above which a bond question needs the director's and the tax
/// commissioner's consent to reach the ballot: R.C. 133.06(C).
#[must_use]
pub fn consent_threshold() -> f64 {
    indebtedness_limit("C").unwrap_or(f64::NAN)
}

/// The renewal sentences of R.C. 5705.21 that let a resolution increase the levy it renews.
///
/// Before H.B. 96 there were two: one for a levy imposed under division (A) or (B), "to increase
/// or decrease a single levy", and one for a permanent improvement levy, "to increase or decrease
/// a single such existing levy". The act struck "increase or" from both. Counted on a body so
/// that the same reading measures the version before and the version in force.
#[must_use]
pub fn renew_and_increase_sentences(body: &str) -> usize {
    body.lines()
        .filter(|l| l.contains("may propose to renew"))
        .filter(|l| l.contains("to increase or decrease a single"))
        .count()
}

/// [`renew_and_increase_sentences`] in R.C. 5705.21 as it read on April 9, 2025.
#[must_use]
pub fn renew_and_increase_sentences_before_hb96() -> usize {
    renew_and_increase_sentences(prior("5705.21", "4-9-2025").body)
}

/// [`renew_and_increase_sentences`] in R.C. 5705.21 as it reads now.
#[must_use]
pub fn renew_and_increase_sentences_in_force() -> usize {
    renew_and_increase_sentences(section("5705.21").body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_shapes_of_a_written_per_cent_are_read() {
        assert_eq!(per_cent_in_words("nine per cent"), Some(0.09));
        assert_eq!(per_cent_in_words("four per cent"), Some(0.04));
        let tenth = per_cent_in_words("one-tenth of one per cent").expect("reads");
        assert!((tenth - 0.001).abs() < 1e-15, "{tenth}");
    }

    #[test]
    fn a_phrase_in_neither_shape_is_refused() {
        assert_eq!(per_cent_in_words("nine percent"), None);
        assert_eq!(per_cent_in_words("ninety per cent"), None);
        assert_eq!(per_cent_in_words("one-third of one per cent"), None);
    }

    #[test]
    fn the_three_limits_are_read_from_their_own_divisions() {
        assert!((unvoted_limit() - 0.001).abs() < 1e-15);
        assert!((total_limit() - 0.09).abs() < 1e-15);
        assert!((consent_threshold() - 0.04).abs() < 1e-15);
        // (D) lists exclusions and states no per cent of tax valuation.
        assert_eq!(indebtedness_limit("D"), None);
    }

    #[test]
    fn hb96_struck_both_renew_and_increase_sentences_and_left_none() {
        assert_eq!(renew_and_increase_sentences_before_hb96(), 2);
        assert_eq!(renew_and_increase_sentences_in_force(), 0);
    }
}
