//! The three supplements outside foundation funding, before the biennium the corpus computes.
//!
//! `parameter/enrolment-supplement-amounts` and `parameter/performance-supplement-rate` are the
//! corpus's two **uncodified** parameters: their authority is a budget act's temporary law rather
//! than a section of the Revised Code. Both record the consequence the same way — an uncodified
//! parameter has no version archive, so a prior value is not retrievable by the route a codified
//! one offers — and both name the LSC greenbook as where an earlier rate would be.
//!
//! It is, and the earlier rates are not what either node expects.
//!
//! | node | asks for | the answer |
//! |---|---|---|
//! | `enrolment-supplement-amounts` | the FY2022-25 values | zero: the appropriation was cut to $0 |
//! | `enrolment-supplement-amounts` | the enrolled act's text | FY2020-21's is H.B. 166's, and it is a taper |
//! | `performance-supplement-rate` | the FY2022-25 values | none, and the FY2016-19 predecessor was equalised |
//!
//! # What the history changes about the readings
//!
//! `enrolment-supplement-amounts` argues that a taper "would have made the same total payment a
//! continuous function of growth" where the 3% cliff makes it a step. Ohio ran that taper: the
//! FY2020-21 supplement paid on the growth *percentage* and admitted every district that grew at
//! all. The cliff is not the only way the payment has been written; it is the second way.
//!
//! `performance-supplement-rate` calls the supplement "the only component of Ohio's funding
//! formula that pays on a measured outcome", flagged `[inference]` on the "only". It is the only
//! current one. FY2016 through FY2019 had two — a graduation bonus and a third grade reading
//! bonus — and both were multiplied by the state share index, which is the wealth equaliser the
//! present supplement does without. Ohio has paid on outcomes twice and equalised it once.

mod common;

use project::greenbook::{self, Greenbook};
use project::ledger::appropriations;

/// The FY2016-17 budget's analysis — Am. Sub. H.B. 64 of the 131st, which added the two bonuses.
fn bonuses() -> Greenbook<'static> {
    greenbook::greenbook("hb64")
}

/// The FY2020-21 budget's — Am. Sub. H.B. 166 of the 133rd, which created the growth supplement.
fn created() -> Greenbook<'static> {
    greenbook::greenbook("hb166")
}

/// Every committed amount for one appropriation line, by fiscal year, enacted where both exist.
fn enacted_series(line_item: &str) -> Vec<(u16, f64)> {
    let mut out: Vec<(u16, f64)> = appropriations::lines()
        .into_iter()
        .filter(|line| line.line_item == line_item && line.kind == "enacted")
        .map(|line| (line.fiscal_year, line.amount))
        .collect();
    out.sort_by_key(|(year, _)| *year);
    out.dedup();
    out
}

/// The growth supplement was a taper before it was a cliff, and the taper had no threshold.
///
/// H.B. 166 paid every district with at least 50 pupils that grew *at all* over three years, on
/// the growth percentage itself. So the payment rose continuously with growth and the eligibility
/// test was zero. The present supplement pays a flat $250 on the whole enrolment of a district
/// past 3% and nothing below it — a different instrument under the same name.
#[test]
fn the_growth_supplement_was_a_taper_before_it_was_a_cliff() {
    assert_eq!(created().first_fiscal_year(), 2020);
    assert!(created().flat().contains(
        "The budget provides $15.5 million in FY 2020 and $23.0 million in FY 2021 to make an \
         additional payment to each school district with a student enrollment of at least 50 that \
         experienced an average annual percentage change in its enrollment between FY 2016 and \
         FY 2019 that is greater than zero. The payment is equal to a district\u{2019}s percentage \
         times 100 times the number of students enrolled in the district in FY 2019 times a per \
         pupil amount of $20 for FY 2020 and $30 for FY 2021."
    ));

    // No later analysis restates it, which is what makes FY2020-21 the whole of the taper's life.
    for book in greenbook::greenbooks() {
        if book.first_fiscal_year() > 2020 {
            assert!(
                !book
                    .flat()
                    .contains("an average annual percentage change in its enrollment"),
                "FY{} states the taper as well",
                book.first_fiscal_year()
            );
        }
    }
}

/// And for the four years between the two versions the answer is zero.
///
/// ALI 200636 carried $15.5m and $23.0m and was then cut to nothing, in both years of the
/// biennium that enacted the plan. So `the FY2022-25 values` is not a rate the corpus is missing:
/// the payment did not exist. What survived is the FY2021 amount, frozen into a guarantee — the
/// formula transition supplement adds each district's FY2021 growth supplement to the floor it
/// holds them at, which pays the last year of the programme forever and the programme never
/// again.
#[test]
fn the_growth_supplement_was_abolished_and_its_last_year_made_permanent() {
    let series = enacted_series("200636");
    assert_eq!(
        series,
        vec![
            (2008, 700_000.0),
            (2009, 700_000.0),
            (2010, 700_000.0),
            (2011, 0.0),
            (2020, 15_500_000.0),
            (2021, 23_000_000.0),
            (2022, 0.0),
            (2023, 0.0),
        ]
    );

    assert!(greenbook::greenbook("hb110").flat().contains(
        "This supplement guarantees that a district\u{2019}s foundation aid in FY 2022 and FY 2023 \
         does not fall below the sum of the foundation aid that was calculated for the district \
         for FY 2021 before budget reductions plus the district\u{2019}s student wellness and \
         success funds for FY 2021 plus the district\u{2019}s enrollment growth supplement for \
         FY 2021."
    ));
    // Still in the base two bienniums later, which is what makes it a floor rather than a bridge.
    assert!(greenbook::greenbook("hb33").flat().contains(
        "including the district\u{2019}s student wellness and success funds and the \
         district\u{2019}s enrollment growth supplement for that year"
    ));
}

