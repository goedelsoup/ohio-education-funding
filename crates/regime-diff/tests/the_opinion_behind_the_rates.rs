//! The charge-off rate series, against the opinion it was transcribed from.
//!
//! [`RATES`] carries four entries with a session-law citation on each, and the parameter node says
//! they come from DeRolph I paragraph 97. Until `ohio-courts` was wired that was a claim about a
//! document nothing in this repository held: the citations were checkable only by a reader who
//! went and found the opinion.
//!
//! The connector's recorded blocker was "opinions are PDFs, and trial-level rulings such as the
//! 2025 EdChoice decision are not in the supreme court archive at all." The two clauses are not
//! equally true. The first stopped being a blocker the moment `Format::Pdf` had a reader. The
//! second is correct and unfixable from here — a common pleas ruling is not in the supreme court's
//! archive because it is not the supreme court's — and it is why this connector is wired for the
//! four DeRolph opinions and not for the case the corpus most recently added a node for.

use regime_diff::charge_off::SourcedTo;

const OPINIONS: &str = include_str!("../fixtures/derolph-opinions.txt");

/// One opinion, through the shared reader of the record format `connect` writes.
///
/// This was a hand-rolled copy of that format's parse, as was the Revised Code extract's in
/// `project`. The writer is `connect::fixtures::statute::build_records`; the reader is
/// [`edfund_core::records`]. See issue #157.
fn opinion(key: &str) -> edfund_core::records::Record<'static> {
    edfund_core::records::record(OPINIONS, key)
}

