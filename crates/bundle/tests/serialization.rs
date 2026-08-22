//! The feed's serialization, exercised through the public API.
//!
//! These lived inside `lib.rs` as a 1,116-line `mod tests`, which put 357 lines of type
//! definitions on the far side of them from the code that serializes those types. They
//! reference nothing private — the one test that did, on `num`, stayed behind — so moving
//! them out needed no visibility change.

use bundle::*;

fn sample() -> District {
    District {
        irn: "049056".into(),
        name: "Northern Local".into(),
        county: "Perry".into(),
        national: Some(NationalPosition {
            local_share: 0.4123,
            local_share_percentile: 0.6104,
            revenue_per_pupil: 16_402.0,
            revenue_per_pupil_percentile: 0.5512,
            spending_per_pupil: 15_118.0,
            spending_per_pupil_percentile: 0.5308,
        }),
        transition: Transition {
            funding_base: 12_400_000.0,
            open_enrollment_prior: 214.5,
            open_enrollment_current: 96.4,
            open_enrollment_threshold: 21.45,
            open_enrollment_adjustment: 800_412.0,
            fy21_funding_base: 13_100_000.0,
            transition_supplement: 41_900.0,
        },
        preschool_special_education: PreschoolSpecialEducation {
            adm: [6.4, 15.2, 1.0, 0.0, 0.0, 1.0],
            aid: [31_800.0, 78_100.0, 5_900.0, 0.0, 0.0, 8_400.0],
            total: 124_200.0,
            flat_component: 90_900.0,
            unprorated: 128_236.0,
        },
        transportation: Transportation {
            public_riders: 812.0,
            nonpublic_riders: 41.0,
            community_riders: 18.0,
            weighted_riders: 921.0,
            per_rider_base: 1_231_538.0,
            per_mile_base: 1_402_119.0,
            paid_on_miles: true,
            effective_state_share: 0.5,
            school_bus: 701_059.5,
            mass_transit: 0.0,
            other: 4_812.0,
            efficiency: 62_190.0,
            density: 91_411.0,
            efficiency_index: 1.2044,
            district_density: 11.7,
            fy21_base: 812_004.0,
            guarantee: 0.0,
            total: 859_472.5,
            special_education: 118_204.0,
            special_education_unprorated: 128_838.0,
        },
        supplements: Supplements {
            stars: Some(4.0),
            progress: Some(3.0),
            performance_eligible: true,
            performance: 70_236.0,
            base_funding: 84_312.0,
            enrollment_change: -0.0412,
            growth_eligible: false,
            growth: 0.0,
            growth_forgone: Some(527_000.0),
        },
        // Two House districts, unevenly split, so the serializer's array separator is
        // exercised and a district that straddles a boundary is the case under test.
        house_districts: vec![
            HouseDistrictShare {
                number: "094".into(),
                share: 0.7312,
            },
            HouseDistrictShare {
                number: "072".into(),
                share: 0.2688,
            },
        ],
        adm: 2_193.81,
        current_year_adm: 2_107.80,
        base_cost_per_pupil: 8_100.0,
        aggregate_base_cost: 17_769_861.0,
        // The serializer writes every element; the values do not matter to what this asserts,
        // which is that the shape reaches the JSON.
        base_cost_build_up: BaseCostBuildUp {
            published_aggregate: 17_769_861.0,
            computed_aggregate: 17_769_860.5,
            residual: -0.5,
            ..BaseCostBuildUp::default()
        },
        // Two tax years, because the serializer writes an array and a single-element one
        // would not exercise the separator between them.
        property_tax: vec![
            PropertyTaxYear {
                tax_year: 2023,
                class1_rate: 20.0,
                ..PropertyTaxYear::default()
            },
            PropertyTaxYear {
                tax_year: 2024,
                class1_rate: 20.0154,
                ..PropertyTaxYear::default()
            },
        ],
        spending_by_function: Some(SpendingByFunction {
            operating_per_pupil: 14_027.17,
            ..SpendingByFunction::default()
        }),
        base_cost_state_share: 6_000_000.0,
        categorical_funding: 8_038_562.0,
        // Special education, English learners and career-technical of the above — the part a
        // base cost lever moves along with base cost.
        base_cost_denominated_categoricals: 2_370_119.0,
        special_education: SpecialEducation {
            adm: [10.9, 105.2, 6.0, 1.0, 10.8, 7.1],
            aid: [21_000.0, 320_000.0, 44_000.0, 9_800.0, 143_000.0, 138_000.0],
        },
        // Each of the five decompositions, with distinguishable values in every slot: the
        // serializer writes arrays and nested objects, and a fixture of zeroes would let a
        // transposed pair through.
        dpia: Dpia {
            economically_disadvantaged_adm: 1_050.25,
            directly_certified_adm: 640.5,
            weighted_adm: 906.84,
            percentage: 0.4302,
            index: 0.6504,
        },
        targeted_assistance: TargetedAssistance {
            property_valuation: 210_000_000.0,
            federal_gross_income: 190_000_000.0,
            weighted_wealth: 202_000_000.0,
            capacity_index: 1.9413,
            capacity_amount: 1_520_000.0,
            wealth_per_pupil: 95_800.25,
            wealth_index: 2.8884,
            wealth_amount: 1_580_000.0,
            resident_adm: 2_108.5,
            supplement_eligible: true,
        },
        career_technical: CareerTechnical {
            fte: [40.5, 22.25, 8.0, 4.5, 1.25],
            aid: [180_000.0, 78_000.0, 10_000.0, 4_800.0, 1_100.0],
            associated_services: 26_100.0,
        },
        english_learners: EnglishLearners {
            adm: [6.5, 3.25, 1.5],
            aid: [7_800.0, 2_900.0, 1_300.0],
        },
        gifted: Gifted {
            identification: 24_500.0,
            referral: 4_200.0,
            fte_k8: 61.0,
            fte_9_12: 28.5,
            coordinator_units: 0.6387,
            coordinator_aid: 41_100.0,
            specialist_k8_units: 0.4357,
            specialist_k8_aid: 29_200.0,
            specialist_9_12_units: 0.3,
            specialist_9_12_aid: 18_200.0,
            entirely_on_the_floor: false,
        },
        categorical_adm: 2_107.80,
        categoricals: Categoricals {
            targeted_assistance: 3_100_000.0,
            special_education: 2_100_000.0,
            dpia: 2_300_000.0,
            english_learners: 12_000.0,
            gifted: 226_562.0,
            career_technical: 300_000.0,
        },
        formula_aid_per_pupil: 6_400.0,
        realized_aid_per_pupil: 6_400.0,
        guarantee: 0.0,
        at_minimum_state_share: false,
        valuation_per_pupil: Some(279_983.24),
        effective_class1_millage: Some(20.0),
        voted_operating_millage: Some(34.9),
        // Northern Local is one of the 75 districts that crossed 20.0000 between the two tax
        // years, which makes it the right fixture: it is at the floor on the profile's TY2023
        // figure and a hundredth of a mill above it on SD-1's TY2024 one.
        millage: Some(MillageAnalysis {
            tax_year: 2024,
            prior_rate: 20.0,
            observed_rate: 20.0154,
            predicted_rate: 20.0,
            residual: 0.0154,
            at_floor: true,
            cumulative_reduction: Some(0.4269),
            yield_per_mill_per_pupil: 227.35,
        }),
        // 23 mills against $279,983 of valuation is $6,440 — more than half of what the
        // charge-off would have deemed Northern Local able to raise toward its own cost.
        regime: Some(RegimeCounterfactual {
            charge_off_mills: 23.0,
            charge_off_local_share: Some(6_439.61),
            local_capacity: Some(5_263.44),
            aid_charge_off: Some(1_660.39),
            aid_fsfp: Some(2_836.56),
            difference: Some(1_176.17),
            residual: Some(0.0),
            exceeds_base_cost: false,
            mills_short_of_charge_off: Some(2.9846),
            // Perry County reappraised in TY2023, so a third of that revaluation is still
            // deferred and the charge-off reaches 92.0% of the district's taxable value —
            // $517 per pupil it is therefore not asked for. The real figures for the real
            // district, so this fixture cannot drift into describing a place that does not
            // exist.
            recognized_share: 0.91965761,
            reappraisal_year: 2023,
            overstated_by: Some(517.374),
        }),
        operating_expenditure_per_pupil: Some(11_986.62),
        economically_disadvantaged: Some(0.3881),
        enrollment_change: Some(-0.03),
        adm_history: [2_173.0, 2_140.0, 2_107.8],
        finances: vec![FinanceYear {
            fiscal_year: 2025,
            state_aid: 10_252_524.0,
            local_tax: 6_000_000.0,
            total_revenue: 21_000_000.0,
            total_expenditure: 22_000_000.0,
            ending_cash: 7_500_000.0,
        }],
        // Two years and a county span, so the emitter's array and its nullable scalar are both
        // exercised by the fixture rather than only by the real feed.
        casino: vec![
            CasinoYear {
                fiscal_year: 2023,
                amount: 210_446.19,
            },
            CasinoYear {
                fiscal_year: 2024,
                amount: 214_003.55,
            },
        ],
        casino_counties: Some(3),
        outcome: Some(DistrictOutcome {
            performance_index: Some(89.9),
            performance_index_prior: Some(89.1),
            performance_index_earliest: Some(88.4),
            progress_effect_size: Some(0.0),
            per_enrolled_pupil: Some(14_512.0),
            progress_effect_size_one_year: Some(0.31),
            per_equivalent_pupil: Some(11_986.62),
            // 4.2% federal, the statewide median, and the two parts add to the whole.
            per_equivalent_pupil_federal: Some(503.44),
            per_equivalent_pupil_state_local: Some(11_483.18),
            economically_disadvantaged: Some(38.8),
            english_learner: Some(0.4),
            students_with_disabilities: Some(15.2),
        }),
    }
}

