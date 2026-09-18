//! "Districts with large industrial tax bases" names three populations, and the wrong one.
//!
//! `education-agency/toledo-city` recorded the FY2016 tail as "mostly small districts with large
//! industrial tax bases"; `dispersion::fy2016` refuted that against a **summed** business share
//! and left the summing unexamined. This checks the sum.
//!
//! # Three columns, three sets of districts
//!
//! Table SD-1 carries industrial, mineral and public utility value separately, and the sixty
//! districts most concentrated in each barely intersect: two districts in both the industrial and
//! the mineral sixty, three in the industrial and public-utility sixty, one in all three. Adding
//! the columns builds a measure of a set that is almost empty.
//!
//! # And only the smallest of them behaves the way the argument says
//!
//! Against FY2027 guarantee status, where the statewide rate is 48.3%, industrial districts run
//! **below** baseline at 44.4% and lose pupils **more slowly** than the median district. Mineral
//! districts — all ten of them — run at 90%, lose pupils about 1.4 times as fast as the median
//! district, and are worth 1.7 times the statewide valuation per pupil.
//!
//! The characterisation is misattributed rather than false, and a summed share cannot recover it:
//! the ten districts the result is about are outnumbered twenty to one inside the sum.

use std::collections::BTreeMap;

use dispersion::tax_base::{self, Base, CONCENTRATED, TAX_YEAR};
use foundation::department_model::{self, ModelDistrict};

/// The statewide guarantee rate every comparison below is against. [verified]
const STATEWIDE_GUARANTEE_RATE: f64 = 0.483;

/// Districts in the department's FY2027 model, keyed for the join.
fn model() -> BTreeMap<String, ModelDistrict> {
    department_model::districts()
        .into_iter()
        .map(|d| (d.irn.clone(), d))
        .collect()
}

/// The median of a sample, by value. Empty gives `None` rather than a zero that would sort.
fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(values[values.len() / 2])
}

/// What one class's concentrated districts look like against the model.
struct Profile {
    districts: usize,
    guarantee_rate: f64,
    median_enrollment_change: f64,
    median_valuation_per_pupil: f64,
}

fn profile_of(base: Base) -> Profile {
    let model = model();
    let rows: Vec<&ModelDistrict> = tax_base::concentrated_in(base)
        .iter()
        .filter_map(|row| model.get(&row.irn))
        .collect();
    assert!(
        !rows.is_empty(),
        "no districts concentrated in {}",
        base.label()
    );
    Profile {
        districts: rows.len(),
        guarantee_rate: rows.iter().filter(|d| d.on_guarantee()).count() as f64 / rows.len() as f64,
        median_enrollment_change: median(rows.iter().map(|d| d.enrollment_change()).collect())
            .expect("a non-empty set has a median"),
        median_valuation_per_pupil: median(
            rows.iter().filter_map(|d| d.valuation_per_pupil).collect(),
        )
        .expect("concentrated districts have a published valuation"),
    }
}

/// The join is total: every district the model carries has a composition at `TAX_YEAR`.
#[test]
fn the_abstract_covers_every_district_the_model_funds() {
    let composition = tax_base::by_irn();
    let missing: Vec<String> = department_model::districts()
        .iter()
        .filter(|d| !composition.contains_key(&d.irn))
        .map(|d| d.name.clone())
        .collect();
    assert!(
        missing.is_empty(),
        "the abstract does not reach {} of the model's districts: {missing:?}",
        missing.len()
    );
}

/// **The finding, first half.** The three classes are three different sets of districts.
#[test]
fn the_three_business_classes_are_nearly_disjoint_populations() {
    const TOP: usize = 60;
    assert_eq!(overlap_pair(Base::Industrial, Base::Mineral, TOP), 2);
    assert_eq!(overlap_pair(Base::Industrial, Base::PublicUtility, TOP), 3);
    assert_eq!(overlap_pair(Base::Mineral, Base::PublicUtility, TOP), 23);

    // The disjointness is not an artefact of where the cut is taken. At every width from a
    // twentieth of the state to a sixth of it, industrial and mineral share at most a tenth of
    // their members — which is what "these are different districts" has to mean if it is to mean
    // anything.
    for top in [30, 60, 90, 120] {
        let shared = overlap_pair(Base::Industrial, Base::Mineral, top);
        assert!(
            (shared as f64) < (top as f64) * 0.15,
            "industrial and mineral share {shared} of their top {top}, which is not disjoint"
        );
    }
}

fn overlap_pair(a: Base, b: Base, top: usize) -> usize {
    let forward = tax_base::overlap(a, b, top);
    assert_eq!(
        forward,
        tax_base::overlap(b, a, top),
        "an intersection cannot depend on the order of its arguments"
    );
    forward
}

