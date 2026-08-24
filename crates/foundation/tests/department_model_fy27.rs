//! Verify the base cost implementation against the department's own FY2027 model, and answer
//! the guarantee question the scenario left open.
//!
//! The fixture is extracted from `FY27 TRAD State Foundation Funding Calculator`, the
//! Department of Education and Workforce's working spreadsheet for the terminal year of the
//! Fair School Funding Plan phase-in. It carries, per district: base cost enrolled ADM broken
//! into the exact grade bands the formula uses, funded teacher counts, teacher base cost,
//! aggregate base cost, base cost per pupil, and the temporary transitional aid guarantee.
//!
//! This is a far stronger check than the single published worked example: 609 districts,
//! spanning three orders of magnitude of enrollment, against a factor set (`fy2027`) two
//! reference years newer than the one the implementation was originally written against.

use foundation::department_model::{self, ModelDistrict as Row};
use foundation::{
    aggregate_base_cost, teacher_base_cost, teacher_salary_refresh_delta, StatewideFactors,
};

/// ADM-weighted statewide average classroom teacher salary, FY2024, from the District Profile
/// Report. The refreshed input a reference-year update would adopt. [verified]
const FY2024_TEACHER_SALARY: f64 = 73_777.08;

/// Every district in the department's model, read through [`foundation::department_model`].
///
/// Was a private parser here. It is the crate's own fixture and two other test files were
/// reading it with two more copies of the column table; see issue #157.
fn rows() -> Vec<Row> {
    department_model::districts()
}

/// Base cost increase from refreshing the teacher salary input, using the department's own
/// funded position counts rather than any reconstruction of them.
///
/// The arithmetic is [`foundation::teacher_salary_refresh_delta`], which this file and
/// `tests/statewide_refresh.rs` each used to spell out with their own copy of the benefit
/// multiplier.
fn refresh_delta(r: &Row, factors: &StatewideFactors) -> f64 {
    teacher_salary_refresh_delta(
        r.funded_positions(),
        factors.teacher_salary,
        FY2024_TEACHER_SALARY,
        factors,
    )
}

/// The FY2020 Base State Funding a district is held at, recovered from its guarantee.
///
/// Only meaningful where the district is on the guarantee. See the section note below for the
/// open-enrollment assumption this rests on.
fn implied_fy2020_baseline(r: &Row) -> f64 {
    r.core_foundation + r.guarantee
}

/// What FY2027's formula produces as a share of the FY2020 level being guaranteed.
fn formula_share_of_baseline(r: &Row) -> f64 {
    r.core_foundation / implied_fy2020_baseline(r)
}

/// Pearson correlation between two equal-length series.
fn correlation(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let cov: f64 = xs.iter().zip(ys).map(|(a, b)| (a - mx) * (b - my)).sum();
    let vx: f64 = xs.iter().map(|a| (a - mx).powi(2)).sum();
    let vy: f64 = ys.iter().map(|b| (b - my).powi(2)).sum();
    cov / (vx * vy).sqrt()
}

/// The median, on the one definition this workspace has.
///
/// Was a local upper-of-two, which disagrees with `dispersion` on every even-length series.
fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    dispersion::median(&values).expect("a median is taken of a non-empty series here")
}

/// Share of a subset that is on the guarantee, as a percentage.
fn guarantee_rate(set: &[&Row]) -> f64 {
    100.0 * set.iter().filter(|r| r.on_guarantee()).count() as f64 / set.len() as f64
}

#[test]
fn fixture_covers_the_department_model() {
    assert_eq!(rows().len(), 609);
}

/// Funded classroom teachers, reconstructed from grade-band ADM, match the department's own
/// count for every district.
#[test]
fn reproduces_departments_funded_teacher_counts_for_every_district() {
    let f = StatewideFactors::fy2027();
    let mut worst = 0.0_f64;
    for r in rows() {
        let mine = teacher_base_cost(&r.enrollment(), &f);
        let dc = (mine.funded_classroom_teachers - r.funded_classroom_teachers).abs();
        let ds = (mine.funded_special_teachers - r.funded_special_teachers).abs();
        worst = worst.max(dc).max(ds);
        assert!(
            dc < 0.02,
            "{}: classroom {} vs department {}",
            r.name,
            mine.funded_classroom_teachers,
            r.funded_classroom_teachers
        );
        assert!(ds < 0.02, "{}: special mismatch", r.name);
    }
    assert!(worst < 0.02, "worst funded-count deviation {worst}");
}

