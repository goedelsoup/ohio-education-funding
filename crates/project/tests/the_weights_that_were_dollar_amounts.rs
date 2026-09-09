//! Where the plan's fifteen categorical weights came from, which is not a costing.
//!
//! Three `parameter/` nodes ask the same question and give the same reason for not answering it:
//!
//! | node | field | why |
//! |---|---|---|
//! | `special-education-category-multiples` | the multiples before FY2022 | "prior versions of R.C. 3317.013" |
//! | `english-learner-category-multiples` | the multiples before FY2026 | "Prior versions of R.C. 3317.016" |
//! | `career-technical-category-multiples` | the multiples before FY2026 | "Prior versions of R.C. 3317.014" |
//!
//! Every one of them is malformed, and in the same way. Before FY2022 there were no multiples:
//! each category carried a **dollar amount per pupil**, and the Fair School Funding Plan
//! converted the schedules by dividing them by a base cost. The answer is not a series of
//! earlier weights. It is that these weights are the earlier dollars, and the arithmetic of the
//! conversion says what each schedule was actually given.
//!
//! # What the conversion shows
//!
//! - **English learner** — the three weights are the per-pupil amounts that had stood since
//!   FY2015, over the FY2022 base cost. To the dollar. The schedule was carried, not repriced.
//! - **Career-technical** — the five categories and associated services are the FY2017 amounts
//!   over a single divisor, every one of them to the rounding of four decimals. Also carried.
//! - **Special education** — the six are the 2009 Evidence-Based Model's six, multiplied by one
//!   constant of 0.8379. That is an 11.1% rise on the FY2017 dollars, applied identically to
//!   every category. One scalar, six categories.
//!
//! And the 2009 six are the 2001 six with a single category doubled. So the ratios Ohio prices
//! disability with today are the ratios it set in the 124th General Assembly, carried through
//! four regimes, re-expressed twice, and repriced never. `special-education-category-multiples`
//! records that no source it holds prices a category; the stronger fact is that no *regime* has,
//! and the `[open]` stands with a much shorter list of places left to look.
//!
//! # What a greenbook is evidence of
//!
//! LSC's account of an act as enrolled — not the act, and where they differ the act governs. The
//! implied divisors below are computed from LSC's own published tables and are the plan's
//! arithmetic rather than a figure any document states; where one is stated the assertion says
//! so.

mod common;

use project::greenbook::{self, Greenbook};
use project::panel::{
    CTE_ASSOCIATED_WEIGHT, CTE_WEIGHTS, ENGLISH_LEARNER_WEIGHTS, SPECIAL_EDUCATION_WEIGHTS,
};

/// LSC's analysis of the act that enacted the plan — Am. Sub. H.B. 110 of the 134th.
fn plan() -> Greenbook<'static> {
    greenbook::greenbook("hb110")
}

/// The statewide average base cost per pupil in the plan's first year, as LSC states it.
///
/// The multiplicand every special education and English learner weight is applied to. Read out
/// of the analysis rather than written here, so that a re-extract which loses the sentence fails
/// instead of agreeing with a number nobody can find.
fn base_cost_fy2022() -> f64 {
    let text = plan().flat();
    assert!(text.contains(
        "The statewide average base cost per pupil, which is used in the calculation of a number \
         of other formula components, is estimated to be $7,202 in FY 2022."
    ));
    7_202.0
}

/// The single divisor a schedule of dollar amounts implies, and how far the six disagree on it.
///
/// Returns the mean and the relative spread. A schedule that was converted by dividing every
/// amount by one number leaves a spread at the size of the rounding — a few parts in ten
/// thousand for figures published to four decimals. A schedule that was re-derived category by
/// category does not.
fn implied_divisor(dollars: &[f64], weights: &[f64]) -> (f64, f64) {
    assert_eq!(dollars.len(), weights.len());
    let each: Vec<f64> = dollars.iter().zip(weights).map(|(d, w)| d / w).collect();
    let mean = each.iter().sum::<f64>() / each.len() as f64;
    let lo = each.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = each.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (mean, (hi - lo) / mean)
}

