//! Six editions of the department's line-by-line explanation, and what they settle.
//!
//! The funding calculator is replaced rather than archived: the live page carries FY2027 alone,
//! and for FY2021 through FY2025 the model itself is gone. This series is what is left of those
//! years — the department's own account of how each component was computed, published annually
//! and still served.
//!
//! It closes a question the corpus has carried as `[open]` since the transportation parameter was
//! entered, and it dates a change that had been visible only as a number moving.
//!
//! # The 180-day multiplier is an annualisation, and the department says so
//!
//! `parameter/transportation-cost-rates` records that the per-mile base is
//! `bus miles × $6.867 × 180` and that **180 appears nowhere in R.C. 3317.0212**, which speaks of
//! "miles driven for school bus service as reported" with no annualisation. The open question was
//! whether the reported figure is a daily one the department annualises by convention.
//!
//! It is. Five consecutive editions state the method:
//!
//! > Calculate the statewide average annual per-mile expenditure for each district **based on 180
//! > days of service** […] The mileage is based on the average number of miles traveled during the
//! > rider count week.
//!
//! Reported miles are a **daily** figure — one week's average — and 180 is the school year it is
//! multiplied out over. The convention is the department's rather than the statute's, which is
//! what the parameter node suspected and could not show.
//!
//! # The density supplement acquired a proration factor in FY2026
//!
//! FY2025 and FY2026 give the same supplement formula except for one term. FY2026 adds
//! `* Proration Factor`, and a sentence that is the plainest statement in any source here of the
//! constraint the corpus had been inferring:
//!
//! > This funding is prorated to remain within amounts provided in the biennial budget.
//!
//! That is an appropriation binding on a formula output, stated by the payer. It is **not** in
//! FY2025, so the year it arrived is known rather than assumed — and special education
//! transportation took its own proration a year later still, at 0.91746 in FY2027 against 1.0 in
//! FY2026. Two transportation lines, two consecutive years, the same pressure.
//!
//! # FY2021 is not missing, it is a different formula
//!
//! The FY2021 edition carries none of this language because it explains H.B. 166's formula, not
//! the Fair School Funding Plan's. It is kept in the series because the series is of the document
//! rather than of the component, and a reader asking why the run starts at FY2022 should find the
//! answer in the file rather than in its absence.

use edfund_core::records::records;

/// The committed extract.
const FIXTURE: &str = include_str!("../fixtures/dew-sfpr-line-by-line.txt");

/// The editions that explain the Fair School Funding Plan's transportation component.
const FSFP_YEARS: &[&str] = &[
    "sfpr-line-by-line-fy22",
    "sfpr-line-by-line-fy23",
    "sfpr-line-by-line-fy24",
    "sfpr-line-by-line-fy25",
    "sfpr-line-by-line-fy26",
];

/// Six editions, oldest first, and none of them empty.
///
/// The ordering is asserted because the file's own header promises it and because the registry's
/// source order has no reason to be chronological.
#[test]
fn the_series_runs_fy2021_to_fy2026_oldest_first() {
    let ids: Vec<&str> = records(FIXTURE).map(|r| r.id).collect();
    assert_eq!(
        ids,
        vec![
            "sfpr-line-by-line-fy21",
            "sfpr-line-by-line-fy22",
            "sfpr-line-by-line-fy23",
            "sfpr-line-by-line-fy24",
            "sfpr-line-by-line-fy25",
            "sfpr-line-by-line-fy26",
        ],
        "six editions, oldest first; FY2027 is not published"
    );

    for record in records(FIXTURE) {
        assert!(
            record.body.len() > 20_000,
            "{}: {} bytes of text, which is too little for a thirty-page specification — \
             the likely cause is a PDF that rendered rather than extracted",
            record.id,
            record.body.len()
        );
    }
}

/// The 180-day multiplier is the department annualising a daily figure, in five editions.
///
/// This is what closes the parameter node's open question, so it is asserted across every edition
/// that should carry it rather than on the one that was read first.
#[test]
fn every_fsfp_edition_states_that_the_per_mile_rate_is_computed_on_180_days_of_service() {
    for id in FSFP_YEARS {
        let body = records(FIXTURE)
            .find(|r| &r.id == id)
            .unwrap_or_else(|| panic!("{id} is not in the extract"))
            .body;
        assert!(
            body.contains("180 days of service"),
            "{id}: does not state the 180-day basis for the per-mile rate"
        );
        assert!(
            body.contains("rider count week"),
            "{id}: does not say the mileage is a count-week figure, which is what makes the \
             180 an annualisation rather than an unexplained constant"
        );
    }
}

/// The proration on the density supplement arrives in FY2026 and is absent in FY2025.
///
/// A dated change rather than a standing fact, which is the whole reason the series is committed
/// rather than only its latest edition.
#[test]
fn the_density_supplement_is_prorated_from_fy2026_and_was_not_before() {
    let body_of = |id: &str| {
        records(FIXTURE)
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("{id} is not in the extract"))
            .body
    };

    let fy25 = body_of("sfpr-line-by-line-fy25");
    let fy26 = body_of("sfpr-line-by-line-fy26");

    assert!(
        fy26.contains("prorated to remain within"),
        "FY2026 should state that the transportation supplement is prorated to the appropriation"
    );
    assert!(
        !fy25.contains("prorated to remain within"),
        "FY2025 should carry no such proration; if it now does, the change is older than \
         FY2026 and the corpus's account of when the appropriation began binding is wrong"
    );

    // The formula line itself gained the term, not merely the surrounding prose.
    assert!(
        fy26.contains("* 0.55 * Proration Factor"),
        "FY2026's supplement formula should carry the proration factor as a term"
    );
    assert!(
        !fy25.contains("Proration Factor"),
        "FY2025's supplement formula should not"
    );
}

/// FY2021 explains a different formula, and that is why it carries none of the above.
///
/// Asserted so the gap reads as a fact about the year rather than as a failed extraction.
#[test]
fn the_fy2021_edition_explains_hb166_rather_than_the_fair_school_funding_plan() {
    let fy21 = records(FIXTURE)
        .find(|r| r.id == "sfpr-line-by-line-fy21")
        .expect("FY2021 is in the extract")
        .body;

    assert!(
        fy21.contains("H. B. 166") || fy21.contains("H.B. 166"),
        "FY2021 should name the act it explains"
    );
    assert!(
        !fy21.contains("180 days of service"),
        "FY2021 predates the component, so finding the language here would mean the \
         transportation method is older than the plan and the series proves it"
    );
}