/// Teacher base cost matches the department to within a cent per district across all 609.
/// This validates the whole pricing chain — benefit multiplier, insurance, substitute rate,
/// professional development days — against a factor set the implementation never saw.
#[test]
fn reproduces_departments_teacher_base_cost_for_every_district() {
    let f = StatewideFactors::fy2027();
    let mut worst = 0.0_f64;
    let mut worst_name = String::new();
    for r in rows() {
        let mine = teacher_base_cost(&r.enrollment(), &f).total;
        let diff = (mine - r.teacher_base_cost).abs();
        if diff > worst {
            worst = diff;
            worst_name = r.name.clone();
        }
    }
    assert!(
        worst < 0.02,
        "largest deviation ${worst:.4} at {worst_name} — expected agreement to the cent"
    );
}

/// The FY2027 factor set is the FY2022 reference year H.B. 96 carries forward. Comparing it to
/// the FY2022 factor set — which was priced from FY2018 — shows what one refresh was worth.
#[test]
fn the_one_reference_year_refresh_that_happened_raised_every_price() {
    let old = StatewideFactors::fy2022();
    let new = StatewideFactors::fy2027();
    assert!(new.teacher_salary > old.teacher_salary);
    assert!(new.insurance > old.insurance);
    assert!(new.principal_salary > old.principal_salary);
    assert!(new.building_per_pupil > old.building_per_pupil);
    // Teacher salary rose 8.5% across that update; the insurance term rose far faster, 20%.
    let salary_growth = new.teacher_salary / old.teacher_salary - 1.0;
    let insurance_growth = new.insurance / old.insurance - 1.0;
    assert!(
        (0.08..0.09).contains(&salary_growth),
        "salary {salary_growth:.4}"
    );
    assert!(
        insurance_growth > salary_growth * 2.0,
        "insurance {insurance_growth:.4}"
    );
}

/// **Nearly half of Ohio's districts are off-formula in the terminal phase-in year.**
#[test]
fn almost_half_of_districts_are_on_the_guarantee_in_fy2027() {
    let rs = rows();
    let on: Vec<&Row> = rs.iter().filter(|r| r.on_guarantee()).collect();
    assert_eq!(on.len(), 294);
    assert!((on.len() as f64 / rs.len() as f64 - 0.483).abs() < 0.005);

    let guaranteed_adm: f64 = on.iter().map(|r| r.base_cost_adm).sum();
    let all_adm: f64 = rs.iter().map(|r| r.base_cost_adm).sum();
    assert!(
        (guaranteed_adm / all_adm - 0.541).abs() < 0.01,
        "guaranteed ADM share was {:.3}",
        guaranteed_adm / all_adm
    );
}

/// The result that qualifies the scenario: the guarantee absorbs almost half of any base cost
/// increase, because a district below its guarantee floor sees the top-up shrink by exactly
/// what the formula gains.
#[test]
fn the_guarantee_absorbs_about_half_of_a_refresh() {
    let f = StatewideFactors::fy2027();
    let rs = rows();

    let computed: f64 = rs.iter().map(|r| refresh_delta(r, &f)).sum();
    let absorbed: f64 = rs
        .iter()
        .filter(|r| r.on_guarantee())
        .map(|r| refresh_delta(r, &f).min(r.guarantee))
        .sum();
    let delivered = computed - absorbed;

    assert!(
        (computed / 1e6 - 465.0).abs() < 3.0,
        "computed increase ${:.1}M",
        computed / 1e6
    );
    assert!(
        (delivered / 1e6 - 242.1).abs() < 3.0,
        "delivered increase ${:.1}M",
        delivered / 1e6
    );
    assert!(
        (delivered / computed - 0.521).abs() < 0.02,
        "delivered share {:.3}",
        delivered / computed
    );
}