/// The Evidence-Based Model's six special education weights, as H.B. 1's own table prints them.
const EVIDENCE_BASED_WEIGHTS: [f64; 6] = [0.2906, 0.7374, 1.7716, 2.3643, 3.2022, 4.7205];
/// The six the Evidence-Based Model replaced, from the same table's "Prior Law" column.
const FOUNDATION_WEIGHTS: [f64; 6] = [0.2892, 0.3691, 1.7695, 2.3646, 3.1129, 4.7342];
/// The six special education per-pupil amounts in FY2017, the last year they moved.
const SPECIAL_EDUCATION_FY2017: [f64; 6] =
    [1_578.0, 4_005.0, 9_622.0, 12_841.0, 17_390.0, 25_637.0];
/// The three English learner per-pupil amounts in FY2015, the last year they moved.
const ENGLISH_LEARNER_FY2015: [f64; 3] = [1_515.0, 1_136.0, 758.0];
/// The five career-technical amounts and associated services in FY2017, the last year they moved.
const CAREER_TECHNICAL_FY2017: [f64; 6] = [5_192.0, 4_921.0, 1_795.0, 1_525.0, 1_308.0, 245.0];

/// Not one of the fifteen has moved since the plan's first year.
///
/// H.B. 110 states each weight inside the formula line for its category. Every one is the
/// constant this workspace computes FY2027 with, so the three nodes' "before FY2026" has the
/// same answer the DPIA amount had: there is no series to find.
#[test]
fn not_one_of_the_fifteen_weights_has_moved_since_the_plan_began() {
    let lsc = plan().flat();
    assert_eq!(plan().first_fiscal_year(), 2022);

    for (category, weight) in SPECIAL_EDUCATION_WEIGHTS.iter().enumerate() {
        let stated = format!(
            "Category {} special education additional aid = (District\u{2019}s category {} \
             special education enrolled ADM) x (Statewide average base cost per pupil) x {weight:.4} \
             x (District\u{2019}s state share percentage)",
            category + 1,
            category + 1
        );
        assert!(lsc.contains(&stated), "FY2022 no longer states {weight}");
    }
    for (category, weight) in ENGLISH_LEARNER_WEIGHTS.iter().enumerate() {
        let stated = format!(
            "Category {} English learner additional aid = (District\u{2019}s category {} English \
             learner enrolled ADM) x (Statewide average base cost per pupil) x {weight:.4} x \
             (District\u{2019}s state share percentage)",
            category + 1,
            category + 1
        );
        assert!(lsc.contains(&stated), "FY2022 no longer states {weight}");
    }
    for (category, weight) in CTE_WEIGHTS.iter().enumerate() {
        let stated = format!(
            "x (Statewide average career-technical base cost per pupil) x {weight:.4} x \
             (District\u{2019}s state share percentage)"
        );
        assert!(
            lsc.contains(&stated),
            "FY2022 no longer states career-technical category {}",
            category + 1
        );
    }
    assert!(lsc.contains(&format!(
        "Career-technical education associated services aid = (District\u{2019}s career-technical \
         education enrolled ADM) x (Statewide average career-technical base cost per pupil) x \
         {CTE_ASSOCIATED_WEIGHT:.4} x (District\u{2019}s state share percentage)"
    )));
}

/// Before the plan they were not multiples at all, and LSC says so twice.
///
/// Which is why "the multiples before FY2026" has no answer in the form the nodes expect. The
/// FY2014 analysis prints a table of dollars per pupil for each of the three schedules.
#[test]
fn before_the_plan_they_were_dollar_amounts_and_the_analysis_says_so() {
    let lsc = plan().flat();
    assert!(lsc.contains(
        "The budget also replaces per-pupil amounts for each of the three categories with weights \
         that are multiplied by the statewide average base cost per pupil"
    ));
    assert!(lsc.contains(
        "The budget replaces per-pupil amounts for each of the five career-technical education \
         categories and the associated services funds with weights"
    ));

    // And the schedules they replaced, in the first analysis that prints them.
    let earlier = greenbook::greenbook("hb59").flat();
    assert_eq!(greenbook::greenbook("hb59").first_fiscal_year(), 2014);
    assert!(earlier.contains("1 Speech only $1,503 $1,517"));
    assert!(earlier.contains(
        "1 LEP students in U.S. schools for no more than 180 days and not $1,500 $1,515"
    ));
    assert!(earlier.contains("3 Career-based intervention programs $1,650 $1,660"));
}

