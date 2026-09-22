//! What districts employ against what R.C. 3317.011 funds, for the one staff category a
//! published source counts.
//!
//! # The question, and the first thing to check
//!
//! `what_the_staffing_minimums_are_worth_and_where_they_stop.rs` priced the section's staffing
//! floors and closed on the one thing it could not establish: whether they are set in the right
//! places, because nothing in this repository carried a district's actual staffing. Issue #408
//! pointed at the District Profile Report, already catalogued and already read, which names
//! *personnel* as one of its five families — and said to open the sheet and list the personnel
//! columns before planning anything, because three outcomes were possible and they are very
//! different issues.
//!
//! **The report carries one count, and it is administrators.** Seven of its sixty columns are
//! personnel: a teachers' average salary, three experience shares, an administrators' average
//! salary, a pupil-administrator ratio and `FTE Number of Administrators`. There is no teacher
//! FTE and no category finer than "administrator". Six of the section's seven floors — special
//! teachers, counselors, wellness staff, fiscal support, EMIS, clerical — meet nothing here, and
//! the question of whether *those* are set in the right places moves to EMIS staff reporting,
//! which the corpus has not catalogued and this file does not assume retrievable.
//!
//! The one count is the department's sum of twelve EMIS position codes: superintendent and
//! deputy, treasurer, principal and assistant principal, director, supervisor, coordinator,
//! education administration specialist, licensed administrative assistant, community school
//! administrator, other officials. That is the union of the section's four administrator
//! elements — (F)(1) superintendent, (F)(2) treasurer, (F)(3) other district administrators and
//! (G)(1) building leaders — and does not include (F)(6)'s leadership support, whose statutory
//! salary band tops out at $65,000. So the comparison is an aggregate against the *same*
//! aggregate, and the floor it can see is the two other administrators at (F)(3).
//!
//! # Funded is not employed
//!
//! Nothing here is an audit. R.C. 3317.011 does not require a district to staff what the
//! build-up itemises, and the level of the gap says as much about what districts choose as
//! about what they need. The finding is the **gradient** — whether the gap behaves differently
//! below the 1,500 ADM threshold than above it — and the levels are stated because a gradient
//! cannot be read without them. Both sides are FY2024: the employed count is the report's, and
//! the funded count is taken at the report's own enrolled ADM with ratios the section has stated
//! since it was enacted. The FY2027 model's base cost ADM is used once, as a robustness check.
//!
//! # What it finds
//!
//! **The level.** 606 districts employ 12,938 FTE administrators; the section funds 6,580
//! positions for them. Districts carry about twice what the build-up counts, and 47 employ fewer
//! than it funds — 44 of them under the threshold.
//!
//! **The gradient runs against a cliff story.** The median employed-to-funded multiple is
//! **1.23 in the smallest sextile** and about 1.8 from the fourth up; 1.52 below the threshold
//! and 1.83 above it:
//!
//! | ADM sextile | median ADM | employed | funded | employed ÷ funded | per 1,000 pupils |
//! |---|--:|--:|--:|--:|--:|
//! | 1 | 552 | 6.25 | 5.23 | **1.23** | 12.3 |
//! | 2 | 886 | 9.70 | 5.97 | 1.61 | 10.8 |
//! | 3 | 1,229 | 11.00 | 6.73 | 1.64 | 9.0 |
//! | 4 | 1,663 | 15.00 | 7.91 | 1.85 | 8.8 |
//! | 5 | 2,548 | 20.00 | 11.06 | 1.80 | 7.8 |
//! | 6 | 5,256 | 39.00 | 20.69 | 1.82 | 7.2 |
//!
//! The floor does not put small districts above their funded count. It brings the funded count
//! closest to payroll exactly where it binds, and employed administrators per thousand pupils
//! fall monotonically across all six sextiles — real staffing has the fixed component the
//! section models. That is the wedge of the earlier file, seen from the payroll side.
//!
//! **The height is about right and the shape is not.** A line through the 319 districts under
//! the threshold has an intercept of 3.85 administrators and a slope of 6.2 per thousand pupils;
//! the section's fixed component is four and its slope is 3.6. But the intercept depends on
//! where the line stops — between 2.6 and 3.8 across knots from 750 to 3,000 ADM, every one of
//! them under four — and the plateau the floor writes into the funded count has no counterpart
//! at all. The section funds exactly two other administrators at every enrolment under 1,500;
//! what districts employ beyond a superintendent, a treasurer and their funded building leaders
//! rises from about three at 250-500 ADM to about six at 1,250-1,500. And the slope contrast
//! either side of 1,500 is the largest of any knot tried, at 1.71 against 1.28-1.62, which is a
//! convexity in the data and not a threshold the statute found: every knot from 1,250 up gives
//! a contrast above 1.5.
//!
//! # What is not established here
//!
//! Anything about the other six floors, for want of a count. And nothing about *why* the
//! smallest districts sit closest to their funded count — whether the floor is generous to
//! them, or they are lean, or the department's position codes catch a superintendent who is also
//! a principal once rather than twice. The report cannot say and this file does not guess.