/// Two in five Ohio districts would receive nothing at all from a reference-year refresh,
/// because their guarantee exceeds the increase several times over.
#[test]
fn two_in_five_districts_would_gain_nothing_from_a_refresh() {
    let f = StatewideFactors::fy2027();
    let rs = rows();
    let stuck: Vec<&Row> = rs
        .iter()
        .filter(|r| r.on_guarantee() && refresh_delta(r, &f) <= r.guarantee)
        .collect();
    let lifted_off = rs
        .iter()
        .filter(|r| r.on_guarantee() && refresh_delta(r, &f) > r.guarantee)
        .count();

    assert_eq!(stuck.len(), 242);
    assert_eq!(lifted_off, 52);
    assert!(
        (stuck.len() as f64 / rs.len() as f64 - 0.397).abs() < 0.01,
        "stuck share {:.3}",
        stuck.len() as f64 / rs.len() as f64
    );

    let stuck_adm: f64 = stuck.iter().map(|r| r.base_cost_adm).sum();
    let all_adm: f64 = rs.iter().map(|r| r.base_cost_adm).sum();
    assert!(
        (stuck_adm / all_adm - 0.418).abs() < 0.01,
        "stuck ADM share {:.3}",
        stuck_adm / all_adm
    );
}

/// Guaranteed districts are not marginal cases. The median one is held more than three times
/// above what a refresh would give it, so the guarantee is not a rounding-error buffer — it is
/// the operative funding mechanism for those districts.
#[test]
fn the_median_guaranteed_district_is_far_above_its_formula_amount() {
    let f = StatewideFactors::fy2027();
    let mut ratios: Vec<f64> = rows()
        .iter()
        .filter(|r| r.on_guarantee())
        .map(|r| r.guarantee / refresh_delta(r, &f))
        .collect();
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = ratios[ratios.len() / 2];
    assert!(
        median > 3.0,
        "median guarantee-to-increase ratio {median:.2}"
    );
}

// ---------------------------------------------------------------------------------------
// Why is nearly half the state on a guarantee? Two candidate causes, tested against each
// other: property wealth and enrollment decline. Both point the same way, so the question is
// which dominates.
// ---------------------------------------------------------------------------------------

/// Guaranteed districts are wealthier, less poor, and shrinking faster than districts on
/// formula. Every one of these medians runs the same direction.
#[test]
fn guaranteed_districts_are_wealthier_and_shrinking_faster() {
    let rs = rows();
    let on: Vec<&Row> = rs.iter().filter(|r| r.on_guarantee()).collect();
    let off: Vec<&Row> = rs.iter().filter(|r| !r.on_guarantee()).collect();

    let val = |set: &[&Row]| median(set.iter().filter_map(|r| r.valuation_per_pupil).collect());
    let trend = |set: &[&Row]| median(set.iter().map(|r| r.enrollment_change()).collect());

    assert!(
        val(&on) > val(&off) * 1.3,
        "guaranteed median valuation ${:.0} vs on-formula ${:.0}",
        val(&on),
        val(&off)
    );
    assert!(
        trend(&on) < trend(&off),
        "guaranteed enrollment change {:.4} vs on-formula {:.4}",
        trend(&on),
        trend(&off)
    );
    // Both groups are shrinking; the guaranteed group is shrinking faster.
    assert!(trend(&on) < 0.0 && trend(&off) < 0.0);
}