/// The English learner weights are the frozen dollars over the new base, to the dollar.
///
/// $1,515 was set for FY2015 and did not move again: H.B. 64 froze it at the FY2015 amount and
/// H.B. 49 at the FY2017 one, and FY2020-21 computed no formula at all. The plan divided that
/// same $1,515 by $7,202 and called the quotient a weight. English learner funding was carried
/// across the regime change at exactly its 2015 value.
#[test]
fn the_english_learner_weights_are_the_frozen_dollars_over_the_new_base() {
    assert!(greenbook::greenbook("hb64").flat().contains(
        "The per-pupil amounts and other various factors for economically disadvantaged funds, \
         gifted education funds, and limited English proficiency funds remain unchanged from \
         those used in FY 2015."
    ));
    assert!(greenbook::greenbook("hb49").flat().contains(
        "The per-pupil amounts and other various factors for special education additional aid, \
         K-3 literacy funds, career-technical education funds, economically disadvantaged funds, \
         gifted education funds, and limited English proficiency funds remain unchanged from \
         those used in FY 2017."
    ));

    let base = base_cost_fy2022();
    for (weight, dollars) in ENGLISH_LEARNER_WEIGHTS.iter().zip(ENGLISH_LEARNER_FY2015) {
        let paid = weight * base;
        assert!(
            (paid - dollars).abs() < 0.5,
            "{weight} x {base} is {paid}, against the {dollars} it replaced"
        );
    }

    let (divisor, spread) = implied_divisor(&ENGLISH_LEARNER_FY2015, &ENGLISH_LEARNER_WEIGHTS);
    assert!(
        spread < 1e-3,
        "the three imply different divisors: {spread}"
    );
    assert!(
        (divisor - base).abs() < 2.0,
        "implied {divisor}, stated {base}"
    );
}

/// The career-technical schedule is one divisor away from the amounts it replaced.
///
/// Five categories and associated services, all six of them the FY2017 amount over the same
/// number, to a few parts in ten thousand — which is the rounding of a weight published to four
/// decimals and not the residue of six separate decisions.
///
/// The divisor is not the base cost the other two schedules use. These weights multiply the
/// career-technical base cost, which the analysis defines by cross-reference and never states,
/// so the number below is what the plan's own arithmetic implies it was in FY2022.
#[test]
fn the_career_technical_weights_are_one_divisor_from_the_amounts_they_replaced() {
    let earlier = greenbook::greenbook("hb64").flat();
    assert!(earlier.contains("3 Career-based intervention programs $1,660 $1,726 $1,795"));
    assert!(earlier.contains("Associated Services $227 $236 $245"));

    let weights: Vec<f64> = CTE_WEIGHTS
        .iter()
        .copied()
        .chain(std::iter::once(CTE_ASSOCIATED_WEIGHT))
        .collect();
    let (divisor, spread) = implied_divisor(&CAREER_TECHNICAL_FY2017, &weights);
    assert!(spread < 5e-4, "the six imply different divisors: {spread}");
    assert!(
        common::agrees_within(1.0, divisor, 8_333.12),
        "implied career-technical base cost {divisor}"
    );

    // It is a different base from the one the other two schedules divide by, which is what makes
    // the two families of weight incomparable — the corpus records the FY2027 gap at 19.6%.
    assert!(divisor > base_cost_fy2022());
}