fn zero_statewide() -> Statewide {
    Statewide {
        districts: 1,
        on_guarantee: 0,
        at_millage_floor: 1,
        near_millage_floor: 0,
        median_voted_millage: 0.0,
        median_effective_millage: 0.0,
        median_millage_reduction: 0.0,
        median_yield_per_mill: 0.0,
        min_yield_per_mill: 0.0,
        max_yield_per_mill: 0.0,
        median_sd1_value_per_pupil: 0.0,
        districts_without_targeted_assistance: 135,
        below_charge_off_rate: 0,
        charge_off_exceeds_base_cost: 0,
        median_regime_difference: 0.0,
        at_minimum_state_share: 0,
        median_valuation_per_pupil: 0.0,
        median_operating_expenditure_per_pupil: 0.0,
        wealth_neutrality_formula: 0.0,
        wealth_neutrality_realized: 0.0,
        guarantee_total: 0.0,
        realized_aid_total: 0.0,
        minimum_state_share: 0.1,
        finances: vec![FinanceYear {
            fiscal_year: 2025,
            state_aid: 7_890_000_000.0,
            local_tax: 11_000_000_000.0,
            total_revenue: 25_090_000_000.0,
            total_expenditure: 27_600_000_000.0,
            ending_cash: 9_140_000_000.0,
        }],
        outcomes: Some(OutcomeStatewide {
            districts: 606,
            poverty_vs_performance: -0.846,
            guarantee_vs_performance: 0.187,
            guarantee_vs_performance_controlled: 0.035,
            spending_vs_growth_controlled: 0.146,
            weighted_spending_vs_performance: -0.015,
            enrolled_spending_vs_performance: -0.337,
            median_performance_on_guarantee: 89.9,
            median_performance_on_formula: 85.6,
            median_federal_share: 0.042,
            max_federal_share: 0.29,
            federal_share_above_tenth: 47,
            federal_share_vs_performance: -0.11,
            federal_share_vs_performance_raw: -0.58,
            growth_measures_disagree: 44,
            growth_measures_determinate: 534,
            growth_measures_disagree_materially: 0,
            growth_measure_agreement: 0.904,
        }),
    }
}