/// The same appropriation line paid an unrelated programme first, which is why an ALI is not a name.
///
/// 200636 was Character Education from FY2007 to FY2010, was zeroed in FY2011, and was reused for
/// the enrollment growth supplement nine years later. A series keyed on the line item alone reads
/// as one programme with a gap; it is two programmes with nothing in common but a number.
#[test]
fn the_line_item_was_a_different_programme_before_it_was_this_one() {
    let titles: Vec<(u16, String)> = appropriations::lines()
        .into_iter()
        .filter(|line| line.line_item == "200636" && line.kind == "enacted")
        .map(|line| (line.fiscal_year, line.title))
        .collect();
    for (year, title) in &titles {
        let expected = if *year <= 2011 {
            "Character Education"
        } else {
            "Enrollment Growth Supplement"
        };
        assert_eq!(title, expected, "FY{year}");
    }
    assert!(titles.iter().any(|(_, t)| t == "Character Education"));
    assert!(titles
        .iter()
        .any(|(_, t)| t == "Enrollment Growth Supplement"));
}

/// The base funding supplement is a twenty-year-old name, and $40 is where it started.
///
/// FY2006 had four base funding supplements, itemised and each tied to a named service, adding to
/// **$40.00** a pupil. They rose to $47.99 in FY2007 and to $50.91 by FY2009, where H.B. 1 froze
/// them for two more years. The name then disappears with the H.B. 59 formula and returns in the
/// current biennium as a single flat $40 with nothing itemised and no service named.
///
/// So the corpus's $40 is the FY2006 total, twenty years on, for a payment that no longer says
/// what it buys.
#[test]
fn the_base_funding_supplement_is_an_old_name_and_an_old_amount() {
    let blue_ribbon = greenbook::greenbook("hb66");
    assert_eq!(blue_ribbon.first_fiscal_year(), 2006);
    assert!(blue_ribbon.flat().contains(
        "Table 4: Base Funding Supplements Per Pupil, FY 2006 and FY 2007 % Change, Supplement \
         FY 2006 FY 2007 FY 2006 - FY 2007 Academic Intervention Services $25.00 $25.50 2.00% \
         Professional Development $3.50 $10.73 206.57% Data-Based Decision Making $5.28 $5.40 \
         2.27% Professional Development \u{2013} Data-Based Decision Making $6.22 $6.36 2.25% \
         Total $40.00 $47.99 19.98%"
    ));
    assert!(common::close(25.00 + 3.50 + 5.28 + 6.22, 40.00));

    assert!(greenbook::greenbook("hb1").flat().contains(
        "sets the base funding supplements as $50.91 for both FY 2010 and FY 2011, which was the \
         amount specified for FY 2009"
    ));

    // Four analyses name it, and none after the FY2012-13 budget.
    let naming: Vec<u16> = greenbook::greenbooks()
        .iter()
        .filter(|book| book.mentions("base funding supplement"))
        .map(Greenbook::first_fiscal_year)
        .collect();
    assert_eq!(naming, vec![2006, 2008, 2010, 2012]);
}

/// Ohio has paid on a measured outcome before, and that time it equalised the payment.
///
/// H.B. 64 added a graduation bonus and a third grade reading bonus, each at 7.5% of the formula
/// amount times a count of pupils who cleared the bar — and each multiplied by the district's
/// **state share index**. H.B. 49 makes the role of that term explicit by removing it: the STEM
/// school version of the same bonus is *"identical to the calculation of the payment for
/// traditional districts except that it does not use the state share index."*
///
/// The present performance supplement pays a flat $13 a pupil per qualifying rating, gross. That
/// is the difference `performance-supplement-rate` measures as a 2.56-fold gradient toward the
/// least-poor fifth of districts, and the difference is a term, not an accident of which
/// districts qualify.
#[test]
fn ohio_paid_on_outcomes_before_and_equalised_it_then() {
    let lsc = bonuses().flat();
    assert_eq!(bonuses().first_fiscal_year(), 2016);
    assert!(lsc.contains(
        "The budget adds two new components based on school district four-year graduation rates \
         and third grade reading proficiency rates in an effort to incentivize performance."
    ));
    assert!(lsc.contains(
        "Graduation bonus = Graduation rate x 0.075 x Formula amount x Graduate count x State \
         share index"
    ));
    assert!(lsc.contains(
        "Third grade reading bonus = Third grade reading proficiency percentage x 0.075 x Formula \
         amount x Number of proficient or higher readers in third grade x State share index"
    ));

    assert!(greenbook::greenbook("hb49").flat().contains(
        "It is identical to the calculation of the payment for traditional districts except that \
         it does not use the state share index."
    ));

    // Two bienniums, and then the Bridge formula, which computed no formula components at all.
    let paying: Vec<u16> = greenbook::greenbooks()
        .iter()
        .filter(|book| book.mentions("graduation bonus"))
        .map(Greenbook::first_fiscal_year)
        .collect();
    assert_eq!(paying, vec![2016, 2018]);
}
