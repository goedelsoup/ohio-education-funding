//! The Catalog's *other* clause, and the `[open]` that said it was not extracted.
//!
//! `program/jon-peterson-special-needs` names the five appropriation lines that pay both
//! foundation aid and the state's scholarships, and closes on: "The Catalog of Budget Line Items
//! would give each of these a per-line history with actuals and a Legal Basis naming the
//! establishing act, and is now retrievable. **Not yet extracted.**"
//!
//! It is extracted, into two committed fixtures, and read by two modules — and one half of the
//! Legal Basis had gone unread anyway. `line_origins::current` takes the parenthetical naming the
//! act that established a line, for a stated reason: an origin does not change. The first clause
//! is the opposite case; it names the Revised Code chapter and act authorising the line *this*
//! biennium, so it changes by design.
//!
//! # What reading it says
//!
//! Ohio replaced its school funding formula with a new chapter of the Revised Code in 2009.
//! **R.C. 3306 reaches exactly one line of the state budget, in exactly three editions**, and
//! appears nowhere else in eighteen. The Evidence-Based Model's whole footprint in the
//! appropriation record is GRF 200550.

use edfund_core::FiscalYear;
use project::ledger::{appropriations, line_origins};

/// **The finding.** A whole funding regime, one line item.
#[test]
fn the_evidence_based_models_chapter_reaches_one_line_in_three_editions() {
    let under = line_origins::lines_under("3306");
    assert_eq!(
        under.keys().copied().collect::<Vec<u16>>(),
        vec![2009, 2010, 2011],
        "the editions citing R.C. 3306"
    );
    for (edition, alis) in &under {
        assert_eq!(
            alis,
            &vec!["200550".to_string()],
            "in the {edition} edition"
        );
    }

    // And it is the formula line, in the general fund.
    let cited: Vec<line_origins::LineAuthority> = line_origins::authority()
        .into_iter()
        .filter(|line| line.chapters.iter().any(|c| c == "3306"))
        .collect();
    assert_eq!(cited.len(), 3);
    assert!(cited
        .iter()
        .all(|line| line.fund == "GRF" && line.name.starts_with("Foundation Funding")));

    // The 2009 and 2010 editions cite the act that created the model. The 2011 edition cites the
    // act that repealed it — H.B. 153 appropriated the formula under the model's own chapter
    // while dismantling it.
    assert_eq!(cited[0].act, "Am. Sub. H.B. 1 of the 128th G.A.");
    assert_eq!(cited[1].act, "Am. Sub. H.B. 1 of the 128th G.A.");
    assert_eq!(cited[2].act, "Am. Sub. H.B. 153 of the 129th G.A.");
    assert_eq!(cited[2].general_assembly, Some(129));
}

/// And the chapter it displaced contracts for exactly those years.
#[test]
fn chapter_3317s_footprint_narrows_under_the_model_and_is_widest_under_the_current_plan() {
    let counts = line_origins::lines_under("3317");
    let at = |edition: u16| {
        counts
            .get(&edition)
            .unwrap_or_else(|| panic!("the {edition} edition"))
            .len()
    };

    // Ten lines before the model, six in each of its three editions.
    assert_eq!((at(2006), at(2008)), (10, 10));
    assert_eq!((at(2009), at(2010), at(2011)), (6, 6, 6));
    // It never returns to ten until the Fair School Funding Plan, and then passes it.
    assert!(
        (2013..=2020).all(|edition| at(edition) < 10),
        "chapter 3317 recovered before FY2021"
    );
    assert_eq!((at(2021), at(2025)), (11, 13));
}

/// **The `[open]`.** Both halves the node asked for are committed and readable.
#[test]
fn the_five_lines_that_pay_the_scholarships_have_a_history_and_an_establishing_act() {
    // The per-line history with actuals, which is what "not yet extracted" denied. Each runs to
    // FY2027, the current biennium's appropriation, and back as far as the line exists.
    for (ali, years, opens) in [
        ("200550", 18, 2008),
        ("200502", 22, 2002),
        ("200604", 16, 2002),
        ("200491", 4, 2024),
        ("200612", 22, 2002),
    ] {
        let history = appropriations::line_history(ali, FiscalYear(2025));
        assert_eq!(history.len(), years, "years of history for {ali}");
        assert_eq!(history.first().expect("years").fiscal_year, opens, "{ali}");
        assert_eq!(history.last().expect("years").fiscal_year, 2027, "{ali}");
        assert!(history.iter().any(|year| year.nominal > 0.0));
    }
    // `200491` is the youngest of the five by twenty-two years, which is the shape of the answer
    // rather than a gap in it: a programme paid from one line established in 1977 and another in
    // 2023 has no single appropriation history to read.

    // And the establishing act for four of the five. The Catalog names none for 200491, which is
    // reported rather than guessed at.
    let origins = line_origins::current();
    let established = |ali: &str| {
        origins
            .iter()
            .find(|o| o.ali == ali)
            .unwrap_or_else(|| panic!("{ali} is in the current edition"))
            .general_assembly
    };
    assert_eq!(established("200550"), Some(126));
    assert_eq!(established("200502"), Some(112));
    assert_eq!(established("200604"), Some(133));
    assert_eq!(established("200612"), Some(122));
    assert_eq!(established("200491"), None);

    // Forty-two years of establishing acts pay one programme's scholarships: the transportation
    // line predates the formula line by fourteen General Assemblies.
    assert_eq!(
        line_origins::convened(112),
        1977,
        "the oldest of the five convened"
    );
    assert_eq!(line_origins::convened(133), 2019, "and the newest");
}

/// The five are not even anchored in the same chapter.
#[test]
fn the_lottery_line_is_authorised_under_the_lottery_chapter_and_not_the_school_funding_one() {
    let authority = line_origins::authority();
    let newest = |ali: &str| {
        authority
            .iter()
            .filter(|line| line.ali == ali)
            .max_by_key(|line| line.edition)
            .unwrap_or_else(|| panic!("{ali} is in the Catalog"))
    };

    for ali in ["200550", "200502", "200604"] {
        assert_eq!(newest(ali).chapters, vec!["3317".to_string()], "{ali}");
    }
    // The lottery fund's foundation line cites R.C. 3770 — the lottery chapter — and not 3317,
    // under the same programme name as the two lines above it.
    assert_eq!(newest("200612").chapters, vec!["3770".to_string()]);
    assert!(newest("200612").name.starts_with("Foundation Funding"));
    // And the support line cites the school-funding chapter beside the casino wagering tax.
    assert_eq!(
        newest("200491").chapters,
        vec!["3317".to_string(), "5753".to_string()]
    );
}

/// The reader itself, on the shape of the fixture rather than on one finding.
#[test]
fn every_edition_is_read_and_a_basis_may_name_no_chapter_at_all() {
    let authority = line_origins::authority();
    assert_eq!(authority.len(), 2008, "rows in the basis fixture");

    let mut editions: Vec<u16> = authority.iter().map(|line| line.edition).collect();
    editions.dedup();
    assert_eq!(editions.len(), 18, "editions, and they arrive in order");
    assert_eq!((editions[0], editions[17]), (2006, 2025));

    // About three quarters of entries cite only session-law sections. Reported as none rather
    // than inferred from an earlier edition, because a line item number is reused.
    let silent = authority
        .iter()
        .filter(|line| line.chapters.is_empty())
        .count();
    assert!(
        silent > authority.len() / 2 && silent < authority.len() * 4 / 5,
        "{silent} of {} name no chapter",
        authority.len()
    );
}