use dispersion::profile::{self, EXPECTED_HEADER};
use foundation::minimums::Minimum;
use project::administrator_staffing::{
    band, districts, fit, sextiles, threshold, within, District, FundedAdministrators,
};
use project::panel::panel;

/// Every district the report counts administrators for.
fn rows() -> Vec<District> {
    districts(&profile::districts(), &panel())
}

/// **The first thing to check**, asserted so the three-outcome question stays answered.
///
/// The report's personnel block carries exactly one count. If the department adds a teacher
/// FTE, or a counselor count, this fails and the per-category comparison #408 wanted first
/// becomes possible.
#[test]
fn the_profile_report_counts_one_kind_of_staff_and_it_is_administrators() {
    let columns: Vec<&str> = EXPECTED_HEADER.split(',').collect();
    let personnel: Vec<&str> = columns
        .iter()
        .copied()
        .filter(|c| c.contains("teacher") || c.contains("administrator"))
        .collect();
    assert_eq!(personnel.len(), 7, "the personnel block is seven columns");
    let counts: Vec<&str> = columns
        .iter()
        .copied()
        .filter(|c| c.starts_with("fte_"))
        .collect();
    assert_eq!(counts, vec!["fte_administrators_fy24"]);

    // And the floors it cannot see, named so the negative is checkable: one word from each
    // of the other seven floors' elements, none of which any column carries.
    assert_eq!(Minimum::ALL.len(), 8);
    for word in [
        "special",
        "counselor",
        "wellness",
        "fiscal",
        "emis",
        "leadership",
        "clerical",
        "building",
    ] {
        assert!(
            !columns.iter().any(|c| c.contains(word)),
            "the extract now carries something about {word} staff"
        );
    }
}

/// **The level**, which is the least interesting number here and the one the rest rests on.
#[test]
fn districts_employ_twice_the_administrators_the_build_up_funds() {
    let rows = rows();
    assert_eq!(
        rows.len(),
        606,
        "every district in the report reports a count"
    );
    assert!(
        rows.iter().all(|d| d.funded_fy27.is_some()),
        "every one of them is in the FY2027 model"
    );

    let employed: f64 = rows.iter().map(|d| d.employed).sum();
    let funded: f64 = rows.iter().map(|d| d.funded.total()).sum();
    assert!(
        (employed - 12_938.03).abs() < 0.01,
        "employed {employed:.2}"
    );
    assert!((funded - 6_579.54).abs() < 0.01, "funded {funded:.2}");
    assert!((employed / funded - 1.966).abs() < 0.001);

    assert_eq!(
        rows.iter()
            .filter(|d| d.employed < d.funded.total())
            .count(),
        47
    );
    assert_eq!(
        rows.iter()
            .filter(|d| d.floored() && d.employed < d.funded.total())
            .count(),
        44,
        "and nearly all of them are under the threshold"
    );

    // The funded side is the same four elements the department's count sums, so the smallest
    // district in the report is funded for 4.33 administrators and employs 6.12.
    let smallest = rows
        .iter()
        .min_by(|a, b| a.adm.total_cmp(&b.adm))
        .expect("the report is not empty");
    assert!(smallest.name.starts_with("Vanlue Local"));
    assert!((smallest.adm - 149.88).abs() < 0.01);
    assert!((smallest.funded.total() - 4.33).abs() < 0.005);
    assert!((smallest.employed - 6.12).abs() < 0.005);
    assert!((FundedAdministrators::FIXED - 4.0).abs() < f64::EPSILON);
}