/// **Wealth dominates.** Splitting on both axes at their medians, the wealth gap in guarantee
/// rate is roughly three times the enrollment gap.
///
/// ```text
///                        enrollment falling fast   stable/rising
///   wealthy (> median)            79%                   65%
///   poor    (< median)            35%                   15%
/// ```
///
/// Holding enrollment constant, being wealthy adds ~45 points. Holding wealth constant, fast
/// decline adds ~15.
#[test]
fn property_wealth_predicts_the_guarantee_three_times_more_strongly_than_decline() {
    let owned = rows();
    let rs: Vec<&Row> = owned
        .iter()
        .filter(|r| r.valuation_per_pupil.is_some())
        .collect();

    let med_val = median(rs.iter().filter_map(|r| r.valuation_per_pupil).collect());
    let med_trend = median(rs.iter().map(|r| r.enrollment_change()).collect());

    let cell = |wealthy: bool, falling: bool| -> f64 {
        let sub: Vec<&Row> = rs
            .iter()
            .filter(|r| {
                (r.valuation_per_pupil.unwrap() > med_val) == wealthy
                    && (r.enrollment_change() < med_trend) == falling
            })
            .copied()
            .collect();
        guarantee_rate(&sub)
    };

    let (wf, ws, pf, ps) = (
        cell(true, true),
        cell(true, false),
        cell(false, true),
        cell(false, false),
    );
    assert!((wf - 79.0).abs() < 4.0, "wealthy+falling {wf:.0}%");
    assert!((ws - 65.0).abs() < 4.0, "wealthy+stable {ws:.0}%");
    assert!((pf - 35.0).abs() < 4.0, "poor+falling {pf:.0}%");
    assert!((ps - 15.0).abs() < 4.0, "poor+stable {ps:.0}%");

    let wealth_effect = ((wf - pf) + (ws - ps)) / 2.0;
    let decline_effect = ((wf - ws) + (pf - ps)) / 2.0;
    assert!(
        wealth_effect > decline_effect * 2.5,
        "wealth {wealth_effect:.0} points vs decline {decline_effect:.0} points"
    );
}

/// The guarantee rate rises monotonically with property wealth, from 13% in the poorest
/// quartile to 76% in the wealthiest. Three-quarters of Ohio's wealthiest districts are held
/// above what the formula computes for them.
#[test]
fn guarantee_rate_rises_monotonically_with_property_wealth() {
    let owned = rows();
    let mut rs: Vec<&Row> = owned
        .iter()
        .filter(|r| r.valuation_per_pupil.is_some())
        .collect();
    rs.sort_by(|a, b| {
        a.valuation_per_pupil
            .unwrap()
            .partial_cmp(&b.valuation_per_pupil.unwrap())
            .unwrap()
    });
    let q = rs.len() / 4;
    let rates: Vec<f64> = (0..4)
        .map(|i| {
            let slice = if i < 3 {
                &rs[i * q..(i + 1) * q]
            } else {
                &rs[3 * q..]
            };
            guarantee_rate(slice)
        })
        .collect();

    assert!((rates[0] - 13.0).abs() < 4.0, "Q1 {:.0}%", rates[0]);
    assert!((rates[3] - 76.0).abs() < 4.0, "Q4 {:.0}%", rates[3]);
    for w in rates.windows(2) {
        assert!(
            w[1] > w[0],
            "guarantee rate must rise with wealth: {rates:?}"
        );
    }
}

/// Enrollment decline predicts the guarantee too, but weakly by comparison — 58% in the
/// fastest-declining quartile against 31% among growing districts.
#[test]
fn enrollment_decline_predicts_the_guarantee_but_weakly() {
    let owned = rows();
    let mut rs: Vec<&Row> = owned.iter().collect();
    rs.sort_by(|a, b| {
        a.enrollment_change()
            .partial_cmp(&b.enrollment_change())
            .unwrap()
    });
    let q = rs.len() / 4;
    let fastest = guarantee_rate(&rs[..q]);
    let growing = guarantee_rate(&rs[3 * q..]);
    assert!(
        (fastest - 58.0).abs() < 5.0,
        "fastest decline {fastest:.0}%"
    );
    assert!((growing - 31.0).abs() < 5.0, "growing {growing:.0}%");
    assert!(fastest > growing);
    // The spread is real but far narrower than the wealth spread of 13% to 76%.
    assert!(fastest - growing < 40.0);
}

// ---------------------------------------------------------------------------------------
// Running the formula without the guarantee. The corpus can compute state aid as the formula
// determines it ([H] core foundation funding) and as districts actually receive it
// ([H] + [I]), and compare the two directly rather than inferring the difference.
// ---------------------------------------------------------------------------------------

/// Districts with valuation data, for the wealth comparisons.
fn with_valuation(rs: &[Row]) -> Vec<&Row> {
    rs.iter()
        .filter(|r| r.valuation_per_pupil.is_some())
        .collect()
}

