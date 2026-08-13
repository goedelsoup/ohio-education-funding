//! Which act created each appropriation line, and the one piece of arithmetic behind it.
//!
//! [`the_catalog_line_item_series`](the_catalog_line_item_series.rs) holds what each line was
//! given. This holds where each line came from — the `originally established by` clause the
//! Catalog prints in every entry's legal basis.
//!
//! # The only inference in the module, and why it is checked rather than asserted
//!
//! The Catalog names a General Assembly and never a date, so a reader is told a line dates from
//! "the 112th" and left there. Ohio's General Assemblies are consecutive and biennial, so the
//! mapping to a year is arithmetic — but an arithmetic mapping with a wrong anchor is wrong
//! everywhere at once and looks right, which is the failure this repository keeps finding.
//!
//! So the anchor is not asserted. Every act this corpus holds carrying both a General Assembly and
//! a year is checked against it below, and they span fifty years — if the constant were off by one
//! biennium, all eight would fail rather than none.

use std::collections::BTreeSet;

use project::line_origins::{convened, current};

/// Acts the corpus holds with both facts attached, as `(general assembly, year convened)`.
///
/// Read off the `legislation` node filenames and their own text: `hb-920-1976` is of the 111th,
/// `hb-96-2025` is of the 136th. A General Assembly convenes in the odd year, and an act of it may
/// be signed in either of its two years — so the check below allows the act's year to be the
/// convening year or the one after it.
const ANCHORS: [(u16, u16); 8] = [
    (111, 1976), // H.B. 920, the tax reduction factors
    (124, 2001), // H.B. 94, the post-DeRolph II formula act
    (126, 2005), // H.B. 66, which established Foundation Funding
    (128, 2009), // H.B. 1, the Evidence-Based Model
    (129, 2011), // H.B. 153, the Bridge formula
    (134, 2021), // H.B. 110, the Fair School Funding Plan
    (135, 2023), // H.B. 33, the universal EdChoice expansion
    (136, 2025), // H.B. 96, the current budget
];

#[test]
fn the_general_assembly_mapping_agrees_with_every_act_the_corpus_holds() {
    /*
     * Eight acts across fifty years. The mapping is `1753 + 2n`, and the test that matters is not
     * that it reproduces one of these but that it reproduces all of them — a constant off by a
     * single biennium would satisfy none, and one off by fifty years would satisfy none either.
     */
    for (general_assembly, year) in ANCHORS {
        let start = convened(general_assembly);
        assert!(
            year == start || year == start + 1,
            "the {general_assembly}th convened in {start}, which cannot carry an act of {year}"
        );
    }
}

#[test]
fn the_mapping_is_biennial_and_increasing() {
    // Two years per General Assembly, and no two share a convening year.
    for n in 100..140u16 {
        assert_eq!(convened(n + 1), convened(n) + 2);
    }
    assert_eq!(convened(136), 2025, "the current General Assembly moved");
}

#[test]
fn the_current_edition_carries_every_line_once() {
    /*
     * Keyed by line item, so a fund code reformatting cannot produce the same line twice — `200612`
     * is printed under fund `017` in some editions and `7017` in others, which is exactly the shape
     * of duplicate this would otherwise admit.
     */
    let lines = current();
    assert!(lines.len() > 50, "only {} line items", lines.len());
    let alis: BTreeSet<&str> = lines.iter().map(|l| l.ali.as_str()).collect();
    assert_eq!(alis.len(), lines.len(), "a line item appears twice");
}

#[test]
fn about_half_the_lines_name_an_establishing_act_and_the_rest_say_nothing() {
    /*
     * The honest shape of this data. Roughly half of the current edition's entries carry an
     * `originally established by` clause and the rest cite only their current authority.
     *
     * The absent ones are reported as unknown rather than filled from an earlier edition carrying
     * the same number, because a line item number is reused: `200604` names three different
     * programmes across three funds in this series. Inheriting an origin down a number would
     * attribute one programme's founding act to another's.
     */
    let lines = current();
    let dated = lines
        .iter()
        .filter(|l| l.general_assembly.is_some())
        .count();
    assert!(dated > 30, "only {dated} lines name an establishing act");
    assert!(
        dated < lines.len(),
        "every line now names one, which this data has never done"
    );
    // And where an act is named, it parses into a plausible General Assembly.
    for line in lines.iter().filter(|l| l.general_assembly.is_some()) {
        let ga = line.general_assembly.expect("filtered");
        assert!((100..=136).contains(&ga), "{} claims the {ga}th", line.ali);
        assert!(
            line.established_by.contains("G.A."),
            "{} has a general assembly but no act text",
            line.ali
        );
    }
}

#[test]
fn the_department_is_accreted_rather_than_designed() {
    /*
     * The finding worth putting on a page. The lines the department is funded through were created
     * by acts spanning roughly half a century — the oldest still-live line predates every funding
     * regime this corpus documents, including the one struck down in DeRolph.
     */
    let lines = current();
    let live: Vec<_> = lines.iter().filter(|l| !l.discontinued).collect();
    assert!(live.len() > 40, "only {} live lines", live.len());

    let assemblies: BTreeSet<u16> = live.iter().filter_map(|l| l.general_assembly).collect();
    assert!(
        assemblies.len() > 10,
        "live lines come from only {} General Assemblies",
        assemblies.len()
    );

    let oldest = *assemblies.iter().next().expect("some dated line");
    let newest = *assemblies.iter().next_back().expect("some dated line");
    assert!(
        convened(newest) - convened(oldest) > 40,
        "the department's lines span only {} years",
        convened(newest) - convened(oldest)
    );
    // The oldest surviving line predates DeRolph I, decided in 1997.
    assert!(convened(oldest) < 1997, "{}", convened(oldest));
}

#[test]
fn discontinued_is_the_publishers_label_and_not_a_finding_about_abolition() {
    /*
     * The Catalog marks lines discontinued and the extract carries the mark, but a line folded
     * into another is discontinued too. `state-foundation-aid` holds the open question of whether
     * the department's disappearing lines were abolished or consolidated, and this cannot settle
     * it — the test exists so nobody reads the flag as if it had.
     */
    let lines = current();
    let discontinued = lines.iter().filter(|l| l.discontinued).count();
    assert!(discontinued > 0, "no line is marked discontinued");
    assert!(
        discontinued < lines.len() / 2,
        "{discontinued} of {} lines are discontinued, which is not a live budget",
        lines.len()
    );
    // A discontinued line still names where it came from, which is the point of carrying them.
    assert!(
        lines
            .iter()
            .any(|l| l.discontinued && l.general_assembly.is_some()),
        "no discontinued line names its establishing act"
    );
}