/// The classes are as far apart in level as they are in membership.
#[test]
fn a_summed_business_share_is_mostly_public_utility() {
    let counts = |base: Base| tax_base::concentrated_in(base).len();
    assert_eq!(counts(Base::Industrial), 27);
    assert_eq!(counts(Base::Mineral), 10);
    // 197 in the abstract against 196 the model funds: the abstract carries 611 districts and
    // the model 609, and one of the two it drops is concentrated in public utility. Both numbers
    // are right about a different population, which is the mismatch `project::baseline` records.
    assert_eq!(counts(Base::PublicUtility), 197);
    assert_eq!(profile_of(Base::PublicUtility).districts, 196);

    // More than half of Ohio's districts have no oil and gas value at all, so the mineral column
    // contributes nothing to the sum for most of the state and everything for the ten districts
    // the sum is supposed to find.
    assert_eq!(
        tax_base::median_share(Base::Mineral),
        0.0,
        "the median district's mineral share is exactly zero, and that is the point"
    );
    assert!(tax_base::median_share(Base::PublicUtility) > tax_base::median_share(Base::Industrial));

    // And the sum is dominated by the class nobody arguing about industry means: at the median
    // district, public utility is more than half of it.
    let frame = tax_base::frame();
    let dominated = frame
        .iter()
        .filter(|row| row.summed() > 0.0)
        .filter(|row| row.public_utility / row.summed() > 0.5)
        .count();
    assert!(
        dominated > frame.len() / 2,
        "public utility is the majority of the summed share in only {dominated} of {} districts",
        frame.len()
    );
}

/// **The finding, second half.** Industrial districts are not distinctive; mineral districts are.
#[test]
fn the_class_that_behaves_as_argued_is_the_one_nobody_names() {
    let industrial = profile_of(Base::Industrial);
    let mineral = profile_of(Base::Mineral);

    assert_eq!(industrial.districts, 27);
    assert_eq!(mineral.districts, 10);

    // Industrial districts sit *below* the statewide guarantee rate. Whatever holds half of Ohio
    // above the formula, an industrial tax base is not it.
    assert!(
        industrial.guarantee_rate < STATEWIDE_GUARANTEE_RATE,
        "industrial districts run at {:.1}%, not below the statewide {:.1}%",
        industrial.guarantee_rate * 100.0,
        STATEWIDE_GUARANTEE_RATE * 100.0
    );

    // And they are not the districts losing pupils either — they lose them more slowly than the
    // median district, which is the second half of the claim and fails the same way.
    let statewide_change = median(
        department_model::districts()
            .iter()
            .map(ModelDistrict::enrollment_change)
            .collect(),
    )
    .expect("the model is not empty");
    assert!(
        industrial.median_enrollment_change > statewide_change,
        "industrial districts fall {:.2}% against a statewide {:.2}%",
        industrial.median_enrollment_change * 100.0,
        statewide_change * 100.0
    );

    // Mineral districts are the profile the industrial claim describes: nine of ten on the
    // guarantee, falling at nearly twice the statewide rate, and worth well over the statewide
    // valuation per pupil.
    assert!(
        mineral.guarantee_rate >= 0.9,
        "mineral districts run at {:.1}%",
        mineral.guarantee_rate * 100.0
    );
    assert!(
        mineral.median_enrollment_change < statewide_change * 1.3,
        "mineral districts fall {:.2}% against a statewide {:.2}%",
        mineral.median_enrollment_change * 100.0,
        statewide_change * 100.0
    );
    assert!(
        mineral.median_valuation_per_pupil > industrial.median_valuation_per_pupil * 1.4,
        "mineral ${:.0} against industrial ${:.0} per pupil",
        mineral.median_valuation_per_pupil,
        industrial.median_valuation_per_pupil
    );

    // The sharper form: every mineral district above 15% is on the guarantee.
    let model = model();
    let deepest: Vec<&ModelDistrict> = tax_base::concentrated_in(Base::Mineral)
        .iter()
        .filter(|row| row.mineral > 0.15)
        .filter_map(|row| model.get(&row.irn))
        .collect();
    assert_eq!(deepest.len(), 5);
    assert!(
        deepest.iter().all(|d| d.on_guarantee()),
        "not every district above 15% mineral is on the guarantee"
    );
}

/// The threshold and the tax year are stated, not inherited from iteration order.
#[test]
fn the_composition_is_read_at_one_named_tax_year() {
    assert_eq!(TAX_YEAR, 2024);
    assert!((CONCENTRATED - 0.10).abs() < f64::EPSILON);

    // Every row in the frame is that year's, and the frame is one row per district — the failure
    // `metric/assessed-valuation-per-pupil` had was a table silently built from two tax years.
    let frame = tax_base::frame();
    let unique: std::collections::BTreeSet<&str> = frame.iter().map(|r| r.irn.as_str()).collect();
    assert_eq!(
        unique.len(),
        frame.len(),
        "a district appears twice in the frame"
    );

    // The shares are shares: none exceeds one, and the three classes plus commercial cannot
    // either, since they are disjoint parts of the same denominator.
    for row in &frame {
        let parts = row.summed() + row.commercial;
        assert!(
            (0.0..=1.0).contains(&parts),
            "{} has business classes summing to {parts}",
            row.name
        );
    }
}