/// The formula equalizes more strongly than realized funding does. Removing the guarantee
/// strengthens the association between wealth and aid from -0.605 to -0.662.
///
/// The effect on the correlation is modest — about 9% — which is worth stating plainly because
/// the level effect below is not modest at all. Correlation measures how tightly aid tracks
/// wealth, not how much money moves.
#[test]
fn the_guarantee_weakens_the_formulas_equalization() {
    let owned = rows();
    let rs = with_valuation(&owned);
    let wealth: Vec<f64> = rs.iter().map(|r| r.valuation_per_pupil.unwrap()).collect();
    let formula: Vec<f64> = rs.iter().map(|r| r.formula_aid_per_pupil()).collect();
    let realized: Vec<f64> = rs.iter().map(|r| r.realized_aid_per_pupil()).collect();

    let cf = correlation(&wealth, &formula);
    let cr = correlation(&wealth, &realized);

    assert!(
        (cf - -0.662).abs() < 0.02,
        "formula-only correlation {cf:.3}"
    );
    assert!((cr - -0.605).abs() < 0.02, "realized correlation {cr:.3}");
    assert!(
        cf.abs() > cr.abs(),
        "the formula alone must equalize more than realized funding does"
    );
}

/// **The level effect, which is where the guarantee actually shows up — and it is starker than
/// the correlation suggests.**
///
/// Median guarantee uplift by valuation quartile: **$0, $0, $685, $1,154**. The typical district
/// in the poorer half of Ohio receives nothing from the guarantee at all, because most such
/// districts are not on it. The typical district in the wealthiest quartile receives $1,154 per
/// pupil, which more than doubles the state aid the formula would give it.
///
/// Comparing quartile medians rather than per-district differences gives $21 and $1,658 — a
/// different question (how far apart typical districts end up) with the same answer in
/// direction. Both are recorded because quoting either alone invites the wrong reading.
#[test]
fn the_guarantee_pays_wealthy_districts_and_the_poorer_half_nothing() {
    let owned = rows();
    let mut rs = with_valuation(&owned);
    rs.sort_by(|a, b| {
        a.valuation_per_pupil
            .unwrap()
            .partial_cmp(&b.valuation_per_pupil.unwrap())
            .unwrap()
    });
    let q = rs.len() / 4;
    let quartile = |i: usize| -> &[&Row] {
        if i < 3 {
            &rs[i * q..(i + 1) * q]
        } else {
            &rs[3 * q..]
        }
    };
    let uplift = |set: &[&Row]| {
        median(
            set.iter()
                .map(|r| r.realized_aid_per_pupil() - r.formula_aid_per_pupil())
                .collect(),
        )
    };

    // Median per-district uplift: nothing at all in the poorer half.
    assert!(uplift(quartile(0)) == 0.0, "Q1 median uplift must be zero");
    assert!(uplift(quartile(1)) == 0.0, "Q2 median uplift must be zero");
    assert!(
        (uplift(quartile(2)) - 685.0).abs() < 80.0,
        "Q3 uplift ${:.0}",
        uplift(quartile(2))
    );
    assert!(
        (uplift(quartile(3)) - 1_154.0).abs() < 120.0,
        "Q4 uplift ${:.0}",
        uplift(quartile(3))
    );

    // The same comparison on quartile medians rather than per-district differences.
    let med_aid = |set: &[&Row], f: fn(&Row) -> f64| median(set.iter().map(|r| f(r)).collect());
    let q1_gap = med_aid(quartile(0), Row::realized_aid_per_pupil)
        - med_aid(quartile(0), Row::formula_aid_per_pupil);
    let q4_gap = med_aid(quartile(3), Row::realized_aid_per_pupil)
        - med_aid(quartile(3), Row::formula_aid_per_pupil);
    assert!((q1_gap - 21.0).abs() < 30.0, "Q1 median gap ${q1_gap:.0}");
    assert!(
        (q4_gap - 1_658.0).abs() < 150.0,
        "Q4 median gap ${q4_gap:.0}"
    );

    // And it more than doubles what the wealthiest quartile would otherwise receive.
    let q4_formula = median(
        quartile(3)
            .iter()
            .map(|r| r.formula_aid_per_pupil())
            .collect(),
    );
    let q4_realized = median(
        quartile(3)
            .iter()
            .map(|r| r.realized_aid_per_pupil())
            .collect(),
    );
    assert!(
        q4_realized > q4_formula * 2.0,
        "Q4 formula ${q4_formula:.0} vs realized ${q4_realized:.0}"
    );
}