/// **The gradient.** The gap is narrowest where the floor binds, and real staffing has a fixed
/// component.
#[test]
fn the_gap_is_narrowest_in_the_smallest_sextile_and_widens_to_a_plateau() {
    let rows = rows();
    let bands = sextiles(&rows);
    assert_eq!(bands.len(), 6);
    assert!(bands.iter().all(|b| b.n == 101));

    let adm: Vec<f64> = bands.iter().map(|b| b.adm).collect();
    for (got, want) in adm
        .iter()
        .zip([552.0, 886.0, 1_229.0, 1_663.0, 2_548.0, 5_256.0])
    {
        assert!((got - want).abs() < 1.0, "median ADM {adm:?}");
    }
    let multiples: Vec<f64> = bands.iter().map(|b| b.multiple).collect();
    for (got, want) in multiples
        .iter()
        .zip([1.2279, 1.6077, 1.6446, 1.8468, 1.7985, 1.8192])
    {
        assert!((got - want).abs() < 0.0005, "multiples {multiples:?}");
    }
    assert!(
        multiples[0] < multiples[1] && multiples[1] < multiples[2] && multiples[2] < multiples[3],
        "rising through the floored sextiles"
    );
    assert!(
        multiples[3..].iter().all(|m| (1.78..1.86).contains(m)),
        "and flat above them: {multiples:?}"
    );

    let per_thousand: Vec<f64> = bands.iter().map(|b| b.per_thousand).collect();
    for window in per_thousand.windows(2) {
        assert!(window[1] < window[0], "not monotone: {per_thousand:?}");
    }
    assert!((per_thousand[0] - 12.31).abs() < 0.01 && (per_thousand[5] - 7.22).abs() < 0.01);

    // Either side of the threshold, which is the split the issue asked about.
    let (below, above) = (
        band(&rows.iter().filter(|d| d.floored()).collect::<Vec<_>>()),
        band(&rows.iter().filter(|d| !d.floored()).collect::<Vec<_>>()),
    );
    assert_eq!((below.n, above.n), (319, 287));
    assert!((threshold() - 1_500.0).abs() < f64::EPSILON);
    assert!(
        (below.multiple - 1.5156).abs() < 0.0005,
        "below {}",
        below.multiple
    );
    assert!(
        (above.multiple - 1.8354).abs() < 0.0005,
        "above {}",
        above.multiple
    );

    // The same ordering on the FY2027 model's base cost ADM moves nothing that matters: the
    // vintage mismatch every other join in this crate carries is not what the gradient is.
    let mut on_model: Vec<(f64, f64)> = rows
        .iter()
        .map(|d| {
            (
                d.adm,
                d.employed / d.funded_fy27.expect("all 606 are in the model").total(),
            )
        })
        .collect();
    on_model.sort_by(|a, b| a.0.total_cmp(&b.0));
    let sixth = on_model.len() / 6;
    let robust: Vec<f64> = (0..6)
        .map(|s| {
            let end = if s == 5 {
                on_model.len()
            } else {
                (s + 1) * sixth
            };
            let mut v: Vec<f64> = on_model[s * sixth..end].iter().map(|p| p.1).collect();
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        })
        .collect();
    for (a, b) in robust.iter().zip(&multiples) {
        assert!((a - b).abs() < 0.03, "on the model's count {robust:?}");
    }
}