/// The six special education weights are the 2009 schedule, multiplied by one number.
///
/// 0.8379, to within a part in eight thousand, on every category. The plan did not re-derive
/// them: it rescaled the Evidence-Based Model's six. In dollars that is a rise of 11.1% on the
/// FY2017 amounts, identical for every category — which is what a single scalar means and is a
/// different act from repricing a schedule.
#[test]
fn the_special_education_weights_are_the_2009_schedule_rescaled_once() {
    let ratios: Vec<f64> = SPECIAL_EDUCATION_WEIGHTS
        .iter()
        .zip(EVIDENCE_BASED_WEIGHTS)
        .map(|(now, then)| now / then)
        .collect();
    let mean = ratios.iter().sum::<f64>() / 6.0;
    let lo = ratios.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (hi - lo) / mean < 2e-4,
        "the six were rescaled by different numbers: {}",
        (hi - lo) / mean
    );
    assert!(common::agrees_within(0.0005, mean, 0.8379));

    // The same statement in dollars, which is the one a reader can price.
    let base = base_cost_fy2022();
    let raises: Vec<f64> = SPECIAL_EDUCATION_WEIGHTS
        .iter()
        .zip(SPECIAL_EDUCATION_FY2017)
        .map(|(weight, before)| weight * base / before - 1.0)
        .collect();
    for raise in &raises {
        assert!(
            (0.111..0.112).contains(raise),
            "one category rose {raise:.4} and the others did not"
        );
    }

    // And the FY2017 amounts were themselves those weights times a per-pupil figure, which is why
    // the ratios survive a regime that paid in dollars.
    let (divisor, spread) = implied_divisor(&SPECIAL_EDUCATION_FY2017, &EVIDENCE_BASED_WEIGHTS);
    assert!(spread < 5e-4, "FY2017 was not one schedule: {spread}");
    assert!(common::agrees_within(1.0, divisor, 5_430.91));
}

/// And the 2009 six are the 2001 six with one category doubled and one kind of pupil moved.
///
/// H.B. 94 states the schedule additively — `1 + 0.2892` — because it weighted a pupil rather
/// than an amount. H.B. 1 prints the old and the new side by side: category two roughly doubles
/// and the other five move by less than 3%. The change that matters more is not in the numbers,
/// and LSC says it in a sentence: vision-impaired pupils were reassigned to a higher category.
#[test]
fn the_2009_schedule_is_the_2001_schedule_with_one_category_doubled() {
    let first = greenbook::greenbook("hb94");
    assert_eq!(first.first_fiscal_year(), 2002);
    let oldest = first.flat();
    assert!(oldest.contains("Category One: 1 + 0.2892 = 1.2892 \u{2013} Speech only"));
    assert!(oldest.contains(
        "Category Six: 1 + 4.7342 = 5.7342 \u{2013} Autism, traumatic brain injury, both visually \
         and hearing disabled"
    ));

    let ebm = greenbook::greenbook("hb1").flat();
    assert_eq!(greenbook::greenbook("hb1").first_fiscal_year(), 2010);
    assert!(ebm.contains("One \u{2013} Speech only 0.2892 0.2906"));
    assert!(ebm.contains(
        "Two \u{2013} Specific learning disabled, developmentally disabled, other health \u{2013} \
         minor 0.3691 0.7374"
    ));
    assert!(ebm.contains(
        "Six \u{2013} Autism, traumatic brain injury, both visually and hearing impaired 4.7342 \
         4.7205"
    ));

    let moved: Vec<f64> = EVIDENCE_BASED_WEIGHTS
        .iter()
        .zip(FOUNDATION_WEIGHTS)
        .map(|(new, old)| new / old)
        .collect();
    assert!(
        (1.99..2.00).contains(&moved[1]),
        "category two moved {}",
        moved[1]
    );
    for (category, ratio) in moved.iter().enumerate() {
        if category == 1 {
            continue;
        }
        assert!(
            (0.99..1.03).contains(ratio),
            "category {} moved {ratio} as well",
            category + 1
        );
    }

    // The reassignment, which no weight records.
    assert!(ebm.contains(
        "the special education categories are changed slightly in that vision impaired and \
         orthopedically disabled students are assigned to categories with a higher weight"
    ));
}
