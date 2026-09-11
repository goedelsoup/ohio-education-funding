//! The transportation guarantee's base, read off the two sources that define it rather than the
//! column that labels it.
//!
//! The calculator heads the column `[F1] FY21 Trans Funding Base`, and the corpus took the year
//! from the header: `formula-component/fsfp-transportation` calls the line "a **third** mechanism
//! anchored to FY2021 … a different base", the class README says Ohio "holds districts harmless
//! against FY2021 in three separate places", and the figure manifest labels the district count
//! "transportation's own FY2021 guarantee".
//!
//! Two sources define the base and neither names FY2021.
//!
//! # The statute
//!
//! R.C. 3317.019(A)(2) — established at
//! [`the_year_the_guarantee_holds_to`](the_year_the_guarantee_holds_to.rs) — pays *temporary
//! transitional transportation aid* on the FY2020 amount under H.B. 166's Section 265.220(A)(2),
//! **before** Executive Order 2020-19D's pandemic reductions, less the district's FY2019 payment
//! under a division of R.C. 3314.091 that no longer exists.
//!
//! # And the department, five years running
//!
//! Every edition of the line-by-line explanation from FY2022 to FY2026 gives the same definition:
//!
//! > Temporary Transitional Aid Guarantee = (FY 2019 Capped Transportation – Community/STEM
//! > School Transportation) – FY 20XX Total Transportation Funding
//!
//! and glosses it as ensuring the district "does not receive less … than what it received in
//! **FY 2020**". Those are the statute's two terms in the department's words: the FY2020 payment
//! was the FY2019 capped amount, and the subtraction of community and STEM transportation is the
//! repealed R.C. 3314.091(D)(2) line.
//!
//! This does not prove the heading false, and the tests below do not claim it. Under H.B. 166
//! both FY2020 and FY2021 were held at FY2019, so the column may name the same dollar the section
//! reaches back for — `parameter/transportation-cost-rates` keeps that identification unfilled,
//! and it stays unfilled, because settling it needs a per-district figure for one of those years.
//! What the five editions add is a **second definition agreeing with the statute**, which is
//! enough to retire "a different base" and "a clause of this formula" and not enough to retire
//! the heading.
//!
//! # So Ohio anchors three hold-harmlesses to two years, not three
//!
//! The guarantee and transportation's own top-up are the two divisions of R.C. 3317.019 and both
//! reach past the same executive order to FY2020. Only the formula transition supplement is on
//! FY2021 — and the greenbook says so, "based on FY 2021 levels, including the district's student
//! wellness and success funds". Two of the three lines are labelled `FY21` in the workbook and
//! only one of those labels is right.

use edfund_core::records::records;

use project::statute;

/// The department's own account of its own method.
const SFPR: &str = include_str!("../fixtures/dew-sfpr-line-by-line.txt");

/// LSC's greenbook for H.B. 96, which names each guarantee's base year in one paragraph.
const GREENBOOK: &str = include_str!("../fixtures/dew-greenbook.txt");

/// The editions that explain the Fair School Funding Plan's transportation component.
const FSFP_YEARS: [(&str, u16); 5] = [
    ("sfpr-line-by-line-fy22", 2022),
    ("sfpr-line-by-line-fy23", 2023),
    ("sfpr-line-by-line-fy24", 2024),
    ("sfpr-line-by-line-fy25", 2025),
    ("sfpr-line-by-line-fy26", 2026),
];