/// A methodological point the corpus needs, because it inverts a habit.
///
/// For *spending*, less dispersion is more equitable. For *state aid*, the opposite: aid is
/// compensatory, so a wide spread means strong targeting and a narrow one means weak targeting.
///
/// Realized aid is measurably **more equal** than formula aid — coefficient of variation 0.544
/// against 0.677, federal range ratio 9.6 against 12.3. That equality is the guarantee
/// flattening the compensation, not an improvement.
#[test]
fn realized_aid_is_more_equal_than_formula_aid_and_that_is_the_problem() {
    let owned = rows();
    let rs = with_valuation(&owned);
    let stats = |vals: Vec<f64>| -> (f64, f64) {
        let n = vals.len() as f64;
        let mean = vals.iter().sum::<f64>() / n;
        let sd = (vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n).sqrt();
        let mut s = vals;
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (sd / mean, s[19 * s.len() / 20] / s[s.len() / 20])
    };

    let (cv_formula, frr_formula) = stats(rs.iter().map(|r| r.formula_aid_per_pupil()).collect());
    let (cv_realized, frr_realized) =
        stats(rs.iter().map(|r| r.realized_aid_per_pupil()).collect());

    assert!(
        (cv_formula - 0.677).abs() < 0.03,
        "formula CV {cv_formula:.3}"
    );
    assert!(
        (cv_realized - 0.544).abs() < 0.03,
        "realized CV {cv_realized:.3}"
    );
    assert!(
        cv_realized < cv_formula,
        "the guarantee compresses the aid distribution"
    );
    assert!(frr_realized < frr_formula, "and narrows its range");
    assert!(
        frr_formula > 10.0,
        "formula aid spans more than 10x across the distribution"
    );
}

// ---------------------------------------------------------------------------------------
// The guarantee's baseline, per the FY2026 payment report: districts are held at what they
// received in **FY2020**. The guarantee is the positive difference between that Base State
// Funding and core foundation funding, less an open-enrollment adjustment that applies only
// where a district has curtailed entering open enrollment beyond a threshold.
//
//     guarantee = max(0, base_state_funding - core_foundation - open_enrollment_adjustment)
//
// The adjustment is zero for districts that have not curtailed open enrollment, so for a
// guaranteed district the FY2020 baseline can be recovered as core + guarantee. Every figure
// below carries that assumption.
// ---------------------------------------------------------------------------------------

/// For a guaranteed district, the Fair School Funding Plan at 100% phase-in produces a median
/// **67.8%** of what the district received in FY2020. A hundred and two of them would receive
/// under half.
#[test]
fn the_formula_pays_guaranteed_districts_two_thirds_of_their_fy2020_level() {
    let owned = rows();
    let on: Vec<&Row> = owned.iter().filter(|r| r.on_guarantee()).collect();

    let shares: Vec<f64> = on.iter().map(|r| formula_share_of_baseline(r)).collect();
    let med = median(shares.clone());
    assert!((med - 0.678).abs() < 0.02, "median share {med:.3}");

    let under_half = shares.iter().filter(|s| **s < 0.5).count();
    assert!(
        (under_half as i64 - 102).abs() <= 5,
        "districts under half: {under_half}"
    );

    // In aggregate the formula reaches 71% of the guaranteed baseline; the guarantee holds the
    // remaining $879M.
    let baseline: f64 = on.iter().map(|r| implied_fy2020_baseline(r)).sum();
    let formula: f64 = on.iter().map(|r| r.core_foundation).sum();
    assert!((formula / baseline - 0.710).abs() < 0.02);
    assert!(((baseline - formula) / 1e6 - 879.0).abs() < 5.0);
}