/// The opinion's text with runs of whitespace collapsed, so a citation split across a line break
/// still reads as one token.
fn flat(key: &str) -> String {
    opinion(key)
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// **All four opinions are present, and each is checked against its own reporter citation.**
///
/// Not by length, and not by "does it mention the court". `derolph-iii` was first wired to
/// 2001-Ohio-114, which is *State ex rel. Smith v. Industrial Commission*, a workers' compensation
/// appeal — a WebCite number guessed from a plausible pattern rather than read off the document.
/// It retrieved a real PDF from the real archive and would have passed any check that did not ask
/// what the document is.
///
/// So the check asks. Each opinion must name the parties and match the volume and page of Ohio
/// Official Reports it was cited to, both of which the archive prints on the first line.
#[test]
fn each_opinion_is_the_case_it_is_cited_as() {
    assert_eq!(edfund_core::records::records(OPINIONS).count(), 4);

    let expected = [
        ("derolph-i", "78 Ohio St.3d 193", "1997-Ohio-84"),
        ("derolph-ii", "89 Ohio St.3d 1", "2000-Ohio-437"),
        ("derolph-iii", "93 Ohio St.3d 309", "2001-Ohio-1343"),
        ("derolph-iv", "97 Ohio St.3d 434", "2002-Ohio-6750"),
    ];
    for (key, reporter, webcite) in expected {
        // The record's own title, which the extract declares rather than deriving from a comment.
        // It carried `DeRolph III, 2001, at 93 Ohio St` when it was cut out of the first sentence
        // of a prose note, so the reporter citation is checked here too.
        let record = opinion(key);
        assert!(
            record.title.contains(reporter),
            "{key} is titled without its reporter citation: {:?}",
            record.title
        );
        let body = flat(key);
        assert!(
            body.contains(&format!("Ohio Official Reports at {reporter}")),
            "{key} is not reported at {reporter}"
        );
        assert!(
            body.contains(&format!("Cite as DeRolph v. State, {webcite}")),
            "{key} does not cite as {webcite}"
        );
        assert!(
            body.contains("DEROLPH ET AL."),
            "{key} is not a DeRolph case"
        );
        assert!(body.len() > 40_000, "{key} is {} chars", body.len());
    }
}

/// **Every authority string in `RATES` is verbatim from DeRolph I, and so is the base.**
///
/// Paragraph 97 recites the whole progression:
///
/// > At the time this case was filed, total assessed value was multiplied by .02 (i.e., twenty
/// > mills times total assessed valuation) to produce the applicable charge-off. 144 Ohio Laws,
/// > Part III, 3987, 4122. During the pendency of the litigation, the twenty-mill multiplier was
/// > increased to 20.5 mills (Am.Sub.H.B. No. 152, Section 36.12, 145 Ohio Laws, Part III,
/// > 4432-4433) and was raised thereafter. Currently, total taxable value is multiplied by .023
/// > for purposes of calculating the charge-off. R.C. 3317.022.
///
/// The corpus built three of its four rate entries from that sentence. This checks the citations
/// character by character rather than trusting a transcription, which is the whole point of
/// holding the document.
#[test]
fn every_charge_off_authority_is_quoted_from_the_opinion() {
    let d1 = flat("derolph-i");

    // The rates, as the opinion states them.
    assert!(d1.contains("multiplied by .02 (i.e., twenty mills times total assessed valuation)"));
    assert!(d1.contains("increased to 20.5 mills"));
    assert!(d1.contains("total taxable value is multiplied by .023"));

    // And the authorities the corpus carries, verbatim.
    //
    // Partitioned on `sourced_to` rather than on a substring of `authority`. The rates that came
    // out of this opinion are a fact about where they were read, not about whether their citation
    // happens to spell "LSC"; searching the prose would quietly exempt a reworded entry and
    // quietly assert an Evidence-Based Model citation against a 1997 opinion when the series is
    // extended, which the series' own doc comment says it will be.
    let from_opinion: Vec<_> = regime_diff::RATES
        .iter()
        .filter(|r| r.sourced_to == SourcedTo::DeRolphI)
        .collect();
    assert!(
        !from_opinion.is_empty(),
        "no rate claims to come from DeRolph I, so this test checks nothing"
    );
    for rate in from_opinion {
        let cited = rate
            .authority
            .split(", as recited")
            .next()
            .expect("non-empty");
        assert!(
            d1.contains(cited),
            "{} mills cites {cited:?}, which is not in the opinion",
            rate.mills
        );
    }
}

/// **The DeRolph-era base is total taxable value, in the opinion's own words.**
///
/// `ValuationBase::TotalTaxable` on the first three rates was an inference from the opinion's
/// phrasing. It is not an inference any more: the opinion defines the term and cites the section
/// that defines it.
///
/// This matters because the base is where the corpus went wrong for fourteen phases — it carried a
/// definition of *recognized* valuation that described a different mechanism entirely. The
/// DeRolph-era half of that distinction is now sourced to the document that drew it.
#[test]
fn the_opinion_names_the_base_the_corpus_assigned_to_those_years() {
    let d1 = flat("derolph-i");
    assert!(d1.contains(
        "The charge-off is the total taxable value of real and tangible personal property in the \
         district times a certain percentage"
    ));
    assert!(d1.contains("defining “total taxable value”"));

    // The expected count is derived from the filter rather than written as 3: a hardcoded length
    // turns "a rate was added to the series" into a mismatch that names neither the rate nor the
    // reason. What is being asserted is that *every* rate read out of this opinion carries the
    // base the opinion defines, however many of them there come to be.
    use regime_diff::charge_off::ValuationBase;
    let derolph_era: Vec<ValuationBase> = regime_diff::RATES
        .iter()
        .filter(|r| r.sourced_to == SourcedTo::DeRolphI)
        .map(|r| r.base)
        .collect();
    assert!(!derolph_era.is_empty(), "no rate comes from DeRolph I");
    assert_eq!(
        derolph_era,
        vec![ValuationBase::TotalTaxable; derolph_era.len()]
    );
}

/// The opinion also states the mechanism the corpus reproduces, which is worth pinning.
///
/// Base amount times a cost-of-doing-business factor times ADM, less the charge-off, equals basic
/// state aid. `regime-diff` computes exactly that shape, and the two have never been set against
/// each other before.
#[test]
fn the_opinions_description_of_the_formula_is_the_shape_regime_diff_computes() {
    let d1 = flat("derolph-i");
    assert!(d1.contains(
        "By multiplying the formula amount, the cost-of-doing-business factor, and the ADM"
    ));
    assert!(d1.contains(
        "Subtracting the applicable charge-off results in a figure constituting the amount for \
         basic state aid"
    ));
    // Which is why the counterfactual holds base cost fixed and substitutes only the local share:
    // that is the one term the opinion's sentence and the plan's formula have in common.
    assert_eq!(regime_diff::ALIGNED.len(), 1);
    assert_eq!(
        regime_diff::ALIGNED[0].predecessor,
        "charge-off-local-share"
    );
}