/// One record's text, flattened across the publisher's line breaks.
fn flat(body: &str) -> String {
    body.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// An edition of the line-by-line explanation, flattened.
fn edition(id: &str) -> String {
    flat(
        records(SFPR)
            .find(|record| record.id == id)
            .unwrap_or_else(|| panic!("no edition {id}"))
            .body,
    )
}

#[test]
fn five_editions_define_the_base_the_same_way_and_none_of_them_says_fy2021() {
    for (id, year) in FSFP_YEARS {
        let text = edition(id);
        assert!(
            text.contains(&format!(
                "Temporary Transitional Aid Guarantee = (FY 2019 Capped Transportation \u{2013} \
                 Community/STEM School Transportation) \u{2013} FY {year} Total Transportation \
                 Funding"
            )),
            "{id} states the base differently"
        );
        assert!(
            text.contains(&format!(
                "ensures the district does not receive less in FY {year} than what it received in"
            )),
            "{id}"
        );
        // The gloss names FY2020 on the same line as the formula, in every edition.
        let at = text
            .find("Temporary Transitional Aid Guarantee for Transportation")
            .expect("the paragraph");
        let paragraph = &text[at..at + 400];
        assert!(paragraph.contains("FY 2020"), "{id}: {paragraph}");
        assert!(
            !paragraph.contains("FY 2021"),
            "{id} names FY2021 in the definition: {paragraph}"
        );
    }
}

#[test]
fn the_departments_definition_is_the_statutes_two_terms() {
    let section = flat(statute::section("3317.019").body);
    // The statute's first term is the FY2020 computation; the department calls the same thing the
    // FY2019 capped amount, because that is what FY2020 was paid on.
    assert!(section.contains(
        "The amount calculated for the district for fiscal year 2020 under division (A)(2) of \
         Section 265.220 of H.B. 166 of the 133rd general assembly"
    ));
    // Its second is the community and STEM transportation the department subtracts by name.
    assert!(section.contains(
        "the district's payment for fiscal year 2019 under division (D)(2) of section 3314.091 of \
         the Revised Code as that division existed prior to September 30, 2021"
    ));
    assert!(
        !section.contains("fiscal year 2021"),
        "the section that defines the base now names FY2021"
    );
}

#[test]
fn a_sentence_copied_forward_for_three_years_carries_its_own_slip() {
    // Corroboration of the same habit the `-FY21` column headers are an instance of. FY2022
    // through FY2024 read "less in FY 20XX than what it received in what districts received in
    // FY 2020"; FY2025 repairs it. Five editions, one sentence, edited once in four years.
    for (id, _) in FSFP_YEARS.iter().take(3) {
        assert!(
            edition(id).contains("than what it received in what districts received in FY 2020"),
            "{id} no longer carries the doubled clause"
        );
    }
    for (id, _) in FSFP_YEARS.iter().skip(3) {
        assert!(
            edition(id).contains("than what it received in FY 2020"),
            "{id}"
        );
        assert!(
            !edition(id).contains("what it received in what districts received in"),
            "{id} still carries the doubled clause"
        );
    }
}

#[test]
fn the_greenbook_names_two_base_years_across_the_three_mechanisms() {
    let text = flat(GREENBOOK);
    assert!(text.contains("temporary transitional aid (based on FY 2020 funding)"));
    assert!(text.contains("the formula transition supplement (based on FY 2021 levels"));
    assert!(text.contains(
        "temporary transitional transportation aid into FY 2026 and FY 2027, which is a separate \
         guarantee based on FY 2020 funding that applies only to a district\u{2019}s pupil \
         transportation aid"
    ));
}

/// Both weighting schedules are statutory, and they are the same two multiples.
///
/// `formula-component/fsfp-transportation` recorded "whether the rider weights and the guarantee
/// are statutory has not been separated out here". The rider weights are R.C. 3317.0212(E)(1)(a);
/// the *mile* weights are (E)(1)(b), the same 1.5 and 2.0 over "the number of miles driven". The
/// crate had shown the department applies them — `bus_miles = public + 2 × nonpublic + 1.5 ×
/// community`, exactly, on every district — without matching that arithmetic to the section that
/// requires it.
#[test]
fn the_section_puts_the_same_two_weights_on_both_bases() {
    let section = flat(statute::section("3317.0212").body);
    for (multiple, kind) in [("1.5", "community schools"), ("2.0", "nonpublic schools")] {
        assert!(
            section.contains(&format!(
                "{multiple} times the statewide transportation cost per student"
            )),
            "the rider base no longer weights {kind} at {multiple}"
        );
        assert!(
            section.contains(&format!(
                "{multiple} times the statewide transportation cost per mile"
            )),
            "the mile base no longer weights {kind} at {multiple}"
        );
    }
    assert!(section.contains(
        "2.0 times the statewide transportation cost per mile times the number of miles driven \
         for school bus service as reported for qualifying riders for the current fiscal year who \
         are enrolled in nonpublic schools"
    ));

    // And the guarantee is not here, which is why a search of this section found nothing.
    assert!(!section.contains("temporary transitional transportation aid"));
}