/// **The shape of the shortfall follows wealth.** Among guaranteed districts, the formula
/// produces 92% of the FY2020 baseline for the poorest quartile and 43% for the wealthiest.
///
/// This closes the causal chain. The local capacity measure reduces aid to property-wealthy
/// districts by design; their FY2020 baseline — set under the Bridge formula, in a year Ohio
/// froze funding rather than computing it — was far higher; the guarantee holds them there.
#[test]
fn the_formula_falls_furthest_below_the_baseline_for_wealthy_districts() {
    let owned = rows();
    let mut on: Vec<&Row> = owned
        .iter()
        .filter(|r| r.on_guarantee() && r.valuation_per_pupil.is_some())
        .collect();
    on.sort_by(|a, b| {
        a.valuation_per_pupil
            .unwrap()
            .partial_cmp(&b.valuation_per_pupil.unwrap())
            .unwrap()
    });
    let q = on.len() / 4;
    let share =
        |slice: &[&Row]| median(slice.iter().map(|r| formula_share_of_baseline(r)).collect());

    let poorest = share(&on[..q]);
    let wealthiest = share(&on[3 * q..]);
    assert!((poorest - 0.919).abs() < 0.04, "Q1 share {poorest:.3}");
    assert!(
        (wealthiest - 0.434).abs() < 0.04,
        "Q4 share {wealthiest:.3}"
    );
    assert!(
        poorest > wealthiest * 1.8,
        "the gradient must be steep: Q1 {poorest:.3} vs Q4 {wealthiest:.3}"
    );

    // Monotonic across all four quartiles.
    let all: Vec<f64> = (0..4)
        .map(|i| {
            if i < 3 {
                share(&on[i * q..(i + 1) * q])
            } else {
                share(&on[3 * q..])
            }
        })
        .collect();
    for w in all.windows(2) {
        assert!(w[1] < w[0], "share must fall as wealth rises: {all:?}");
    }
}

/// No guaranteed district's formula amount exceeds its baseline — which is what "guarantee"
/// means, and a check that the recovered baseline is coherent rather than an artefact.
#[test]
fn no_guaranteed_district_exceeds_its_own_baseline() {
    for r in rows().iter().filter(|r| r.on_guarantee()) {
        let share = formula_share_of_baseline(r);
        assert!(
            (0.0..=1.0).contains(&share),
            "{} formula/baseline = {share:.4}, outside [0,1]",
            r.name
        );
    }
}

/// **The whole build-up reproduces the department's published aggregate, for every district.**
///
/// The teacher test above proves one sub-component of five. This proves all twenty-two elements
/// together, against the aggregate the department printed, across the full panel — which is what
/// licenses the web layer to *show* the build-up rather than quote the total.
///
/// Not to the cent, and the reason is arithmetic rather than disagreement: twenty-two elements
/// each rounded at the point the department rounds, summed. The worst district is off by $1.09 on
/// an aggregate base cost of $11.77 billion statewide, and the median district by four parts in a
/// hundred million. A tolerance of two dollars is far below any figure that would represent a
/// different reading of R.C. 3317.011 and far above the drift that accumulated rounding produces.
#[test]
fn reproduces_departments_aggregate_base_cost_for_every_district() {
    let f = StatewideFactors::fy2027();
    let mut worst = 0.0_f64;
    let mut worst_name = String::new();
    let (mut published_total, mut computed_total) = (0.0_f64, 0.0_f64);

    for r in rows() {
        let mine = aggregate_base_cost(&r.enrollment(), &f);
        let diff = (mine.aggregate - r.aggregate_base_cost).abs();
        published_total += r.aggregate_base_cost;
        computed_total += mine.aggregate;
        if diff > worst {
            worst = diff;
            worst_name = r.name.clone();
        }
    }

    assert!(
        worst < 2.0,
        "largest deviation ${worst:.2} at {worst_name} — the build-up and the department's \
         aggregate have diverged by more than accumulated rounding explains"
    );
    // And the errors are noise rather than a bias: they cancel to nothing across the state.
    let drift = (computed_total - published_total).abs() / published_total;
    assert!(
        drift < 1e-7,
        "statewide drift {drift:.3e} is directional, not rounding"
    );
}
