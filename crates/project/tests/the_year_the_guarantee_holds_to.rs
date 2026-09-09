//! What year the guarantee's base is, read off the section that defines it.
//!
//! `parameter/guarantee-funding-base` states the anchor as "the FY2020 level, the year Ohio froze
//! funding under the Bridge formula", sourced to the department's own plain-English gloss: the
//! guarantee "ensures that districts do not receive less in FY 2026 than what they received in
//! **FY 2020**".
//!
//! The department's sentence is a fair description of what the guarantee *does*. It is not what
//! the statute says the base *is*, and the difference is a quarter of a billion dollars.
//!
//! # The base is a counterfactual FY2020
//!
//! R.C. 3317.02(N)(1)(a)(i) anchors the general funding base to the FY2020 amount under
//! H.B. 166's Section 265.220(A)(1) **"prior to any funding reductions authorized by Executive
//! Order 2020-19D, 'Implementing Additional Spending Controls to Balance the State Budget' issued
//! on May 7, 2020"** — the pandemic mid-year cut. R.C. 3317.019(A)(2) does the same for the
//! transportation base.
//!
//! So the number half of Ohio is funded at is not what districts received in FY2020. It is what
//! they would have received had the cut not happened, and the General Assembly wrote the
//! executive order into the definition in order to reach past it.
//!
//! # And a section the corpus concluded did not exist
//!
//! `parameter/transportation-cost-rates` records that "the FY2021 transportation guarantee — the
//! line that holds 38 districts at their FY2021 payment — is likewise absent from the section",
//! and that it is "alive in the calculator and not in permanent law". It is in permanent law, at
//! R.C. 3317.019(A)(2), which is committed in the same extract and which no test had read. The
//! search missed it because the statute does not call it a guarantee: it is "temporary
//! transitional transportation aid", and the word *guarantee* appears nowhere in the section.

use std::collections::BTreeMap;

use project::statute;

/// The executive order both bases reach past, quoted as the sections quote it.
const SPENDING_CONTROLS: &str = "prior to any funding reductions authorized by Executive Order \
                                 2020-19D, \"Implementing Additional Spending Controls to Balance \
                                 the State Budget\" issued on May 7, 2020";

