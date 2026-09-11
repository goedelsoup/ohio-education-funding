//! `metric/assessed-valuation-per-pupil` measured this divergence, published it, and pinned none
//! of it — and what it published is three tax years mixed into one table.
//!
//! The node's `calculator` field reads "Not yet implemented. [open] Target: a helper within
//! `local-capacity`." The helper now exists, the denominator travels with the number, and this
//! file recomputes the whole finding on one stated year.
//!
//! # The identity is the finding's foundation and it holds
//!
//! The two agencies carry the same numerator: on all 606 districts that join, the profile's
//! valuation per pupil times its enrolled ADM reproduces Table SD-1's total taxable value. So the
//! entire gap between the two published per-pupil figures is the pupil count and nothing else.
//!
//! # What the year costs
//!
//! Everything below is [`valuation::TAX_YEAR`]. The node's published figures are not: its table has
//! one TY2024 row and two TY2023 rows, its "78 of 606" and one of its two medians reproduce only on
//! TY2024, and the numerator identity it rests on holds only on TY2023.

use dispersion::valuation;
use local_capacity::PupilCount;

/// **The foundation.** Same property, same dollar, on every district.
#[test]
fn the_two_agencies_carry_the_same_numerator_on_every_district() {
    let (exact, compared) = valuation::numerators_agree();
    assert_eq!(
        (exact, compared),
        (606, 606),
        "districts whose numerators reproduce to a tenth of a per cent"
    );
    assert_eq!(valuation::TAX_YEAR, 2023, "the year the identity holds on");
}

/// **The divergence**, which is therefore the denominator alone.
#[test]
fn the_gap_is_the_pupil_count_and_it_runs_to_more_than_two_to_one() {
    // (name, IRN, resident per enrolled, Education's figure, Taxation's)
    for (name, irn, ratio, enrolled, resident) in [
        ("Columbus", "043802", 1.6725, 405_349.85, 242_367.70),
        ("Dayton", "043844", 1.9267, 172_344.99, 89_449.95),
        ("Youngstown", "045161", 2.2337, 172_999.40, 77_450.72),
    ] {
        let district = valuation::district(irn).unwrap_or_else(|| panic!("{name} is in the frame"));
        assert!(
            (district.denominator_ratio() - ratio).abs() < 0.0005,
            "{name} is charged for {:.4} resident pupils per pupil it teaches",
            district.denominator_ratio()
        );
        assert!(
            (district.on_enrolled().value - enrolled).abs() < 0.01
                && (district.on_resident().value - resident).abs() < 0.01,
            "{name} reads {:.2} and {:.2}",
            district.on_enrolled().value,
            district.on_resident().value
        );
        // And it runs the same way every time: the district looks wealthier on the count of the
        // children it teaches than on the count of the children who live in it.
        assert!(district.on_enrolled().value > district.on_resident().value);
    }
}

/// Concentrated rather than general, which is what makes it dangerous.
#[test]
fn the_medians_are_six_per_cent_apart_and_the_districts_are_not() {
    let (agreeing, compared) = valuation::agreement();
    assert_eq!(
        (agreeing, compared),
        (63, 606),
        "districts whose two published figures agree within 2%"
    );

    let (enrolled, resident) = valuation::medians();
    assert!(
        (enrolled - 248_096.51).abs() < 0.01 && (resident - 233_319.65).abs() < 0.01,
        "the medians are {enrolled:.2} and {resident:.2}"
    );
    // A tenth of the state agrees and the medians are within a fifteenth, so a page that compares
    // one table's district against the other's median is right for most of Ohio and wrong by a
    // factor of two for the districts a reader is most likely to look up.
    assert!(agreeing * 8 < compared);
    assert!((enrolled / resident - 1.0).abs() < 0.07);
}

/// **The structural fix.** The type will not let the two be divided.
#[test]
fn a_per_pupil_valuation_carries_its_denominator_and_refuses_the_other_one() {
    let youngstown = valuation::district("045161").expect("in the frame");
    let (teaches, resides) = (youngstown.on_enrolled(), youngstown.on_resident());
    assert_eq!(teaches.basis, PupilCount::Enrolled);
    assert_eq!(resides.basis, PupilCount::Resident);

    let crossed = teaches.ratio_to(&resides);
    assert!(
        crossed.is_err(),
        "the cross-basis ratio should not be given"
    );
    let message = crossed.expect_err("an error").to_string();
    assert!(
        message.contains("Department of Education") && message.contains("Department of Taxation"),
        "the error should name both publishers: {message}"
    );

    // On one basis it is an ordinary division.
    let columbus = valuation::district("043802").expect("in the frame");
    let same = columbus
        .on_resident()
        .ratio_to(&resides)
        .expect("one basis");
    assert!(
        (same - 3.1293).abs() < 0.0005,
        "Columbus over Youngstown on Taxation's count is {same:.4}"
    );
}

/// And the helper refuses a denominator that is not there at all.
#[test]
fn a_district_with_no_pupils_has_no_valuation_per_pupil_rather_than_an_infinite_one() {
    assert!(local_capacity::valuation_per_pupil(1.0e9, 0.0, PupilCount::Funded).is_none());
    assert!(local_capacity::valuation_per_pupil(1.0e9, -1.0, PupilCount::Funded).is_none());
    let real = local_capacity::valuation_per_pupil(1.0e9, 1_000.0, PupilCount::Funded)
        .expect("a positive count");
    assert!((real.value - 1_000_000.0).abs() < f64::EPSILON);
    assert_eq!(real.basis, PupilCount::Funded);
}