fn bundle(districts: Vec<District>, checkpoints: Vec<Checkpoint>) -> Bundle {
    Bundle {
        // One draft, with one provision of each kind, so the emitter's empty-`lever` branch is
        // exercised by the fixture rather than only by the real feed — an unpriced provision
        // that vanished in serialization is the defect this block exists to catch.
        drafts: vec![Draft {
            slug: "a-draft".into(),
            provisions: vec![
                DraftProvision {
                    ordinal: 1,
                    title: "A clause a lever reaches".into(),
                    authority: "R.C. 3317.011".into(),
                    parameter: "base-cost-per-pupil".into(),
                    lever: "base-cost".into(),
                    proposed: "1.05".into(),
                    note: "Sized to nothing; this is a fixture.".into(),
                },
                DraftProvision {
                    ordinal: 2,
                    title: "A clause no lever reaches".into(),
                    authority: "R.C. 3317.014".into(),
                    parameter: String::new(),
                    lever: String::new(),
                    proposed: "each weight times 1.08".into(),
                    note: "No lever expresses a categorical weight.".into(),
                },
            ],
        }],
        // Two entries and two reckonings, so the emitter's sort and its `kind` discriminant
        // are both exercised by the fixture every other test in this module builds on.
        series_years: vec![
            SeriesYear {
                series: "millage".into(),
                kind: YearKind::Tax,
                label: "2024".into(),
                source: "Table SD-1".into(),
            },
            SeriesYear {
                series: "formula".into(),
                kind: YearKind::Fiscal,
                label: "FY2027".into(),
                source: "DEW FY27 calculator".into(),
            },
        ],
        senate_districts: vec![HouseDistrict {
            number: "031".into(),
            adm: 4_812.3,
            realized_aid: 30_795_000.0,
            base_cost_state_share: 18_300_000.0,
            categorical_funding: 12_495_000.0,
            guarantee: 0.0,
            districts_on_guarantee: 1,
            districts_at_minimum_state_share: 2,
            districts_wholly_inside: 1,
            members: vec![HouseDistrictMember {
                irn: "049056".into(),
                name: "Northern Local".into(),
                share: 1.0,
                share_of_house_district: 1.0,
                adm: 4_812.3,
                realized_aid: 30_795_000.0,
                wholly_inside: true,
            }],
        }],
        house_districts: vec![HouseDistrict {
            number: "094".into(),
            adm: 1_604.1,
            realized_aid: 10_265_000.0,
            base_cost_state_share: 6_100_000.0,
            categorical_funding: 4_165_000.0,
            guarantee: 0.0,
            districts_on_guarantee: 0,
            districts_at_minimum_state_share: 1,
            districts_wholly_inside: 0,
            members: vec![HouseDistrictMember {
                irn: "049056".into(),
                name: "Northern Local".into(),
                share: 0.7312,
                share_of_house_district: 1.0,
                adm: 1_604.1,
                realized_aid: 10_265_000.0,
                wholly_inside: false,
            }],
        }],
        contract_version: CONTRACT_VERSION.into(),
        provenance: "test".into(),
        fiscal_year: 2027,
        statewide: zero_statewide(),
        checkpoints,
        projection: None,
        deflator: None,
        national: None,
        history: Vec::new(),
        // Two years across the source change, so anything serializing this fixture carries
        // both labels rather than one.
        // One dated line and one undated, so a serializer that omitted `null` rather than
        // writing it would fail here.
        appropriation_lines: vec![
            AppropriationLine {
                fund: "GRF".into(),
                ali: "200502".into(),
                name: "Pupil Transportation".into(),
                established_by: "H.B. 191 of the 112th G.A.".into(),
                general_assembly: Some(112),
                convened: Some(1977),
                discontinued: false,
            },
            AppropriationLine {
                fund: "GRF".into(),
                ali: "200321".into(),
                name: "Operating Expenses".into(),
                established_by: String::new(),
                general_assembly: None,
                convened: None,
                discontinued: false,
            },
        ],
        appropriations: vec![
            AppropriationYear {
                fiscal_year: 2013,
                enacted: 9_322_046_458.0,
                foundation_funding: 6_349_290_686.0,
                items: 115,
                source: "catalog".into(),
            },
            AppropriationYear {
                fiscal_year: 2014,
                enacted: 9_871_965_322.0,
                foundation_funding: 6_547_098_389.0,
                items: 109,
                source: "workbook".into(),
            },
        ],
        // Three rows spanning both breaks: the basis change, so anything serializing this
        // fixture has to carry both names rather than one, and the split into three files,
        // so it has to carry a row whose share is absent rather than zero.
        meal_program: vec![
            MealProgramYear {
                fiscal_year: 2009,
                sponsors: 812,
                enrollment: 1_000_000.0,
                approved: 412_000.0,
                identified: 0.0,
                share: Some(0.412),
                floor: 0.412,
                ceiling: 0.412,
                without_applications: 0.0,
                streams: 1,
                basis: "adm".into(),
            },
            MealProgramYear {
                fiscal_year: 2010,
                sponsors: 844,
                enrollment: 1_000_000.0,
                approved: 437_000.0,
                identified: 0.0,
                share: Some(0.437),
                floor: 0.437,
                ceiling: 0.437,
                without_applications: 0.0,
                streams: 1,
                basis: "ce".into(),
            },
            MealProgramYear {
                fiscal_year: 2014,
                sponsors: 901,
                enrollment: 1_000_000.0,
                approved: 333_000.0,
                identified: 105_000.0,
                share: None,
                floor: 0.438,
                ceiling: 0.484,
                without_applications: 0.166,
                streams: 3,
                basis: "ce".into(),
            },
        ],
        // Three years, one of them the closure, so a consumer reading this fixture meets the
        // shape the real block has: a series that falls by a quarter in the middle for a
        // reason no appropriation records.
        casino: vec![
            CasinoYear {
                fiscal_year: 2020,
                amount: 95_985_938.04,
            },
            CasinoYear {
                fiscal_year: 2021,
                amount: 73_873_804.95,
            },
            CasinoYear {
                fiscal_year: 2022,
                amount: 109_385_274.99,
            },
        ],
        districts,
    }
}

