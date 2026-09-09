//! What *DeRolph I* did with *Walter*, which the corpus recorded as unclear.
//!
//! `litigation/cincinnati-v-walter-1979` carries an `[open]`: *"Whether DeRolph I overruled
//! Walter or distinguished it on the evidentiary record is a question the opinions handle less
//! cleanly than one might expect."*
//!
//! Neither. The opinion **applies** *Walter* — its test, its jurisdiction, and in the dissent its
//! reading of the record — and reaches the opposite result on facts. The word "overrule" does not
//! appear in *DeRolph I* at all.
//!
//! # Why that is the interesting answer rather than a technicality
//!
//! *Walter* upheld the Equal Yield Formula 4-3 in 1979 and *DeRolph I* struck down its successor
//! in 1997 on the same constitutional text. If *Walter* were overruled, the change would be in
//! the law and the corpus would be reading two doctrines. It is not: the majority writes that
//! *Walter*'s observations "are as applicable today as they were at the time Walter was decided"
//! and then applies *Walter*'s own words — "starved for funds", "lack teachers, buildings, or
//! equipment" — to a record eighteen years later. One standard, two records, two answers.
//!
//! That is also why the three courts below disagreed about *Walter* rather than about the
//! Constitution: the trial court held it "confined to its own set of facts", the court of appeals
//! held that nothing had substantially changed and *Walter* therefore controlled, and the Supreme
//! Court agreed with the trial court. The whole dispute is about whether the facts moved.
//!
//! # What the corpus still does not hold
//!
//! *Walter* itself. Everything here is *DeRolph*'s quotation of it, which is enough to settle what
//! *DeRolph* did and not enough for the node's second `[open]` — reading *Walter*'s dissent
//! against *DeRolph*'s majority — because *DeRolph I* never quotes that dissent.

const OPINIONS: &str = include_str!("../fixtures/derolph-opinions.txt");

/// One opinion's text with runs of whitespace collapsed, so a quotation split across a line break
/// still reads as one token. These are PDF extracts of law reports; every line is broken.
fn flat(key: &str) -> String {
    edfund_core::records::record(OPINIONS, key)
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The word does not appear. Not once, in an opinion that names *Walter* 76 times.
///
/// *DeRolph II* carries the only occurrence in the four, and it is a citation signal about
/// *Ursuline Academy* — "overruled in part on other grounds" — in a dissent's string cite about
/// self-executing constitutional provisions. Nothing in the sequence overrules *Walter*.
#[test]
fn no_opinion_in_the_sequence_overrules_walter() {
    let first = flat("derolph-i");
    assert_eq!(first.matches("Walter").count(), 76);
    assert_eq!(first.to_lowercase().matches("overrul").count(), 0);

    let second = flat("derolph-ii");
    assert_eq!(second.to_lowercase().matches("overrul").count(), 1);
    assert!(second.contains(
        "Ursuline Academy of Cleveland v. Bd. of Tax Appeals (1943), 141 Ohio St. 563, 26 O.O. \
         152, 49 N.E.2d 674, overruled in part on other grounds"
    ));

    for key in ["derolph-iii", "derolph-iv"] {
        assert_eq!(
            flat(key).to_lowercase().matches("overrul").count(),
            0,
            "{key}"
        );
    }
}

/// The majority applies *Walter*'s test and gets the other answer.
///
/// Two sentences carry it. One keeps the law: *Walter*'s observations on judicial power "are as
/// applicable today as they were at the time Walter was decided". The other applies it: "when we
/// apply the tests of Miller and Walter … the evidence is overwhelming that many districts are
/// 'starved for funds,' and lack teachers, buildings, or equipment" — which is *Walter*'s own
/// phrase for the limit on legislative discretion, turned on a later record.
#[test]
fn the_majority_applies_walters_test_and_reaches_the_other_answer() {
    let opinion = flat("derolph-i");
    assert!(opinion.contains(
        "Also, when we apply the tests of Miller and Walter as to what is meant by the words \
         \u{201c}thorough and efficient,\u{201d} the evidence is overwhelming that many districts \
         are \u{201c}starved for funds,\u{201d} and lack teachers, buildings, or equipment."
    ));
    assert!(opinion.contains(
        "However, the following observations in Walter concerning the power of the judiciary are \
         as applicable today as they were at the time Walter was decided"
    ));

    // And the statute is what the opinion says changed, not the doctrine.
    assert!(opinion.contains(
        "this court entertained certain constitutional challenges to a statutory system for \
         school funding that has since been repealed and replaced with the current statutory \
         framework"
    ));
}

/// Three courts read *Walter*'s reach three ways, and the disagreement is about facts.
///
/// The trial court held it not controlling; the court of appeals held that nothing had
/// substantially changed and that it therefore dictated the result; the Supreme Court agreed with
/// the trial court that it is "confined to its own set of facts". Whether *Walter* governs is the
/// question the case turned on below, and it is a question about the record.
#[test]
fn three_courts_read_walters_reach_three_ways() {
    let opinion = flat("derolph-i");
    assert!(opinion.contains(
        "The trial court found that Walter was not controlling precedent on the issues involved \
         in the case at bar."
    ));
    assert!(opinion.contains(
        "The court of appeals disagreed, concluding that the system of educational funding had \
         not substantially changed since Walter was decided"
    ));
    assert!(opinion.contains(
        "The trial court held that this court\u{2019}s decision in Walter \u{201c}is confined to \
         its own set of facts.\u{201d} I agree."
    ));
}

/// The dissent reads the same record *Walter* read, which is the clearest sign the split is factual.
///
/// It answers the majority not by defending *Walter*'s law but by finding *Walter*'s facts again:
/// no student short of the required days of instruction, and districts calling themselves starved
/// while offering programmes above the state minimum. The quotation inside it is *Walter*'s own.
#[test]
fn the_dissent_reads_the_same_record_walter_read() {
    assert!(flat("derolph-i").contains(
        "But, as in Walter, plaintiffs did not provide evidence that any student received fewer \
         than the full number of days of instruction required by law, and, as in Walter"
    ));
}

/// And *Walter* is the authority *DeRolph I* relies on for having jurisdiction at all.
///
/// The dissent argued the question was a nonjusticiable political one. Justice Resnick's answer is
/// that *Walter* settled justiciability in 1979 — so the case the state would have wanted as its
/// precedent is the case that lets the court hear the challenge.
#[test]
fn walter_is_the_authority_for_hearing_the_case() {
    assert!(flat("derolph-i").contains(
        "this court made clear that in certain instances we would have jurisdiction to determine \
         the constitutionality of Ohio\u{2019}s system of funding public"
    ));
}

/// What had not happened since *Walter* is a costing, and the opinion says so with a date.
///
/// *"the General Assembly does not know the actual per-pupil cost of education in Ohio, since it
/// has not calculated the cost of a quality education since 1973-1974."* Written in 1997, about a
/// figure last computed twenty-four years earlier.
///
/// That is the corpus's own subject arriving in the opinion: the base cost the funding formula
/// multiplies is the thing the court says nobody had priced. `parameter/base-cost-per-pupil`
/// carries the modern build-up, and this dates the gap it was built to close.
#[test]
fn what_had_not_happened_since_walter_is_a_costing() {
    assert!(flat("derolph-i").contains(
        "the General Assembly does not know the actual per-pupil cost of education in Ohio, since \
         it has not calculated the cost of a quality education since 1973-1974"
    ));
}