/// **The height and the shape.** The fixed component the smallest districts carry is close to
/// the section's four; the plateau the floor writes is not in the data.
#[test]
fn the_floors_height_is_about_right_and_its_plateau_has_no_counterpart() {
    let rows = rows();
    let below = fit(&rows.iter().filter(|d| d.floored()).collect::<Vec<_>>());
    assert_eq!(below.n, 319);
    assert!(
        (below.intercept - 3.847).abs() < 0.001,
        "intercept {}",
        below.intercept
    );
    assert!(
        (below.slope * 1_000.0 - 6.173).abs() < 0.001,
        "slope {} per thousand",
        below.slope * 1_000.0
    );
    assert!(
        (FundedAdministrators::slope_per_pupil() * 1_000.0 - 3.556).abs() < 0.001,
        "the section's slope above the threshold"
    );
    assert!(below.intercept < FundedAdministrators::FIXED);

    // The intercept is a property of where the line stops, so it is not one number. Every
    // knot from 750 to 3,000 ADM puts it under four, and 1,500 puts it highest.
    let knots = [
        750.0, 1_000.0, 1_250.0, 1_500.0, 1_750.0, 2_000.0, 2_500.0, 3_000.0,
    ];
    let intercepts: Vec<f64> = knots
        .iter()
        .map(|k| fit(&within(&rows, 0.0, *k)).intercept)
        .collect();
    assert!(
        intercepts.iter().all(|i| (2.5..4.0).contains(i)),
        "{intercepts:?}"
    );
    assert!(
        (intercepts[3] - 3.847).abs() < 0.001 && intercepts.iter().all(|i| *i <= intercepts[3]),
        "{intercepts:?}"
    );

    // The slope contrast either side of a knot is a convexity, not a threshold: 1,500 gives
    // the largest, and every knot from 1,250 up gives one above 1.5.
    let contrasts: Vec<f64> = knots
        .iter()
        .map(|k| fit(&within(&rows, *k, f64::MAX)).slope / fit(&within(&rows, 0.0, *k)).slope)
        .collect();
    assert!((contrasts[3] - 1.712).abs() < 0.001, "{contrasts:?}");
    assert!(contrasts.iter().all(|c| *c <= contrasts[3]));
    assert!(
        contrasts[2..].iter().all(|c| (1.4..1.8).contains(c)),
        "{contrasts:?}"
    );
    assert!(contrasts[..2].iter().all(|c| *c < 1.4), "{contrasts:?}");

    // The plateau. The section funds two other administrators at every enrolment under the
    // threshold; what districts employ beyond the three elements with no floor does not sit
    // at two anywhere but the very smallest bin, and doubles across the floored range.
    let mut lo = 250.0;
    let mut beyond = Vec::new();
    while lo < threshold() {
        let bin = within(&rows, lo, lo + 250.0);
        assert!(
            bin.iter()
                .all(|d| (d.funded.other - 2.0).abs() < f64::EPSILON),
            "the floor is what sets the count at {lo}"
        );
        beyond.push(band(&bin).beyond_the_fixed_elements);
        lo += 250.0;
    }
    assert_eq!(beyond.len(), 5);
    assert!(
        (beyond[0] - 2.90).abs() < 0.005 && (beyond[4] - 6.29).abs() < 0.005,
        "{beyond:?}"
    );
    assert!(beyond[4] > 2.0 * beyond[0]);
    for window in beyond[..4].windows(2) {
        assert!(window[1] > window[0], "{beyond:?}");
    }
}

/// **What the count cannot say**, kept visible. Six districts report fewer than four
/// administrators, and one reports a single FTE for 687 pupils. Whether that is a
/// superintendent-principal counted once or a reporting artefact the report does not say, and
/// the fixed component above is fitted with them in.
#[test]
fn six_districts_report_fewer_administrators_than_the_section_funds_as_fixed() {
    let rows = rows();
    let few: Vec<&District> = rows
        .iter()
        .filter(|d| d.employed < FundedAdministrators::FIXED)
        .collect();
    assert_eq!(few.len(), 6);
    assert!(few.iter().all(|d| d.floored()));
    let one = few
        .iter()
        .find(|d| (d.employed - 1.0).abs() < f64::EPSILON)
        .expect("one district reports a single administrator");
    assert!(one.name.starts_with("Manchester Local (000442)"));
    assert!((one.adm - 687.29).abs() < 0.01);
}