fn projection() -> Projection {
    Projection {
        base_year: 2026,
        horizon: 2036,
        method: "damped".into(),
        damping: 0.85,
        sigma: 0.023_456_7,
        z: 1.0,
        prior_source: "cross-sectional spread of district annual enrolled-ADM growth".into(),
        checkpoints: vec![ForecastCheckpoint {
            label: "current law, FY2032".into(),
            policy: checkpoint().policy,
            fiscal_year: 2032,
            realized_aid: 7_100_000_000.0,
            low: 6_860_000_000.0,
            high: 7_350_000_000.0,
            adm: 1_500_000.0,
            on_guarantee: 320,
        }],
    }
}

fn checkpoint() -> Checkpoint {
    Checkpoint {
        label: "guarantee removed".into(),
        policy: PolicyShape {
            guarantee: "removed",
            guarantee_argument: 0.0,
            base_cost_scale: 1.0,
            minimum_state_share: 0.1,
            phase_in_base_cost: 1.0,
            phase_in_categorical: 1.0,
        },
        cost: -879_000_000.0,
        realized_aid: 6_402_000_000.0,
        gainers: 0,
        losers: 294,
        unmoved: 315,
        held_throughout: 294,
        lifted_off: 0,
        pushed_on: 0,
        on_guarantee: 0,
    }
}

#[test]
fn a_district_with_no_guarantee_is_on_formula() {
    assert!(!sample().on_guarantee());
}