/// A section's text, flattened across the publisher's line breaks.
fn flat(number: &str) -> String {
    statute::section(number)
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Both funding bases are defined as the FY2020 figure before the pandemic reduction.
#[test]
fn the_funding_base_is_the_fiscal_year_before_the_cut_and_not_the_year() {
    let definitions = flat("3317.02");
    assert!(
        definitions.contains(
            "The amount calculated for the district for fiscal year 2020 under division (A)(1) of \
             Section 265.220 of H.B. 166 of the 133rd general assembly"
        ),
        "the general funding base no longer reaches H.B. 166's FY2020 computation"
    );
    assert!(
        definitions.contains(SPENDING_CONTROLS),
        "the general funding base no longer excludes the executive order's reductions"
    );

    let transitional = flat("3317.019");
    assert!(transitional.contains(
        "The amount calculated for the district for fiscal year 2020 under division (A)(2) of \
         Section 265.220 of H.B. 166 of the 133rd general assembly"
    ));
    assert!(transitional.contains(SPENDING_CONTROLS));

    // One order, named identically in two sections, which is what says it is the same exclusion
    // rather than two drafting accidents.
    assert_eq!(definitions.matches(SPENDING_CONTROLS).count(), 1);
    assert_eq!(transitional.matches(SPENDING_CONTROLS).count(), 1);
}

/// The transportation guarantee is in permanent law under a name that is not "guarantee".
///
/// `parameter/transportation-cost-rates` looked in R.C. 3317.0212, the transportation payment
/// section, and was right that it is not there. It is one section away.
#[test]
fn the_transportation_guarantee_is_in_the_section_the_corpus_did_not_search() {
    let payment = flat("3317.0212");
    assert!(
        !payment.contains("guarantee") && !payment.contains("fiscal year 2020"),
        "the transportation payment section now carries the guarantee and the node's account of \
         where it is not needs redoing"
    );

    let transitional = flat("3317.019");
    assert!(
        !transitional.contains("guarantee"),
        "the section that holds it still does not use the word, which is why a search for it \
         missed the section"
    );
    assert!(transitional.contains(
        "the department shall pay temporary transitional transportation aid to that district"
    ));

    // Three subtractions, each reaching a different year, and two of them reaching provisions
    // that no longer exist.
    assert!(transitional.contains(
        "the district's payment for fiscal year 2019 under division (D)(2) of section 3314.091 of \
         the Revised Code as that division existed prior to September 30, 2021"
    ));
    assert!(transitional.contains(
        "the district's payment under section 3317.0212 of the Revised Code for the fiscal year \
         for which the payment is computed"
    ));
    assert!(
        transitional.contains(
            "If the computation made under division (A)(2) of this section results in a negative \
             number, the district's funding under division (A)(2) of this section shall be zero."
        ),
        "the transportation top-up floors at zero the same way the general one does"
    );
}

/// The statute does not contemplate re-basing. It contemplates expiring.
///
/// `formula-component/temporary-transitional-aid-guarantee` records as open "whether the statute
/// or subsequent budget acts contemplate re-basing". The section says neither word: it is scoped
/// to fiscal years 2026 and 2027 in every division that pays anything, names no later year, and
/// contains no provision for moving the base. What happens after FY2027 is not a re-basing rule
/// but the absence of a section — the same handover
/// [`the_statute_behind_the_weights`](the_statute_behind_the_weights.rs) records across the plan.
///
/// The budget acts are the other half of the question and they answer it the other way.
/// `funding-regime/bridge-formula` records four consecutive acts each re-reading the anchor onto
/// the year before its own biennium; the plan has re-read it none. Re-basing is what Ohio did
/// every two years for a decade and stopped doing in FY2022, which is a fact about practice and
/// not about statutory design.
#[test]
fn the_section_is_scoped_to_one_biennium_and_provides_no_way_to_move_the_base() {
    let transitional = flat("3317.019");
    assert_eq!(
        transitional.matches("fiscal years 2026 and 2027").count(),
        3
    );
    assert!(!transitional.contains("2028"));
    for absent in ["rebase", "re-base", "recalculat", "adjust the funding base"] {
        assert!(
            !transitional.to_lowercase().contains(absent),
            "the section now says {absent:?} and the re-basing question needs re-asking"
        );
    }

    // The one adjustment it does provide runs the other way, and only for a district joining a
    // new joint vocational district.
    assert!(transitional.contains(
        "the department shall adjust, as necessary, the district's funding base, as that term is \
         defined in section 3317.02 of the Revised Code, according to the amounts received by the \
         district in the immediately preceding fiscal year for career-technical education students"
    ));
}

/// How much the statute reaches past, from the corpus's own appropriation series.
///
/// Foundation funding came in **$254,956,620 below its enacted FY2020 appropriation — 3.67%**.
/// That is not ordinary lapse. FY2014 through FY2025, every year outside the two pandemic ones
/// lands within a single percentage point of its appropriation, between −0.39% and +0.92%; the
/// only comparable shortfalls anywhere in the series are FY2008 and FY2009.
///
/// **FY2021 misses too, by 1.06%**, and the two misses are not the same event. FY2020's is a
/// mid-year reduction against a figure already appropriated — which is what an executive order
/// can do and is why the statute names one. FY2021's is smaller and sits under an appropriation
/// that had *already* been cut: its enacted figure is $168m below FY2020's. The General Assembly
/// reached past the first and not the second, and the second is the one the transportation base
/// is labelled for in the department's calculator.
///
/// This is the appropriation and not the guarantee base, so it is the size of the event the
/// statute excludes rather than the size of the exclusion. The exclusion is per district and no
/// committed source carries it. What the appropriation establishes is that the event was real and
/// large, which is what makes the drafting deliberate rather than boilerplate.
#[test]
fn the_year_the_base_excludes_is_the_only_ordinary_year_that_missed_its_appropriation() {
    let mut by_year: BTreeMap<u16, BTreeMap<String, f64>> = BTreeMap::new();
    for line in project::ledger::appropriations::lines() {
        if line.line_item == "200550" {
            by_year
                .entry(line.fiscal_year)
                .or_default()
                .insert(line.kind.clone(), line.amount);
        }
    }

    let gap = |year: u16| -> Option<(f64, f64)> {
        let row = by_year.get(&year)?;
        let (enacted, actual) = (row.get("enacted")?, row.get("actual")?);
        Some((actual - enacted, (actual - enacted) / enacted * 100.0))
    };

    let (dollars, percent) = gap(2020).expect("FY2020 carries both an enacted and an actual");
    assert!((dollars + 254_956_619.56).abs() < 0.01, "{dollars:.2}");
    assert!((percent + 3.67).abs() < 0.005, "{percent:.4}%");

    // FY2021 is the other pandemic year and misses by a third as much.
    let (_, after) = gap(2021).expect("FY2021 carries both");
    assert!((after + 1.06).abs() < 0.005, "{after:.4}%");
    assert!(
        percent.abs() > 3.0 * after.abs(),
        "FY2020 is no longer several times the size of FY2021's miss"
    );

    // And the FY2021 appropriation was itself cut before the year began, which is why its actual
    // sits closer to it: the reduction had already been enacted rather than ordered.
    let enacted = |year: u16| by_year[&year]["enacted"];
    assert!(enacted(2021) < enacted(2020) - 168_000_000.0);

    // Every year of the two formula regimes this corpus computes over, outside the two.
    for year in 2014..=2025 {
        if year == 2020 || year == 2021 {
            continue;
        }
        let Some((_, other)) = gap(year) else {
            continue;
        };
        assert!(
            other.abs() < 1.0,
            "FY{year} misses its appropriation by {other:.2}%, which makes FY2020 unremarkable"
        );
    }
}