/// A district with no SD-1 block falls back to the profile's effective rate.
fn without_sd1(effective: Option<f64>) -> District {
    District {
        millage: None,
        property_tax: Vec::new(),
        effective_class1_millage: effective,
        ..sample()
    }
}

#[test]
fn exactly_twenty_mills_counts_as_the_floor() {
    assert!(without_sd1(Some(20.0)).at_millage_floor());
    assert!(!without_sd1(Some(37.09)).at_millage_floor());
    assert!(!without_sd1(None).at_millage_floor());
}

/// The bug this contract version exists for. Six districts never voted twenty mills of
/// current operating levy, so reduction factors have nothing to reduce; comparing their rate
/// to a literal `20.0` for equality reported them as being above the floor with the factors
/// operative, which is the reverse of their position.
#[test]
fn a_rate_below_twenty_mills_is_at_the_floor_not_above_it() {
    // Vinton County Local: 18.70 voted, 18.70 effective, reduction factor zero.
    let vinton = without_sd1(Some(18.7));
    assert!(
        vinton.at_millage_floor(),
        "a district charging 18.70 mills cannot be above a twenty-mill floor"
    );
    assert!(
        !vinton.near_millage_floor(),
        "it is at the floor, not near it"
    );
}

/// The floor is the crate's, not a number written here — so a change to the statute is a
/// change in one place.
#[test]
fn the_floor_comes_from_the_millage_crate() {
    let floor = millage::floor_for(edfund_core::AgencyType::City).expect("a school district");
    assert!(without_sd1(Some(floor)).at_millage_floor());
    assert!(!without_sd1(Some(floor + 1.0)).at_millage_floor());
    assert_eq!(
        millage::floor_for(edfund_core::AgencyType::JointVocational),
        Some(2.0),
        "the JVSD floor differs, which is why this is not a literal"
    );
}

/// Where the binary stops carrying information. The fixture is Northern Local, which sits at
/// the floor on the profile's TY2023 rate and 0.0154 mills above it on SD-1's TY2024 one.
#[test]
fn a_hundredth_of_a_mill_above_the_floor_is_counted_as_near_it() {
    let northern = sample();
    assert!(!northern.at_millage_floor());
    assert!(northern.near_millage_floor());

    let clearly_above = District {
        millage: Some(MillageAnalysis {
            observed_rate: 24.71,
            ..northern.millage.expect("the fixture has one")
        }),
        ..sample()
    };
    assert!(!clearly_above.at_millage_floor());
    assert!(!clearly_above.near_millage_floor());
}

/// SD-1 is the later observation and two departments disagree about 75 districts, so the
/// classification has to say which one it is using.
#[test]
fn sd1_outranks_the_profile_where_both_have_a_rate() {
    let conflicting = District {
        effective_class1_millage: Some(20.0),
        millage: Some(MillageAnalysis {
            observed_rate: 25.31,
            ..sample().millage.expect("the fixture has one")
        }),
        ..sample()
    };
    assert!(
        !conflicting.at_millage_floor(),
        "the profile says floor and SD-1 says 25.31 mills; SD-1 is the later observation"
    );
}

#[test]
fn the_fy2020_baseline_is_only_recoverable_on_the_guarantee() {
    assert_eq!(sample().implied_fy2020_baseline_per_pupil(), None);
    let guaranteed = District {
        guarantee: 1_000_000.0,
        realized_aid_per_pupil: 7_100.0,
        ..sample()
    };
    assert_eq!(
        guaranteed.implied_fy2020_baseline_per_pupil(),
        Some(7_100.0)
    );
}

#[test]
fn json_escapes_quotes_and_backslashes_in_district_names() {
    let odd = District {
        name: r#"St. "Mary" \ Local"#.into(),
        ..sample()
    };
    assert!(bundle(vec![odd], vec![])
        .to_json()
        .contains(r#"St. \"Mary\" \\ Local"#));
}

#[test]
fn missing_values_serialize_as_null_not_zero() {
    let sparse = District {
        valuation_per_pupil: None,
        effective_class1_millage: None,
        operating_expenditure_per_pupil: None,
        economically_disadvantaged: None,
        enrollment_change: None,
        ..sample()
    };
    let json = bundle(vec![sparse], vec![]).to_json();
    assert!(json.contains("\"valuation_per_pupil\": null"));
    assert!(
        !json.contains("\"valuation_per_pupil\": 0"),
        "a missing value must not be indistinguishable from zero"
    );
}

/// No key appears twice inside one district object.
///
/// `special_education` and `categoricals` were each emitted twice per district for several
/// contract versions. Nothing was visibly wrong — JSON takes the last occurrence, and both
/// copies were identical — so about 120KB of every 3.4MB feed was a copy of itself and no test
/// noticed. Duplicate keys are also the shape a genuine bug takes when two branches both write
/// a field and only one is right.
#[test]
fn a_district_object_repeats_no_key() {
    let json = bundle(vec![sample()], vec![checkpoint()]).to_json();
    let start = json.find("\"districts\": [").expect("a districts array");
    let district = &json[start..];

    // Scan the first district object only, tracking brace depth so nested objects do not end
    // it early. Keys at depth one are the district's own.
    let mut depth = 0_i32;
    let mut seen: Vec<&str> = Vec::new();
    let bytes = district.as_bytes();
    let mut i = district.find('{').expect("an object");
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            b'"' if depth == 1 => {
                let rest = &district[i + 1..];
                if let Some(end) = rest.find('"') {
                    let key = &rest[..end];
                    // A key is followed by a colon; a string *value* is not.
                    if district[i + 1 + end + 1..].starts_with(':') {
                        assert!(
                            !seen.contains(&key),
                            "the district object emits \"{key}\" twice"
                        );
                        seen.push(key);
                    }
                    i += end + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    assert!(
        seen.len() > 20,
        "expected to have walked a full district, saw {} keys: {seen:?}",
        seen.len()
    );
}

#[test]
fn serialization_is_deterministic() {
    let b = bundle(vec![sample(), sample()], vec![checkpoint()]);
    assert_eq!(b.to_json(), b.to_json());
}

#[test]
fn the_property_tax_years_survive_serialization_in_order() {
    // The page reads the pair as a change, so a reversed or collapsed array would invert every
    // direction it reports rather than failing visibly.
    let json = bundle(vec![sample()], vec![]).to_json();
    let start = json
        .find("\"property_tax\": [")
        .expect("the array is written");
    // Bounded by the array's own close rather than a byte count: one year's block runs to
    // several hundred characters, and a short slice would only ever find the first.
    let end = start + json[start..].find(']').expect("the array closes");
    let block = &json[start..end];
    let first = block.find("\"tax_year\": 2023").expect("the earlier year");
    let second = block.find("\"tax_year\": 2024").expect("the later year");
    assert!(first < second, "tax years are not oldest first: {block}");
}

#[test]
fn a_district_without_a_spending_row_serializes_as_null() {
    // Two of the 609 have none. Writing zeros would be a claim about their spending rather
    // than about the file, and the page needs to be able to tell the difference.
    let mut district = sample();
    district.spending_by_function = None;
    let json = bundle(vec![district], vec![]).to_json();
    assert!(json.contains("\"spending_by_function\": null"), "{json}");
}

#[test]
fn every_meal_program_row_names_the_count_it_divides_by() {
    // The denominator changes inside this series, at FY2010. A row that does not carry its
    // own basis is a row a consumer will plot against the one before it, and the whole reason
    // the block is safe to publish is that it refuses to let that happen silently.
    let json = bundle(vec![], vec![]).to_json();
    // The counts are asserted with the share so the row stays self-checking: 412000/1000000
    // is 0.412, and a serializer that dropped a field or transposed two would fail here
    // rather than ship a share nothing can verify.
    assert!(
        json.contains(
            "\"enrollment\": 1000000, \"approved\": 412000, \"identified\": 0, \
             \"share\": 0.412, \"floor\": 0.412, \"ceiling\": 0.412, \
             \"without_applications\": 0, \"streams\": 1, \"basis\": \"adm\""
        ),
        "{json}"
    );
    assert!(
        json.contains(
            "\"enrollment\": 1000000, \"approved\": 437000, \"identified\": 0, \
             \"share\": 0.437, \"floor\": 0.437, \"ceiling\": 0.437, \
             \"without_applications\": 0, \"streams\": 1, \"basis\": \"ce\""
        ),
        "{json}"
    );
    // And the split year writes a null rather than a number, beside a band that is not
    // degenerate. A serializer that wrote `0` here would publish a poverty rate of nothing.
    assert!(
        json.contains(
            "\"approved\": 333000, \"identified\": 105000, \"share\": null, \
             \"floor\": 0.438, \"ceiling\": 0.484, \"without_applications\": 0.166, \
             \"streams\": 3, \"basis\": \"ce\""
        ),
        "{json}"
    );
}

#[test]
fn the_meal_program_block_carries_no_dollars() {
    // A share is dimensionless and needs no deflator. If a dollar field ever lands here it
    // will need one, and the deflator does not reach FY2001 — so the failure would be a
    // nominal figure silently presented across a span in which prices rose by half.
    let json = bundle(vec![], vec![]).to_json();
    let block = json
        .split("\"meal_program\": [")
        .nth(1)
        .and_then(|rest| rest.split("],").next())
        .unwrap_or_default();
    for money in ["_per_pupil", "dollars", "amount", "total"] {
        assert!(
            !block.contains(money),
            "meal_program grew a `{money}` field; give it a denominator in \
             web/src/lib/denominators.ts and a deflator that reaches FY2001, or drop it"
        );
    }
}

#[test]
fn the_casino_rows_carry_a_year_and_a_dollar_amount_and_nothing_else() {
    /*
     * The guard against the field this block will be asked for. A per-pupil column here would
     * divide by the count R.C. 5753.11 defines — county-resident pupils, community and STEM
     * and joint vocational enrolment included, dual-enrolled pupils counted twice — which is a
     * fifth Ohio pupil count and a partition of nothing. It would sit in the feed beside four
     * other per-pupil figures and be joined to them by the first consumer that tried.
     *
     * Written against the emitted text rather than the struct because the struct is not what a
     * consumer reads, and it is the JSON that would carry the new key.
     */
    let json = bundle(vec![sample()], vec![]).to_json();
    for block in json.split("\"casino\": [").skip(1) {
        let rows = block.split(']').next().unwrap_or_default();
        for key in ["per_pupil", "adm", "share", "students", "population"] {
            assert!(
                !rows.contains(&format!("\"{key}\"")),
                "the casino block grew a `{key}` field; its denominator is not in this feed"
            );
        }
    }
    assert!(json.contains("{\"fiscal_year\": 2024, \"amount\": 214003.55}"));
}

#[test]
fn a_district_outside_the_last_distribution_says_null_rather_than_zero() {
    // Zero counties and "not named in the most recent distribution" are different claims, and
    // the second is the true one. A zero would render as a district paid from no county fund
    // at all, which is not a thing that happens.
    let json = Bundle {
        ..bundle(
            vec![District {
                casino: vec![],
                casino_counties: None,
                ..sample()
            }],
            vec![],
        )
    }
    .to_json();
    assert!(json.contains("\"casino\": []"));
    assert!(json.contains("\"casino_counties\": null"));
}

#[test]
fn the_bundle_declares_its_contract_version() {
    // Against the constant rather than a literal. A hard-coded version here means a bump has
    // to be made in two places, and the one that gets forgotten is the test — which then fails
    // for the right reason at the wrong moment, long after the change that caused it.
    assert!(bundle(vec![], vec![])
        .to_json()
        .contains(&format!("\"contract_version\": \"{CONTRACT_VERSION}\"")));
}

#[test]
fn a_feed_without_a_projection_says_null_rather_than_omitting_the_key() {
    // A consumer must be able to tell "this feed cannot be projected" from "this feed is
    // from a build that predates projection". The first disables a band; the second is a
    // contract mismatch and should have been caught by the version guard.
    assert!(bundle(vec![], vec![])
        .to_json()
        .contains("\"projection\": null"));
}

#[test]
fn the_projection_block_carries_its_method_and_the_prior_the_band_rests_on() {
    let b = Bundle {
        projection: Some(projection()),
        ..bundle(vec![sample()], vec![checkpoint()])
    };
    let json = b.to_json();
    assert!(json.contains("\"method\": \"damped\""));
    assert!(json.contains("\"damping\": 0.85"));
    assert!(json.contains("\"base_year\": 2026"));
    assert!(json.contains("cross-sectional spread"));
}

#[test]
fn sigma_keeps_six_places_because_four_would_move_a_ten_year_band() {
    // `num` rounds to four, which turns 0.0234567 into 0.0235 — a 0.2% shift in the half
    // width at a ten-year horizon, which is enough to fail the checkpoint it exists to pass.
    let json = Bundle {
        projection: Some(projection()),
        ..bundle(vec![], vec![])
    }
    .to_json();
    assert!(json.contains("\"sigma\": 0.023457"), "{json}");
}

#[test]
fn a_forecast_checkpoint_carries_both_ends_of_its_band() {
    // A point with no interval is the thing this whole axis exists to not ship.
    let json = Bundle {
        projection: Some(projection()),
        ..bundle(vec![], vec![])
    }
    .to_json();
    assert!(json.contains("\"realized_aid\": 7100000000"));
    assert!(json.contains("\"low\": 6860000000"));
    assert!(json.contains("\"high\": 7350000000"));
    assert!(json.contains("\"fiscal_year\": 2032"));
}

#[test]
fn every_district_carries_the_three_years_the_projection_is_fitted_from() {
    // Not nullable: a district without a history cannot be projected, and a page that
    // silently dropped it would report a statewide total over a subset of the panel.
    let json = bundle(vec![sample()], vec![]).to_json();
    assert!(
        json.contains("\"adm_history\": [2173, 2140, 2107.8]"),
        "{json}"
    );
}

#[test]
fn a_district_without_a_report_card_serializes_a_null_outcome() {
    // Three districts have none. `null` rather than an object of nulls, so a consumer can
    // tell "no report card" from "a report card with nothing in it".
    let none = District {
        outcome: None,
        ..sample()
    };
    let json = bundle(vec![none], vec![]).to_json();
    assert!(json.contains("\"outcome\": null"));
    assert!(!json.contains("\"performance_index\""));
}

#[test]
fn the_outcome_block_carries_both_spending_denominators() {
    // The corpus's central denominator finding is the gap between them. Shipping one would
    // make it unstateable in the interface meant to explain it.
    let json = bundle(vec![sample()], vec![]).to_json();
    assert!(json.contains("\"per_enrolled_pupil\": 14512"));
    assert!(json.contains("\"per_equivalent_pupil\": 11986.62"));
}

#[test]
fn the_statewide_outcomes_carry_the_raw_and_the_controlled_figure() {
    // A page showing +0.187 without +0.035 beside it would be stating the confound as a
    // finding, which is the specific thing this axis was built to prevent.
    let json = bundle(vec![], vec![]).to_json();
    assert!(json.contains("\"guarantee_vs_performance\": 0.187"));
    assert!(json.contains("\"guarantee_vs_performance_controlled\": 0.035"));
}

#[test]
fn checkpoints_carry_the_policy_that_produced_them() {
    let json = bundle(vec![], vec![checkpoint()]).to_json();
    assert!(json.contains("\"guarantee\": \"removed\""));
    assert!(json.contains("\"cost\": -879000000"));
    assert!(json.contains("\"unmoved\": 315"));
}

#[test]
fn an_empty_checkpoint_list_still_produces_valid_json() {
    assert!(bundle(vec![sample()], vec![])
        .to_json()
        .contains("\"checkpoints\": [\n  ],"));
}

#[test]
fn the_scenario_inputs_are_present_for_every_district() {
    // The web layer cannot re-derive a policy without these four.
    let json = bundle(vec![sample()], vec![]).to_json();
    for field in [
        "aggregate_base_cost",
        "base_cost_state_share",
        "categorical_funding",
        "current_year_adm",
    ] {
        assert!(json.contains(field), "{field} missing from the feed");
    }
}

#[test]
fn the_year_index_is_emitted_sorted_whatever_order_it_was_assembled_in() {
    // Every fixture in this repository has to rebuild byte-identically from a clean checkout,
    // and the caller assembles these in the order the blocks happen to be built. The fixture
    // holds `millage` before `formula` for exactly this reason.
    let json = bundle(vec![sample()], vec![]).to_json();
    let formula = json
        .find("\"series\": \"formula\"")
        .expect("formula is in the index");
    let millage = json
        .find("\"series\": \"millage\"")
        .expect("millage is in the index");
    assert!(formula < millage, "the index is written in key order");
}

#[test]
fn a_year_carries_the_reckoning_it_is_on_and_not_only_its_digits() {
    /*
     * The whole point of the block. A tax year is a calendar year whose revenue reaches the
     * district in the *following* fiscal year, so `2024` on a millage figure and `FY2024` on a
     * spending figure are eleven months apart. A consumer that gets only the digits cannot
     * tell them apart and will happily subtract them.
     */
    let json = bundle(vec![sample()], vec![]).to_json();
    assert!(json.contains("{\"series\": \"millage\", \"kind\": \"tax\", \"label\": \"2024\""));
    assert!(json.contains("{\"series\": \"formula\", \"kind\": \"fiscal\", \"label\": \"FY2027\""));
}

#[test]
fn an_absent_history_is_an_empty_array_rather_than_a_missing_key() {
    // A consumer that reads `history` and finds nothing there should get a series of length
    // zero and render the rest of the page, not a `undefined.map` two components later.
    assert!(bundle(vec![sample()], vec![])
        .to_json()
        .contains("\"history\": [\n  ],"));
}

#[test]
fn every_history_year_carries_both_halves_of_what_the_page_draws() {
    let mut feed = bundle(vec![sample()], vec![]);
    feed.history = vec![HistoryYear {
        fiscal_year: 2009,
        districts: 612,
        local_share: 0.4801,
        state_share: 0.4302,
        federal_share: 0.0897,
        poorest_local_per_pupil: 4_012.0,
        richest_local_per_pupil: 9_988.0,
        gap_per_pupil: 5_976.0,
        state_closes_per_pupil: 2_760.0,
        federal_closes_per_pupil: 570.0,
    }];
    let json = feed.to_json();
    // The mix and the equalization measure are two findings joined on the year, and a page
    // drawing one without the other would report where the money came from while saying
    // nothing about whom it reached.
    for field in [
        "\"fiscal_year\": 2009",
        "\"districts\": 612",
        "\"local_share\": 0.4801",
        "\"gap_per_pupil\": 5976",
        "\"state_closes_per_pupil\": 2760",
        "\"federal_closes_per_pupil\": 570",
    ] {
        assert!(json.contains(field), "{field} missing: {json}");
    }
}

#[test]
fn the_residual_is_what_no_level_of_government_closes() {
    // The number the series exists to show. State aid's *share* of the gap holds steady
    // across the panel while the gap grows, so the part nobody closes grows with it — and a
    // page that showed only the percentage would report that as stability.
    let year = HistoryYear {
        gap_per_pupil: 5_976.0,
        state_closes_per_pupil: 2_760.0,
        federal_closes_per_pupil: 570.0,
        ..HistoryYear::default()
    };
    assert!((year.residual_per_pupil() - 2_646.0).abs() < 1e-9);
}
